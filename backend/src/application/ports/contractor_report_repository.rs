use crate::domain::entities::contractor_report::ContractorReport;
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait ContractorReportRepository: Send + Sync {
    /// Crée un nouveau rapport de travaux
    async fn create(&self, report: &ContractorReport) -> Result<ContractorReport, String>;

    /// Récupère un rapport par son ID
    async fn find_by_id(&self, id: Uuid) -> Result<Option<ContractorReport>, String>;

    /// Récupère un rapport via son magic token (accès PWA sans auth)
    async fn find_by_magic_token(
        &self,
        token_hash: &str,
    ) -> Result<Option<ContractorReport>, String>;

    /// Liste les rapports d'un ticket donné
    async fn find_by_ticket(&self, ticket_id: Uuid) -> Result<Vec<ContractorReport>, String>;

    /// Liste les rapports d'un quote/devis donné
    async fn find_by_quote(&self, quote_id: Uuid) -> Result<Vec<ContractorReport>, String>;

    /// Liste tous les rapports d'un bâtiment
    async fn find_by_building(&self, building_id: Uuid) -> Result<Vec<ContractorReport>, String>;

    /// Liste tous les rapports d'une organisation
    async fn find_by_organization(
        &self,
        organization_id: Uuid,
    ) -> Result<Vec<ContractorReport>, String>;

    /// Met à jour le rapport (status, photos, compte-rendu, etc.)
    async fn update(&self, report: &ContractorReport) -> Result<ContractorReport, String>;

    /// Supprime un rapport (Draft seulement)
    async fn delete(&self, id: Uuid) -> Result<bool, String>;
}
