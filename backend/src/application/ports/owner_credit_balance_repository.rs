use crate::domain::entities::OwnerCreditBalance;
use async_trait::async_trait;
use uuid::Uuid;

/// Repository trait for OwnerCreditBalance entity (SEL credit tracking)
#[async_trait]
pub trait OwnerCreditBalanceRepository: Send + Sync {
    /// Create a new credit balance (starts at 0)
    async fn create(&self, balance: &OwnerCreditBalance) -> Result<OwnerCreditBalance, String>;

    /// Find a credit balance by owner and building
    async fn find_by_owner_and_building(
        &self,
        owner_id: Uuid,
        building_id: Uuid,
    ) -> Result<Option<OwnerCreditBalance>, String>;

    /// Find all balances for a building
    async fn find_by_building(&self, building_id: Uuid) -> Result<Vec<OwnerCreditBalance>, String>;

    /// Find all balances for an owner (across all buildings)
    async fn find_by_owner(&self, owner_id: Uuid) -> Result<Vec<OwnerCreditBalance>, String>;

    /// Get or create a balance (ensures balance exists)
    async fn get_or_create(
        &self,
        owner_id: Uuid,
        building_id: Uuid,
    ) -> Result<OwnerCreditBalance, String>;

    /// Update a credit balance
    async fn update(&self, balance: &OwnerCreditBalance) -> Result<OwnerCreditBalance, String>;

    /// Delete a credit balance
    async fn delete(&self, owner_id: Uuid, building_id: Uuid) -> Result<bool, String>;

    /// Get leaderboard (top contributors by balance)
    async fn get_leaderboard(
        &self,
        building_id: Uuid,
        limit: i32,
    ) -> Result<Vec<OwnerCreditBalance>, String>;

    /// Get active participants count (owners with at least 1 exchange)
    async fn count_active_participants(&self, building_id: Uuid) -> Result<i64, String>;

    /// Get total credits in circulation for a building
    async fn get_total_credits_in_circulation(&self, building_id: Uuid) -> Result<i32, String>;
}
