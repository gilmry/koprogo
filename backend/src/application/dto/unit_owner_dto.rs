use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use validator::Validate;

/// DTO for adding an owner to a unit
#[derive(Debug, Deserialize, Validate)]
pub struct AddOwnerToUnitDto {
    #[validate(length(min = 1))]
    pub owner_id: String,

    #[validate(range(min = 0.0, max = 1.0))]
    pub ownership_percentage: f64,

    pub is_primary_contact: bool,
}

/// DTO for updating ownership details
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateOwnershipDto {
    #[validate(range(min = 0.0, max = 1.0))]
    pub ownership_percentage: Option<f64>,

    pub is_primary_contact: Option<bool>,
}

/// Response DTO for a unit-owner relationship
#[derive(Debug, Serialize, Clone)]
pub struct UnitOwnerResponseDto {
    pub id: String,
    pub unit_id: String,
    pub owner_id: String,
    pub ownership_percentage: f64,
    pub start_date: DateTime<Utc>,
    pub end_date: Option<DateTime<Utc>>,
    pub is_primary_contact: bool,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Response DTO for a unit with its owners
#[derive(Debug, Serialize)]
pub struct UnitWithOwnersDto {
    pub unit_id: String,
    pub unit_number: String,
    pub floor: Option<i32>,
    pub area: Option<f64>,
    pub owners: Vec<UnitOwnerWithDetailsDto>,
    pub total_ownership_percentage: f64,
}

/// Response DTO for an owner with their units
#[derive(Debug, Serialize)]
pub struct OwnerWithUnitsDto {
    pub owner_id: String,
    pub owner_name: String,
    pub owner_email: String,
    pub units: Vec<UnitOwnerWithDetailsDto>,
}

/// Detailed unit-owner relationship with entity details
#[derive(Debug, Serialize, Clone)]
pub struct UnitOwnerWithDetailsDto {
    pub relationship_id: String,
    pub ownership_percentage: f64,
    pub is_primary_contact: bool,
    pub start_date: DateTime<Utc>,
    pub end_date: Option<DateTime<Utc>>,
    pub is_active: bool,

    // Unit details (when viewing from owner perspective)
    pub unit_id: Option<String>,
    pub unit_number: Option<String>,
    pub floor: Option<i32>,
    pub area: Option<f64>,
    pub building_id: Option<String>,

    // Owner details (when viewing from unit perspective)
    pub owner_id: Option<String>,
    pub owner_first_name: Option<String>,
    pub owner_last_name: Option<String>,
    pub owner_email: Option<String>,
    pub owner_phone: Option<String>,
}

/// DTO for transferring ownership
#[derive(Debug, Deserialize, Validate)]
pub struct TransferOwnershipDto {
    #[validate(length(min = 1))]
    pub from_owner_id: String,

    #[validate(length(min = 1))]
    pub to_owner_id: String,
}
