use crate::application::dto::PageRequest;
use crate::application::error::AppError;
use crate::domain::entities::{Budget, BudgetStatus};
use async_trait::async_trait;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Repository trait for Budget persistence
#[async_trait]
pub trait BudgetRepository: Send + Sync {
    /// Create a new budget
    async fn create(&self, budget: &Budget) -> Result<Budget, AppError>;

    /// Find budget by ID
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Budget>, AppError>;

    /// Find budget by building and fiscal year (should be unique)
    async fn find_by_building_and_fiscal_year(
        &self,
        building_id: Uuid,
        fiscal_year: i32,
    ) -> Result<Option<Budget>, AppError>;

    /// Find all budgets for a building
    async fn find_by_building(&self, building_id: Uuid) -> Result<Vec<Budget>, AppError>;

    /// Find active budget for a building (status = Approved, most recent fiscal year)
    async fn find_active_by_building(&self, building_id: Uuid) -> Result<Option<Budget>, AppError>;

    /// Find budgets by fiscal year across all buildings in organization
    async fn find_by_fiscal_year(
        &self,
        organization_id: Uuid,
        fiscal_year: i32,
    ) -> Result<Vec<Budget>, AppError>;

    /// Find budgets by status
    async fn find_by_status(
        &self,
        organization_id: Uuid,
        status: BudgetStatus,
    ) -> Result<Vec<Budget>, AppError>;

    /// Find all budgets paginated
    async fn find_all_paginated(
        &self,
        page_request: &PageRequest,
        organization_id: Option<Uuid>,
        building_id: Option<Uuid>,
        status: Option<BudgetStatus>,
    ) -> Result<(Vec<Budget>, i64), AppError>;

    /// Update existing budget
    async fn update(&self, budget: &Budget) -> Result<Budget, AppError>;

    /// Delete budget by ID
    async fn delete(&self, id: Uuid) -> Result<bool, AppError>;

    /// Get budget statistics for dashboard
    async fn get_stats(&self, organization_id: Uuid) -> Result<BudgetStatsResponse, AppError>;

    /// Get budget variance analysis (budget vs actual expenses)
    async fn get_variance(
        &self,
        budget_id: Uuid,
    ) -> Result<Option<BudgetVarianceResponse>, AppError>;
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
    // #661 — moyennes de montants : Decimal comme le reste du PCMN.
    #[serde(with = "rust_decimal::serde::float")]
    pub average_total_budget: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub average_monthly_provision: Decimal,
}

/// Variance analysis response
///
/// Issue #661 — tous les montants sont en `Decimal` : ce sont des charges de
/// copropriété (PCMN), et l'ADR-0008 §A n'accorde aucun carve-out `f64` à un
/// montant. Les `*_pct` suivent, parce qu'ils alimentent le seuil métier
/// `has_overruns` (dépassement > 10%) — un pourcentage comparé à un seuil
/// n'est pas un pourcentage d'affichage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetVarianceResponse {
    pub budget_id: Uuid,
    pub fiscal_year: i32,
    pub building_id: Uuid,
    #[serde(with = "rust_decimal::serde::float")]
    pub budgeted_ordinary: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub budgeted_extraordinary: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub budgeted_total: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub actual_ordinary: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub actual_extraordinary: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub actual_total: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub variance_ordinary: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub variance_extraordinary: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub variance_total: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub variance_ordinary_pct: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub variance_extraordinary_pct: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub variance_total_pct: Decimal,
    pub has_overruns: bool,
    pub overrun_categories: Vec<String>,
    pub months_elapsed: i32,
    #[serde(with = "rust_decimal::serde::float")]
    pub projected_year_end_total: Decimal,
}
