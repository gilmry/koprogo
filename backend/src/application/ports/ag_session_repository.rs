use crate::domain::entities::ag_session::AgSession;
use async_trait::async_trait;
use uuid::Uuid;

/// Port (interface) pour le repository des sessions AG visioconférence
#[async_trait]
pub trait AgSessionRepository: Send + Sync {
    async fn create(&self, session: &AgSession) -> Result<AgSession, String>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<AgSession>, String>;
    async fn find_by_meeting_id(&self, meeting_id: Uuid) -> Result<Option<AgSession>, String>;
    async fn find_by_organization(&self, organization_id: Uuid) -> Result<Vec<AgSession>, String>;
    async fn update(&self, session: &AgSession) -> Result<AgSession, String>;
    async fn delete(&self, id: Uuid) -> Result<bool, String>;
    /// Trouve les sessions planifiées dont la date de début est passée (à démarrer)
    async fn find_pending_start(&self) -> Result<Vec<AgSession>, String>;
}
