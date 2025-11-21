use crate::application::ports::OwnerCreditBalanceRepository;
use crate::domain::entities::OwnerCreditBalance;
use crate::infrastructure::database::pool::DbPool;
use async_trait::async_trait;
use sqlx::Row;
use uuid::Uuid;

pub struct PostgresOwnerCreditBalanceRepository {
    pool: DbPool,
}

impl PostgresOwnerCreditBalanceRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl OwnerCreditBalanceRepository for PostgresOwnerCreditBalanceRepository {
    async fn create(&self, balance: &OwnerCreditBalance) -> Result<OwnerCreditBalance, String> {
        sqlx::query(
            r#"
            INSERT INTO owner_credit_balances (
                owner_id, building_id, credits_earned, credits_spent,
                balance, total_exchanges, average_rating,
                created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
        )
        .bind(balance.owner_id)
        .bind(balance.building_id)
        .bind(balance.credits_earned)
        .bind(balance.credits_spent)
        .bind(balance.balance)
        .bind(balance.total_exchanges)
        .bind(balance.average_rating)
        .bind(balance.created_at)
        .bind(balance.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(balance.clone())
    }

    async fn find_by_owner_and_building(
        &self,
        owner_id: Uuid,
        building_id: Uuid,
    ) -> Result<Option<OwnerCreditBalance>, String> {
        let row = sqlx::query(
            r#"
            SELECT
                owner_id, building_id, credits_earned, credits_spent,
                balance, total_exchanges, average_rating,
                created_at, updated_at
            FROM owner_credit_balances
            WHERE owner_id = $1 AND building_id = $2
            "#,
        )
        .bind(owner_id)
        .bind(building_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(row.map(|row| map_row_to_balance(&row)))
    }

    async fn find_by_building(&self, building_id: Uuid) -> Result<Vec<OwnerCreditBalance>, String> {
        let rows = sqlx::query(
            r#"
            SELECT
                owner_id, building_id, credits_earned, credits_spent,
                balance, total_exchanges, average_rating,
                created_at, updated_at
            FROM owner_credit_balances
            WHERE building_id = $1
            ORDER BY balance DESC
            "#,
        )
        .bind(building_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(rows.iter().map(map_row_to_balance).collect())
    }

    async fn find_by_owner(&self, owner_id: Uuid) -> Result<Vec<OwnerCreditBalance>, String> {
        let rows = sqlx::query(
            r#"
            SELECT
                owner_id, building_id, credits_earned, credits_spent,
                balance, total_exchanges, average_rating,
                created_at, updated_at
            FROM owner_credit_balances
            WHERE owner_id = $1
            ORDER BY building_id
            "#,
        )
        .bind(owner_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(rows.iter().map(map_row_to_balance).collect())
    }

    async fn get_or_create(
        &self,
        owner_id: Uuid,
        building_id: Uuid,
    ) -> Result<OwnerCreditBalance, String> {
        // Try to find existing balance
        if let Some(balance) = self
            .find_by_owner_and_building(owner_id, building_id)
            .await?
        {
            return Ok(balance);
        }

        // Create new balance if not exists
        let new_balance = OwnerCreditBalance::new(owner_id, building_id);
        self.create(&new_balance).await
    }

    async fn update(&self, balance: &OwnerCreditBalance) -> Result<OwnerCreditBalance, String> {
        sqlx::query(
            r#"
            UPDATE owner_credit_balances
            SET
                credits_earned = $3,
                credits_spent = $4,
                balance = $5,
                total_exchanges = $6,
                average_rating = $7,
                updated_at = $8
            WHERE owner_id = $1 AND building_id = $2
            "#,
        )
        .bind(balance.owner_id)
        .bind(balance.building_id)
        .bind(balance.credits_earned)
        .bind(balance.credits_spent)
        .bind(balance.balance)
        .bind(balance.total_exchanges)
        .bind(balance.average_rating)
        .bind(balance.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(balance.clone())
    }

    async fn delete(&self, owner_id: Uuid, building_id: Uuid) -> Result<bool, String> {
        let result = sqlx::query(
            "DELETE FROM owner_credit_balances WHERE owner_id = $1 AND building_id = $2",
        )
        .bind(owner_id)
        .bind(building_id)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(result.rows_affected() > 0)
    }

    async fn get_leaderboard(
        &self,
        building_id: Uuid,
        limit: i32,
    ) -> Result<Vec<OwnerCreditBalance>, String> {
        let rows = sqlx::query(
            r#"
            SELECT
                owner_id, building_id, credits_earned, credits_spent,
                balance, total_exchanges, average_rating,
                created_at, updated_at
            FROM owner_credit_balances
            WHERE building_id = $1
            ORDER BY balance DESC
            LIMIT $2
            "#,
        )
        .bind(building_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(rows.iter().map(map_row_to_balance).collect())
    }

    async fn count_active_participants(&self, building_id: Uuid) -> Result<i64, String> {
        let count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*)
            FROM owner_credit_balances
            WHERE building_id = $1 AND total_exchanges > 0
            "#,
        )
        .bind(building_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(count)
    }

    async fn get_total_credits_in_circulation(&self, building_id: Uuid) -> Result<i32, String> {
        let total: Option<i64> = sqlx::query_scalar(
            r#"
            SELECT COALESCE(SUM(ABS(balance)), 0)
            FROM owner_credit_balances
            WHERE building_id = $1
            "#,
        )
        .bind(building_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(total.unwrap_or(0) as i32)
    }
}

/// Helper function to map PostgreSQL row to OwnerCreditBalance entity
fn map_row_to_balance(row: &sqlx::postgres::PgRow) -> OwnerCreditBalance {
    OwnerCreditBalance {
        owner_id: row.get("owner_id"),
        building_id: row.get("building_id"),
        credits_earned: row.get("credits_earned"),
        credits_spent: row.get("credits_spent"),
        balance: row.get("balance"),
        total_exchanges: row.get("total_exchanges"),
        average_rating: row.get("average_rating"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}
