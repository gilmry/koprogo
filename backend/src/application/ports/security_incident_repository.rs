use crate::domain::entities::SecurityIncident;
use async_trait::async_trait;
use uuid::Uuid;

/// Filtres pour la liste des incidents
pub struct SecurityIncidentFilters {
    pub severity: Option<String>,
    pub status: Option<String>,
    pub page: i64,
    pub per_page: i64,
}

/// Port (interface) pour le repository des incidents de sécurité (GDPR Art. 33)
#[async_trait]
pub trait SecurityIncidentRepository: Send + Sync {
    async fn create(&self, incident: &SecurityIncident) -> Result<SecurityIncident, String>;

    async fn find_by_id(
        &self,
        id: Uuid,
        organization_id: Option<Uuid>,
    ) -> Result<Option<SecurityIncident>, String>;

    async fn find_all(
        &self,
        organization_id: Option<Uuid>,
        filters: SecurityIncidentFilters,
    ) -> Result<(Vec<SecurityIncident>, i64), String>;

    async fn report_to_apd(
        &self,
        id: Uuid,
        organization_id: Option<Uuid>,
        apd_reference_number: String,
        investigation_notes: Option<String>,
    ) -> Result<Option<SecurityIncident>, String>;

    async fn find_overdue(
        &self,
        organization_id: Option<Uuid>,
    ) -> Result<Vec<SecurityIncident>, String>;
}
