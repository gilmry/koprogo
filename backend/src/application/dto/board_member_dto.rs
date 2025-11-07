use serde::{Deserialize, Serialize};
use validator::Validate;

/// DTO pour créer un nouveau membre du conseil
/// Note: board members must be property owners (Owner), not platform users
#[derive(Debug, Serialize, Deserialize, Validate, Clone)]
pub struct CreateBoardMemberDto {
    pub owner_id: String, // UUID as string - ID du copropriétaire (Owner)

    pub building_id: String, // UUID as string

    #[validate(length(min = 1, message = "Position cannot be empty"))]
    pub position: String, // "president", "treasurer", or "member"

    pub mandate_start: String, // ISO 8601 datetime string

    pub mandate_end: String, // ISO 8601 datetime string

    pub elected_by_meeting_id: String, // UUID as string - ID de l'AG qui élit
}

/// DTO pour renouveler un mandat existant
#[derive(Debug, Serialize, Deserialize, Validate, Clone)]
pub struct RenewMandateDto {
    pub new_elected_by_meeting_id: String, // UUID de la nouvelle AG qui renouvelle
}

/// DTO pour la réponse API d'un membre du conseil
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BoardMemberResponseDto {
    pub id: String,
    pub owner_id: String, // ID du copropriétaire (Owner)
    pub building_id: String,
    pub position: String,
    pub mandate_start: String,
    pub mandate_end: String,
    pub elected_by_meeting_id: String,
    pub is_active: bool,     // Calculé: mandat actuellement actif?
    pub days_remaining: i64, // Calculé: jours restants dans le mandat
    pub expires_soon: bool,  // Calculé: expire dans < 60 jours?
    pub created_at: String,
    pub updated_at: String,
}

/// DTO pour obtenir des statistiques sur le conseil
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BoardStatsDto {
    pub building_id: String,
    pub total_members: i64,
    pub active_members: i64,
    pub expiring_soon: i64, // Nombre de mandats expirant dans < 60 jours
    pub has_president: bool,
    pub has_treasurer: bool,
}
