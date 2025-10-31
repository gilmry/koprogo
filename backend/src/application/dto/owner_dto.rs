use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Deserialize, Validate, Clone)]
pub struct CreateOwnerDto {
    #[serde(default)]
    pub organization_id: String,

    #[validate(length(min = 1))]
    pub first_name: String,

    #[validate(length(min = 1))]
    pub last_name: String,

    #[validate(email)]
    pub email: String,

    pub phone: Option<String>,

    #[validate(length(min = 1))]
    pub address: String,

    #[validate(length(min = 1))]
    pub city: String,

    #[validate(length(min = 1))]
    pub postal_code: String,

    #[validate(length(min = 1))]
    pub country: String,
}

#[derive(Debug, Serialize)]
pub struct OwnerResponseDto {
    pub id: String,
    pub organization_id: String,
    pub user_id: Option<String>, // Link to User account
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub phone: Option<String>,
    pub address: String,
    pub city: String,
    pub postal_code: String,
    pub country: String,
}
