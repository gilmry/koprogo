// Application Use Cases: Dashboard
//
// Business logic for dashboard statistics and recent transactions

use crate::application::dto::{
    AccountantDashboardStats, ExpenseFilters, PageRequest, RecentTransaction, TransactionType,
};
use crate::application::ports::{ExpenseRepository, OwnerContributionRepository};
use crate::domain::entities::ApprovalStatus;
use chrono::{Datelike, Timelike, Utc};
use std::sync::Arc;
use uuid::Uuid;

pub struct DashboardUseCases {
    expense_repo: Arc<dyn ExpenseRepository>,
    owner_contribution_repo: Arc<dyn OwnerContributionRepository>,
}

impl DashboardUseCases {
    pub fn new(
        expense_repo: Arc<dyn ExpenseRepository>,
        owner_contribution_repo: Arc<dyn OwnerContributionRepository>,
    ) -> Self {
        Self {
            expense_repo,
            owner_contribution_repo,
        }
    }

    /// Get accountant dashboard statistics
    pub async fn get_accountant_stats(
        &self,
        organization_id: Uuid,
    ) -> Result<AccountantDashboardStats, String> {
        // Get all expenses for the organization
        let filters = ExpenseFilters {
            organization_id: Some(organization_id),
            ..Default::default()
        };

        // Get all expenses (use large page size to get all)
        let page_request = PageRequest {
            page: 1,
            per_page: 10000, // Large enough to get all expenses
            sort_by: None,
            order: Default::default(),
        };

        let (all_expenses, _total) = self
            .expense_repo
            .find_all_paginated(&page_request, &filters)
            .await?;

        // Get current month start date
        let now = Utc::now();
        let current_month_start = Utc::now()
            .with_day(1)
            .and_then(|d| d.with_hour(0))
            .and_then(|d| d.with_minute(0))
            .and_then(|d| d.with_second(0))
            .unwrap_or(now);

        // Filter expenses for current month
        let current_month_expenses: Vec<_> = all_expenses
            .iter()
            .filter(|e| e.expense_date >= current_month_start)
            .collect();

        // Calculate total expenses for current month
        let total_expenses_current_month: f64 = current_month_expenses
            .iter()
            .map(|e| e.amount_incl_vat.unwrap_or(0.0))
            .sum();

        // Calculate paid expenses (status = Approved AND paid_date is set)
        let paid_expenses: Vec<_> = all_expenses
            .iter()
            .filter(|e| e.approval_status == ApprovalStatus::Approved && e.paid_date.is_some())
            .collect();

        let total_paid: f64 = paid_expenses
            .iter()
            .map(|e| e.amount_incl_vat.unwrap_or(0.0))
            .sum();

        // Calculate pending expenses (not paid)
        let pending_expenses: Vec<_> = all_expenses
            .iter()
            .filter(|e| e.paid_date.is_none())
            .collect();

        let total_pending: f64 = pending_expenses
            .iter()
            .map(|e| e.amount_incl_vat.unwrap_or(0.0))
            .sum();

        // Calculate percentages
        let total_all = total_paid + total_pending;
        let paid_percentage = if total_all > 0.0 {
            (total_paid / total_all) * 100.0
        } else {
            0.0
        };
        let pending_percentage = if total_all > 0.0 {
            (total_pending / total_all) * 100.0
        } else {
            0.0
        };

        // TODO: Calculate owners with overdue payments from payment_reminders
        // For now, return a placeholder
        let owners_with_overdue = 0;

        Ok(AccountantDashboardStats {
            total_expenses_current_month,
            total_paid,
            paid_percentage,
            total_pending,
            pending_percentage,
            owners_with_overdue,
        })
    }

    /// Get recent transactions for dashboard
    pub async fn get_recent_transactions(
        &self,
        organization_id: Uuid,
        limit: usize,
    ) -> Result<Vec<RecentTransaction>, String> {
        // Get all expenses for the organization
        let filters = ExpenseFilters {
            organization_id: Some(organization_id),
            ..Default::default()
        };

        let page_request = PageRequest {
            page: 1,
            per_page: 1000, // Get enough for sorting
            sort_by: None,
            order: Default::default(),
        };

        let (all_expenses, _total) = self
            .expense_repo
            .find_all_paginated(&page_request, &filters)
            .await?;

        // Get all owner contributions for the organization
        let all_contributions = self
            .owner_contribution_repo
            .find_by_organization(organization_id)
            .await?;

        // Convert expenses to transactions (OUTGOING = negative)
        let expense_transactions: Vec<RecentTransaction> = all_expenses
            .iter()
            .map(|expense| {
                let transaction_type = TransactionType::PaymentMade;
                let amount_value = expense.amount_incl_vat.unwrap_or(0.0);
                let amount = -amount_value; // Negative for expenses

                RecentTransaction {
                    id: expense.id,
                    transaction_type,
                    description: expense.description.clone(),
                    related_entity: expense.supplier.clone(),
                    amount,
                    date: expense.expense_date,
                }
            })
            .collect();

        // Convert owner contributions to transactions (INCOMING = positive)
        let contribution_transactions: Vec<RecentTransaction> = all_contributions
            .iter()
            .map(|contribution| {
                let transaction_type = TransactionType::PaymentReceived;
                let amount = contribution.amount; // Positive for revenue

                RecentTransaction {
                    id: contribution.id,
                    transaction_type,
                    description: contribution.description.clone(),
                    related_entity: Some("Copropriétaire".to_string()), // Could link to owner name if needed
                    amount,
                    date: contribution.contribution_date,
                }
            })
            .collect();

        // Merge both transaction types
        let mut all_transactions = Vec::new();
        all_transactions.extend(expense_transactions);
        all_transactions.extend(contribution_transactions);

        // Sort by date (most recent first)
        all_transactions.sort_by(|a, b| b.date.cmp(&a.date));

        // Take the most recent ones
        let recent_transactions: Vec<RecentTransaction> =
            all_transactions.into_iter().take(limit).collect();

        Ok(recent_transactions)
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_percentage_calculation() {
        let total_paid = 41270.0;
        let total_pending = 4580.0;
        let total = total_paid + total_pending;

        let paid_percentage = (total_paid / total) * 100.0;
        let pending_percentage = (total_pending / total) * 100.0;

        assert!((paid_percentage - 90.0_f64).abs() < 0.1);
        assert!((pending_percentage - 10.0_f64).abs() < 0.1);
    }

    #[test]
    fn test_percentage_with_zero_total() {
        let total_paid = 0.0;
        let total_pending = 0.0;
        let total = total_paid + total_pending;

        let paid_percentage = if total > 0.0 {
            (total_paid / total) * 100.0
        } else {
            0.0
        };

        assert_eq!(paid_percentage, 0.0);
    }
}
