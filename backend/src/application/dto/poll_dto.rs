use crate::domain::entities::{PollOption, PollStatus, PollType};
use serde::{Deserialize, Serialize};
use validator::Validate;

/// Create a new poll
#[derive(Debug, Deserialize, Validate, Clone)]
pub struct CreatePollDto {
    pub building_id: String,

    #[validate(length(min = 1, max = 255))]
    pub title: String,

    pub description: Option<String>,
    pub poll_type: PollType,
    pub options: Vec<CreatePollOptionDto>,
    pub is_anonymous: bool,
    pub allow_multiple_votes: bool,
    pub require_all_owners: bool,
    pub ends_at: String, // ISO 8601 format
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CreatePollOptionDto {
    #[serde(default)]
    pub id: Option<String>, // Optional UUID, will be generated if not provided
    pub option_text: String,
    pub attachment_url: Option<String>,
    pub display_order: i32,
}

/// Update poll (only draft polls can be updated)
#[derive(Debug, Deserialize, Validate, Clone)]
pub struct UpdatePollDto {
    #[validate(length(min = 1, max = 255))]
    pub title: Option<String>,

    pub description: Option<String>,
    pub options: Option<Vec<CreatePollOptionDto>>,
    pub is_anonymous: Option<bool>,
    pub allow_multiple_votes: Option<bool>,
    pub require_all_owners: Option<bool>,
    pub ends_at: Option<String>,
}

/// Cast a vote on a poll
#[derive(Debug, Deserialize, Validate, Clone)]
pub struct CastVoteDto {
    pub poll_id: String,

    // Only one of these should be populated based on poll_type
    pub selected_option_ids: Option<Vec<String>>, // For YesNo/MultipleChoice
    pub rating_value: Option<i32>,                 // For Rating (1-5)
    pub open_text: Option<String>,                 // For OpenEnded
}

/// Poll response DTO
#[derive(Debug, Serialize)]
pub struct PollResponseDto {
    pub id: String,
    pub building_id: String,
    pub created_by: String,
    pub title: String,
    pub description: Option<String>,
    pub poll_type: PollType,
    pub options: Vec<PollOptionDto>,
    pub is_anonymous: bool,
    pub allow_multiple_votes: bool,
    pub require_all_owners: bool,
    pub starts_at: String,
    pub ends_at: String,
    pub status: PollStatus,
    pub total_eligible_voters: i32,
    pub total_votes_cast: i32,
    pub participation_rate: f64,
    pub is_active: bool,
    pub is_ended: bool,
    pub winning_option: Option<PollOptionDto>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct PollOptionDto {
    pub id: String,
    pub option_text: String,
    pub attachment_url: Option<String>,
    pub vote_count: i32,
    pub vote_percentage: f64,
    pub display_order: i32,
}

impl From<&PollOption> for PollOptionDto {
    fn from(option: &PollOption) -> Self {
        Self {
            id: option.id.to_string(),
            option_text: option.option_text.clone(),
            attachment_url: option.attachment_url.clone(),
            vote_count: option.vote_count,
            vote_percentage: 0.0, // Will be calculated in use case
            display_order: option.display_order,
        }
    }
}

/// Poll vote response DTO
#[derive(Debug, Serialize)]
pub struct PollVoteResponseDto {
    pub id: String,
    pub poll_id: String,
    pub owner_id: Option<String>,
    pub building_id: String,
    pub selected_option_ids: Vec<String>,
    pub rating_value: Option<i32>,
    pub open_text: Option<String>,
    pub voted_at: String,
}

/// Poll list response with pagination
#[derive(Debug, Serialize)]
pub struct PollListResponseDto {
    pub polls: Vec<PollResponseDto>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
}

/// Poll results summary
#[derive(Debug, Serialize)]
pub struct PollResultsDto {
    pub poll_id: String,
    pub title: String,
    pub poll_type: PollType,
    pub status: PollStatus,
    pub total_eligible_voters: i32,
    pub total_votes_cast: i32,
    pub participation_rate: f64,
    pub options: Vec<PollOptionDto>,
    pub winning_option: Option<PollOptionDto>,
    pub closed_at: Option<String>,
}

/// Poll filters for queries
#[derive(Debug, Deserialize, Default, Clone)]
pub struct PollFilters {
    pub building_id: Option<String>,
    pub created_by: Option<String>,
    pub status: Option<PollStatus>,
    pub poll_type: Option<PollType>,
    pub ends_before: Option<String>,
    pub ends_after: Option<String>,
}
