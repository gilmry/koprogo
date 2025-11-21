use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::entities::{EnergyCampaign, ProviderOffer};

/// Repository trait for energy campaign persistence
#[async_trait]
pub trait EnergyCampaignRepository: Send + Sync {
    /// Create a new energy campaign
    async fn create(&self, campaign: &EnergyCampaign) -> Result<EnergyCampaign, String>;

    /// Find campaign by ID
    async fn find_by_id(&self, id: Uuid) -> Result<Option<EnergyCampaign>, String>;

    /// Find all campaigns for an organization
    async fn find_by_organization(
        &self,
        organization_id: Uuid,
    ) -> Result<Vec<EnergyCampaign>, String>;

    /// Find all campaigns for a building
    async fn find_by_building(&self, building_id: Uuid) -> Result<Vec<EnergyCampaign>, String>;

    /// Update campaign
    async fn update(&self, campaign: &EnergyCampaign) -> Result<EnergyCampaign, String>;

    /// Delete campaign
    async fn delete(&self, id: Uuid) -> Result<(), String>;

    /// Add provider offer to campaign
    async fn add_offer(
        &self,
        campaign_id: Uuid,
        offer: &ProviderOffer,
    ) -> Result<ProviderOffer, String>;

    /// Get all offers for a campaign
    async fn get_offers(&self, campaign_id: Uuid) -> Result<Vec<ProviderOffer>, String>;

    /// Update provider offer
    async fn update_offer(&self, offer: &ProviderOffer) -> Result<ProviderOffer, String>;

    /// Delete provider offer
    async fn delete_offer(&self, offer_id: Uuid) -> Result<(), String>;

    /// Get offer by ID
    async fn find_offer_by_id(&self, offer_id: Uuid) -> Result<Option<ProviderOffer>, String>;

    /// Update campaign aggregated statistics (total_kwh, participants, etc.)
    async fn update_aggregation(
        &self,
        campaign_id: Uuid,
        total_kwh_electricity: Option<f64>,
        total_kwh_gas: Option<f64>,
        avg_kwh_per_unit: Option<f64>,
    ) -> Result<(), String>;
}
