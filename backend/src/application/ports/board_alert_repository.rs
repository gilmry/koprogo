use crate::application::error::AppError;
use crate::domain::entities::BoardAlert;
use async_trait::async_trait;
use uuid::Uuid;

/// Port (interface) pour le repository des alertes du conseil de copropriété
/// (Story 4.7 / #582).
#[async_trait]
pub trait BoardAlertRepository: Send + Sync {
    /// Crée une nouvelle alerte.
    async fn create(&self, alert: &BoardAlert) -> Result<BoardAlert, AppError>;

    /// Liste les alertes visibles à une AG donnée (`target_meeting_id`).
    async fn find_by_target_meeting(&self, meeting_id: Uuid) -> Result<Vec<BoardAlert>, AppError>;
}
