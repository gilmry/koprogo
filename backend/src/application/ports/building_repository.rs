use crate::domain::entities::Building;
use async_trait::async_trait;
use uuid::Uuid;

/// Port (interface) pour le repository de bâtiments
#[async_trait]
pub trait BuildingRepository: Send + Sync {
    async fn create(&self, building: &Building) -> Result<Building, String>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Building>, String>;
    async fn find_all(&self) -> Result<Vec<Building>, String>;
    async fn update(&self, building: &Building) -> Result<Building, String>;
    async fn delete(&self, id: Uuid) -> Result<bool, String>;
}
