use crate::domain::entities::Quote;
use async_trait::async_trait;
use uuid::Uuid;

/// Port (interface) for Quote repository (Belgian professional best practice: 3 quotes >5000€)
#[async_trait]
pub trait QuoteRepository: Send + Sync {
    /// Create new quote
    async fn create(&self, quote: &Quote) -> Result<Quote, String>;

    /// Find quote by ID
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Quote>, String>;

    /// Find all quotes for a building
    async fn find_by_building(&self, building_id: Uuid) -> Result<Vec<Quote>, String>;

    /// Find all quotes for a contractor
    async fn find_by_contractor(&self, contractor_id: Uuid) -> Result<Vec<Quote>, String>;

    /// Find quotes by status
    async fn find_by_status(&self, building_id: Uuid, status: &str) -> Result<Vec<Quote>, String>;

    /// Find multiple quotes by IDs (for comparison)
    async fn find_by_ids(&self, ids: Vec<Uuid>) -> Result<Vec<Quote>, String>;

    /// Find quotes for a specific project (by title)
    async fn find_by_project_title(
        &self,
        building_id: Uuid,
        project_title: &str,
    ) -> Result<Vec<Quote>, String>;

    /// Find expired quotes (for background job)
    async fn find_expired(&self) -> Result<Vec<Quote>, String>;

    /// Update quote
    async fn update(&self, quote: &Quote) -> Result<Quote, String>;

    /// Delete quote
    async fn delete(&self, id: Uuid) -> Result<bool, String>;

    /// Count quotes by building
    async fn count_by_building(&self, building_id: Uuid) -> Result<i64, String>;

    /// Count quotes by status for a building
    async fn count_by_status(&self, building_id: Uuid, status: &str) -> Result<i64, String>;
}
