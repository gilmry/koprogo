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
    pub total_tantiemes: i32,
    pub construction_year: Option<i32>,

    // Public syndic information (Belgian legal requirement - Issue #92)
    pub syndic_name: Option<String>,
    pub syndic_email: Option<String>,
    pub syndic_phone: Option<String>,
    pub syndic_address: Option<String>,
    pub syndic_office_hours: Option<String>,
    pub syndic_emergency_contact: Option<String>,
    pub slug: Option<String>,

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
        total_tantiemes: i32,
        construction_year: Option<i32>,
    ) -> Result<Self, String> {
        if name.is_empty() {
            return Err("Building name cannot be empty".to_string());
        }
        if total_units <= 0 {
            return Err("Total units must be greater than 0".to_string());
        }
        if total_tantiemes <= 0 {
            return Err("Total tantiemes must be greater than 0".to_string());
        }

        let now = Utc::now();
        let slug = Self::generate_slug(&name, &address, &city);

        Ok(Self {
            id: Uuid::new_v4(),
            organization_id,
            name,
            address,
            city,
            postal_code,
            country,
            total_units,
            total_tantiemes,
            construction_year,
            syndic_name: None,
            syndic_email: None,
            syndic_phone: None,
            syndic_address: None,
            syndic_office_hours: None,
            syndic_emergency_contact: None,
            slug: Some(slug),
            created_at: now,
            updated_at: now,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn update_info(
        &mut self,
        name: String,
        address: String,
        city: String,
        postal_code: String,
        country: String,
        total_units: i32,
        total_tantiemes: i32,
        construction_year: Option<i32>,
    ) {
        self.name = name.clone();
        self.address = address.clone();
        self.city = city.clone();
        self.postal_code = postal_code;
        self.country = country;
        self.total_units = total_units;
        self.total_tantiemes = total_tantiemes;
        self.construction_year = construction_year;

        // Regenerate slug if name, address, or city changed
        self.slug = Some(Self::generate_slug(&name, &address, &city));

        self.updated_at = Utc::now();
    }

    /// Update syndic public information (Belgian legal requirement)
    #[allow(clippy::too_many_arguments)]
    pub fn update_syndic_info(
        &mut self,
        syndic_name: Option<String>,
        syndic_email: Option<String>,
        syndic_phone: Option<String>,
        syndic_address: Option<String>,
        syndic_office_hours: Option<String>,
        syndic_emergency_contact: Option<String>,
    ) {
        self.syndic_name = syndic_name;
        self.syndic_email = syndic_email;
        self.syndic_phone = syndic_phone;
        self.syndic_address = syndic_address;
        self.syndic_office_hours = syndic_office_hours;
        self.syndic_emergency_contact = syndic_emergency_contact;
        self.updated_at = Utc::now();
    }

    /// Generate SEO-friendly slug from building name, address, and city
    /// Example: "Residence Les Jardins, 123 Rue de la Paix, Paris" -> "residence-les-jardins-paris"
    fn generate_slug(name: &str, _address: &str, city: &str) -> String {
        let combined = format!("{} {}", name, city);

        combined
            .to_lowercase()
            .chars()
            .map(|c| {
                if c.is_alphanumeric() {
                    c
                } else if c.is_whitespace() || c == '-' {
                    '-'
                } else {
                    // Remove accents and special characters
                    match c {
                        'à' | 'á' | 'â' | 'ã' | 'ä' => 'a',
                        'è' | 'é' | 'ê' | 'ë' => 'e',
                        'ì' | 'í' | 'î' | 'ï' => 'i',
                        'ò' | 'ó' | 'ô' | 'õ' | 'ö' => 'o',
                        'ù' | 'ú' | 'û' | 'ü' => 'u',
                        'ç' => 'c',
                        'ñ' => 'n',
                        _ => '-',
                    }
                }
            })
            .collect::<String>()
            .split('-')
            .filter(|s| !s.is_empty())
            .collect::<Vec<&str>>()
            .join("-")
    }

    /// Check if building has public syndic information available
    pub fn has_public_syndic_info(&self) -> bool {
        self.syndic_name.is_some()
            || self.syndic_email.is_some()
            || self.syndic_phone.is_some()
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
            1000,
            Some(1985),
        );

        assert!(building.is_ok());
        let building = building.unwrap();
        assert_eq!(building.organization_id, org_id);
        assert_eq!(building.name, "Résidence Les Jardins");
        assert_eq!(building.total_units, 50);
        assert_eq!(building.total_tantiemes, 1000);
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
            1000,
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
            1000,
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
            1000,
            None,
        )
        .unwrap();

        let old_updated_at = building.updated_at;

        building.update_info(
            "New Name".to_string(),
            "New Address".to_string(),
            "New City".to_string(),
            "11111".to_string(),
            "France".to_string(),
            10,
            1500,
            None,
        );

        assert_eq!(building.name, "New Name");
        assert_eq!(building.address, "New Address");
        assert_eq!(building.total_tantiemes, 1500);
        assert!(building.updated_at > old_updated_at);
    }
}
