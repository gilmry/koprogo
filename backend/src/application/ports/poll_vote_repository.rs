use crate::domain::entities::PollVote;
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait PollVoteRepository: Send + Sync {
    async fn create(&self, vote: &PollVote) -> Result<PollVote, String>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<PollVote>, String>;
    async fn find_by_poll(&self, poll_id: Uuid) -> Result<Vec<PollVote>, String>;
    async fn find_by_poll_and_owner(
        &self,
        poll_id: Uuid,
        owner_id: Uuid,
    ) -> Result<Option<PollVote>, String>;

    /// Find all votes for a building (all polls)
    async fn find_by_building(&self, building_id: Uuid) -> Result<Vec<PollVote>, String>;

    /// Count votes for a specific poll
    async fn count_votes(&self, poll_id: Uuid) -> Result<i64, String>;

    /// Check if owner has already voted on a poll
    async fn has_voted(&self, poll_id: Uuid, owner_id: Uuid) -> Result<bool, String>;

    /// Get vote count per option for a poll
    async fn get_option_vote_counts(&self, poll_id: Uuid) -> Result<Vec<OptionVoteCount>, String>;

    /// Get all open-ended responses for a poll
    async fn get_open_responses(&self, poll_id: Uuid) -> Result<Vec<String>, String>;

    async fn delete(&self, id: Uuid) -> Result<bool, String>;
}

#[derive(Debug, Clone)]
pub struct OptionVoteCount {
    pub option_id: Uuid,
    pub vote_count: i64,
}
