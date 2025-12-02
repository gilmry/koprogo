use crate::domain::entities::{MajorityType, Resolution, ResolutionStatus, ResolutionType};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Response DTO for Resolution
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ResolutionResponse {
    pub id: Uuid,
    pub meeting_id: Uuid,
    pub title: String,
    pub description: String,
    pub resolution_type: ResolutionType,
    pub majority_required: MajorityType,
    pub vote_count_pour: i32,
    pub vote_count_contre: i32,
    pub vote_count_abstention: i32,
    pub total_voting_power_pour: f64,
    pub total_voting_power_contre: f64,
    pub total_voting_power_abstention: f64,
    pub status: ResolutionStatus,
    pub created_at: DateTime<Utc>,
    pub voted_at: Option<DateTime<Utc>>,
    // Calculated fields
    pub total_votes: i32,
    pub pour_percentage: f64,
    pub contre_percentage: f64,
    pub abstention_percentage: f64,
}

impl From<Resolution> for ResolutionResponse {
    fn from(resolution: Resolution) -> Self {
        Self {
            id: resolution.id,
            meeting_id: resolution.meeting_id,
            title: resolution.title.clone(),
            description: resolution.description.clone(),
            resolution_type: resolution.resolution_type.clone(),
            majority_required: resolution.majority_required.clone(),
            vote_count_pour: resolution.vote_count_pour,
            vote_count_contre: resolution.vote_count_contre,
            vote_count_abstention: resolution.vote_count_abstention,
            total_voting_power_pour: resolution.total_voting_power_pour,
            total_voting_power_contre: resolution.total_voting_power_contre,
            total_voting_power_abstention: resolution.total_voting_power_abstention,
            status: resolution.status.clone(),
            created_at: resolution.created_at,
            voted_at: resolution.voted_at,
            // Calculated
            total_votes: resolution.total_votes(),
            pour_percentage: resolution.pour_percentage(),
            contre_percentage: resolution.contre_percentage(),
            abstention_percentage: resolution.abstention_percentage(),
        }
    }
}

/// Request DTO for creating a resolution
#[derive(Debug, Deserialize)]
pub struct CreateResolutionRequest {
    pub meeting_id: Uuid,
    pub title: String,
    pub description: String,
    pub resolution_type: ResolutionType,
    pub majority_required: MajorityType,
}

/// Request DTO for updating a resolution
#[derive(Debug, Deserialize)]
pub struct UpdateResolutionRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub resolution_type: Option<ResolutionType>,
    pub majority_required: Option<MajorityType>,
}

/// Request DTO for closing voting on a resolution
#[derive(Debug, Deserialize)]
pub struct CloseVotingRequest {
    pub total_voting_power: f64,
}
