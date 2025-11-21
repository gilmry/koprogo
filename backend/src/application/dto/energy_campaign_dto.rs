use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::entities::{
    CampaignStatus, CampaignType, ContractType, EnergyCampaign, EnergyType, ProviderOffer,
};

/// DTO for creating a new energy campaign
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct CreateEnergyCampaignRequest {
    pub building_id: Option<Uuid>,
    pub campaign_name: String,
    pub deadline_participation: DateTime<Utc>,
    pub energy_types: Vec<EnergyType>,
    pub contract_duration_months: Option<i32>,
    pub contract_type: Option<ContractType>,
}

/// DTO for energy campaign response
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct EnergyCampaignResponse {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub building_id: Option<Uuid>,
    pub campaign_name: String,
    pub campaign_type: CampaignType,
    pub status: CampaignStatus,
    pub deadline_participation: DateTime<Utc>,
    pub deadline_vote: Option<DateTime<Utc>>,
    pub contract_start_date: Option<DateTime<Utc>>,
    pub energy_types: Vec<EnergyType>,
    pub contract_duration_months: i32,
    pub contract_type: ContractType,
    pub total_participants: i32,
    pub total_kwh_electricity: Option<f64>,
    pub total_kwh_gas: Option<f64>,
    pub avg_kwh_per_unit: Option<f64>,
    pub offers_received: Vec<ProviderOfferResponse>,
    pub selected_offer_id: Option<Uuid>,
    pub estimated_savings_pct: Option<f64>,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<EnergyCampaign> for EnergyCampaignResponse {
    fn from(campaign: EnergyCampaign) -> Self {
        Self {
            id: campaign.id,
            organization_id: campaign.organization_id,
            building_id: campaign.building_id,
            campaign_name: campaign.campaign_name,
            campaign_type: campaign.campaign_type,
            status: campaign.status,
            deadline_participation: campaign.deadline_participation,
            deadline_vote: campaign.deadline_vote,
            contract_start_date: campaign.contract_start_date,
            energy_types: campaign.energy_types,
            contract_duration_months: campaign.contract_duration_months,
            contract_type: campaign.contract_type,
            total_participants: campaign.total_participants,
            total_kwh_electricity: campaign.total_kwh_electricity,
            total_kwh_gas: campaign.total_kwh_gas,
            avg_kwh_per_unit: campaign.avg_kwh_per_unit,
            offers_received: campaign
                .offers_received
                .into_iter()
                .map(ProviderOfferResponse::from)
                .collect(),
            selected_offer_id: campaign.selected_offer_id,
            estimated_savings_pct: campaign.estimated_savings_pct,
            created_by: campaign.created_by,
            created_at: campaign.created_at,
            updated_at: campaign.updated_at,
        }
    }
}

/// DTO for campaign status update
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct UpdateCampaignStatusRequest {
    pub status: CampaignStatus,
}

/// DTO for campaign statistics (anonymized)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct CampaignStatsResponse {
    pub total_participants: i32,
    pub participation_rate: f64,
    pub total_kwh_electricity: Option<f64>,
    pub total_kwh_gas: Option<f64>,
    pub avg_kwh_per_unit: Option<f64>,
    pub can_negotiate: bool,
    pub estimated_savings_pct: Option<f64>,
    pub k_anonymity_met: bool, // True if >= 5 participants
}

/// DTO for creating provider offer
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct CreateProviderOfferRequest {
    pub provider_name: String,
    pub price_kwh_electricity: Option<f64>,
    pub price_kwh_gas: Option<f64>,
    pub fixed_monthly_fee: f64,
    pub green_energy_pct: f64,
    pub contract_duration_months: i32,
    pub estimated_savings_pct: f64,
    pub offer_valid_until: DateTime<Utc>,
}

/// DTO for provider offer response
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct ProviderOfferResponse {
    pub id: Uuid,
    pub campaign_id: Uuid,
    pub provider_name: String,
    pub price_kwh_electricity: Option<f64>,
    pub price_kwh_gas: Option<f64>,
    pub fixed_monthly_fee: f64,
    pub green_energy_pct: f64,
    pub green_score: i32, // Calculated 0/5/10
    pub contract_duration_months: i32,
    pub estimated_savings_pct: f64,
    pub offer_valid_until: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<ProviderOffer> for ProviderOfferResponse {
    fn from(offer: ProviderOffer) -> Self {
        let green_score = offer.green_score();

        Self {
            id: offer.id,
            campaign_id: offer.campaign_id,
            provider_name: offer.provider_name,
            price_kwh_electricity: offer.price_kwh_electricity,
            price_kwh_gas: offer.price_kwh_gas,
            fixed_monthly_fee: offer.fixed_monthly_fee,
            green_energy_pct: offer.green_energy_pct,
            green_score,
            contract_duration_months: offer.contract_duration_months,
            estimated_savings_pct: offer.estimated_savings_pct,
            offer_valid_until: offer.offer_valid_until,
            created_at: offer.created_at,
            updated_at: offer.updated_at,
        }
    }
}

/// DTO for selecting winning offer
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct SelectOfferRequest {
    pub offer_id: Uuid,
    pub poll_id: Option<Uuid>, // Reference to voting poll (Issue #51)
}
