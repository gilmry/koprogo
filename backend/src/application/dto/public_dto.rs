use crate::domain::entities::Building;
use serde::{Deserialize, Serialize};

/// Public syndic information response (no authentication required)
/// Belgian legal requirement: syndics must publicly display contact information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicSyndicInfoResponse {
    // Building information (public)
    pub building_name: String,
    pub building_address: String,
    pub building_city: String,
    pub building_postal_code: String,
    pub building_country: String,
    pub slug: String,

    // Syndic contact information (public per Belgian law)
    pub syndic_name: Option<String>,
    pub syndic_email: Option<String>,
    pub syndic_phone: Option<String>,
    pub syndic_address: Option<String>,
    pub syndic_office_hours: Option<String>,
    pub syndic_emergency_contact: Option<String>,

    // Metadata
    pub has_syndic_info: bool,
}

impl From<Building> for PublicSyndicInfoResponse {
    fn from(building: Building) -> Self {
        let has_syndic_info = building.has_public_syndic_info();

        Self {
            building_name: building.name,
            building_address: building.address,
            building_city: building.city,
            building_postal_code: building.postal_code,
            building_country: building.country,
            slug: building.slug.unwrap_or_default(),
            syndic_name: building.syndic_name,
            syndic_email: building.syndic_email,
            syndic_phone: building.syndic_phone,
            syndic_address: building.syndic_address,
            syndic_office_hours: building.syndic_office_hours,
            syndic_emergency_contact: building.syndic_emergency_contact,
            has_syndic_info,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_public_syndic_info_conversion() {
        let building = Building::new(
            Uuid::new_v4(),
            "Résidence Les Jardins".to_string(),
            "123 Rue de la Paix".to_string(),
            "Brussels".to_string(),
            "1000".to_string(),
            "Belgium".to_string(),
            50,
            1000,
            Some(1985),
        )
        .unwrap();

        let response = PublicSyndicInfoResponse::from(building);

        assert_eq!(response.building_name, "Résidence Les Jardins");
        assert_eq!(response.building_city, "Brussels");
        assert_eq!(response.slug, "residence-les-jardins-brussels");
        assert!(!response.has_syndic_info); // No syndic info yet
    }

    #[test]
    fn test_public_syndic_info_with_syndic() {
        let mut building = Building::new(
            Uuid::new_v4(),
            "Résidence Les Jardins".to_string(),
            "123 Rue de la Paix".to_string(),
            "Brussels".to_string(),
            "1000".to_string(),
            "Belgium".to_string(),
            50,
            1000,
            Some(1985),
        )
        .unwrap();

        building.update_syndic_info(
            Some("Syndic ASBL".to_string()),
            Some("contact@syndic.be".to_string()),
            Some("+32 2 123 4567".to_string()),
            Some("Avenue Louise 123, 1000 Brussels".to_string()),
            Some("Mon-Fri 9h-17h".to_string()),
            Some("+32 475 123 456".to_string()),
        );

        let response = PublicSyndicInfoResponse::from(building);

        assert!(response.has_syndic_info);
        assert_eq!(response.syndic_name, Some("Syndic ASBL".to_string()));
        assert_eq!(
            response.syndic_email,
            Some("contact@syndic.be".to_string())
        );
        assert_eq!(
            response.syndic_phone,
            Some("+32 2 123 4567".to_string())
        );
    }
}
