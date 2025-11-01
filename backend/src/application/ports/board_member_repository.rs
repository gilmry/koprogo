use crate::domain::entities::BoardMember;
use async_trait::async_trait;
use uuid::Uuid;

/// Port (interface) pour le repository des membres du conseil de copropriété
#[async_trait]
pub trait BoardMemberRepository: Send + Sync {
    /// Crée un nouveau membre du conseil
    async fn create(&self, board_member: &BoardMember) -> Result<BoardMember, String>;

    /// Trouve un membre du conseil par son ID
    async fn find_by_id(&self, id: Uuid) -> Result<Option<BoardMember>, String>;

    /// Trouve tous les membres du conseil d'un immeuble
    async fn find_by_building(&self, building_id: Uuid) -> Result<Vec<BoardMember>, String>;

    /// Trouve tous les membres du conseil actifs d'un immeuble
    /// (mandats non expirés à la date actuelle)
    async fn find_active_by_building(&self, building_id: Uuid) -> Result<Vec<BoardMember>, String>;

    /// Trouve les membres du conseil dont le mandat expire bientôt (< 60 jours)
    async fn find_expiring_soon(
        &self,
        building_id: Uuid,
        days_threshold: i32,
    ) -> Result<Vec<BoardMember>, String>;

    /// Trouve tous les mandats d'un copropriétaire (historique complet)
    async fn find_by_owner(&self, owner_id: Uuid) -> Result<Vec<BoardMember>, String>;

    /// Trouve le mandat d'un copropriétaire pour un immeuble spécifique (actif ou non)
    async fn find_by_owner_and_building(
        &self,
        owner_id: Uuid,
        building_id: Uuid,
    ) -> Result<Option<BoardMember>, String>;

    /// Vérifie si un copropriétaire a un mandat actif pour un immeuble donné
    async fn has_active_mandate(&self, owner_id: Uuid, building_id: Uuid) -> Result<bool, String>;

    /// Met à jour un membre du conseil (pour renouvellement de mandat)
    async fn update(&self, board_member: &BoardMember) -> Result<BoardMember, String>;

    /// Supprime un membre du conseil (fin de mandat anticipée)
    async fn delete(&self, id: Uuid) -> Result<bool, String>;

    /// Compte le nombre de membres actifs du conseil pour un immeuble
    async fn count_active_by_building(&self, building_id: Uuid) -> Result<i64, String>;
}
