// DTOs for Individual Members API (Issue #280)

use serde::{Deserialize, Serialize};
use validator::Validate;
use crate::domain::entities::IndividualMember;

/// Request DTO for joining a campaign as individual member
#[derive(Debug, Serialize, Deserialize, Validate, Clone)]
pub struct JoinCampaignAsIndividualDto {
    #[validate(email(message = "Invalid email address"))]
    pub email: String,

    #[validate(length(min = 4, max = 10, message = "Postal code must be 4-10 characters"))]
    pub postal_code: String,

    pub annual_consumption_kwh: Option<f64>,

    pub current_provider: Option<String>,

    pub ean_code: Option<String>,
}

/// Request DTO for granting GDPR consent
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GrantConsentDto {
    pub has_consent: bool,
}

/// Request DTO for updating consumption data
#[derive(Debug, Serialize, Deserialize, Validate, Clone)]
pub struct UpdateConsumptionDto {
    #[validate(range(min = 0.0, message = "Annual consumption must be non-negative"))]
    pub annual_consumption_kwh: f64,

    pub current_provider: Option<String>,

    pub ean_code: Option<String>,
}

/// Response DTO for individual member
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IndividualMemberResponseDto {
    pub id: String,
    pub campaign_id: String,
    pub email: String,
    pub postal_code: String,
    pub has_gdpr_consent: bool,
    pub consent_at: Option<String>,
    pub annual_consumption_kwh: Option<f64>,
    pub current_provider: Option<String>,
    pub ean_code: Option<String>,
    pub is_active: bool,
    pub unsubscribed_at: Option<String>,
    pub created_at: String,
}

/// Request DTO for unsubscribing from campaign (email token required in URL)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UnsubscribeRequestDto {
    pub confirm: bool,
}

/// Response DTO for unsubscribe confirmation
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UnsubscribeConfirmationDto {
    pub success: bool,
    pub message: String,
    pub email: String,
}

impl From<IndividualMember> for IndividualMemberResponseDto {
    fn from(member: IndividualMember) -> Self {
        IndividualMemberResponseDto {
            id: member.id.to_string(),
            campaign_id: member.campaign_id.to_string(),
            email: member.email,
            postal_code: member.postal_code,
            has_gdpr_consent: member.has_gdpr_consent,
            consent_at: member.consent_at.map(|dt| dt.to_rfc3339()),
            annual_consumption_kwh: member.annual_consumption_kwh,
            current_provider: member.current_provider,
            ean_code: member.ean_code,
            is_active: member.is_active(),
            unsubscribed_at: member.unsubscribed_at.map(|dt| dt.to_rfc3339()),
            created_at: member.created_at.to_rfc3339(),
        }
    }
}
