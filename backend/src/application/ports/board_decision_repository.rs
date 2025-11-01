use crate::domain::entities::{BoardDecision, DecisionStatus};
use async_trait::async_trait;
use uuid::Uuid;

/// Port (interface) pour le repository des décisions du conseil
#[async_trait]
pub trait BoardDecisionRepository: Send + Sync {
    /// Crée une nouvelle décision à suivre
    async fn create(&self, decision: &BoardDecision) -> Result<BoardDecision, String>;

    /// Trouve une décision par son ID
    async fn find_by_id(&self, id: Uuid) -> Result<Option<BoardDecision>, String>;

    /// Trouve toutes les décisions d'un immeuble
    async fn find_by_building(&self, building_id: Uuid) -> Result<Vec<BoardDecision>, String>;

    /// Trouve toutes les décisions d'une assemblée générale
    async fn find_by_meeting(&self, meeting_id: Uuid) -> Result<Vec<BoardDecision>, String>;

    /// Trouve les décisions par statut pour un immeuble
    async fn find_by_status(
        &self,
        building_id: Uuid,
        status: DecisionStatus,
    ) -> Result<Vec<BoardDecision>, String>;

    /// Trouve les décisions en retard (deadline dépassée et statut != completed/cancelled)
    async fn find_overdue(&self, building_id: Uuid) -> Result<Vec<BoardDecision>, String>;

    /// Trouve les décisions avec deadline proche (< N jours)
    async fn find_deadline_approaching(
        &self,
        building_id: Uuid,
        days_threshold: i32,
    ) -> Result<Vec<BoardDecision>, String>;

    /// Met à jour une décision (changement de statut, ajout de notes)
    async fn update(&self, decision: &BoardDecision) -> Result<BoardDecision, String>;

    /// Supprime une décision
    async fn delete(&self, id: Uuid) -> Result<bool, String>;

    /// Compte les décisions par statut pour un immeuble
    async fn count_by_status(
        &self,
        building_id: Uuid,
        status: DecisionStatus,
    ) -> Result<i64, String>;

    /// Compte le total de décisions en retard pour un immeuble
    async fn count_overdue(&self, building_id: Uuid) -> Result<i64, String>;
}
