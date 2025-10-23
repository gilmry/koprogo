use crate::application::dto::{Claims, LoginRequest, LoginResponse, RegisterRequest, UserResponse};
use crate::application::ports::UserRepository;
use crate::domain::entities::{User, UserRole};
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::Utc;
use jsonwebtoken::{encode, EncodingKey, Header};
use std::sync::Arc;

pub struct AuthUseCases {
    user_repo: Arc<dyn UserRepository>,
    jwt_secret: String,
}

impl AuthUseCases {
    pub fn new(user_repo: Arc<dyn UserRepository>, jwt_secret: String) -> Self {
        Self {
            user_repo,
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

        // Generate JWT token
        let token = self.generate_token(&user)?;

        Ok(LoginResponse {
            token,
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

        // Generate JWT token
        let token = self.generate_token(&created_user)?;

        Ok(LoginResponse {
            token,
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

    fn generate_token(&self, user: &User) -> Result<String, String> {
        let now = Utc::now().timestamp();
        let expiration = now + (24 * 3600); // 24 hours

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
}
