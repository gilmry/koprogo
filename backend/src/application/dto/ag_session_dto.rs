use crate::domain::entities::ag_session::AgSession;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// DTO de création d'une session AG visioconférence
#[derive(Debug, Clone, Deserialize)]
pub struct CreateAgSessionDto {
    pub meeting_id: Uuid,
    pub platform: String, // "zoom" | "microsoft_teams" | "google_meet" | "jitsi" | "whereby" | "other"
    pub video_url: String,
    pub host_url: Option<String>,
    pub scheduled_start: DateTime<Utc>,
    pub access_password: Option<String>,
    pub waiting_room_enabled: Option<bool>,
    pub recording_enabled: Option<bool>,
}

/// DTO pour terminer une session
#[derive(Debug, Clone, Deserialize)]
pub struct EndAgSessionDto {
    pub recording_url: Option<String>,
}

/// DTO pour enregistrer un participant distant
#[derive(Debug, Clone, Deserialize)]
pub struct RecordRemoteJoinDto {
    pub voting_power: Decimal,
    pub total_building_quotas: Decimal,
}

/// Réponse API pour une session AG
#[derive(Debug, Clone, Serialize)]
pub struct AgSessionResponse {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub meeting_id: Uuid,
    pub platform: String,
    pub video_url: String,
    pub host_url: Option<String>,
    pub status: String,
    pub scheduled_start: DateTime<Utc>,
    pub actual_start: Option<DateTime<Utc>>,
    pub actual_end: Option<DateTime<Utc>>,
    pub remote_attendees_count: i32,
    pub remote_voting_power: Decimal,
    pub quorum_remote_contribution: Decimal,
    pub waiting_room_enabled: bool,
    pub recording_enabled: bool,
    pub recording_url: Option<String>,
    pub duration_minutes: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Uuid,
}

/// Réponse pour le calcul du quorum combiné
#[derive(Debug, Clone, Serialize)]
pub struct CombinedQuorumResponse {
    pub session_id: Uuid,
    pub meeting_id: Uuid,
    pub physical_quotas: Decimal,
    pub remote_quotas: Decimal,
    pub total_building_quotas: Decimal,
    pub combined_percentage: Decimal,
    /// Volet « têtes » du quorum double (#661) — présents physiquement,
    /// connectés en visio, et total des copropriétaires.
    pub physical_owners_count: i32,
    pub remote_attendees_count: i32,
    pub total_owners_count: i32,
    /// Art. 3.87 §5 CC — décidé par `AgSession::is_combined_quorum_reached`,
    /// qui applique le quorum DOUBLE (têtes ET quotités) de
    /// `Meeting::double_quorum_reached`. Jamais recalculé ici.
    pub quorum_reached: bool,
}

impl From<&AgSession> for AgSessionResponse {
    fn from(s: &AgSession) -> Self {
        Self {
            id: s.id,
            organization_id: s.organization_id,
            meeting_id: s.meeting_id,
            platform: s.platform.to_db_str().to_string(),
            video_url: s.video_url.clone(),
            host_url: s.host_url.clone(),
            status: s.status.to_db_str().to_string(),
            scheduled_start: s.scheduled_start,
            actual_start: s.actual_start,
            actual_end: s.actual_end,
            remote_attendees_count: s.remote_attendees_count,
            remote_voting_power: s.remote_voting_power,
            quorum_remote_contribution: s.quorum_remote_contribution,
            waiting_room_enabled: s.waiting_room_enabled,
            recording_enabled: s.recording_enabled,
            recording_url: s.recording_url.clone(),
            duration_minutes: s.duration_minutes(),
            created_at: s.created_at,
            updated_at: s.updated_at,
            created_by: s.created_by,
        }
    }
}
