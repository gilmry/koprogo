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

use crate::application::ports::journal_entry_repository::JournalEntryRepository;
use crate::domain::entities::journal_entry::{JournalEntry, JournalEntryLine};
use chrono::{DateTime, Utc};
use std::sync::Arc;
use uuid::Uuid;

pub struct JournalEntryUseCases {
    journal_entry_repo: Arc<dyn JournalEntryRepository>,
}

impl JournalEntryUseCases {
    pub fn new(journal_entry_repo: Arc<dyn JournalEntryRepository>) -> Self {
        Self { journal_entry_repo }
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
        lines: Vec<(String, f64, f64, String)>, // (account_code, debit, credit, line_description)
    ) -> Result<JournalEntry, String> {
        // Validate journal type if provided (inspired by Noalyss journal types)
        if let Some(ref jtype) = journal_type {
            if !["ACH", "VEN", "FIN", "ODS"].contains(&jtype.as_str()) {
                return Err(format!(
                    "Invalid journal type: {}. Must be one of: ACH (Purchases), VEN (Sales), FIN (Financial), ODS (Miscellaneous)",
                    jtype
                ));
            }
        }

        // Validate that we have at least 2 lines (double-entry principle)
        if lines.len() < 2 {
            return Err("Journal entry must have at least 2 lines (debit and credit)".to_string());
        }

        // Calculate totals and validate balance (Noalyss principle)
        let total_debit: f64 = lines.iter().map(|(_, debit, _, _)| debit).sum();
        let total_credit: f64 = lines.iter().map(|(_, _, credit, _)| credit).sum();

        if (total_debit - total_credit).abs() > 0.01 {
            return Err(format!(
                "Journal entry is unbalanced: debits={:.2} credits={:.2}. Debits must equal credits.",
                total_debit, total_credit
            ));
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

        // Create journal entry
        let journal_entry = JournalEntry {
            id: entry_id,
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
        self.journal_entry_repo
            .create_manual_entry(&journal_entry, &journal_lines)
            .await?;

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
    }

    impl MockJournalEntryRepository {
        fn new() -> Self {
            Self {
                entries: Mutex::new(HashMap::new()),
                lines: Mutex::new(HashMap::new()),
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
        ) -> Result<HashMap<String, f64>, String> {
            Ok(HashMap::new())
        }

        async fn calculate_account_balances_for_period(
            &self,
            _organization_id: Uuid,
            _start_date: DateTime<Utc>,
            _end_date: DateTime<Utc>,
        ) -> Result<HashMap<String, f64>, String> {
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
        ) -> Result<HashMap<String, f64>, String> {
            Ok(HashMap::new())
        }

        async fn calculate_account_balances_for_building_and_period(
            &self,
            _organization_id: Uuid,
            _building_id: Uuid,
            _start_date: DateTime<Utc>,
            _end_date: DateTime<Utc>,
        ) -> Result<HashMap<String, f64>, String> {
            Ok(HashMap::new())
        }

        async fn create_manual_entry(
            &self,
            entry: &JournalEntry,
            entry_lines: &[JournalEntryLine],
        ) -> Result<(), String> {
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

    fn make_use_cases(repo: MockJournalEntryRepository) -> JournalEntryUseCases {
        JournalEntryUseCases::new(Arc::new(repo))
    }

    /// Balanced lines: 1000 debit on 6100, 1000 credit on 4400
    fn balanced_lines() -> Vec<(String, f64, f64, String)> {
        vec![
            (
                "6100".to_string(),
                1000.0,
                0.0,
                "Utilities expense".to_string(),
            ),
            (
                "4400".to_string(),
                0.0,
                1000.0,
                "Supplier payable".to_string(),
            ),
        ]
    }

    // ========== Tests ==========

    #[tokio::test]
    async fn test_create_manual_entry_success_balanced() {
        let repo = MockJournalEntryRepository::new();
        let uc = make_use_cases(repo);
        let org_id = Uuid::new_v4();

        let result = uc
            .create_manual_entry(
                org_id,
                None,
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

    #[tokio::test]
    async fn test_create_manual_entry_fail_unbalanced() {
        let repo = MockJournalEntryRepository::new();
        let uc = make_use_cases(repo);
        let org_id = Uuid::new_v4();

        let unbalanced_lines = vec![
            ("6100".to_string(), 1000.0, 0.0, "Debit".to_string()),
            ("4400".to_string(), 0.0, 800.0, "Credit".to_string()),
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
        assert!(err.contains("unbalanced"));
        assert!(err.contains("debits=1000.00"));
        assert!(err.contains("credits=800.00"));
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
        assert!(err.contains("Invalid journal type: INVALID"));
        assert!(err.contains("ACH"));
        assert!(err.contains("VEN"));
        assert!(err.contains("FIN"));
        assert!(err.contains("ODS"));
    }

    #[tokio::test]
    async fn test_create_manual_entry_fail_less_than_2_lines() {
        let repo = MockJournalEntryRepository::new();
        let uc = make_use_cases(repo);
        let org_id = Uuid::new_v4();

        let single_line = vec![("6100".to_string(), 1000.0, 0.0, "Only debit".to_string())];

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
        assert!(result.unwrap_err().contains("must have at least 2 lines"));
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
                None,
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
}
