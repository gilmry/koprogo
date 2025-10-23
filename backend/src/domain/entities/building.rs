use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Représente un immeuble en copropriété
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Building {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub name: String,
    pub address: String,
    pub city: String,
    pub postal_code: String,
    pub country: String,
    pub total_units: i32,
    pub construction_year: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Building {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        organization_id: Uuid,
        name: String,
        address: String,
        city: String,
        postal_code: String,
        country: String,
        total_units: i32,
        construction_year: Option<i32>,
    ) -> Result<Self, String> {
        if name.is_empty() {
            return Err("Building name cannot be empty".to_string());
        }
        if total_units <= 0 {
            return Err("Total units must be greater than 0".to_string());
        }

        let now = Utc::now();
        Ok(Self {
            id: Uuid::new_v4(),
            organization_id,
            name,
            address,
            city,
            postal_code,
            country,
            total_units,
            construction_year,
            created_at: now,
            updated_at: now,
        })
    }

    pub fn update_info(
        &mut self,
        name: String,
        address: String,
        city: String,
        postal_code: String,
    ) {
        self.name = name;
        self.address = address;
        self.city = city;
        self.postal_code = postal_code;
        self.updated_at = Utc::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_building_success() {
        let org_id = Uuid::new_v4();
        let building = Building::new(
            org_id,
            "Résidence Les Jardins".to_string(),
            "123 Rue de la Paix".to_string(),
            "Paris".to_string(),
            "75001".to_string(),
            "France".to_string(),
            50,
            Some(1985),
        );

        assert!(building.is_ok());
        let building = building.unwrap();
        assert_eq!(building.organization_id, org_id);
        assert_eq!(building.name, "Résidence Les Jardins");
        assert_eq!(building.total_units, 50);
    }

    #[test]
    fn test_create_building_empty_name_fails() {
        let org_id = Uuid::new_v4();
        let building = Building::new(
            org_id,
            "".to_string(),
            "123 Rue de la Paix".to_string(),
            "Paris".to_string(),
            "75001".to_string(),
            "France".to_string(),
            50,
            Some(1985),
        );

        assert!(building.is_err());
        assert_eq!(building.unwrap_err(), "Building name cannot be empty");
    }

    #[test]
    fn test_create_building_zero_units_fails() {
        let org_id = Uuid::new_v4();
        let building = Building::new(
            org_id,
            "Résidence Les Jardins".to_string(),
            "123 Rue de la Paix".to_string(),
            "Paris".to_string(),
            "75001".to_string(),
            "France".to_string(),
            0,
            Some(1985),
        );

        assert!(building.is_err());
        assert_eq!(building.unwrap_err(), "Total units must be greater than 0");
    }

    #[test]
    fn test_update_building_info() {
        let org_id = Uuid::new_v4();
        let mut building = Building::new(
            org_id,
            "Old Name".to_string(),
            "Old Address".to_string(),
            "Old City".to_string(),
            "00000".to_string(),
            "France".to_string(),
            10,
            None,
        )
        .unwrap();

        let old_updated_at = building.updated_at;

        building.update_info(
            "New Name".to_string(),
            "New Address".to_string(),
            "New City".to_string(),
            "11111".to_string(),
        );

        assert_eq!(building.name, "New Name");
        assert_eq!(building.address, "New Address");
        assert!(building.updated_at > old_updated_at);
    }
}
