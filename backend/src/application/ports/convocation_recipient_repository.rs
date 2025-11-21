use crate::domain::entities::{AttendanceStatus, ConvocationRecipient};
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait ConvocationRecipientRepository: Send + Sync {
    /// Create a new convocation recipient
    async fn create(
        &self,
        recipient: &ConvocationRecipient,
    ) -> Result<ConvocationRecipient, String>;

    /// Create multiple recipients at once (bulk insert)
    async fn create_many(
        &self,
        recipients: &[ConvocationRecipient],
    ) -> Result<Vec<ConvocationRecipient>, String>;

    /// Find recipient by ID
    async fn find_by_id(&self, id: Uuid) -> Result<Option<ConvocationRecipient>, String>;

    /// Find all recipients for a convocation
    async fn find_by_convocation(
        &self,
        convocation_id: Uuid,
    ) -> Result<Vec<ConvocationRecipient>, String>;

    /// Find recipient by convocation and owner
    async fn find_by_convocation_and_owner(
        &self,
        convocation_id: Uuid,
        owner_id: Uuid,
    ) -> Result<Option<ConvocationRecipient>, String>;

    /// Find recipients by owner (all convocations sent to this owner)
    async fn find_by_owner(&self, owner_id: Uuid) -> Result<Vec<ConvocationRecipient>, String>;

    /// Find recipients by attendance status
    async fn find_by_attendance_status(
        &self,
        convocation_id: Uuid,
        status: AttendanceStatus,
    ) -> Result<Vec<ConvocationRecipient>, String>;

    /// Find recipients who need reminder (email sent but not opened, reminder not sent yet)
    async fn find_needing_reminder(
        &self,
        convocation_id: Uuid,
    ) -> Result<Vec<ConvocationRecipient>, String>;

    /// Find recipients with failed emails
    async fn find_failed_emails(
        &self,
        convocation_id: Uuid,
    ) -> Result<Vec<ConvocationRecipient>, String>;

    /// Update recipient
    async fn update(
        &self,
        recipient: &ConvocationRecipient,
    ) -> Result<ConvocationRecipient, String>;

    /// Delete recipient
    async fn delete(&self, id: Uuid) -> Result<bool, String>;

    /// Count recipients by convocation
    async fn count_by_convocation(&self, convocation_id: Uuid) -> Result<i64, String>;

    /// Count recipients who opened email
    async fn count_opened(&self, convocation_id: Uuid) -> Result<i64, String>;

    /// Count recipients by attendance status
    async fn count_by_attendance_status(
        &self,
        convocation_id: Uuid,
        status: AttendanceStatus,
    ) -> Result<i64, String>;

    /// Get tracking summary for a convocation (opened count, attendance counts)
    async fn get_tracking_summary(
        &self,
        convocation_id: Uuid,
    ) -> Result<RecipientTrackingSummary, String>;
}

/// Tracking summary for convocation recipients
#[derive(Debug, Clone)]
pub struct RecipientTrackingSummary {
    pub total_count: i64,
    pub opened_count: i64,
    pub will_attend_count: i64,
    pub will_not_attend_count: i64,
    pub attended_count: i64,
    pub did_not_attend_count: i64,
    pub pending_count: i64,
    pub failed_email_count: i64,
}
