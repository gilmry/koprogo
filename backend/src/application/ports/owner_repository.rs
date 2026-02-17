use crate::application::dto::{OwnerFilters, PageRequest};
use crate::domain::entities::Owner;
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait OwnerRepository: Send + Sync {
    async fn create(&self, owner: &Owner) -> Result<Owner, String>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Owner>, String>;
    async fn find_by_user_id(&self, user_id: Uuid) -> Result<Option<Owner>, String>;
    async fn find_by_email(&self, email: &str) -> Result<Option<Owner>, String>;
    async fn find_all(&self) -> Result<Vec<Owner>, String>;

    /// Find all owners with pagination and filters
    /// Returns tuple of (owners, total_count)
    async fn find_all_paginated(
        &self,
        page_request: &PageRequest,
        filters: &OwnerFilters,
    ) -> Result<(Vec<Owner>, i64), String>;

    async fn update(&self, owner: &Owner) -> Result<Owner, String>;
    async fn delete(&self, id: Uuid) -> Result<bool, String>;
}
