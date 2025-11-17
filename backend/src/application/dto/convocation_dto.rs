use crate::domain::entities::{Convocation, ConvocationStatus, ConvocationType};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConvocationResponse {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub building_id: Uuid,
    pub meeting_id: Uuid,
    pub meeting_type: ConvocationType,
    pub meeting_date: DateTime<Utc>,
    pub status: ConvocationStatus,

    // Legal deadline tracking
    pub minimum_send_date: DateTime<Utc>,
    pub actual_send_date: Option<DateTime<Utc>>,
    pub scheduled_send_date: Option<DateTime<Utc>>,

    // PDF generation
    pub pdf_file_path: Option<String>,
    pub language: String,

    // Tracking
    pub total_recipients: i32,
    pub opened_count: i32,
    pub will_attend_count: i32,
    pub will_not_attend_count: i32,

    // Computed fields
    pub opening_rate: f64,        // opened / total * 100
    pub attendance_rate: f64,     // will_attend / total * 100
    pub days_until_meeting: i64,  // Computed from meeting_date
    pub respects_legal_deadline: bool,  // Computed: actual_send_date <= minimum_send_date

    // Reminders
    pub reminder_sent_at: Option<DateTime<Utc>>,

    // Audit
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Uuid,
}

impl From<Convocation> for ConvocationResponse {
    fn from(convocation: Convocation) -> Self {
        let opening_rate = convocation.opening_rate();
        let attendance_rate = convocation.attendance_rate();
        let days_until_meeting = convocation.days_until_meeting();
        let respects_legal_deadline = convocation.respects_legal_deadline();

        Self {
            id: convocation.id,
            organization_id: convocation.organization_id,
            building_id: convocation.building_id,
            meeting_id: convocation.meeting_id,
            meeting_type: convocation.meeting_type,
            meeting_date: convocation.meeting_date,
            status: convocation.status,
            minimum_send_date: convocation.minimum_send_date,
            actual_send_date: convocation.actual_send_date,
            scheduled_send_date: convocation.scheduled_send_date,
            pdf_file_path: convocation.pdf_file_path,
            language: convocation.language,
            total_recipients: convocation.total_recipients,
            opened_count: convocation.opened_count,
            will_attend_count: convocation.will_attend_count,
            will_not_attend_count: convocation.will_not_attend_count,
            opening_rate,
            attendance_rate,
            days_until_meeting,
            respects_legal_deadline,
            reminder_sent_at: convocation.reminder_sent_at,
            created_at: convocation.created_at,
            updated_at: convocation.updated_at,
            created_by: convocation.created_by,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateConvocationRequest {
    pub building_id: Uuid,
    pub meeting_id: Uuid,
    pub meeting_type: ConvocationType,
    pub meeting_date: DateTime<Utc>,
    pub language: String,  // FR, NL, DE, EN
}

#[derive(Debug, Deserialize)]
pub struct ScheduleConvocationRequest {
    pub send_date: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct SendConvocationRequest {
    pub recipient_owner_ids: Vec<Uuid>,  // List of owner IDs to send to
}

#[derive(Debug, Serialize)]
pub struct ConvocationSummaryResponse {
    pub id: Uuid,
    pub meeting_id: Uuid,
    pub meeting_date: DateTime<Utc>,
    pub status: ConvocationStatus,
    pub total_recipients: i32,
    pub opened_count: i32,
    pub will_attend_count: i32,
    pub days_until_meeting: i64,
}

impl From<Convocation> for ConvocationSummaryResponse {
    fn from(convocation: Convocation) -> Self {
        Self {
            id: convocation.id,
            meeting_id: convocation.meeting_id,
            meeting_date: convocation.meeting_date,
            status: convocation.status.clone(),
            total_recipients: convocation.total_recipients,
            opened_count: convocation.opened_count,
            will_attend_count: convocation.will_attend_count,
            days_until_meeting: convocation.days_until_meeting(),
        }
    }
}
