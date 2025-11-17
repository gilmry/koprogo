use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::entities::{
    ContributionPaymentMethod, ContributionPaymentStatus, ContributionType, OwnerContribution,
};

/// DTO for creating a new owner contribution
#[derive(Debug, Deserialize)]
pub struct CreateOwnerContributionRequest {
    pub owner_id: Uuid,
    pub unit_id: Option<Uuid>,
    pub description: String,
    pub amount: f64,
    pub contribution_type: ContributionType,
    pub contribution_date: DateTime<Utc>,
    pub account_code: Option<String>,
}

/// DTO for recording a payment
#[derive(Debug, Deserialize)]
pub struct RecordPaymentRequest {
    pub payment_date: DateTime<Utc>,
    pub payment_method: ContributionPaymentMethod,
    pub payment_reference: Option<String>,
}

/// DTO for owner contribution response
#[derive(Debug, Serialize)]
pub struct OwnerContributionResponse {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub owner_id: Uuid,
    pub unit_id: Option<Uuid>,
    pub description: String,
    pub amount: f64,
    pub account_code: Option<String>,
    pub contribution_type: ContributionType,
    pub contribution_date: DateTime<Utc>,
    pub payment_date: Option<DateTime<Utc>>,
    pub payment_method: Option<ContributionPaymentMethod>,
    pub payment_reference: Option<String>,
    pub payment_status: ContributionPaymentStatus,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<OwnerContribution> for OwnerContributionResponse {
    fn from(contribution: OwnerContribution) -> Self {
        Self {
            id: contribution.id,
            organization_id: contribution.organization_id,
            owner_id: contribution.owner_id,
            unit_id: contribution.unit_id,
            description: contribution.description,
            amount: contribution.amount,
            account_code: contribution.account_code,
            contribution_type: contribution.contribution_type,
            contribution_date: contribution.contribution_date,
            payment_date: contribution.payment_date,
            payment_method: contribution.payment_method,
            payment_reference: contribution.payment_reference,
            payment_status: contribution.payment_status,
            notes: contribution.notes,
            created_at: contribution.created_at,
            updated_at: contribution.updated_at,
        }
    }
}
