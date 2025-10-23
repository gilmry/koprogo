use crate::application::dto::{Claims, LoginRequest, LoginResponse, RefreshTokenRequest, RegisterRequest, UserResponse};
use crate::application::ports::{RefreshTokenRepository, UserRepository};
use crate::domain::entities::{RefreshToken, User, UserRole};
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::Utc;
use jsonwebtoken::{encode, EncodingKey, Header};
use std::sync::Arc;
use uuid::Uuid;

pub struct AuthUseCases {
    user_repo: Arc<dyn UserRepository>,
    refresh_token_repo: Arc<dyn RefreshTokenRepository>,
    jwt_secret: String,
}

impl AuthUseCases {
    pub fn new(
        user_repo: Arc<dyn UserRepository>,
        refresh_token_repo: Arc<dyn RefreshTokenRepository>,
        jwt_secret: String,
    ) -> Self {
        Self {
            user_repo,
            refresh_token_repo,
            jwt_secret,
        }
    }

    pub async fn login(&self, request: LoginRequest) -> Result<LoginResponse, String> {
        // Find user by email
        let user = self
            .user_repo
            .find_by_email(&request.email)
            .await?
            .ok_or("Invalid email or password")?;

        // Check if user is active
        if !user.is_active {
            return Err("User account is deactivated".to_string());
        }

        // Verify password
        let is_valid = verify(&request.password, &user.password_hash)
            .map_err(|e| format!("Password verification failed: {}", e))?;

        if !is_valid {
            return Err("Invalid email or password".to_string());
        }

        // Generate JWT access token (15 minutes)
        let token = self.generate_token(&user)?;

        // Generate refresh token (7 days)
        let refresh_token_string = self.generate_refresh_token_string(&user);
        let refresh_token = RefreshToken::new(user.id, refresh_token_string.clone());
        self.refresh_token_repo.create(&refresh_token).await?;

        Ok(LoginResponse {
            token,
            refresh_token: refresh_token_string,
            user: UserResponse {
                id: user.id,
                email: user.email,
                first_name: user.first_name,
                last_name: user.last_name,
                role: user.role.to_string(),
                organization_id: user.organization_id,
                is_active: user.is_active,
            },
        })
    }

    pub async fn register(&self, request: RegisterRequest) -> Result<LoginResponse, String> {
        // Check if email already exists
        if (self.user_repo.find_by_email(&request.email).await?).is_some() {
            return Err("Email already exists".to_string());
        }

        // Parse role
        let role: UserRole = request
            .role
            .parse()
            .map_err(|e| format!("Invalid role: {}", e))?;

        // Hash password
        let password_hash = hash(&request.password, DEFAULT_COST)
            .map_err(|e| format!("Failed to hash password: {}", e))?;

        // Create user
        let user = User::new(
            request.email,
            password_hash,
            request.first_name,
            request.last_name,
            role,
            request.organization_id,
        )?;

        // Save user
        let created_user = self.user_repo.create(&user).await?;

        // Generate JWT access token (15 minutes)
        let token = self.generate_token(&created_user)?;

        // Generate refresh token (7 days)
        let refresh_token_string = self.generate_refresh_token_string(&created_user);
        let refresh_token = RefreshToken::new(created_user.id, refresh_token_string.clone());
        self.refresh_token_repo.create(&refresh_token).await?;

        Ok(LoginResponse {
            token,
            refresh_token: refresh_token_string,
            user: UserResponse {
                id: created_user.id,
                email: created_user.email,
                first_name: created_user.first_name,
                last_name: created_user.last_name,
                role: created_user.role.to_string(),
                organization_id: created_user.organization_id,
                is_active: created_user.is_active,
            },
        })
    }

    pub async fn get_user_by_id(&self, user_id: uuid::Uuid) -> Result<UserResponse, String> {
        let user = self
            .user_repo
            .find_by_id(user_id)
            .await?
            .ok_or("User not found")?;

        Ok(UserResponse {
            id: user.id,
            email: user.email,
            first_name: user.first_name,
            last_name: user.last_name,
            role: user.role.to_string(),
            organization_id: user.organization_id,
            is_active: user.is_active,
        })
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

    /// Refresh access token using a refresh token
    pub async fn refresh_token(&self, request: RefreshTokenRequest) -> Result<LoginResponse, String> {
        // Find refresh token in database
        let refresh_token = self
            .refresh_token_repo
            .find_by_token(&request.refresh_token)
            .await?
            .ok_or("Invalid refresh token")?;

        // Check if token is valid
        if !refresh_token.is_valid() {
            return Err("Refresh token expired or revoked".to_string());
        }

        // Get user
        let user = self
            .user_repo
            .find_by_id(refresh_token.user_id)
            .await?
            .ok_or("User not found")?;

        // Check if user is active
        if !user.is_active {
            return Err("User account is deactivated".to_string());
        }

        // Generate new access token (15 minutes)
        let token = self.generate_token(&user)?;

        // Generate new refresh token (7 days) and revoke old one
        self.refresh_token_repo
            .revoke(&request.refresh_token)
            .await?;

        let new_refresh_token_string = self.generate_refresh_token_string(&user);
        let new_refresh_token = RefreshToken::new(user.id, new_refresh_token_string.clone());
        self.refresh_token_repo.create(&new_refresh_token).await?;

        Ok(LoginResponse {
            token,
            refresh_token: new_refresh_token_string,
            user: UserResponse {
                id: user.id,
                email: user.email.clone(),
                first_name: user.first_name.clone(),
                last_name: user.last_name.clone(),
                role: user.role.to_string(),
                organization_id: user.organization_id,
                is_active: user.is_active,
            },
        })
    }

    /// Revoke all refresh tokens for a user (logout from all devices)
    pub async fn revoke_all_refresh_tokens(&self, user_id: Uuid) -> Result<u64, String> {
        self.refresh_token_repo.revoke_all_for_user(user_id).await
    }

    fn generate_token(&self, user: &User) -> Result<String, String> {
        let now = Utc::now().timestamp();
        let expiration = now + (15 * 60); // 15 minutes

        let claims = Claims {
            sub: user.id.to_string(),
            email: user.email.clone(),
            role: user.role.to_string(),
            organization_id: user.organization_id,
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
        let random_suffix = Uuid::new_v4();
        format!("{}:{}:{}", user.id, now, random_suffix)
    }
}
