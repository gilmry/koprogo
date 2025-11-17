use crate::domain::entities::{AttendanceStatus, ConvocationRecipient};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConvocationRecipientResponse {
    pub id: Uuid,
    pub convocation_id: Uuid,
    pub owner_id: Uuid,
    pub email: String,

    // Email tracking
    pub email_sent_at: Option<DateTime<Utc>>,
    pub email_opened_at: Option<DateTime<Utc>>,
    pub email_failed: bool,
    pub email_failure_reason: Option<String>,

    // Reminder tracking
    pub reminder_sent_at: Option<DateTime<Utc>>,
    pub reminder_opened_at: Option<DateTime<Utc>>,

    // Attendance tracking
    pub attendance_status: AttendanceStatus,
    pub attendance_updated_at: Option<DateTime<Utc>>,

    // Proxy delegation
    pub proxy_owner_id: Option<Uuid>,

    // Computed fields
    pub has_opened_email: bool,
    pub has_opened_reminder: bool,
    pub needs_reminder: bool,
    pub has_confirmed_attendance: bool,

    // Audit
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<ConvocationRecipient> for ConvocationRecipientResponse {
    fn from(recipient: ConvocationRecipient) -> Self {
        let has_opened_email = recipient.has_opened_email();
        let has_opened_reminder = recipient.has_opened_reminder();
        let needs_reminder = recipient.needs_reminder();
        let has_confirmed_attendance = recipient.has_confirmed_attendance();

        Self {
            id: recipient.id,
            convocation_id: recipient.convocation_id,
            owner_id: recipient.owner_id,
            email: recipient.email,
            email_sent_at: recipient.email_sent_at,
            email_opened_at: recipient.email_opened_at,
            email_failed: recipient.email_failed,
            email_failure_reason: recipient.email_failure_reason,
            reminder_sent_at: recipient.reminder_sent_at,
            reminder_opened_at: recipient.reminder_opened_at,
            attendance_status: recipient.attendance_status,
            attendance_updated_at: recipient.attendance_updated_at,
            proxy_owner_id: recipient.proxy_owner_id,
            has_opened_email,
            has_opened_reminder,
            needs_reminder,
            has_confirmed_attendance,
            created_at: recipient.created_at,
            updated_at: recipient.updated_at,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct UpdateAttendanceRequest {
    pub attendance_status: AttendanceStatus,
}

#[derive(Debug, Deserialize)]
pub struct SetProxyRequest {
    pub proxy_owner_id: Uuid,
}

#[derive(Debug, Serialize)]
pub struct RecipientSummaryResponse {
    pub id: Uuid,
    pub owner_id: Uuid,
    pub email: String,
    pub email_opened: bool,
    pub attendance_status: AttendanceStatus,
}

impl From<ConvocationRecipient> for RecipientSummaryResponse {
    fn from(recipient: ConvocationRecipient) -> Self {
        Self {
            id: recipient.id,
            owner_id: recipient.owner_id,
            email: recipient.email,
            email_opened: recipient.has_opened_email(),
            attendance_status: recipient.attendance_status,
        }
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct RecipientTrackingSummaryResponse {
    pub total_count: i64,
    pub opened_count: i64,
    pub will_attend_count: i64,
    pub will_not_attend_count: i64,
    pub attended_count: i64,
    pub did_not_attend_count: i64,
    pub pending_count: i64,
    pub failed_email_count: i64,

    // Computed percentages
    pub opening_rate: f64,
    pub attendance_rate: f64,
}

impl RecipientTrackingSummaryResponse {
    pub fn new(
        total_count: i64,
        opened_count: i64,
        will_attend_count: i64,
        will_not_attend_count: i64,
        attended_count: i64,
        did_not_attend_count: i64,
        pending_count: i64,
        failed_email_count: i64,
    ) -> Self {
        let opening_rate = if total_count > 0 {
            (opened_count as f64 / total_count as f64) * 100.0
        } else {
            0.0
        };

        let attendance_rate = if total_count > 0 {
            (will_attend_count as f64 / total_count as f64) * 100.0
        } else {
            0.0
        };

        Self {
            total_count,
            opened_count,
            will_attend_count,
            will_not_attend_count,
            attended_count,
            did_not_attend_count,
            pending_count,
            failed_email_count,
            opening_rate,
            attendance_rate,
        }
    }
}
