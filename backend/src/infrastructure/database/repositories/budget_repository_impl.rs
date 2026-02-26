use crate::application::dto::PageRequest;
use crate::application::ports::{BudgetRepository, BudgetStatsResponse, BudgetVarianceResponse};
use crate::domain::entities::{Budget, BudgetStatus, ExpenseCategory};
use async_trait::async_trait;
use chrono::Datelike;
use sqlx::postgres::PgRow;
use sqlx::{PgPool, Row};
use uuid::Uuid;

pub struct PostgresBudgetRepository {
    pool: PgPool,
}

/// Budget SELECT columns with casts for enums and decimals
const BUDGET_COLUMNS: &str = r#"
    id, organization_id, building_id, fiscal_year,
    ordinary_budget::float8 as ordinary_budget,
    extraordinary_budget::float8 as extraordinary_budget,
    total_budget::float8 as total_budget,
    status::text as status_text,
    submitted_date, approved_date, approved_by_meeting_id,
    monthly_provision_amount::float8 as monthly_provision_amount,
    notes, created_at, updated_at
"#;

impl PostgresBudgetRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Helper: Convert database row to Budget entity
    fn row_to_budget(&self, row: PgRow) -> Budget {
        let status_str: String = row.get("status_text");
        let status = match status_str.as_str() {
            "draft" => BudgetStatus::Draft,
            "submitted" => BudgetStatus::Submitted,
            "approved" => BudgetStatus::Approved,
            "rejected" => BudgetStatus::Rejected,
            "archived" => BudgetStatus::Archived,
            _ => BudgetStatus::Draft,
        };

        Budget {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            building_id: row.get("building_id"),
            fiscal_year: row.get("fiscal_year"),
            ordinary_budget: row.get("ordinary_budget"),
            extraordinary_budget: row.get("extraordinary_budget"),
            total_budget: row.get("total_budget"),
            status,
            submitted_date: row.get("submitted_date"),
            approved_date: row.get("approved_date"),
            approved_by_meeting_id: row.get("approved_by_meeting_id"),
            monthly_provision_amount: row.get("monthly_provision_amount"),
            notes: row.get("notes"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }
}

#[async_trait]
impl BudgetRepository for PostgresBudgetRepository {
    async fn create(&self, budget: &Budget) -> Result<Budget, String> {
        let status_str = match budget.status {
            BudgetStatus::Draft => "draft",
            BudgetStatus::Submitted => "submitted",
            BudgetStatus::Approved => "approved",
            BudgetStatus::Rejected => "rejected",
            BudgetStatus::Archived => "archived",
        };

        let row = sqlx::query(
            r#"
            INSERT INTO budgets (
                id, organization_id, building_id, fiscal_year,
                ordinary_budget, extraordinary_budget, total_budget,
                status, submitted_date, approved_date, approved_by_meeting_id,
                monthly_provision_amount, notes,
                created_at, updated_at
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8::budget_status,
                $9, $10, $11, $12, $13, $14, $15
            )
            RETURNING id, organization_id, building_id, fiscal_year,
                ordinary_budget::float8 as ordinary_budget,
                extraordinary_budget::float8 as extraordinary_budget,
                total_budget::float8 as total_budget,
                status::text as status_text,
                submitted_date, approved_date, approved_by_meeting_id,
                monthly_provision_amount::float8 as monthly_provision_amount,
                notes, created_at, updated_at
            "#,
        )
        .bind(budget.id)
        .bind(budget.organization_id)
        .bind(budget.building_id)
        .bind(budget.fiscal_year)
        .bind(budget.ordinary_budget)
        .bind(budget.extraordinary_budget)
        .bind(budget.total_budget)
        .bind(status_str)
        .bind(budget.submitted_date)
        .bind(budget.approved_date)
        .bind(budget.approved_by_meeting_id)
        .bind(budget.monthly_provision_amount)
        .bind(&budget.notes)
        .bind(budget.created_at)
        .bind(budget.updated_at)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to create budget: {}", e))?;

        Ok(self.row_to_budget(row))
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Budget>, String> {
        let sql = format!("SELECT {} FROM budgets WHERE id = $1", BUDGET_COLUMNS);
        let result = sqlx::query(&sql)
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| format!("Failed to find budget: {}", e))?;

        Ok(result.map(|row| self.row_to_budget(row)))
    }

    async fn find_by_building_and_fiscal_year(
        &self,
        building_id: Uuid,
        fiscal_year: i32,
    ) -> Result<Option<Budget>, String> {
        let sql = format!(
            "SELECT {} FROM budgets WHERE building_id = $1 AND fiscal_year = $2",
            BUDGET_COLUMNS
        );
        let result = sqlx::query(&sql)
            .bind(building_id)
            .bind(fiscal_year)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| format!("Failed to find budget: {}", e))?;

        Ok(result.map(|row| self.row_to_budget(row)))
    }

    async fn find_by_building(&self, building_id: Uuid) -> Result<Vec<Budget>, String> {
        let sql = format!(
            "SELECT {} FROM budgets WHERE building_id = $1 ORDER BY fiscal_year DESC",
            BUDGET_COLUMNS
        );
        let rows = sqlx::query(&sql)
            .bind(building_id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| format!("Failed to find budgets: {}", e))?;

        Ok(rows
            .into_iter()
            .map(|row| self.row_to_budget(row))
            .collect())
    }

    async fn find_active_by_building(&self, building_id: Uuid) -> Result<Option<Budget>, String> {
        let sql = format!("SELECT {} FROM budgets WHERE building_id = $1 AND status = 'approved' ORDER BY fiscal_year DESC LIMIT 1", BUDGET_COLUMNS);
        let result = sqlx::query(&sql)
            .bind(building_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| format!("Failed to find active budget: {}", e))?;

        Ok(result.map(|row| self.row_to_budget(row)))
    }

    async fn find_by_fiscal_year(
        &self,
        organization_id: Uuid,
        fiscal_year: i32,
    ) -> Result<Vec<Budget>, String> {
        let sql = format!("SELECT {} FROM budgets WHERE organization_id = $1 AND fiscal_year = $2 ORDER BY created_at DESC", BUDGET_COLUMNS);
        let rows = sqlx::query(&sql)
            .bind(organization_id)
            .bind(fiscal_year)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| format!("Failed to find budgets: {}", e))?;

        Ok(rows
            .into_iter()
            .map(|row| self.row_to_budget(row))
            .collect())
    }

    async fn find_by_status(
        &self,
        organization_id: Uuid,
        status: BudgetStatus,
    ) -> Result<Vec<Budget>, String> {
        let status_str = match status {
            BudgetStatus::Draft => "draft",
            BudgetStatus::Submitted => "submitted",
            BudgetStatus::Approved => "approved",
            BudgetStatus::Rejected => "rejected",
            BudgetStatus::Archived => "archived",
        };

        let sql = format!("SELECT {} FROM budgets WHERE organization_id = $1 AND status = $2::budget_status ORDER BY created_at DESC", BUDGET_COLUMNS);
        let rows = sqlx::query(&sql)
            .bind(organization_id)
            .bind(status_str)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| format!("Failed to find budgets: {}", e))?;

        Ok(rows
            .into_iter()
            .map(|row| self.row_to_budget(row))
            .collect())
    }

    async fn find_all_paginated(
        &self,
        page_request: &PageRequest,
        organization_id: Option<Uuid>,
        building_id: Option<Uuid>,
        status: Option<BudgetStatus>,
    ) -> Result<(Vec<Budget>, i64), String> {
        let offset = (page_request.page - 1) * page_request.per_page;

        // Build dynamic query
        let mut query = format!("SELECT {} FROM budgets WHERE 1=1", BUDGET_COLUMNS);
        let mut count_query = String::from("SELECT COUNT(*) as count FROM budgets WHERE 1=1");

        let mut bind_index = 1;
        let mut bindings: Vec<String> = Vec::new();

        if let Some(org_id) = organization_id {
            query.push_str(&format!(" AND organization_id = ${}::uuid", bind_index));
            count_query.push_str(&format!(" AND organization_id = ${}::uuid", bind_index));
            bindings.push(org_id.to_string());
            bind_index += 1;
        }

        if let Some(bldg_id) = building_id {
            query.push_str(&format!(" AND building_id = ${}::uuid", bind_index));
            count_query.push_str(&format!(" AND building_id = ${}::uuid", bind_index));
            bindings.push(bldg_id.to_string());
            bind_index += 1;
        }

        if let Some(s) = status {
            let status_str = match s {
                BudgetStatus::Draft => "draft",
                BudgetStatus::Submitted => "submitted",
                BudgetStatus::Approved => "approved",
                BudgetStatus::Rejected => "rejected",
                BudgetStatus::Archived => "archived",
            };
            query.push_str(&format!(" AND status = ${}::budget_status", bind_index));
            count_query.push_str(&format!(" AND status = ${}::budget_status", bind_index));
            bindings.push(status_str.to_string());
            bind_index += 1;
        }

        query.push_str(" ORDER BY fiscal_year DESC, created_at DESC");
        query.push_str(&format!(
            " LIMIT ${} OFFSET ${}",
            bind_index,
            bind_index + 1
        ));

        // Execute count query
        let mut count_q = sqlx::query(&count_query);
        for binding in &bindings {
            count_q = count_q.bind(binding);
        }
        let count_row = count_q
            .fetch_one(&self.pool)
            .await
            .map_err(|e| format!("Failed to count budgets: {}", e))?;
        let total: i64 = count_row.get("count");

        // Execute main query
        let mut main_q = sqlx::query(&query);
        for binding in &bindings {
            main_q = main_q.bind(binding);
        }
        main_q = main_q.bind(page_request.per_page as i64);
        main_q = main_q.bind(offset as i64);

        let rows = main_q
            .fetch_all(&self.pool)
            .await
            .map_err(|e| format!("Failed to fetch budgets: {}", e))?;

        let budgets = rows
            .into_iter()
            .map(|row| self.row_to_budget(row))
            .collect();

        Ok((budgets, total))
    }

    async fn update(&self, budget: &Budget) -> Result<Budget, String> {
        let status_str = match budget.status {
            BudgetStatus::Draft => "draft",
            BudgetStatus::Submitted => "submitted",
            BudgetStatus::Approved => "approved",
            BudgetStatus::Rejected => "rejected",
            BudgetStatus::Archived => "archived",
        };

        let row = sqlx::query(
            r#"
            UPDATE budgets SET
                ordinary_budget = $2,
                extraordinary_budget = $3,
                total_budget = $4,
                status = $5::budget_status,
                submitted_date = $6,
                approved_date = $7,
                approved_by_meeting_id = $8,
                monthly_provision_amount = $9,
                notes = $10,
                updated_at = $11
            WHERE id = $1
            RETURNING id, organization_id, building_id, fiscal_year,
                ordinary_budget::float8 as ordinary_budget,
                extraordinary_budget::float8 as extraordinary_budget,
                total_budget::float8 as total_budget,
                status::text as status_text,
                submitted_date, approved_date, approved_by_meeting_id,
                monthly_provision_amount::float8 as monthly_provision_amount,
                notes, created_at, updated_at
            "#,
        )
        .bind(budget.id)
        .bind(budget.ordinary_budget)
        .bind(budget.extraordinary_budget)
        .bind(budget.total_budget)
        .bind(status_str)
        .bind(budget.submitted_date)
        .bind(budget.approved_date)
        .bind(budget.approved_by_meeting_id)
        .bind(budget.monthly_provision_amount)
        .bind(&budget.notes)
        .bind(budget.updated_at)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to update budget: {}", e))?;

        Ok(self.row_to_budget(row))
    }

    async fn delete(&self, id: Uuid) -> Result<bool, String> {
        let result = sqlx::query("DELETE FROM budgets WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("Failed to delete budget: {}", e))?;

        Ok(result.rows_affected() > 0)
    }

    async fn get_stats(&self, organization_id: Uuid) -> Result<BudgetStatsResponse, String> {
        let row = sqlx::query(
            r#"
            SELECT
                COUNT(*) as total_budgets,
                COUNT(*) FILTER (WHERE status = 'draft') as draft_count,
                COUNT(*) FILTER (WHERE status = 'submitted') as submitted_count,
                COUNT(*) FILTER (WHERE status = 'approved') as approved_count,
                COUNT(*) FILTER (WHERE status = 'rejected') as rejected_count,
                COUNT(*) FILTER (WHERE status = 'archived') as archived_count,
                COALESCE(AVG(total_budget), 0)::float8 as average_total_budget,
                COALESCE(AVG(monthly_provision_amount), 0)::float8 as average_monthly_provision
            FROM budgets
            WHERE organization_id = $1
            "#,
        )
        .bind(organization_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to get budget stats: {}", e))?;

        Ok(BudgetStatsResponse {
            total_budgets: row.get("total_budgets"),
            draft_count: row.get("draft_count"),
            submitted_count: row.get("submitted_count"),
            approved_count: row.get("approved_count"),
            rejected_count: row.get("rejected_count"),
            archived_count: row.get("archived_count"),
            average_total_budget: row.get("average_total_budget"),
            average_monthly_provision: row.get("average_monthly_provision"),
        })
    }

    async fn get_variance(
        &self,
        budget_id: Uuid,
    ) -> Result<Option<BudgetVarianceResponse>, String> {
        // First, get the budget
        let budget = match self.find_by_id(budget_id).await? {
            Some(b) => b,
            None => return Ok(None),
        };

        // Get actual expenses for this budget's fiscal year and building
        let fiscal_year_start = format!("{}-01-01", budget.fiscal_year);
        let fiscal_year_end = format!("{}-12-31", budget.fiscal_year);

        let expense_rows = sqlx::query(
            r#"
            SELECT
                category,
                COALESCE(SUM(amount), 0) as total_amount
            FROM expenses
            WHERE building_id = $1
              AND expense_date >= $2::date
              AND expense_date <= $3::date
              AND payment_status = 'paid'
            GROUP BY category
            "#,
        )
        .bind(budget.building_id)
        .bind(fiscal_year_start)
        .bind(fiscal_year_end)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to get expenses: {}", e))?;

        let mut actual_ordinary = 0.0;
        let mut actual_extraordinary = 0.0;
        let mut overrun_categories = Vec::new();

        for row in expense_rows {
            let category_str: String = row.get("category");
            let amount: f64 = row.get("total_amount");

            let category = match category_str.as_str() {
                "utilities" => ExpenseCategory::Utilities,
                "maintenance" => ExpenseCategory::Maintenance,
                "repairs" => ExpenseCategory::Repairs,
                "insurance" => ExpenseCategory::Insurance,
                "cleaning" => ExpenseCategory::Cleaning,
                "administration" => ExpenseCategory::Administration,
                "works" => ExpenseCategory::Works,
                _ => ExpenseCategory::Other,
            };

            match category {
                ExpenseCategory::Works => actual_extraordinary += amount,
                _ => actual_ordinary += amount,
            }
        }

        let actual_total = actual_ordinary + actual_extraordinary;

        // Calculate variances
        let variance_ordinary = budget.ordinary_budget - actual_ordinary;
        let variance_extraordinary = budget.extraordinary_budget - actual_extraordinary;
        let variance_total = budget.total_budget - actual_total;

        // Calculate variance percentages
        let variance_ordinary_pct = if budget.ordinary_budget > 0.0 {
            (variance_ordinary / budget.ordinary_budget) * 100.0
        } else {
            0.0
        };

        let variance_extraordinary_pct = if budget.extraordinary_budget > 0.0 {
            (variance_extraordinary / budget.extraordinary_budget) * 100.0
        } else {
            0.0
        };

        let variance_total_pct = if budget.total_budget > 0.0 {
            (variance_total / budget.total_budget) * 100.0
        } else {
            0.0
        };

        // Check for overruns (> 10%)
        let has_overruns = variance_ordinary_pct < -10.0
            || variance_extraordinary_pct < -10.0
            || variance_total_pct < -10.0;

        if variance_ordinary_pct < -10.0 {
            overrun_categories.push("Ordinary charges".to_string());
        }
        if variance_extraordinary_pct < -10.0 {
            overrun_categories.push("Extraordinary charges".to_string());
        }

        // Calculate months elapsed (simplified - uses current month)
        let now = chrono::Utc::now();
        let months_elapsed = if now.year() == budget.fiscal_year {
            now.month() as i32
        } else if now.year() > budget.fiscal_year {
            12
        } else {
            0
        };

        // Project year-end total (linear projection)
        let projected_year_end_total = if months_elapsed > 0 {
            (actual_total / months_elapsed as f64) * 12.0
        } else {
            0.0
        };

        Ok(Some(BudgetVarianceResponse {
            budget_id: budget.id,
            fiscal_year: budget.fiscal_year,
            building_id: budget.building_id,
            budgeted_ordinary: budget.ordinary_budget,
            budgeted_extraordinary: budget.extraordinary_budget,
            budgeted_total: budget.total_budget,
            actual_ordinary,
            actual_extraordinary,
            actual_total,
            variance_ordinary,
            variance_extraordinary,
            variance_total,
            variance_ordinary_pct,
            variance_extraordinary_pct,
            variance_total_pct,
            has_overruns,
            overrun_categories,
            months_elapsed,
            projected_year_end_total,
        }))
    }
}
