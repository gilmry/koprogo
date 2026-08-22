//! Port (trait) pour le repository Portfolio — Story 2.1.
//!
//! Hexagonal : trait côté application, implémentation PostgreSQL dans
//! `infrastructure/database/repositories/portfolio_repository_impl.rs`.
//!
//! Toutes les méthodes retournent `Result<_, AppError>` (CRITICAL.md §4 —
//! pas de `Result<_, String>` pour les NEW use-cases).
//!
//! Source : `docs/maury/refonte-ux-multi-role-acp/architecture.md` §3.1.

use crate::application::error::AppError;
use crate::domain::entities::{Portfolio, PortfolioBuilding, PortfolioShare};
use async_trait::async_trait;
use uuid::Uuid;

/// Entrée de listing : un building du portfolio + flag favori + ordre
/// déterministe (favoris d'abord, puis `added_at DESC`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PortfolioBuildingEntry {
    pub portfolio_id: Uuid,
    pub building_id: Uuid,
    pub is_favorite: bool,
}

/// Port repository Portfolio.
#[async_trait]
pub trait PortfolioRepository: Send + Sync {
    /// Persiste un nouveau portfolio. Retourne l'entité telle que stockée.
    async fn create(&self, portfolio: &Portfolio) -> Result<Portfolio, AppError>;

    /// Récupère par id. `None` si absent (pas une erreur).
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Portfolio>, AppError>;

    /// Liste les portfolios dont `user_id` est `owner_user_id` ou
    /// figure dans `portfolio_shares`. Tri : `created_at DESC`.
    async fn list_for_user(&self, user_id: Uuid) -> Result<Vec<Portfolio>, AppError>;

    /// Met à jour un portfolio existant. Retourne `AppError::NotFound`
    /// si aucune ligne affectée.
    async fn update(&self, portfolio: &Portfolio) -> Result<Portfolio, AppError>;

    /// Supprime un portfolio (DELETE physique — cascade sur
    /// `portfolio_buildings` et `portfolio_shares`).
    async fn delete(&self, id: Uuid) -> Result<(), AppError>;

    /// Ajoute (ou remplace) un building dans le portfolio.
    /// Idempotent : un `ON CONFLICT (portfolio_id, building_id) DO UPDATE`
    /// rafraîchit `is_favorite` si la ligne existe déjà.
    async fn add_building(
        &self,
        portfolio_id: Uuid,
        building_id: Uuid,
        is_favorite: bool,
    ) -> Result<PortfolioBuilding, AppError>;

    /// Retire un building du portfolio. `AppError::NotFound` si aucune
    /// ligne supprimée.
    async fn remove_building(&self, portfolio_id: Uuid, building_id: Uuid) -> Result<(), AppError>;

    /// Liste les buildings d'un portfolio.
    /// **Tri stable** : favoris d'abord (`is_favorite DESC`) puis
    /// `added_at DESC` (cf. AC @happy Story 2.1).
    async fn list_buildings(
        &self,
        portfolio_id: Uuid,
    ) -> Result<Vec<PortfolioBuildingEntry>, AppError>;

    /// Partage le portfolio avec un autre user.
    /// Idempotent : `ON CONFLICT (portfolio_id, shared_with_user_id) DO UPDATE`
    /// rafraîchit `can_edit`.
    async fn share_with(
        &self,
        portfolio_id: Uuid,
        shared_with_user_id: Uuid,
        can_edit: bool,
    ) -> Result<PortfolioShare, AppError>;

    /// Retire un partage. `AppError::NotFound` si aucune ligne supprimée.
    async fn unshare(&self, portfolio_id: Uuid, shared_with_user_id: Uuid) -> Result<(), AppError>;

    /// Liste les partages d'un portfolio.
    async fn list_shares(&self, portfolio_id: Uuid) -> Result<Vec<PortfolioShare>, AppError>;
}
