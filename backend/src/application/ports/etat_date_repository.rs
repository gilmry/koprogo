use crate::application::dto::etat_date_dto::EtatDateStatsResponse;
use crate::application::dto::PageRequest;
use crate::domain::entities::{EtatDate, EtatDateStatus};
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait EtatDateRepository: Send + Sync {
    /// Create a new état daté
    async fn create(&self, etat_date: &EtatDate) -> Result<EtatDate, String>;

    /// Find état daté by ID
    async fn find_by_id(&self, id: Uuid) -> Result<Option<EtatDate>, String>;

    /// Find état daté by reference number
    async fn find_by_reference_number(
        &self,
        reference_number: &str,
    ) -> Result<Option<EtatDate>, String>;

    /// Find all états datés for a unit
    async fn find_by_unit(&self, unit_id: Uuid) -> Result<Vec<EtatDate>, String>;

    /// Find all états datés for a building
    async fn find_by_building(&self, building_id: Uuid) -> Result<Vec<EtatDate>, String>;

    /// Find all états datés for an organization (paginated)
    async fn find_all_paginated(
        &self,
        page_request: &PageRequest,
        organization_id: Option<Uuid>,
        status: Option<EtatDateStatus>,
    ) -> Result<(Vec<EtatDate>, i64), String>;

    /// Find overdue états datés (>10 days since request, not yet generated)
    async fn find_overdue(&self, organization_id: Uuid) -> Result<Vec<EtatDate>, String>;

    /// Find expired états datés (>3 months since reference date)
    async fn find_expired(&self, organization_id: Uuid) -> Result<Vec<EtatDate>, String>;

    /// Update an état daté
    async fn update(&self, etat_date: &EtatDate) -> Result<EtatDate, String>;

    /// Delete an état daté
    async fn delete(&self, id: Uuid) -> Result<bool, String>;

    /// Get statistics for dashboard
    async fn get_stats(&self, organization_id: Uuid) -> Result<EtatDateStatsResponse, String>;

    /// Count états datés by status
    async fn count_by_status(
        &self,
        organization_id: Uuid,
        status: EtatDateStatus,
    ) -> Result<i64, String>;
}
