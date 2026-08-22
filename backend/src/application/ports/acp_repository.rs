//! Port (trait) pour le repository ACP — Story 1.1.
//!
//! Hexagonal : ce trait vit côté application, l'implémentation PostgreSQL vit
//! dans `infrastructure/database/repositories/acp_repository_impl.rs`.
//!
//! Toutes les méthodes retournent `Result<_, AppError>` (CRITICAL.md §4 —
//! pas de `Result<_, String>` pour les NEW use-cases).

use crate::application::error::AppError;
use crate::domain::entities::{Acp, AcpMetrics};
use async_trait::async_trait;
use uuid::Uuid;

/// Scope de filtrage pour `list`.
///
/// - `All` : admin sans filtre (voit toutes les ACPs).
/// - `Organization` : syndic / accountant — toutes les ACPs d'un cabinet.
/// - `Owner` : owner / cdc — uniquement les ACPs où l'utilisateur a un
///   `UserRoleAssignment` actif (scope=acp ou via building/unit).
///   Story 1.3 enrichira ce scope (filtre transitif via building/unit) —
///   ici on s'en tient au scope direct ACP.
#[derive(Debug, Clone)]
pub enum ListScope {
    All,
    Organization(Uuid),
    Owner(Uuid),
}

/// Port repository ACP.
#[async_trait]
pub trait AcpRepository: Send + Sync {
    /// Persiste une nouvelle ACP. Retourne l'entité telle que stockée
    /// (incl. `created_at` / `updated_at` côté DB si différents).
    async fn create(&self, acp: &Acp) -> Result<Acp, AppError>;

    /// Récupère par id. `None` si absent (pas une erreur).
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Acp>, AppError>;

    /// Story H6 (CL1) — récupère l'ACP **avec ses métriques agrégées** (Σ units,
    /// Σ lots déclarés, Σ quotités, nb blocs) sur TOUS ses buildings. Source
    /// de vérité de la conformité ACP-level (`Acp::assert_conformant`, ADR-0010).
    /// `None` si l'ACP n'existe pas.
    async fn find_by_id_with_metrics(
        &self,
        id: Uuid,
    ) -> Result<Option<(Acp, AcpMetrics)>, AppError>;

    /// Liste filtrée par scope. Tri implémentation : `created_at DESC`.
    async fn list(&self, scope: ListScope) -> Result<Vec<Acp>, AppError>;

    /// Met à jour une ACP existante (UPDATE … WHERE id = $1).
    /// Retourne `AppError::NotFound` si aucune ligne affectée.
    async fn update(&self, acp: &Acp) -> Result<Acp, AppError>;

    /// "Archive" = DELETE physique pour cette Story 1.1 (pas de soft-delete
    /// en v0.1.0 sur cette table ; la story 5.1 introduira `archived_at` sur
    /// `acp_enabled_modules`, pas ici). Retourne `Ok(())` si suppression OK,
    /// `AppError::NotFound` si aucune ligne affectée.
    async fn archive(&self, id: Uuid) -> Result<(), AppError>;

    /// Compte les buildings rattachés à l'ACP (utilisé par use-cases /
    /// fiche ACP). Renvoie 0 tant que la story 1.2 n'a pas ajouté la
    /// colonne `buildings.acp_id` ; alors la méthode reste compatible
    /// puisque le SQL utilise `COALESCE`.
    async fn count_buildings(&self, id: Uuid) -> Result<i64, AppError>;
}
