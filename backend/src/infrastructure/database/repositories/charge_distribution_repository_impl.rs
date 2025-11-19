use crate::application::ports::ChargeDistributionRepository;
use crate::domain::entities::ChargeDistribution;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

/// PostgreSQL implementation of ChargeDistributionRepository
///
/// Handles automatic charge distribution calculation based on ownership percentages.
/// Part of Issue #73 - Invoice Workflow with charge distribution.
pub struct PostgresChargeDistributionRepository {
    pool: PgPool,
}

impl PostgresChargeDistributionRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ChargeDistributionRepository for PostgresChargeDistributionRepository {
    async fn create(
        &self,
        distribution: &ChargeDistribution,
    ) -> Result<ChargeDistribution, String> {
        let result = sqlx::query_as!(
            ChargeDistributionRow,
            r#"
            INSERT INTO charge_distributions (
                id, expense_id, unit_id, owner_id, quota_percentage, amount_due, created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, expense_id, unit_id, owner_id, quota_percentage, amount_due, created_at
            "#,
            distribution.id,
            distribution.expense_id,
            distribution.unit_id,
            distribution.owner_id,
            distribution.quota_percentage,
            distribution.amount_due,
            distribution.created_at
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to create charge distribution: {}", e))?;

        Ok(result.to_entity())
    }

    async fn create_bulk(
        &self,
        distributions: &[ChargeDistribution],
    ) -> Result<Vec<ChargeDistribution>, String> {
        if distributions.is_empty() {
            return Ok(Vec::new());
        }

        // Use transaction for atomicity
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| format!("Failed to begin transaction: {}", e))?;

        let mut created = Vec::new();

        for dist in distributions {
            let result = sqlx::query_as!(
                ChargeDistributionRow,
                r#"
                INSERT INTO charge_distributions (
                    id, expense_id, unit_id, owner_id, quota_percentage, amount_due, created_at
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7)
                RETURNING id, expense_id, unit_id, owner_id, quota_percentage, amount_due, created_at
                "#,
                dist.id,
                dist.expense_id,
                dist.unit_id,
                dist.owner_id,
                dist.quota_percentage,
                dist.amount_due,
                dist.created_at
            )
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| format!("Failed to create charge distribution in bulk: {}", e))?;

            created.push(result.to_entity());
        }

        tx.commit()
            .await
            .map_err(|e| format!("Failed to commit transaction: {}", e))?;

        Ok(created)
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<ChargeDistribution>, String> {
        let result = sqlx::query_as!(
            ChargeDistributionRow,
            r#"
            SELECT id, expense_id, unit_id, owner_id, quota_percentage, amount_due, created_at
            FROM charge_distributions
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Failed to find charge distribution by id: {}", e))?;

        Ok(result.map(|r| r.to_entity()))
    }

    async fn find_by_expense(&self, expense_id: Uuid) -> Result<Vec<ChargeDistribution>, String> {
        let results = sqlx::query_as!(
            ChargeDistributionRow,
            r#"
            SELECT id, expense_id, unit_id, owner_id, quota_percentage, amount_due, created_at
            FROM charge_distributions
            WHERE expense_id = $1
            ORDER BY created_at DESC
            "#,
            expense_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to find charge distributions by expense: {}", e))?;

        Ok(results.into_iter().map(|r| r.to_entity()).collect())
    }

    async fn find_by_unit(&self, unit_id: Uuid) -> Result<Vec<ChargeDistribution>, String> {
        let results = sqlx::query_as!(
            ChargeDistributionRow,
            r#"
            SELECT id, expense_id, unit_id, owner_id, quota_percentage, amount_due, created_at
            FROM charge_distributions
            WHERE unit_id = $1
            ORDER BY created_at DESC
            "#,
            unit_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to find charge distributions by unit: {}", e))?;

        Ok(results.into_iter().map(|r| r.to_entity()).collect())
    }

    async fn find_by_owner(&self, owner_id: Uuid) -> Result<Vec<ChargeDistribution>, String> {
        let results = sqlx::query_as!(
            ChargeDistributionRow,
            r#"
            SELECT id, expense_id, unit_id, owner_id, quota_percentage, amount_due, created_at
            FROM charge_distributions
            WHERE owner_id = $1
            ORDER BY created_at DESC
            "#,
            owner_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to find charge distributions by owner: {}", e))?;

        Ok(results.into_iter().map(|r| r.to_entity()).collect())
    }

    async fn delete_by_expense(&self, expense_id: Uuid) -> Result<(), String> {
        sqlx::query!(
            r#"
            DELETE FROM charge_distributions
            WHERE expense_id = $1
            "#,
            expense_id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to delete charge distributions by expense: {}", e))?;

        Ok(())
    }

    async fn get_total_due_by_owner(&self, owner_id: Uuid) -> Result<f64, String> {
        let result = sqlx::query!(
            r#"
            SELECT COALESCE(SUM(amount_due), 0) as "total!"
            FROM charge_distributions
            WHERE owner_id = $1
            "#,
            owner_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to get total due by owner: {}", e))?;

        Ok(result.total)
    }
}

/// Database row representation for charge_distributions table
#[derive(Debug)]
struct ChargeDistributionRow {
    id: Uuid,
    expense_id: Uuid,
    unit_id: Uuid,
    owner_id: Uuid,
    quota_percentage: rust_decimal::Decimal,
    amount_due: rust_decimal::Decimal,
    created_at: DateTime<Utc>,
}

impl ChargeDistributionRow {
    fn to_entity(self) -> ChargeDistribution {
        ChargeDistribution {
            id: self.id,
            expense_id: self.expense_id,
            unit_id: self.unit_id,
            owner_id: self.owner_id,
            quota_percentage: self.quota_percentage.to_string().parse().unwrap_or(0.0),
            amount_due: self.amount_due.to_string().parse().unwrap_or(0.0),
            created_at: self.created_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_charge_distribution_row_to_entity() {
        use rust_decimal::Decimal;

        let row = ChargeDistributionRow {
            id: Uuid::new_v4(),
            expense_id: Uuid::new_v4(),
            unit_id: Uuid::new_v4(),
            owner_id: Uuid::new_v4(),
            quota_percentage: Decimal::new(2500, 4), // 0.2500
            amount_due: Decimal::new(50000, 2),     // 500.00
            created_at: Utc::now(),
        };

        let entity = row.to_entity();
        assert_eq!(entity.quota_percentage, 0.25);
        assert_eq!(entity.amount_due, 500.0);
    }

    #[test]
    fn test_charge_distribution_row_to_entity_edge_cases() {
        use rust_decimal::Decimal;

        let row = ChargeDistributionRow {
            id: Uuid::new_v4(),
            expense_id: Uuid::new_v4(),
            unit_id: Uuid::new_v4(),
            owner_id: Uuid::new_v4(),
            quota_percentage: Decimal::new(10000, 4), // 1.0000 (100%)
            amount_due: Decimal::new(0, 2),          // 0.00
            created_at: Utc::now(),
        };

        let entity = row.to_entity();
        assert_eq!(entity.quota_percentage, 1.0);
        assert_eq!(entity.amount_due, 0.0);
    }
}
