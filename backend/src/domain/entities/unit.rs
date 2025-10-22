use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Type de lot (appartement, cave, parking, etc.)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UnitType {
    Apartment,
    Parking,
    Cellar,
    Commercial,
    Other,
}

/// Représente un lot dans la copropriété
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Unit {
    pub id: Uuid,
    pub building_id: Uuid,
    pub unit_number: String,
    pub unit_type: UnitType,
    pub floor: Option<i32>,
    pub surface_area: f64, // en m²
    pub quota: f64,        // Quote-part en millièmes
    pub owner_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Unit {
    pub fn new(
        building_id: Uuid,
        unit_number: String,
        unit_type: UnitType,
        floor: Option<i32>,
        surface_area: f64,
        quota: f64,
    ) -> Result<Self, String> {
        if unit_number.is_empty() {
            return Err("Unit number cannot be empty".to_string());
        }
        if surface_area <= 0.0 {
            return Err("Surface area must be greater than 0".to_string());
        }
        if quota <= 0.0 || quota > 1000.0 {
            return Err("Quota must be between 0 and 1000".to_string());
        }

        let now = Utc::now();
        Ok(Self {
            id: Uuid::new_v4(),
            building_id,
            unit_number,
            unit_type,
            floor,
            surface_area,
            quota,
            owner_id: None,
            created_at: now,
            updated_at: now,
        })
    }

    pub fn assign_owner(&mut self, owner_id: Uuid) {
        self.owner_id = Some(owner_id);
        self.updated_at = Utc::now();
    }

    pub fn remove_owner(&mut self) {
        self.owner_id = None;
        self.updated_at = Utc::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_unit_success() {
        let building_id = Uuid::new_v4();
        let unit = Unit::new(
            building_id,
            "A101".to_string(),
            UnitType::Apartment,
            Some(1),
            75.5,
            50.0,
        );

        assert!(unit.is_ok());
        let unit = unit.unwrap();
        assert_eq!(unit.unit_number, "A101");
        assert_eq!(unit.surface_area, 75.5);
    }

    #[test]
    fn test_create_unit_invalid_surface_fails() {
        let building_id = Uuid::new_v4();
        let unit = Unit::new(
            building_id,
            "A101".to_string(),
            UnitType::Apartment,
            Some(1),
            0.0,
            50.0,
        );

        assert!(unit.is_err());
    }

    #[test]
    fn test_assign_owner() {
        let building_id = Uuid::new_v4();
        let mut unit = Unit::new(
            building_id,
            "A101".to_string(),
            UnitType::Apartment,
            Some(1),
            75.5,
            50.0,
        )
        .unwrap();

        let owner_id = Uuid::new_v4();
        unit.assign_owner(owner_id);

        assert_eq!(unit.owner_id, Some(owner_id));
    }
}
