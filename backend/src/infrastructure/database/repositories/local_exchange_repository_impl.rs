use crate::application::ports::LocalExchangeRepository;
use crate::domain::entities::{ExchangeStatus, ExchangeType, LocalExchange};
use crate::infrastructure::database::pool::DbPool;
use async_trait::async_trait;
use sqlx::Row;
use uuid::Uuid;

pub struct PostgresLocalExchangeRepository {
    pool: DbPool,
}

impl PostgresLocalExchangeRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl LocalExchangeRepository for PostgresLocalExchangeRepository {
    async fn create(&self, exchange: &LocalExchange) -> Result<LocalExchange, String> {
        sqlx::query(
            r#"
            INSERT INTO local_exchanges (
                id, building_id, provider_id, requester_id, exchange_type,
                title, description, credits, status, offered_at,
                requested_at, started_at, completed_at, cancelled_at,
                cancellation_reason, provider_rating, requester_rating,
                created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19)
            "#,
        )
        .bind(exchange.id)
        .bind(exchange.building_id)
        .bind(exchange.provider_id)
        .bind(exchange.requester_id)
        .bind(exchange.exchange_type.to_sql())
        .bind(&exchange.title)
        .bind(&exchange.description)
        .bind(exchange.credits)
        .bind(exchange.status.to_sql())
        .bind(exchange.offered_at)
        .bind(exchange.requested_at)
        .bind(exchange.started_at)
        .bind(exchange.completed_at)
        .bind(exchange.cancelled_at)
        .bind(&exchange.cancellation_reason)
        .bind(exchange.provider_rating)
        .bind(exchange.requester_rating)
        .bind(exchange.created_at)
        .bind(exchange.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(exchange.clone())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<LocalExchange>, String> {
        let row = sqlx::query(
            r#"
            SELECT
                id, building_id, provider_id, requester_id, exchange_type,
                title, description, credits, status, offered_at,
                requested_at, started_at, completed_at, cancelled_at,
                cancellation_reason, provider_rating, requester_rating,
                created_at, updated_at
            FROM local_exchanges
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(row.map(|row| map_row_to_exchange(&row)))
    }

    async fn find_by_building(&self, building_id: Uuid) -> Result<Vec<LocalExchange>, String> {
        let rows = sqlx::query(
            r#"
            SELECT
                id, building_id, provider_id, requester_id, exchange_type,
                title, description, credits, status, offered_at,
                requested_at, started_at, completed_at, cancelled_at,
                cancellation_reason, provider_rating, requester_rating,
                created_at, updated_at
            FROM local_exchanges
            WHERE building_id = $1
            ORDER BY offered_at DESC
            "#,
        )
        .bind(building_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(rows.iter().map(|row| map_row_to_exchange(row)).collect())
    }

    async fn find_by_building_and_status(
        &self,
        building_id: Uuid,
        status: &str,
    ) -> Result<Vec<LocalExchange>, String> {
        let rows = sqlx::query(
            r#"
            SELECT
                id, building_id, provider_id, requester_id, exchange_type,
                title, description, credits, status, offered_at,
                requested_at, started_at, completed_at, cancelled_at,
                cancellation_reason, provider_rating, requester_rating,
                created_at, updated_at
            FROM local_exchanges
            WHERE building_id = $1 AND status = $2
            ORDER BY offered_at DESC
            "#,
        )
        .bind(building_id)
        .bind(status)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(rows.iter().map(|row| map_row_to_exchange(row)).collect())
    }

    async fn find_by_provider(&self, provider_id: Uuid) -> Result<Vec<LocalExchange>, String> {
        let rows = sqlx::query(
            r#"
            SELECT
                id, building_id, provider_id, requester_id, exchange_type,
                title, description, credits, status, offered_at,
                requested_at, started_at, completed_at, cancelled_at,
                cancellation_reason, provider_rating, requester_rating,
                created_at, updated_at
            FROM local_exchanges
            WHERE provider_id = $1
            ORDER BY offered_at DESC
            "#,
        )
        .bind(provider_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(rows.iter().map(|row| map_row_to_exchange(row)).collect())
    }

    async fn find_by_requester(&self, requester_id: Uuid) -> Result<Vec<LocalExchange>, String> {
        let rows = sqlx::query(
            r#"
            SELECT
                id, building_id, provider_id, requester_id, exchange_type,
                title, description, credits, status, offered_at,
                requested_at, started_at, completed_at, cancelled_at,
                cancellation_reason, provider_rating, requester_rating,
                created_at, updated_at
            FROM local_exchanges
            WHERE requester_id = $1
            ORDER BY requested_at DESC
            "#,
        )
        .bind(requester_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(rows.iter().map(|row| map_row_to_exchange(row)).collect())
    }

    async fn find_by_owner(&self, owner_id: Uuid) -> Result<Vec<LocalExchange>, String> {
        let rows = sqlx::query(
            r#"
            SELECT
                id, building_id, provider_id, requester_id, exchange_type,
                title, description, credits, status, offered_at,
                requested_at, started_at, completed_at, cancelled_at,
                cancellation_reason, provider_rating, requester_rating,
                created_at, updated_at
            FROM local_exchanges
            WHERE provider_id = $1 OR requester_id = $1
            ORDER BY offered_at DESC
            "#,
        )
        .bind(owner_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(rows.iter().map(|row| map_row_to_exchange(row)).collect())
    }

    async fn find_active_by_building(
        &self,
        building_id: Uuid,
    ) -> Result<Vec<LocalExchange>, String> {
        let rows = sqlx::query(
            r#"
            SELECT
                id, building_id, provider_id, requester_id, exchange_type,
                title, description, credits, status, offered_at,
                requested_at, started_at, completed_at, cancelled_at,
                cancellation_reason, provider_rating, requester_rating,
                created_at, updated_at
            FROM local_exchanges
            WHERE building_id = $1
              AND status IN ('Offered', 'Requested', 'InProgress')
            ORDER BY offered_at DESC
            "#,
        )
        .bind(building_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(rows.iter().map(|row| map_row_to_exchange(row)).collect())
    }

    async fn find_available_by_building(
        &self,
        building_id: Uuid,
    ) -> Result<Vec<LocalExchange>, String> {
        let rows = sqlx::query(
            r#"
            SELECT
                id, building_id, provider_id, requester_id, exchange_type,
                title, description, credits, status, offered_at,
                requested_at, started_at, completed_at, cancelled_at,
                cancellation_reason, provider_rating, requester_rating,
                created_at, updated_at
            FROM local_exchanges
            WHERE building_id = $1 AND status = 'Offered'
            ORDER BY offered_at DESC
            "#,
        )
        .bind(building_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(rows.iter().map(|row| map_row_to_exchange(row)).collect())
    }

    async fn find_by_type(
        &self,
        building_id: Uuid,
        exchange_type: &str,
    ) -> Result<Vec<LocalExchange>, String> {
        let rows = sqlx::query(
            r#"
            SELECT
                id, building_id, provider_id, requester_id, exchange_type,
                title, description, credits, status, offered_at,
                requested_at, started_at, completed_at, cancelled_at,
                cancellation_reason, provider_rating, requester_rating,
                created_at, updated_at
            FROM local_exchanges
            WHERE building_id = $1 AND exchange_type = $2
            ORDER BY offered_at DESC
            "#,
        )
        .bind(building_id)
        .bind(exchange_type)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(rows.iter().map(|row| map_row_to_exchange(row)).collect())
    }

    async fn update(&self, exchange: &LocalExchange) -> Result<LocalExchange, String> {
        sqlx::query(
            r#"
            UPDATE local_exchanges
            SET
                building_id = $2,
                provider_id = $3,
                requester_id = $4,
                exchange_type = $5,
                title = $6,
                description = $7,
                credits = $8,
                status = $9,
                offered_at = $10,
                requested_at = $11,
                started_at = $12,
                completed_at = $13,
                cancelled_at = $14,
                cancellation_reason = $15,
                provider_rating = $16,
                requester_rating = $17,
                updated_at = $18
            WHERE id = $1
            "#,
        )
        .bind(exchange.id)
        .bind(exchange.building_id)
        .bind(exchange.provider_id)
        .bind(exchange.requester_id)
        .bind(exchange.exchange_type.to_sql())
        .bind(&exchange.title)
        .bind(&exchange.description)
        .bind(exchange.credits)
        .bind(exchange.status.to_sql())
        .bind(exchange.offered_at)
        .bind(exchange.requested_at)
        .bind(exchange.started_at)
        .bind(exchange.completed_at)
        .bind(exchange.cancelled_at)
        .bind(&exchange.cancellation_reason)
        .bind(exchange.provider_rating)
        .bind(exchange.requester_rating)
        .bind(exchange.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(exchange.clone())
    }

    async fn delete(&self, id: Uuid) -> Result<bool, String> {
        let result = sqlx::query("DELETE FROM local_exchanges WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("Database error: {}", e))?;

        Ok(result.rows_affected() > 0)
    }

    async fn count_by_building(&self, building_id: Uuid) -> Result<i64, String> {
        let count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM local_exchanges WHERE building_id = $1")
                .bind(building_id)
                .fetch_one(&self.pool)
                .await
                .map_err(|e| format!("Database error: {}", e))?;

        Ok(count)
    }

    async fn count_by_building_and_status(
        &self,
        building_id: Uuid,
        status: &str,
    ) -> Result<i64, String> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM local_exchanges WHERE building_id = $1 AND status = $2",
        )
        .bind(building_id)
        .bind(status)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(count)
    }

    async fn get_total_credits_exchanged(&self, building_id: Uuid) -> Result<i32, String> {
        let total: Option<i64> = sqlx::query_scalar(
            r#"
            SELECT COALESCE(SUM(credits), 0)
            FROM local_exchanges
            WHERE building_id = $1 AND status = 'Completed'
            "#,
        )
        .bind(building_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(total.unwrap_or(0) as i32)
    }

    async fn get_average_exchange_rating(&self, building_id: Uuid) -> Result<Option<f32>, String> {
        let avg: Option<f32> = sqlx::query_scalar(
            r#"
            SELECT AVG((provider_rating + requester_rating) / 2.0)
            FROM local_exchanges
            WHERE building_id = $1
              AND status = 'Completed'
              AND provider_rating IS NOT NULL
              AND requester_rating IS NOT NULL
            "#,
        )
        .bind(building_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(avg)
    }
}

/// Helper function to map PostgreSQL row to LocalExchange entity
fn map_row_to_exchange(row: &sqlx::postgres::PgRow) -> LocalExchange {
    LocalExchange {
        id: row.get("id"),
        building_id: row.get("building_id"),
        provider_id: row.get("provider_id"),
        requester_id: row.get("requester_id"),
        exchange_type: ExchangeType::from_sql(row.get("exchange_type"))
            .unwrap_or(ExchangeType::Service),
        title: row.get("title"),
        description: row.get("description"),
        credits: row.get("credits"),
        status: ExchangeStatus::from_sql(row.get("status")).unwrap_or(ExchangeStatus::Offered),
        offered_at: row.get("offered_at"),
        requested_at: row.get("requested_at"),
        started_at: row.get("started_at"),
        completed_at: row.get("completed_at"),
        cancelled_at: row.get("cancelled_at"),
        cancellation_reason: row.get("cancellation_reason"),
        provider_rating: row.get("provider_rating"),
        requester_rating: row.get("requester_rating"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}
