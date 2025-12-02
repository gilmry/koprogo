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
