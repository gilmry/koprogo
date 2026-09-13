use crate::domain::entities::{Budget, BudgetStatus};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Request pour créer un nouveau budget
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateBudgetRequest {
    #[serde(default)]
    pub organization_id: Uuid, // Will be overridden by JWT token
    pub building_id: Uuid,
    pub fiscal_year: i32,
    pub ordinary_budget: Decimal,
    pub extraordinary_budget: Decimal,
    pub notes: Option<String>,
}

/// Request pour mettre à jour un budget (Draft uniquement)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateBudgetRequest {
    pub ordinary_budget: Option<Decimal>,
    pub extraordinary_budget: Option<Decimal>,
    pub notes: Option<String>,
}

/// Response DTO pour un budget
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetResponse {
    pub id: Uuid,
    /// L'ACP dont ce budget est le budget (ADR-0045).
    pub acp_id: Uuid,
    /// Le syndic qui l'a préparé. Trace d'auteur, pas un droit d'accès.
    pub organization_id: Uuid,
    pub building_id: Uuid,
    pub fiscal_year: i32,
    pub ordinary_budget: Decimal,
    pub extraordinary_budget: Decimal,
    pub total_budget: Decimal,
    pub status: BudgetStatus,
    pub submitted_date: Option<DateTime<Utc>>,
    pub approved_date: Option<DateTime<Utc>>,
    pub approved_by_meeting_id: Option<Uuid>,
    pub monthly_provision_amount: Decimal,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    // Computed fields
    pub is_active: bool,
    pub is_editable: bool,
}

impl From<Budget> for BudgetResponse {
    fn from(budget: Budget) -> Self {
        let is_active = budget.is_active();
        let is_editable = budget.is_editable();

        Self {
            id: budget.id,
            acp_id: budget.acp_id,
            organization_id: budget.organization_id,
            building_id: budget.building_id,
            fiscal_year: budget.fiscal_year,
            ordinary_budget: budget.ordinary_budget,
            extraordinary_budget: budget.extraordinary_budget,
            total_budget: budget.total_budget,
            status: budget.status,
            submitted_date: budget.submitted_date,
            approved_date: budget.approved_date,
            approved_by_meeting_id: budget.approved_by_meeting_id,
            monthly_provision_amount: budget.monthly_provision_amount,
            notes: budget.notes,
            created_at: budget.created_at,
            updated_at: budget.updated_at,
            is_active,
            is_editable,
        }
    }
}

/// Response DTO pour variance analysis (budget vs actual)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetVarianceResponse {
    pub budget_id: Uuid,
    pub fiscal_year: i32,
    pub building_id: Uuid,

    // Budget prévu
    #[serde(with = "rust_decimal::serde::float")]
    pub budgeted_ordinary: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub budgeted_extraordinary: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub budgeted_total: Decimal,

    // Réalisé (YTD - Year To Date)
    #[serde(with = "rust_decimal::serde::float")]
    pub actual_ordinary: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub actual_extraordinary: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub actual_total: Decimal,

    // Variance (budget - actual)
    #[serde(with = "rust_decimal::serde::float")]
    pub variance_ordinary: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub variance_extraordinary: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub variance_total: Decimal,

    // Variance en % ((budget - actual) / budget * 100)
    #[serde(with = "rust_decimal::serde::float")]
    pub variance_ordinary_pct: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub variance_extraordinary_pct: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub variance_total_pct: Decimal,

    // Alertes
    pub has_overruns: bool,              // Dépassements > 10%
    pub overrun_categories: Vec<String>, // Catégories en dépassement
    pub months_elapsed: i32,             // Mois écoulés dans l'exercice
    #[serde(with = "rust_decimal::serde::float")]
    pub projected_year_end_total: Decimal, // Projection fin d'année
}

/// Response avec statistiques budgétaires
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
