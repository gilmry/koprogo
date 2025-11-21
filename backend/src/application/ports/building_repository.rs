use crate::application::dto::{BuildingFilters, PageRequest};
use crate::domain::entities::Building;
use async_trait::async_trait;
use uuid::Uuid;

/// Port (interface) pour le repository de bâtiments
#[async_trait]
pub trait BuildingRepository: Send + Sync {
    async fn create(&self, building: &Building) -> Result<Building, String>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Building>, String>;
    async fn find_all(&self) -> Result<Vec<Building>, String>;

    /// Find all buildings with pagination and filters
    /// Returns tuple of (buildings, total_count)
    async fn find_all_paginated(
        &self,
        page_request: &PageRequest,
        filters: &BuildingFilters,
    ) -> Result<(Vec<Building>, i64), String>;

    async fn update(&self, building: &Building) -> Result<Building, String>;
    async fn delete(&self, id: Uuid) -> Result<bool, String>;

    /// Find building by URL slug (for public pages - Issue #92)
    async fn find_by_slug(&self, slug: &str) -> Result<Option<Building>, String>;
}
