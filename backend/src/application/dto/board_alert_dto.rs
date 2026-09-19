use serde::{Deserialize, Serialize};
use validator::Validate;

/// DTO pour créer une alerte du conseil de copropriété (Story 4.7 / #582).
///
/// `building_id` n'y figure pas : il vient du chemin de la route
/// (`/buildings/{building_id}/cdc/alerts`), pas du corps — une route scopée
/// ne doit pas laisser le corps affirmer un autre immeuble que celui de
/// l'URL.
///
/// `target_meeting_id` est fourni par l'appelant plutôt que résolu
/// automatiquement — cf. `CdcUseCases::create_alert` pour la justification.
#[derive(Debug, Serialize, Deserialize, Validate, Clone, utoipa::ToSchema)]
pub struct CreateBoardAlertDto {
    #[validate(length(min = 1, message = "Alert text cannot be empty"))]
    pub text: String,

    /// "info", "warning" ou "critical".
    pub severity: String,

    pub target_meeting_id: String,
}

/// DTO pour la réponse API d'une alerte du conseil.
#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct BoardAlertResponseDto {
    pub id: String,
    pub building_id: String,
    pub raised_by_board_member_id: String,
    pub text: String,
    pub severity: String,
    pub target_meeting_id: String,
    pub created_at: String,
}

/// Un candidat à élire au conseil de copropriété.
#[derive(Debug, Serialize, Deserialize, Validate, Clone, utoipa::ToSchema)]
pub struct CdcCandidateDto {
    pub owner_id: String,
    pub position: String, // "president", "treasurer" ou "member"
}

/// DTO pour élire les membres du conseil à l'issue d'une AG (Story 4.7).
///
/// `building_id` vient du chemin (`/buildings/{building_id}/cdc/elections`),
/// pas du corps — même raison que `CreateBoardAlertDto`.
///
/// L'AG référencée par `meeting_id` doit être `Completed` : sa clôture
/// prouve déjà le quorum double (Art. 3.87 §5 CC) via
/// `Meeting::assert_can_complete`. Cf. `CdcUseCases::elect_members`.
#[derive(Debug, Serialize, Deserialize, Validate, Clone, utoipa::ToSchema)]
pub struct ElectCdcMembersDto {
    pub meeting_id: String,
    #[validate(length(min = 1, message = "At least one candidate is required"))]
    pub candidates: Vec<CdcCandidateDto>,
}
