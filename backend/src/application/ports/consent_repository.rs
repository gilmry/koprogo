use crate::domain::entities::consent::{ConsentRecord, ConsentStatus};
use async_trait::async_trait;
use uuid::Uuid;

/// Port (interface) for consent record repository
/// Handles GDPR Art. 7 consent persistence and querying
#[async_trait]
pub trait ConsentRepository: Send + Sync {
    /// Create a new consent record (append-only, immutable)
    async fn create(&self, record: &ConsentRecord) -> Result<ConsentRecord, String>;

    /// Find the latest consent of a given type for a user
    async fn find_latest_by_user_and_type(
        &self,
        user_id: Uuid,
        consent_type: &str,
    ) -> Result<Option<ConsentRecord>, String>;

    /// Find all consent records for a user (audit trail)
    async fn find_all_by_user(&self, user_id: Uuid) -> Result<Vec<ConsentRecord>, String>;

    /// Check if a user has accepted a specific consent type
    async fn has_accepted(&self, user_id: Uuid, consent_type: &str) -> Result<bool, String>;

    /// Build a consent status summary for a user
    async fn get_consent_status(&self, user_id: Uuid) -> Result<ConsentStatus, String>;
}
