use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Deserialize, Validate, Clone)]
pub struct CreateBuildingDto {
    #[validate(length(min = 1, message = "Name cannot be empty"))]
    pub name: String,

    #[validate(length(min = 1))]
    pub address: String,

    #[validate(length(min = 1))]
    pub city: String,

    #[validate(length(min = 1))]
    pub postal_code: String,

    #[validate(length(min = 1))]
    pub country: String,

    #[validate(range(min = 1, message = "Total units must be greater than 0"))]
    pub total_units: i32,

    pub construction_year: Option<i32>,
}

#[derive(Debug, Deserialize, Validate, Clone)]
pub struct UpdateBuildingDto {
    #[validate(length(min = 1))]
    pub name: String,

    #[validate(length(min = 1))]
    pub address: String,

    #[validate(length(min = 1))]
    pub city: String,

    #[validate(length(min = 1))]
    pub postal_code: String,
}

#[derive(Debug, Serialize)]
pub struct BuildingResponseDto {
    pub id: String,
    pub name: String,
    pub address: String,
    pub city: String,
    pub postal_code: String,
    pub country: String,
    pub total_units: i32,
    pub construction_year: Option<i32>,
    pub created_at: String,
    pub updated_at: String,
}
