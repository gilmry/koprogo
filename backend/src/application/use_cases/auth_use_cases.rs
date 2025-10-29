use crate::application::dto::{
    Claims, LoginRequest, LoginResponse, RefreshTokenRequest, RegisterRequest, UserResponse,
    UserRoleSummary,
};
use crate::application::ports::{RefreshTokenRepository, UserRepository, UserRoleRepository};
use crate::domain::entities::{RefreshToken, User, UserRole, UserRoleAssignment};
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
            .ok_or("Invalid email or password")?;

        if !user.is_active {
            return Err("User account is deactivated".to_string());
        }

        let is_valid = verify(&request.password, &user.password_hash)
            .map_err(|e| format!("Password verification failed: {}", e))?;

        if !is_valid {
            return Err("Invalid email or password".to_string());
        }

        let (roles, active_role) = self.ensure_role_assignments(&user).await?;
        let user_for_token = self.apply_active_role_metadata(&user, &active_role).await?;
        let token = self.generate_token(&user_for_token, &active_role)?;

        let refresh_token_string = self.generate_refresh_token_string(&user_for_token);
        let refresh_token = RefreshToken::new(user_for_token.id, refresh_token_string.clone());
        self.refresh_token_repo.create(&refresh_token).await?;

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
            .ok_or("Invalid refresh token")?;

        if !refresh_token.is_valid() {
            return Err("Refresh token expired or revoked".to_string());
        }

        let user = self
            .user_repo
            .find_by_id(refresh_token.user_id)
            .await?
            .ok_or("User not found")?;

        if !user.is_active {
            return Err("User account is deactivated".to_string());
        }

        let (roles, active_role) = self.ensure_role_assignments(&user).await?;
        let user_for_token = self.apply_active_role_metadata(&user, &active_role).await?;
        let token = self.generate_token(&user_for_token, &active_role)?;

        self.refresh_token_repo
            .revoke(&request.refresh_token)
            .await?;

        let new_refresh_token_string = self.generate_refresh_token_string(&user_for_token);
        let new_refresh_token =
            RefreshToken::new(user_for_token.id, new_refresh_token_string.clone());
        self.refresh_token_repo.create(&new_refresh_token).await?;

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
