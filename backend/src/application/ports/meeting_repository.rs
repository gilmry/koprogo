use crate::application::dto::PageRequest;
use crate::domain::entities::Meeting;
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait MeetingRepository: Send + Sync {
    async fn create(&self, meeting: &Meeting) -> Result<Meeting, String>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Meeting>, String>;
    async fn find_by_building(&self, building_id: Uuid) -> Result<Vec<Meeting>, String>;
    async fn update(&self, meeting: &Meeting) -> Result<Meeting, String>;
    async fn delete(&self, id: Uuid) -> Result<bool, String>;

    /// Find all meetings with pagination
    async fn find_all_paginated(
        &self,
        page_request: &PageRequest,
        organization_id: Option<Uuid>,
    ) -> Result<(Vec<Meeting>, i64), String>;
}
