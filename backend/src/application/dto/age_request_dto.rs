use crate::domain::entities::age_request::{AgeRequest, AgeRequestCosignatory, AgeRequestStatus};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Créer une nouvelle demande d'AGE
#[derive(Debug, Deserialize)]
pub struct CreateAgeRequestDto {
    pub building_id: Uuid,
    pub title: String,
    pub description: Option<String>,
}

/// Ajouter un cosignataire
#[derive(Debug, Deserialize)]
pub struct AddCosignatoryDto {
    pub owner_id: Uuid,
    /// Quote-part du copropriétaire (0.0 à 1.0)
    pub shares_pct: f64,
}

/// Réponse du syndic (accept ou reject)
#[derive(Debug, Deserialize)]
pub struct SyndicResponseDto {
    pub accepted: bool,
    /// Obligatoire si accepted = false
    pub notes: Option<String>,
}

/// DTO cosignataire dans la réponse
#[derive(Debug, Serialize)]
pub struct AgeRequestCosignatoryDto {
    pub id: Uuid,
    pub owner_id: Uuid,
    pub shares_pct: f64,
    pub signed_at: DateTime<Utc>,
}

impl From<&AgeRequestCosignatory> for AgeRequestCosignatoryDto {
    fn from(c: &AgeRequestCosignatory) -> Self {
        Self {
            id: c.id,
            owner_id: c.owner_id,
            shares_pct: c.shares_pct,
            signed_at: c.signed_at,
        }
    }
}

/// Réponse complète d'une demande d'AGE
#[derive(Debug, Serialize)]
pub struct AgeRequestResponseDto {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub building_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    pub created_by: Uuid,
    pub cosignatories: Vec<AgeRequestCosignatoryDto>,
    pub total_shares_pct: f64,
    pub threshold_pct: f64,
    pub threshold_reached: bool,
    pub threshold_reached_at: Option<DateTime<Utc>>,
    pub submitted_to_syndic_at: Option<DateTime<Utc>>,
    pub syndic_deadline_at: Option<DateTime<Utc>>,
    pub syndic_response_at: Option<DateTime<Utc>>,
    pub syndic_notes: Option<String>,
    pub auto_convocation_triggered: bool,
    pub meeting_id: Option<Uuid>,
    pub concertation_poll_id: Option<Uuid>,
    /// Pourcentage manquant pour atteindre le seuil (0.0 si déjà atteint)
    pub shares_pct_missing: f64,
    /// Le délai syndic est-il dépassé ?
    pub is_deadline_expired: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<&AgeRequest> for AgeRequestResponseDto {
    fn from(r: &AgeRequest) -> Self {
        Self {
            id: r.id,
            organization_id: r.organization_id,
            building_id: r.building_id,
            title: r.title.clone(),
            description: r.description.clone(),
            status: r.status.to_db_str().to_string(),
            created_by: r.created_by,
            cosignatories: r
                .cosignatories
                .iter()
                .map(AgeRequestCosignatoryDto::from)
                .collect(),
            total_shares_pct: r.total_shares_pct,
            threshold_pct: r.threshold_pct,
            threshold_reached: r.threshold_reached,
            threshold_reached_at: r.threshold_reached_at,
            submitted_to_syndic_at: r.submitted_to_syndic_at,
            syndic_deadline_at: r.syndic_deadline_at,
            syndic_response_at: r.syndic_response_at,
            syndic_notes: r.syndic_notes.clone(),
            auto_convocation_triggered: r.auto_convocation_triggered,
            meeting_id: r.meeting_id,
            concertation_poll_id: r.concertation_poll_id,
            shares_pct_missing: r.shares_pct_missing(),
            is_deadline_expired: r.is_deadline_expired(),
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}

impl From<AgeRequestStatus> for String {
    fn from(s: AgeRequestStatus) -> Self {
        s.to_db_str().to_string()
    }
}
