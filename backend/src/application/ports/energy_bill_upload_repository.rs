use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::entities::EnergyBillUpload;

/// Repository trait for energy bill upload persistence
#[async_trait]
pub trait EnergyBillUploadRepository: Send + Sync {
    /// Create a new energy bill upload
    async fn create(&self, upload: &EnergyBillUpload) -> Result<EnergyBillUpload, String>;

    /// Find bill upload by ID
    async fn find_by_id(&self, id: Uuid) -> Result<Option<EnergyBillUpload>, String>;

    /// Find all uploads for a campaign
    async fn find_by_campaign(&self, campaign_id: Uuid) -> Result<Vec<EnergyBillUpload>, String>;

    /// Find all uploads for a unit
    async fn find_by_unit(&self, unit_id: Uuid) -> Result<Vec<EnergyBillUpload>, String>;

    /// Find upload for a specific campaign and unit
    async fn find_by_campaign_and_unit(
        &self,
        campaign_id: Uuid,
        unit_id: Uuid,
    ) -> Result<Option<EnergyBillUpload>, String>;

    /// Find uploads by building
    async fn find_by_building(
        &self,
        building_id: Uuid,
    ) -> Result<Vec<EnergyBillUpload>, String>;

    /// Update bill upload
    async fn update(&self, upload: &EnergyBillUpload) -> Result<EnergyBillUpload, String>;

    /// Delete bill upload (soft delete)
    async fn delete(&self, id: Uuid) -> Result<(), String>;

    /// Find uploads that need auto-deletion (retention period expired)
    async fn find_expired(&self) -> Result<Vec<EnergyBillUpload>, String>;

    /// Get count of verified uploads for a campaign (for k-anonymity check)
    async fn count_verified_by_campaign(&self, campaign_id: Uuid) -> Result<i32, String>;

    /// Find all verified, non-deleted uploads for a campaign (for aggregation)
    async fn find_verified_by_campaign(
        &self,
        campaign_id: Uuid,
    ) -> Result<Vec<EnergyBillUpload>, String>;

    /// Batch delete expired bills (GDPR auto-cleanup)
    async fn delete_expired(&self) -> Result<i32, String>;
}
