use crate::domain::entities::PollVote;
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait PollVoteRepository: Send + Sync {
    /// Create a new poll vote
    async fn create(&self, vote: &PollVote) -> Result<PollVote, String>;

    /// Find vote by ID
    async fn find_by_id(&self, id: Uuid) -> Result<Option<PollVote>, String>;

    /// Find all votes for a poll
    async fn find_by_poll(&self, poll_id: Uuid) -> Result<Vec<PollVote>, String>;

    /// Find vote by poll and owner (for duplicate checking)
    async fn find_by_poll_and_owner(
        &self,
        poll_id: Uuid,
        owner_id: Uuid,
    ) -> Result<Option<PollVote>, String>;

    /// Find all votes by an owner
    async fn find_by_owner(&self, owner_id: Uuid) -> Result<Vec<PollVote>, String>;

    /// Delete a vote
    async fn delete(&self, id: Uuid) -> Result<bool, String>;
}
