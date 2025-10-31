use crate::domain::entities::UnitType;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Deserialize, Validate, Clone)]
pub struct CreateUnitDto {
    pub organization_id: String,
    pub building_id: String,

    #[validate(length(min = 1))]
    pub unit_number: String,

    pub unit_type: UnitType,
    pub floor: Option<i32>,

    #[validate(range(min = 0.1))]
    pub surface_area: f64,

    #[validate(range(min = 0.1, max = 1000.0))]
    pub quota: f64,
}

#[derive(Debug, Deserialize, Validate, Clone)]
pub struct UpdateUnitDto {
    #[validate(length(min = 1))]
    pub unit_number: String,

    pub unit_type: UnitType,
    pub floor: i32,

    #[validate(range(min = 0.1))]
    pub surface_area: f64,

    #[validate(range(min = 0.1, max = 1000.0))]
    pub quota: f64,
}

#[derive(Debug, Serialize)]
pub struct UnitResponseDto {
    pub id: String,
    pub building_id: String,
    pub unit_number: String,
    pub unit_type: UnitType,
    pub floor: Option<i32>,
    pub surface_area: f64,
    pub quota: f64,
    pub owner_id: Option<String>,
}
