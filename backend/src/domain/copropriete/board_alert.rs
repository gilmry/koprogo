//! L'alerte du conseil de copropriété — Story 4.7.
//!
//! Art. 3.90 § 1er confie au conseil la mission de « veiller à la bonne
//! exécution par le syndic de ses missions », mais ne lui donne aucun canal
//! pour agir sur ce constat : un titre sans instrument. `create_alert` est cet
//! instrument minimal — un membre en mandat signale un fait à l'assemblée
//! suivante ; celle-ci en décide.
//!
//! Voir issue #582.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Gravité déclarée par le membre qui émet l'alerte.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}

impl std::fmt::Display for AlertSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AlertSeverity::Info => write!(f, "info"),
            AlertSeverity::Warning => write!(f, "warning"),
            AlertSeverity::Critical => write!(f, "critical"),
        }
    }
}

impl std::str::FromStr for AlertSeverity {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "info" => Ok(AlertSeverity::Info),
            "warning" => Ok(AlertSeverity::Warning),
            "critical" => Ok(AlertSeverity::Critical),
            _ => Err(format!("Invalid alert severity: {}", s)),
        }
    }
}

/// Ce qui empêche la création d'une alerte. Typé — pas de `String` — pour
/// que la couche application le mappe explicitement (cf. `AppError::from`).
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum AlerteRefusee {
    #[error("Le texte de l'alerte ne peut pas être vide")]
    TexteVide,
}

/// Une alerte émise par le conseil de copropriété à destination de la
/// prochaine assemblée générale.
///
/// `target_meeting_id` fixe la cible au moment de la création — c'est la
/// traduction opérationnelle de `target=AG_next` : la couche application
/// résout « la prochaine AG » (l'AG à venir de l'immeuble) et fige cette
/// résolution ici, plutôt que de la recalculer dynamiquement à chaque
/// lecture (une AG qui se tient entre-temps ne doit pas faire migrer une
/// alerte déjà émise vers l'AG suivante).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoardAlert {
    pub id: Uuid,
    pub building_id: Uuid,
    /// Le membre du conseil (mandat en cours au moment de l'émission) qui
    /// a levé l'alerte — référence `BoardMember.id`, pas `owner_id` : elle
    /// trace le mandat précis, pas seulement la personne.
    pub raised_by_board_member_id: Uuid,
    pub text: String,
    pub severity: AlertSeverity,
    pub target_meeting_id: Uuid,
    pub created_at: DateTime<Utc>,
}

impl BoardAlert {
    pub fn new(
        building_id: Uuid,
        raised_by_board_member_id: Uuid,
        text: String,
        severity: AlertSeverity,
        target_meeting_id: Uuid,
    ) -> Result<Self, AlerteRefusee> {
        if text.trim().is_empty() {
            return Err(AlerteRefusee::TexteVide);
        }

        Ok(Self {
            id: Uuid::new_v4(),
            building_id,
            raised_by_board_member_id,
            text,
            severity,
            target_meeting_id,
            created_at: Utc::now(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ids() -> (Uuid, Uuid, Uuid) {
        (Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4())
    }

    // ── @happy ───────────────────────────────────────────────────────

    #[test]
    fn happy_create_alert_succeeds() {
        let (building_id, member_id, meeting_id) = ids();
        let alert = BoardAlert::new(
            building_id,
            member_id,
            "L'ascenseur est bloqué depuis trois jours".to_string(),
            AlertSeverity::Warning,
            meeting_id,
        )
        .unwrap();

        assert_eq!(alert.building_id, building_id);
        assert_eq!(alert.raised_by_board_member_id, member_id);
        assert_eq!(alert.target_meeting_id, meeting_id);
        assert_eq!(alert.severity, AlertSeverity::Warning);
    }

    // ── @edge ────────────────────────────────────────────────────────

    /// Un texte fait uniquement d'espaces n'informe personne — même refus
    /// que `BoardDecision::new` sur `decision_text` vide.
    #[test]
    fn edge_whitespace_only_text_is_refused() {
        let (building_id, member_id, meeting_id) = ids();
        let result = BoardAlert::new(
            building_id,
            member_id,
            "   \t  ".to_string(),
            AlertSeverity::Info,
            meeting_id,
        );
        assert_eq!(result.unwrap_err(), AlerteRefusee::TexteVide);
    }

    // ── @negative ────────────────────────────────────────────────────

    #[test]
    fn negative_empty_text_is_refused() {
        let (building_id, member_id, meeting_id) = ids();
        let result = BoardAlert::new(
            building_id,
            member_id,
            String::new(),
            AlertSeverity::Critical,
            meeting_id,
        );
        assert_eq!(result.unwrap_err(), AlerteRefusee::TexteVide);
    }

    #[test]
    fn negative_unknown_severity_string_fails_to_parse() {
        let result: Result<AlertSeverity, String> = "urgentissime".parse();
        assert!(result.is_err());
    }

    #[test]
    fn happy_severity_display_and_from_str_round_trip() {
        for s in [
            AlertSeverity::Info,
            AlertSeverity::Warning,
            AlertSeverity::Critical,
        ] {
            let parsed: AlertSeverity = s.to_string().parse().unwrap();
            assert_eq!(parsed, s);
        }
    }
}
