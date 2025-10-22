use crate::domain::entities::Organization;
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait OrganizationRepository: Send + Sync {
    async fn create(&self, org: &Organization) -> Result<Organization, String>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Organization>, String>;
    async fn find_by_slug(&self, slug: &str) -> Result<Option<Organization>, String>;
    async fn find_all(&self) -> Result<Vec<Organization>, String>;
    async fn update(&self, org: &Organization) -> Result<Organization, String>;
    async fn delete(&self, id: Uuid) -> Result<bool, String>;
    async fn count_buildings(&self, org_id: Uuid) -> Result<i64, String>;
}
