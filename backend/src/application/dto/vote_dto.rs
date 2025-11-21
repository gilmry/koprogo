use crate::domain::entities::{Vote, VoteChoice};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Response DTO for Vote
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VoteResponse {
    pub id: Uuid,
    pub resolution_id: Uuid,
    pub owner_id: Uuid,
    pub unit_id: Uuid,
    pub vote_choice: VoteChoice,
    pub voting_power: f64,
    pub proxy_owner_id: Option<Uuid>,
    pub voted_at: DateTime<Utc>,
    pub is_proxy_vote: bool,
}

impl From<Vote> for VoteResponse {
    fn from(vote: Vote) -> Self {
        Self {
            id: vote.id,
            resolution_id: vote.resolution_id,
            owner_id: vote.owner_id,
            unit_id: vote.unit_id,
            vote_choice: vote.vote_choice.clone(),
            voting_power: vote.voting_power,
            proxy_owner_id: vote.proxy_owner_id,
            voted_at: vote.voted_at,
            is_proxy_vote: vote.is_proxy_vote(),
        }
    }
}

/// Request DTO for casting a vote
#[derive(Debug, Deserialize)]
pub struct CastVoteRequest {
    pub owner_id: Uuid,
    pub unit_id: Uuid,
    pub vote_choice: VoteChoice,
    pub voting_power: f64,
    pub proxy_owner_id: Option<Uuid>,
}

/// Request DTO for changing a vote
#[derive(Debug, Deserialize)]
pub struct ChangeVoteRequest {
    pub vote_choice: VoteChoice,
}
