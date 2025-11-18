use crate::application::dto::{PageRequest, PollFilters};
use crate::domain::entities::Poll;
use async_trait::async_trait;
use serde::Serialize;
use uuid::Uuid;

#[async_trait]
pub trait PollRepository: Send + Sync {
    async fn create(&self, poll: &Poll) -> Result<Poll, String>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Poll>, String>;
    async fn find_by_building(&self, building_id: Uuid) -> Result<Vec<Poll>, String>;
    async fn find_by_created_by(&self, created_by: Uuid) -> Result<Vec<Poll>, String>;

    /// Find all polls with pagination and filters
    /// Returns tuple of (polls, total_count)
    async fn find_all_paginated(
        &self,
        page_request: &PageRequest,
        filters: &PollFilters,
    ) -> Result<(Vec<Poll>, i64), String>;

    /// Find active polls (status = active and within time range)
    async fn find_active(&self, building_id: Uuid) -> Result<Vec<Poll>, String>;

    /// Find polls by status
    async fn find_by_status(
        &self,
        building_id: Uuid,
        status: &str,
    ) -> Result<Vec<Poll>, String>;

    /// Find expired polls that should be auto-closed
    async fn find_expired_active(&self) -> Result<Vec<Poll>, String>;

    async fn update(&self, poll: &Poll) -> Result<Poll, String>;
    async fn delete(&self, id: Uuid) -> Result<bool, String>;

    /// Get poll statistics for a building
    async fn get_building_statistics(&self, building_id: Uuid) -> Result<PollStatistics, String>;
}

#[derive(Debug, Clone, Serialize)]
pub struct PollStatistics {
    pub total_polls: i64,
    pub active_polls: i64,
    pub closed_polls: i64,
    pub average_participation_rate: f64,
}
