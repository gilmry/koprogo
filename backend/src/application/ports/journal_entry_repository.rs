// Application Port: Journal Entry Repository
//
// CREDITS & ATTRIBUTION:
// This implementation is inspired by the Noalyss project (https://gitlab.com/noalyss/noalyss)
// Noalyss is a free accounting software for Belgian and French accounting
// License: GPL-2.0-or-later (GNU General Public License version 2 or later)
// Copyright: (C) 1989, 1991 Free Software Foundation, Inc.
// Copyright: Dany De Bontridder <dany@alchimerys.eu>

use crate::domain::entities::{JournalEntry, JournalEntryLine};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use uuid::Uuid;

/// Repository port for journal entries (double-entry bookkeeping)
///
/// This trait defines operations for managing accounting journal entries
/// inspired by Noalyss' jrn/jrnx table structure.
#[async_trait]
pub trait JournalEntryRepository: Send + Sync {
    /// Create a new journal entry with its lines
    ///
    /// # Arguments
    /// - `entry`: The journal entry to create (must be balanced)
    ///
    /// # Returns
    /// - `Ok(JournalEntry)` with generated IDs and timestamps
    /// - `Err(String)` if validation fails or database error
    ///
    /// # Database Constraints
    /// - Triggers validate that total debits = total credits
    /// - Foreign keys validate account codes exist
    async fn create(&self, entry: &JournalEntry) -> Result<JournalEntry, String>;

    /// Find all journal entries for an organization
    ///
    /// Returns entries ordered by entry_date DESC.
    async fn find_by_organization(
        &self,
        organization_id: Uuid,
    ) -> Result<Vec<JournalEntry>, String>;

    /// Find journal entries linked to a specific expense
    ///
    /// Returns all entries that were auto-generated from this expense.
    async fn find_by_expense(&self, expense_id: Uuid) -> Result<Vec<JournalEntry>, String>;

    /// Find journal entries for a date range
    ///
    /// Useful for generating period reports (income statement).
    async fn find_by_date_range(
        &self,
        organization_id: Uuid,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> Result<Vec<JournalEntry>, String>;

    /// Calculate account balances from journal entry lines
    ///
    /// This replaces the old method of calculating balances directly from expenses.
    ///
    /// # Arguments
    /// - `organization_id`: Organization to calculate for
    ///
    /// # Returns
    /// - `HashMap<account_code, balance>` where:
    ///   - Assets/Expenses: balance = debits - credits
    ///   - Liabilities/Revenue: balance = credits - debits
    ///
    /// # Example
    /// ```ignore
    /// {
    ///   "6100": 5000.0,   // Utilities expense
    ///   "4110": 1050.0,   // VAT recoverable
    ///   "4400": -6050.0,  // Suppliers payable (negative = liability)
    ///   "5500": 6050.0    // Bank (after payment)
    /// }
    /// ```
    async fn calculate_account_balances(
        &self,
        organization_id: Uuid,
    ) -> Result<HashMap<String, f64>, String>;

    /// Calculate account balances for a specific period
    ///
    /// Same as `calculate_account_balances` but filtered by entry_date.
    async fn calculate_account_balances_for_period(
        &self,
        organization_id: Uuid,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> Result<HashMap<String, f64>, String>;

    /// Get all journal entry lines for an account
    ///
    /// Useful for displaying account ledgers (grand-livre).
    async fn find_lines_by_account(
        &self,
        organization_id: Uuid,
        account_code: &str,
    ) -> Result<Vec<JournalEntryLine>, String>;

    /// Validate that an entry is balanced (debits = credits)
    ///
    /// This is a safety check before persisting. Database triggers also enforce this.
    async fn validate_balance(&self, entry_id: Uuid) -> Result<bool, String>;

    /// Calculate account balances for a specific building
    ///
    /// Filters journal entries by those linked to expenses/contributions for the building.
    async fn calculate_account_balances_for_building(
        &self,
        organization_id: Uuid,
        building_id: Uuid,
    ) -> Result<HashMap<String, f64>, String>;

    /// Calculate account balances for a specific building and period
    async fn calculate_account_balances_for_building_and_period(
        &self,
        organization_id: Uuid,
        building_id: Uuid,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> Result<HashMap<String, f64>, String>;

    /// Create a manual journal entry with multiple lines
    async fn create_manual_entry(
        &self,
        entry: &JournalEntry,
        lines: &[JournalEntryLine],
    ) -> Result<(), String>;

    /// List journal entries with filters
    #[allow(clippy::too_many_arguments)]
    async fn list_entries(
        &self,
        organization_id: Uuid,
        building_id: Option<Uuid>,
        journal_type: Option<String>,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<JournalEntry>, String>;

    /// Find a journal entry by ID
    async fn find_by_id(
        &self,
        entry_id: Uuid,
        organization_id: Uuid,
    ) -> Result<JournalEntry, String>;

    /// Find all lines for a journal entry
    async fn find_lines_by_entry(
        &self,
        entry_id: Uuid,
        organization_id: Uuid,
    ) -> Result<Vec<JournalEntryLine>, String>;

    /// Delete a journal entry and its lines
    async fn delete_entry(&self, entry_id: Uuid, organization_id: Uuid) -> Result<(), String>;
}
