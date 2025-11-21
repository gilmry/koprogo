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
use crate::domain::entities::{Expense, JournalEntry, JournalEntryLine};
use chrono::Utc;
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
    /// ```
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
        if vat_amount > 0.01 {
            lines.push(
                JournalEntryLine::new_debit(
                    entry_id,
                    expense.organization_id,
                    "4110".to_string(), // VAT Recoverable account
                    vat_amount,
                    Some(format!(
                        "TVA récupérable {}%",
                        expense.vat_rate.unwrap_or(0.0) * 100.0
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
                "4400".to_string(), // Suppliers account
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
    /// ```
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
        let payment_account = payment_account.unwrap_or_else(|| "5500".to_string());
        let total_amount = expense.amount;
        let entry_id = Uuid::new_v4();

        let mut lines = Vec::new();

        // Line 1: Debit supplier (reduce liability)
        lines.push(
            JournalEntryLine::new_debit(
                entry_id,
                expense.organization_id,
                "4400".to_string(),
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
                    if payment_account == "5500" {
                        "Banque"
                    } else if payment_account == "5700" {
                        "Caisse"
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

    /// Check if expense already has journal entries
    ///
    /// Prevents duplicate journal entries for the same expense.
    pub async fn expense_has_journal_entries(&self, expense_id: Uuid) -> Result<bool, String> {
        let entries = self.journal_entry_repo.find_by_expense(expense_id).await?;
        Ok(!entries.is_empty())
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
        ) -> Result<std::collections::HashMap<String, f64>, String> {
            unimplemented!()
        }
        async fn calculate_account_balances_for_period(
            &self,
            _organization_id: Uuid,
            _start_date: chrono::DateTime<chrono::Utc>,
            _end_date: chrono::DateTime<chrono::Utc>,
        ) -> Result<std::collections::HashMap<String, f64>, String> {
            unimplemented!()
        }
        async fn calculate_account_balances_for_building(
            &self,
            _organization_id: Uuid,
            _building_id: Uuid,
        ) -> Result<std::collections::HashMap<String, f64>, String> {
            unimplemented!()
        }
        async fn calculate_account_balances_for_building_and_period(
            &self,
            _organization_id: Uuid,
            _building_id: Uuid,
            _start_date: chrono::DateTime<chrono::Utc>,
            _end_date: chrono::DateTime<chrono::Utc>,
        ) -> Result<std::collections::HashMap<String, f64>, String> {
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
            amount: 1210.0,                // Total TTC
            amount_excl_vat: Some(1000.0), // HT
            vat_rate: Some(0.21),
            vat_amount: Some(210.0),
            amount_incl_vat: Some(1210.0),
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
        assert_eq!(entry.total_debits(), 1210.0);
        assert_eq!(entry.total_credits(), 1210.0);

        // Verify line details
        let expense_line = entry
            .lines
            .iter()
            .find(|l| l.account_code == "6100")
            .unwrap();
        assert_eq!(expense_line.debit, 1000.0);

        let vat_line = entry
            .lines
            .iter()
            .find(|l| l.account_code == "4110")
            .unwrap();
        assert_eq!(vat_line.debit, 210.0);

        let supplier_line = entry
            .lines
            .iter()
            .find(|l| l.account_code == "4400")
            .unwrap();
        assert_eq!(supplier_line.credit, 1210.0);
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
            amount: 1210.0,
            amount_excl_vat: Some(1000.0),
            vat_rate: Some(0.21),
            vat_amount: Some(210.0),
            amount_incl_vat: Some(1210.0),
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
        };

        let result = service.generate_payment_entry(&expense, None, None).await;

        assert!(result.is_ok());
        let entry = result.unwrap();

        // Should have 2 lines: supplier debit, bank credit
        assert_eq!(entry.lines.len(), 2);

        // Verify balances
        assert!(entry.is_balanced());
        assert_eq!(entry.total_debits(), 1210.0);
        assert_eq!(entry.total_credits(), 1210.0);

        // Verify line details
        let supplier_line = entry
            .lines
            .iter()
            .find(|l| l.account_code == "4400")
            .unwrap();
        assert_eq!(supplier_line.debit, 1210.0);

        let bank_line = entry
            .lines
            .iter()
            .find(|l| l.account_code == "5500")
            .unwrap();
        assert_eq!(bank_line.credit, 1210.0);
    }
}
