//! Portfolio use-cases — Story 2.1.
//!
//! Use-cases :
//! - `create_portfolio`         : tout authentifié peut créer son portfolio
//! - `get_portfolio`            : owner OU shared user
//! - `list_portfolios`          : owner + shared
//! - `update_portfolio`         : owner only (ou shared can_edit)
//! - `delete_portfolio`         : owner only
//! - `add_building`             : owner OU shared can_edit
//! - `remove_building`          : owner OU shared can_edit
//! - `list_buildings`           : owner OU shared (lecture seule)
//! - `share_with`               : owner only
//! - `unshare`                  : owner only
//! - `list_shares`              : owner only
//!
//! Tous les retours sont typés `Result<T, AppError>` (CRITICAL §4).
//! Le scope de permission est centralisé dans `assert_can_read` /
//! `assert_can_write`.
//!
//! ADR refs : ADR-0011 (Portefeuille entité backend).

use crate::application::dto::{
    AddBuildingDto, CreatePortfolioDto, PortfolioBuildingResponseDto, PortfolioResponseDto,
    PortfolioShareResponseDto, SharePortfolioDto, UpdatePortfolioDto,
};
use crate::application::error::AppError;
use crate::application::ports::{BuildingRepository, PortfolioRepository, UserRepository};
use crate::domain::entities::{Portfolio, PortfolioShare};
use std::sync::Arc;
use uuid::Uuid;

/// Caller pour les use-cases portfolio.
///
/// Le user est toujours identifié par `user_id` — la table est portée par
/// `users` (cf. migration `20260601050000_create_portfolios.sql`). On ne
/// distingue pas les rôles ici : tout authentifié peut créer son portfolio.
/// Les permissions fines (read/write/share) se règlent par owner/shared via
/// `assert_can_read` / `assert_can_write`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PortfolioCaller {
    pub user_id: Uuid,
}

pub struct PortfolioUseCases {
    repository: Arc<dyn PortfolioRepository>,
    building_repository: Arc<dyn BuildingRepository>,
    user_repository: Arc<dyn UserRepository>,
}

impl PortfolioUseCases {
    pub fn new(
        repository: Arc<dyn PortfolioRepository>,
        building_repository: Arc<dyn BuildingRepository>,
        user_repository: Arc<dyn UserRepository>,
    ) -> Self {
        Self {
            repository,
            building_repository,
            user_repository,
        }
    }

    /// Crée un portfolio pour le caller.
    pub async fn create_portfolio(
        &self,
        caller: &PortfolioCaller,
        dto: CreatePortfolioDto,
    ) -> Result<PortfolioResponseDto, AppError> {
        let portfolio = Portfolio::new(caller.user_id, dto.name, dto.description)?;
        let created = self.repository.create(&portfolio).await?;
        Ok(Self::to_response(&created))
    }

    /// Récupère un portfolio par id (owner OU shared).
    pub async fn get_portfolio(
        &self,
        caller: &PortfolioCaller,
        id: Uuid,
    ) -> Result<PortfolioResponseDto, AppError> {
        let portfolio = self
            .repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Portfolio {} not found", id)))?;
        self.assert_can_read(caller, &portfolio).await?;
        Ok(Self::to_response(&portfolio))
    }

    /// Liste les portfolios visibles pour le caller.
    pub async fn list_portfolios(
        &self,
        caller: &PortfolioCaller,
    ) -> Result<Vec<PortfolioResponseDto>, AppError> {
        let list = self.repository.list_for_user(caller.user_id).await?;
        Ok(list.iter().map(Self::to_response).collect())
    }

    /// Met à jour un portfolio (owner OU shared can_edit).
    pub async fn update_portfolio(
        &self,
        caller: &PortfolioCaller,
        id: Uuid,
        dto: UpdatePortfolioDto,
    ) -> Result<PortfolioResponseDto, AppError> {
        let mut portfolio = self
            .repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Portfolio {} not found", id)))?;
        self.assert_can_write(caller, &portfolio).await?;
        portfolio.update_info(dto.name, dto.description)?;
        let updated = self.repository.update(&portfolio).await?;
        Ok(Self::to_response(&updated))
    }

    /// Supprime un portfolio (owner only).
    pub async fn delete_portfolio(
        &self,
        caller: &PortfolioCaller,
        id: Uuid,
    ) -> Result<(), AppError> {
        let portfolio = self
            .repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Portfolio {} not found", id)))?;
        self.assert_is_owner(caller, &portfolio)?;
        self.repository.delete(id).await
    }

    /// Ajoute un building au portfolio (owner OU shared can_edit).
    pub async fn add_building(
        &self,
        caller: &PortfolioCaller,
        portfolio_id: Uuid,
        dto: AddBuildingDto,
    ) -> Result<PortfolioBuildingResponseDto, AppError> {
        let portfolio = self
            .repository
            .find_by_id(portfolio_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Portfolio {} not found", portfolio_id)))?;
        self.assert_can_write(caller, &portfolio).await?;

        let building_id = Uuid::parse_str(&dto.building_id)
            .map_err(|_| AppError::Validation("Invalid building_id format".to_string()))?;

        // Vérification d'existence — AC @negative : 404 typé sur building
        // inexistant (sinon FK violation côté DB serait remappée en
        // Database 500).
        let exists = self
            .building_repository
            .find_by_id(building_id)
            .await
            .map_err(AppError::from)?
            .is_some();
        if !exists {
            return Err(AppError::NotFound(format!(
                "Building {} not found",
                building_id
            )));
        }

        let entry = self
            .repository
            .add_building(portfolio_id, building_id, dto.is_favorite)
            .await?;
        Ok(PortfolioBuildingResponseDto {
            portfolio_id: entry.portfolio_id.to_string(),
            building_id: entry.building_id.to_string(),
            is_favorite: entry.is_favorite,
        })
    }

    /// Retire un building du portfolio (owner OU shared can_edit).
    pub async fn remove_building(
        &self,
        caller: &PortfolioCaller,
        portfolio_id: Uuid,
        building_id: Uuid,
    ) -> Result<(), AppError> {
        let portfolio = self
            .repository
            .find_by_id(portfolio_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Portfolio {} not found", portfolio_id)))?;
        self.assert_can_write(caller, &portfolio).await?;
        self.repository
            .remove_building(portfolio_id, building_id)
            .await
    }

    /// Liste les buildings d'un portfolio (owner OU shared).
    ///
    /// **Tri** : favoris d'abord puis `added_at DESC` (cf. AC @happy Story 2.1).
    pub async fn list_buildings(
        &self,
        caller: &PortfolioCaller,
        portfolio_id: Uuid,
    ) -> Result<Vec<PortfolioBuildingResponseDto>, AppError> {
        let portfolio = self
            .repository
            .find_by_id(portfolio_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Portfolio {} not found", portfolio_id)))?;
        self.assert_can_read(caller, &portfolio).await?;
        let entries = self.repository.list_buildings(portfolio_id).await?;
        Ok(entries
            .into_iter()
            .map(|e| PortfolioBuildingResponseDto {
                portfolio_id: e.portfolio_id.to_string(),
                building_id: e.building_id.to_string(),
                is_favorite: e.is_favorite,
            })
            .collect())
    }

    /// Partage le portfolio avec un autre user (owner only).
    pub async fn share_with(
        &self,
        caller: &PortfolioCaller,
        portfolio_id: Uuid,
        dto: SharePortfolioDto,
    ) -> Result<PortfolioShareResponseDto, AppError> {
        let portfolio = self
            .repository
            .find_by_id(portfolio_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Portfolio {} not found", portfolio_id)))?;
        self.assert_is_owner(caller, &portfolio)?;

        let user_id = Uuid::parse_str(&dto.shared_with_user_id)
            .map_err(|_| AppError::Validation("Invalid shared_with_user_id format".to_string()))?;
        // Vérification d'existence (404 si user inconnu plutôt que FK 500).
        let exists = self
            .user_repository
            .find_by_id(user_id)
            .await
            .map_err(AppError::from)?
            .is_some();
        if !exists {
            return Err(AppError::NotFound(format!("User {} not found", user_id)));
        }

        let share = self
            .repository
            .share_with(portfolio_id, user_id, dto.can_edit)
            .await?;
        Ok(Self::share_to_response(&share))
    }

    /// Retire un partage (owner only).
    pub async fn unshare(
        &self,
        caller: &PortfolioCaller,
        portfolio_id: Uuid,
        shared_with_user_id: Uuid,
    ) -> Result<(), AppError> {
        let portfolio = self
            .repository
            .find_by_id(portfolio_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Portfolio {} not found", portfolio_id)))?;
        self.assert_is_owner(caller, &portfolio)?;
        self.repository
            .unshare(portfolio_id, shared_with_user_id)
            .await
    }

    /// Liste les partages d'un portfolio (owner only).
    pub async fn list_shares(
        &self,
        caller: &PortfolioCaller,
        portfolio_id: Uuid,
    ) -> Result<Vec<PortfolioShareResponseDto>, AppError> {
        let portfolio = self
            .repository
            .find_by_id(portfolio_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Portfolio {} not found", portfolio_id)))?;
        self.assert_is_owner(caller, &portfolio)?;
        let shares = self.repository.list_shares(portfolio_id).await?;
        Ok(shares.iter().map(Self::share_to_response).collect())
    }

    // -----------------------------------------------------------------------
    // Permissions
    // -----------------------------------------------------------------------

    /// Le caller est-il owner du portfolio ?
    fn assert_is_owner(
        &self,
        caller: &PortfolioCaller,
        portfolio: &Portfolio,
    ) -> Result<(), AppError> {
        if portfolio.owner_user_id == caller.user_id {
            Ok(())
        } else {
            Err(AppError::Forbidden(format!(
                "Portfolio {} not owned by user",
                portfolio.id
            )))
        }
    }

    /// Le caller peut-il lire le portfolio (owner OU shared) ?
    async fn assert_can_read(
        &self,
        caller: &PortfolioCaller,
        portfolio: &Portfolio,
    ) -> Result<(), AppError> {
        if portfolio.owner_user_id == caller.user_id {
            return Ok(());
        }
        // Lecture autorisée si présent dans `portfolio_shares`.
        let shares = self.repository.list_shares(portfolio.id).await?;
        if shares
            .iter()
            .any(|s| s.shared_with_user_id == caller.user_id)
        {
            Ok(())
        } else {
            // 403 typé — pas 404 pour ne pas révéler "il existe mais pas pour
            // toi" différemment de "n'existe pas". On choisit 403 ici car
            // l'appelant a explicitement demandé un id qu'il "ne devrait pas
            // connaître" — la fuite d'existence n'est pas critique
            // (UUIDs v4 non énumérables).
            Err(AppError::Forbidden(format!(
                "Portfolio {} not accessible",
                portfolio.id
            )))
        }
    }

    /// Le caller peut-il écrire (owner OU shared can_edit) ?
    async fn assert_can_write(
        &self,
        caller: &PortfolioCaller,
        portfolio: &Portfolio,
    ) -> Result<(), AppError> {
        if portfolio.owner_user_id == caller.user_id {
            return Ok(());
        }
        let shares = self.repository.list_shares(portfolio.id).await?;
        if shares
            .iter()
            .any(|s| s.shared_with_user_id == caller.user_id && s.can_edit)
        {
            Ok(())
        } else {
            Err(AppError::Forbidden(format!(
                "Portfolio {} not writable by user",
                portfolio.id
            )))
        }
    }

    fn to_response(p: &Portfolio) -> PortfolioResponseDto {
        PortfolioResponseDto {
            id: p.id.to_string(),
            owner_user_id: p.owner_user_id.to_string(),
            name: p.name.clone(),
            description: p.description.clone(),
            created_at: p.created_at.to_rfc3339(),
            updated_at: p.updated_at.to_rfc3339(),
        }
    }

    fn share_to_response(s: &PortfolioShare) -> PortfolioShareResponseDto {
        PortfolioShareResponseDto {
            portfolio_id: s.portfolio_id.to_string(),
            shared_with_user_id: s.shared_with_user_id.to_string(),
            can_edit: s.can_edit,
            shared_at: s.shared_at.to_rfc3339(),
        }
    }
}

// ============================================================================
// Tests — taxonomie 4-cat avec mocks (CRITICAL.md §3).
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::ports::{
        BuildingRepository, PortfolioBuildingEntry, PortfolioRepository, UserRepository,
    };
    use crate::domain::entities::{Building, Portfolio, PortfolioBuilding, PortfolioShare, User};
    use async_trait::async_trait;
    use mockall::mock;

    mock! {
        PortfolioRepo {}

        #[async_trait]
        impl PortfolioRepository for PortfolioRepo {
            async fn create(&self, portfolio: &Portfolio) -> Result<Portfolio, AppError>;
            async fn find_by_id(&self, id: Uuid) -> Result<Option<Portfolio>, AppError>;
            async fn list_for_user(&self, user_id: Uuid) -> Result<Vec<Portfolio>, AppError>;
            async fn update(&self, portfolio: &Portfolio) -> Result<Portfolio, AppError>;
            async fn delete(&self, id: Uuid) -> Result<(), AppError>;
            async fn add_building(
                &self,
                portfolio_id: Uuid,
                building_id: Uuid,
                is_favorite: bool,
            ) -> Result<PortfolioBuilding, AppError>;
            async fn remove_building(&self, portfolio_id: Uuid, building_id: Uuid) -> Result<(), AppError>;
            async fn list_buildings(&self, portfolio_id: Uuid) -> Result<Vec<PortfolioBuildingEntry>, AppError>;
            async fn share_with(
                &self,
                portfolio_id: Uuid,
                shared_with_user_id: Uuid,
                can_edit: bool,
            ) -> Result<PortfolioShare, AppError>;
            async fn unshare(&self, portfolio_id: Uuid, shared_with_user_id: Uuid) -> Result<(), AppError>;
            async fn list_shares(&self, portfolio_id: Uuid) -> Result<Vec<PortfolioShare>, AppError>;
        }
    }

    mock! {
        BuildingRepo {}

        #[async_trait]
        impl BuildingRepository for BuildingRepo {
            async fn create(&self, building: &Building) -> Result<Building, String>;
            async fn find_by_id(&self, id: Uuid) -> Result<Option<Building>, String>;
            async fn find_all(&self) -> Result<Vec<Building>, String>;
            async fn find_all_paginated(
                &self,
                page_request: &crate::application::dto::PageRequest,
                filters: &crate::application::dto::BuildingFilters,
            ) -> Result<(Vec<Building>, i64), String>;
            async fn update(&self, building: &Building) -> Result<Building, String>;
            async fn delete(&self, id: Uuid) -> Result<bool, String>;
            async fn find_by_slug(&self, slug: &str) -> Result<Option<Building>, String>;
            async fn find_by_id_with_metrics(
                &self,
                id: Uuid,
            ) -> Result<Option<(Building, crate::domain::entities::BuildingMetrics)>, String>;
        }
    }

    mock! {
        UserRepo {}

        #[async_trait]
        impl UserRepository for UserRepo {
            async fn create(&self, user: &User) -> Result<User, String>;
            async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, String>;
            async fn find_by_email(&self, email: &str) -> Result<Option<User>, String>;
            async fn find_all(&self) -> Result<Vec<User>, String>;
            async fn find_by_organization(&self, org_id: Uuid) -> Result<Vec<User>, String>;
            async fn update(&self, user: &User) -> Result<User, String>;
            async fn update_password(&self, id: Uuid, password_hash: &str) -> Result<bool, String>;
            async fn activate(&self, id: Uuid) -> Result<Option<User>, String>;
            async fn deactivate(&self, id: Uuid) -> Result<Option<User>, String>;
            async fn delete(&self, id: Uuid) -> Result<bool, String>;
            async fn count_by_organization(&self, org_id: Uuid) -> Result<i64, String>;
        }
    }

    fn make_portfolio(owner: Uuid, name: &str) -> Portfolio {
        Portfolio::new(owner, name.to_string(), None).unwrap()
    }

    fn make_use_cases(
        pr: MockPortfolioRepo,
        br: MockBuildingRepo,
        ur: MockUserRepo,
    ) -> PortfolioUseCases {
        PortfolioUseCases::new(Arc::new(pr), Arc::new(br), Arc::new(ur))
    }

    // ----- @happy ------------------------------------------------------------

    #[tokio::test]
    async fn happy_user_creates_portfolio() {
        let user_id = Uuid::new_v4();
        let mut pr = MockPortfolioRepo::new();
        pr.expect_create().returning(|p| Ok(p.clone()));
        let uc = make_use_cases(pr, MockBuildingRepo::new(), MockUserRepo::new());

        let dto = CreatePortfolioDto {
            name: "Mes immeubles favoris".to_string(),
            description: None,
        };
        let resp = uc
            .create_portfolio(&PortfolioCaller { user_id }, dto)
            .await
            .expect("ok");
        assert_eq!(resp.name, "Mes immeubles favoris");
        assert_eq!(resp.owner_user_id, user_id.to_string());
    }

    // ----- @edge -------------------------------------------------------------

    #[tokio::test]
    async fn edge_empty_buildings_listing_returns_empty_vec() {
        let user_id = Uuid::new_v4();
        let portfolio = make_portfolio(user_id, "Vide");
        let portfolio_id = portfolio.id;

        let mut pr = MockPortfolioRepo::new();
        pr.expect_find_by_id()
            .returning(move |_| Ok(Some(portfolio.clone())));
        pr.expect_list_buildings().returning(|_| Ok(Vec::new()));
        let uc = make_use_cases(pr, MockBuildingRepo::new(), MockUserRepo::new());

        let buildings = uc
            .list_buildings(&PortfolioCaller { user_id }, portfolio_id)
            .await
            .expect("ok");
        assert!(buildings.is_empty());
    }

    // ----- @security ---------------------------------------------------------

    #[tokio::test]
    async fn security_non_owner_non_shared_cannot_read_portfolio() {
        let owner = Uuid::new_v4();
        let other = Uuid::new_v4();
        let portfolio = make_portfolio(owner, "Owner Portfolio");
        let portfolio_id = portfolio.id;

        let mut pr = MockPortfolioRepo::new();
        pr.expect_find_by_id()
            .returning(move |_| Ok(Some(portfolio.clone())));
        pr.expect_list_shares().returning(|_| Ok(Vec::new()));
        let uc = make_use_cases(pr, MockBuildingRepo::new(), MockUserRepo::new());

        let err = uc
            .get_portfolio(&PortfolioCaller { user_id: other }, portfolio_id)
            .await
            .unwrap_err();
        match err {
            AppError::Forbidden(_) => {}
            other => panic!("expected Forbidden, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn security_shared_user_can_read_but_not_share() {
        let owner = Uuid::new_v4();
        let shared = Uuid::new_v4();
        let portfolio = make_portfolio(owner, "Shared Portfolio");
        let portfolio_id = portfolio.id;

        let mut pr = MockPortfolioRepo::new();
        let pclone = portfolio.clone();
        pr.expect_find_by_id()
            .returning(move |_| Ok(Some(pclone.clone())));
        pr.expect_list_shares().returning(move |_| {
            Ok(vec![PortfolioShare {
                portfolio_id,
                shared_with_user_id: shared,
                can_edit: false,
                shared_at: chrono::Utc::now(),
            }])
        });
        let uc = make_use_cases(pr, MockBuildingRepo::new(), MockUserRepo::new());

        // Shared user CAN read.
        let resp = uc
            .get_portfolio(&PortfolioCaller { user_id: shared }, portfolio_id)
            .await
            .expect("read ok");
        assert_eq!(resp.id, portfolio_id.to_string());

        // Shared user CANNOT manage shares (owner only).
        let err = uc
            .share_with(
                &PortfolioCaller { user_id: shared },
                portfolio_id,
                SharePortfolioDto {
                    shared_with_user_id: Uuid::new_v4().to_string(),
                    can_edit: false,
                },
            )
            .await
            .unwrap_err();
        match err {
            AppError::Forbidden(_) => {}
            other => panic!("expected Forbidden, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn security_shared_read_only_cannot_add_building() {
        let owner = Uuid::new_v4();
        let shared = Uuid::new_v4();
        let portfolio = make_portfolio(owner, "Shared RO");
        let portfolio_id = portfolio.id;

        let mut pr = MockPortfolioRepo::new();
        pr.expect_find_by_id()
            .returning(move |_| Ok(Some(portfolio.clone())));
        pr.expect_list_shares().returning(move |_| {
            Ok(vec![PortfolioShare {
                portfolio_id,
                shared_with_user_id: shared,
                can_edit: false,
                shared_at: chrono::Utc::now(),
            }])
        });
        let uc = make_use_cases(pr, MockBuildingRepo::new(), MockUserRepo::new());

        let err = uc
            .add_building(
                &PortfolioCaller { user_id: shared },
                portfolio_id,
                AddBuildingDto {
                    building_id: Uuid::new_v4().to_string(),
                    is_favorite: false,
                },
            )
            .await
            .unwrap_err();
        match err {
            AppError::Forbidden(_) => {}
            other => panic!("expected Forbidden, got {:?}", other),
        }
    }

    // ----- @negative ---------------------------------------------------------

    #[tokio::test]
    async fn negative_create_with_empty_name_returns_validation() {
        let user_id = Uuid::new_v4();
        let pr = MockPortfolioRepo::new();
        let uc = make_use_cases(pr, MockBuildingRepo::new(), MockUserRepo::new());
        let dto = CreatePortfolioDto {
            name: "".to_string(),
            description: None,
        };
        let err = uc
            .create_portfolio(&PortfolioCaller { user_id }, dto)
            .await
            .unwrap_err();
        match err {
            AppError::Validation(_) => {}
            other => panic!("expected Validation, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn negative_get_unknown_returns_not_found() {
        let user_id = Uuid::new_v4();
        let mut pr = MockPortfolioRepo::new();
        pr.expect_find_by_id().returning(|_| Ok(None));
        let uc = make_use_cases(pr, MockBuildingRepo::new(), MockUserRepo::new());
        let err = uc
            .get_portfolio(&PortfolioCaller { user_id }, Uuid::new_v4())
            .await
            .unwrap_err();
        match err {
            AppError::NotFound(_) => {}
            other => panic!("expected NotFound, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn negative_add_unknown_building_returns_not_found() {
        let owner = Uuid::new_v4();
        let portfolio = make_portfolio(owner, "Portfolio");
        let portfolio_id = portfolio.id;

        let mut pr = MockPortfolioRepo::new();
        pr.expect_find_by_id()
            .returning(move |_| Ok(Some(portfolio.clone())));
        let mut br = MockBuildingRepo::new();
        br.expect_find_by_id().returning(|_| Ok(None));
        let uc = make_use_cases(pr, br, MockUserRepo::new());

        let err = uc
            .add_building(
                &PortfolioCaller { user_id: owner },
                portfolio_id,
                AddBuildingDto {
                    building_id: Uuid::new_v4().to_string(),
                    is_favorite: false,
                },
            )
            .await
            .unwrap_err();
        match err {
            AppError::NotFound(_) => {}
            other => panic!("expected NotFound, got {:?}", other),
        }
    }
}
