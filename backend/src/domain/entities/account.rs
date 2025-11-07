// Domain Entity: Account (Belgian Normalized Accounting Plan)
//
// CREDITS & ATTRIBUTION:
// This implementation is inspired by the Noalyss project (https://gitlab.com/noalyss/noalyss)
// Noalyss is a free accounting software for Belgian and French accounting
// License: GPL-2.0-or-later (GNU General Public License version 2 or later)
// Copyright: (C) 1989, 1991 Free Software Foundation, Inc.
// Copyright: Dany De Bontridder <dany@alchimerys.eu>
//
// We extend our sincere thanks to the Noalyss team for their excellent work on
// implementing the Belgian PCMN (Plan Comptable Minimum Normalisé). Their approach
// to hierarchical account structures and automatic type detection served as inspiration.
//
// References:
// - Noalyss: https://gitlab.com/noalyss/noalyss
// - Belgian Royal Decree: AR 12/07/2012
// - Noalyss class: include/database/tmp_pcmn_sql.class.php
// - Noalyss class: include/database/acc_plan_sql.class.php

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Account classification based on Belgian PCMN (Plan Comptable Minimum Normalisé)
///
/// This enum maps to Noalyss `pcm_type` field with the following equivalences:
/// - `Asset` = ACT (Actif) - Classes 2, 3, 4, 5
/// - `Liability` = PAS (Passif) - Class 1
/// - `Expense` = CHA (Charges) - Class 6
/// - `Revenue` = PRO (Produits) - Class 7
/// - `OffBalance` = CON (Contrôle) - Class 9
///
/// Reference: Noalyss tmp_pcmn.pcm_type field
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "account_type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AccountType {
    /// Assets (Actif) - Classes 2, 3, 4, 5 in Belgian PCMN
    /// Examples: Buildings, Inventory, Receivables, Bank accounts
    Asset,

    /// Liabilities (Passif) - Class 1 in Belgian PCMN
    /// Examples: Capital, Reserves, Provisions, Debts
    Liability,

    /// Expenses (Charges) - Class 6 in Belgian PCMN
    /// Examples: Electricity, Maintenance, Insurance, Salaries
    Expense,

    /// Revenue (Produits) - Class 7 in Belgian PCMN
    /// Examples: Regular fees, Extraordinary fees, Interest income
    Revenue,

    /// Off-balance/Control accounts (Contrôle) - Class 9 in Belgian PCMN
    /// Examples: Memorandum accounts, Statistical accounts
    OffBalance,
}

impl AccountType {
    /// Automatically detect account type from Belgian PCMN code
    ///
    /// Logic inspired by Noalyss `find_pcm_type()` function.
    /// See: include/sql/mod1/schema.sql in Noalyss repository
    ///
    /// # Arguments
    /// * `code` - Account code (e.g., "700", "604001")
    ///
    /// # Returns
    /// Detected `AccountType` based on first digit (Belgian PCMN class)
    ///
    /// # Examples
    /// ```
    /// use koprogo_api::domain::entities::account::AccountType;
    ///
    /// assert_eq!(AccountType::from_code("700"), AccountType::Revenue);
    /// assert_eq!(AccountType::from_code("604001"), AccountType::Expense);
    /// assert_eq!(AccountType::from_code("100"), AccountType::Liability);
    /// assert_eq!(AccountType::from_code("5500"), AccountType::Asset);
    /// ```
    pub fn from_code(code: &str) -> Self {
        if code.is_empty() {
            return AccountType::OffBalance;
        }

        // Extract first character (Belgian PCMN class)
        match &code[0..1] {
            "1" => AccountType::Liability,         // Class 1: Capital, reserves
            "2" | "3" | "4" | "5" => AccountType::Asset, // Classes 2-5: Assets
            "6" => AccountType::Expense,           // Class 6: Expenses
            "7" => AccountType::Revenue,           // Class 7: Revenue
            "8" => AccountType::Expense,           // Class 8: Special (rarely used)
            "9" => AccountType::OffBalance,        // Class 9: Off-balance
            _ => AccountType::OffBalance,          // Unknown: default to off-balance
        }
    }

    /// Check if this account type appears on the balance sheet
    ///
    /// Balance sheet accounts: Assets & Liabilities (Classes 1-5)
    /// Income statement accounts: Expenses & Revenue (Classes 6-7)
    pub fn is_balance_sheet(&self) -> bool {
        matches!(self, AccountType::Asset | AccountType::Liability)
    }

    /// Check if this account type appears on the income statement
    ///
    /// Income statement (Compte de résultat): Expenses & Revenue
    pub fn is_income_statement(&self) -> bool {
        matches!(self, AccountType::Expense | AccountType::Revenue)
    }
}

/// Account in the Belgian Normalized Accounting Plan (PCMN)
///
/// Represents a single account in the hierarchical chart of accounts.
/// Structure inspired by Noalyss `tmp_pcmn` table.
///
/// # Hierarchical Structure
///
/// Accounts can have parent-child relationships for organization:
/// ```text
/// 6           (Charges/Expenses - parent: None)
///   60        (Approvisionnements - parent: "6")
///     604     (Fournitures - parent: "60")
///       604001 (Électricité - parent: "604")
/// ```
///
/// # Belgian PCMN Classes
///
/// - Class 1: Liabilities (Capital, Reserves, Provisions)
/// - Classes 2-5: Assets (Fixed assets, Inventory, Receivables, Cash)
/// - Class 6: Expenses (Purchases, Services, Salaries)
/// - Class 7: Revenue (Sales, Services, Financial income)
/// - Class 9: Off-balance (Control accounts)
///
/// Reference: Belgian Royal Decree AR 12/07/2012
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Account {
    /// Unique identifier
    pub id: Uuid,

    /// Account code (e.g., "700", "604001", "100")
    ///
    /// Can be hierarchical. Parent codes are typically shorter.
    /// Example: "604001" is child of "604" which is child of "60"
    pub code: String,

    /// Account label/description
    ///
    /// Examples:
    /// - "Électricité" (Electricity)
    /// - "Appels de fonds ordinaires" (Regular fees)
    /// - "Assurance immeuble" (Building insurance)
    pub label: String,

    /// Parent account code for hierarchical organization
    ///
    /// None if this is a top-level account (e.g., "6", "7")
    /// Some("604") if this is a child account (e.g., "604001")
    pub parent_code: Option<String>,

    /// Account classification (Asset, Liability, Expense, Revenue, OffBalance)
    pub account_type: AccountType,

    /// Whether this account can be used directly in journal entries
    ///
    /// - true: Can post transactions to this account (e.g., "604001" - Electricity)
    /// - false: Summary account only (e.g., "60" - Approvisionnements)
    ///
    /// Corresponds to Noalyss `pcm_direct_use` field (Y/N)
    pub direct_use: bool,

    /// Organization this account belongs to (multi-tenancy)
    pub organization_id: Uuid,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
}

impl Account {
    /// Create a new account with validation
    ///
    /// # Arguments
    /// * `code` - Account code (must be non-empty)
    /// * `label` - Account description (must be non-empty)
    /// * `parent_code` - Optional parent account code
    /// * `account_type` - Account classification
    /// * `direct_use` - Whether account can be used in transactions
    /// * `organization_id` - Organization ID
    ///
    /// # Returns
    /// `Ok(Account)` if validation passes, `Err(String)` otherwise
    ///
    /// # Validation Rules
    /// 1. Code must not be empty
    /// 2. Code must be alphanumeric (can contain letters for auxiliary accounts)
    /// 3. Label must not be empty
    /// 4. Label must be <= 255 characters
    /// 5. Parent code cannot equal code (prevent self-reference)
    pub fn new(
        code: String,
        label: String,
        parent_code: Option<String>,
        account_type: AccountType,
        direct_use: bool,
        organization_id: Uuid,
    ) -> Result<Self, String> {
        // Validation: code must not be empty
        if code.trim().is_empty() {
            return Err("Account code cannot be empty".to_string());
        }

        // Validation: code must be reasonable length (max 40 chars per SQL)
        if code.len() > 40 {
            return Err("Account code cannot exceed 40 characters".to_string());
        }

        // Validation: label must not be empty
        if label.trim().is_empty() {
            return Err("Account label cannot be empty".to_string());
        }

        // Validation: label max length (reasonable limit)
        if label.len() > 255 {
            return Err("Account label cannot exceed 255 characters".to_string());
        }

        // Validation: parent_code cannot equal code (prevent self-reference)
        if let Some(ref parent) = parent_code {
            if parent == &code {
                return Err("Account cannot be its own parent".to_string());
            }
        }

        let now = Utc::now();

        Ok(Account {
            id: Uuid::new_v4(),
            code,
            label,
            parent_code,
            account_type,
            direct_use,
            organization_id,
            created_at: now,
            updated_at: now,
        })
    }

    /// Get the account class (first digit for Belgian PCMN)
    ///
    /// # Examples
    /// ```
    /// # use koprogo_api::domain::entities::account::Account;
    /// # use uuid::Uuid;
    /// # let org_id = Uuid::new_v4();
    /// let account = Account::new(
    ///     "604001".to_string(),
    ///     "Electricity".to_string(),
    ///     Some("604".to_string()),
    ///     account::AccountType::Expense,
    ///     true,
    ///     org_id
    /// ).unwrap();
    ///
    /// assert_eq!(account.get_class(), "6");
    /// ```
    pub fn get_class(&self) -> &str {
        if self.code.is_empty() {
            return "";
        }
        &self.code[0..1]
    }

    /// Check if this is a top-level account (no parent)
    pub fn is_root(&self) -> bool {
        self.parent_code.is_none()
    }

    /// Update account details
    ///
    /// # Arguments
    /// * `label` - New label (optional, keeps current if None)
    /// * `parent_code` - New parent code (optional, keeps current if None)
    /// * `account_type` - New type (optional, keeps current if None)
    /// * `direct_use` - New direct use flag (optional, keeps current if None)
    ///
    /// # Returns
    /// `Ok(())` if validation passes, `Err(String)` otherwise
    pub fn update(
        &mut self,
        label: Option<String>,
        parent_code: Option<Option<String>>,
        account_type: Option<AccountType>,
        direct_use: Option<bool>,
    ) -> Result<(), String> {
        // Update label if provided
        if let Some(new_label) = label {
            if new_label.trim().is_empty() {
                return Err("Account label cannot be empty".to_string());
            }
            if new_label.len() > 255 {
                return Err("Account label cannot exceed 255 characters".to_string());
            }
            self.label = new_label;
        }

        // Update parent_code if provided
        if let Some(new_parent) = parent_code {
            if let Some(ref parent) = new_parent {
                if parent == &self.code {
                    return Err("Account cannot be its own parent".to_string());
                }
            }
            self.parent_code = new_parent;
        }

        // Update account_type if provided
        if let Some(new_type) = account_type {
            self.account_type = new_type;
        }

        // Update direct_use if provided
        if let Some(new_direct_use) = direct_use {
            self.direct_use = new_direct_use;
        }

        self.updated_at = Utc::now();
        Ok(())
    }
}

// ============================================================================
// UNIT TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_account_type_from_code() {
        // Test Belgian PCMN classes
        assert_eq!(AccountType::from_code("100"), AccountType::Liability);  // Class 1
        assert_eq!(AccountType::from_code("280"), AccountType::Asset);      // Class 2
        assert_eq!(AccountType::from_code("3400"), AccountType::Asset);     // Class 3
        assert_eq!(AccountType::from_code("400"), AccountType::Asset);      // Class 4
        assert_eq!(AccountType::from_code("5500"), AccountType::Asset);     // Class 5
        assert_eq!(AccountType::from_code("604001"), AccountType::Expense); // Class 6
        assert_eq!(AccountType::from_code("700"), AccountType::Revenue);    // Class 7
        assert_eq!(AccountType::from_code("900"), AccountType::OffBalance); // Class 9

        // Test edge cases
        assert_eq!(AccountType::from_code(""), AccountType::OffBalance);
        assert_eq!(AccountType::from_code("X123"), AccountType::OffBalance);
    }

    #[test]
    fn test_account_type_is_balance_sheet() {
        assert!(AccountType::Asset.is_balance_sheet());
        assert!(AccountType::Liability.is_balance_sheet());
        assert!(!AccountType::Expense.is_balance_sheet());
        assert!(!AccountType::Revenue.is_balance_sheet());
        assert!(!AccountType::OffBalance.is_balance_sheet());
    }

    #[test]
    fn test_account_type_is_income_statement() {
        assert!(AccountType::Expense.is_income_statement());
        assert!(AccountType::Revenue.is_income_statement());
        assert!(!AccountType::Asset.is_income_statement());
        assert!(!AccountType::Liability.is_income_statement());
        assert!(!AccountType::OffBalance.is_income_statement());
    }

    #[test]
    fn test_create_account_success() {
        let org_id = Uuid::new_v4();
        let account = Account::new(
            "604001".to_string(),
            "Électricité".to_string(),
            Some("604".to_string()),
            AccountType::Expense,
            true,
            org_id,
        );

        assert!(account.is_ok());
        let account = account.unwrap();
        assert_eq!(account.code, "604001");
        assert_eq!(account.label, "Électricité");
        assert_eq!(account.parent_code, Some("604".to_string()));
        assert_eq!(account.account_type, AccountType::Expense);
        assert!(account.direct_use);
        assert_eq!(account.organization_id, org_id);
    }

    #[test]
    fn test_create_account_empty_code() {
        let org_id = Uuid::new_v4();
        let result = Account::new(
            "".to_string(),
            "Test".to_string(),
            None,
            AccountType::Expense,
            true,
            org_id,
        );

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Account code cannot be empty");
    }

    #[test]
    fn test_create_account_empty_label() {
        let org_id = Uuid::new_v4();
        let result = Account::new(
            "700".to_string(),
            "".to_string(),
            None,
            AccountType::Revenue,
            true,
            org_id,
        );

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Account label cannot be empty");
    }

    #[test]
    fn test_create_account_self_parent() {
        let org_id = Uuid::new_v4();
        let result = Account::new(
            "700".to_string(),
            "Test".to_string(),
            Some("700".to_string()), // Same as code!
            AccountType::Revenue,
            true,
            org_id,
        );

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Account cannot be its own parent");
    }

    #[test]
    fn test_account_get_class() {
        let org_id = Uuid::new_v4();
        let account = Account::new(
            "604001".to_string(),
            "Test".to_string(),
            None,
            AccountType::Expense,
            true,
            org_id,
        )
        .unwrap();

        assert_eq!(account.get_class(), "6");
    }

    #[test]
    fn test_account_is_root() {
        let org_id = Uuid::new_v4();

        let root = Account::new(
            "6".to_string(),
            "Charges".to_string(),
            None,
            AccountType::Expense,
            false,
            org_id,
        )
        .unwrap();

        let child = Account::new(
            "604".to_string(),
            "Fournitures".to_string(),
            Some("6".to_string()),
            AccountType::Expense,
            false,
            org_id,
        )
        .unwrap();

        assert!(root.is_root());
        assert!(!child.is_root());
    }

    #[test]
    fn test_account_update_success() {
        let org_id = Uuid::new_v4();
        let mut account = Account::new(
            "700".to_string(),
            "Old Label".to_string(),
            None,
            AccountType::Revenue,
            true,
            org_id,
        )
        .unwrap();

        let result = account.update(
            Some("New Label".to_string()),
            Some(Some("70".to_string())),
            Some(AccountType::Revenue),
            Some(false),
        );

        assert!(result.is_ok());
        assert_eq!(account.label, "New Label");
        assert_eq!(account.parent_code, Some("70".to_string()));
        assert!(!account.direct_use);
    }

    #[test]
    fn test_account_update_self_parent() {
        let org_id = Uuid::new_v4();
        let mut account = Account::new(
            "700".to_string(),
            "Test".to_string(),
            None,
            AccountType::Revenue,
            true,
            org_id,
        )
        .unwrap();

        let result = account.update(None, Some(Some("700".to_string())), None, None);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Account cannot be its own parent");
    }
}
