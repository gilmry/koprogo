use crate::domain::entities::Owner;
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait OwnerRepository: Send + Sync {
    async fn create(&self, owner: &Owner) -> Result<Owner, String>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Owner>, String>;
    async fn find_by_email(&self, email: &str) -> Result<Option<Owner>, String>;
    async fn find_all(&self) -> Result<Vec<Owner>, String>;
    async fn update(&self, owner: &Owner) -> Result<Owner, String>;
    async fn delete(&self, id: Uuid) -> Result<bool, String>;
}
