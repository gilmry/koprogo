use crate::domain::entities::{CallForFunds, CallForFundsStatus, ContributionType};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Request to create a new call for funds
#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(deny_unknown_fields)]
pub struct CreateCallForFundsRequest {
    pub building_id: Uuid,
    pub title: String,
    pub description: String,
    pub total_amount: rust_decimal::Decimal,
    pub contribution_type: String, // "regular", "extraordinary", "advance", "adjustment"
    pub call_date: DateTime<Utc>,
    pub due_date: DateTime<Utc>,
    pub account_code: Option<String>,
    /// Part du montant appelé affectée au fonds de réserve.
    ///
    /// Art. 3.86 § 3 al. 7 : le syndic doit la communiquer **lors de l'appel**.
    /// Absente, elle vaut zéro — ce qui reste une communication explicite,
    /// contrairement au silence d'avant.
    #[serde(default)]
    pub reserve_fund_share: rust_decimal::Decimal,
}

/// Response containing call for funds details
#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct CallForFundsResponse {
    pub id: Uuid,
    /// L'ACP propriétaire de la pièce (ADR-0045).
    pub acp_id: Uuid,
    pub organization_id: Uuid,
    pub building_id: Uuid,
    pub title: String,
    pub description: String,
    pub total_amount: rust_decimal::Decimal,
    /// Part affectée au fonds de réserve, communiquée avec l'appel
    /// (Art. 3.86 § 3 al. 7). C'est ce que le copropriétaire ne récupérera pas
    /// en vendant son lot : elle suit le lot, pas le vendeur.
    pub reserve_fund_share: rust_decimal::Decimal,
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
            acp_id: call.acp_id,
            organization_id: call.organization_id,
            building_id: call.building_id,
            title: call.title,
            description: call.description,
            total_amount: call.total_amount,
            reserve_fund_share: call.reserve_fund_share,
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
#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct SendCallForFundsRequest {
    // Empty for now, could add fields like send_date override, notification preferences, etc.
}

/// Response after sending a call for funds
#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct SendCallForFundsResponse {
    pub call_for_funds: CallForFundsResponse,
    pub contributions_generated: usize,
}
