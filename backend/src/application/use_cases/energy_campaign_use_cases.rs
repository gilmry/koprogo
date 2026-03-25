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

        let avg_kwh = if !uploads.is_empty() {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::dto::{BuildingFilters, PageRequest};
    use crate::application::ports::{
        BuildingRepository, EnergyBillUploadRepository, EnergyCampaignRepository,
    };
    use crate::domain::entities::{
        Building, CampaignStatus, CampaignType, ContractType, EnergyCampaign,
        EnergyBillUpload, EnergyType, ProviderOffer,
    };
    use async_trait::async_trait;
    use chrono::Utc;
    use std::collections::HashMap;
    use std::sync::Mutex;
    use uuid::Uuid;

    // ─── Mock EnergyCampaignRepository ──────────────────────────────────

    struct MockCampaignRepo {
        campaigns: Mutex<HashMap<Uuid, EnergyCampaign>>,
        offers: Mutex<HashMap<Uuid, ProviderOffer>>,
    }

    impl MockCampaignRepo {
        fn new() -> Self {
            Self {
                campaigns: Mutex::new(HashMap::new()),
                offers: Mutex::new(HashMap::new()),
            }
        }

        fn with_campaign(campaign: EnergyCampaign) -> Self {
            let mut map = HashMap::new();
            map.insert(campaign.id, campaign);
            Self {
                campaigns: Mutex::new(map),
                offers: Mutex::new(HashMap::new()),
            }
        }

        fn with_campaign_and_offer(campaign: EnergyCampaign, offer: ProviderOffer) -> Self {
            let mut c_map = HashMap::new();
            c_map.insert(campaign.id, campaign);
            let mut o_map = HashMap::new();
            o_map.insert(offer.id, offer);
            Self {
                campaigns: Mutex::new(c_map),
                offers: Mutex::new(o_map),
            }
        }
    }

    #[async_trait]
    impl EnergyCampaignRepository for MockCampaignRepo {
        async fn create(&self, campaign: &EnergyCampaign) -> Result<EnergyCampaign, String> {
            let mut store = self.campaigns.lock().unwrap();
            store.insert(campaign.id, campaign.clone());
            Ok(campaign.clone())
        }

        async fn find_by_id(&self, id: Uuid) -> Result<Option<EnergyCampaign>, String> {
            let store = self.campaigns.lock().unwrap();
            Ok(store.get(&id).cloned())
        }

        async fn find_by_organization(
            &self,
            organization_id: Uuid,
        ) -> Result<Vec<EnergyCampaign>, String> {
            let store = self.campaigns.lock().unwrap();
            Ok(store
                .values()
                .filter(|c| c.organization_id == organization_id)
                .cloned()
                .collect())
        }

        async fn find_by_building(
            &self,
            building_id: Uuid,
        ) -> Result<Vec<EnergyCampaign>, String> {
            let store = self.campaigns.lock().unwrap();
            Ok(store
                .values()
                .filter(|c| c.building_id == Some(building_id))
                .cloned()
                .collect())
        }

        async fn update(&self, campaign: &EnergyCampaign) -> Result<EnergyCampaign, String> {
            let mut store = self.campaigns.lock().unwrap();
            store.insert(campaign.id, campaign.clone());
            Ok(campaign.clone())
        }

        async fn delete(&self, id: Uuid) -> Result<(), String> {
            let mut store = self.campaigns.lock().unwrap();
            store.remove(&id);
            Ok(())
        }

        async fn add_offer(
            &self,
            _campaign_id: Uuid,
            offer: &ProviderOffer,
        ) -> Result<ProviderOffer, String> {
            let mut store = self.offers.lock().unwrap();
            store.insert(offer.id, offer.clone());
            Ok(offer.clone())
        }

        async fn get_offers(&self, campaign_id: Uuid) -> Result<Vec<ProviderOffer>, String> {
            let store = self.offers.lock().unwrap();
            Ok(store
                .values()
                .filter(|o| o.campaign_id == campaign_id)
                .cloned()
                .collect())
        }

        async fn update_offer(&self, offer: &ProviderOffer) -> Result<ProviderOffer, String> {
            let mut store = self.offers.lock().unwrap();
            store.insert(offer.id, offer.clone());
            Ok(offer.clone())
        }

        async fn delete_offer(&self, offer_id: Uuid) -> Result<(), String> {
            let mut store = self.offers.lock().unwrap();
            store.remove(&offer_id);
            Ok(())
        }

        async fn find_offer_by_id(&self, offer_id: Uuid) -> Result<Option<ProviderOffer>, String> {
            let store = self.offers.lock().unwrap();
            Ok(store.get(&offer_id).cloned())
        }

        async fn update_aggregation(
            &self,
            _campaign_id: Uuid,
            _total_kwh_electricity: Option<f64>,
            _total_kwh_gas: Option<f64>,
            _avg_kwh_per_unit: Option<f64>,
        ) -> Result<(), String> {
            Ok(())
        }
    }

    // ─── Mock EnergyBillUploadRepository ────────────────────────────────

    struct MockUploadRepo {
        uploads: Mutex<HashMap<Uuid, EnergyBillUpload>>,
    }

    impl MockUploadRepo {
        fn new() -> Self {
            Self {
                uploads: Mutex::new(HashMap::new()),
            }
        }
    }

    #[async_trait]
    impl EnergyBillUploadRepository for MockUploadRepo {
        async fn create(&self, upload: &EnergyBillUpload) -> Result<EnergyBillUpload, String> {
            let mut store = self.uploads.lock().unwrap();
            store.insert(upload.id, upload.clone());
            Ok(upload.clone())
        }

        async fn find_by_id(&self, id: Uuid) -> Result<Option<EnergyBillUpload>, String> {
            let store = self.uploads.lock().unwrap();
            Ok(store.get(&id).cloned())
        }

        async fn find_by_campaign(
            &self,
            campaign_id: Uuid,
        ) -> Result<Vec<EnergyBillUpload>, String> {
            let store = self.uploads.lock().unwrap();
            Ok(store
                .values()
                .filter(|u| u.campaign_id == campaign_id)
                .cloned()
                .collect())
        }

        async fn find_by_unit(&self, unit_id: Uuid) -> Result<Vec<EnergyBillUpload>, String> {
            let store = self.uploads.lock().unwrap();
            Ok(store
                .values()
                .filter(|u| u.unit_id == unit_id)
                .cloned()
                .collect())
        }

        async fn find_by_campaign_and_unit(
            &self,
            campaign_id: Uuid,
            unit_id: Uuid,
        ) -> Result<Option<EnergyBillUpload>, String> {
            let store = self.uploads.lock().unwrap();
            Ok(store
                .values()
                .find(|u| u.campaign_id == campaign_id && u.unit_id == unit_id)
                .cloned())
        }

        async fn find_by_building(
            &self,
            building_id: Uuid,
        ) -> Result<Vec<EnergyBillUpload>, String> {
            let store = self.uploads.lock().unwrap();
            Ok(store
                .values()
                .filter(|u| u.building_id == building_id)
                .cloned()
                .collect())
        }

        async fn update(&self, upload: &EnergyBillUpload) -> Result<EnergyBillUpload, String> {
            let mut store = self.uploads.lock().unwrap();
            store.insert(upload.id, upload.clone());
            Ok(upload.clone())
        }

        async fn delete(&self, id: Uuid) -> Result<(), String> {
            let mut store = self.uploads.lock().unwrap();
            store.remove(&id);
            Ok(())
        }

        async fn find_expired(&self) -> Result<Vec<EnergyBillUpload>, String> {
            Ok(vec![])
        }

        async fn count_verified_by_campaign(&self, _campaign_id: Uuid) -> Result<i32, String> {
            Ok(0)
        }

        async fn find_verified_by_campaign(
            &self,
            _campaign_id: Uuid,
        ) -> Result<Vec<EnergyBillUpload>, String> {
            Ok(vec![])
        }

        async fn delete_expired(&self) -> Result<i32, String> {
            Ok(0)
        }
    }

    // ─── Mock BuildingRepository ────────────────────────────────────────

    struct MockBuildingRepo {
        buildings: Mutex<HashMap<Uuid, Building>>,
    }

    impl MockBuildingRepo {
        fn new() -> Self {
            Self {
                buildings: Mutex::new(HashMap::new()),
            }
        }

        fn with_building(building: Building) -> Self {
            let mut map = HashMap::new();
            map.insert(building.id, building);
            Self {
                buildings: Mutex::new(map),
            }
        }
    }

    #[async_trait]
    impl BuildingRepository for MockBuildingRepo {
        async fn create(&self, building: &Building) -> Result<Building, String> {
            let mut store = self.buildings.lock().unwrap();
            store.insert(building.id, building.clone());
            Ok(building.clone())
        }

        async fn find_by_id(&self, id: Uuid) -> Result<Option<Building>, String> {
            let store = self.buildings.lock().unwrap();
            Ok(store.get(&id).cloned())
        }

        async fn find_all(&self) -> Result<Vec<Building>, String> {
            let store = self.buildings.lock().unwrap();
            Ok(store.values().cloned().collect())
        }

        async fn find_all_paginated(
            &self,
            _page_request: &PageRequest,
            _filters: &BuildingFilters,
        ) -> Result<(Vec<Building>, i64), String> {
            let store = self.buildings.lock().unwrap();
            let all: Vec<Building> = store.values().cloned().collect();
            let count = all.len() as i64;
            Ok((all, count))
        }

        async fn update(&self, building: &Building) -> Result<Building, String> {
            let mut store = self.buildings.lock().unwrap();
            store.insert(building.id, building.clone());
            Ok(building.clone())
        }

        async fn delete(&self, id: Uuid) -> Result<bool, String> {
            let mut store = self.buildings.lock().unwrap();
            Ok(store.remove(&id).is_some())
        }

        async fn find_by_slug(&self, slug: &str) -> Result<Option<Building>, String> {
            let store = self.buildings.lock().unwrap();
            Ok(store
                .values()
                .find(|b| b.slug.as_deref() == Some(slug))
                .cloned())
        }
    }

    // ─── Helpers ────────────────────────────────────────────────────────

    fn make_building(id: Uuid) -> Building {
        let org_id = Uuid::new_v4();
        let mut building = Building::new(
            org_id,
            "Test Building".to_string(),
            "123 Test Street".to_string(),
            "Brussels".to_string(),
            "1000".to_string(),
            "Belgium".to_string(),
            25,
            1000,
            Some(1990),
        )
        .unwrap();
        building.id = id;
        building
    }

    fn make_campaign_for_building(building_id: Uuid) -> EnergyCampaign {
        EnergyCampaign::new(
            Uuid::new_v4(),
            Some(building_id),
            "Winter Campaign 2025".to_string(),
            Utc::now() + chrono::Duration::days(30),
            vec![EnergyType::Electricity],
            Uuid::new_v4(),
        )
        .unwrap()
    }

    fn make_negotiating_campaign(building_id: Uuid) -> EnergyCampaign {
        let mut campaign = make_campaign_for_building(building_id);
        campaign.status = CampaignStatus::Negotiating;
        campaign
    }

    fn make_offer(campaign_id: Uuid) -> ProviderOffer {
        ProviderOffer::new(
            campaign_id,
            "Lampiris".to_string(),
            Some(0.27),
            None,
            12.50,
            100.0,
            12,
            15.0,
            Utc::now() + chrono::Duration::days(30),
        )
        .unwrap()
    }

    // ─── Tests ──────────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_create_campaign_success() {
        let building_id = Uuid::new_v4();
        let building = make_building(building_id);
        let campaign = make_campaign_for_building(building_id);

        let uc = EnergyCampaignUseCases::new(
            Arc::new(MockCampaignRepo::new()),
            Arc::new(MockUploadRepo::new()),
            Arc::new(MockBuildingRepo::with_building(building)),
        );

        let result = uc.create_campaign(campaign).await;
        assert!(result.is_ok());
        let created = result.unwrap();
        assert_eq!(created.status, CampaignStatus::Draft);
        assert_eq!(created.building_id, Some(building_id));
    }

    #[tokio::test]
    async fn test_create_campaign_building_not_found() {
        let building_id = Uuid::new_v4();
        let campaign = make_campaign_for_building(building_id);

        let uc = EnergyCampaignUseCases::new(
            Arc::new(MockCampaignRepo::new()),
            Arc::new(MockUploadRepo::new()),
            Arc::new(MockBuildingRepo::new()), // Empty -- building not found
        );

        let result = uc.create_campaign(campaign).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Building not found");
    }

    #[tokio::test]
    async fn test_create_campaign_no_building_id() {
        // Campaign without building_id (multi-building) should succeed without building check
        let campaign = EnergyCampaign::new(
            Uuid::new_v4(),
            None, // No building_id
            "Multi-Building Campaign".to_string(),
            Utc::now() + chrono::Duration::days(30),
            vec![EnergyType::Both],
            Uuid::new_v4(),
        )
        .unwrap();

        let uc = EnergyCampaignUseCases::new(
            Arc::new(MockCampaignRepo::new()),
            Arc::new(MockUploadRepo::new()),
            Arc::new(MockBuildingRepo::new()),
        );

        let result = uc.create_campaign(campaign).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_campaign_success() {
        let building_id = Uuid::new_v4();
        let campaign = make_campaign_for_building(building_id);
        let campaign_id = campaign.id;

        let uc = EnergyCampaignUseCases::new(
            Arc::new(MockCampaignRepo::with_campaign(campaign)),
            Arc::new(MockUploadRepo::new()),
            Arc::new(MockBuildingRepo::new()),
        );

        let result = uc.get_campaign(campaign_id).await;
        assert!(result.is_ok());
        let found = result.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, campaign_id);
    }

    #[tokio::test]
    async fn test_get_campaign_not_found() {
        let uc = EnergyCampaignUseCases::new(
            Arc::new(MockCampaignRepo::new()),
            Arc::new(MockUploadRepo::new()),
            Arc::new(MockBuildingRepo::new()),
        );

        let result = uc.get_campaign(Uuid::new_v4()).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_add_offer_success() {
        let building_id = Uuid::new_v4();
        let campaign = make_negotiating_campaign(building_id);
        let campaign_id = campaign.id;
        let offer = make_offer(campaign_id);

        let uc = EnergyCampaignUseCases::new(
            Arc::new(MockCampaignRepo::with_campaign(campaign)),
            Arc::new(MockUploadRepo::new()),
            Arc::new(MockBuildingRepo::new()),
        );

        let result = uc.add_offer(campaign_id, offer).await;
        assert!(result.is_ok());
        let created_offer = result.unwrap();
        assert_eq!(created_offer.provider_name, "Lampiris");
    }

    #[tokio::test]
    async fn test_add_offer_wrong_status() {
        let building_id = Uuid::new_v4();
        let campaign = make_campaign_for_building(building_id); // Draft status
        let campaign_id = campaign.id;
        let offer = make_offer(campaign_id);

        let uc = EnergyCampaignUseCases::new(
            Arc::new(MockCampaignRepo::with_campaign(campaign)),
            Arc::new(MockUploadRepo::new()),
            Arc::new(MockBuildingRepo::new()),
        );

        let result = uc.add_offer(campaign_id, offer).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("Campaign must be in Negotiating status to add offers"));
    }

    #[tokio::test]
    async fn test_select_offer_success() {
        let building_id = Uuid::new_v4();
        let mut campaign = make_negotiating_campaign(building_id);
        let campaign_id = campaign.id;
        let offer = make_offer(campaign_id);
        let offer_id = offer.id;

        // Add the offer to campaign's offers_received so select_offer domain method works
        campaign.offers_received.push(offer.clone());

        let uc = EnergyCampaignUseCases::new(
            Arc::new(MockCampaignRepo::with_campaign_and_offer(campaign, offer)),
            Arc::new(MockUploadRepo::new()),
            Arc::new(MockBuildingRepo::new()),
        );

        let result = uc.select_offer(campaign_id, offer_id).await;
        assert!(result.is_ok());
        let updated = result.unwrap();
        assert_eq!(updated.selected_offer_id, Some(offer_id));
    }

    #[tokio::test]
    async fn test_select_offer_not_in_campaign() {
        let building_id = Uuid::new_v4();
        let campaign = make_negotiating_campaign(building_id);
        let campaign_id = campaign.id;

        // Create an offer that belongs to a different campaign
        let other_campaign_id = Uuid::new_v4();
        let offer = make_offer(other_campaign_id);
        let offer_id = offer.id;

        let repo = MockCampaignRepo::with_campaign_and_offer(campaign, offer);

        let uc = EnergyCampaignUseCases::new(
            Arc::new(repo),
            Arc::new(MockUploadRepo::new()),
            Arc::new(MockBuildingRepo::new()),
        );

        let result = uc.select_offer(campaign_id, offer_id).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("Offer does not belong to this campaign"));
    }

    #[tokio::test]
    async fn test_cancel_campaign_success() {
        let building_id = Uuid::new_v4();
        let campaign = make_campaign_for_building(building_id);
        let campaign_id = campaign.id;

        let uc = EnergyCampaignUseCases::new(
            Arc::new(MockCampaignRepo::with_campaign(campaign)),
            Arc::new(MockUploadRepo::new()),
            Arc::new(MockBuildingRepo::new()),
        );

        let result = uc.cancel_campaign(campaign_id).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().status, CampaignStatus::Cancelled);
    }

    #[tokio::test]
    async fn test_cancel_campaign_not_found() {
        let uc = EnergyCampaignUseCases::new(
            Arc::new(MockCampaignRepo::new()),
            Arc::new(MockUploadRepo::new()),
            Arc::new(MockBuildingRepo::new()),
        );

        let result = uc.cancel_campaign(Uuid::new_v4()).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Campaign not found");
    }

    #[tokio::test]
    async fn test_delete_campaign_success() {
        let building_id = Uuid::new_v4();
        let campaign = make_campaign_for_building(building_id);
        let campaign_id = campaign.id;

        let uc = EnergyCampaignUseCases::new(
            Arc::new(MockCampaignRepo::with_campaign(campaign)),
            Arc::new(MockUploadRepo::new()),
            Arc::new(MockBuildingRepo::new()),
        );

        let result = uc.delete_campaign(campaign_id).await;
        assert!(result.is_ok());
    }
}
