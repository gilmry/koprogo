use crate::application::dto::{PageRequest, WorkReportFilters};
use crate::domain::entities::WorkReport;
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait WorkReportRepository: Send + Sync {
    async fn create(&self, work_report: &WorkReport) -> Result<WorkReport, String>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<WorkReport>, String>;
    async fn find_by_building(&self, building_id: Uuid) -> Result<Vec<WorkReport>, String>;
    async fn find_by_organization(&self, organization_id: Uuid) -> Result<Vec<WorkReport>, String>;

    /// Find all work reports with pagination and filters
    /// Returns tuple of (work_reports, total_count)
    async fn find_all_paginated(
        &self,
        page_request: &PageRequest,
        filters: &WorkReportFilters,
    ) -> Result<(Vec<WorkReport>, i64), String>;

    /// Find work reports with active warranties
    async fn find_with_active_warranty(
        &self,
        building_id: Uuid,
    ) -> Result<Vec<WorkReport>, String>;

    /// Find work reports with expiring warranties (within N days)
    async fn find_with_expiring_warranty(
        &self,
        building_id: Uuid,
        days: i32,
    ) -> Result<Vec<WorkReport>, String>;

    async fn update(&self, work_report: &WorkReport) -> Result<WorkReport, String>;
    async fn delete(&self, id: Uuid) -> Result<bool, String>;
}
