use crate::domain::entities::Meeting;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
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
    /// Millièmes représentés par les distanciels — `Decimal` exact (ADR-0008 :
    /// une quote-part n'est jamais un `f64`, elle alimente un seuil légal).
    pub remote_voting_power: Decimal,
    /// % de contribution distancielle au quorum total. `Decimal` et non `f64` :
    /// la valeur est **persistée** en `NUMERIC(8,4)`, un f64 intermédiaire
    /// introduirait l'aller-retour Decimal→f64→Decimal interdit par ADR-0008.
    pub quorum_remote_contribution: Decimal,

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
            remote_voting_power: Decimal::ZERO,
            quorum_remote_contribution: Decimal::ZERO,
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
        voting_power: Decimal,
        total_building_quotas: Decimal,
    ) -> Result<(), String> {
        if self.status != AgSessionStatus::Live {
            return Err(
                "Impossible d'enregistrer un participant : session non démarrée".to_string(),
            );
        }
        if voting_power < Decimal::ZERO || voting_power > total_building_quotas {
            return Err(format!(
                "Pouvoir de vote invalide : {} (total bâtiment : {})",
                voting_power, total_building_quotas
            ));
        }
        self.remote_attendees_count += 1;
        self.remote_voting_power += voting_power;
        if total_building_quotas > Decimal::ZERO {
            self.quorum_remote_contribution =
                (self.remote_voting_power / total_building_quotas) * dec!(100);
        }
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Calcule le quorum combiné (présentiel + distanciel) en pourcentage.
    ///
    /// Art. 3.87 §5 CC : les participants en visio comptent au même titre que
    /// les présents physiquement. Calcul en `Decimal` exact de bout en bout —
    /// aucun aller-retour vers `f64` sur le chemin du seuil légal (ADR-0008).
    pub fn calculate_combined_quorum(
        &self,
        physical_quotas: Decimal,
        total_building_quotas: Decimal,
    ) -> Result<Decimal, String> {
        if total_building_quotas <= Decimal::ZERO {
            return Err("Total des quotas du bâtiment doit être positif".to_string());
        }
        if physical_quotas < Decimal::ZERO {
            return Err("Les quotas présentiels ne peuvent pas être négatifs".to_string());
        }
        let combined = physical_quotas + self.remote_voting_power;
        if combined > total_building_quotas {
            return Err(format!(
                "Quorum combiné invalide : {} quotes-parts pour un total bâtiment de {}",
                combined, total_building_quotas
            ));
        }
        Ok((combined / total_building_quotas) * dec!(100))
    }

    /// Le quorum **double** combiné est-il atteint ? Art. 3.87 §5 CC.
    ///
    /// Applique la règle de `Meeting::double_quorum_reached` — la même que le
    /// chemin présentiel (Story H9) — aux totaux **combinés** présentiel +
    /// distanciel :
    ///   - têtes : présents physiquement + `remote_attendees_count` ;
    ///   - quotités : `physical_quotas` + `remote_voting_power`.
    ///
    /// Avant #661, ce chemin ne testait que les quotités, avec son propre
    /// littéral `> 50.0` en `f64` : une AG hybride pouvait être déclarée en
    /// quorum sans que le volet « têtes » exigé par la loi soit vérifié.
    ///
    /// C'est **cette** méthode que doit appeler la couche application : aucun
    /// seuil légal ne doit être ré-écrit dans un use case.
    pub fn is_combined_quorum_reached(
        &self,
        physical_quotas: Decimal,
        total_building_quotas: Decimal,
        physical_owners_count: i32,
        total_owners_count: i32,
    ) -> Result<bool, String> {
        // Valide les entrées (et interdit le double comptage) avant de juger.
        self.calculate_combined_quorum(physical_quotas, total_building_quotas)?;

        if physical_owners_count < 0 {
            return Err("Le nombre de présents physiques ne peut pas être négatif".to_string());
        }
        let combined_owners = physical_owners_count + self.remote_attendees_count;
        if total_owners_count > 0 && combined_owners > total_owners_count {
            return Err(format!(
                "Quorum en têtes invalide : {} présents pour {} copropriétaires",
                combined_owners, total_owners_count
            ));
        }

        Ok(Meeting::double_quorum_reached(
            physical_quotas + self.remote_voting_power,
            total_building_quotas,
            combined_owners,
            total_owners_count,
        ))
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

    // ------------------------------------------------------------------
    // Quorum combiné Art. 3.87 §5 CC — Issue #661
    // 4 catégories obligatoires : @happy / @edge / @security / @negative.
    // Toutes les assertions sont des égalités Decimal EXACTES : plus aucune
    // tolérance `.abs() < 0.01`, qui masquait précisément la dérive f64 que
    // l'issue dénonce sur un seuil juridique.
    // ------------------------------------------------------------------

    /// @happy — un participant distant alimente le quorum distanciel.
    #[test]
    fn test_record_remote_join_and_quorum() {
        let mut session = make_session();
        session.start().unwrap();

        // 150 millièmes rejoignent en visio sur 1000 total
        assert!(session.record_remote_join(dec!(150), dec!(1000)).is_ok());
        assert_eq!(session.remote_attendees_count, 1);
        assert_eq!(session.remote_voting_power, dec!(150));
        assert_eq!(session.quorum_remote_contribution, dec!(15));

        // 2e participant : 200 millièmes
        assert!(session.record_remote_join(dec!(200), dec!(1000)).is_ok());
        assert_eq!(session.remote_attendees_count, 2);
        assert_eq!(session.remote_voting_power, dec!(350));
    }

    /// @happy — quorum combiné présentiel + distanciel atteint.
    #[test]
    fn test_calculate_combined_quorum() {
        let mut session = make_session();
        session.start().unwrap();
        session.record_remote_join(dec!(200), dec!(1000)).unwrap(); // 20% en visio

        // 310 présents + 200 visio = 510/1000 = 51% des quotités, et 6 têtes
        // sur 10 (5 physiques + 1 distant) → quorum double atteint.
        assert_eq!(
            session
                .calculate_combined_quorum(dec!(310), dec!(1000))
                .unwrap(),
            dec!(51)
        );
        assert!(session
            .is_combined_quorum_reached(dec!(310), dec!(1000), 5, 10)
            .unwrap());
    }

    /// @edge — les deux bornes de 50% de l'Art. 3.87 §5 CC, qui ne sont **pas**
    /// de même nature :
    ///   - quotités : « au moins la moitié » → **inclusif**, 50% pile suffit ;
    ///   - têtes    : « plus de la moitié »  → **strict**, 50% pile ne suffit pas.
    ///
    /// C'est le test que l'ancien code en `f64` ne pouvait pas garantir : une
    /// borne exacte y dépendait de l'arrondi binaire, pas de la loi.
    #[test]
    fn test_combined_quorum_at_exactly_50_percent() {
        let mut session = make_session();
        session.start().unwrap();
        session.record_remote_join(dec!(200), dec!(1000)).unwrap();

        // 300 présents + 200 visio = 500/1000 = exactement 50% des quotités.
        let pct = session
            .calculate_combined_quorum(dec!(300), dec!(1000))
            .unwrap();
        assert_eq!(pct, dec!(50), "le pourcentage doit valoir 50 exactement");

        // Quotités à 50% pile + têtes 6/10 (> 50%) → ATTEINT (borne inclusive).
        assert!(
            session
                .is_combined_quorum_reached(dec!(300), dec!(1000), 5, 10)
                .unwrap(),
            "quotités : « au moins la moitié » — 50% pile satisfait ce volet"
        );

        // Mêmes quotités, mais têtes 5/10 (4 physiques + 1 distant) = 50% pile
        // → NON atteint (borne stricte côté têtes).
        assert!(
            !session
                .is_combined_quorum_reached(dec!(300), dec!(1000), 4, 10)
                .unwrap(),
            "têtes : « plus de la moitié » — 50% pile ne suffit pas"
        );
    }

    /// @security #661 — les quotités seules n'emportent jamais le quorum.
    /// Avant le correctif, ce chemin ne testait que les quotités : une AG
    /// hybride pouvait être déclarée en quorum avec deux copropriétaires
    /// détenant la moitié de l'immeuble.
    #[test]
    fn test_quotas_alone_do_not_carry_the_quorum() {
        let mut session = make_session();
        session.start().unwrap();
        session.record_remote_join(dec!(250), dec!(1000)).unwrap();

        // 550/1000 quotités (≥ 50%) mais 2 têtes sur 10 → refusé.
        assert!(!session
            .is_combined_quorum_reached(dec!(300), dec!(1000), 1, 10)
            .unwrap());
    }

    /// @edge — alternative de l'Art. 3.87 §5 : > 3/4 des quotités emportent le
    /// quorum quelles que soient les têtes.
    #[test]
    fn test_three_quarters_of_quotas_carry_the_quorum_alone() {
        let mut session = make_session();
        session.start().unwrap();
        session.record_remote_join(dec!(300), dec!(1000)).unwrap();

        // 800/1000 = 80% > 75%, avec seulement 2 têtes sur 10.
        assert!(session
            .is_combined_quorum_reached(dec!(500), dec!(1000), 1, 10)
            .unwrap());

        // 750/1000 = 75% PILE : la borne des 3/4 est stricte, donc l'alternative
        // ne joue pas, et les têtes insuffisantes font échouer le quorum.
        let mut session = make_session();
        session.start().unwrap();
        session.record_remote_join(dec!(250), dec!(1000)).unwrap();
        assert!(!session
            .is_combined_quorum_reached(dec!(500), dec!(1000), 1, 10)
            .unwrap());
    }

    /// @edge — tiers non représentable en binaire : 1/3 des quotes-parts.
    /// En `f64`, (100/300)*100 ne vaut pas 33.333… mais 33.33333333333333**57**.
    #[test]
    fn test_combined_quorum_exact_on_non_binary_fraction() {
        let mut session = make_session();
        session.start().unwrap();
        session.record_remote_join(dec!(100), dec!(300)).unwrap();

        assert_eq!(
            session.quorum_remote_contribution.round_dp(4),
            dec!(33.3333)
        );
        // 200/300 = 66.66…% des quotités, 6 têtes sur 10 → quorum atteint,
        // sans qu'aucun arrondi n'entre dans la décision.
        assert!(session
            .is_combined_quorum_reached(dec!(100), dec!(300), 5, 10)
            .unwrap());
    }

    /// @security — un pouvoir de vote distant ne peut pas dépasser le total du
    /// bâtiment ni être négatif : sinon un participant gonfle artificiellement
    /// le quorum et fait valider une AG qui n'a pas lieu d'être.
    #[test]
    fn test_record_remote_join_rejects_forged_voting_power() {
        let mut session = make_session();
        session.start().unwrap();

        assert!(session.record_remote_join(dec!(1001), dec!(1000)).is_err());
        assert!(session.record_remote_join(dec!(-1), dec!(1000)).is_err());
        // Aucune des deux tentatives n'a pollué l'état.
        assert_eq!(session.remote_attendees_count, 0);
        assert_eq!(session.remote_voting_power, Decimal::ZERO);
    }

    /// @security — le cumul présentiel + distanciel ne peut pas excéder le
    /// total du bâtiment (double comptage d'un copropriétaire présent ET
    /// connecté en visio).
    #[test]
    fn test_combined_quorum_rejects_double_counting() {
        let mut session = make_session();
        session.start().unwrap();
        session.record_remote_join(dec!(600), dec!(1000)).unwrap();

        let result = session.calculate_combined_quorum(dec!(500), dec!(1000));
        assert!(result.is_err(), "1100/1000 doit être rejeté, pas arrondi");
    }

    /// @negative — entrées invalides : erreur typée, jamais de panic ni de NaN.
    #[test]
    fn test_combined_quorum_rejects_invalid_inputs() {
        let mut session = make_session();
        session.start().unwrap();

        assert!(session
            .calculate_combined_quorum(dec!(100), Decimal::ZERO)
            .is_err());
        assert!(session
            .calculate_combined_quorum(dec!(100), dec!(-1000))
            .is_err());
        assert!(session
            .calculate_combined_quorum(dec!(-100), dec!(1000))
            .is_err());
    }

    /// @negative — enregistrer un participant sur une session non démarrée est
    /// refusé (le quorum ne se constitue que pendant la séance).
    #[test]
    fn test_record_remote_join_requires_live_session() {
        let mut session = make_session();
        assert!(session.record_remote_join(dec!(100), dec!(1000)).is_err());
    }

    /// @security #661 — le chemin distanciel et le chemin présentiel jugent
    /// contre la MÊME règle. Ce test échoue si quelqu'un réintroduit un seuil
    /// local dans l'un des deux.
    #[test]
    fn test_quorum_rule_is_shared_with_meeting_path() {
        let mut session = make_session();
        session.start().unwrap();
        session.record_remote_join(dec!(100), dec!(1000)).unwrap();

        // Pour tout jeu d'entrées, le verdict du chemin hybride doit être celui
        // de `Meeting::double_quorum_reached` sur les totaux combinés.
        for (physical, heads) in [(dec!(0), 0), (dec!(400), 4), (dec!(500), 6), (dec!(900), 1)] {
            let expected = Meeting::double_quorum_reached(
                physical + session.remote_voting_power,
                dec!(1000),
                heads + session.remote_attendees_count,
                10,
            );
            assert_eq!(
                session
                    .is_combined_quorum_reached(physical, dec!(1000), heads, 10)
                    .unwrap(),
                expected,
                "divergence pour physical={physical}, heads={heads}"
            );
        }
    }
}
