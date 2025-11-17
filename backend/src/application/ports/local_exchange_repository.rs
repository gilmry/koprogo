use crate::domain::entities::LocalExchange;
use async_trait::async_trait;
use uuid::Uuid;

/// Repository trait for LocalExchange entity (SEL - Local Exchange Trading System)
#[async_trait]
pub trait LocalExchangeRepository: Send + Sync {
    /// Create a new exchange offer
    async fn create(&self, exchange: &LocalExchange) -> Result<LocalExchange, String>;

    /// Find an exchange by ID
    async fn find_by_id(&self, id: Uuid) -> Result<Option<LocalExchange>, String>;

    /// Find all exchanges for a building
    async fn find_by_building(&self, building_id: Uuid) -> Result<Vec<LocalExchange>, String>;

    /// Find exchanges by building and status
    async fn find_by_building_and_status(
        &self,
        building_id: Uuid,
        status: &str,
    ) -> Result<Vec<LocalExchange>, String>;

    /// Find exchanges by provider
    async fn find_by_provider(&self, provider_id: Uuid) -> Result<Vec<LocalExchange>, String>;

    /// Find exchanges by requester
    async fn find_by_requester(&self, requester_id: Uuid) -> Result<Vec<LocalExchange>, String>;

    /// Find exchanges by owner (as provider OR requester)
    async fn find_by_owner(&self, owner_id: Uuid) -> Result<Vec<LocalExchange>, String>;

    /// Find active exchanges (Offered, Requested, InProgress)
    async fn find_active_by_building(
        &self,
        building_id: Uuid,
    ) -> Result<Vec<LocalExchange>, String>;

    /// Find available exchanges (status = Offered)
    async fn find_available_by_building(
        &self,
        building_id: Uuid,
    ) -> Result<Vec<LocalExchange>, String>;

    /// Find exchanges by type
    async fn find_by_type(
        &self,
        building_id: Uuid,
        exchange_type: &str,
    ) -> Result<Vec<LocalExchange>, String>;

    /// Update an exchange
    async fn update(&self, exchange: &LocalExchange) -> Result<LocalExchange, String>;

    /// Delete an exchange
    async fn delete(&self, id: Uuid) -> Result<bool, String>;

    /// Count exchanges by building
    async fn count_by_building(&self, building_id: Uuid) -> Result<i64, String>;

    /// Count exchanges by building and status
    async fn count_by_building_and_status(
        &self,
        building_id: Uuid,
        status: &str,
    ) -> Result<i64, String>;

    /// Get total credits exchanged in a building (sum of completed exchanges)
    async fn get_total_credits_exchanged(&self, building_id: Uuid) -> Result<i32, String>;

    /// Get average exchange rating for a building
    async fn get_average_exchange_rating(&self, building_id: Uuid) -> Result<Option<f32>, String>;
}
