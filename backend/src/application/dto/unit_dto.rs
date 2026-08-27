use crate::domain::entities::UnitType;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Deserialize, Validate, Clone)]
pub struct CreateUnitDto {
    /// Story H15 — FK vers `acps.id` (anciennement `organization_id`).
    /// Le lot dérive son ACP de son building parent (cf. #602) ; le scoping
    /// org se fait via `acps.organization_id`.
    ///
    /// OPTIONNEL depuis 2026-08-27. Le champ était obligatoire, ce qui
    /// contredisait la ligne au-dessus : si l'ACP se dérive du building, le
    /// client n'a pas à la fournir. Deux conséquences mesurées :
    ///
    ///   1. Un `POST /units` sans `acp_id` était rejeté par serde AVANT
    ///      d'atteindre le handler, avec un corps en TEXTE BRUT
    ///      (« Json deserialize error: missing field `acp_id` »). Le garde-fou
    ///      `if dto.acp_id.is_empty()` du handler, qui rend un JSON propre,
    ///      était donc mort pour ce cas : il ne se déclenchait que sur une
    ///      chaîne vide explicite.
    ///
    ///   2. Tout appelant faisant `.json()` sur cette réponse recevait
    ///      « Unexpected token 'J' », un message qui ne dit rien du défaut.
    ///      C'est ce qui faisait échouer `02-ag-full-cycle` (gate de
    ///      caractérisation) et taire `seedConformantUnits` en `status=400`.
    ///
    /// Absent ou vide, l'ACP est désormais lue sur le building parent, qui
    /// est la source de vérité. Fournie, elle est utilisée telle quelle :
    /// le comportement des appelants existants est inchangé.
    #[serde(default)]
    pub acp_id: Option<String>,
    pub building_id: String,

    #[validate(length(min = 1))]
    pub unit_number: String,

    pub unit_type: UnitType,
    pub floor: Option<i32>,

    #[validate(range(min = 0.1))]
    pub surface_area: f64,

    /// Quote-part en millièmes (Decimal exact, range 0.1..=1000 enforced en domain).
    pub quota: Decimal,
}

#[derive(Debug, Deserialize, Validate, Clone)]
pub struct UpdateUnitDto {
    #[validate(length(min = 1))]
    pub unit_number: String,

    pub unit_type: UnitType,
    pub floor: i32,

    #[validate(range(min = 0.1))]
    pub surface_area: f64,

    /// Quote-part en millièmes (Decimal exact, range 0.1..=1000 enforced en domain).
    pub quota: Decimal,
}

#[derive(Debug, Serialize)]
pub struct UnitResponseDto {
    pub id: String,
    pub building_id: String,
    pub unit_number: String,
    pub unit_type: UnitType,
    pub floor: Option<i32>,
    pub surface_area: f64,
    pub quota: Decimal,
    pub owner_id: Option<String>,
}
