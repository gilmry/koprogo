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
