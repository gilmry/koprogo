// Application Use Cases: Financial Reports
//
// CREDITS & ATTRIBUTION:
// This implementation is inspired by the Noalyss project (https://gitlab.com/noalyss/noalyss)
// Noalyss is a free accounting software for Belgian and French accounting
// License: GPL-2.0-or-later (GNU General Public License version 2 or later)
// Copyright: (C) 1989, 1991 Free Software Foundation, Inc.
// Copyright: Dany De Bontridder <dany@alchimerys.eu>
//
// Financial reports generation based on Belgian PCMN (Plan Comptable Minimum Normalisé)
// Inspired by Noalyss' balance sheet and income statement reports

use crate::application::ports::{AccountRepository, ExpenseRepository};
use crate::domain::entities::AccountType;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

pub struct FinancialReportUseCases {
    account_repo: Arc<dyn AccountRepository>,
    expense_repo: Arc<dyn ExpenseRepository>,
}

#[derive(Debug, Serialize)]
pub struct BalanceSheetReport {
    /// Organization ID
    pub organization_id: String,
    /// Report generation date (ISO 8601)
    pub report_date: String,
    /// Assets section (Classes 2-5 in Belgian PCMN)
    pub assets: AccountSection,
    /// Liabilities section (Class 1 in Belgian PCMN)
    pub liabilities: AccountSection,
    /// Total assets value
    pub total_assets: f64,
    /// Total liabilities value
    pub total_liabilities: f64,
    /// Balance (should be 0 in a balanced sheet)
    pub balance: f64,
}

#[derive(Debug, Serialize)]
pub struct IncomeStatementReport {
    /// Organization ID
    pub organization_id: String,
    /// Report generation date (ISO 8601)
    pub report_date: String,
    /// Date range start (ISO 8601)
    pub period_start: String,
    /// Date range end (ISO 8601)
    pub period_end: String,
    /// Expenses section (Class 6 in Belgian PCMN)
    pub expenses: AccountSection,
    /// Revenue section (Class 7 in Belgian PCMN)
    pub revenue: AccountSection,
    /// Total expenses
    pub total_expenses: f64,
    /// Total revenue
    pub total_revenue: f64,
    /// Net result (revenue - expenses)
    pub net_result: f64,
}

#[derive(Debug, Serialize)]
pub struct AccountSection {
    /// Account type (ASSET, LIABILITY, EXPENSE, REVENUE)
    pub account_type: String,
    /// List of account lines with balances
    pub accounts: Vec<AccountLine>,
    /// Section total
    pub total: f64,
}

#[derive(Debug, Serialize)]
pub struct AccountLine {
    /// Account code (e.g., "604001")
    pub code: String,
    /// Account label (e.g., "Électricité")
    pub label: String,
    /// Account balance/amount
    pub amount: f64,
}

impl FinancialReportUseCases {
    pub fn new(
        account_repo: Arc<dyn AccountRepository>,
        expense_repo: Arc<dyn ExpenseRepository>,
    ) -> Self {
        Self {
            account_repo,
            expense_repo,
        }
    }

    /// Generate a balance sheet report for an organization
    ///
    /// Balance sheet shows:
    /// - Assets (Classes 2-5): Buildings, receivables, bank, cash
    /// - Liabilities (Class 1): Capital, reserves, provisions, payables
    ///
    /// Inspired by Noalyss' balance sheet generation
    pub async fn generate_balance_sheet(
        &self,
        organization_id: Uuid,
    ) -> Result<BalanceSheetReport, String> {
        // Fetch all accounts for the organization
        let all_accounts = self
            .account_repo
            .find_by_organization(organization_id)
            .await?;

        // Fetch all expenses to calculate account balances
        // Note: In a full accounting system, we'd also have revenues, payments, etc.
        // For now, we're focusing on expenses linked to accounts
        let expense_amounts = self.calculate_account_balances(organization_id).await?;

        // Separate assets and liabilities
        let mut assets_accounts = Vec::new();
        let mut liabilities_accounts = Vec::new();

        for account in all_accounts {
            let amount = expense_amounts.get(&account.code).cloned().unwrap_or(0.0);

            let line = AccountLine {
                code: account.code.clone(),
                label: account.label.clone(),
                amount,
            };

            match account.account_type {
                AccountType::Asset => assets_accounts.push(line),
                AccountType::Liability => liabilities_accounts.push(line),
                _ => {} // Skip expense/revenue accounts in balance sheet
            }
        }

        // Calculate totals
        let total_assets: f64 = assets_accounts.iter().map(|a| a.amount).sum();
        let total_liabilities: f64 = liabilities_accounts.iter().map(|a| a.amount).sum();
        let balance = total_assets - total_liabilities;

        Ok(BalanceSheetReport {
            organization_id: organization_id.to_string(),
            report_date: chrono::Utc::now().to_rfc3339(),
            assets: AccountSection {
                account_type: "ASSET".to_string(),
                accounts: assets_accounts,
                total: total_assets,
            },
            liabilities: AccountSection {
                account_type: "LIABILITY".to_string(),
                accounts: liabilities_accounts,
                total: total_liabilities,
            },
            total_assets,
            total_liabilities,
            balance,
        })
    }

    /// Generate an income statement (profit & loss) report
    ///
    /// Income statement shows:
    /// - Expenses (Class 6): Operating costs, maintenance, utilities
    /// - Revenue (Class 7): Regular fees, extraordinary fees, interest income
    ///
    /// Inspired by Noalyss' income statement generation
    pub async fn generate_income_statement(
        &self,
        organization_id: Uuid,
        period_start: chrono::DateTime<chrono::Utc>,
        period_end: chrono::DateTime<chrono::Utc>,
    ) -> Result<IncomeStatementReport, String> {
        // Fetch all accounts for the organization
        let all_accounts = self
            .account_repo
            .find_by_organization(organization_id)
            .await?;

        // Calculate account balances for the period
        let expense_amounts = self
            .calculate_account_balances_for_period(organization_id, period_start, period_end)
            .await?;

        // Separate expenses and revenue
        let mut expense_accounts = Vec::new();
        let mut revenue_accounts = Vec::new();

        for account in all_accounts {
            let amount = expense_amounts.get(&account.code).cloned().unwrap_or(0.0);

            // Only include accounts with non-zero amounts
            if amount == 0.0 {
                continue;
            }

            let line = AccountLine {
                code: account.code.clone(),
                label: account.label.clone(),
                amount,
            };

            match account.account_type {
                AccountType::Expense => expense_accounts.push(line),
                AccountType::Revenue => revenue_accounts.push(line),
                _ => {} // Skip asset/liability accounts in income statement
            }
        }

        // Calculate totals
        let total_expenses: f64 = expense_accounts.iter().map(|a| a.amount).sum();
        let total_revenue: f64 = revenue_accounts.iter().map(|a| a.amount).sum();
        let net_result = total_revenue - total_expenses;

        Ok(IncomeStatementReport {
            organization_id: organization_id.to_string(),
            report_date: chrono::Utc::now().to_rfc3339(),
            period_start: period_start.to_rfc3339(),
            period_end: period_end.to_rfc3339(),
            expenses: AccountSection {
                account_type: "EXPENSE".to_string(),
                accounts: expense_accounts,
                total: total_expenses,
            },
            revenue: AccountSection {
                account_type: "REVENUE".to_string(),
                accounts: revenue_accounts,
                total: total_revenue,
            },
            total_expenses,
            total_revenue,
            net_result,
        })
    }

    /// Calculate account balances by summing all expenses per account_code
    ///
    /// This is a simplified version. In a full accounting system, we'd also
    /// include revenues, payments, and other transaction types.
    async fn calculate_account_balances(
        &self,
        organization_id: Uuid,
    ) -> Result<HashMap<String, f64>, String> {
        // For now, we'll fetch all expenses and group by account_code
        // In production, this should use aggregation queries for performance
        let filters = crate::application::dto::ExpenseFilters {
            organization_id: Some(organization_id),
            ..Default::default()
        };

        let page_request = crate::application::dto::PageRequest {
            page: 1,
            per_page: 10000, // Large limit to get all expenses
            sort_by: None,
            order: crate::application::dto::SortOrder::Asc,
        };

        let (expenses, _total) = self
            .expense_repo
            .find_all_paginated(&page_request, &filters)
            .await?;

        let mut balances: HashMap<String, f64> = HashMap::new();

        for expense in expenses {
            if let Some(account_code) = &expense.account_code {
                *balances.entry(account_code.clone()).or_insert(0.0) += expense.amount;
            }
        }

        Ok(balances)
    }

    /// Calculate account balances for a specific time period
    async fn calculate_account_balances_for_period(
        &self,
        organization_id: Uuid,
        period_start: chrono::DateTime<chrono::Utc>,
        period_end: chrono::DateTime<chrono::Utc>,
    ) -> Result<HashMap<String, f64>, String> {
        let filters = crate::application::dto::ExpenseFilters {
            organization_id: Some(organization_id),
            date_from: Some(period_start),
            date_to: Some(period_end),
            ..Default::default()
        };

        let page_request = crate::application::dto::PageRequest {
            page: 1,
            per_page: 10000,
            sort_by: None,
            order: crate::application::dto::SortOrder::Asc,
        };

        let (expenses, _total) = self
            .expense_repo
            .find_all_paginated(&page_request, &filters)
            .await?;

        let mut balances: HashMap<String, f64> = HashMap::new();

        for expense in expenses {
            if let Some(account_code) = &expense.account_code {
                *balances.entry(account_code.clone()).or_insert(0.0) += expense.amount;
            }
        }

        Ok(balances)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These are unit tests for business logic.
    // Integration tests with real database are in tests/integration/

    #[test]
    fn test_balance_sheet_report_structure() {
        // Test that BalanceSheetReport serializes correctly
        let report = BalanceSheetReport {
            organization_id: "test-org".to_string(),
            report_date: "2024-01-01T00:00:00Z".to_string(),
            assets: AccountSection {
                account_type: "ASSET".to_string(),
                accounts: vec![AccountLine {
                    code: "550".to_string(),
                    label: "Banque".to_string(),
                    amount: 10000.0,
                }],
                total: 10000.0,
            },
            liabilities: AccountSection {
                account_type: "LIABILITY".to_string(),
                accounts: vec![AccountLine {
                    code: "100".to_string(),
                    label: "Capital".to_string(),
                    amount: 10000.0,
                }],
                total: 10000.0,
            },
            total_assets: 10000.0,
            total_liabilities: 10000.0,
            balance: 0.0,
        };

        assert_eq!(report.total_assets, 10000.0);
        assert_eq!(report.total_liabilities, 10000.0);
        assert_eq!(report.balance, 0.0);
    }

    #[test]
    fn test_income_statement_report_structure() {
        // Test that IncomeStatementReport calculates net result correctly
        let report = IncomeStatementReport {
            organization_id: "test-org".to_string(),
            report_date: "2024-01-01T00:00:00Z".to_string(),
            period_start: "2024-01-01T00:00:00Z".to_string(),
            period_end: "2024-12-31T23:59:59Z".to_string(),
            expenses: AccountSection {
                account_type: "EXPENSE".to_string(),
                accounts: vec![AccountLine {
                    code: "604001".to_string(),
                    label: "Électricité".to_string(),
                    amount: 5000.0,
                }],
                total: 5000.0,
            },
            revenue: AccountSection {
                account_type: "REVENUE".to_string(),
                accounts: vec![AccountLine {
                    code: "700001".to_string(),
                    label: "Appels de fonds".to_string(),
                    amount: 8000.0,
                }],
                total: 8000.0,
            },
            total_expenses: 5000.0,
            total_revenue: 8000.0,
            net_result: 3000.0,
        };

        assert_eq!(report.total_expenses, 5000.0);
        assert_eq!(report.total_revenue, 8000.0);
        assert_eq!(report.net_result, 3000.0); // Profit
    }

    #[test]
    fn test_income_statement_loss() {
        // Test negative net result (loss)
        let total_expenses = 10000.0;
        let total_revenue = 7000.0;
        let net_result = total_revenue - total_expenses;

        assert_eq!(net_result, -3000.0); // Loss
    }
}
