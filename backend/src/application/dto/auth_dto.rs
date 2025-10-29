use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(email(message = "Email must be valid"))]
    pub email: String,

    #[validate(length(min = 6, message = "Password must be at least 6 characters"))]
    pub password: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(email(message = "Email must be valid"))]
    pub email: String,

    #[validate(length(min = 6, message = "Password must be at least 6 characters"))]
    pub password: String,

    #[validate(length(min = 2, message = "First name must be at least 2 characters"))]
    pub first_name: String,

    #[validate(length(min = 2, message = "Last name must be at least 2 characters"))]
    pub last_name: String,

    pub role: String,

    pub organization_id: Option<uuid::Uuid>,
}

#[derive(Debug, Serialize, Clone)]
pub struct LoginResponse {
    pub token: String,
    pub refresh_token: String,
    pub user: UserResponse,
}

#[derive(Debug, Deserialize)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserRoleSummary {
    pub id: uuid::Uuid,
    pub role: String,
    pub organization_id: Option<uuid::Uuid>,
    pub is_primary: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserResponse {
    pub id: uuid::Uuid,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub role: String,
    pub organization_id: Option<uuid::Uuid>,
    pub is_active: bool,
    pub roles: Vec<UserRoleSummary>,
    pub active_role: Option<UserRoleSummary>,
}

#[derive(Debug, Deserialize)]
pub struct SwitchRoleRequest {
    pub role_id: uuid::Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // user id
    pub email: String,
    pub role: String,
    pub organization_id: Option<uuid::Uuid>,
    pub role_id: Option<uuid::Uuid>,
    pub exp: i64, // expiration timestamp
    pub iat: i64, // issued at
}
