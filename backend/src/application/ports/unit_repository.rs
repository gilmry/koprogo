use crate::application::dto::{PageRequest, UnitFilters};
use crate::domain::entities::Unit;
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait UnitRepository: Send + Sync {
    async fn create(&self, unit: &Unit) -> Result<Unit, String>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Unit>, String>;
    async fn find_by_building(&self, building_id: Uuid) -> Result<Vec<Unit>, String>;
    async fn find_by_owner(&self, owner_id: Uuid) -> Result<Vec<Unit>, String>;

    /// Find all units with pagination and filters
    /// Returns tuple of (units, total_count)
    async fn find_all_paginated(
        &self,
        page_request: &PageRequest,
        filters: &UnitFilters,
    ) -> Result<(Vec<Unit>, i64), String>;

    async fn update(&self, unit: &Unit) -> Result<Unit, String>;
    async fn delete(&self, id: Uuid) -> Result<bool, String>;
}
