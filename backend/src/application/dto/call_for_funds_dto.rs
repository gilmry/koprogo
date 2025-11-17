use crate::domain::entities::{CallForFunds, CallForFundsStatus, ContributionType};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Request to create a new call for funds
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateCallForFundsRequest {
    pub building_id: Uuid,
    pub title: String,
    pub description: String,
    pub total_amount: f64,
    pub contribution_type: String, // "regular", "extraordinary", "advance", "adjustment"
    pub call_date: DateTime<Utc>,
    pub due_date: DateTime<Utc>,
    pub account_code: Option<String>,
}

/// Response containing call for funds details
#[derive(Debug, Serialize, Deserialize)]
pub struct CallForFundsResponse {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub building_id: Uuid,
    pub title: String,
    pub description: String,
    pub total_amount: f64,
    pub contribution_type: String,
    pub call_date: DateTime<Utc>,
    pub due_date: DateTime<Utc>,
    pub sent_date: Option<DateTime<Utc>>,
    pub status: String,
    pub account_code: Option<String>,
    pub notes: Option<String>,
    pub is_overdue: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<Uuid>,
}

impl From<CallForFunds> for CallForFundsResponse {
    fn from(call: CallForFunds) -> Self {
        let status_str = match call.status {
            CallForFundsStatus::Draft => "draft",
            CallForFundsStatus::Sent => "sent",
            CallForFundsStatus::Partial => "partial",
            CallForFundsStatus::Completed => "completed",
            CallForFundsStatus::Cancelled => "cancelled",
        };

        let contribution_type_str = match call.contribution_type {
            ContributionType::Regular => "regular",
            ContributionType::Extraordinary => "extraordinary",
            ContributionType::Advance => "advance",
            ContributionType::Adjustment => "adjustment",
        };

        let is_overdue = call.is_overdue();

        Self {
            id: call.id,
            organization_id: call.organization_id,
            building_id: call.building_id,
            title: call.title,
            description: call.description,
            total_amount: call.total_amount,
            contribution_type: contribution_type_str.to_string(),
            call_date: call.call_date,
            due_date: call.due_date,
            sent_date: call.sent_date,
            status: status_str.to_string(),
            account_code: call.account_code,
            notes: call.notes,
            is_overdue,
            created_at: call.created_at,
            updated_at: call.updated_at,
            created_by: call.created_by,
        }
    }
}

/// Request to send a call for funds (triggers automatic contribution generation)
#[derive(Debug, Serialize, Deserialize)]
pub struct SendCallForFundsRequest {
    // Empty for now, could add fields like send_date override, notification preferences, etc.
}

/// Response after sending a call for funds
#[derive(Debug, Serialize, Deserialize)]
pub struct SendCallForFundsResponse {
    pub call_for_funds: CallForFundsResponse,
    pub contributions_generated: usize,
}
