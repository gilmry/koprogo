//! ACP use-cases — Story 1.1.
//!
//! 5 use-cases : `create`, `get`, `list`, `update`, `archive`.
//!
//! Permissions :
//! - `create` / `update` / `archive` : admin (superadmin OR admin role).
//! - `list` / `get` : tout authentifié (filtré par scope rôle).
//!
//! Tous les retours sont typés `Result<T, AppError>` (CRITICAL §4).
//!
//! L'audit est consigné via `infrastructure::audit::AuditLogEntry` côté
//! handler (pattern existant — cf. `building_handlers.rs`). Ce use-case
//! reste pur logique métier + permission.

use crate::application::dto::{AcpResponseDto, CreateAcpDto, UpdateAcpDto};
use crate::application::error::AppError;
use crate::application::ports::{AcpRepository, ListScope, OrganizationRepository};
use crate::domain::entities::Acp;
use std::sync::Arc;
use uuid::Uuid;

/// Rôle effectif de l'appelant pour les permissions ACP.
///
/// Mappé depuis `AuthenticatedUser.role` côté handler. Le mapping vit ici
/// (et pas dans `web::middleware`) pour rester testable en pur Rust sans
/// AppState.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AcpCaller {
    /// SuperAdmin SaaS — accès global, peut tout faire.
    SuperAdmin,
    /// Admin métier d'un cabinet syndic — peut CRUD sur les ACPs de son
    /// cabinet.
    Admin { organization_id: Uuid },
    /// Syndic — lecture seule des ACPs de son cabinet (Story 1.1 ; les
    /// permissions write s'étoffent en stories suivantes).
    Syndic { organization_id: Uuid },
    /// Owner — lecture seule des ACPs où il a un rôle assigné.
    Owner { user_id: Uuid },
}

impl AcpCaller {
    /// L'appelant a-t-il le droit de créer/mettre à jour/archiver une ACP ?
    /// Story 1.1 : seul `SuperAdmin` ou `Admin` (admin métier cabinet).
    pub fn can_mutate(&self) -> bool {
        matches!(self, AcpCaller::SuperAdmin | AcpCaller::Admin { .. })
    }

    /// Scope de listing par rôle. Le SuperAdmin voit tout.
    pub fn list_scope(&self) -> ListScope {
        match self {
            AcpCaller::SuperAdmin => ListScope::All,
            AcpCaller::Admin { organization_id } | AcpCaller::Syndic { organization_id } => {
                ListScope::Organization(*organization_id)
            }
            AcpCaller::Owner { user_id } => ListScope::Owner(*user_id),
        }
    }
}

pub struct AcpUseCases {
    repository: Arc<dyn AcpRepository>,
    organization_repository: Arc<dyn OrganizationRepository>,
}

impl AcpUseCases {
    pub fn new(
        repository: Arc<dyn AcpRepository>,
        organization_repository: Arc<dyn OrganizationRepository>,
    ) -> Self {
        Self {
            repository,
            organization_repository,
        }
    }

    /// Crée une ACP. Admin only.
    pub async fn create_acp(
        &self,
        caller: &AcpCaller,
        dto: CreateAcpDto,
    ) -> Result<AcpResponseDto, AppError> {
        if !caller.can_mutate() {
            return Err(AppError::Forbidden(
                "Only admin can create ACPs".to_string(),
            ));
        }

        // Parse organization_id si fourni, et vérifie qu'il existe en DB.
        let org_id = match dto.organization_id.as_deref() {
            Some(s) if !s.is_empty() => {
                let parsed = Uuid::parse_str(s).map_err(|_| {
                    AppError::Validation("Invalid organization_id format".to_string())
                })?;
                // Vérification d'existence : pattern story 1.1 §AC @negative.
                let exists = self
                    .organization_repository
                    .find_by_id(parsed)
                    .await
                    .map_err(AppError::from)?
                    .is_some();
                if !exists {
                    return Err(AppError::Validation(format!(
                        "Organization {} does not exist",
                        parsed
                    )));
                }
                Some(parsed)
            }
            _ => None,
        };

        // Admin métier ne peut créer une ACP que dans son propre cabinet
        // (ou auto-gérée). Le SuperAdmin peut tout.
        if let AcpCaller::Admin { organization_id } = caller {
            if let Some(target) = org_id {
                if target != *organization_id {
                    return Err(AppError::Forbidden(
                        "Admin can only create ACPs within their own organization".to_string(),
                    ));
                }
            }
        }

        let acp = Acp::new(
            org_id,
            dto.name,
            dto.address_street,
            dto.address_postal_code,
            dto.address_city,
            dto.bce_number,
        )?; // AcpError -> AppError::Validation via From impl

        let created = self.repository.create(&acp).await?;
        Ok(Self::to_response_dto(&created))
    }

    /// Récupère une ACP par id, avec scope guard.
    pub async fn get_acp(&self, caller: &AcpCaller, id: Uuid) -> Result<AcpResponseDto, AppError> {
        let acp = self
            .repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("ACP {} not found", id)))?;

        Self::assert_scope(caller, &acp)?;
        Ok(Self::to_response_dto(&acp))
    }

    /// Liste les ACPs visibles pour l'appelant.
    pub async fn list_acps(&self, caller: &AcpCaller) -> Result<Vec<AcpResponseDto>, AppError> {
        let scope = caller.list_scope();
        let acps = self.repository.list(scope).await?;
        Ok(acps.iter().map(Self::to_response_dto).collect())
    }

    /// Met à jour une ACP. Admin only.
    pub async fn update_acp(
        &self,
        caller: &AcpCaller,
        id: Uuid,
        dto: UpdateAcpDto,
    ) -> Result<AcpResponseDto, AppError> {
        if !caller.can_mutate() {
            return Err(AppError::Forbidden(
                "Only admin can update ACPs".to_string(),
            ));
        }

        let mut acp = self
            .repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("ACP {} not found", id)))?;

        Self::assert_scope(caller, &acp)?;

        // Si on demande explicitement à changer l'organization_id.
        if let Some(opt_str) = dto.organization_id {
            let new_org_id = match opt_str {
                Some(s) if !s.is_empty() => {
                    let parsed = Uuid::parse_str(&s).map_err(|_| {
                        AppError::Validation("Invalid organization_id format".to_string())
                    })?;
                    let exists = self
                        .organization_repository
                        .find_by_id(parsed)
                        .await
                        .map_err(AppError::from)?
                        .is_some();
                    if !exists {
                        return Err(AppError::Validation(format!(
                            "Organization {} does not exist",
                            parsed
                        )));
                    }
                    Some(parsed)
                }
                _ => None,
            };

            // Admin métier ne peut pas déplacer une ACP vers un autre cabinet.
            if let AcpCaller::Admin { organization_id } = caller {
                if let Some(t) = new_org_id {
                    if t != *organization_id {
                        return Err(AppError::Forbidden(
                            "Admin cannot move ACP to a different organization".to_string(),
                        ));
                    }
                }
            }

            acp.set_organization(new_org_id);
        }

        acp.update_info(
            dto.name,
            dto.address_street,
            dto.address_postal_code,
            dto.address_city,
            dto.bce_number,
        )?;

        let updated = self.repository.update(&acp).await?;
        Ok(Self::to_response_dto(&updated))
    }

    /// Archive (=DELETE physique en v0.1.0) une ACP. Admin only.
    pub async fn archive_acp(&self, caller: &AcpCaller, id: Uuid) -> Result<(), AppError> {
        if !caller.can_mutate() {
            return Err(AppError::Forbidden(
                "Only admin can archive ACPs".to_string(),
            ));
        }
        let acp = self
            .repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("ACP {} not found", id)))?;
        Self::assert_scope(caller, &acp)?;
        self.repository.archive(id).await
    }

    /// Public scope-check used by Story 1.3 `scope_guard` middleware.
    /// Loads the ACP by id then delegates to `assert_scope`.
    /// Refuses with `AcpNotInScope` if the ACP does not exist (no
    /// resource-existence leak across cabinets).
    pub async fn assert_can_see_acp(
        &self,
        caller: &AcpCaller,
        acp_id: Uuid,
    ) -> Result<(), AppError> {
        let acp = self
            .repository
            .find_by_id(acp_id)
            .await?
            .ok_or(AppError::AcpNotInScope { acp_id })?;
        Self::assert_scope(caller, &acp)
    }

    /// Vérifie que l'appelant a le droit de voir cette ACP précise.
    /// Centralise la logique pour `get` / `update` / `archive` — un seul
    /// chemin = un seul test à maintenir (cf. mémoire `audit-to-issue-first`).
    fn assert_scope(caller: &AcpCaller, acp: &Acp) -> Result<(), AppError> {
        match caller {
            AcpCaller::SuperAdmin => Ok(()),
            AcpCaller::Admin { organization_id } | AcpCaller::Syndic { organization_id } => {
                match acp.organization_id {
                    Some(org) if org == *organization_id => Ok(()),
                    _ => Err(AppError::AcpNotInScope { acp_id: acp.id }),
                }
            }
            // Owner scope : Story 1.1 — on délègue la vérification fine
            // (UserRoleAssignment) au repository `list`. Pour `get`, on
            // refuse par défaut sauf si l'ACP est ressortie de `list` —
            // ici on refuse car on n'a pas la table user_role_assignment
            // sur l'ACP encore. Story 1.3 enrichira.
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
// Tests — taxonomie 4-cat avec mocks (CRITICAL.md §3).
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::ports::{AcpRepository, ListScope, OrganizationRepository};
    use crate::domain::entities::{Acp, Organization, SubscriptionPlan};
    use async_trait::async_trait;
    use mockall::mock;

    mock! {
        AcpRepo {}

        #[async_trait]
        impl AcpRepository for AcpRepo {
            async fn create(&self, acp: &Acp) -> Result<Acp, AppError>;
            async fn find_by_id(&self, id: Uuid) -> Result<Option<Acp>, AppError>;
            async fn list(&self, scope: ListScope) -> Result<Vec<Acp>, AppError>;
            async fn update(&self, acp: &Acp) -> Result<Acp, AppError>;
            async fn archive(&self, id: Uuid) -> Result<(), AppError>;
            async fn count_buildings(&self, id: Uuid) -> Result<i64, AppError>;
        }
    }

    mock! {
        OrgRepo {}

        #[async_trait]
        impl OrganizationRepository for OrgRepo {
            async fn create(&self, org: &Organization) -> Result<Organization, String>;
            async fn find_by_id(&self, id: Uuid) -> Result<Option<Organization>, String>;
            async fn find_by_slug(&self, slug: &str) -> Result<Option<Organization>, String>;
            async fn find_all(&self) -> Result<Vec<Organization>, String>;
            async fn update(&self, org: &Organization) -> Result<Organization, String>;
            async fn delete(&self, id: Uuid) -> Result<bool, String>;
            async fn count_buildings(&self, org_id: Uuid) -> Result<i64, String>;
        }
    }

    fn make_dto(name: &str, org_id: Option<Uuid>) -> CreateAcpDto {
        CreateAcpDto {
            organization_id: org_id.map(|u| u.to_string()),
            name: name.to_string(),
            address_street: "Rue X 1".to_string(),
            address_postal_code: "1000".to_string(),
            address_city: "Bruxelles".to_string(),
            bce_number: None,
        }
    }

    fn make_org(id: Uuid) -> Organization {
        let mut o = Organization::new(
            "Test Cabinet".to_string(),
            "test@cabinet.be".to_string(),
            None,
            SubscriptionPlan::Starter,
        )
        .expect("valid org");
        o.id = id;
        o
    }

    // ----- @happy --------------------------------------------------------------

    #[tokio::test]
    async fn happy_admin_creates_acp_with_organization() {
        let org_id = Uuid::new_v4();
        let mut acp_repo = MockAcpRepo::new();
        let mut org_repo = MockOrgRepo::new();

        org_repo
            .expect_find_by_id()
            .returning(move |id| Ok(Some(make_org(id))));
        acp_repo.expect_create().returning(|a| Ok(a.clone()));

        let uc = AcpUseCases::new(Arc::new(acp_repo), Arc::new(org_repo));
        let dto = make_dto("Residence Maury", Some(org_id));
        let res = uc.create_acp(&AcpCaller::SuperAdmin, dto).await;

        assert!(res.is_ok(), "expected Ok, got {:?}", res);
        let resp = res.unwrap();
        assert_eq!(resp.name, "Residence Maury");
        assert_eq!(resp.organization_id, Some(org_id.to_string()));
    }

    #[tokio::test]
    async fn happy_admin_creates_self_managed_acp() {
        let mut acp_repo = MockAcpRepo::new();
        let org_repo = MockOrgRepo::new();
        acp_repo.expect_create().returning(|a| Ok(a.clone()));

        let uc = AcpUseCases::new(Arc::new(acp_repo), Arc::new(org_repo));
        let dto = make_dto("Autogeree", None);
        let resp = uc
            .create_acp(&AcpCaller::SuperAdmin, dto)
            .await
            .expect("ok");
        assert!(resp.organization_id.is_none());
    }

    // ----- @edge --------------------------------------------------------------

    #[tokio::test]
    async fn edge_create_with_empty_org_id_string_is_treated_as_none() {
        let mut acp_repo = MockAcpRepo::new();
        let org_repo = MockOrgRepo::new();
        acp_repo.expect_create().returning(|a| Ok(a.clone()));

        let uc = AcpUseCases::new(Arc::new(acp_repo), Arc::new(org_repo));
        let dto = CreateAcpDto {
            organization_id: Some("".to_string()),
            name: "Edge Acp".to_string(),
            address_street: "Rue X 1".to_string(),
            address_postal_code: "1000".to_string(),
            address_city: "Bruxelles".to_string(),
            bce_number: None,
        };
        let resp = uc.create_acp(&AcpCaller::SuperAdmin, dto).await.unwrap();
        assert!(resp.organization_id.is_none());
    }

    // ----- @security ----------------------------------------------------------

    #[tokio::test]
    async fn security_non_admin_cannot_create_acp() {
        let acp_repo = MockAcpRepo::new();
        let org_repo = MockOrgRepo::new();
        let uc = AcpUseCases::new(Arc::new(acp_repo), Arc::new(org_repo));
        let dto = make_dto("X", None);
        let err = uc
            .create_acp(
                &AcpCaller::Syndic {
                    organization_id: Uuid::new_v4(),
                },
                dto,
            )
            .await
            .unwrap_err();
        match err {
            AppError::Forbidden(_) => {}
            other => panic!("expected Forbidden, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn security_syndic_cabinet_b_cannot_read_acp_of_cabinet_a() {
        let cabinet_a = Uuid::new_v4();
        let cabinet_b = Uuid::new_v4();

        let acp = Acp::new(
            Some(cabinet_a),
            "Acp A".to_string(),
            "Rue X 1".to_string(),
            "1000".to_string(),
            "Bruxelles".to_string(),
            None,
        )
        .unwrap();
        let acp_id = acp.id;

        let mut acp_repo = MockAcpRepo::new();
        let org_repo = MockOrgRepo::new();
        acp_repo
            .expect_find_by_id()
            .returning(move |_| Ok(Some(acp.clone())));

        let uc = AcpUseCases::new(Arc::new(acp_repo), Arc::new(org_repo));
        let err = uc
            .get_acp(
                &AcpCaller::Syndic {
                    organization_id: cabinet_b,
                },
                acp_id,
            )
            .await
            .unwrap_err();

        match err {
            AppError::AcpNotInScope { acp_id: a } => assert_eq!(a, acp_id),
            other => panic!("expected AcpNotInScope, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn security_admin_cannot_create_acp_in_different_cabinet() {
        let own_cabinet = Uuid::new_v4();
        let other_cabinet = Uuid::new_v4();

        let mut org_repo = MockOrgRepo::new();
        org_repo
            .expect_find_by_id()
            .returning(move |id| Ok(Some(make_org(id))));
        let acp_repo = MockAcpRepo::new();
        let uc = AcpUseCases::new(Arc::new(acp_repo), Arc::new(org_repo));

        let dto = make_dto("Forbidden Acp", Some(other_cabinet));
        let err = uc
            .create_acp(
                &AcpCaller::Admin {
                    organization_id: own_cabinet,
                },
                dto,
            )
            .await
            .unwrap_err();
        match err {
            AppError::Forbidden(_) => {}
            other => panic!("expected Forbidden, got {:?}", other),
        }
    }

    // ----- @negative ----------------------------------------------------------

    #[tokio::test]
    async fn negative_create_with_unknown_organization_returns_validation() {
        let mut org_repo = MockOrgRepo::new();
        org_repo.expect_find_by_id().returning(|_| Ok(None));
        let acp_repo = MockAcpRepo::new();
        let uc = AcpUseCases::new(Arc::new(acp_repo), Arc::new(org_repo));

        let dto = make_dto("Phantom", Some(Uuid::new_v4()));
        let err = uc
            .create_acp(&AcpCaller::SuperAdmin, dto)
            .await
            .unwrap_err();
        match err {
            AppError::Validation(_) => {}
            other => panic!("expected Validation, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn negative_get_unknown_id_returns_not_found() {
        let mut acp_repo = MockAcpRepo::new();
        let org_repo = MockOrgRepo::new();
        acp_repo.expect_find_by_id().returning(|_| Ok(None));
        let uc = AcpUseCases::new(Arc::new(acp_repo), Arc::new(org_repo));

        let err = uc
            .get_acp(&AcpCaller::SuperAdmin, Uuid::new_v4())
            .await
            .unwrap_err();
        match err {
            AppError::NotFound(_) => {}
            other => panic!("expected NotFound, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn negative_update_unknown_id_returns_not_found() {
        let mut acp_repo = MockAcpRepo::new();
        let org_repo = MockOrgRepo::new();
        acp_repo.expect_find_by_id().returning(|_| Ok(None));
        let uc = AcpUseCases::new(Arc::new(acp_repo), Arc::new(org_repo));

        let dto = UpdateAcpDto {
            organization_id: None,
            name: "X".to_string(),
            address_street: "Rue X".to_string(),
            address_postal_code: "1000".to_string(),
            address_city: "Bruxelles".to_string(),
            bce_number: None,
        };
        let err = uc
            .update_acp(&AcpCaller::SuperAdmin, Uuid::new_v4(), dto)
            .await
            .unwrap_err();
        match err {
            // "X" trop court → AppError::Validation prend le dessus si on a
            // déjà la ressource ; sinon NotFound. find_by_id renvoie None →
            // NotFound est prioritaire.
            AppError::NotFound(_) => {}
            other => panic!("expected NotFound, got {:?}", other),
        }
    }
}
