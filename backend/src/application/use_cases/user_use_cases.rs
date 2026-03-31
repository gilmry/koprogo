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
}
