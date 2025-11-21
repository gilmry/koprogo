use crate::domain::entities::Vote;
use async_trait::async_trait;
use uuid::Uuid;

/// Port (trait) for Vote repository operations
#[async_trait]
pub trait VoteRepository: Send + Sync {
    /// Cast a vote on a resolution
    async fn create(&self, vote: &Vote) -> Result<Vote, String>;

    /// Find a vote by ID
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Vote>, String>;

    /// Find all votes for a resolution
    async fn find_by_resolution_id(&self, resolution_id: Uuid) -> Result<Vec<Vote>, String>;

    /// Find all votes by an owner (across all resolutions)
    async fn find_by_owner_id(&self, owner_id: Uuid) -> Result<Vec<Vote>, String>;

    /// Find a vote for a specific unit on a specific resolution
    async fn find_by_resolution_and_unit(
        &self,
        resolution_id: Uuid,
        unit_id: Uuid,
    ) -> Result<Option<Vote>, String>;

    /// Check if a unit has already voted on a resolution
    async fn has_voted(&self, resolution_id: Uuid, unit_id: Uuid) -> Result<bool, String>;

    /// Update a vote (for changing vote choice)
    async fn update(&self, vote: &Vote) -> Result<Vote, String>;

    /// Delete a vote
    async fn delete(&self, id: Uuid) -> Result<bool, String>;

    /// Count votes for a resolution by choice
    async fn count_by_resolution_and_choice(
        &self,
        resolution_id: Uuid,
    ) -> Result<(i32, i32, i32), String>; // (pour, contre, abstention)

    /// Get total voting power for a resolution by choice
    async fn sum_voting_power_by_resolution(
        &self,
        resolution_id: Uuid,
    ) -> Result<(f64, f64, f64), String>; // (pour, contre, abstention)
}
