use crate::application::dto::{PageRequest, TechnicalInspectionFilters};
use crate::domain::entities::TechnicalInspection;
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait TechnicalInspectionRepository: Send + Sync {
    async fn create(
        &self,
        inspection: &TechnicalInspection,
    ) -> Result<TechnicalInspection, String>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<TechnicalInspection>, String>;
    async fn find_by_building(&self, building_id: Uuid) -> Result<Vec<TechnicalInspection>, String>;
    async fn find_by_organization(
        &self,
        organization_id: Uuid,
    ) -> Result<Vec<TechnicalInspection>, String>;

    /// Find all technical inspections with pagination and filters
    /// Returns tuple of (inspections, total_count)
    async fn find_all_paginated(
        &self,
        page_request: &PageRequest,
        filters: &TechnicalInspectionFilters,
    ) -> Result<(Vec<TechnicalInspection>, i64), String>;

    /// Find overdue inspections for a building
    async fn find_overdue(&self, building_id: Uuid) -> Result<Vec<TechnicalInspection>, String>;

    /// Find upcoming inspections (due within N days)
    async fn find_upcoming(
        &self,
        building_id: Uuid,
        days: i32,
    ) -> Result<Vec<TechnicalInspection>, String>;

    /// Find inspections by type for a building
    async fn find_by_type(
        &self,
        building_id: Uuid,
        inspection_type: &str,
    ) -> Result<Vec<TechnicalInspection>, String>;

    async fn update(&self, inspection: &TechnicalInspection) -> Result<TechnicalInspection, String>;
    async fn delete(&self, id: Uuid) -> Result<bool, String>;
}
