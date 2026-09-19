// Use Cases: Journal Entry (Manual Accounting Operations)
//
// CREDITS & ATTRIBUTION:
// This implementation is inspired by the Noalyss project (https://gitlab.com/noalyss/noalyss)
// Noalyss is a free accounting software for Belgian and French accounting
// License: GPL-2.0-or-later (GNU General Public License version 2 or later)
// Copyright: (C) 1989, 1991 Free Software Foundation, Inc.
// Copyright: Dany De Bontridder <dany@alchimerys.eu>
//
// Noalyss features that inspired this implementation:
// - Journal types (ACH=Purchases, VEN=Sales, FIN=Financial, ODS=Miscellaneous)
// - Double-entry bookkeeping with debit/credit columns
// - Quick codes for account selection
// - Automatic balance validation
//
// Use cases for manual journal entry creation and retrieval

use crate::application::error::AppError;
use crate::application::ports::journal_entry_repository::JournalEntryRepository;
use crate::domain::entities::journal_entry::{JournalEntry, JournalEntryLine};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::sync::Arc;
use uuid::Uuid;

pub struct JournalEntryUseCases {
    journal_entry_repo: Arc<dyn JournalEntryRepository>,
    /// Résolution de l'ACP dont on tient les comptes.
    ///
    /// Optionnel pour ne pas casser les constructeurs des tests, mais son
    /// absence fait échouer la saisie : une écriture sans ACP est une écriture
    /// dans les livres de personne (ADR-0045).
    building_repository: Option<Arc<dyn crate::application::ports::BuildingRepository>>,
}

impl JournalEntryUseCases {
    pub fn new(journal_entry_repo: Arc<dyn JournalEntryRepository>) -> Self {
        Self {
            journal_entry_repo,
            building_repository: None,
        }
    }

    /// Câble la résolution de l'ACP depuis l'immeuble.
    pub fn with_acp_resolution(
        mut self,
        building_repository: Arc<dyn crate::application::ports::BuildingRepository>,
    ) -> Self {
        self.building_repository = Some(building_repository);
        self
    }

    /// L'ACP dont on tient les comptes, résolue depuis l'immeuble.
    ///
    /// Art. 3.89 § 5, 15° : le syndic tient les comptes *de l'association*.
    /// Une saisie manuelle qui ne désigne pas d'immeuble ne dit pas dans quels
    /// livres elle s'inscrit — on refuse plutôt que d'écrire au hasard.
    ///
    /// Limite connue : une ACP à plusieurs immeubles (Art. 3.84) peut avoir des
    /// écritures qui ne se rattachent à aucun bloc. Elles ne sont pas encore
    /// saisissables par cette voie ; il faudra désigner l'ACP directement.
    ///
    /// #762 : le résultat porte sa catégorie dans le TYPE (`AppError`), pas
    /// dans un motif à chercher dans le message. Une saisie sans immeuble et
    /// un immeuble inexistant sont tous deux des erreurs d'entrée client,
    /// mais de nature différente (`Validation` : rien n'a été désigné ;
    /// `NotFound` : ce qui a été désigné n'existe pas) — la distinction ne
    /// se lit plus a posteriori dans le texte, elle est faite une fois ici.
    /// Seul le câblage manquant (`building_repository` absent, une erreur de
    /// configuration et non de saisie) reste `Internal`.
    async fn resoudre_lacp(&self, building_id: Option<Uuid>) -> Result<Uuid, AppError> {
        let building_id = building_id.ok_or_else(|| {
            AppError::Validation(
                "Impossible de déterminer l'ACP : une écriture manuelle doit désigner un immeuble"
                    .to_string(),
            )
        })?;
        let Some(building_repo) = &self.building_repository else {
            return Err(AppError::Internal(
                "Impossible de déterminer l'ACP : dépôt d'immeubles non câblé".to_string(),
            ));
        };
        let building = building_repo
            .find_by_id(building_id)
            .await
            .map_err(AppError::from)?
            .ok_or_else(|| AppError::NotFound("Immeuble introuvable".to_string()))?;
        Ok(building.acp_id)
    }

    /// Create a manual journal entry with multiple lines
    ///
    /// This follows the Noalyss approach where each journal entry can have multiple lines
    /// with debit and credit columns. The total debits must equal total credits.
    ///
    /// # Arguments
    /// * `organization_id` - Organization ID
    /// * `building_id` - Optional building ID for building-specific entries
    /// * `journal_type` - Type of journal (ACH, VEN, FIN, ODS)
    /// * `entry_date` - Date of the accounting operation
    /// * `description` - Description of the operation
    /// * `reference` - Optional reference number (invoice, receipt, etc.)
    /// * `lines` - Vector of journal entry lines with account_code, debit, credit, description
    #[allow(clippy::too_many_arguments)]
    pub async fn create_manual_entry(
        &self,
        organization_id: Uuid,
        building_id: Option<Uuid>,
        journal_type: Option<String>,
        entry_date: DateTime<Utc>,
        description: Option<String>,
        document_ref: Option<String>,
        lines: Vec<(String, Decimal, Decimal, String)>, // (account_code, debit, credit, line_description)
    ) -> Result<JournalEntry, AppError> {
        // #762 : chaque refus est un `AppError::Validation` — une erreur de
        // saisie typée, pas une String que le gestionnaire HTTP devrait
        // ensuite reclasser en devinant sur son contenu.
        //
        // Validate journal type if provided (inspired by Noalyss journal types)
        if let Some(ref jtype) = journal_type {
            if !["ACH", "VEN", "FIN", "ODS"].contains(&jtype.as_str()) {
                return Err(AppError::Validation(format!(
                    "Invalid journal type: {}. Must be one of: ACH (Purchases), VEN (Sales), FIN (Financial), ODS (Miscellaneous)",
                    jtype
                )));
            }
        }

        // Validate that we have at least 2 lines (double-entry principle)
        if lines.len() < 2 {
            return Err(AppError::Validation(
                "Journal entry must have at least 2 lines (debit and credit)".to_string(),
            ));
        }

        // Calculate totals and validate balance (Noalyss principle)
        let total_debit: Decimal = lines.iter().map(|(_, debit, _, _)| *debit).sum();
        let total_credit: Decimal = lines.iter().map(|(_, _, credit, _)| *credit).sum();

        if (total_debit - total_credit).abs() > dec!(0.01) {
            return Err(AppError::Validation(format!(
                "Journal entry is unbalanced: debits={:.2} credits={:.2}. Debits must equal credits.",
                total_debit, total_credit
            )));
        }

        // Create journal entry ID
        let entry_id = Uuid::new_v4();

        // Create journal entry lines
        let mut journal_lines = Vec::new();
        for (account_code, debit, credit, line_desc) in lines {
            let line = JournalEntryLine {
                id: Uuid::new_v4(),
                journal_entry_id: entry_id,
                organization_id,
                account_code: account_code.clone(),
                debit,
                credit,
                description: Some(line_desc),
                created_at: Utc::now(),
            };
            journal_lines.push(line);
        }

        // L'ACP est résolue APRÈS les validations de forme : une écriture
        // déséquilibrée ou à une seule ligne doit échouer sur son motif, pas
        // sur une résolution d'ACP qu'elle n'aurait jamais dû atteindre.
        let acp_id = self.resoudre_lacp(building_id).await?;

        // Create journal entry
        let journal_entry = JournalEntry {
            id: entry_id,
            acp_id,
            organization_id,
            building_id,
            entry_date,
            description,
            document_ref,
            journal_type,
            expense_id: None,
            contribution_id: None,
            lines: journal_lines.clone(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            created_by: None,
        };

        // Save to repository
        //
        // #762 @security : le dépôt (Postgres) renvoie encore `Result<_,
        // String>` — hors du périmètre borné de cette story, qui porte sur
        // le CLASSEMENT dans le gestionnaire, pas sur la migration complète
        // du port (#555). Mais on ne relit pas ce message pour le classer :
        // `AppError::from(String)` le range en `Internal`, qu'`error_response`
        // masque avant de répondre au client (jamais de nom de contrainte
        // SQL ni de table renvoyé tel quel).
        self.journal_entry_repo
            .create_manual_entry(&journal_entry, &journal_lines)
            .await
            .map_err(AppError::from)?;

        Ok(journal_entry)
    }

    /// List journal entries for an organization
    ///
    /// # Arguments
    /// * `organization_id` - Organization ID
    /// * `building_id` - Optional building ID filter
    /// * `journal_type` - Optional journal type filter
    /// * `start_date` - Optional start date filter
    /// * `end_date` - Optional end date filter
    /// * `limit` - Maximum number of entries to return
    /// * `offset` - Number of entries to skip
    #[allow(clippy::too_many_arguments)]
    pub async fn list_entries(
        &self,
        organization_id: Uuid,
        building_id: Option<Uuid>,
        journal_type: Option<String>,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<JournalEntry>, String> {
        self.journal_entry_repo
            .list_entries(
                organization_id,
                building_id,
                journal_type,
                start_date,
                end_date,
                limit,
                offset,
            )
            .await
    }

    /// Get a single journal entry with its lines
    ///
    /// # Arguments
    /// * `entry_id` - Journal entry ID
    /// * `organization_id` - Organization ID for authorization
    pub async fn get_entry_with_lines(
        &self,
        entry_id: Uuid,
        organization_id: Uuid,
    ) -> Result<(JournalEntry, Vec<JournalEntryLine>), String> {
        let entry = self
            .journal_entry_repo
            .find_by_id(entry_id, organization_id)
            .await?;

        let lines = self
            .journal_entry_repo
            .find_lines_by_entry(entry_id, organization_id)
            .await?;

        Ok((entry, lines))
    }

    /// Delete a manual journal entry
    ///
    /// Only manual entries (not auto-generated from expenses/contributions) can be deleted.
    ///
    /// # Arguments
    /// * `entry_id` - Journal entry ID
    /// * `organization_id` - Organization ID for authorization
    pub async fn delete_manual_entry(
        &self,
        entry_id: Uuid,
        organization_id: Uuid,
    ) -> Result<(), String> {
        // Check if entry exists and is manual
        let entry = self
            .journal_entry_repo
            .find_by_id(entry_id, organization_id)
            .await?;

        if entry.expense_id.is_some() || entry.contribution_id.is_some() {
            return Err(
                "Cannot delete auto-generated journal entries. Only manual entries can be deleted."
                    .to_string(),
            );
        }

        self.journal_entry_repo
            .delete_entry(entry_id, organization_id)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::ports::journal_entry_repository::JournalEntryRepository;
    use crate::domain::entities::journal_entry::{JournalEntry, JournalEntryLine};
    use async_trait::async_trait;
    use std::collections::HashMap;
    use std::sync::Mutex;

    // ========== Mock Repository ==========

    struct MockJournalEntryRepository {
        entries: Mutex<HashMap<Uuid, JournalEntry>>,
        lines: Mutex<HashMap<Uuid, Vec<JournalEntryLine>>>,
        /// #762 @security : simule un dépôt (Postgres) qui échoue à
        /// l'écriture avec un message brut de contrainte SQL — le genre de
        /// message que ce dépôt renvoie réellement aujourd'hui (voir
        /// `journal_entry_repository_impl.rs::create_manual_entry`,
        /// `format!("Failed to insert journal entry: {}", e)`).
        echoue_a_lecriture_comme_la_base: bool,
    }

    impl MockJournalEntryRepository {
        fn new() -> Self {
            Self {
                entries: Mutex::new(HashMap::new()),
                lines: Mutex::new(HashMap::new()),
                echoue_a_lecriture_comme_la_base: false,
            }
        }

        /// #762 @security : un dépôt qui échoue comme Postgres échoue —
        /// avec un message qui nomme une contrainte et une table.
        fn qui_echoue_comme_la_base() -> Self {
            Self {
                echoue_a_lecriture_comme_la_base: true,
                ..Self::new()
            }
        }
    }

    #[async_trait]
    impl JournalEntryRepository for MockJournalEntryRepository {
        async fn create(&self, entry: &JournalEntry) -> Result<JournalEntry, String> {
            let mut entries = self.entries.lock().unwrap();
            entries.insert(entry.id, entry.clone());
            let mut lines = self.lines.lock().unwrap();
            lines.insert(entry.id, entry.lines.clone());
            Ok(entry.clone())
        }

        async fn find_by_organization(
            &self,
            organization_id: Uuid,
        ) -> Result<Vec<JournalEntry>, String> {
            let entries = self.entries.lock().unwrap();
            Ok(entries
                .values()
                .filter(|e| e.organization_id == organization_id)
                .cloned()
                .collect())
        }

        async fn find_by_expense(&self, expense_id: Uuid) -> Result<Vec<JournalEntry>, String> {
            let entries = self.entries.lock().unwrap();
            Ok(entries
                .values()
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
                .values()
                .filter(|e| e.contribution_id == Some(contribution_id))
                .cloned()
                .collect())
        }

        async fn find_by_date_range(
            &self,
            organization_id: Uuid,
            start_date: DateTime<Utc>,
            end_date: DateTime<Utc>,
        ) -> Result<Vec<JournalEntry>, String> {
            let entries = self.entries.lock().unwrap();
            Ok(entries
                .values()
                .filter(|e| {
                    e.organization_id == organization_id
                        && e.entry_date >= start_date
                        && e.entry_date <= end_date
                })
                .cloned()
                .collect())
        }

        async fn calculate_account_balances(
            &self,
            _organization_id: Uuid,
        ) -> Result<HashMap<String, Decimal>, String> {
            Ok(HashMap::new())
        }

        async fn calculate_account_balances_for_period(
            &self,
            _organization_id: Uuid,
            _start_date: DateTime<Utc>,
            _end_date: DateTime<Utc>,
        ) -> Result<HashMap<String, Decimal>, String> {
            Ok(HashMap::new())
        }

        async fn find_lines_by_account(
            &self,
            _organization_id: Uuid,
            _account_code: &str,
        ) -> Result<Vec<JournalEntryLine>, String> {
            Ok(Vec::new())
        }

        async fn validate_balance(&self, entry_id: Uuid) -> Result<bool, String> {
            let entries = self.entries.lock().unwrap();
            match entries.get(&entry_id) {
                Some(entry) => Ok(entry.is_balanced()),
                None => Err("Entry not found".to_string()),
            }
        }

        async fn calculate_account_balances_for_building(
            &self,
            _organization_id: Uuid,
            _building_id: Uuid,
        ) -> Result<HashMap<String, Decimal>, String> {
            Ok(HashMap::new())
        }

        async fn calculate_account_balances_for_building_and_period(
            &self,
            _organization_id: Uuid,
            _building_id: Uuid,
            _start_date: DateTime<Utc>,
            _end_date: DateTime<Utc>,
        ) -> Result<HashMap<String, Decimal>, String> {
            Ok(HashMap::new())
        }

        async fn create_manual_entry(
            &self,
            entry: &JournalEntry,
            entry_lines: &[JournalEntryLine],
        ) -> Result<(), String> {
            if self.echoue_a_lecriture_comme_la_base {
                return Err("Failed to insert journal entry: insert or update on table \
                     \"journal_entry_lines\" violates foreign key constraint \
                     \"fk_account\""
                    .to_string());
            }
            let mut entries = self.entries.lock().unwrap();
            entries.insert(entry.id, entry.clone());
            let mut lines = self.lines.lock().unwrap();
            lines.insert(entry.id, entry_lines.to_vec());
            Ok(())
        }

        async fn list_entries(
            &self,
            organization_id: Uuid,
            _building_id: Option<Uuid>,
            _journal_type: Option<String>,
            _start_date: Option<DateTime<Utc>>,
            _end_date: Option<DateTime<Utc>>,
            _limit: i64,
            _offset: i64,
        ) -> Result<Vec<JournalEntry>, String> {
            let entries = self.entries.lock().unwrap();
            Ok(entries
                .values()
                .filter(|e| e.organization_id == organization_id)
                .cloned()
                .collect())
        }

        async fn find_by_id(
            &self,
            entry_id: Uuid,
            _organization_id: Uuid,
        ) -> Result<JournalEntry, String> {
            let entries = self.entries.lock().unwrap();
            entries
                .get(&entry_id)
                .cloned()
                .ok_or_else(|| "Journal entry not found".to_string())
        }

        async fn find_lines_by_entry(
            &self,
            entry_id: Uuid,
            _organization_id: Uuid,
        ) -> Result<Vec<JournalEntryLine>, String> {
            let lines = self.lines.lock().unwrap();
            Ok(lines.get(&entry_id).cloned().unwrap_or_default())
        }

        async fn delete_entry(&self, entry_id: Uuid, _organization_id: Uuid) -> Result<(), String> {
            let mut entries = self.entries.lock().unwrap();
            let mut lines = self.lines.lock().unwrap();
            entries.remove(&entry_id);
            lines.remove(&entry_id);
            Ok(())
        }
    }

    // ========== Helpers ==========

    /// Dépôt d'immeubles minimal : un immeuble rattaché à une ACP nommée.
    ///
    /// Il n'existait pas ici parce que l'écriture ne cherchait pas ses livres.
    /// Elle les cherche désormais (ADR-0045).
    struct MockBuildingRepository {
        acp_id: Uuid,
    }

    impl MockBuildingRepository {
        fn immeuble_de(acp_id: Uuid) -> Self {
            Self { acp_id }
        }

        fn immeuble(&self) -> crate::domain::entities::Building {
            crate::domain::entities::Building::new(
                self.acp_id,
                "Résidence du Parc".to_string(),
                "12 Rue de la Loi".to_string(),
                "Brussels".to_string(),
                "1000".to_string(),
                "Belgium".to_string(),
                10,
                1000,
                Some(2015),
            )
            .expect("immeuble valide")
        }
    }

    #[async_trait::async_trait]
    impl crate::application::ports::BuildingRepository for MockBuildingRepository {
        async fn create(
            &self,
            b: &crate::domain::entities::Building,
        ) -> Result<crate::domain::entities::Building, String> {
            Ok(b.clone())
        }
        async fn find_by_id(
            &self,
            _id: Uuid,
        ) -> Result<Option<crate::domain::entities::Building>, String> {
            Ok(Some(self.immeuble()))
        }
        async fn find_all(&self) -> Result<Vec<crate::domain::entities::Building>, String> {
            Ok(vec![self.immeuble()])
        }
        async fn find_all_paginated(
            &self,
            _p: &crate::application::dto::PageRequest,
            _f: &crate::application::dto::BuildingFilters,
        ) -> Result<(Vec<crate::domain::entities::Building>, i64), String> {
            Ok((vec![self.immeuble()], 1))
        }
        async fn update(
            &self,
            b: &crate::domain::entities::Building,
        ) -> Result<crate::domain::entities::Building, String> {
            Ok(b.clone())
        }
        async fn delete(&self, _id: Uuid) -> Result<bool, String> {
            Ok(true)
        }
        async fn find_by_slug(
            &self,
            _s: &str,
        ) -> Result<Option<crate::domain::entities::Building>, String> {
            Ok(Some(self.immeuble()))
        }
        async fn find_by_id_with_metrics(
            &self,
            _id: Uuid,
        ) -> Result<
            Option<(
                crate::domain::entities::Building,
                crate::domain::entities::BuildingMetrics,
            )>,
            String,
        > {
            Ok(None)
        }
    }

    fn make_use_cases(repo: MockJournalEntryRepository) -> JournalEntryUseCases {
        make_use_cases_pour_lacp(repo, Uuid::new_v4())
    }

    /// Les mêmes use-cases, en nommant l'ACP dont on tient les comptes.
    fn make_use_cases_pour_lacp(
        repo: MockJournalEntryRepository,
        acp_id: Uuid,
    ) -> JournalEntryUseCases {
        JournalEntryUseCases::new(Arc::new(repo))
            .with_acp_resolution(Arc::new(MockBuildingRepository::immeuble_de(acp_id)))
    }

    /// Balanced lines: 1000 debit on 6100, 1000 credit on 4400
    fn balanced_lines() -> Vec<(String, Decimal, Decimal, String)> {
        vec![
            (
                "6100".to_string(),
                dec!(1000),
                Decimal::ZERO,
                "Utilities expense".to_string(),
            ),
            (
                "4400".to_string(),
                Decimal::ZERO,
                dec!(1000),
                "Supplier payable".to_string(),
            ),
        ]
    }

    // ========== Tests ==========

    /// Art. 3.89 § 5, 15° et ADR-0045 : le grand livre est celui de l'ACP.
    ///
    /// L'ACP se lit sur l'immeuble, jamais sur l'appelant.
    #[tokio::test]
    async fn test_lecriture_sinscrit_dans_les_livres_de_lacp_pas_du_syndic() {
        let acp_des_livres = Uuid::new_v4();
        let cabinet_qui_saisit = Uuid::new_v4();
        let uc = make_use_cases_pour_lacp(MockJournalEntryRepository::new(), acp_des_livres);

        let ecriture = uc
            .create_manual_entry(
                cabinet_qui_saisit,
                Some(Uuid::new_v4()),
                Some("ODS".to_string()),
                Utc::now(),
                Some("Régularisation".to_string()),
                None,
                balanced_lines(),
            )
            .await
            .expect("écriture valide");

        assert_eq!(
            ecriture.acp_id, acp_des_livres,
            "l'écriture s'inscrit dans les livres de l'ACP de l'immeuble"
        );
        assert_eq!(
            ecriture.organization_id, cabinet_qui_saisit,
            "le syndic reste tracé comme auteur de la saisie"
        );
    }

    /// @negative — Issue #762, cas constaté le 2026-09-04 : une écriture qui
    /// ne désigne pas d'immeuble ne dit pas dans quels livres elle s'inscrit.
    /// Le refus, en français (« Impossible de déterminer l'ACP… »), ne
    /// correspondait à aucun motif anglais cherché par le gestionnaire HTTP
    /// et ressortait en 500. Il est maintenant typé `AppError::Validation` :
    /// sa langue n'a plus d'incidence sur le code retourné.
    #[tokio::test]
    async fn test_pas_decriture_sans_livres_identifiables() {
        let uc = make_use_cases(MockJournalEntryRepository::new());

        let erreur = uc
            .create_manual_entry(
                Uuid::new_v4(),
                None, // aucun immeuble
                Some("ODS".to_string()),
                Utc::now(),
                Some("Régularisation".to_string()),
                None,
                balanced_lines(),
            )
            .await
            .expect_err("doit refuser");

        // #762 : la catégorie de l'erreur (400, saisie incomplète) tient au
        // TYPE `AppError::Validation`, pas à un mot cherché dans le message.
        // Le message reste utile pour l'humain ; ce n'est plus sur lui que
        // le gestionnaire HTTP s'appuie pour choisir le code.
        assert!(
            matches!(erreur, AppError::Validation(ref msg) if msg.contains("ACP")),
            "le refus doit être une erreur de VALIDATION nommant ce qui manque : {erreur:?}"
        );
    }

    /// Une écriture mal formée échoue sur son motif, pas sur l'ACP.
    ///
    /// L'ordre compte : si la résolution passait avant les validations de
    /// forme, un déséquilibre remonterait comme un problème de rattachement.
    #[tokio::test]
    async fn test_le_desequilibre_est_signale_avant_la_resolution_de_lacp() {
        let uc = make_use_cases(MockJournalEntryRepository::new());

        let erreur = uc
            .create_manual_entry(
                Uuid::new_v4(),
                None, // pas d'immeuble non plus, et pourtant…
                Some("ODS".to_string()),
                Utc::now(),
                None,
                None,
                vec![
                    ("600".to_string(), dec!(100), dec!(0), "débit".to_string()),
                    ("440".to_string(), dec!(0), dec!(50), "crédit".to_string()),
                ],
            )
            .await
            .expect_err("doit refuser");

        // #762 : même remarque — le TYPE dit déjà « validation », le message
        // ne sert plus qu'à l'humain qui lit la réponse.
        assert!(
            matches!(erreur, AppError::Validation(ref msg) if msg.contains("unbalanced")),
            "…c'est le déséquilibre qui doit être signalé, pas l'ACP : {erreur:?}"
        );
    }

    #[tokio::test]
    async fn test_create_manual_entry_success_balanced() {
        let repo = MockJournalEntryRepository::new();
        let uc = make_use_cases(repo);
        let org_id = Uuid::new_v4();

        let result = uc
            .create_manual_entry(
                org_id,
                Some(Uuid::new_v4()), // l'écriture désigne l'immeuble dont elle relève
                Some("ACH".to_string()),
                Utc::now(),
                Some("Facture eau janvier".to_string()),
                Some("INV-2026-001".to_string()),
                balanced_lines(),
            )
            .await;

        assert!(result.is_ok());
        let entry = result.unwrap();
        assert_eq!(entry.organization_id, org_id);
        assert_eq!(entry.journal_type, Some("ACH".to_string()));
        assert_eq!(entry.description, Some("Facture eau janvier".to_string()));
        assert_eq!(entry.document_ref, Some("INV-2026-001".to_string()));
        assert!(entry.expense_id.is_none());
        assert!(entry.contribution_id.is_none());
        assert_eq!(entry.lines.len(), 2);
    }

    /// @happy — Issue #762 : le chemin nominal d'une erreur applicative
    /// typée. Le déséquilibre remonte en `AppError::Validation`, la variante
    /// que le futur gestionnaire HTTP traduira en 400 sans lire le message.
    #[tokio::test]
    async fn test_create_manual_entry_fail_unbalanced() {
        let repo = MockJournalEntryRepository::new();
        let uc = make_use_cases(repo);
        let org_id = Uuid::new_v4();

        let unbalanced_lines = vec![
            (
                "6100".to_string(),
                dec!(1000),
                Decimal::ZERO,
                "Debit".to_string(),
            ),
            (
                "4400".to_string(),
                Decimal::ZERO,
                dec!(800),
                "Credit".to_string(),
            ),
        ];

        let result = uc
            .create_manual_entry(
                org_id,
                None,
                Some("ACH".to_string()),
                Utc::now(),
                Some("Test unbalanced".to_string()),
                None,
                unbalanced_lines,
            )
            .await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        // #762 : le déséquilibre est une erreur de VALIDATION typée — plus
        // une String que le gestionnaire HTTP devrait reclasser en devinant
        // sur "unbalanced".
        assert!(matches!(err, AppError::Validation(_)), "{err:?}");
        let msg = err.to_string();
        assert!(msg.contains("unbalanced"));
        assert!(msg.contains("debits=1000.00"));
        assert!(msg.contains("credits=800.00"));
    }

    #[tokio::test]
    async fn test_create_manual_entry_fail_invalid_journal_type() {
        let repo = MockJournalEntryRepository::new();
        let uc = make_use_cases(repo);
        let org_id = Uuid::new_v4();

        let result = uc
            .create_manual_entry(
                org_id,
                None,
                Some("INVALID".to_string()),
                Utc::now(),
                Some("Test invalid type".to_string()),
                None,
                balanced_lines(),
            )
            .await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        // #762 : type de journal invalide → VALIDATION typée, jamais Internal.
        assert!(matches!(err, AppError::Validation(_)), "{err:?}");
        let msg = err.to_string();
        assert!(msg.contains("Invalid journal type: INVALID"));
        assert!(msg.contains("ACH"));
        assert!(msg.contains("VEN"));
        assert!(msg.contains("FIN"));
        assert!(msg.contains("ODS"));
    }

    /// @edge — Issue #762 : avant cette story, ce cas n'était même pas dans
    /// la liste de motifs `.contains()` reconnue par le gestionnaire HTTP —
    /// il tombait en 500 par défaut, sans qu'on l'ait jamais remarqué. Le
    /// TYPE (`AppError::Validation`) suffit désormais, sans qu'il ait fallu
    /// l'y ajouter nommément : c'est tout le point d'une erreur typée.
    #[tokio::test]
    async fn test_create_manual_entry_fail_less_than_2_lines() {
        let repo = MockJournalEntryRepository::new();
        let uc = make_use_cases(repo);
        let org_id = Uuid::new_v4();

        let single_line = vec![(
            "6100".to_string(),
            dec!(1000),
            Decimal::ZERO,
            "Only debit".to_string(),
        )];

        let result = uc
            .create_manual_entry(
                org_id,
                None,
                Some("ODS".to_string()),
                Utc::now(),
                Some("Test single line".to_string()),
                None,
                single_line,
            )
            .await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, AppError::Validation(_)), "{err:?}");
        assert!(err.to_string().contains("must have at least 2 lines"));
    }

    #[tokio::test]
    async fn test_delete_manual_entry_success() {
        let repo = MockJournalEntryRepository::new();
        let uc = make_use_cases(repo);
        let org_id = Uuid::new_v4();

        // First create a manual entry
        let created = uc
            .create_manual_entry(
                org_id,
                Some(Uuid::new_v4()), // l'écriture désigne l'immeuble dont elle relève
                Some("FIN".to_string()),
                Utc::now(),
                Some("Manual entry to delete".to_string()),
                None,
                balanced_lines(),
            )
            .await
            .unwrap();

        // Delete it
        let result = uc.delete_manual_entry(created.id, org_id).await;
        assert!(result.is_ok());

        // Verify it was deleted (find_by_id should fail)
        let find_result = uc.get_entry_with_lines(created.id, org_id).await;
        assert!(find_result.is_err());
    }

    #[tokio::test]
    async fn test_delete_manual_entry_fail_auto_generated_with_expense_id() {
        let repo = MockJournalEntryRepository::new();
        let org_id = Uuid::new_v4();
        let entry_id = Uuid::new_v4();
        let expense_id = Uuid::new_v4();

        // Insert an auto-generated entry (has expense_id set)
        {
            let mut entries = repo.entries.lock().unwrap();
            let auto_entry = JournalEntry {
                acp_id: Uuid::new_v4(),
                id: entry_id,
                organization_id: org_id,
                building_id: None,
                entry_date: Utc::now(),
                description: Some("Auto-generated from expense".to_string()),
                document_ref: None,
                journal_type: Some("ACH".to_string()),
                expense_id: Some(expense_id),
                contribution_id: None,
                lines: Vec::new(),
                created_at: Utc::now(),
                updated_at: Utc::now(),
                created_by: None,
            };
            entries.insert(entry_id, auto_entry);
        }

        let uc = make_use_cases(repo);

        let result = uc.delete_manual_entry(entry_id, org_id).await;

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("Cannot delete auto-generated journal entries"));
    }

    /// @security — Issue #762 : classer par sous-chaîne (`foreign key`,
    /// `violates`) suppose qu'on LIT le message brut de la base — donc qu'on
    /// est à un `?` près de le renvoyer tel quel au client. Un échec de
    /// dépôt (contrainte SQL, table interne) doit devenir `AppError::Internal`,
    /// que `AppError::error_response()` masque avant de répondre («
    /// Internal server error », voir application/error.rs) — jamais une
    /// variante qui affiche son contenu tel quel (`Validation`, `Conflict`,
    /// `NotFound`), ce qui exposerait le nom de la contrainte et de la table.
    #[tokio::test]
    async fn negative_762_echec_de_depot_ne_devient_pas_une_validation_qui_exposerait_le_message_brut(
    ) {
        let uc = make_use_cases(MockJournalEntryRepository::qui_echoue_comme_la_base());
        let org_id = Uuid::new_v4();

        let erreur = uc
            .create_manual_entry(
                org_id,
                Some(Uuid::new_v4()),
                Some("ACH".to_string()),
                Utc::now(),
                Some("Test échec dépôt".to_string()),
                None,
                balanced_lines(),
            )
            .await
            .expect_err("le dépôt doit échouer");

        assert!(
            matches!(erreur, AppError::Internal(_)),
            "un échec de dépôt (base, contrainte SQL) reste interne — jamais \
             une catégorie déduite du contenu de son message : {erreur:?}"
        );
        assert!(
            !matches!(
                erreur,
                AppError::Validation(_) | AppError::Conflict(_) | AppError::NotFound(_)
            ),
            "ces variantes affichent leur contenu tel quel au client : le \
             message brut de la base y fuirait : {erreur:?}"
        );
    }
}
