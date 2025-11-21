use crate::domain::entities::{Resolution, ResolutionStatus};
use async_trait::async_trait;
use uuid::Uuid;

/// Port (trait) for Resolution repository operations
#[async_trait]
pub trait ResolutionRepository: Send + Sync {
    /// Create a new resolution
    async fn create(&self, resolution: &Resolution) -> Result<Resolution, String>;

    /// Find a resolution by ID
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Resolution>, String>;

    /// Find all resolutions for a meeting
    async fn find_by_meeting_id(&self, meeting_id: Uuid) -> Result<Vec<Resolution>, String>;

    /// Find resolutions by status
    async fn find_by_status(&self, status: ResolutionStatus) -> Result<Vec<Resolution>, String>;

    /// Update a resolution (for vote counts and status changes)
    async fn update(&self, resolution: &Resolution) -> Result<Resolution, String>;

    /// Delete a resolution
    async fn delete(&self, id: Uuid) -> Result<bool, String>;

    /// Update vote counts for a resolution
    async fn update_vote_counts(
        &self,
        resolution_id: Uuid,
        vote_count_pour: i32,
        vote_count_contre: i32,
        vote_count_abstention: i32,
        total_voting_power_pour: f64,
        total_voting_power_contre: f64,
        total_voting_power_abstention: f64,
    ) -> Result<(), String>;

    /// Close voting on a resolution and set final status
    async fn close_voting(
        &self,
        resolution_id: Uuid,
        final_status: ResolutionStatus,
    ) -> Result<(), String>;

    /// Get vote summary for all resolutions in a meeting
    async fn get_meeting_vote_summary(&self, meeting_id: Uuid) -> Result<Vec<Resolution>, String>;
}
