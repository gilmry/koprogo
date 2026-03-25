use crate::application::dto::{
    Claims, LoginRequest, LoginResponse, RefreshTokenRequest, RegisterRequest, UserResponse,
    UserRoleSummary,
};
use crate::application::ports::{RefreshTokenRepository, UserRepository, UserRoleRepository};
use crate::domain::entities::{RefreshToken, User, UserRole, UserRoleAssignment};
use crate::infrastructure::audit::{log_audit_event, AuditEventType};
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::Utc;
use jsonwebtoken::{encode, EncodingKey, Header};
use std::sync::Arc;
use uuid::Uuid;

pub struct AuthUseCases {
    user_repo: Arc<dyn UserRepository>,
    refresh_token_repo: Arc<dyn RefreshTokenRepository>,
    user_role_repo: Arc<dyn UserRoleRepository>,
    jwt_secret: String,
}

impl AuthUseCases {
    pub fn new(
        user_repo: Arc<dyn UserRepository>,
        refresh_token_repo: Arc<dyn RefreshTokenRepository>,
        user_role_repo: Arc<dyn UserRoleRepository>,
        jwt_secret: String,
    ) -> Self {
        Self {
            user_repo,
            refresh_token_repo,
            user_role_repo,
            jwt_secret,
        }
    }

    pub async fn login(&self, request: LoginRequest) -> Result<LoginResponse, String> {
        let user = self
            .user_repo
            .find_by_email(&request.email)
            .await?
            .ok_or_else(|| {
                // Audit failed login attempt
                tokio::spawn(async move {
                    log_audit_event(
                        AuditEventType::AuthenticationFailed,
                        None,
                        None,
                        Some(format!("Failed login attempt for email: {}", request.email)),
                        None,
                    )
                    .await;
                });
                "Invalid email or password".to_string()
            })?;

        if !user.is_active {
            // Audit login attempt on deactivated account
            let user_id = user.id;
            tokio::spawn(async move {
                log_audit_event(
                    AuditEventType::AuthenticationFailed,
                    Some(user_id),
                    None,
                    Some("Login attempt on deactivated account".to_string()),
                    None,
                )
                .await;
            });
            return Err("User account is deactivated".to_string());
        }

        let is_valid = verify(&request.password, &user.password_hash)
            .map_err(|e| format!("Password verification failed: {}", e))?;

        if !is_valid {
            // Audit failed password verification
            let user_id = user.id;
            tokio::spawn(async move {
                log_audit_event(
                    AuditEventType::AuthenticationFailed,
                    Some(user_id),
                    None,
                    Some("Invalid password".to_string()),
                    None,
                )
                .await;
            });
            return Err("Invalid email or password".to_string());
        }

        let (roles, active_role) = self.ensure_role_assignments(&user).await?;
        let user_for_token = self.apply_active_role_metadata(&user, &active_role).await?;
        let token = self.generate_token(&user_for_token, &active_role)?;

        let refresh_token_string = self.generate_refresh_token_string(&user_for_token);
        let refresh_token = RefreshToken::new(user_for_token.id, refresh_token_string.clone());
        self.refresh_token_repo.create(&refresh_token).await?;

        // Audit successful login
        let user_id = user_for_token.id;
        let organization_id = active_role.organization_id;
        tokio::spawn(async move {
            log_audit_event(
                AuditEventType::UserLogin,
                Some(user_id),
                organization_id,
                Some("Successful login".to_string()),
                None,
            )
            .await;
        });

        Ok(LoginResponse {
            token,
            refresh_token: refresh_token_string,
            user: self.build_user_response(&user_for_token, &roles, &active_role),
        })
    }

    pub async fn register(&self, request: RegisterRequest) -> Result<LoginResponse, String> {
        if (self.user_repo.find_by_email(&request.email).await?).is_some() {
            return Err("Email already exists".to_string());
        }

        let role: UserRole = request
            .role
            .parse()
            .map_err(|e| format!("Invalid role: {}", e))?;

        let password_hash = hash(&request.password, DEFAULT_COST)
            .map_err(|e| format!("Failed to hash password: {}", e))?;

        let user = User::new(
            request.email,
            password_hash,
            request.first_name,
            request.last_name,
            role.clone(),
            request.organization_id,
        )?;

        let created_user = self.user_repo.create(&user).await?;

        // Create primary role assignment
        let primary_assignment = self
            .user_role_repo
            .create(&UserRoleAssignment::new(
                created_user.id,
                role,
                created_user.organization_id,
                true,
            ))
            .await?;
        let roles = vec![primary_assignment.clone()];
        let user_for_token = self
            .apply_active_role_metadata(&created_user, &primary_assignment)
            .await?;

        let token = self.generate_token(&user_for_token, &primary_assignment)?;
        let refresh_token_string = self.generate_refresh_token_string(&user_for_token);
        let refresh_token = RefreshToken::new(user_for_token.id, refresh_token_string.clone());
        self.refresh_token_repo.create(&refresh_token).await?;

        // Audit successful registration
        let user_id = created_user.id;
        let organization_id = created_user.organization_id;
        let email = created_user.email.clone();
        tokio::spawn(async move {
            log_audit_event(
                AuditEventType::UserRegistration,
                Some(user_id),
                organization_id,
                Some(format!("New user registered: {}", email)),
                None,
            )
            .await;
        });

        Ok(LoginResponse {
            token,
            refresh_token: refresh_token_string,
            user: self.build_user_response(&user_for_token, &roles, &primary_assignment),
        })
    }

    pub async fn switch_active_role(
        &self,
        user_id: Uuid,
        role_id: Uuid,
    ) -> Result<LoginResponse, String> {
        let user = self
            .user_repo
            .find_by_id(user_id)
            .await?
            .ok_or("User not found")?;

        if !user.is_active {
            return Err("User account is deactivated".to_string());
        }

        let target_role = self
            .user_role_repo
            .find_by_id(role_id)
            .await?
            .ok_or("Role assignment not found")?;

        if target_role.user_id != user.id {
            return Err("Role assignment does not belong to user".to_string());
        }

        let updated_primary = self
            .user_role_repo
            .set_primary_role(user.id, role_id)
            .await?;

        let roles = self.user_role_repo.list_for_user(user.id).await?;
        let active_role = roles
            .iter()
            .find(|assignment| assignment.is_primary)
            .cloned()
            .unwrap_or(updated_primary.clone());

        let updated_user = self.apply_active_role_metadata(&user, &active_role).await?;

        let token = self.generate_token(&updated_user, &active_role)?;
        let refresh_token_string = self.generate_refresh_token_string(&updated_user);
        let refresh_token = RefreshToken::new(updated_user.id, refresh_token_string.clone());
        self.refresh_token_repo.create(&refresh_token).await?;

        Ok(LoginResponse {
            token,
            refresh_token: refresh_token_string,
            user: self.build_user_response(&updated_user, &roles, &active_role),
        })
    }

    pub async fn get_user_by_id(&self, user_id: uuid::Uuid) -> Result<UserResponse, String> {
        let user = self
            .user_repo
            .find_by_id(user_id)
            .await?
            .ok_or("User not found")?;

        let (roles, active_role) = self.ensure_role_assignments(&user).await?;
        Ok(self.build_user_response(&user, &roles, &active_role))
    }

    pub fn verify_token(&self, token: &str) -> Result<Claims, String> {
        use jsonwebtoken::{decode, DecodingKey, Validation};

        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.jwt_secret.as_bytes()),
            &Validation::default(),
        )
        .map_err(|e| format!("Invalid token: {}", e))?;

        Ok(token_data.claims)
    }

    pub async fn refresh_token(
        &self,
        request: RefreshTokenRequest,
    ) -> Result<LoginResponse, String> {
        let refresh_token = self
            .refresh_token_repo
            .find_by_token(&request.refresh_token)
            .await?
            .ok_or_else(|| {
                // Audit invalid refresh token attempt
                tokio::spawn(async {
                    log_audit_event(
                        AuditEventType::InvalidToken,
                        None,
                        None,
                        Some("Invalid refresh token attempted".to_string()),
                        None,
                    )
                    .await;
                });
                "Invalid refresh token".to_string()
            })?;

        if !refresh_token.is_valid() {
            // Audit expired/revoked token attempt
            let user_id = refresh_token.user_id;
            let reason = if refresh_token.is_expired() {
                "Expired refresh token"
            } else {
                "Revoked refresh token"
            };
            tokio::spawn(async move {
                log_audit_event(
                    AuditEventType::InvalidToken,
                    Some(user_id),
                    None,
                    Some(format!("{} attempted", reason)),
                    None,
                )
                .await;
            });
            return Err("Refresh token expired or revoked".to_string());
        }

        let user = self
            .user_repo
            .find_by_id(refresh_token.user_id)
            .await?
            .ok_or("User not found")?;

        if !user.is_active {
            // Audit refresh attempt on deactivated account
            let user_id = user.id;
            tokio::spawn(async move {
                log_audit_event(
                    AuditEventType::AuthenticationFailed,
                    Some(user_id),
                    None,
                    Some("Refresh token attempt on deactivated account".to_string()),
                    None,
                )
                .await;
            });
            return Err("User account is deactivated".to_string());
        }

        let (roles, active_role) = self.ensure_role_assignments(&user).await?;
        let user_for_token = self.apply_active_role_metadata(&user, &active_role).await?;
        let token = self.generate_token(&user_for_token, &active_role)?;

        // Revoke old token (refresh token rotation)
        self.refresh_token_repo
            .revoke(&request.refresh_token)
            .await?;

        let new_refresh_token_string = self.generate_refresh_token_string(&user_for_token);
        let new_refresh_token =
            RefreshToken::new(user_for_token.id, new_refresh_token_string.clone());
        self.refresh_token_repo.create(&new_refresh_token).await?;

        // Audit successful token refresh
        let user_id = user_for_token.id;
        let organization_id = active_role.organization_id;
        tokio::spawn(async move {
            log_audit_event(
                AuditEventType::TokenRefresh,
                Some(user_id),
                organization_id,
                Some("Refresh token successfully exchanged".to_string()),
                None,
            )
            .await;
        });

        Ok(LoginResponse {
            token,
            refresh_token: new_refresh_token_string,
            user: self.build_user_response(&user_for_token, &roles, &active_role),
        })
    }

    pub async fn revoke_all_refresh_tokens(&self, user_id: Uuid) -> Result<u64, String> {
        self.refresh_token_repo.revoke_all_for_user(user_id).await
    }

    fn build_user_response(
        &self,
        user: &User,
        roles: &[UserRoleAssignment],
        active_role: &UserRoleAssignment,
    ) -> UserResponse {
        UserResponse {
            id: user.id,
            email: user.email.clone(),
            first_name: user.first_name.clone(),
            last_name: user.last_name.clone(),
            role: active_role.role.to_string(),
            organization_id: active_role.organization_id,
            is_active: user.is_active,
            roles: roles.iter().map(Self::summarize_role).collect(),
            active_role: Some(Self::summarize_role(active_role)),
        }
    }

    async fn ensure_role_assignments(
        &self,
        user: &User,
    ) -> Result<(Vec<UserRoleAssignment>, UserRoleAssignment), String> {
        let mut assignments = self.user_role_repo.list_for_user(user.id).await?;

        if assignments.is_empty() {
            let assignment = self
                .user_role_repo
                .create(&UserRoleAssignment::new(
                    user.id,
                    user.role.clone(),
                    user.organization_id,
                    true,
                ))
                .await?;
            assignments.push(assignment.clone());
        }

        if !assignments.iter().any(|assignment| assignment.is_primary) {
            let first = assignments[0].id;
            self.user_role_repo.set_primary_role(user.id, first).await?;
            assignments = self.user_role_repo.list_for_user(user.id).await?;
        }

        let active = assignments
            .iter()
            .find(|assignment| assignment.is_primary)
            .cloned()
            .unwrap_or_else(|| assignments[0].clone());

        Ok((assignments, active))
    }

    async fn apply_active_role_metadata(
        &self,
        user: &User,
        active_role: &UserRoleAssignment,
    ) -> Result<User, String> {
        let mut updated_user = user.clone();
        let mut requires_update = false;

        if updated_user.role != active_role.role {
            updated_user.role = active_role.role.clone();
            requires_update = true;
        }

        if updated_user.organization_id != active_role.organization_id {
            updated_user.organization_id = active_role.organization_id;
            requires_update = true;
        }

        if requires_update {
            updated_user.updated_at = Utc::now();
            return self.user_repo.update(&updated_user).await;
        }

        Ok(updated_user)
    }

    fn summarize_role(assignment: &UserRoleAssignment) -> UserRoleSummary {
        UserRoleSummary {
            id: assignment.id,
            role: assignment.role.to_string(),
            organization_id: assignment.organization_id,
            is_primary: assignment.is_primary,
        }
    }

    fn generate_token(
        &self,
        user: &User,
        active_role: &UserRoleAssignment,
    ) -> Result<String, String> {
        let now = Utc::now().timestamp();
        let expiration = now + (15 * 60);

        let claims = Claims {
            sub: user.id.to_string(),
            email: user.email.clone(),
            role: active_role.role.to_string(),
            organization_id: active_role.organization_id,
            role_id: Some(active_role.id),
            exp: expiration,
            iat: now,
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.jwt_secret.as_bytes()),
        )
        .map_err(|e| format!("Failed to generate token: {}", e))
    }

    fn generate_refresh_token_string(&self, user: &User) -> String {
        let now = Utc::now().timestamp();
        format!("{}:{}:{}", user.id, now, uuid::Uuid::new_v4())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::ports::{RefreshTokenRepository, UserRepository, UserRoleRepository};
    use crate::domain::entities::{RefreshToken, User, UserRole, UserRoleAssignment};
    use async_trait::async_trait;
    use mockall::mock;
    use std::sync::Arc;

    // Re-use the existing MockUserRepo from the user_repository port
    use crate::application::ports::user_repository::MockUserRepo;

    // Create mock for RefreshTokenRepository (not defined elsewhere)
    mock! {
        pub RefreshTokenRepo {}

        #[async_trait]
        impl RefreshTokenRepository for RefreshTokenRepo {
            async fn create(&self, refresh_token: &RefreshToken) -> Result<RefreshToken, String>;
            async fn find_by_token(&self, token: &str) -> Result<Option<RefreshToken>, String>;
            async fn find_by_user_id(&self, user_id: Uuid) -> Result<Vec<RefreshToken>, String>;
            async fn revoke(&self, token: &str) -> Result<bool, String>;
            async fn revoke_all_for_user(&self, user_id: Uuid) -> Result<u64, String>;
            async fn delete_expired(&self) -> Result<u64, String>;
        }
    }

    // Create mock for UserRoleRepository (not defined elsewhere)
    mock! {
        pub UserRoleRepo {}

        #[async_trait]
        impl UserRoleRepository for UserRoleRepo {
            async fn create(&self, assignment: &UserRoleAssignment) -> Result<UserRoleAssignment, String>;
            async fn list_for_user(&self, user_id: Uuid) -> Result<Vec<UserRoleAssignment>, String>;
            async fn find_by_id(&self, id: Uuid) -> Result<Option<UserRoleAssignment>, String>;
            async fn set_primary_role(&self, user_id: Uuid, role_id: Uuid) -> Result<UserRoleAssignment, String>;
        }
    }

    const TEST_JWT_SECRET: &str = "test-secret-key-that-is-long-enough-for-jwt";

    /// Helper: create a valid active User with a bcrypt-hashed password.
    fn make_user(email: &str, password: &str, org_id: Option<Uuid>) -> User {
        let hash = bcrypt::hash(password, 4).expect("bcrypt hash");
        User::new(
            email.to_string(),
            hash,
            "Test".to_string(),
            "User".to_string(),
            UserRole::Syndic,
            org_id,
        )
        .expect("valid user")
    }

    /// Helper: create a UserRoleAssignment marked as primary for a given user.
    fn make_primary_role(user_id: Uuid, org_id: Option<Uuid>) -> UserRoleAssignment {
        UserRoleAssignment::new(user_id, UserRole::Syndic, org_id, true)
    }

    /// Build an AuthUseCases from the three mocks.
    fn build_use_cases(
        user_repo: MockUserRepo,
        refresh_repo: MockRefreshTokenRepo,
        role_repo: MockUserRoleRepo,
    ) -> AuthUseCases {
        AuthUseCases::new(
            Arc::new(user_repo),
            Arc::new(refresh_repo),
            Arc::new(role_repo),
            TEST_JWT_SECRET.to_string(),
        )
    }

    // ── 1. login success ────────────────────────────────────────────────

    #[tokio::test]
    async fn test_login_success() {
        let org_id = Some(Uuid::new_v4());
        let user = make_user("login@example.com", "password123", org_id);
        let user_clone = user.clone();
        let user_for_update = user.clone();
        let role = make_primary_role(user.id, org_id);
        let role_clone = role.clone();

        let mut user_repo = MockUserRepo::new();
        user_repo
            .expect_find_by_email()
            .withf(|e| e == "login@example.com")
            .returning(move |_| Ok(Some(user_clone.clone())));
        user_repo
            .expect_update()
            .returning(move |u| Ok(u.clone()));

        let mut refresh_repo = MockRefreshTokenRepo::new();
        refresh_repo
            .expect_create()
            .returning(|rt| Ok(rt.clone()));

        let mut role_repo = MockUserRoleRepo::new();
        role_repo
            .expect_list_for_user()
            .returning(move |_| Ok(vec![role_clone.clone()]));

        let uc = build_use_cases(user_repo, refresh_repo, role_repo);

        let result = uc
            .login(LoginRequest {
                email: "login@example.com".to_string(),
                password: "password123".to_string(),
            })
            .await;

        assert!(result.is_ok(), "login should succeed: {:?}", result.err());
        let response = result.unwrap();
        assert!(!response.token.is_empty());
        assert!(!response.refresh_token.is_empty());
        assert_eq!(response.user.email, "login@example.com");
        assert!(response.user.is_active);
    }

    // ── 2. login — user not found ───────────────────────────────────────

    #[tokio::test]
    async fn test_login_invalid_email() {
        let mut user_repo = MockUserRepo::new();
        user_repo
            .expect_find_by_email()
            .returning(|_| Ok(None));

        let uc = build_use_cases(user_repo, MockRefreshTokenRepo::new(), MockUserRoleRepo::new());

        let result = uc
            .login(LoginRequest {
                email: "nonexistent@example.com".to_string(),
                password: "whatever".to_string(),
            })
            .await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Invalid email or password");
    }

    // ── 3. login — wrong password ───────────────────────────────────────

    #[tokio::test]
    async fn test_login_invalid_password() {
        let user = make_user("user@example.com", "correct_password", None);
        let user_clone = user.clone();

        let mut user_repo = MockUserRepo::new();
        user_repo
            .expect_find_by_email()
            .returning(move |_| Ok(Some(user_clone.clone())));

        let uc = build_use_cases(user_repo, MockRefreshTokenRepo::new(), MockUserRoleRepo::new());

        let result = uc
            .login(LoginRequest {
                email: "user@example.com".to_string(),
                password: "wrong_password".to_string(),
            })
            .await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Invalid email or password");
    }

    // ── 4. login — deactivated account ──────────────────────────────────

    #[tokio::test]
    async fn test_login_deactivated_account() {
        let mut user = make_user("deactivated@example.com", "password123", None);
        user.deactivate();
        let user_clone = user.clone();

        let mut user_repo = MockUserRepo::new();
        user_repo
            .expect_find_by_email()
            .returning(move |_| Ok(Some(user_clone.clone())));

        let uc = build_use_cases(user_repo, MockRefreshTokenRepo::new(), MockUserRoleRepo::new());

        let result = uc
            .login(LoginRequest {
                email: "deactivated@example.com".to_string(),
                password: "password123".to_string(),
            })
            .await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "User account is deactivated");
    }

    // ── 5. register success ─────────────────────────────────────────────

    #[tokio::test]
    async fn test_register_success() {
        let mut user_repo = MockUserRepo::new();
        user_repo
            .expect_find_by_email()
            .returning(|_| Ok(None)); // email not taken
        user_repo
            .expect_create()
            .returning(|u| Ok(u.clone()));
        user_repo
            .expect_update()
            .returning(|u| Ok(u.clone()));

        let mut refresh_repo = MockRefreshTokenRepo::new();
        refresh_repo
            .expect_create()
            .returning(|rt| Ok(rt.clone()));

        let mut role_repo = MockUserRoleRepo::new();
        role_repo
            .expect_create()
            .returning(|a| Ok(a.clone()));

        let uc = build_use_cases(user_repo, refresh_repo, role_repo);

        let result = uc
            .register(RegisterRequest {
                email: "new@example.com".to_string(),
                password: "password123".to_string(),
                first_name: "Alice".to_string(),
                last_name: "Dupont".to_string(),
                role: "syndic".to_string(),
                organization_id: None,
            })
            .await;

        assert!(result.is_ok(), "register should succeed: {:?}", result.err());
        let response = result.unwrap();
        assert!(!response.token.is_empty());
        assert_eq!(response.user.email, "new@example.com");
        assert_eq!(response.user.first_name, "Alice");
    }

    // ── 6. register — duplicate email ───────────────────────────────────

    #[tokio::test]
    async fn test_register_duplicate_email() {
        let existing = make_user("taken@example.com", "pw123456", None);
        let existing_clone = existing.clone();

        let mut user_repo = MockUserRepo::new();
        user_repo
            .expect_find_by_email()
            .returning(move |_| Ok(Some(existing_clone.clone())));

        let uc = build_use_cases(user_repo, MockRefreshTokenRepo::new(), MockUserRoleRepo::new());

        let result = uc
            .register(RegisterRequest {
                email: "taken@example.com".to_string(),
                password: "password123".to_string(),
                first_name: "Bob".to_string(),
                last_name: "Martin".to_string(),
                role: "owner".to_string(),
                organization_id: None,
            })
            .await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Email already exists");
    }

    // ── 7. switch_active_role success ───────────────────────────────────

    #[tokio::test]
    async fn test_switch_role_success() {
        let org_id = Some(Uuid::new_v4());
        let user = make_user("multi@example.com", "password123", org_id);
        let user_clone = user.clone();

        // The target role that we want to switch to
        let target_role = UserRoleAssignment::new(user.id, UserRole::Accountant, org_id, false);
        let target_role_id = target_role.id;
        let target_clone = target_role.clone();

        // After switching, the target role becomes primary
        let mut switched_role = target_role.clone();
        switched_role.is_primary = true;
        let switched_clone = switched_role.clone();
        let switched_for_list = switched_role.clone();

        let mut user_repo = MockUserRepo::new();
        user_repo
            .expect_find_by_id()
            .returning(move |_| Ok(Some(user_clone.clone())));
        user_repo
            .expect_update()
            .returning(|u| Ok(u.clone()));

        let mut refresh_repo = MockRefreshTokenRepo::new();
        refresh_repo
            .expect_create()
            .returning(|rt| Ok(rt.clone()));

        let mut role_repo = MockUserRoleRepo::new();
        role_repo
            .expect_find_by_id()
            .returning(move |_| Ok(Some(target_clone.clone())));
        role_repo
            .expect_set_primary_role()
            .returning(move |_, _| Ok(switched_clone.clone()));
        role_repo
            .expect_list_for_user()
            .returning(move |_| Ok(vec![switched_for_list.clone()]));

        let uc = build_use_cases(user_repo, refresh_repo, role_repo);

        let result = uc.switch_active_role(user.id, target_role_id).await;

        assert!(result.is_ok(), "switch_role should succeed: {:?}", result.err());
        let response = result.unwrap();
        assert!(!response.token.is_empty());
        assert_eq!(response.user.role, "accountant");
    }

    // ── 8. switch_active_role — role not found ──────────────────────────

    #[tokio::test]
    async fn test_switch_role_not_found() {
        let user = make_user("user@example.com", "password123", None);
        let user_clone = user.clone();

        let mut user_repo = MockUserRepo::new();
        user_repo
            .expect_find_by_id()
            .returning(move |_| Ok(Some(user_clone.clone())));

        let mut role_repo = MockUserRoleRepo::new();
        role_repo
            .expect_find_by_id()
            .returning(|_| Ok(None)); // role not found

        let uc = build_use_cases(user_repo, MockRefreshTokenRepo::new(), role_repo);

        let result = uc.switch_active_role(user.id, Uuid::new_v4()).await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Role assignment not found");
    }

    // ── 9. verify_token — valid token ───────────────────────────────────

    #[tokio::test]
    async fn test_verify_token_valid() {
        let org_id = Some(Uuid::new_v4());
        let user = make_user("verify@example.com", "password123", org_id);
        let role = make_primary_role(user.id, org_id);

        let uc = build_use_cases(
            MockUserRepo::new(),
            MockRefreshTokenRepo::new(),
            MockUserRoleRepo::new(),
        );

        // Generate a token using the same secret
        let token = uc.generate_token(&user, &role).expect("token generation");

        let claims = uc.verify_token(&token);
        assert!(claims.is_ok(), "verify should succeed: {:?}", claims.err());
        let claims = claims.unwrap();
        assert_eq!(claims.sub, user.id.to_string());
        assert_eq!(claims.email, "verify@example.com");
        assert_eq!(claims.role, "syndic");
        assert_eq!(claims.organization_id, org_id);
    }

    // ── 10. verify_token — invalid token ────────────────────────────────

    #[tokio::test]
    async fn test_verify_token_invalid() {
        let uc = build_use_cases(
            MockUserRepo::new(),
            MockRefreshTokenRepo::new(),
            MockUserRoleRepo::new(),
        );

        let result = uc.verify_token("this.is.not.a.valid.jwt");
        assert!(result.is_err());
        assert!(result.unwrap_err().starts_with("Invalid token:"));
    }

    // ── 11. revoke_all_refresh_tokens ───────────────────────────────────

    #[tokio::test]
    async fn test_revoke_all_refresh_tokens() {
        let user_id = Uuid::new_v4();

        let mut refresh_repo = MockRefreshTokenRepo::new();
        refresh_repo
            .expect_revoke_all_for_user()
            .withf(move |id| *id == user_id)
            .returning(|_| Ok(3));

        let uc = build_use_cases(MockUserRepo::new(), refresh_repo, MockUserRoleRepo::new());

        let result = uc.revoke_all_refresh_tokens(user_id).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 3);
    }
}
