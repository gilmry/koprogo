use crate::application::dto::PageRequest;
use crate::domain::entities::Document;
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait DocumentRepository: Send + Sync {
    async fn create(&self, document: &Document) -> Result<Document, String>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Document>, String>;
    async fn find_by_building(&self, building_id: Uuid) -> Result<Vec<Document>, String>;
    async fn find_by_meeting(&self, meeting_id: Uuid) -> Result<Vec<Document>, String>;
    async fn update(&self, document: &Document) -> Result<Document, String>;
    async fn delete(&self, id: Uuid) -> Result<bool, String>;

    /// Find all documents with pagination
    async fn find_all_paginated(
        &self,
        page_request: &PageRequest,
        organization_id: Option<Uuid>,
    ) -> Result<(Vec<Document>, i64), String>;
}
