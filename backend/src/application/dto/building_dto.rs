use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Validate, Clone, utoipa::ToSchema)]
pub struct CreateBuildingDto {
    /// Story 1.2 — FK vers `acps.id` (anciennement `organization_id`).
    /// La migration 040000 a DROP la colonne ; le scoping org se fait
    /// désormais via `acps.organization_id`.
    pub acp_id: String,

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

    #[validate(range(min = 1, message = "Total tantiemes must be greater than 0"))]
    pub total_tantiemes: Option<i32>,

    pub construction_year: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Validate, Clone, utoipa::ToSchema)]
pub struct UpdateBuildingDto {
    /// Story 1.2 — Réaffectation de l'ACP parente (SuperAdmin uniquement).
    pub acp_id: Option<String>,

    #[validate(length(min = 1))]
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

    #[validate(range(min = 1, message = "Total tantiemes must be greater than 0"))]
    pub total_tantiemes: Option<i32>,

    pub construction_year: Option<i32>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct BuildingResponseDto {
    pub id: String,
    /// Story 1.2 — FK vers `acps.id` (anciennement `organization_id`).
    pub acp_id: String,
    pub name: String,
    pub address: String,
    pub city: String,
    pub postal_code: String,
    pub country: String,
    pub total_units: i32,
    pub total_tantiemes: i32,
    pub construction_year: Option<i32>,
    pub created_at: String,
    pub updated_at: String,

    // Story 1.4 — FR11/FR12/FR23 : conformité immeuble exposée par défaut.
    // `quota_sum` et `quota_delta` sont sérialisés en **string** Decimal-équivalent
    // (cf. ADR-0007 + mémoire `no-f64-in-money`) — jamais f64/NaN côté API.
    /// Nombre réel de `units` rattachées (COUNT(*) côté repo, JOIN units).
    #[serde(default)]
    pub units_count: i32,
    /// Somme des quotas (Decimal-as-string, ex: "1000" / "999.5" / "0").
    #[serde(default)]
    pub quota_sum: String,
    /// `units_count == total_units && quota_sum == 1000` (Decimal strict,
    /// aucune tolérance d'arrondi).
    #[serde(default)]
    pub is_conformant: bool,
    /// Delta vs 1000 (positif = surplus, négatif = manque) — pour message UX.
    #[serde(default)]
    pub quota_delta: String,
}
