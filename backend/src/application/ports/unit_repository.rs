use crate::domain::entities::Unit;
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait UnitRepository: Send + Sync {
    async fn create(&self, unit: &Unit) -> Result<Unit, String>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Unit>, String>;
    async fn find_by_building(&self, building_id: Uuid) -> Result<Vec<Unit>, String>;
    async fn find_by_owner(&self, owner_id: Uuid) -> Result<Vec<Unit>, String>;
    async fn update(&self, unit: &Unit) -> Result<Unit, String>;
    async fn delete(&self, id: Uuid) -> Result<bool, String>;
}
