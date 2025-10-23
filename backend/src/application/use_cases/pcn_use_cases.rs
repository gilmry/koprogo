use crate::application::dto::{PcnReportLineDto, PcnReportRequest, PcnReportResponse};
use crate::application::ports::ExpenseRepository;
use crate::domain::services::PcnMapper;
use chrono::Utc;
use std::sync::Arc;

pub struct PcnUseCases {
    expense_repo: Arc<dyn ExpenseRepository>,
}

impl PcnUseCases {
    pub fn new(expense_repo: Arc<dyn ExpenseRepository>) -> Self {
        Self { expense_repo }
    }

    /// Generate PCN report for a building
    /// Aggregates expenses by PCN account and returns structured report
    pub async fn generate_report(
        &self,
        request: PcnReportRequest,
    ) -> Result<PcnReportResponse, String> {
        // Fetch expenses for the building
        let all_expenses = self
            .expense_repo
            .find_by_building(request.building_id)
            .await?;

        // Filter by date range if provided
        let expenses: Vec<_> = all_expenses
            .into_iter()
            .filter(|e| {
                let after_start = request
                    .start_date
                    .map(|start| e.expense_date >= start)
                    .unwrap_or(true);
                let before_end = request
                    .end_date
                    .map(|end| e.expense_date <= end)
                    .unwrap_or(true);
                after_start && before_end
            })
            .collect();

        // Generate PCN report using domain service
        let report_lines = PcnMapper::generate_report(&expenses);

        // Calculate totals
        let total_amount: f64 = report_lines.iter().map(|l| l.total_amount).sum();
        let total_entries: usize = report_lines.iter().map(|l| l.entry_count).sum();

        // Convert to DTOs
        let lines: Vec<PcnReportLineDto> = report_lines
            .into_iter()
            .map(PcnReportLineDto::from)
            .collect();

        Ok(PcnReportResponse {
            building_id: request.building_id,
            generated_at: Utc::now(),
            period_start: request.start_date,
            period_end: request.end_date,
            lines,
            total_amount,
            total_entries,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::dto::{ExpenseFilters, PageRequest};
    use crate::application::ports::ExpenseRepository;
    use crate::domain::entities::{Expense, ExpenseCategory};
    use async_trait::async_trait;
    use chrono::Utc;
    use uuid::Uuid;

    struct MockExpenseRepository {
        expenses: Vec<Expense>,
    }

    #[async_trait]
    impl ExpenseRepository for MockExpenseRepository {
        async fn create(&self, _expense: &Expense) -> Result<Expense, String> {
            unimplemented!()
        }

        async fn find_by_id(&self, _id: Uuid) -> Result<Option<Expense>, String> {
            unimplemented!()
        }

        async fn find_by_building(&self, building_id: Uuid) -> Result<Vec<Expense>, String> {
            Ok(self
                .expenses
                .iter()
                .filter(|e| e.building_id == building_id)
                .cloned()
                .collect())
        }

        async fn update(&self, _expense: &Expense) -> Result<Expense, String> {
            unimplemented!()
        }

        async fn delete(&self, _id: Uuid) -> Result<bool, String> {
            unimplemented!()
        }

        async fn find_all_paginated(
            &self,
            _page_request: &PageRequest,
            _filters: &ExpenseFilters,
        ) -> Result<(Vec<Expense>, i64), String> {
            unimplemented!()
        }
    }

    fn create_test_expense(
        building_id: Uuid,
        category: ExpenseCategory,
        amount: f64,
    ) -> Expense {
        Expense::new(
            building_id,
            category,
            "Test expense".to_string(),
            amount,
            Utc::now(),
            Some("Supplier".to_string()),
            Some("INV-001".to_string()),
        )
        .unwrap()
    }

    #[tokio::test]
    async fn test_generate_report_success() {
        let building_id = Uuid::new_v4();
        let expenses = vec![
            create_test_expense(building_id, ExpenseCategory::Maintenance, 100.0),
            create_test_expense(building_id, ExpenseCategory::Maintenance, 150.0),
            create_test_expense(building_id, ExpenseCategory::Utilities, 50.0),
        ];

        let repo = Arc::new(MockExpenseRepository { expenses });
        let use_cases = PcnUseCases::new(repo);

        let request = PcnReportRequest {
            building_id,
            start_date: None,
            end_date: None,
        };

        let result = use_cases.generate_report(request).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.building_id, building_id);
        assert_eq!(response.lines.len(), 2); // Maintenance + Utilities
        assert_eq!(response.total_amount, 300.0);
        assert_eq!(response.total_entries, 3);

        // Verify Maintenance account (611)
        let maintenance = response
            .lines
            .iter()
            .find(|l| l.account_code == "611")
            .unwrap();
        assert_eq!(maintenance.total_amount, 250.0);
        assert_eq!(maintenance.entry_count, 2);
    }

    #[tokio::test]
    async fn test_generate_report_empty() {
        let building_id = Uuid::new_v4();
        let repo = Arc::new(MockExpenseRepository {
            expenses: vec![],
        });
        let use_cases = PcnUseCases::new(repo);

        let request = PcnReportRequest {
            building_id,
            start_date: None,
            end_date: None,
        };

        let result = use_cases.generate_report(request).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.lines.len(), 0);
        assert_eq!(response.total_amount, 0.0);
        assert_eq!(response.total_entries, 0);
    }
}
