use crate::domain::entities::ChargeDistribution;
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait ChargeDistributionRepository: Send + Sync {
    /// Create a single charge distribution
    async fn create(&self, distribution: &ChargeDistribution) -> Result<ChargeDistribution, String>;

    /// Create multiple charge distributions in bulk (for performance)
    async fn create_bulk(&self, distributions: &[ChargeDistribution]) -> Result<Vec<ChargeDistribution>, String>;

    /// Find charge distribution by ID
    async fn find_by_id(&self, id: Uuid) -> Result<Option<ChargeDistribution>, String>;

    /// Find all charge distributions for an expense/invoice
    async fn find_by_expense(&self, expense_id: Uuid) -> Result<Vec<ChargeDistribution>, String>;

    /// Find all charge distributions for a unit
    async fn find_by_unit(&self, unit_id: Uuid) -> Result<Vec<ChargeDistribution>, String>;

    /// Find all charge distributions for an owner
    async fn find_by_owner(&self, owner_id: Uuid) -> Result<Vec<ChargeDistribution>, String>;

    /// Delete all distributions for an expense (e.g., if invoice is cancelled)
    async fn delete_by_expense(&self, expense_id: Uuid) -> Result<(), String>;

    /// Get total amount due for an owner across all pending distributions
    async fn get_total_due_by_owner(&self, owner_id: Uuid) -> Result<f64, String>;
}
