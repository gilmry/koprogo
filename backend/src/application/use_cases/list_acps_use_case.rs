//! `ListAcps` use-case — Story 1.3.
//!
//! Wrap `AcpRepository::list(scope)` derived from the caller's role
//! (`AcpCaller::list_scope()`). Provides a dedicated entry point separated
//! from the broader `AcpUseCases::list_acps` so that the scope-guard
//! middleware (cf. `infrastructure::web::middleware::scope_guard`) can
//! reuse the same `assert_caller_can_see_acp` helper without pulling in
//! the full CRUD use-cases.
//!
//! Permissions (cf. architecture §3.3 + ADR-0010) :
//! - `SuperAdmin` → `ListScope::All` (sees all ACPs)
//! - `Admin { org_id }` / `Syndic { org_id }` → `ListScope::Organization(org)`
//! - `Owner { user_id }` → `ListScope::Owner(user_id)`
//!
//! All returns are typed `Result<T, AppError>` — CRITICAL §4. No
//! `Result<_, String>` introduced (cf. epic #555). The wider repository
//! port `AcpRepository` already returns `AppError` natively (Story 1.1).

use crate::application::dto::AcpResponseDto;
use crate::application::error::AppError;
use crate::application::ports::AcpRepository;
use crate::application::use_cases::acp_use_cases::AcpCaller;
use crate::domain::entities::Acp;
use std::sync::Arc;
use uuid::Uuid;

/// Dedicated use-case for role-based ACP listing — Story 1.3.
///
/// Distinct from `AcpUseCases` to keep the middleware dependency surface
/// minimal: the scope_guard middleware needs *listing* + *scope-check*
/// helpers, not the full CRUD lifecycle.
pub struct ListAcpsUseCase {
    repository: Arc<dyn AcpRepository>,
}

impl ListAcpsUseCase {
    pub fn new(repository: Arc<dyn AcpRepository>) -> Self {
        Self { repository }
    }

    /// List ACPs visible to the caller (derived from role + scope).
    pub async fn list_for_user(&self, caller: &AcpCaller) -> Result<Vec<AcpResponseDto>, AppError> {
        let scope = caller.list_scope();
        let acps = self.repository.list(scope).await?;
        Ok(acps.iter().map(Self::to_response_dto).collect())
    }

    /// Assert that `caller` is allowed to see (read/use) the ACP `acp_id`.
    ///
    /// Used by the scope_guard middleware: the user may attach a
    /// `X-Scope-AcpId` header or `?acp_id=` query, and we must refuse 403
    /// `AcpNotInScope` if they try to address an ACP outside their
    /// effective scope.
    ///
    /// Implementation: load the requested ACP, then delegate to
    /// `caller_can_see(&caller, &acp)` (centralised here so the middleware
    /// does not need to know domain entities).
    pub async fn assert_caller_can_see(
        &self,
        caller: &AcpCaller,
        acp_id: Uuid,
    ) -> Result<(), AppError> {
        let acp = self
            .repository
            .find_by_id(acp_id)
            .await?
            .ok_or(AppError::AcpNotInScope { acp_id })?;
        Self::caller_can_see(caller, &acp)
    }

    /// Pure permission check (no I/O). Public so tests can exercise the
    /// rule without spinning up a repository.
    pub fn caller_can_see(caller: &AcpCaller, acp: &Acp) -> Result<(), AppError> {
        match caller {
            AcpCaller::SuperAdmin => Ok(()),
            AcpCaller::Admin { organization_id } | AcpCaller::Syndic { organization_id } => {
                match acp.organization_id {
                    Some(org) if org == *organization_id => Ok(()),
                    _ => Err(AppError::AcpNotInScope { acp_id: acp.id }),
                }
            }
            // Story 1.3 — owner direct visibility on a single ACP is
            // derived from `list_for_user(Owner { user_id })`. For a
            // direct id-lookup we conservatively refuse here: the
            // middleware will instead consult the listing to verify
            // belonging once Story 3.5 ships scope/scope_id on
            // user_role_assignments. Until then, an owner cannot pin a
            // specific ACP via X-Scope-AcpId.
            AcpCaller::Owner { .. } => Err(AppError::AcpNotInScope { acp_id: acp.id }),
        }
    }

    fn to_response_dto(acp: &Acp) -> AcpResponseDto {
        AcpResponseDto {
            id: acp.id.to_string(),
            organization_id: acp.organization_id.map(|u| u.to_string()),
            name: acp.name.clone(),
            slug: acp.slug.clone(),
            legal_status: acp.legal_status.as_db_str().to_string(),
            total_tantiemes: acp.total_tantiemes,
            bce_number: acp.bce_number.clone(),
            address_street: acp.address_street.clone(),
            address_postal_code: acp.address_postal_code.clone(),
            address_city: acp.address_city.clone(),
            created_at: acp.created_at.to_rfc3339(),
            updated_at: acp.updated_at.to_rfc3339(),
        }
    }
}

// ============================================================================
// Tests — taxonomie 4-cat (CRITICAL.md §3).
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::ports::{AcpRepository, ListScope};
    use crate::domain::entities::Acp;
    use async_trait::async_trait;
    use mockall::mock;

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

    fn acp(org_id: Option<Uuid>, name: &str) -> Acp {
        Acp::new(
            org_id,
            name.to_string(),
            "Rue X 1".to_string(),
            "1000".to_string(),
            "Bruxelles".to_string(),
            None,
        )
        .expect("valid acp")
    }

    // ----- @happy --------------------------------------------------------------

    #[tokio::test]
    async fn happy_super_admin_lists_all_acps() {
        let org_a = Uuid::new_v4();
        let org_b = Uuid::new_v4();
        let acps = vec![acp(Some(org_a), "Acp A1"), acp(Some(org_b), "Acp B1")];

        let mut repo = MockAcpRepo::new();
        let acps_clone = acps.clone();
        repo.expect_list()
            .withf(|s| matches!(s, ListScope::All))
            .returning(move |_| Ok(acps_clone.clone()));

        let uc = ListAcpsUseCase::new(Arc::new(repo));
        let list = uc.list_for_user(&AcpCaller::SuperAdmin).await.expect("ok");
        assert_eq!(list.len(), 2);
    }

    #[tokio::test]
    async fn happy_syndic_lists_only_own_cabinet() {
        let org_a = Uuid::new_v4();
        let mut repo = MockAcpRepo::new();
        repo.expect_list()
            .withf(move |s| matches!(s, ListScope::Organization(o) if *o == org_a))
            .returning(move |_| Ok(vec![acp(Some(org_a), "A1"), acp(Some(org_a), "A2")]));

        let uc = ListAcpsUseCase::new(Arc::new(repo));
        let list = uc
            .list_for_user(&AcpCaller::Syndic {
                organization_id: org_a,
            })
            .await
            .expect("ok");
        assert_eq!(list.len(), 2);
    }

    // ----- @edge ---------------------------------------------------------------

    #[tokio::test]
    async fn edge_owner_with_no_assignment_sees_empty_list() {
        let user_id = Uuid::new_v4();
        let mut repo = MockAcpRepo::new();
        repo.expect_list()
            .withf(move |s| matches!(s, ListScope::Owner(u) if *u == user_id))
            .returning(|_| Ok(vec![]));

        let uc = ListAcpsUseCase::new(Arc::new(repo));
        let list = uc
            .list_for_user(&AcpCaller::Owner { user_id })
            .await
            .expect("ok");
        assert!(list.is_empty());
    }

    // ----- @security -----------------------------------------------------------

    #[tokio::test]
    async fn security_syndic_cannot_see_acp_of_other_cabinet() {
        let cabinet_a = Uuid::new_v4();
        let cabinet_b = Uuid::new_v4();
        let target = acp(Some(cabinet_a), "Foreign Acp");
        let target_id = target.id;

        let mut repo = MockAcpRepo::new();
        let target_clone = target.clone();
        repo.expect_find_by_id()
            .returning(move |_| Ok(Some(target_clone.clone())));

        let uc = ListAcpsUseCase::new(Arc::new(repo));
        let err = uc
            .assert_caller_can_see(
                &AcpCaller::Syndic {
                    organization_id: cabinet_b,
                },
                target_id,
            )
            .await
            .unwrap_err();
        match err {
            AppError::AcpNotInScope { acp_id } => assert_eq!(acp_id, target_id),
            other => panic!("expected AcpNotInScope, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn security_owner_cannot_pin_arbitrary_acp_via_scope_header() {
        let user_id = Uuid::new_v4();
        let target = acp(Some(Uuid::new_v4()), "Some Acp");
        let target_id = target.id;
        let mut repo = MockAcpRepo::new();
        let target_clone = target.clone();
        repo.expect_find_by_id()
            .returning(move |_| Ok(Some(target_clone.clone())));

        let uc = ListAcpsUseCase::new(Arc::new(repo));
        let err = uc
            .assert_caller_can_see(&AcpCaller::Owner { user_id }, target_id)
            .await
            .unwrap_err();
        assert!(matches!(err, AppError::AcpNotInScope { .. }));
    }

    // ----- @negative -----------------------------------------------------------

    #[tokio::test]
    async fn negative_assert_unknown_acp_returns_acp_not_in_scope() {
        let mut repo = MockAcpRepo::new();
        repo.expect_find_by_id().returning(|_| Ok(None));
        let uc = ListAcpsUseCase::new(Arc::new(repo));
        let err = uc
            .assert_caller_can_see(&AcpCaller::SuperAdmin, Uuid::new_v4())
            .await
            .unwrap_err();
        assert!(matches!(err, AppError::AcpNotInScope { .. }));
    }
}
