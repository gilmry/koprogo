use crate::application::error::AppError;
use crate::application::ports::{UserRepository, UserRoleRepository};
use crate::domain::entities::{User, UserRole, UserRoleAssignment};
use chrono::{DateTime, Utc};
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Serialize, Clone)]
pub struct RoleResponse {
    pub id: String,
    pub role: String,
    pub organization_id: Option<String>,
    pub is_primary: bool,
}

#[derive(Serialize, Clone)]
pub struct UserResponse {
    pub id: String,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub role: String,
    pub organization_id: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub roles: Vec<RoleResponse>,
    pub active_role: Option<RoleResponse>,
}

pub struct UserUseCases {
    user_repo: Arc<dyn UserRepository>,
    role_repo: Arc<dyn UserRoleRepository>,
}

impl UserUseCases {
    pub fn new(user_repo: Arc<dyn UserRepository>, role_repo: Arc<dyn UserRoleRepository>) -> Self {
        Self {
            user_repo,
            role_repo,
        }
    }

    fn to_role_response(a: &UserRoleAssignment) -> RoleResponse {
        RoleResponse {
            id: a.id.to_string(),
            role: a.role.to_string(),
            organization_id: a.organization_id.map(|id| id.to_string()),
            is_primary: a.is_primary,
        }
    }

    fn fallback_role(role: &str, organization_id: Option<Uuid>) -> RoleResponse {
        RoleResponse {
            id: Uuid::new_v4().to_string(),
            role: role.to_string(),
            organization_id: organization_id.map(|id| id.to_string()),
            is_primary: true,
        }
    }

    fn ensure_primary(roles: &mut [RoleResponse]) {
        if roles.is_empty() {
            return;
        }
        if roles.iter().filter(|r| r.is_primary).count() == 0 {
            roles[0].is_primary = true;
        }
        roles.sort_by_key(|r| std::cmp::Reverse(r.is_primary));
    }

    fn build_response(user: User, assignments: Vec<UserRoleAssignment>) -> UserResponse {
        let mut roles: Vec<RoleResponse> = if assignments.is_empty() {
            vec![Self::fallback_role(
                &user.role.to_string(),
                user.organization_id,
            )]
        } else {
            assignments.iter().map(Self::to_role_response).collect()
        };

        Self::ensure_primary(&mut roles);
        let active_role = roles
            .iter()
            .find(|r| r.is_primary)
            .cloned()
            .or_else(|| roles.first().cloned());

        UserResponse {
            id: user.id.to_string(),
            email: user.email.clone(),
            first_name: user.first_name.clone(),
            last_name: user.last_name.clone(),
            role: active_role
                .as_ref()
                .map(|r| r.role.clone())
                .unwrap_or_else(|| user.role.to_string()),
            organization_id: active_role
                .as_ref()
                .and_then(|r| r.organization_id.clone())
                .or_else(|| user.organization_id.map(|id| id.to_string())),
            is_active: user.is_active,
            created_at: user.created_at,
            roles,
            active_role,
        }
    }

    /// Look up a user's display name by id (first_name + last_name).
    /// Returns None if the user does not exist.
    pub async fn find_display_name(&self, id: Uuid) -> Result<Option<String>, String> {
        Ok(self.user_repo.find_by_id(id).await?.map(|u| {
            format!("{} {}", u.first_name, u.last_name)
                .trim()
                .to_string()
        }))
    }

    /// List all users with their roles.
    pub async fn list_all(&self) -> Result<Vec<UserResponse>, String> {
        let users = self.user_repo.find_all().await?;
        let user_ids: Vec<Uuid> = users.iter().map(|u| u.id).collect();
        let mut roles_map: HashMap<Uuid, Vec<UserRoleAssignment>> =
            self.role_repo.list_for_users(&user_ids).await?;

        Ok(users
            .into_iter()
            .map(|user| {
                let assignments = roles_map.remove(&user.id).unwrap_or_default();
                Self::build_response(user, assignments)
            })
            .collect())
    }

    /// List users belonging to a given organization, with their roles.
    ///
    /// Authorization (syndic/accountant own org, superadmin any org) is
    /// enforced upstream by the handler via `AuthenticatedUser::verify_org_access`.
    pub async fn list_by_organization(
        &self,
        organization_id: Uuid,
    ) -> Result<Vec<UserResponse>, String> {
        let users = self.user_repo.find_by_organization(organization_id).await?;
        let user_ids: Vec<Uuid> = users.iter().map(|u| u.id).collect();
        let mut roles_map: HashMap<Uuid, Vec<UserRoleAssignment>> =
            self.role_repo.list_for_users(&user_ids).await?;

        Ok(users
            .into_iter()
            .map(|user| {
                let assignments = roles_map.remove(&user.id).unwrap_or_default();
                Self::build_response(user, assignments)
            })
            .collect())
    }

    /// Create a new user with role assignments.
    /// Returns `Err("email_exists")` on duplicate email.
    pub async fn create(
        &self,
        email: String,
        password_hash: String,
        first_name: String,
        last_name: String,
        primary_role: UserRole,
        primary_org: Option<Uuid>,
        role_assignments: Vec<UserRoleAssignment>,
    ) -> Result<UserResponse, String> {
        let user = User::new(
            email,
            password_hash,
            first_name,
            last_name,
            primary_role,
            primary_org,
        )?;
        let created = self.user_repo.create(&user).await?;

        // Build assignments with the real user_id
        let assignments: Vec<UserRoleAssignment> = role_assignments
            .into_iter()
            .map(|mut a| {
                a.user_id = created.id;
                a
            })
            .collect();
        self.role_repo.replace_all(created.id, &assignments).await?;

        let final_roles = self.role_repo.list_for_user(created.id).await?;
        Ok(Self::build_response(created, final_roles))
    }

    /// Update an existing user. Returns `None` if not found.
    /// Returns `Err("email_exists")` on duplicate email.
    pub async fn update(
        &self,
        id: Uuid,
        email: String,
        first_name: String,
        last_name: String,
        primary_role: UserRole,
        primary_org: Option<Uuid>,
        password_hash: Option<String>,
        role_assignments: Vec<UserRoleAssignment>,
    ) -> Result<Option<UserResponse>, String> {
        let mut user = match self.user_repo.find_by_id(id).await? {
            Some(u) => u,
            None => return Ok(None),
        };

        user.email = email.trim().to_lowercase();
        user.first_name = first_name.trim().to_string();
        user.last_name = last_name.trim().to_string();
        user.role = primary_role;
        user.organization_id = primary_org;
        user.updated_at = Utc::now();

        if let Some(pw) = password_hash {
            self.user_repo.update_password(id, &pw).await?;
        }

        self.user_repo.update(&user).await?;

        let assignments: Vec<UserRoleAssignment> = role_assignments
            .into_iter()
            .map(|mut a| {
                a.user_id = id;
                a
            })
            .collect();
        self.role_repo.replace_all(id, &assignments).await?;

        let final_roles = self.role_repo.list_for_user(id).await?;
        Ok(Some(Self::build_response(user, final_roles)))
    }

    /// Activate a user. Returns `None` if not found.
    pub async fn activate(&self, id: Uuid) -> Result<Option<UserResponse>, String> {
        let user = match self.user_repo.activate(id).await? {
            Some(u) => u,
            None => return Ok(None),
        };
        let roles = self.role_repo.list_for_user(id).await?;
        Ok(Some(Self::build_response(user, roles)))
    }

    /// Deactivate a user. Returns `None` if not found.
    pub async fn deactivate(&self, id: Uuid) -> Result<Option<UserResponse>, String> {
        let user = match self.user_repo.deactivate(id).await? {
            Some(u) => u,
            None => return Ok(None),
        };
        let roles = self.role_repo.list_for_user(id).await?;
        Ok(Some(Self::build_response(user, roles)))
    }

    /// Delete a user. Returns `false` if not found.
    pub async fn delete(&self, id: Uuid) -> Result<bool, String> {
        self.user_repo.delete(id).await
    }

    /// Verify a user exists and holds the given role.
    /// Returns `Err("User not found")` or `Err("User must have role '...' ...")`.
    pub async fn validate_user_has_role(&self, user_id: Uuid, role: &str) -> Result<(), String> {
        self.user_repo
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| "User not found".to_string())?;

        let roles = self.role_repo.list_for_user(user_id).await?;
        if !roles.iter().any(|r| r.role.to_string() == role) {
            return Err(format!(
                "User must have role '{}' to be linked to an owner entity",
                role
            ));
        }
        Ok(())
    }

    // ======================================================================
    // Story B0bis — CRUD REST sur user_role_assignments (gap Story 3.1).
    //
    // Story 3.1 a livré l'entité + repo + helpers RBAC mais n'a JAMAIS exposé
    // un endpoint pour assigner / lister / révoquer un sous-rôle. Cela
    // bloquait toute UI d'administration ainsi que la Story B1 du Phase B FE
    // (`RoleAssignmentForm` / `RoleAssignmentList`).
    //
    // Ces 3 méthodes sont **AppError-typées** (cf. CRITICAL.md §4) et
    // appliquent les invariants :
    //   - target user existant ;
    //   - role parsable (whitelist UserRole::from_str) ;
    //   - valid_until > now() si Some ;
    //   - aucune double-attribution active sur le tuple
    //     (user_id, role, organization_id).
    // ======================================================================

    /// Assigne un sous-rôle à un user existant.
    ///
    /// L'autorisation (superadmin / syndic-sur-son-org) est vérifiée AMONT par
    /// le handler — ici on applique uniquement les invariants métier.
    ///
    /// # Errors
    ///
    /// - `AppError::NotFound` si `target_user_id` n'existe pas.
    /// - `AppError::Validation` si `valid_until` n'est pas dans le futur, ou
    ///   si un user tente de s'auto-attribuer un rôle à privilèges élevés
    ///   (cohérent invariants MagicLink / Mandate §3.2 §3.4).
    /// - `AppError::RoleAlreadyAssigned` si une assignment active existe déjà
    ///   pour le tuple `(user_id, role, organization_id)`.
    pub async fn assign_role(
        &self,
        target_user_id: Uuid,
        role: UserRole,
        organization_id: Option<Uuid>,
        valid_until: Option<DateTime<Utc>>,
        granted_by_user_id: Uuid,
    ) -> Result<UserRoleAssignment, AppError> {
        // --- @edge : valid_until doit être strictement futur si fourni -----
        if let Some(t) = valid_until {
            if t <= Utc::now() {
                return Err(AppError::Validation(
                    "valid_until must be strictly in the future".to_string(),
                ));
            }
        }

        // --- @security : auto-attribution interdite pour rôles à privilèges
        //     élevés (superadmin / syndic). Cohérent avec l'invariant
        //     subject!=issuer des MagicLink / Mandate / RoleDelegation.
        let high_privilege = matches!(role, UserRole::SuperAdmin | UserRole::Syndic);
        if high_privilege && granted_by_user_id == target_user_id {
            return Err(AppError::Validation(
                "A user cannot self-grant a high-privilege role".to_string(),
            ));
        }

        // --- target user doit exister -------------------------------------
        let target = self
            .user_repo
            .find_by_id(target_user_id)
            .await
            .map_err(AppError::from)?
            .ok_or_else(|| AppError::NotFound(format!("User {} not found", target_user_id)))?;

        // --- @negative : double-attribution -------------------------------
        let existing = self
            .role_repo
            .list_for_user(target_user_id)
            .await
            .map_err(AppError::from)?;
        let already = existing.iter().any(|a| {
            a.role == role && a.organization_id == organization_id && a.is_currently_active()
        });
        if already {
            return Err(AppError::RoleAlreadyAssigned {
                user_id: target_user_id,
                role: role.to_string(),
            });
        }

        // --- Build assignment (delegated if `valid_until` is Some) --------
        let is_first = existing.is_empty();
        let assignment = match valid_until {
            Some(t) => UserRoleAssignment::new_delegated(
                target_user_id,
                role,
                organization_id,
                t,
                granted_by_user_id,
            ),
            None => UserRoleAssignment::new(target_user_id, role, organization_id, is_first),
        };

        let saved = self
            .role_repo
            .create(&assignment)
            .await
            .map_err(AppError::from)?;
        // Silence the "unused" lint on the user we fetched to enforce existence.
        let _ = target.id;
        Ok(saved)
    }

    /// Liste toutes les assignments d'un user (actives + expirées) afin que
    /// le FE puisse afficher l'historique.
    ///
    /// L'autorisation (superadmin / syndic-sur-son-org / self) est vérifiée
    /// AMONT par le handler.
    pub async fn list_assignments_for_user(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<UserRoleAssignment>, AppError> {
        self.role_repo
            .list_for_user(user_id)
            .await
            .map_err(AppError::from)
    }

    /// Révoque (= supprime) une assignment.
    ///
    /// L'autorisation est vérifiée AMONT par le handler. Renvoie
    /// `AppError::NotFound` si l'assignment n'existe pas.
    pub async fn revoke_assignment(&self, assignment_id: Uuid) -> Result<(), AppError> {
        let existing = self
            .role_repo
            .find_by_id(assignment_id)
            .await
            .map_err(AppError::from)?
            .ok_or_else(|| {
                AppError::NotFound(format!("Role assignment {} not found", assignment_id))
            })?;
        let _ = existing.id;
        let deleted = self
            .role_repo
            .delete_by_id(assignment_id)
            .await
            .map_err(AppError::from)?;
        if !deleted {
            // Race condition : found-then-deleted-by-someone-else.
            return Err(AppError::NotFound(format!(
                "Role assignment {} not found",
                assignment_id
            )));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::ports::user_repository::MockUserRepo;
    use crate::application::ports::user_role_repository::MockUserRoleRepo;
    use chrono::Utc;

    fn make_user(id: Uuid) -> User {
        User {
            id,
            email: "alice@example.com".to_string(),
            password_hash: "hash".to_string(),
            first_name: "Alice".to_string(),
            last_name: "Smith".to_string(),
            role: UserRole::Syndic,
            organization_id: Some(Uuid::new_v4()),
            is_active: true,
            processing_restricted: false,
            processing_restricted_at: None,
            marketing_opt_out: false,
            marketing_opt_out_at: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[tokio::test]
    async fn test_list_all_returns_users_with_roles() {
        let user_id = Uuid::new_v4();
        let user = make_user(user_id);

        let mut mock_user = MockUserRepo::new();
        mock_user
            .expect_find_all()
            .returning(move || Ok(vec![user.clone()]));

        let mut mock_role = MockUserRoleRepo::new();
        mock_role
            .expect_list_for_users()
            .returning(|_| Ok(std::collections::HashMap::new()));

        let uc = UserUseCases::new(Arc::new(mock_user), Arc::new(mock_role));
        let result = uc.list_all().await.unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].email, "alice@example.com");
    }

    #[tokio::test]
    async fn test_activate_not_found() {
        let mut mock_user = MockUserRepo::new();
        mock_user.expect_activate().returning(|_| Ok(None));

        let mock_role = MockUserRoleRepo::new();
        let uc = UserUseCases::new(Arc::new(mock_user), Arc::new(mock_role));
        let result = uc.activate(Uuid::new_v4()).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_delete_delegates_to_repo() {
        let mut mock_user = MockUserRepo::new();
        mock_user.expect_delete().returning(|_| Ok(true));

        let mock_role = MockUserRoleRepo::new();
        let uc = UserUseCases::new(Arc::new(mock_user), Arc::new(mock_role));
        let result = uc.delete(Uuid::new_v4()).await.unwrap();
        assert!(result);
    }

    // ======================================================================
    // Story B0bis — assign_role / list_assignments_for_user / revoke
    // — taxonomie 4 catégories (CRITICAL.md §3).
    // ======================================================================

    fn make_uc_with_mocks(mock_user: MockUserRepo, mock_role: MockUserRoleRepo) -> UserUseCases {
        UserUseCases::new(Arc::new(mock_user), Arc::new(mock_role))
    }

    // ---- @happy ---------------------------------------------------------

    #[tokio::test]
    async fn happy_assign_role_creates_native_assignment() {
        let target = Uuid::new_v4();
        let granted_by = Uuid::new_v4();
        let user = make_user(target);

        let mut mock_user = MockUserRepo::new();
        let u = user.clone();
        mock_user
            .expect_find_by_id()
            .returning(move |_| Ok(Some(u.clone())));

        let mut mock_role = MockUserRoleRepo::new();
        mock_role.expect_list_for_user().returning(|_| Ok(vec![]));
        mock_role
            .expect_create()
            .returning(|a: &UserRoleAssignment| Ok(a.clone()));

        let uc = make_uc_with_mocks(mock_user, mock_role);
        let saved = uc
            .assign_role(target, UserRole::AccountantEncodeur, None, None, granted_by)
            .await
            .unwrap();
        assert_eq!(saved.user_id, target);
        assert_eq!(saved.role, UserRole::AccountantEncodeur);
        assert!(saved.valid_until.is_none(), "native assignment");
    }

    #[tokio::test]
    async fn happy_list_assignments_returns_repo_rows() {
        let user_id = Uuid::new_v4();
        let a = UserRoleAssignment::new(user_id, UserRole::Owner, None, true);

        let mock_user = MockUserRepo::new();
        let mut mock_role = MockUserRoleRepo::new();
        let row = a.clone();
        mock_role
            .expect_list_for_user()
            .returning(move |_| Ok(vec![row.clone()]));

        let uc = make_uc_with_mocks(mock_user, mock_role);
        let rows = uc.list_assignments_for_user(user_id).await.unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].id, a.id);
    }

    #[tokio::test]
    async fn happy_revoke_assignment_deletes_existing() {
        let user_id = Uuid::new_v4();
        let a = UserRoleAssignment::new(user_id, UserRole::Owner, None, true);
        let a_id = a.id;

        let mock_user = MockUserRepo::new();
        let mut mock_role = MockUserRoleRepo::new();
        let row = a.clone();
        mock_role
            .expect_find_by_id()
            .returning(move |_| Ok(Some(row.clone())));
        mock_role.expect_delete_by_id().returning(|_| Ok(true));

        let uc = make_uc_with_mocks(mock_user, mock_role);
        uc.revoke_assignment(a_id).await.unwrap();
    }

    // ---- @edge ----------------------------------------------------------

    #[tokio::test]
    async fn edge_assign_role_with_future_valid_until_creates_delegated() {
        let target = Uuid::new_v4();
        let granted_by = Uuid::new_v4();
        let user = make_user(target);

        let mut mock_user = MockUserRepo::new();
        let u = user.clone();
        mock_user
            .expect_find_by_id()
            .returning(move |_| Ok(Some(u.clone())));

        let mut mock_role = MockUserRoleRepo::new();
        mock_role.expect_list_for_user().returning(|_| Ok(vec![]));
        mock_role
            .expect_create()
            .returning(|a: &UserRoleAssignment| Ok(a.clone()));

        let uc = make_uc_with_mocks(mock_user, mock_role);
        let valid_until = Utc::now() + chrono::Duration::seconds(60);
        let saved = uc
            .assign_role(
                target,
                UserRole::Lawyer,
                None,
                Some(valid_until),
                granted_by,
            )
            .await
            .unwrap();
        assert!(saved.is_delegated());
        assert_eq!(saved.valid_until, Some(valid_until));
    }

    #[tokio::test]
    async fn edge_assign_role_with_past_valid_until_is_rejected() {
        let target = Uuid::new_v4();
        let granted_by = Uuid::new_v4();

        let mock_user = MockUserRepo::new();
        let mock_role = MockUserRoleRepo::new();

        let uc = make_uc_with_mocks(mock_user, mock_role);
        let valid_until = Utc::now() - chrono::Duration::seconds(1);
        let err = uc
            .assign_role(target, UserRole::Owner, None, Some(valid_until), granted_by)
            .await
            .unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }

    // ---- @security ------------------------------------------------------

    #[tokio::test]
    async fn security_assign_high_privilege_role_to_self_is_rejected() {
        // INV: a user cannot self-grant Syndic / SuperAdmin (cohérent §3.2 §3.4).
        let same = Uuid::new_v4();

        let mock_user = MockUserRepo::new();
        let mock_role = MockUserRoleRepo::new();

        let uc = make_uc_with_mocks(mock_user, mock_role);
        let err = uc
            .assign_role(same, UserRole::Syndic, None, None, same)
            .await
            .unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }

    #[tokio::test]
    async fn security_assign_non_privileged_self_role_is_allowed() {
        // Self-assign d'un rôle non-privilégié (community.moderator, …) reste
        // OK — pas de loophole, c'est juste qu'on ne traite pas tous les
        // rôles comme égaux en risque.
        let same = Uuid::new_v4();
        let user = make_user(same);

        let mut mock_user = MockUserRepo::new();
        let u = user.clone();
        mock_user
            .expect_find_by_id()
            .returning(move |_| Ok(Some(u.clone())));

        let mut mock_role = MockUserRoleRepo::new();
        mock_role.expect_list_for_user().returning(|_| Ok(vec![]));
        mock_role
            .expect_create()
            .returning(|a: &UserRoleAssignment| Ok(a.clone()));

        let uc = make_uc_with_mocks(mock_user, mock_role);
        let saved = uc
            .assign_role(same, UserRole::CommunityModerator, None, None, same)
            .await
            .unwrap();
        assert_eq!(saved.role, UserRole::CommunityModerator);
    }

    // ---- @negative ------------------------------------------------------

    #[tokio::test]
    async fn negative_assign_role_to_unknown_user_returns_not_found() {
        let target = Uuid::new_v4();
        let granted_by = Uuid::new_v4();

        let mut mock_user = MockUserRepo::new();
        mock_user.expect_find_by_id().returning(|_| Ok(None));

        let mock_role = MockUserRoleRepo::new();

        let uc = make_uc_with_mocks(mock_user, mock_role);
        let err = uc
            .assign_role(target, UserRole::Owner, None, None, granted_by)
            .await
            .unwrap_err();
        assert!(matches!(err, AppError::NotFound(_)));
    }

    #[tokio::test]
    async fn negative_assign_role_double_returns_conflict() {
        let target = Uuid::new_v4();
        let granted_by = Uuid::new_v4();
        let user = make_user(target);
        // Pre-existing native assignment on the same (role, org).
        let existing = UserRoleAssignment::new(target, UserRole::Owner, None, true);

        let mut mock_user = MockUserRepo::new();
        let u = user.clone();
        mock_user
            .expect_find_by_id()
            .returning(move |_| Ok(Some(u.clone())));

        let mut mock_role = MockUserRoleRepo::new();
        let row = existing.clone();
        mock_role
            .expect_list_for_user()
            .returning(move |_| Ok(vec![row.clone()]));

        let uc = make_uc_with_mocks(mock_user, mock_role);
        let err = uc
            .assign_role(target, UserRole::Owner, None, None, granted_by)
            .await
            .unwrap_err();
        assert!(
            matches!(err, AppError::RoleAlreadyAssigned { .. }),
            "expected RoleAlreadyAssigned, got {:?}",
            err
        );
    }

    #[tokio::test]
    async fn negative_revoke_unknown_assignment_returns_not_found() {
        let mock_user = MockUserRepo::new();
        let mut mock_role = MockUserRoleRepo::new();
        mock_role.expect_find_by_id().returning(|_| Ok(None));

        let uc = make_uc_with_mocks(mock_user, mock_role);
        let err = uc.revoke_assignment(Uuid::new_v4()).await.unwrap_err();
        assert!(matches!(err, AppError::NotFound(_)));
    }
}
