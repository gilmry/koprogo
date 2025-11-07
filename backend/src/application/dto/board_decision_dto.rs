use serde::{Deserialize, Serialize};
use validator::Validate;

/// DTO pour créer une nouvelle décision à suivre
#[derive(Debug, Serialize, Deserialize, Validate, Clone)]
pub struct CreateBoardDecisionDto {
    pub building_id: String, // UUID as string

    pub meeting_id: String, // UUID as string - AG qui a pris la décision

    #[validate(length(min = 1, message = "Subject cannot be empty"))]
    pub subject: String,

    #[validate(length(min = 1, message = "Decision text cannot be empty"))]
    pub decision_text: String,

    pub deadline: Option<String>, // ISO 8601 datetime string (optionnel)
}

/// DTO pour mettre à jour une décision
#[derive(Debug, Serialize, Deserialize, Validate, Clone)]
pub struct UpdateBoardDecisionDto {
    #[validate(length(min = 1, message = "Status cannot be empty"))]
    pub status: String, // "pending", "in_progress", "completed", "overdue", "cancelled"

    pub notes: Option<String>, // Notes du conseil
}

/// DTO pour ajouter des notes à une décision
#[derive(Debug, Serialize, Deserialize, Validate, Clone)]
pub struct AddDecisionNotesDto {
    #[validate(length(min = 1, message = "Notes cannot be empty"))]
    pub notes: String,
}

/// DTO pour la réponse API d'une décision
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BoardDecisionResponseDto {
    pub id: String,
    pub building_id: String,
    pub meeting_id: String,
    pub subject: String,
    pub decision_text: String,
    pub deadline: Option<String>,
    pub status: String,
    pub completed_at: Option<String>,
    pub notes: Option<String>,
    pub is_overdue: bool,                 // Calculé: deadline dépassée?
    pub days_until_deadline: Option<i64>, // Calculé: jours restants jusqu'à deadline
    pub created_at: String,
    pub updated_at: String,
}

/// DTO pour obtenir des statistiques sur les décisions
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DecisionStatsDto {
    pub building_id: String,
    pub total_decisions: i64,
    pub pending: i64,
    pub in_progress: i64,
    pub completed: i64,
    pub overdue: i64,
    pub cancelled: i64,
}

/// DTO pour les alertes de deadlines approchant
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DeadlineAlertDto {
    pub decision_id: String,
    pub subject: String,
    pub deadline: String,
    pub days_remaining: i64,
    pub urgency: String, // "critical" (<= 7 days), "high" (<= 14 days), "medium" (<= 30 days)
}
