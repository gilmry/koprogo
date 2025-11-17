use crate::application::ports::CallForFundsRepository;
use crate::domain::entities::{CallForFunds, CallForFundsStatus, ContributionType};
use crate::infrastructure::database::pool::DbPool;
use async_trait::async_trait;
use sqlx::Row;
use uuid::Uuid;

pub struct PostgresCallForFundsRepository {
    pool: DbPool,
}

impl PostgresCallForFundsRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl CallForFundsRepository for PostgresCallForFundsRepository {
    async fn create(&self, call_for_funds: &CallForFunds) -> Result<CallForFunds, String> {
        let status_str = match call_for_funds.status {
            CallForFundsStatus::Draft => "draft",
            CallForFundsStatus::Sent => "sent",
            CallForFundsStatus::Partial => "partial",
            CallForFundsStatus::Completed => "completed",
            CallForFundsStatus::Cancelled => "cancelled",
        };

        let contribution_type_str = match call_for_funds.contribution_type {
            ContributionType::Regular => "regular",
            ContributionType::Extraordinary => "extraordinary",
            ContributionType::Advance => "advance",
            ContributionType::Adjustment => "adjustment",
        };

        sqlx::query(
            r#"
            INSERT INTO call_for_funds (
                id, organization_id, building_id, title, description,
                total_amount, contribution_type, call_date, due_date,
                sent_date, status, account_code, notes, created_at,
                updated_at, created_by
            )
            VALUES ($1, $2, $3, $4, $5, $6, CAST($7 AS contribution_type),
                    $8, $9, $10, CAST($11 AS call_for_funds_status),
                    $12, $13, $14, $15, $16)
            "#,
        )
        .bind(call_for_funds.id)
        .bind(call_for_funds.organization_id)
        .bind(call_for_funds.building_id)
        .bind(&call_for_funds.title)
        .bind(&call_for_funds.description)
        .bind(call_for_funds.total_amount)
        .bind(contribution_type_str)
        .bind(call_for_funds.call_date)
        .bind(call_for_funds.due_date)
        .bind(call_for_funds.sent_date)
        .bind(status_str)
        .bind(&call_for_funds.account_code)
        .bind(&call_for_funds.notes)
        .bind(call_for_funds.created_at)
        .bind(call_for_funds.updated_at)
        .bind(call_for_funds.created_by)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(call_for_funds.clone())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<CallForFunds>, String> {
        let row = sqlx::query(
            r#"
            SELECT id, organization_id, building_id, title, description,
                   total_amount, contribution_type::text AS contribution_type,
                   call_date, due_date, sent_date,
                   status::text AS status, account_code, notes,
                   created_at, updated_at, created_by
            FROM call_for_funds
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(row.map(|row| self.map_row_to_entity(&row)))
    }

    async fn find_by_building(&self, building_id: Uuid) -> Result<Vec<CallForFunds>, String> {
        let rows = sqlx::query(
            r#"
            SELECT id, organization_id, building_id, title, description,
                   total_amount, contribution_type::text AS contribution_type,
                   call_date, due_date, sent_date,
                   status::text AS status, account_code, notes,
                   created_at, updated_at, created_by
            FROM call_for_funds
            WHERE building_id = $1
            ORDER BY call_date DESC
            "#,
        )
        .bind(building_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(rows.iter().map(|row| self.map_row_to_entity(row)).collect())
    }

    async fn find_by_organization(
        &self,
        organization_id: Uuid,
    ) -> Result<Vec<CallForFunds>, String> {
        let rows = sqlx::query(
            r#"
            SELECT id, organization_id, building_id, title, description,
                   total_amount, contribution_type::text AS contribution_type,
                   call_date, due_date, sent_date,
                   status::text AS status, account_code, notes,
                   created_at, updated_at, created_by
            FROM call_for_funds
            WHERE organization_id = $1
            ORDER BY call_date DESC
            "#,
        )
        .bind(organization_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(rows.iter().map(|row| self.map_row_to_entity(row)).collect())
    }

    async fn update(&self, call_for_funds: &CallForFunds) -> Result<CallForFunds, String> {
        let status_str = match call_for_funds.status {
            CallForFundsStatus::Draft => "draft",
            CallForFundsStatus::Sent => "sent",
            CallForFundsStatus::Partial => "partial",
            CallForFundsStatus::Completed => "completed",
            CallForFundsStatus::Cancelled => "cancelled",
        };

        let contribution_type_str = match call_for_funds.contribution_type {
            ContributionType::Regular => "regular",
            ContributionType::Extraordinary => "extraordinary",
            ContributionType::Advance => "advance",
            ContributionType::Adjustment => "adjustment",
        };

        sqlx::query(
            r#"
            UPDATE call_for_funds
            SET title = $2,
                description = $3,
                total_amount = $4,
                contribution_type = CAST($5 AS contribution_type),
                call_date = $6,
                due_date = $7,
                sent_date = $8,
                status = CAST($9 AS call_for_funds_status),
                account_code = $10,
                notes = $11,
                updated_at = $12
            WHERE id = $1
            "#,
        )
        .bind(call_for_funds.id)
        .bind(&call_for_funds.title)
        .bind(&call_for_funds.description)
        .bind(call_for_funds.total_amount)
        .bind(contribution_type_str)
        .bind(call_for_funds.call_date)
        .bind(call_for_funds.due_date)
        .bind(call_for_funds.sent_date)
        .bind(status_str)
        .bind(&call_for_funds.account_code)
        .bind(&call_for_funds.notes)
        .bind(call_for_funds.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(call_for_funds.clone())
    }

    async fn delete(&self, id: Uuid) -> Result<bool, String> {
        let result = sqlx::query("DELETE FROM call_for_funds WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("Database error: {}", e))?;

        Ok(result.rows_affected() > 0)
    }

    async fn find_overdue(&self) -> Result<Vec<CallForFunds>, String> {
        let rows = sqlx::query(
            r#"
            SELECT id, organization_id, building_id, title, description,
                   total_amount, contribution_type::text AS contribution_type,
                   call_date, due_date, sent_date,
                   status::text AS status, account_code, notes,
                   created_at, updated_at, created_by
            FROM call_for_funds
            WHERE due_date < NOW()
              AND status NOT IN ('completed', 'cancelled')
            ORDER BY due_date ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(rows.iter().map(|row| self.map_row_to_entity(row)).collect())
    }
}

impl PostgresCallForFundsRepository {
    fn map_row_to_entity(&self, row: &sqlx::postgres::PgRow) -> CallForFunds {
        let status_str: String = row.get("status");
        let status = match status_str.as_str() {
            "sent" => CallForFundsStatus::Sent,
            "partial" => CallForFundsStatus::Partial,
            "completed" => CallForFundsStatus::Completed,
            "cancelled" => CallForFundsStatus::Cancelled,
            _ => CallForFundsStatus::Draft,
        };

        let contribution_type_str: String = row.get("contribution_type");
        let contribution_type = match contribution_type_str.as_str() {
            "extraordinary" => ContributionType::Extraordinary,
            "advance" => ContributionType::Advance,
            "adjustment" => ContributionType::Adjustment,
            _ => ContributionType::Regular,
        };

        CallForFunds {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            building_id: row.get("building_id"),
            title: row.get("title"),
            description: row.get("description"),
            total_amount: row.get("total_amount"),
            contribution_type,
            call_date: row.get("call_date"),
            due_date: row.get("due_date"),
            sent_date: row.try_get("sent_date").ok(),
            status,
            account_code: row.try_get("account_code").ok(),
            notes: row.try_get("notes").ok(),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            created_by: row.try_get("created_by").ok(),
        }
    }
}
