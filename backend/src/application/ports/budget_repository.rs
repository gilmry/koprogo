use crate::application::dto::PageRequest;
use crate::domain::entities::{Budget, BudgetStatus};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Repository trait for Budget persistence
#[async_trait]
pub trait BudgetRepository: Send + Sync {
    /// Create a new budget
    async fn create(&self, budget: &Budget) -> Result<Budget, String>;

    /// Find budget by ID
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Budget>, String>;

    /// Find budget by building and fiscal year (should be unique)
    async fn find_by_building_and_fiscal_year(
        &self,
        building_id: Uuid,
        fiscal_year: i32,
    ) -> Result<Option<Budget>, String>;

    /// Find all budgets for a building
    async fn find_by_building(&self, building_id: Uuid) -> Result<Vec<Budget>, String>;

    /// Find active budget for a building (status = Approved, most recent fiscal year)
    async fn find_active_by_building(&self, building_id: Uuid) -> Result<Option<Budget>, String>;

    /// Find budgets by fiscal year across all buildings in organization
    async fn find_by_fiscal_year(
        &self,
        organization_id: Uuid,
        fiscal_year: i32,
    ) -> Result<Vec<Budget>, String>;

    /// Find budgets by status
    async fn find_by_status(
        &self,
        organization_id: Uuid,
        status: BudgetStatus,
    ) -> Result<Vec<Budget>, String>;

    /// Find all budgets paginated
    async fn find_all_paginated(
        &self,
        page_request: &PageRequest,
        organization_id: Option<Uuid>,
        building_id: Option<Uuid>,
        status: Option<BudgetStatus>,
    ) -> Result<(Vec<Budget>, i64), String>;

    /// Update existing budget
    async fn update(&self, budget: &Budget) -> Result<Budget, String>;

    /// Delete budget by ID
    async fn delete(&self, id: Uuid) -> Result<bool, String>;

    /// Get budget statistics for dashboard
    async fn get_stats(&self, organization_id: Uuid) -> Result<BudgetStatsResponse, String>;

    /// Get budget variance analysis (budget vs actual expenses)
    async fn get_variance(
        &self,
        budget_id: Uuid,
    ) -> Result<Option<BudgetVarianceResponse>, String>;
}

/// Statistics response for budgets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetStatsResponse {
    pub total_budgets: i64,
    pub draft_count: i64,
    pub submitted_count: i64,
    pub approved_count: i64,
    pub rejected_count: i64,
    pub archived_count: i64,
    pub average_total_budget: f64,
    pub average_monthly_provision: f64,
}

/// Variance analysis response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetVarianceResponse {
    pub budget_id: Uuid,
    pub fiscal_year: i32,
    pub building_id: Uuid,
    pub budgeted_ordinary: f64,
    pub budgeted_extraordinary: f64,
    pub budgeted_total: f64,
    pub actual_ordinary: f64,
    pub actual_extraordinary: f64,
    pub actual_total: f64,
    pub variance_ordinary: f64,
    pub variance_extraordinary: f64,
    pub variance_total: f64,
    pub variance_ordinary_pct: f64,
    pub variance_extraordinary_pct: f64,
    pub variance_total_pct: f64,
    pub has_overruns: bool,
    pub overrun_categories: Vec<String>,
    pub months_elapsed: i32,
    pub projected_year_end_total: f64,
}
