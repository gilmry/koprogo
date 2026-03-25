use std::sync::Arc;
use uuid::Uuid;

use crate::application::ports::{EnergyBillUploadRepository, EnergyCampaignRepository};
use crate::domain::entities::{CampaignStatus, EnergyBillUpload};

pub struct EnergyBillUploadUseCases {
    upload_repo: Arc<dyn EnergyBillUploadRepository>,
    campaign_repo: Arc<dyn EnergyCampaignRepository>,
}

impl EnergyBillUploadUseCases {
    pub fn new(
        upload_repo: Arc<dyn EnergyBillUploadRepository>,
        campaign_repo: Arc<dyn EnergyCampaignRepository>,
    ) -> Self {
        Self {
            upload_repo,
            campaign_repo,
        }
    }

    /// Upload energy bill with GDPR consent
    pub async fn upload_bill(&self, upload: EnergyBillUpload) -> Result<EnergyBillUpload, String> {
        // Validate campaign exists and is accepting uploads
        let campaign = self
            .campaign_repo
            .find_by_id(upload.campaign_id)
            .await?
            .ok_or_else(|| "Campaign not found".to_string())?;

        if campaign.status != CampaignStatus::CollectingData {
            return Err("Campaign is not collecting data".to_string());
        }

        // Check if unit already uploaded for this campaign
        let existing = self
            .upload_repo
            .find_by_campaign_and_unit(upload.campaign_id, upload.unit_id)
            .await?;

        if existing.is_some() {
            return Err("Unit has already uploaded bill for this campaign".to_string());
        }

        self.upload_repo.create(&upload).await
    }

    /// Get upload by ID
    pub async fn get_upload(&self, id: Uuid) -> Result<Option<EnergyBillUpload>, String> {
        self.upload_repo.find_by_id(id).await
    }

    /// Get all uploads for a campaign
    pub async fn get_uploads_by_campaign(
        &self,
        campaign_id: Uuid,
    ) -> Result<Vec<EnergyBillUpload>, String> {
        self.upload_repo.find_by_campaign(campaign_id).await
    }

    /// Get my uploads (for a specific unit)
    pub async fn get_my_uploads(&self, unit_id: Uuid) -> Result<Vec<EnergyBillUpload>, String> {
        self.upload_repo.find_by_unit(unit_id).await
    }

    /// Verify upload (manual verification by admin)
    pub async fn verify_upload(
        &self,
        upload_id: Uuid,
        verified_by: Uuid,
    ) -> Result<EnergyBillUpload, String> {
        let mut upload = self
            .upload_repo
            .find_by_id(upload_id)
            .await?
            .ok_or_else(|| "Upload not found".to_string())?;

        upload.mark_verified(verified_by)?;
        self.upload_repo.update(&upload).await
    }

    /// Anonymize upload (add to building aggregate)
    pub async fn anonymize_upload(&self, upload_id: Uuid) -> Result<EnergyBillUpload, String> {
        let mut upload = self
            .upload_repo
            .find_by_id(upload_id)
            .await?
            .ok_or_else(|| "Upload not found".to_string())?;

        upload.anonymize()?;
        self.upload_repo.update(&upload).await
    }

    /// Batch anonymize all verified uploads for a campaign
    pub async fn batch_anonymize_campaign(&self, campaign_id: Uuid) -> Result<i32, String> {
        let uploads = self
            .upload_repo
            .find_verified_by_campaign(campaign_id)
            .await?;

        let mut count = 0;
        for mut upload in uploads {
            if !upload.anonymized {
                if upload.anonymize().is_ok() {
                    self.upload_repo.update(&upload).await?;
                    count += 1;
                }
            }
        }

        Ok(count)
    }

    /// Decrypt consumption data (requires encryption key and ownership)
    pub async fn decrypt_consumption(
        &self,
        upload_id: Uuid,
        requester_unit_id: Uuid,
        encryption_key: &[u8; 32],
    ) -> Result<f64, String> {
        let upload = self
            .upload_repo
            .find_by_id(upload_id)
            .await?
            .ok_or_else(|| "Upload not found".to_string())?;

        // Verify requester owns this unit (authorization check)
        if upload.unit_id != requester_unit_id {
            return Err("Unauthorized: You can only access your own data".to_string());
        }

        upload.decrypt_kwh(encryption_key)
    }

    /// Delete upload (GDPR Art. 17 - Right to erasure)
    pub async fn delete_upload(
        &self,
        upload_id: Uuid,
        requester_unit_id: Uuid,
    ) -> Result<(), String> {
        let mut upload = self
            .upload_repo
            .find_by_id(upload_id)
            .await?
            .ok_or_else(|| "Upload not found".to_string())?;

        // Verify requester owns this unit
        if upload.unit_id != requester_unit_id {
            return Err("Unauthorized: You can only delete your own data".to_string());
        }

        upload.delete()?;
        self.upload_repo.update(&upload).await?;
        Ok(())
    }

    /// Withdraw consent (GDPR Art. 7.3 - Immediate deletion)
    pub async fn withdraw_consent(
        &self,
        upload_id: Uuid,
        requester_unit_id: Uuid,
    ) -> Result<(), String> {
        let mut upload = self
            .upload_repo
            .find_by_id(upload_id)
            .await?
            .ok_or_else(|| "Upload not found".to_string())?;

        // Verify requester owns this unit
        if upload.unit_id != requester_unit_id {
            return Err("Unauthorized: You can only withdraw your own consent".to_string());
        }

        upload.withdraw_consent()?;
        self.upload_repo.update(&upload).await?;
        Ok(())
    }

    /// Get count of verified uploads for a campaign (k-anonymity check)
    pub async fn get_verified_count(&self, campaign_id: Uuid) -> Result<i32, String> {
        self.upload_repo
            .count_verified_by_campaign(campaign_id)
            .await
    }

    /// Check if k-anonymity threshold is met (minimum 5 participants)
    pub async fn check_k_anonymity(&self, campaign_id: Uuid) -> Result<bool, String> {
        let count = self.get_verified_count(campaign_id).await?;
        Ok(count >= 5)
    }

    /// Auto-delete expired uploads (GDPR retention policy)
    pub async fn cleanup_expired(&self) -> Result<i32, String> {
        self.upload_repo.delete_expired().await
    }

    /// Get expired uploads count (for reporting)
    pub async fn get_expired_count(&self) -> Result<usize, String> {
        let expired = self.upload_repo.find_expired().await?;
        Ok(expired.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::ports::{EnergyBillUploadRepository, EnergyCampaignRepository};
    use crate::domain::entities::{
        CampaignStatus, CampaignType, ContractType, EnergyBillUpload, EnergyCampaign, EnergyType,
        ProviderOffer,
    };
    use async_trait::async_trait;
    use chrono::Utc;
    use std::collections::HashMap;
    use std::sync::Mutex;
    use uuid::Uuid;

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

        fn with_upload(upload: EnergyBillUpload) -> Self {
            let mut map = HashMap::new();
            map.insert(upload.id, upload);
            Self {
                uploads: Mutex::new(map),
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
            let store = self.uploads.lock().unwrap();
            Ok(store
                .values()
                .filter(|u| u.should_auto_delete())
                .cloned()
                .collect())
        }

        async fn count_verified_by_campaign(&self, campaign_id: Uuid) -> Result<i32, String> {
            let store = self.uploads.lock().unwrap();
            Ok(store
                .values()
                .filter(|u| u.campaign_id == campaign_id && u.verified_at.is_some())
                .count() as i32)
        }

        async fn find_verified_by_campaign(
            &self,
            campaign_id: Uuid,
        ) -> Result<Vec<EnergyBillUpload>, String> {
            let store = self.uploads.lock().unwrap();
            Ok(store
                .values()
                .filter(|u| u.campaign_id == campaign_id && u.verified_at.is_some())
                .cloned()
                .collect())
        }

        async fn delete_expired(&self) -> Result<i32, String> {
            let mut store = self.uploads.lock().unwrap();
            let expired_ids: Vec<Uuid> = store
                .values()
                .filter(|u| u.should_auto_delete())
                .map(|u| u.id)
                .collect();
            let count = expired_ids.len() as i32;
            for id in expired_ids {
                store.remove(&id);
            }
            Ok(count)
        }
    }

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

        async fn find_by_building(&self, building_id: Uuid) -> Result<Vec<EnergyCampaign>, String> {
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

    // ─── Helpers ────────────────────────────────────────────────────────

    fn get_test_encryption_key() -> [u8; 32] {
        *b"test_master_key_for_32bytes!##!!"
    }

    fn make_collecting_campaign() -> EnergyCampaign {
        let mut campaign = EnergyCampaign::new(
            Uuid::new_v4(),
            Some(Uuid::new_v4()),
            "Winter Campaign 2025".to_string(),
            Utc::now() + chrono::Duration::days(30),
            vec![EnergyType::Electricity],
            Uuid::new_v4(),
        )
        .unwrap();
        // Move to CollectingData status
        campaign.status = CampaignStatus::CollectingData;
        campaign
    }

    fn make_upload(campaign_id: Uuid, unit_id: Uuid, building_id: Uuid) -> EnergyBillUpload {
        let key = get_test_encryption_key();
        EnergyBillUpload::new(
            campaign_id,
            unit_id,
            building_id,
            Uuid::new_v4(),
            Utc::now() - chrono::Duration::days(365),
            Utc::now(),
            2400.0,
            EnergyType::Electricity,
            "1050".to_string(),
            "abc123hash".to_string(),
            "/encrypted/path".to_string(),
            Uuid::new_v4(),
            "192.168.1.1".to_string(),
            "Mozilla/5.0".to_string(),
            &key,
        )
        .unwrap()
    }

    // ─── Tests ──────────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_upload_bill_success() {
        let campaign = make_collecting_campaign();
        let campaign_id = campaign.id;
        let building_id = campaign.building_id.unwrap();
        let unit_id = Uuid::new_v4();

        let upload = make_upload(campaign_id, unit_id, building_id);

        let uc = EnergyBillUploadUseCases::new(
            Arc::new(MockUploadRepo::new()),
            Arc::new(MockCampaignRepo::with_campaign(campaign)),
        );

        let result = uc.upload_bill(upload).await;
        assert!(result.is_ok());
        let created = result.unwrap();
        assert_eq!(created.campaign_id, campaign_id);
        assert_eq!(created.unit_id, unit_id);
    }

    #[tokio::test]
    async fn test_upload_bill_campaign_not_collecting() {
        let mut campaign = make_collecting_campaign();
        campaign.status = CampaignStatus::Draft; // Not collecting
        let campaign_id = campaign.id;
        let building_id = campaign.building_id.unwrap();
        let unit_id = Uuid::new_v4();

        let upload = make_upload(campaign_id, unit_id, building_id);

        let uc = EnergyBillUploadUseCases::new(
            Arc::new(MockUploadRepo::new()),
            Arc::new(MockCampaignRepo::with_campaign(campaign)),
        );

        let result = uc.upload_bill(upload).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Campaign is not collecting data");
    }

    #[tokio::test]
    async fn test_upload_bill_duplicate_unit() {
        let campaign = make_collecting_campaign();
        let campaign_id = campaign.id;
        let building_id = campaign.building_id.unwrap();
        let unit_id = Uuid::new_v4();

        // Pre-populate the repo with an existing upload for the same unit+campaign
        let existing_upload = make_upload(campaign_id, unit_id, building_id);
        let upload_repo = MockUploadRepo::with_upload(existing_upload);

        let new_upload = make_upload(campaign_id, unit_id, building_id);

        let uc = EnergyBillUploadUseCases::new(
            Arc::new(upload_repo),
            Arc::new(MockCampaignRepo::with_campaign(campaign)),
        );

        let result = uc.upload_bill(new_upload).await;
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Unit has already uploaded bill for this campaign"
        );
    }

    #[tokio::test]
    async fn test_get_upload_success() {
        let campaign_id = Uuid::new_v4();
        let unit_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let upload = make_upload(campaign_id, unit_id, building_id);
        let upload_id = upload.id;

        let uc = EnergyBillUploadUseCases::new(
            Arc::new(MockUploadRepo::with_upload(upload)),
            Arc::new(MockCampaignRepo::new()),
        );

        let result = uc.get_upload(upload_id).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_some());
    }

    #[tokio::test]
    async fn test_get_upload_not_found() {
        let uc = EnergyBillUploadUseCases::new(
            Arc::new(MockUploadRepo::new()),
            Arc::new(MockCampaignRepo::new()),
        );

        let result = uc.get_upload(Uuid::new_v4()).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_verify_upload_success() {
        let campaign_id = Uuid::new_v4();
        let unit_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let upload = make_upload(campaign_id, unit_id, building_id);
        let upload_id = upload.id;
        let verifier_id = Uuid::new_v4();

        let uc = EnergyBillUploadUseCases::new(
            Arc::new(MockUploadRepo::with_upload(upload)),
            Arc::new(MockCampaignRepo::new()),
        );

        let result = uc.verify_upload(upload_id, verifier_id).await;
        assert!(result.is_ok());
        let verified = result.unwrap();
        assert!(verified.manually_verified);
        assert_eq!(verified.verified_by, Some(verifier_id));
    }

    #[tokio::test]
    async fn test_verify_upload_not_found() {
        let uc = EnergyBillUploadUseCases::new(
            Arc::new(MockUploadRepo::new()),
            Arc::new(MockCampaignRepo::new()),
        );

        let result = uc.verify_upload(Uuid::new_v4(), Uuid::new_v4()).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Upload not found");
    }

    #[tokio::test]
    async fn test_delete_upload_success() {
        let campaign_id = Uuid::new_v4();
        let unit_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let upload = make_upload(campaign_id, unit_id, building_id);
        let upload_id = upload.id;

        let uc = EnergyBillUploadUseCases::new(
            Arc::new(MockUploadRepo::with_upload(upload)),
            Arc::new(MockCampaignRepo::new()),
        );

        let result = uc.delete_upload(upload_id, unit_id).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_delete_upload_unauthorized() {
        let campaign_id = Uuid::new_v4();
        let unit_id = Uuid::new_v4();
        let other_unit_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let upload = make_upload(campaign_id, unit_id, building_id);
        let upload_id = upload.id;

        let uc = EnergyBillUploadUseCases::new(
            Arc::new(MockUploadRepo::with_upload(upload)),
            Arc::new(MockCampaignRepo::new()),
        );

        let result = uc.delete_upload(upload_id, other_unit_id).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("Unauthorized: You can only delete your own data"));
    }

    #[tokio::test]
    async fn test_check_k_anonymity_met() {
        let campaign_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();

        // Create 5 verified uploads
        let mut upload_map = HashMap::new();
        for _ in 0..5 {
            let mut upload = make_upload(campaign_id, Uuid::new_v4(), building_id);
            upload.mark_verified(Uuid::new_v4()).unwrap();
            upload_map.insert(upload.id, upload);
        }
        let upload_repo = MockUploadRepo {
            uploads: Mutex::new(upload_map),
        };

        let uc =
            EnergyBillUploadUseCases::new(Arc::new(upload_repo), Arc::new(MockCampaignRepo::new()));

        let result = uc.check_k_anonymity(campaign_id).await;
        assert!(result.is_ok());
        assert!(result.unwrap()); // k >= 5
    }

    #[tokio::test]
    async fn test_check_k_anonymity_not_met() {
        let campaign_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();

        // Create only 3 verified uploads (below threshold of 5)
        let mut upload_map = HashMap::new();
        for _ in 0..3 {
            let mut upload = make_upload(campaign_id, Uuid::new_v4(), building_id);
            upload.mark_verified(Uuid::new_v4()).unwrap();
            upload_map.insert(upload.id, upload);
        }
        let upload_repo = MockUploadRepo {
            uploads: Mutex::new(upload_map),
        };

        let uc =
            EnergyBillUploadUseCases::new(Arc::new(upload_repo), Arc::new(MockCampaignRepo::new()));

        let result = uc.check_k_anonymity(campaign_id).await;
        assert!(result.is_ok());
        assert!(!result.unwrap()); // k < 5
    }
}
