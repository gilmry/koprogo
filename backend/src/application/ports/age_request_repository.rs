use crate::domain::entities::age_request::{AgeRequest, AgeRequestCosignatory};
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait AgeRequestRepository: Send + Sync {
    /// Crée une nouvelle demande d'AGE
    async fn create(&self, age_request: &AgeRequest) -> Result<AgeRequest, String>;

    /// Récupère une demande par son ID (avec cosignataires chargés)
    async fn find_by_id(&self, id: Uuid) -> Result<Option<AgeRequest>, String>;

    /// Liste toutes les demandes d'un bâtiment
    async fn find_by_building(&self, building_id: Uuid) -> Result<Vec<AgeRequest>, String>;

    /// Liste toutes les demandes d'une organisation
    async fn find_by_organization(&self, organization_id: Uuid) -> Result<Vec<AgeRequest>, String>;

    /// Met à jour une demande (status, timestamps, etc.)
    async fn update(&self, age_request: &AgeRequest) -> Result<AgeRequest, String>;

    /// Supprime une demande (Draft/Withdrawn seulement)
    async fn delete(&self, id: Uuid) -> Result<bool, String>;

    /// Ajoute un cosignataire
    async fn add_cosignatory(&self, cosignatory: &AgeRequestCosignatory) -> Result<(), String>;

    /// Retire un cosignataire
    async fn remove_cosignatory(
        &self,
        age_request_id: Uuid,
        owner_id: Uuid,
    ) -> Result<bool, String>;

    /// Récupère les cosignataires d'une demande
    async fn find_cosignatories(
        &self,
        age_request_id: Uuid,
    ) -> Result<Vec<AgeRequestCosignatory>, String>;

    /// Trouve les demandes Submitted dont le délai syndic est dépassé (job de fond)
    async fn find_expired_deadlines(&self) -> Result<Vec<AgeRequest>, String>;
}
