use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Plateforme de visioconférence supportée
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum VideoPlatform {
    Zoom,
    MicrosoftTeams,
    GoogleMeet,
    Jitsi, // Open-source, recommandé pour copropriétés (RGPD)
    Whereby,
    Other,
}

impl VideoPlatform {
    pub fn from_db_string(s: &str) -> Result<Self, String> {
        match s {
            "zoom" => Ok(Self::Zoom),
            "microsoft_teams" => Ok(Self::MicrosoftTeams),
            "google_meet" => Ok(Self::GoogleMeet),
            "jitsi" => Ok(Self::Jitsi),
            "whereby" => Ok(Self::Whereby),
            "other" => Ok(Self::Other),
            _ => Err(format!("Unknown video platform: {}", s)),
        }
    }

    pub fn to_db_str(&self) -> &'static str {
        match self {
            Self::Zoom => "zoom",
            Self::MicrosoftTeams => "microsoft_teams",
            Self::GoogleMeet => "google_meet",
            Self::Jitsi => "jitsi",
            Self::Whereby => "whereby",
            Self::Other => "other",
        }
    }
}

/// Statut de la session vidéo
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AgSessionStatus {
    Scheduled, // Lien créé, pas encore démarré
    Live,      // Session en cours
    Ended,     // Session terminée normalement
    Cancelled, // Session annulée
}

impl AgSessionStatus {
    pub fn from_db_string(s: &str) -> Result<Self, String> {
        match s {
            "scheduled" => Ok(Self::Scheduled),
            "live" => Ok(Self::Live),
            "ended" => Ok(Self::Ended),
            "cancelled" => Ok(Self::Cancelled),
            _ => Err(format!("Unknown ag session status: {}", s)),
        }
    }

    pub fn to_db_str(&self) -> &'static str {
        match self {
            Self::Scheduled => "scheduled",
            Self::Live => "live",
            Self::Ended => "ended",
            Self::Cancelled => "cancelled",
        }
    }
}

/// Session de visioconférence pour une Assemblée Générale (Art. 3.87 §1 CC)
///
/// L'Art. 3.87 §1 CC permet aux copropriétaires de participer à l'AG
/// "physiquement ou à distance au moyen d'une communication électronique".
/// Cette entité gère la session vidéo associée à une réunion.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AgSession {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub meeting_id: Uuid, // Lien vers la réunion AG
    pub platform: VideoPlatform,
    pub video_url: String,        // URL de la réunion (généré ou saisi)
    pub host_url: Option<String>, // URL hôte (avec droits admin, privé)
    pub status: AgSessionStatus,
    pub scheduled_start: DateTime<Utc>,
    pub actual_start: Option<DateTime<Utc>>,
    pub actual_end: Option<DateTime<Utc>>,

    // Quorum combiné — Art. 3.87 §5 CC
    // présentiels + participants distanciels comptent ensemble
    pub remote_attendees_count: i32, // Nb de participants en visio
    pub remote_voting_power: f64,    // Millièmes représentés par les distanciels
    pub quorum_remote_contribution: f64, // % contribution distancielle au quorum total

    // Accès et sécurité
    pub access_password: Option<String>, // Mot de passe de réunion (haché si nécessaire)
    pub waiting_room_enabled: bool,      // Salle d'attente activée (recommandée)
    pub recording_enabled: bool,         // Enregistrement (RGPD : consentement requis)
    pub recording_url: Option<String>,   // URL enregistrement post-session

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Uuid,
}

impl AgSession {
    /// Crée une nouvelle session de visioconférence
    pub fn new(
        organization_id: Uuid,
        meeting_id: Uuid,
        platform: VideoPlatform,
        video_url: String,
        host_url: Option<String>,
        scheduled_start: DateTime<Utc>,
        access_password: Option<String>,
        waiting_room_enabled: bool,
        recording_enabled: bool,
        created_by: Uuid,
    ) -> Result<Self, String> {
        if video_url.trim().is_empty() {
            return Err("L'URL de la session vidéo est obligatoire".to_string());
        }

        if !video_url.starts_with("https://") {
            return Err(
                "L'URL de la session vidéo doit utiliser HTTPS (sécurité obligatoire)".to_string(),
            );
        }

        if scheduled_start <= Utc::now() {
            return Err("La session doit être planifiée dans le futur".to_string());
        }

        let now = Utc::now();
        Ok(Self {
            id: Uuid::new_v4(),
            organization_id,
            meeting_id,
            platform,
            video_url,
            host_url,
            status: AgSessionStatus::Scheduled,
            scheduled_start,
            actual_start: None,
            actual_end: None,
            remote_attendees_count: 0,
            remote_voting_power: 0.0,
            quorum_remote_contribution: 0.0,
            access_password,
            waiting_room_enabled,
            recording_enabled,
            recording_url: None,
            created_at: now,
            updated_at: now,
            created_by,
        })
    }

    /// Démarre la session (Scheduled → Live)
    pub fn start(&mut self) -> Result<(), String> {
        if self.status != AgSessionStatus::Scheduled {
            return Err(format!(
                "Impossible de démarrer une session en statut {:?}",
                self.status
            ));
        }
        self.status = AgSessionStatus::Live;
        self.actual_start = Some(Utc::now());
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Termine la session (Live → Ended)
    pub fn end(&mut self, recording_url: Option<String>) -> Result<(), String> {
        if self.status != AgSessionStatus::Live {
            return Err(format!(
                "Impossible de terminer une session en statut {:?}",
                self.status
            ));
        }
        self.status = AgSessionStatus::Ended;
        self.actual_end = Some(Utc::now());
        self.recording_url = recording_url;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Annule la session (Scheduled → Cancelled)
    pub fn cancel(&mut self) -> Result<(), String> {
        if self.status != AgSessionStatus::Scheduled {
            return Err(format!(
                "Impossible d'annuler une session en statut {:?} (uniquement Scheduled)",
                self.status
            ));
        }
        self.status = AgSessionStatus::Cancelled;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Enregistre un participant distant et met à jour le quorum distanciel
    ///
    /// Art. 3.87 §5 CC : les participants en visio comptent pour le quorum
    /// au même titre que les présents physiquement.
    pub fn record_remote_join(
        &mut self,
        voting_power: f64,
        total_building_quotas: f64,
    ) -> Result<(), String> {
        if self.status != AgSessionStatus::Live {
            return Err(
                "Impossible d'enregistrer un participant : session non démarrée".to_string(),
            );
        }
        if voting_power < 0.0 || voting_power > total_building_quotas {
            return Err(format!(
                "Pouvoir de vote invalide : {} (total bâtiment : {})",
                voting_power, total_building_quotas
            ));
        }
        self.remote_attendees_count += 1;
        self.remote_voting_power += voting_power;
        if total_building_quotas > 0.0 {
            self.quorum_remote_contribution =
                (self.remote_voting_power / total_building_quotas) * 100.0;
        }
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Calcule le quorum combiné (présentiel + distanciel)
    ///
    /// Art. 3.87 §5 CC : nécessite >50% des millièmes
    pub fn calculate_combined_quorum(
        &self,
        physical_quotas: f64,
        total_building_quotas: f64,
    ) -> Result<f64, String> {
        if total_building_quotas <= 0.0 {
            return Err("Total des quotas du bâtiment doit être positif".to_string());
        }
        let combined = physical_quotas + self.remote_voting_power;
        Ok((combined / total_building_quotas) * 100.0)
    }

    /// Vérifie si la session est active (Live)
    pub fn is_live(&self) -> bool {
        self.status == AgSessionStatus::Live
    }

    /// Durée de la session en minutes (si terminée)
    pub fn duration_minutes(&self) -> Option<i64> {
        match (self.actual_start, self.actual_end) {
            (Some(start), Some(end)) => Some((end - start).num_minutes()),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_session() -> AgSession {
        let future = Utc::now() + chrono::Duration::hours(2);
        AgSession::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            VideoPlatform::Jitsi,
            "https://meet.jit.si/koprogo-ago-2026".to_string(),
            None,
            future,
            None,
            true,
            false,
            Uuid::new_v4(),
        )
        .unwrap()
    }

    #[test]
    fn test_create_ag_session_success() {
        let session = make_session();
        assert_eq!(session.status, AgSessionStatus::Scheduled);
        assert_eq!(session.remote_attendees_count, 0);
        assert!(session.waiting_room_enabled);
    }

    #[test]
    fn test_create_session_rejects_http_url() {
        let future = Utc::now() + chrono::Duration::hours(2);
        let result = AgSession::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            VideoPlatform::Zoom,
            "http://zoom.us/j/123".to_string(), // HTTP not allowed
            None,
            future,
            None,
            true,
            false,
            Uuid::new_v4(),
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("HTTPS"));
    }

    #[test]
    fn test_create_session_rejects_past_date() {
        let past = Utc::now() - chrono::Duration::hours(1);
        let result = AgSession::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            VideoPlatform::Jitsi,
            "https://meet.jit.si/test".to_string(),
            None,
            past,
            None,
            true,
            false,
            Uuid::new_v4(),
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_start_session() {
        let mut session = make_session();
        assert!(session.start().is_ok());
        assert_eq!(session.status, AgSessionStatus::Live);
        assert!(session.actual_start.is_some());
    }

    #[test]
    fn test_start_session_twice_fails() {
        let mut session = make_session();
        session.start().unwrap();
        assert!(session.start().is_err());
    }

    #[test]
    fn test_end_session() {
        let mut session = make_session();
        session.start().unwrap();
        assert!(session
            .end(Some("https://recording.example.com/abc".to_string()))
            .is_ok());
        assert_eq!(session.status, AgSessionStatus::Ended);
        assert!(session.actual_end.is_some());
        assert!(session.recording_url.is_some());
    }

    #[test]
    fn test_cancel_session() {
        let mut session = make_session();
        assert!(session.cancel().is_ok());
        assert_eq!(session.status, AgSessionStatus::Cancelled);
    }

    #[test]
    fn test_cancel_live_session_fails() {
        let mut session = make_session();
        session.start().unwrap();
        assert!(session.cancel().is_err());
    }

    #[test]
    fn test_record_remote_join_and_quorum() {
        let mut session = make_session();
        session.start().unwrap();

        // 150 millièmes rejoignent en visio sur 1000 total
        assert!(session.record_remote_join(150.0, 1000.0).is_ok());
        assert_eq!(session.remote_attendees_count, 1);
        assert!((session.remote_voting_power - 150.0).abs() < 0.01);
        assert!((session.quorum_remote_contribution - 15.0).abs() < 0.01);

        // 2e participant : 200 millièmes
        assert!(session.record_remote_join(200.0, 1000.0).is_ok());
        assert_eq!(session.remote_attendees_count, 2);
        assert!((session.remote_voting_power - 350.0).abs() < 0.01);
    }

    #[test]
    fn test_calculate_combined_quorum() {
        let mut session = make_session();
        session.start().unwrap();
        session.record_remote_join(200.0, 1000.0).unwrap(); // 20% en visio

        // 300 présents physiquement + 200 en visio = 500/1000 = 50% (pas suffisant, >50% requis)
        let combined = session.calculate_combined_quorum(300.0, 1000.0).unwrap();
        assert!((combined - 50.0).abs() < 0.01);

        // 310 présents + 200 visio = 510/1000 = 51% → quorum atteint
        let combined2 = session.calculate_combined_quorum(310.0, 1000.0).unwrap();
        assert!(combined2 > 50.0);
    }
}
