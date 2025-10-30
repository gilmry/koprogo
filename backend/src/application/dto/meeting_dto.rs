use crate::domain::entities::{Meeting, MeetingStatus, MeetingType};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Response DTO for Meeting
#[derive(Debug, Serialize, Deserialize)]
pub struct MeetingResponse {
    pub id: Uuid,
    pub building_id: Uuid,
    pub meeting_type: MeetingType,
    pub title: String,
    pub description: Option<String>,
    pub scheduled_date: DateTime<Utc>,
    pub location: String,
    pub status: MeetingStatus,
    pub agenda: Vec<String>,
    pub attendees_count: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Meeting> for MeetingResponse {
    fn from(meeting: Meeting) -> Self {
        Self {
            id: meeting.id,
            building_id: meeting.building_id,
            meeting_type: meeting.meeting_type,
            title: meeting.title,
            description: meeting.description,
            scheduled_date: meeting.scheduled_date,
            location: meeting.location,
            status: meeting.status,
            agenda: meeting.agenda,
            attendees_count: meeting.attendees_count,
            created_at: meeting.created_at,
            updated_at: meeting.updated_at,
        }
    }
}

/// Request DTO for creating a meeting
#[derive(Debug, Deserialize)]
pub struct CreateMeetingRequest {
    pub organization_id: Uuid,
    pub building_id: Uuid,
    pub meeting_type: MeetingType,
    pub title: String,
    pub description: Option<String>,
    pub scheduled_date: DateTime<Utc>,
    pub location: String,
}

/// Request DTO for updating a meeting
#[derive(Debug, Deserialize)]
pub struct UpdateMeetingRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub scheduled_date: Option<DateTime<Utc>>,
    pub location: Option<String>,
}

/// Request DTO for adding agenda item
#[derive(Debug, Deserialize)]
pub struct AddAgendaItemRequest {
    pub item: String,
}

/// Request DTO for completing a meeting
#[derive(Debug, Deserialize)]
pub struct CompleteMeetingRequest {
    pub attendees_count: i32,
}

/// Request DTO for rescheduling a meeting
#[derive(Debug, Deserialize)]
pub struct RescheduleMeetingRequest {
    pub scheduled_date: DateTime<Utc>,
}
