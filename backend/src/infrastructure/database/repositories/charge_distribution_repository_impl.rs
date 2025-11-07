use crate::application::ports::ChargeDistributionRepository;
use crate::domain::entities::ChargeDistribution;
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

/// Stub implementation of ChargeDistributionRepository
/// TODO: Issue #73 - Complete implementation with actual database operations
pub struct PostgresChargeDistributionRepository {
    _pool: PgPool,
}

impl PostgresChargeDistributionRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { _pool: pool }
    }
}

#[async_trait]
impl ChargeDistributionRepository for PostgresChargeDistributionRepository {
    async fn create(
        &self,
        _distribution: &ChargeDistribution,
    ) -> Result<ChargeDistribution, String> {
        Err("Charge distribution repository not yet implemented (Issue #73)".to_string())
    }

    async fn create_bulk(
        &self,
        _distributions: &[ChargeDistribution],
    ) -> Result<Vec<ChargeDistribution>, String> {
        Err("Charge distribution repository not yet implemented (Issue #73)".to_string())
    }

    async fn find_by_id(&self, _id: Uuid) -> Result<Option<ChargeDistribution>, String> {
        Err("Charge distribution repository not yet implemented (Issue #73)".to_string())
    }

    async fn find_by_expense(&self, _expense_id: Uuid) -> Result<Vec<ChargeDistribution>, String> {
        Err("Charge distribution repository not yet implemented (Issue #73)".to_string())
    }

    async fn find_by_unit(&self, _unit_id: Uuid) -> Result<Vec<ChargeDistribution>, String> {
        Err("Charge distribution repository not yet implemented (Issue #73)".to_string())
    }

    async fn find_by_owner(&self, _owner_id: Uuid) -> Result<Vec<ChargeDistribution>, String> {
        Err("Charge distribution repository not yet implemented (Issue #73)".to_string())
    }

    async fn delete_by_expense(&self, _expense_id: Uuid) -> Result<(), String> {
        Err("Charge distribution repository not yet implemented (Issue #73)".to_string())
    }

    async fn get_total_due_by_owner(&self, _owner_id: Uuid) -> Result<f64, String> {
        Err("Charge distribution repository not yet implemented (Issue #73)".to_string())
    }
}
