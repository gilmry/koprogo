use std::sync::Arc;
use uuid::Uuid;

use crate::application::ports::{
    BuildingRepository, EnergyBillUploadRepository, EnergyCampaignRepository,
};
use crate::domain::entities::{CampaignStatus, EnergyCampaign, ProviderOffer};

pub struct EnergyCampaignUseCases {
    campaign_repo: Arc<dyn EnergyCampaignRepository>,
    bill_upload_repo: Arc<dyn EnergyBillUploadRepository>,
    building_repo: Arc<dyn BuildingRepository>,
}

impl EnergyCampaignUseCases {
    pub fn new(
        campaign_repo: Arc<dyn EnergyCampaignRepository>,
        bill_upload_repo: Arc<dyn EnergyBillUploadRepository>,
        building_repo: Arc<dyn BuildingRepository>,
    ) -> Self {
        Self {
            campaign_repo,
            bill_upload_repo,
            building_repo,
        }
    }

    /// Create a new energy campaign
    pub async fn create_campaign(
        &self,
        campaign: EnergyCampaign,
    ) -> Result<EnergyCampaign, String> {
        // Validate building exists if building_id is provided
        if let Some(building_id) = campaign.building_id {
            let building = self
                .building_repo
                .find_by_id(building_id)
                .await
                .map_err(|e| format!("Failed to validate building: {}", e))?;

            if building.is_none() {
                return Err("Building not found".to_string());
            }
        }

        self.campaign_repo.create(&campaign).await
    }

    /// Get campaign by ID
    pub async fn get_campaign(&self, id: Uuid) -> Result<Option<EnergyCampaign>, String> {
        self.campaign_repo.find_by_id(id).await
    }

    /// Get all campaigns for an organization
    pub async fn get_campaigns_by_organization(
        &self,
        organization_id: Uuid,
    ) -> Result<Vec<EnergyCampaign>, String> {
        self.campaign_repo
            .find_by_organization(organization_id)
            .await
    }

    /// Get all campaigns for a building
    pub async fn get_campaigns_by_building(
        &self,
        building_id: Uuid,
    ) -> Result<Vec<EnergyCampaign>, String> {
        self.campaign_repo.find_by_building(building_id).await
    }

    /// Update campaign status
    pub async fn update_campaign_status(
        &self,
        id: Uuid,
        new_status: CampaignStatus,
    ) -> Result<EnergyCampaign, String> {
        let mut campaign = self
            .campaign_repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| "Campaign not found".to_string())?;

        campaign.status = new_status;
        self.campaign_repo.update(&campaign).await
    }

    /// Add provider offer to campaign
    pub async fn add_offer(
        &self,
        campaign_id: Uuid,
        offer: ProviderOffer,
    ) -> Result<ProviderOffer, String> {
        // Verify campaign exists and is in Negotiating status
        let campaign = self
            .campaign_repo
            .find_by_id(campaign_id)
            .await?
            .ok_or_else(|| "Campaign not found".to_string())?;

        if campaign.status != CampaignStatus::Negotiating {
            return Err("Campaign must be in Negotiating status to add offers".to_string());
        }

        self.campaign_repo.add_offer(campaign_id, &offer).await
    }

    /// Get all offers for a campaign
    pub async fn get_campaign_offers(
        &self,
        campaign_id: Uuid,
    ) -> Result<Vec<ProviderOffer>, String> {
        self.campaign_repo.get_offers(campaign_id).await
    }

    /// Select winning offer
    pub async fn select_offer(
        &self,
        campaign_id: Uuid,
        offer_id: Uuid,
    ) -> Result<EnergyCampaign, String> {
        let mut campaign = self
            .campaign_repo
            .find_by_id(campaign_id)
            .await?
            .ok_or_else(|| "Campaign not found".to_string())?;

        // Validate offer exists
        let offer = self
            .campaign_repo
            .find_offer_by_id(offer_id)
            .await?
            .ok_or_else(|| "Offer not found".to_string())?;

        if offer.campaign_id != campaign_id {
            return Err("Offer does not belong to this campaign".to_string());
        }

        campaign.select_offer(offer_id)?;
        self.campaign_repo.update(&campaign).await
    }

    /// Calculate and update campaign aggregations (called after bill uploads)
    pub async fn update_campaign_aggregation(
        &self,
        campaign_id: Uuid,
        encryption_key: &[u8; 32],
    ) -> Result<(), String> {
        // Get all verified bill uploads for this campaign
        let uploads = self
            .bill_upload_repo
            .find_verified_by_campaign(campaign_id)
            .await?;

        if uploads.is_empty() {
            return Ok(());
        }

        // Decrypt and aggregate consumption data
        let mut total_kwh_electricity = 0.0;
        let mut total_kwh_gas = 0.0;
        let mut count_electricity = 0;
        let mut count_gas = 0;

        for upload in &uploads {
            let kwh = upload.decrypt_kwh(encryption_key)?;

            match upload.energy_type {
                crate::domain::entities::EnergyType::Electricity => {
                    total_kwh_electricity += kwh;
                    count_electricity += 1;
                }
                crate::domain::entities::EnergyType::Gas => {
                    total_kwh_gas += kwh;
                    count_gas += 1;
                }
                crate::domain::entities::EnergyType::Both => {
                    // For "Both", split 50/50 (simplified logic)
                    total_kwh_electricity += kwh / 2.0;
                    total_kwh_gas += kwh / 2.0;
                    count_electricity += 1;
                    count_gas += 1;
                }
            }
        }

        let total_kwh_elec = if count_electricity > 0 {
            Some(total_kwh_electricity)
        } else {
            None
        };

        let total_kwh_g = if count_gas > 0 {
            Some(total_kwh_gas)
        } else {
            None
        };

        let avg_kwh = if uploads.len() > 0 {
            Some((total_kwh_electricity + total_kwh_gas) / uploads.len() as f64)
        } else {
            None
        };

        // Update campaign with aggregated data
        self.campaign_repo
            .update_aggregation(campaign_id, total_kwh_elec, total_kwh_g, avg_kwh)
            .await
    }

    /// Get campaign statistics (anonymized)
    pub async fn get_campaign_stats(&self, campaign_id: Uuid) -> Result<CampaignStats, String> {
        let campaign = self
            .campaign_repo
            .find_by_id(campaign_id)
            .await?
            .ok_or_else(|| "Campaign not found".to_string())?;

        // Get building total units if building_id is set
        let total_units = if let Some(building_id) = campaign.building_id {
            let building = self
                .building_repo
                .find_by_id(building_id)
                .await?
                .ok_or_else(|| "Building not found".to_string())?;
            building.total_units
        } else {
            // For multi-building campaigns, we'd need to sum all buildings
            0
        };

        let participation_rate = if total_units > 0 {
            campaign.participation_rate(total_units)
        } else {
            0.0
        };

        let can_negotiate = if total_units > 0 {
            campaign.can_negotiate(total_units)
        } else {
            false
        };

        Ok(CampaignStats {
            total_participants: campaign.total_participants,
            participation_rate,
            total_kwh_electricity: campaign.total_kwh_electricity,
            total_kwh_gas: campaign.total_kwh_gas,
            avg_kwh_per_unit: campaign.avg_kwh_per_unit,
            can_negotiate,
            estimated_savings_pct: campaign.estimated_savings_pct,
        })
    }

    /// Finalize campaign (after final vote)
    pub async fn finalize_campaign(&self, id: Uuid) -> Result<EnergyCampaign, String> {
        let mut campaign = self
            .campaign_repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| "Campaign not found".to_string())?;

        campaign.finalize()?;
        self.campaign_repo.update(&campaign).await
    }

    /// Complete campaign (contracts signed)
    pub async fn complete_campaign(&self, id: Uuid) -> Result<EnergyCampaign, String> {
        let mut campaign = self
            .campaign_repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| "Campaign not found".to_string())?;

        campaign.complete()?;
        self.campaign_repo.update(&campaign).await
    }

    /// Cancel campaign
    pub async fn cancel_campaign(&self, id: Uuid) -> Result<EnergyCampaign, String> {
        let mut campaign = self
            .campaign_repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| "Campaign not found".to_string())?;

        campaign.cancel()?;
        self.campaign_repo.update(&campaign).await
    }

    /// Delete campaign
    pub async fn delete_campaign(&self, id: Uuid) -> Result<(), String> {
        self.campaign_repo.delete(id).await
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct CampaignStats {
    pub total_participants: i32,
    pub participation_rate: f64,
    pub total_kwh_electricity: Option<f64>,
    pub total_kwh_gas: Option<f64>,
    pub avg_kwh_per_unit: Option<f64>,
    pub can_negotiate: bool,
    pub estimated_savings_pct: Option<f64>,
}
