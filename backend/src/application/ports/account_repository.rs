// Application Port: AccountRepository
//
// CREDITS & ATTRIBUTION:
// This repository interface is inspired by the Noalyss project (https://gitlab.com/noalyss/noalyss)
// Noalyss is a free accounting software for Belgian and French accounting
// License: GPL-2.0-or-later (GNU General Public License version 2 or later)
// Copyright: (C) 1989, 1991 Free Software Foundation, Inc.
// Copyright: Dany De Bontridder <dany@alchimerys.eu>
//
// The repository pattern provides abstraction over the Belgian PCMN (Plan Comptable Minimum Normalisé)
// data access layer, following Noalyss' approach to account management.

use crate::domain::entities::{Account, AccountType};
use async_trait::async_trait;
use uuid::Uuid;

/// Repository trait for managing accounts in the Belgian accounting plan
///
/// This port defines the contract for account persistence operations.
/// Implementations must handle:
/// - Multi-tenancy (organization_id filtering)
/// - Hierarchical account relationships (parent_code)
/// - Account code uniqueness within organization
///
/// Inspired by Noalyss Acc_Plan_SQL and Tmp_Pcmn_SQL classes
/// See: include/database/acc_plan_sql.class.php in Noalyss repository
#[async_trait]
pub trait AccountRepository: Send + Sync {
    /// Create a new account in the chart of accounts
    ///
    /// # Arguments
    /// * `account` - Account to create
    ///
    /// # Returns
    /// - `Ok(Account)` - Created account with database-generated ID
    /// - `Err(String)` - Error message if creation fails
    ///
    /// # Errors
    /// - Duplicate account code within organization
    /// - Parent account code does not exist
    /// - Database constraint violation
    async fn create(&self, account: &Account) -> Result<Account, String>;

    /// Find account by ID
    ///
    /// # Arguments
    /// * `id` - Account ID
    ///
    /// # Returns
    /// - `Ok(Some(Account))` - Account found
    /// - `Ok(None)` - Account not found
    /// - `Err(String)` - Database error
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Account>, String>;

    /// Find account by code within an organization
    ///
    /// # Arguments
    /// * `code` - Account code (e.g., "700", "604001")
    /// * `organization_id` - Organization ID
    ///
    /// # Returns
    /// - `Ok(Some(Account))` - Account found
    /// - `Ok(None)` - Account not found
    /// - `Err(String)` - Database error
    async fn find_by_code(
        &self,
        code: &str,
        organization_id: Uuid,
    ) -> Result<Option<Account>, String>;

    /// Find all accounts for an organization
    ///
    /// # Arguments
    /// * `organization_id` - Organization ID
    ///
    /// # Returns
    /// - `Ok(Vec<Account>)` - All accounts for the organization (can be empty)
    /// - `Err(String)` - Database error
    ///
    /// Note: Results are ordered by code ASC for hierarchical display
    async fn find_by_organization(&self, organization_id: Uuid) -> Result<Vec<Account>, String>;

    /// Find accounts by type within an organization
    ///
    /// # Arguments
    /// * `account_type` - Account type (Asset, Liability, Expense, Revenue, OffBalance)
    /// * `organization_id` - Organization ID
    ///
    /// # Returns
    /// - `Ok(Vec<Account>)` - Matching accounts (can be empty)
    /// - `Err(String)` - Database error
    ///
    /// Useful for generating financial reports:
    /// - Balance sheet: Asset + Liability accounts
    /// - Income statement: Expense + Revenue accounts
    async fn find_by_type(
        &self,
        account_type: AccountType,
        organization_id: Uuid,
    ) -> Result<Vec<Account>, String>;

    /// Find child accounts of a parent account
    ///
    /// # Arguments
    /// * `parent_code` - Parent account code (e.g., "60" to find "600", "604", etc.)
    /// * `organization_id` - Organization ID
    ///
    /// # Returns
    /// - `Ok(Vec<Account>)` - Direct children of the parent account (can be empty)
    /// - `Err(String)` - Database error
    ///
    /// Note: Returns only direct children, not all descendants
    async fn find_by_parent_code(
        &self,
        parent_code: &str,
        organization_id: Uuid,
    ) -> Result<Vec<Account>, String>;

    /// Find accounts that can be used directly in transactions
    ///
    /// # Arguments
    /// * `organization_id` - Organization ID
    ///
    /// # Returns
    /// - `Ok(Vec<Account>)` - Accounts with direct_use = true (can be empty)
    /// - `Err(String)` - Database error
    ///
    /// Summary accounts (direct_use = false) cannot be used in journal entries
    async fn find_direct_use_accounts(&self, organization_id: Uuid)
        -> Result<Vec<Account>, String>;

    /// Search accounts by code pattern
    ///
    /// # Arguments
    /// * `code_pattern` - SQL LIKE pattern (e.g., "60%", "604%")
    /// * `organization_id` - Organization ID
    ///
    /// # Returns
    /// - `Ok(Vec<Account>)` - Matching accounts (can be empty)
    /// - `Err(String)` - Database error
    ///
    /// Useful for finding all accounts in a class or sub-class:
    /// - "6%" - All expenses (class 6)
    /// - "60%" - All class 60 expenses
    /// - "604%" - All accounts under 604
    async fn search_by_code_pattern(
        &self,
        code_pattern: &str,
        organization_id: Uuid,
    ) -> Result<Vec<Account>, String>;

    /// Update an existing account
    ///
    /// # Arguments
    /// * `account` - Account with updated fields
    ///
    /// # Returns
    /// - `Ok(Account)` - Updated account
    /// - `Err(String)` - Error message if update fails
    ///
    /// # Errors
    /// - Account not found
    /// - Code change would create duplicate
    /// - Parent code does not exist
    /// - Database constraint violation
    async fn update(&self, account: &Account) -> Result<Account, String>;

    /// Delete an account
    ///
    /// # Arguments
    /// * `id` - Account ID
    ///
    /// # Returns
    /// - `Ok(())` - Account deleted successfully
    /// - `Err(String)` - Error message if deletion fails
    ///
    /// # Errors
    /// - Account not found
    /// - Account has child accounts (cannot delete parent)
    /// - Account is used in expenses/transactions (referential integrity)
    ///
    /// Inspired by Noalyss Acc_Plan_SQL::delete() validation logic
    async fn delete(&self, id: Uuid) -> Result<(), String>;

    /// Check if an account code exists within an organization
    ///
    /// # Arguments
    /// * `code` - Account code
    /// * `organization_id` - Organization ID
    ///
    /// # Returns
    /// - `Ok(true)` - Account exists
    /// - `Ok(false)` - Account does not exist
    /// - `Err(String)` - Database error
    async fn exists(&self, code: &str, organization_id: Uuid) -> Result<bool, String>;

    /// Count accounts in an organization
    ///
    /// # Arguments
    /// * `organization_id` - Organization ID
    ///
    /// # Returns
    /// - `Ok(i64)` - Number of accounts
    /// - `Err(String)` - Database error
    async fn count_by_organization(&self, organization_id: Uuid) -> Result<i64, String>;
}
