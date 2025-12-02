use crate::domain::entities::OwnerContribution;
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait OwnerContributionRepository: Send + Sync {
    async fn create(&self, contribution: &OwnerContribution) -> Result<OwnerContribution, String>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<OwnerContribution>, String>;
    async fn find_by_organization(
        &self,
        organization_id: Uuid,
    ) -> Result<Vec<OwnerContribution>, String>;
    async fn find_by_owner(&self, owner_id: Uuid) -> Result<Vec<OwnerContribution>, String>;
    async fn update(&self, contribution: &OwnerContribution) -> Result<OwnerContribution, String>;
}
