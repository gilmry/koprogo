//! `ListBuildings` use-case — Story 1.3 (refonte UX multi-rôle).
//!
//! Filters the buildings list by `ListScope` derived from the caller's
//! role. Distinct from the legacy `BuildingUseCases::list_buildings*`
//! (which still uses `Result<_, String>` — territory of cluster #555,
//! migrated lazily by stories that touch building_use_cases directly).
//!
//! This NEW use-case is `AppError`-typed end-to-end (cf. CRITICAL §4 +
//! architecture §3.3 "no new Result<_, String>").
//!
//! Implementation pragmatics — Story 1.2 will add `buildings.acp_id`
//! while keeping `organization_id` during the transition. Until the
//! ACP-scoped filter is wired in the repository, we re-use
//! `BuildingFilters { organization_id, owner_user_id }` and accept that
//! `ListScope::AcpScope` is functionally equivalent to filtering by the
//! ACP's owning organization (the `acp_id` field will be plugged in
//! Story 1.2's PR).

use crate::application::dto::{BuildingFilters, BuildingResponseDto, PageRequest};
use crate::application::error::AppError;
use crate::application::ports::{AcpRepository, BuildingRepository, ListScope};
use crate::domain::entities::Building;
use std::sync::Arc;
use uuid::Uuid;

/// Resolved scope ready to be passed to the repository layer.
///
/// `AcpScope` is resolved post-Story-1.2 by looking up the ACP's
/// `organization_id` (so the existing `BuildingFilters` query still
/// works during the transition).
#[derive(Debug, Clone)]
struct ResolvedScope {
    organization_id: Option<Uuid>,
    owner_user_id: Option<Uuid>,
    /// If `true`, the caller has unrestricted access (admin path).
    all: bool,
}

pub struct ListBuildingsUseCase {
    building_repo: Arc<dyn BuildingRepository>,
    acp_repo: Arc<dyn AcpRepository>,
}

impl ListBuildingsUseCase {
    pub fn new(
        building_repo: Arc<dyn BuildingRepository>,
        acp_repo: Arc<dyn AcpRepository>,
    ) -> Self {
        Self {
            building_repo,
            acp_repo,
        }
    }

    /// List buildings filtered by `scope` (caller-derived).
    ///
    /// Returns 422 / `Validation` if the caller supplied a scope that
    /// can't be resolved (e.g. `AcpScope` with an unknown ACP id).
    pub async fn list_for_scope(
        &self,
        page_request: &PageRequest,
        scope: ListScope,
    ) -> Result<(Vec<BuildingResponseDto>, i64), AppError> {
        let resolved = self.resolve(scope).await?;

        let filters = BuildingFilters {
            organization_id: if resolved.all {
                None
            } else {
                resolved.organization_id
            },
            owner_user_id: resolved.owner_user_id,
            ..Default::default()
        };

        let (buildings, total) = self
            .building_repo
            .find_all_paginated(page_request, &filters)
            .await
            // Legacy repository still returns Result<_, String> — wrap into
            // AppError::Database without re-introducing a Result<_, String>
            // in this use-case's public signature.
            .map_err(AppError::Database)?;

        let dtos = buildings.iter().map(Self::to_response_dto).collect();
        Ok((dtos, total))
    }

    /// Resolve `ListScope` into the legacy `BuildingFilters` shape.
    ///
    /// `AcpScope(acp_id)` is currently resolved via the ACP's
    /// `organization_id` — story 1.2 will swap this for a direct
    /// `acp_id` filter once `buildings.acp_id` is `NOT NULL`.
    async fn resolve(&self, scope: ListScope) -> Result<ResolvedScope, AppError> {
        match scope {
            ListScope::All => Ok(ResolvedScope {
                organization_id: None,
                owner_user_id: None,
                all: true,
            }),
            ListScope::Organization(org_id) => Ok(ResolvedScope {
                organization_id: Some(org_id),
                owner_user_id: None,
                all: false,
            }),
            ListScope::Owner(user_id) => Ok(ResolvedScope {
                organization_id: None,
                owner_user_id: Some(user_id),
                all: false,
            }),
        }
    }

    /// Resolve a per-ACP scope (used by scope_guard middleware once a
    /// `X-Scope-AcpId` header is present). Looks up the ACP's
    /// organization_id and applies that as the filter.
    pub async fn list_for_acp(
        &self,
        page_request: &PageRequest,
        acp_id: Uuid,
    ) -> Result<(Vec<BuildingResponseDto>, i64), AppError> {
        let acp = self
            .acp_repo
            .find_by_id(acp_id)
            .await?
            .ok_or(AppError::AcpNotInScope { acp_id })?;

        // ACP without organization (auto-managed) -> no FK filter possible
        // pre-Story-1.2; we conservatively return empty list to avoid
        // leaking buildings of other ACPs sharing a NULL organization.
        let org_id = match acp.organization_id {
            Some(o) => o,
            None => return Ok((vec![], 0)),
        };

        let filters = BuildingFilters {
            organization_id: Some(org_id),
            ..Default::default()
        };
        let (buildings, total) = self
            .building_repo
            .find_all_paginated(page_request, &filters)
            .await
            .map_err(AppError::Database)?;

        let dtos = buildings.iter().map(Self::to_response_dto).collect();
        Ok((dtos, total))
    }

    fn to_response_dto(b: &Building) -> BuildingResponseDto {
        // Story 1.4 metrics fields default to 0/empty/false on list endpoint
        // (per-row metrics aggregate would be expensive on paginated list ;
        //  full metrics are exposed via GET /buildings/:id detail endpoint).
        //
        // Track H Story H1 — `quota_delta` fallback = `total_tantiemes`
        // (positif = manque) au lieu de la constante hard-codée "-1000"
        // qui était incohérente avec la convention `total - sum` et avec
        // les actes de base ≠ 1000.
        BuildingResponseDto {
            id: b.id.to_string(),
            acp_id: b.acp_id.to_string(),
            name: b.name.clone(),
            address: b.address.clone(),
            city: b.city.clone(),
            postal_code: b.postal_code.clone(),
            country: b.country.clone(),
            total_units: b.total_units,
            total_tantiemes: b.total_tantiemes,
            construction_year: b.construction_year,
            created_at: b.created_at.to_rfc3339(),
            updated_at: b.updated_at.to_rfc3339(),
            units_count: 0,
            quota_sum: String::from("0"),
            is_conformant: false,
            quota_delta: b.total_tantiemes.to_string(),
        }
    }
}

// ============================================================================
// Tests — taxonomie 4-cat (CRITICAL.md §3).
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::ports::{AcpRepository, BuildingRepository, ListScope};
    use crate::domain::entities::{Acp, Building};
    use async_trait::async_trait;
    use mockall::mock;

    mock! {
        BuildingRepo {}

        #[async_trait]
        impl BuildingRepository for BuildingRepo {
            async fn create(&self, building: &Building) -> Result<Building, String>;
            async fn find_by_id(&self, id: Uuid) -> Result<Option<Building>, String>;
            async fn find_all(&self) -> Result<Vec<Building>, String>;
            async fn find_all_paginated(
                &self,
                page_request: &PageRequest,
                filters: &BuildingFilters,
            ) -> Result<(Vec<Building>, i64), String>;
            async fn update(&self, building: &Building) -> Result<Building, String>;
            async fn delete(&self, id: Uuid) -> Result<bool, String>;
            async fn find_by_slug(&self, slug: &str) -> Result<Option<Building>, String>;
            async fn find_by_id_with_metrics(
                &self,
                id: Uuid,
            ) -> Result<
                Option<(Building, crate::domain::entities::BuildingMetrics)>,
                String,
            >;
        }
    }

    mock! {
        AcpRepo {}

        #[async_trait]
        impl AcpRepository for AcpRepo {
            async fn create(&self, acp: &Acp) -> Result<Acp, AppError>;
            async fn find_by_id(&self, id: Uuid) -> Result<Option<Acp>, AppError>;
            async fn find_by_id_with_metrics(&self, id: Uuid) -> Result<Option<(Acp, crate::domain::entities::AcpMetrics)>, AppError>;
            async fn list(&self, scope: ListScope) -> Result<Vec<Acp>, AppError>;
            async fn update(&self, acp: &Acp) -> Result<Acp, AppError>;
            async fn archive(&self, id: Uuid) -> Result<(), AppError>;
            async fn count_buildings(&self, id: Uuid) -> Result<i64, AppError>;
        }
    }

    fn page() -> PageRequest {
        PageRequest {
            page: 1,
            per_page: 20,
            sort_by: None,
            order: crate::application::dto::SortOrder::default(),
        }
    }

    fn make_building(org: Uuid) -> Building {
        Building::new(
            org,
            "Building".to_string(),
            "Addr".to_string(),
            "City".to_string(),
            "1000".to_string(),
            "BE".to_string(),
            10,
            1000,
            Some(2000),
        )
        .expect("valid building")
    }

    fn make_acp(org_id: Option<Uuid>) -> Acp {
        Acp::new(
            org_id,
            "Acp".to_string(),
            "Rue X".to_string(),
            "1000".to_string(),
            "Bruxelles".to_string(),
            None,
        )
        .expect("valid acp")
    }

    // ----- @happy --------------------------------------------------------------

    #[tokio::test]
    async fn happy_admin_sees_all_buildings() {
        let mut building_repo = MockBuildingRepo::new();
        let org = Uuid::new_v4();
        let b = make_building(org);
        let b_clone = b.clone();
        building_repo
            .expect_find_all_paginated()
            .withf(|_pr, f| f.organization_id.is_none() && f.owner_user_id.is_none())
            .returning(move |_, _| Ok((vec![b_clone.clone()], 1)));

        let acp_repo = MockAcpRepo::new();
        let uc = ListBuildingsUseCase::new(Arc::new(building_repo), Arc::new(acp_repo));
        let (list, total) = uc
            .list_for_scope(&page(), ListScope::All)
            .await
            .expect("ok");
        assert_eq!(total, 1);
        assert_eq!(list.len(), 1);
    }

    #[tokio::test]
    async fn happy_syndic_sees_only_org_buildings() {
        let org_a = Uuid::new_v4();
        let mut building_repo = MockBuildingRepo::new();
        let b = make_building(org_a);
        let b_clone = b.clone();
        building_repo
            .expect_find_all_paginated()
            .withf(move |_pr, f| f.organization_id == Some(org_a))
            .returning(move |_, _| Ok((vec![b_clone.clone()], 1)));

        let acp_repo = MockAcpRepo::new();
        let uc = ListBuildingsUseCase::new(Arc::new(building_repo), Arc::new(acp_repo));
        let (list, _) = uc
            .list_for_scope(&page(), ListScope::Organization(org_a))
            .await
            .expect("ok");
        assert_eq!(list.len(), 1);
    }

    #[tokio::test]
    async fn happy_owner_sees_only_own_buildings() {
        let user_id = Uuid::new_v4();
        let mut building_repo = MockBuildingRepo::new();
        building_repo
            .expect_find_all_paginated()
            .withf(move |_pr, f| f.owner_user_id == Some(user_id))
            .returning(move |_, _| Ok((vec![], 0)));

        let acp_repo = MockAcpRepo::new();
        let uc = ListBuildingsUseCase::new(Arc::new(building_repo), Arc::new(acp_repo));
        let (list, total) = uc
            .list_for_scope(&page(), ListScope::Owner(user_id))
            .await
            .expect("ok");
        assert!(list.is_empty());
        assert_eq!(total, 0);
    }

    // ----- @edge ---------------------------------------------------------------

    #[tokio::test]
    async fn edge_list_for_acp_with_auto_managed_acp_returns_empty() {
        let acp = make_acp(None);
        let acp_id = acp.id;
        let mut acp_repo = MockAcpRepo::new();
        acp_repo
            .expect_find_by_id()
            .returning(move |_| Ok(Some(acp.clone())));

        let building_repo = MockBuildingRepo::new();
        let uc = ListBuildingsUseCase::new(Arc::new(building_repo), Arc::new(acp_repo));
        let (list, total) = uc.list_for_acp(&page(), acp_id).await.expect("ok");
        assert!(list.is_empty());
        assert_eq!(total, 0);
    }

    // ----- @security -----------------------------------------------------------

    #[tokio::test]
    async fn security_list_for_unknown_acp_returns_acp_not_in_scope() {
        let mut acp_repo = MockAcpRepo::new();
        acp_repo.expect_find_by_id().returning(|_| Ok(None));
        let building_repo = MockBuildingRepo::new();
        let uc = ListBuildingsUseCase::new(Arc::new(building_repo), Arc::new(acp_repo));
        let err = uc.list_for_acp(&page(), Uuid::new_v4()).await.unwrap_err();
        assert!(matches!(err, AppError::AcpNotInScope { .. }));
    }

    // ----- @negative -----------------------------------------------------------

    #[tokio::test]
    async fn negative_repository_error_maps_to_database_apperror() {
        let mut building_repo = MockBuildingRepo::new();
        building_repo
            .expect_find_all_paginated()
            .returning(|_, _| Err("connection lost".to_string()));
        let acp_repo = MockAcpRepo::new();
        let uc = ListBuildingsUseCase::new(Arc::new(building_repo), Arc::new(acp_repo));
        let err = uc
            .list_for_scope(&page(), ListScope::All)
            .await
            .unwrap_err();
        match err {
            AppError::Database(msg) => assert!(msg.contains("connection lost")),
            other => panic!("expected Database, got {:?}", other),
        }
    }
}
