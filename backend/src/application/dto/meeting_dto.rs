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
    // Quorum — Art. 3.87 §5 CC
    pub quorum_validated: bool,
    pub quorum_percentage: Option<f64>,
    pub total_quotas: Option<f64>,
    pub present_quotas: Option<f64>,
    pub is_second_convocation: bool,
    pub minutes_document_id: Option<Uuid>,
    pub minutes_sent_at: Option<DateTime<Utc>>,
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
            quorum_validated: meeting.quorum_validated,
            quorum_percentage: meeting.quorum_percentage,
            total_quotas: meeting.total_quotas,
            present_quotas: meeting.present_quotas,
            is_second_convocation: meeting.is_second_convocation,
            minutes_document_id: meeting.minutes_document_id,
            minutes_sent_at: meeting.minutes_sent_at,
            created_at: meeting.created_at,
            updated_at: meeting.updated_at,
        }
    }
}

/// Request DTO for validating quorum (Art. 3.87 §5 CC)
#[derive(Debug, Deserialize)]
pub struct ValidateQuorumRequest {
    /// Millièmes présents + représentés par procuration
    pub present_quotas: f64,
    /// Total millièmes du bâtiment (généralement 1000)
    pub total_quotas: f64,
}

/// Request DTO for creating a meeting
#[derive(Debug, Deserialize)]
pub struct CreateMeetingRequest {
    #[serde(default)]
    pub organization_id: Uuid, // Will be overridden by JWT token
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

/// Request DTO for attaching minutes document to a completed meeting (Issue #313)
#[derive(Debug, Deserialize)]
pub struct AttachMinutesRequest {
    pub document_id: Uuid,
}

/// Request DTO for scheduling a second convocation (Issue #311)
#[derive(Debug, Deserialize)]
pub struct ScheduleSecondConvocationRequest {
    pub building_id: Uuid,
    pub first_meeting_id: Uuid,
    pub new_meeting_date: DateTime<Utc>,
    pub language: String,
}
