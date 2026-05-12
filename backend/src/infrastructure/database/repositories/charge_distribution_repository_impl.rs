use crate::application::ports::ChargeDistributionRepository;
use crate::domain::entities::ChargeDistribution;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

/// PostgreSQL implementation of ChargeDistributionRepository
///
/// Handles automatic charge distribution calculation based on ownership percentages.
/// Part of Issue #73 - Invoice Workflow with charge distribution.
///
/// MONETARY: amount_due/quota_percentage use rust_decimal::Decimal (cf. ADR-0007/0008).
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
        let result = sqlx::query_as::<_, ChargeDistributionRow>(
            r#"
            INSERT INTO charge_distributions (
                id, expense_id, unit_id, owner_id, quota_percentage, amount_due, created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, expense_id, unit_id, owner_id, quota_percentage, amount_due, created_at
            "#,
        )
        .bind(distribution.id)
        .bind(distribution.expense_id)
        .bind(distribution.unit_id)
        .bind(distribution.owner_id)
        .bind(distribution.quota_percentage)
        .bind(distribution.amount_due)
        .bind(distribution.created_at)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to create charge distribution: {}", e))?;

        Ok(result.into_entity())
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
            let result = sqlx::query_as::<_, ChargeDistributionRow>(
                r#"
                INSERT INTO charge_distributions (
                    id, expense_id, unit_id, owner_id, quota_percentage, amount_due, created_at
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7)
                RETURNING id, expense_id, unit_id, owner_id, quota_percentage, amount_due, created_at
                "#
            )
            .bind(dist.id)
            .bind(dist.expense_id)
            .bind(dist.unit_id)
            .bind(dist.owner_id)
            .bind(dist.quota_percentage)
            .bind(dist.amount_due)
            .bind(dist.created_at)
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| format!("Failed to create charge distribution in bulk: {}", e))?;

            created.push(result.into_entity());
        }

        tx.commit()
            .await
            .map_err(|e| format!("Failed to commit transaction: {}", e))?;

        Ok(created)
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<ChargeDistribution>, String> {
        let result = sqlx::query_as::<_, ChargeDistributionRow>(
            r#"
            SELECT id, expense_id, unit_id, owner_id, quota_percentage, amount_due, created_at
            FROM charge_distributions
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Failed to find charge distribution by id: {}", e))?;

        Ok(result.map(|r| r.into_entity()))
    }

    async fn find_by_expense(&self, expense_id: Uuid) -> Result<Vec<ChargeDistribution>, String> {
        let results = sqlx::query_as::<_, ChargeDistributionRow>(
            r#"
            SELECT id, expense_id, unit_id, owner_id, quota_percentage, amount_due, created_at
            FROM charge_distributions
            WHERE expense_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(expense_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to find charge distributions by expense: {}", e))?;

        Ok(results.into_iter().map(|r| r.into_entity()).collect())
    }

    async fn find_by_unit(&self, unit_id: Uuid) -> Result<Vec<ChargeDistribution>, String> {
        let results = sqlx::query_as::<_, ChargeDistributionRow>(
            r#"
            SELECT id, expense_id, unit_id, owner_id, quota_percentage, amount_due, created_at
            FROM charge_distributions
            WHERE unit_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(unit_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to find charge distributions by unit: {}", e))?;

        Ok(results.into_iter().map(|r| r.into_entity()).collect())
    }

    async fn find_by_owner(&self, owner_id: Uuid) -> Result<Vec<ChargeDistribution>, String> {
        let results = sqlx::query_as::<_, ChargeDistributionRow>(
            r#"
            SELECT id, expense_id, unit_id, owner_id, quota_percentage, amount_due, created_at
            FROM charge_distributions
            WHERE owner_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(owner_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to find charge distributions by owner: {}", e))?;

        Ok(results.into_iter().map(|r| r.into_entity()).collect())
    }

    async fn delete_by_expense(&self, expense_id: Uuid) -> Result<(), String> {
        sqlx::query(
            r#"
            DELETE FROM charge_distributions
            WHERE expense_id = $1
            "#,
        )
        .bind(expense_id)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to delete charge distributions by expense: {}", e))?;

        Ok(())
    }

    async fn get_total_due_by_owner(&self, owner_id: Uuid) -> Result<Decimal, String> {
        let result: (Decimal,) = sqlx::query_as(
            r#"
            SELECT COALESCE(SUM(amount_due), 0)
            FROM charge_distributions
            WHERE owner_id = $1
            "#,
        )
        .bind(owner_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to get total due by owner: {}", e))?;

        Ok(result.0)
    }
}

/// Database row representation for charge_distributions table
#[derive(Debug, sqlx::FromRow)]
struct ChargeDistributionRow {
    id: Uuid,
    expense_id: Uuid,
    unit_id: Uuid,
    owner_id: Uuid,
    quota_percentage: Decimal,
    amount_due: Decimal,
    created_at: DateTime<Utc>,
}

impl ChargeDistributionRow {
    fn into_entity(self) -> ChargeDistribution {
        ChargeDistribution {
            id: self.id,
            expense_id: self.expense_id,
            unit_id: self.unit_id,
            owner_id: self.owner_id,
            quota_percentage: self.quota_percentage,
            amount_due: self.amount_due,
            created_at: self.created_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_charge_distribution_row_to_entity() {
        let row = ChargeDistributionRow {
            id: Uuid::new_v4(),
            expense_id: Uuid::new_v4(),
            unit_id: Uuid::new_v4(),
            owner_id: Uuid::new_v4(),
            quota_percentage: Decimal::new(2500, 4), // 0.2500
            amount_due: Decimal::new(50000, 2),      // 500.00
            created_at: Utc::now(),
        };

        let entity = row.into_entity();
        assert_eq!(entity.quota_percentage, dec!(0.2500));
        assert_eq!(entity.amount_due, dec!(500.00));
    }

    #[test]
    fn test_charge_distribution_row_to_entity_edge_cases() {
        let row = ChargeDistributionRow {
            id: Uuid::new_v4(),
            expense_id: Uuid::new_v4(),
            unit_id: Uuid::new_v4(),
            owner_id: Uuid::new_v4(),
            quota_percentage: Decimal::new(10000, 4), // 1.0000 (100%)
            amount_due: Decimal::new(0, 2),           // 0.00
            created_at: Utc::now(),
        };

        let entity = row.into_entity();
        assert_eq!(entity.quota_percentage, dec!(1.0000));
        assert_eq!(entity.amount_due, dec!(0.00));
    }
}
