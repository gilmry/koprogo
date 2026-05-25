use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Total des tantièmes attendu pour qu'un immeuble soit conformant
/// (Art. 577-2 §4 CC belge — typiquement 1000 millièmes).
///
/// Strict Decimal — aucune tolérance d'arrondi (cf. mémoire
/// `no-f64-in-money` + ADR-0007). Story 1.4.
pub const CONFORMANT_QUOTA_TOTAL: Decimal = dec!(1000);

/// Métriques agrégées d'un immeuble (calculées par le repository via
/// `LEFT JOIN units` + `COUNT(*)` + `SUM(quota::NUMERIC)`).
///
/// Volontairement **non stockées** dans la table `buildings` (avoid stale state)
/// — recalculées à chaque lecture. Story 1.4 / FR23.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BuildingMetrics {
    /// Nombre réel de `units` rattachées au building (COUNT(*) côté repo).
    pub units_count: i32,
    /// Somme exacte des quotas (SUM(quota::NUMERIC) — Decimal strict, jamais f64).
    pub quota_sum: Decimal,
}

impl BuildingMetrics {
    /// Métriques vides (utile pour les builds sans units).
    pub fn empty() -> Self {
        Self {
            units_count: 0,
            quota_sum: Decimal::ZERO,
        }
    }
}

/// Représente un immeuble en copropriété
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, utoipa::ToSchema)]
pub struct Building {
    pub id: Uuid,
    /// Story 1.2 — FK vers `acps.id` (anciennement `organization_id`).
    /// La migration 20260601040000 a DROP la colonne `organization_id` ;
    /// le scoping org se fait désormais via `acps.organization_id`.
    pub acp_id: Uuid,
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
        acp_id: Uuid,
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
            acp_id,
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
            .chars()
            .map(|c| {
                // Remove accents and special characters BEFORE lowercase
                match c {
                    'À' | 'Á' | 'Â' | 'Ã' | 'Ä' | 'à' | 'á' | 'â' | 'ã' | 'ä' => 'a',
                    'È' | 'É' | 'Ê' | 'Ë' | 'è' | 'é' | 'ê' | 'ë' => 'e',
                    'Ì' | 'Í' | 'Î' | 'Ï' | 'ì' | 'í' | 'î' | 'ï' => 'i',
                    'Ò' | 'Ó' | 'Ô' | 'Õ' | 'Ö' | 'ò' | 'ó' | 'ô' | 'õ' | 'ö' => 'o',
                    'Ù' | 'Ú' | 'Û' | 'Ü' | 'ù' | 'ú' | 'û' | 'ü' => 'u',
                    'Ç' | 'ç' => 'c',
                    'Ñ' | 'ñ' => 'n',
                    _ if c.is_alphanumeric() => c.to_ascii_lowercase(),
                    _ if c.is_whitespace() || c == '-' => '-',
                    _ => '-',
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
        self.syndic_name.is_some() || self.syndic_email.is_some() || self.syndic_phone.is_some()
    }

    // ========================================================================
    // Story 1.4 — Conformité immeuble (FR11, FR12, INV-1).
    //
    // Règle (mémoire `admin-publishes-conform-buildings` + `validate-before-compute`) :
    // `is_conformant := count(units) == total_units && SUM(units.quota) == 1000`
    //
    // Pureté hexagonale : ces méthodes n'utilisent que des valeurs primitives
    // (`Decimal`, `i32`) — pas de sqlx, pas d'I/O. Les métriques arrivent via
    // `BuildingMetrics` calculé dans le repository.
    // ========================================================================

    /// Méthode d'instance : l'immeuble est-il conformant, étant donné les
    /// métriques agrégées (count units + SUM quotas) ?
    ///
    /// Strict Decimal — aucune tolérance d'arrondi (cf. ADR-0007 + mémoire
    /// `no-f64-in-money`). Un immeuble à 999/1000 millièmes est non-conformant.
    pub fn is_conformant(&self, metrics: &BuildingMetrics) -> bool {
        Self::compute_is_conformant(self.total_units, metrics)
    }

    /// Variante statique (pour tests purs sans instancier Building).
    pub fn compute_is_conformant(declared_units: i32, metrics: &BuildingMetrics) -> bool {
        metrics.units_count == declared_units && metrics.quota_sum == CONFORMANT_QUOTA_TOTAL
    }

    /// Delta des quotas vs 1000 (positif = surplus, négatif = manque).
    /// Utilisé par la fiche immeuble pour afficher un message explicite à
    /// l'utilisateur (FR11).
    pub fn quota_delta(metrics: &BuildingMetrics) -> Decimal {
        metrics.quota_sum - CONFORMANT_QUOTA_TOTAL
    }

    /// Validate unit shares distribution according to Art. 577-2 §4 Code Civil belge.
    /// Sum of unit shares must equal building total_shares (typically 1000 millièmes).
    /// Returns Ok(()) if valid, Err if invalid or incomplete, Warning if under-distributed.
    ///
    /// Belgian legal requirement: All units' shares must sum to the building's total_shares
    /// to ensure proper copropriété governance and voting/cost allocation.
    pub fn validate_unit_shares_distribution(
        units: &[crate::domain::entities::Unit],
    ) -> Result<(), String> {
        // Quotas en millièmes — Decimal exact, conversion via .trunc() vers i32 pour la borne 1000.
        use rust_decimal::prelude::ToPrimitive;
        let total_shares_decimal: rust_decimal::Decimal = units.iter().map(|u| u.quota).sum();
        let total_shares: i32 = total_shares_decimal.trunc().to_i32().unwrap_or(0);

        // Note: During setup, units may not sum to total_shares yet (incomplete distribution is OK)
        // Full validation happens at building completion/first AG
        // However, we can warn if distribution is excessive
        if total_shares > 1000 {
            return Err(format!(
                "Total unit shares ({}) exceeds maximum 1000 (Art. 577-2 §4 CC). \
                 Sum of all unit quotas cannot exceed building total_shares.",
                total_shares
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_building_success() {
        let acp_id = Uuid::new_v4();
        let building = Building::new(
            acp_id,
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
        assert_eq!(building.acp_id, acp_id);
        assert_eq!(building.name, "Résidence Les Jardins");
        assert_eq!(building.total_units, 50);
        assert_eq!(building.total_tantiemes, 1000);
    }

    #[test]
    fn test_create_building_empty_name_fails() {
        let acp_id = Uuid::new_v4();
        let building = Building::new(
            acp_id,
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
        let acp_id = Uuid::new_v4();
        let building = Building::new(
            acp_id,
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

    // ========================================================================
    // Story 1.4 — Tests `is_conformant` 4-cat (CRITICAL §3).
    // ========================================================================

    fn make_building(total_units: i32) -> Building {
        Building::new(
            Uuid::new_v4(),
            "Test".to_string(),
            "Rue Test 1".to_string(),
            "Bruxelles".to_string(),
            "1000".to_string(),
            "Belgium".to_string(),
            total_units,
            1000,
            None,
        )
        .unwrap()
    }

    #[test]
    fn happy_is_conformant_when_count_matches_and_quota_1000() {
        let b = make_building(2);
        let metrics = BuildingMetrics {
            units_count: 2,
            quota_sum: dec!(1000),
        };
        assert!(b.is_conformant(&metrics));
        assert_eq!(Building::quota_delta(&metrics), dec!(0));
    }

    #[test]
    fn edge_is_not_conformant_when_quota_off_by_one_millieme() {
        let b = make_building(2);
        let metrics = BuildingMetrics {
            units_count: 2,
            quota_sum: dec!(999),
        };
        assert!(!b.is_conformant(&metrics), "no rounding tolerance");
        assert_eq!(Building::quota_delta(&metrics), dec!(-1));
    }

    #[test]
    fn edge_is_not_conformant_when_units_count_mismatch() {
        let b = make_building(3);
        let metrics = BuildingMetrics {
            units_count: 2,
            quota_sum: dec!(1000),
        };
        assert!(!b.is_conformant(&metrics));
    }

    #[test]
    fn edge_empty_metrics_returns_zero_not_nan() {
        let metrics = BuildingMetrics::empty();
        assert_eq!(metrics.quota_sum, Decimal::ZERO);
        assert_eq!(metrics.units_count, 0);
        let b = make_building(1);
        assert!(!b.is_conformant(&metrics));
        assert_eq!(Building::quota_delta(&metrics), dec!(-1000));
    }

    #[test]
    fn security_compute_is_conformant_is_pure_no_side_effect() {
        // Méthode statique : appelable sans Building → pas de fuite d'état.
        assert!(Building::compute_is_conformant(
            3,
            &BuildingMetrics {
                units_count: 3,
                quota_sum: dec!(1000),
            }
        ));
        assert!(!Building::compute_is_conformant(
            3,
            &BuildingMetrics {
                units_count: 3,
                quota_sum: dec!(1001),
            }
        ));
    }

    #[test]
    fn negative_quota_delta_for_surplus_is_positive() {
        let metrics = BuildingMetrics {
            units_count: 2,
            quota_sum: dec!(1500),
        };
        assert_eq!(Building::quota_delta(&metrics), dec!(500));
    }

    #[test]
    fn negative_quota_delta_for_deficit_is_negative() {
        let metrics = BuildingMetrics {
            units_count: 2,
            quota_sum: dec!(900),
        };
        assert_eq!(Building::quota_delta(&metrics), dec!(-100));
    }

    #[test]
    fn test_update_building_info() {
        let acp_id = Uuid::new_v4();
        let mut building = Building::new(
            acp_id,
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
