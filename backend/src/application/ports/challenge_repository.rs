use crate::domain::entities::{Challenge, ChallengeProgress, ChallengeStatus};
use async_trait::async_trait;
use uuid::Uuid;

/// Repository trait for Challenge persistence operations
#[async_trait]
pub trait ChallengeRepository: Send + Sync {
    /// Create a new challenge
    async fn create(&self, challenge: &Challenge) -> Result<Challenge, String>;

    /// Find challenge by ID
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Challenge>, String>;

    /// Find all challenges for an organization
    async fn find_by_organization(&self, organization_id: Uuid) -> Result<Vec<Challenge>, String>;

    /// Find challenges by organization and status
    async fn find_by_organization_and_status(
        &self,
        organization_id: Uuid,
        status: ChallengeStatus,
    ) -> Result<Vec<Challenge>, String>;

    /// Find challenges by building
    async fn find_by_building(&self, building_id: Uuid) -> Result<Vec<Challenge>, String>;

    /// Find active challenges (status = Active AND now >= start_date AND now < end_date)
    async fn find_active(&self, organization_id: Uuid) -> Result<Vec<Challenge>, String>;

    /// Find challenges that have ended but not yet marked Completed
    async fn find_ended_not_completed(&self) -> Result<Vec<Challenge>, String>;

    /// Update challenge
    async fn update(&self, challenge: &Challenge) -> Result<Challenge, String>;

    /// Delete challenge
    async fn delete(&self, id: Uuid) -> Result<(), String>;

    /// Count challenges by organization
    async fn count_by_organization(&self, organization_id: Uuid) -> Result<i64, String>;
}

/// Repository trait for ChallengeProgress persistence operations
#[async_trait]
pub trait ChallengeProgressRepository: Send + Sync {
    /// Create new challenge progress tracking
    async fn create(&self, progress: &ChallengeProgress) -> Result<ChallengeProgress, String>;

    /// Find progress by ID
    async fn find_by_id(&self, id: Uuid) -> Result<Option<ChallengeProgress>, String>;

    /// Find progress by user and challenge
    async fn find_by_user_and_challenge(
        &self,
        user_id: Uuid,
        challenge_id: Uuid,
    ) -> Result<Option<ChallengeProgress>, String>;

    /// Find all progress for a user
    async fn find_by_user(&self, user_id: Uuid) -> Result<Vec<ChallengeProgress>, String>;

    /// Find all progress for a challenge
    async fn find_by_challenge(&self, challenge_id: Uuid)
        -> Result<Vec<ChallengeProgress>, String>;

    /// Find active progress for user (challenge is Active and not completed)
    async fn find_active_by_user(&self, user_id: Uuid) -> Result<Vec<ChallengeProgress>, String>;

    /// Update progress
    async fn update(&self, progress: &ChallengeProgress) -> Result<ChallengeProgress, String>;

    /// Count completed challenges for user
    async fn count_completed_by_user(&self, user_id: Uuid) -> Result<i64, String>;

    /// Get leaderboard data (top users by points)
    async fn get_leaderboard(
        &self,
        organization_id: Uuid,
        building_id: Option<Uuid>,
        limit: i64,
    ) -> Result<Vec<(Uuid, i32)>, String>; // Returns (user_id, total_points)
}
