use crate::domain::entities::{Convocation, ConvocationStatus};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[async_trait]
pub trait ConvocationRepository: Send + Sync {
    /// Create a new convocation
    async fn create(&self, convocation: &Convocation) -> Result<Convocation, String>;

    /// Find convocation by ID
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Convocation>, String>;

    /// Find convocation by meeting ID
    async fn find_by_meeting_id(&self, meeting_id: Uuid) -> Result<Option<Convocation>, String>;

    /// Find all convocations for a building
    async fn find_by_building(&self, building_id: Uuid) -> Result<Vec<Convocation>, String>;

    /// Find all convocations for an organization
    async fn find_by_organization(
        &self,
        organization_id: Uuid,
    ) -> Result<Vec<Convocation>, String>;

    /// Find convocations by status
    async fn find_by_status(
        &self,
        organization_id: Uuid,
        status: ConvocationStatus,
    ) -> Result<Vec<Convocation>, String>;

    /// Find convocations scheduled to be sent (status = Scheduled and scheduled_send_date <= now)
    async fn find_pending_scheduled(&self, now: DateTime<Utc>) -> Result<Vec<Convocation>, String>;

    /// Find sent convocations that need reminder (sent but 3 days before meeting, reminder not sent yet)
    async fn find_needing_reminder(&self, now: DateTime<Utc>) -> Result<Vec<Convocation>, String>;

    /// Update convocation
    async fn update(&self, convocation: &Convocation) -> Result<Convocation, String>;

    /// Delete convocation
    async fn delete(&self, id: Uuid) -> Result<bool, String>;

    /// Count convocations by building
    async fn count_by_building(&self, building_id: Uuid) -> Result<i64, String>;

    /// Count convocations by status
    async fn count_by_status(
        &self,
        organization_id: Uuid,
        status: ConvocationStatus,
    ) -> Result<i64, String>;
}
