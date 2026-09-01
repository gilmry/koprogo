use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Type de lot (appartement, cave, parking, etc.)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, utoipa::ToSchema)]
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
    /// Story H15 — FK vers `acps.id` (anciennement `organization_id`).
    /// La migration 20260630030000 a DROP la colonne `units.organization_id` ;
    /// le scoping org se fait désormais via `acps.organization_id` (le lot
    /// dérive son ACP de son building parent, cf. #602).
    pub acp_id: Uuid,
    pub building_id: Uuid,
    pub unit_number: String,
    pub unit_type: UnitType,
    pub floor: Option<i32>,
    pub surface_area: f64, // en m² (mesure physique, f64 OK — cf. ADR-0009)
    pub quota: Decimal,    // Quote-part en millièmes (Decimal exact — cf. ADR-0007)
    pub owner_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Unit {
    pub fn new(
        acp_id: Uuid,
        building_id: Uuid,
        unit_number: String,
        unit_type: UnitType,
        floor: Option<i32>,
        surface_area: f64,
        quota: Decimal,
    ) -> Result<Self, String> {
        if unit_number.is_empty() {
            return Err("Unit number cannot be empty".to_string());
        }
        if surface_area <= 0.0 {
            return Err("Surface area must be greater than 0".to_string());
        }
        // Validate shares (tantièmes) — Art. 3.84 CC.
        // Story H8 (CL2) : plus de borne supérieure hard-codée à 1000. L'acte de
        // base peut être 1000 / 10000 / autre (cf. ADR-0010). La borne haute
        // (Σ quotités ≤ acte de base) est vérifiée à l'AGRÉGAT par
        // `Building::validate_unit_shares_distribution(units, total_tantiemes)`
        // et `Acp::assert_conformant` — pas au niveau d'un lot isolé qui ignore
        // l'acte de base de sa copropriété. Ici on garde l'invariant unitaire
        // minimal : une quote-part est strictement positive.
        if quota <= Decimal::ZERO {
            return Err("Quota (shares) must be strictly positive (Art. 3.84 CC)".to_string());
        }

        let now = Utc::now();
        Ok(Self {
            id: Uuid::new_v4(),
            acp_id,
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

    pub fn validate_update(&self) -> Result<(), String> {
        if self.unit_number.is_empty() {
            return Err("Unit number cannot be empty".to_string());
        }
        if self.surface_area <= 0.0 {
            return Err("Surface area must be greater than 0".to_string());
        }
        if self.quota <= Decimal::ZERO {
            return Err("Quota must be strictly positive (Art. 3.84 CC)".to_string());
        }
        Ok(())
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
    use rust_decimal_macros::dec;

    // ----- Story H8 (CL2) — borne quotité = acte de base (4-cat) -------------

    #[test]
    fn happy_unit_quota_5000_on_acte_10000_accepted() {
        // Avant H8 : rejeté par le cap unitaire 1000. Après : accepté — la borne
        // supérieure est l'acte de base (10000 ici), vérifiée à l'agrégat.
        let u = Unit::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "B12".to_string(),
            UnitType::Apartment,
            Some(2),
            120.0,
            dec!(5000),
        );
        assert!(
            u.is_ok(),
            "quota 5000 doit être accepté (acte de base 10000)"
        );
    }

    #[test]
    fn edge_unit_quota_just_above_1000_accepted() {
        let u = Unit::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "B13".to_string(),
            UnitType::Apartment,
            Some(2),
            60.0,
            dec!(1001),
        );
        assert!(
            u.is_ok(),
            "1001 ne doit plus être rejeté (plus de cap 1000)"
        );
    }

    #[test]
    fn security_unit_quota_zero_rejected() {
        let u = Unit::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "B14".to_string(),
            UnitType::Apartment,
            None,
            50.0,
            Decimal::ZERO,
        );
        assert!(u.is_err(), "quota nul rejeté (invariant unitaire minimal)");
    }

    #[test]
    fn negative_unit_quota_negative_rejected() {
        let u = Unit::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "B15".to_string(),
            UnitType::Apartment,
            None,
            50.0,
            dec!(-1),
        );
        assert!(u.is_err());
    }

    #[test]
    fn test_create_unit_success() {
        let acp_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let unit = Unit::new(
            acp_id,
            building_id,
            "A101".to_string(),
            UnitType::Apartment,
            Some(1),
            75.5,
            dec!(50),
        );

        assert!(unit.is_ok());
        let unit = unit.unwrap();
        assert_eq!(unit.acp_id, acp_id);
        assert_eq!(unit.unit_number, "A101");
        assert_eq!(unit.surface_area, 75.5);
    }

    #[test]
    fn test_create_unit_invalid_surface_fails() {
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let unit = Unit::new(
            org_id,
            building_id,
            "A101".to_string(),
            UnitType::Apartment,
            Some(1),
            0.0,
            dec!(50),
        );

        assert!(unit.is_err());
    }

    #[test]
    fn test_assign_owner() {
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let mut unit = Unit::new(
            org_id,
            building_id,
            "A101".to_string(),
            UnitType::Apartment,
            Some(1),
            75.5,
            dec!(50),
        )
        .unwrap();

        let owner_id = Uuid::new_v4();
        unit.assign_owner(owner_id);

        assert_eq!(unit.owner_id, Some(owner_id));
    }
}
