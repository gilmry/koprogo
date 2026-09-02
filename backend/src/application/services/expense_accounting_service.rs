// Application Service: Expense Accounting Service
//
// CREDITS & ATTRIBUTION:
// This implementation is inspired by the Noalyss project (https://gitlab.com/noalyss/noalyss)
// Noalyss is a free accounting software for Belgian and French accounting
// License: GPL-2.0-or-later (GNU General Public License version 2 or later)
// Copyright: (C) 1989, 1991 Free Software Foundation, Inc.
// Copyright: Dany De Bontridder <dany@alchimerys.eu>
//
// Auto-generates double-entry journal entries from expense transactions

use crate::application::ports::JournalEntryRepository;
use crate::domain::entities::{Expense, JournalEntry, JournalEntryLine, OwnerContribution};
use chrono::Utc;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::sync::Arc;
use uuid::Uuid;

/// Service for automatically generating journal entries from expenses
///
/// This service implements Belgian accounting logic based on PCMN (AR 12/07/2012):
/// - Expense creates debit to expense account (class 6)
/// - VAT creates debit to VAT recoverable account (4110)
/// - Total amount creates credit to supplier account (4400)
///
/// Inspired by Noalyss' automatic journal entry generation
pub struct ExpenseAccountingService {
    journal_entry_repo: Arc<dyn JournalEntryRepository>,
}

// ── Comptes du PCMN utilisés par la génération automatique ──────────────────
//
// Ces codes DOIVENT exister dans le plan comptable provisionné par
// `AccountUseCases::seed_belgian_pcmn`. `journal_entry_lines` porte une clé
// étrangère `(organization_id, account_code) → accounts` : un code absent fait
// échouer l'insertion.
//
// Ils étaient auparavant écrits en dur dans le corps des méthodes, sur QUATRE
// chiffres — `4110`, `4400`, `5500` — alors que le plan en compte trois :
// `411`, `440`, `550`. Aucune écriture automatique n'a donc jamais pu être
// enregistrée contre une organisation correctement provisionnée. Et personne
// ne l'a vu : les deux appelants (`approve_invoice` et `mark_as_paid`)
// journalisent l'échec en `warn!` et poursuivent, pour ne pas faire échouer
// une approbation à cause d'une écriture comptable.
//
// C'est ce que décrit le constat F7 du rapport du 2026-09-01 — non pas « la
// saisie est 100 % manuelle », mais « l'automatisation existe et échoue en
// silence ».
//
// `test_les_comptes_utilises_existent_dans_le_plan` verrouille la
// correspondance.

/// TVA récupérable (classe 4).
const COMPTE_TVA_RECUPERABLE: &str = "411";
/// Fournisseurs (classe 4) — dette envers le prestataire.
const COMPTE_FOURNISSEURS: &str = "440";
/// Compte courant bancaire (classe 5).
const COMPTE_BANQUE: &str = "550";
/// Copropriétaires — appels de fonds (classe 4) : la créance sur le
/// copropriétaire, soldée quand sa quote-part est encaissée.
const COMPTE_COPROPRIETAIRES: &str = "400";

impl ExpenseAccountingService {
    pub fn new(journal_entry_repo: Arc<dyn JournalEntryRepository>) -> Self {
        Self { journal_entry_repo }
    }

    /// Generate journal entry for an expense
    ///
    /// # Belgian Accounting Logic (PCMN)
    ///
    /// Example: 1,000€ HT + 210€ VAT (21%) = 1,210€ TTC
    ///
    /// ```text
    /// Debit:  6100 (Expense account)     1,000.00€
    /// Debit:  4110 (VAT Recoverable)       210.00€
    /// Credit: 4400 (Suppliers)           1,210.00€
    /// ```
    ///
    /// # Arguments
    /// - `expense`: The expense to generate journal entry for
    /// - `created_by`: User who created the expense
    ///
    /// # Returns
    /// - `Ok(JournalEntry)` if generation successful
    /// - `Err(String)` if validation fails or expense has no account_code
    pub async fn generate_journal_entry_for_expense(
        &self,
        expense: &Expense,
        created_by: Option<Uuid>,
    ) -> Result<JournalEntry, String> {
        // Idempotence : une seule écriture d'achat par dépense.
        //
        // `approve_invoice` refuse déjà d'approuver deux fois, mais la garde
        // vaut aussi pour tout autre appelant — le seed, une reprise de
        // données, un futur endpoint de régénération.
        if self.has_entry_in_journal(expense.id, "ACH").await? {
            return Err(format!(
                "Journal entry already exists for expense {} (ACH)",
                expense.id
            ));
        }

        // Validate expense has account code
        let account_code = expense
            .account_code
            .as_ref()
            .ok_or("Expense must have an account_code to generate journal entry")?;

        // Calculate amounts
        let amount_excl_vat = expense.amount_excl_vat.unwrap_or(expense.amount);
        let vat_amount = expense.amount - amount_excl_vat;
        let total_amount = expense.amount;

        // Create journal entry lines
        let mut lines = Vec::new();
        let entry_id = Uuid::new_v4();

        // Line 1: Debit expense account (class 6)
        lines.push(
            JournalEntryLine::new_debit(
                entry_id,
                expense.organization_id,
                account_code.clone(),
                amount_excl_vat,
                Some(format!("Dépense: {}", expense.description)),
            )
            .map_err(|e| format!("Failed to create expense debit line: {}", e))?,
        );

        // Line 2: Debit VAT recoverable (4110) if VAT > 0
        if vat_amount > dec!(0.01) {
            lines.push(
                JournalEntryLine::new_debit(
                    entry_id,
                    expense.organization_id,
                    COMPTE_TVA_RECUPERABLE.to_string(),
                    vat_amount,
                    // `vat_rate` est DÉJÀ un pourcentage (21.0 pour 21 %,
                    // cf. `Expense::vat_rate` et sa validation `0..=100`). Le
                    // multiplier par 100 écrivait « TVA récupérable 2100% »
                    // sur chaque pièce comptable générée.
                    Some(format!(
                        "TVA récupérable {} %",
                        expense.vat_rate.unwrap_or(Decimal::ZERO)
                    )),
                )
                .map_err(|e| format!("Failed to create VAT debit line: {}", e))?,
            );
        }

        // Line 3: Credit supplier account (4400)
        lines.push(
            JournalEntryLine::new_credit(
                entry_id,
                expense.organization_id,
                COMPTE_FOURNISSEURS.to_string(),
                total_amount,
                expense
                    .supplier
                    .as_ref()
                    .map(|s| format!("Fournisseur: {}", s)),
            )
            .map_err(|e| format!("Failed to create supplier credit line: {}", e))?,
        );

        // Create journal entry
        let journal_entry = JournalEntry::new(
            expense.organization_id,
            Some(expense.building_id), // building_id
            expense.expense_date,
            Some(format!("{} - {:?}", expense.description, expense.category)),
            expense.invoice_number.clone(), // Use invoice number as document ref
            Some("ACH".to_string()),        // journal_type: ACH (Purchases/Achats)
            Some(expense.id),
            None, // contribution_id
            lines,
            created_by,
        )
        .map_err(|e| format!("Failed to create journal entry: {}", e))?;

        // Persist to database
        self.journal_entry_repo
            .create(&journal_entry)
            .await
            .map_err(|e| format!("Failed to persist journal entry: {}", e))
    }

    /// Generate journal entry for expense payment
    ///
    /// When an expense is paid, we record the payment:
    ///
    /// ```text
    /// Debit:  4400 (Suppliers)           1,210.00€
    /// Credit: 5500 (Bank)                1,210.00€
    /// ```
    ///
    /// # Arguments
    /// - `expense`: The expense being paid
    /// - `payment_account`: Account used for payment (default: 5500 Bank)
    /// - `created_by`: User who recorded the payment
    pub async fn generate_payment_entry(
        &self,
        expense: &Expense,
        payment_account: Option<String>,
        created_by: Option<Uuid>,
    ) -> Result<JournalEntry, String> {
        // Idempotence : une seule écriture de règlement par dépense.
        //
        // `unpay_expense` ramène une dépense de `Paid` à `Pending` SANS
        // contre-passer l'écriture déjà passée. Un cycle payer → dépayer →
        // repayer créditait donc la banque deux fois pour un seul règlement,
        // et le compte fournisseur ne se soldait jamais.
        //
        // Contre-passer serait le traitement comptable complet, mais c'est une
        // décision de méthode : une écriture passée ne se supprime pas, elle
        // s'annule par une écriture inverse datée. En attendant cet arbitrage,
        // refuser le doublon vaut mieux que fausser les comptes.
        if self.has_entry_in_journal(expense.id, "FIN").await? {
            return Err(format!(
                "Payment journal entry already exists for expense {} (FIN)",
                expense.id
            ));
        }

        let payment_account = payment_account.unwrap_or_else(|| COMPTE_BANQUE.to_string());
        let total_amount = expense.amount;
        let entry_id = Uuid::new_v4();

        let mut lines = Vec::new();

        // Line 1: Debit supplier (reduce liability)
        lines.push(
            JournalEntryLine::new_debit(
                entry_id,
                expense.organization_id,
                COMPTE_FOURNISSEURS.to_string(),
                total_amount,
                Some(format!("Paiement: {}", expense.description)),
            )
            .map_err(|e| format!("Failed to create supplier debit line: {}", e))?,
        );

        // Line 2: Credit bank/cash (reduce asset)
        lines.push(
            JournalEntryLine::new_credit(
                entry_id,
                expense.organization_id,
                payment_account.clone(),
                total_amount,
                Some(format!(
                    "Paiement via {}",
                    if payment_account == COMPTE_BANQUE {
                        "Banque"
                    } else {
                        "Autre"
                    }
                )),
            )
            .map_err(|e| format!("Failed to create payment credit line: {}", e))?,
        );

        // Create journal entry
        let journal_entry = JournalEntry::new(
            expense.organization_id,
            Some(expense.building_id), // building_id
            expense.paid_date.unwrap_or_else(Utc::now),
            Some(format!("Paiement: {}", expense.description)),
            expense.invoice_number.clone(),
            Some("FIN".to_string()), // journal_type: FIN (Financial/Financier)
            Some(expense.id),
            None, // contribution_id
            lines,
            created_by,
        )
        .map_err(|e| format!("Failed to create payment journal entry: {}", e))?;

        // Persist to database
        self.journal_entry_repo
            .create(&journal_entry)
            .await
            .map_err(|e| format!("Failed to persist payment journal entry: {}", e))
    }

    /// Génère l'écriture d'ENCAISSEMENT d'une quote-part de copropriétaire.
    ///
    /// # Logique comptable (PCMN belge)
    ///
    /// Exemple : quote-part de 2 000 € reçue par virement.
    ///
    /// ```text
    /// Débit :  550 (Compte courant bancaire)   2 000,00 €
    /// Crédit : 400 (Copropriétaires)           2 000,00 €
    /// ```
    ///
    /// L'argent entre (actif en hausse), la créance sur le copropriétaire
    /// s'éteint (actif en baisse). Journal FIN.
    ///
    /// # Ce que cette écriture n'est PAS
    ///
    /// Elle ne constate pas le PRODUIT. L'appel de fonds est un produit de
    /// classe 7 constaté à son émission, pas à son encaissement ; cette
    /// écriture-ci ne fait que solder une créance déjà comptabilisée. Générer
    /// un produit ici le compterait deux fois.
    ///
    /// # Idempotence
    ///
    /// Une quote-part peut être soldée par deux voies : `mark-paid` depuis
    /// l'interface, ou la réussite d'un paiement portant `contribution_id`.
    /// Sans ce contrôle, un même encaissement débiterait la banque deux fois.
    pub async fn generate_contribution_receipt_entry(
        &self,
        contribution: &OwnerContribution,
        building_id: Option<Uuid>,
        payment_account: Option<String>,
        created_by: Option<Uuid>,
    ) -> Result<JournalEntry, String> {
        let existantes = self
            .journal_entry_repo
            .find_by_contribution(contribution.id)
            .await?;
        if existantes
            .iter()
            .any(|e| e.journal_type.as_deref() == Some("FIN"))
        {
            return Err(format!(
                "Receipt journal entry already exists for contribution {} (FIN)",
                contribution.id
            ));
        }

        let payment_account = payment_account.unwrap_or_else(|| COMPTE_BANQUE.to_string());
        let entry_id = Uuid::new_v4();
        let montant = contribution.amount;

        let lines = vec![
            JournalEntryLine::new_debit(
                entry_id,
                contribution.organization_id,
                payment_account,
                montant,
                Some(format!("Encaissement: {}", contribution.description)),
            )
            .map_err(|e| format!("Failed to create bank debit line: {}", e))?,
            JournalEntryLine::new_credit(
                entry_id,
                contribution.organization_id,
                COMPTE_COPROPRIETAIRES.to_string(),
                montant,
                Some(format!("Quote-part {}", contribution.owner_id)),
            )
            .map_err(|e| format!("Failed to create owner credit line: {}", e))?,
        ];

        let journal_entry = JournalEntry::new(
            contribution.organization_id,
            building_id,
            contribution.payment_date.unwrap_or_else(Utc::now),
            Some(format!("Encaissement: {}", contribution.description)),
            contribution.payment_reference.clone(),
            Some("FIN".to_string()),
            None,
            Some(contribution.id),
            lines,
            created_by,
        )
        .map_err(|e| format!("Failed to create contribution journal entry: {}", e))?;

        self.journal_entry_repo
            .create(&journal_entry)
            .await
            .map_err(|e| format!("Failed to persist contribution journal entry: {}", e))
    }

    /// Check if expense already has journal entries
    ///
    /// Prevents duplicate journal entries for the same expense.
    pub async fn expense_has_journal_entries(&self, expense_id: Uuid) -> Result<bool, String> {
        let entries = self.journal_entry_repo.find_by_expense(expense_id).await?;
        Ok(!entries.is_empty())
    }

    /// Une écriture d'un journal donné existe-t-elle déjà pour cette dépense ?
    ///
    /// Distingue l'achat (ACH, généré à l'approbation) du règlement (FIN,
    /// généré au paiement) : `expense_has_journal_entries` ne le pouvait pas,
    /// et c'est pour cela qu'elle n'a jamais servi malgré son commentaire
    /// « Prevents duplicate journal entries ».
    async fn has_entry_in_journal(
        &self,
        expense_id: Uuid,
        journal_type: &str,
    ) -> Result<bool, String> {
        let entries = self.journal_entry_repo.find_by_expense(expense_id).await?;
        Ok(entries
            .iter()
            .any(|e| e.journal_type.as_deref() == Some(journal_type)))
    }

    /// Get journal entries for an expense
    ///
    /// Returns all entries (expense entry + payment entry if paid).
    pub async fn get_expense_journal_entries(
        &self,
        expense_id: Uuid,
    ) -> Result<Vec<JournalEntry>, String> {
        self.journal_entry_repo.find_by_expense(expense_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::{ApprovalStatus, ExpenseCategory, PaymentStatus};

    // Mock repository for testing
    struct MockJournalEntryRepository {
        entries: std::sync::Mutex<Vec<JournalEntry>>,
    }

    impl MockJournalEntryRepository {
        fn new() -> Self {
            Self {
                entries: std::sync::Mutex::new(Vec::new()),
            }
        }
    }

    #[async_trait::async_trait]
    impl JournalEntryRepository for MockJournalEntryRepository {
        async fn create(&self, entry: &JournalEntry) -> Result<JournalEntry, String> {
            let mut entries = self.entries.lock().unwrap();
            entries.push(entry.clone());
            Ok(entry.clone())
        }

        async fn find_by_expense(&self, expense_id: Uuid) -> Result<Vec<JournalEntry>, String> {
            let entries = self.entries.lock().unwrap();
            Ok(entries
                .iter()
                .filter(|e| e.expense_id == Some(expense_id))
                .cloned()
                .collect())
        }

        async fn find_by_contribution(
            &self,
            contribution_id: Uuid,
        ) -> Result<Vec<JournalEntry>, String> {
            let entries = self.entries.lock().unwrap();
            Ok(entries
                .iter()
                .filter(|e| e.contribution_id == Some(contribution_id))
                .cloned()
                .collect())
        }

        // Other methods not needed for tests
        async fn find_by_id(
            &self,
            _id: Uuid,
            _organization_id: Uuid,
        ) -> Result<JournalEntry, String> {
            unimplemented!()
        }
        async fn find_by_organization(
            &self,
            _organization_id: Uuid,
        ) -> Result<Vec<JournalEntry>, String> {
            unimplemented!()
        }
        async fn find_by_date_range(
            &self,
            _organization_id: Uuid,
            _start_date: chrono::DateTime<chrono::Utc>,
            _end_date: chrono::DateTime<chrono::Utc>,
        ) -> Result<Vec<JournalEntry>, String> {
            unimplemented!()
        }
        async fn calculate_account_balances(
            &self,
            _organization_id: Uuid,
        ) -> Result<std::collections::HashMap<String, Decimal>, String> {
            unimplemented!()
        }
        async fn calculate_account_balances_for_period(
            &self,
            _organization_id: Uuid,
            _start_date: chrono::DateTime<chrono::Utc>,
            _end_date: chrono::DateTime<chrono::Utc>,
        ) -> Result<std::collections::HashMap<String, Decimal>, String> {
            unimplemented!()
        }
        async fn calculate_account_balances_for_building(
            &self,
            _organization_id: Uuid,
            _building_id: Uuid,
        ) -> Result<std::collections::HashMap<String, Decimal>, String> {
            unimplemented!()
        }
        async fn calculate_account_balances_for_building_and_period(
            &self,
            _organization_id: Uuid,
            _building_id: Uuid,
            _start_date: chrono::DateTime<chrono::Utc>,
            _end_date: chrono::DateTime<chrono::Utc>,
        ) -> Result<std::collections::HashMap<String, Decimal>, String> {
            unimplemented!()
        }
        async fn create_manual_entry(
            &self,
            _entry: &JournalEntry,
            _lines: &[JournalEntryLine],
        ) -> Result<(), String> {
            unimplemented!()
        }
        #[allow(clippy::too_many_arguments)]
        async fn list_entries(
            &self,
            _organization_id: Uuid,
            _building_id: Option<Uuid>,
            _journal_type: Option<String>,
            _start_date: Option<chrono::DateTime<chrono::Utc>>,
            _end_date: Option<chrono::DateTime<chrono::Utc>>,
            _limit: i64,
            _offset: i64,
        ) -> Result<Vec<JournalEntry>, String> {
            unimplemented!()
        }
        async fn find_lines_by_account(
            &self,
            _organization_id: Uuid,
            _account_code: &str,
        ) -> Result<Vec<JournalEntryLine>, String> {
            unimplemented!()
        }
        async fn find_lines_by_entry(
            &self,
            _entry_id: Uuid,
            _organization_id: Uuid,
        ) -> Result<Vec<JournalEntryLine>, String> {
            unimplemented!()
        }
        async fn delete_entry(
            &self,
            _entry_id: Uuid,
            _organization_id: Uuid,
        ) -> Result<(), String> {
            unimplemented!()
        }
        async fn validate_balance(&self, _entry_id: Uuid) -> Result<bool, String> {
            unimplemented!()
        }
    }

    #[tokio::test]
    async fn test_generate_journal_entry_for_expense_with_vat() {
        let repo = Arc::new(MockJournalEntryRepository::new());
        let service = ExpenseAccountingService::new(repo.clone());

        let org_id = Uuid::new_v4();
        let expense = Expense {
            id: Uuid::new_v4(),
            organization_id: org_id,
            building_id: Uuid::new_v4(),
            description: "Facture eau".to_string(),
            amount: dec!(1210),                // Total TTC
            amount_excl_vat: Some(dec!(1000)), // HT
            vat_rate: Some(dec!(21)),
            vat_amount: Some(dec!(210)),
            amount_incl_vat: Some(dec!(1210)),
            expense_date: Utc::now(),
            invoice_date: None,
            due_date: None,
            paid_date: None,
            category: ExpenseCategory::Utilities,
            payment_status: PaymentStatus::Pending,
            approval_status: ApprovalStatus::Approved,
            supplier: Some("Vivaqua".to_string()),
            invoice_number: Some("INV-2025-001".to_string()),
            account_code: Some("6100".to_string()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            submitted_at: None,
            approved_at: Some(Utc::now()),
            approved_by: None,
            rejection_reason: None,
            contractor_report_id: None,
        };

        let result = service
            .generate_journal_entry_for_expense(&expense, None)
            .await;

        assert!(result.is_ok());
        let entry = result.unwrap();

        // Should have 3 lines: expense debit, VAT debit, supplier credit
        assert_eq!(entry.lines.len(), 3);

        // Verify balances
        assert!(entry.is_balanced());
        assert_eq!(entry.total_debits(), dec!(1210));
        assert_eq!(entry.total_credits(), dec!(1210));

        // Verify line details
        let expense_line = entry
            .lines
            .iter()
            .find(|l| l.account_code == "6100")
            .unwrap();
        assert_eq!(expense_line.debit, dec!(1000));

        let vat_line = entry
            .lines
            .iter()
            .find(|l| l.account_code == COMPTE_TVA_RECUPERABLE)
            .unwrap();
        assert_eq!(vat_line.debit, dec!(210));

        let supplier_line = entry
            .lines
            .iter()
            .find(|l| l.account_code == COMPTE_FOURNISSEURS)
            .unwrap();
        assert_eq!(supplier_line.credit, dec!(1210));
    }

    #[tokio::test]
    async fn test_generate_payment_entry() {
        let repo = Arc::new(MockJournalEntryRepository::new());
        let service = ExpenseAccountingService::new(repo.clone());

        let org_id = Uuid::new_v4();
        let expense = Expense {
            id: Uuid::new_v4(),
            organization_id: org_id,
            building_id: Uuid::new_v4(),
            description: "Facture eau".to_string(),
            amount: dec!(1210),
            amount_excl_vat: Some(dec!(1000)),
            vat_rate: Some(dec!(21)),
            vat_amount: Some(dec!(210)),
            amount_incl_vat: Some(dec!(1210)),
            expense_date: Utc::now(),
            invoice_date: None,
            due_date: None,
            paid_date: Some(Utc::now()),
            category: ExpenseCategory::Utilities,
            payment_status: PaymentStatus::Paid,
            approval_status: ApprovalStatus::Approved,
            supplier: Some("Vivaqua".to_string()),
            invoice_number: Some("INV-2025-001".to_string()),
            account_code: Some("6100".to_string()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            submitted_at: None,
            approved_at: Some(Utc::now()),
            approved_by: None,
            rejection_reason: None,
            contractor_report_id: None,
        };

        let result = service.generate_payment_entry(&expense, None, None).await;

        assert!(result.is_ok());
        let entry = result.unwrap();

        // Should have 2 lines: supplier debit, bank credit
        assert_eq!(entry.lines.len(), 2);

        // Verify balances
        assert!(entry.is_balanced());
        assert_eq!(entry.total_debits(), dec!(1210));
        assert_eq!(entry.total_credits(), dec!(1210));

        // Verify line details
        let supplier_line = entry
            .lines
            .iter()
            .find(|l| l.account_code == COMPTE_FOURNISSEURS)
            .unwrap();
        assert_eq!(supplier_line.debit, dec!(1210));

        let bank_line = entry
            .lines
            .iter()
            .find(|l| l.account_code == COMPTE_BANQUE)
            .unwrap();
        assert_eq!(bank_line.credit, dec!(1210));
    }

    /// F7 — l'encaissement d'une quote-part produit son écriture.
    #[tokio::test]
    async fn test_encaissement_quote_part_genere_lecriture() {
        let repo = Arc::new(MockJournalEntryRepository::new());
        let service = ExpenseAccountingService::new(repo.clone());

        let org_id = Uuid::new_v4();
        let mut contribution = crate::domain::entities::OwnerContribution::new(
            org_id,
            Uuid::new_v4(),
            None,
            "Charges Q3 2026".to_string(),
            dec!(2000),
            crate::domain::entities::ContributionType::Regular,
            Utc::now(),
            Some("700001".to_string()),
        )
        .unwrap();
        contribution.mark_as_paid(
            Utc::now(),
            crate::domain::entities::ContributionPaymentMethod::BankTransfer,
            Some("VIR-2026-42".to_string()),
        );

        let entry = service
            .generate_contribution_receipt_entry(&contribution, None, None, None)
            .await
            .expect("écriture générée");

        assert!(entry.is_balanced());
        assert_eq!(entry.total_debits(), dec!(2000));
        assert_eq!(entry.journal_type.as_deref(), Some("FIN"));
        assert_eq!(entry.contribution_id, Some(contribution.id));

        // D 550 (banque) : l'argent entre.
        let banque = entry
            .lines
            .iter()
            .find(|l| l.account_code == COMPTE_BANQUE)
            .expect("ligne banque");
        assert_eq!(banque.debit, dec!(2000));

        // C 400 (copropriétaires) : la créance s'éteint.
        let copro = entry
            .lines
            .iter()
            .find(|l| l.account_code == COMPTE_COPROPRIETAIRES)
            .expect("ligne copropriétaires");
        assert_eq!(copro.credit, dec!(2000));

        // Aucune ligne de classe 7 : l'appel de fonds est un produit constaté à
        // son ÉMISSION. En constater un ici le compterait deux fois.
        assert!(
            !entry.lines.iter().any(|l| l.account_code.starts_with('7')),
            "l'encaissement ne constate pas de produit"
        );
    }

    /// @edge — une quote-part peut être soldée par deux voies : la banque ne
    /// doit être débitée qu'une fois.
    ///
    /// `mark-paid` depuis l'interface, ou la réussite d'un paiement portant
    /// `contribution_id` : les deux appellent ce service.
    #[tokio::test]
    async fn test_encaissement_non_duplique() {
        let repo = Arc::new(MockJournalEntryRepository::new());
        let service = ExpenseAccountingService::new(repo.clone());

        let org_id = Uuid::new_v4();
        let mut contribution = crate::domain::entities::OwnerContribution::new(
            org_id,
            Uuid::new_v4(),
            None,
            "Charges Q3 2026".to_string(),
            dec!(2000),
            crate::domain::entities::ContributionType::Regular,
            Utc::now(),
            Some("700001".to_string()),
        )
        .unwrap();
        contribution.mark_as_paid(
            Utc::now(),
            crate::domain::entities::ContributionPaymentMethod::BankTransfer,
            None,
        );

        service
            .generate_contribution_receipt_entry(&contribution, None, None, None)
            .await
            .expect("première écriture");

        let err = service
            .generate_contribution_receipt_entry(&contribution, None, None, None)
            .await
            .expect_err("le doublon doit être refusé");
        assert!(format!("{err}").contains("already exists"), "{err}");
    }

    /// Le test qui aurait évité le défaut F7.
    ///
    /// `journal_entry_lines` porte une clé étrangère
    /// `(organization_id, account_code) → accounts`. Un code absent du plan
    /// comptable fait échouer l'insertion — et les deux appelants
    /// (`approve_invoice`, `mark_as_paid`) journalisent l'échec en `warn!`
    /// avant de poursuivre, pour ne pas faire échouer une approbation à cause
    /// d'une écriture. L'automatisation pouvait donc ne JAMAIS produire une
    /// seule écriture sans que rien ne s'en aperçoive.
    ///
    /// C'est ce qui s'est passé : le service référençait `4110`, `4400`,
    /// `5500` et `5700` là où le plan provisionne `411`, `440` et `550`.
    ///
    /// Les deux tests ci-dessus ne pouvaient pas le voir : ils vérifient la
    /// forme de l'écriture contre un dépôt simulé, sans base ni contrainte.
    /// Une écriture parfaitement équilibrée sur des comptes inexistants leur
    /// paraît correcte.
    #[test]
    fn test_les_comptes_utilises_existent_dans_le_plan() {
        let plan: Vec<&str> =
            crate::application::use_cases::account_use_cases::get_belgian_pcmn_seed_data()
                .into_iter()
                .map(|(code, ..)| code)
                .collect();

        for compte in [
            COMPTE_TVA_RECUPERABLE,
            COMPTE_FOURNISSEURS,
            COMPTE_BANQUE,
            COMPTE_COPROPRIETAIRES,
        ] {
            assert!(
                plan.contains(&compte),
                "le compte {compte} est utilisé par la génération automatique \
                 mais absent du plan provisionné par `seed_belgian_pcmn` : \
                 la clé étrangère de `journal_entry_lines` rejettera l'écriture, \
                 et l'échec sera avalé par le `warn!` de l'appelant"
            );
        }
    }
}
