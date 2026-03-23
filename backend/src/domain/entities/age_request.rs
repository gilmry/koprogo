use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Statut d'une demande d'AGE par les copropriétaires
///
/// Machine d'état (Art. 3.87 §2 CC):
/// Draft → Open → Reached → Submitted → Accepted/Expired/Rejected
/// Tout état → Withdrawn
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AgeRequestStatus {
    /// En cours de rédaction (collecte de signatures privée)
    Draft,
    /// Ouverte pour signatures (visible aux copropriétaires du bâtiment)
    Open,
    /// Seuil 1/5 atteint — prête à soumettre au syndic
    Reached,
    /// Soumise au syndic (délai 15j démarré)
    Submitted,
    /// Syndic a accepté de convoquer l'AGE
    Accepted,
    /// Délai syndic expiré — auto-convocation par les copropriétaires
    Expired,
    /// Syndic a refusé (avec motif)
    Rejected,
    /// Retirée par les demandeurs
    Withdrawn,
}

impl AgeRequestStatus {
    pub fn from_db_string(s: &str) -> Result<Self, String> {
        match s {
            "draft" => Ok(Self::Draft),
            "open" => Ok(Self::Open),
            "reached" => Ok(Self::Reached),
            "submitted" => Ok(Self::Submitted),
            "accepted" => Ok(Self::Accepted),
            "expired" => Ok(Self::Expired),
            "rejected" => Ok(Self::Rejected),
            "withdrawn" => Ok(Self::Withdrawn),
            _ => Err(format!("Unknown age_request_status: {}", s)),
        }
    }

    pub fn to_db_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Open => "open",
            Self::Reached => "reached",
            Self::Submitted => "submitted",
            Self::Accepted => "accepted",
            Self::Expired => "expired",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }

    /// Retourne true si l'état est terminal (ne peut plus évoluer)
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            Self::Accepted | Self::Expired | Self::Rejected | Self::Withdrawn
        )
    }
}

/// Cosignataire d'une demande d'AGE
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AgeRequestCosignatory {
    pub id: Uuid,
    pub age_request_id: Uuid,
    pub owner_id: Uuid,
    /// Quote-part de ce copropriétaire (0.0 à 1.0, ex: 0.10 = 10%)
    pub shares_pct: f64,
    pub signed_at: DateTime<Utc>,
}

impl AgeRequestCosignatory {
    pub fn new(age_request_id: Uuid, owner_id: Uuid, shares_pct: f64) -> Result<Self, String> {
        if shares_pct <= 0.0 || shares_pct > 1.0 {
            return Err(format!(
                "shares_pct doit être entre 0 et 1, reçu: {}",
                shares_pct
            ));
        }
        Ok(Self {
            id: Uuid::new_v4(),
            age_request_id,
            owner_id,
            shares_pct,
            signed_at: Utc::now(),
        })
    }
}

/// Demande d'Assemblée Générale Extraordinaire par les copropriétaires
///
/// Art. 3.87 §2 Code Civil Belge :
/// "Tout copropriétaire peut demander au syndic de convoquer une assemblée générale.
/// Si la demande émane d'un ou de plusieurs copropriétaires représentant au moins
/// un cinquième des quotes-parts dans les parties communes, le syndic est tenu de
/// convoquer cette assemblée."
///
/// Délai syndic : 15 jours pour répondre/agir, sinon les demandeurs peuvent
/// convoquer eux-mêmes l'assemblée.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AgeRequest {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub building_id: Uuid,

    /// Objet/titre de la demande d'AGE
    pub title: String,

    /// Description détaillée des raisons de la demande
    pub description: Option<String>,

    /// Statut courant de la demande
    pub status: AgeRequestStatus,

    /// Copropriétaire initiateur
    pub created_by: Uuid,

    /// Liste des cosignataires
    pub cosignatories: Vec<AgeRequestCosignatory>,

    /// Total des quotes-parts des cosignataires (0.0 à 1.0)
    pub total_shares_pct: f64,

    /// Seuil légal à atteindre (0.2 = 20% = 1/5, Art. 3.87 §2)
    pub threshold_pct: f64,

    /// Seuil atteint ?
    pub threshold_reached: bool,

    /// Date à laquelle le seuil a été atteint
    pub threshold_reached_at: Option<DateTime<Utc>>,

    /// Date de soumission formelle au syndic
    pub submitted_to_syndic_at: Option<DateTime<Utc>>,

    /// Délai imparti au syndic (soumission + 15j)
    pub syndic_deadline_at: Option<DateTime<Utc>>,

    /// Date de réponse du syndic
    pub syndic_response_at: Option<DateTime<Utc>>,

    /// Notes du syndic (raison d'acceptation, de refus, etc.)
    pub syndic_notes: Option<String>,

    /// Auto-convocation déclenchée car syndic inactif > 15j
    pub auto_convocation_triggered: bool,

    /// Réunion AG convoquée (set lors de l'acceptation ou l'expiration)
    pub meeting_id: Option<Uuid>,

    /// Sondage de concertation pré-AGE (lien vers Poll)
    pub concertation_poll_id: Option<Uuid>,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl AgeRequest {
    /// Délai légal accordé au syndic pour agir (Art. 3.87 §2 CC)
    pub const SYNDIC_DEADLINE_DAYS: i64 = 15;

    /// Seuil légal : 1/5 des quotes-parts (Art. 3.87 §2 CC)
    pub const DEFAULT_THRESHOLD_PCT: f64 = 0.20;

    /// Crée une nouvelle demande d'AGE
    pub fn new(
        organization_id: Uuid,
        building_id: Uuid,
        title: String,
        description: Option<String>,
        created_by: Uuid,
    ) -> Result<Self, String> {
        if title.trim().is_empty() {
            return Err("Le titre de la demande d'AGE ne peut pas être vide".to_string());
        }
        if title.len() > 255 {
            return Err("Le titre ne peut pas dépasser 255 caractères".to_string());
        }

        let now = Utc::now();
        Ok(Self {
            id: Uuid::new_v4(),
            organization_id,
            building_id,
            title: title.trim().to_string(),
            description,
            status: AgeRequestStatus::Draft,
            created_by,
            cosignatories: Vec::new(),
            total_shares_pct: 0.0,
            threshold_pct: Self::DEFAULT_THRESHOLD_PCT,
            threshold_reached: false,
            threshold_reached_at: None,
            submitted_to_syndic_at: None,
            syndic_deadline_at: None,
            syndic_response_at: None,
            syndic_notes: None,
            auto_convocation_triggered: false,
            meeting_id: None,
            concertation_poll_id: None,
            created_at: now,
            updated_at: now,
        })
    }

    /// Ouvre la demande pour signatures publiques (Draft → Open)
    pub fn open(&mut self) -> Result<(), String> {
        if self.status != AgeRequestStatus::Draft {
            return Err(format!(
                "Impossible d'ouvrir une demande en statut {:?}",
                self.status
            ));
        }
        self.status = AgeRequestStatus::Open;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Ajoute un cosignataire et recalcule le total des quotes-parts
    /// Retourne true si le seuil 1/5 vient d'être atteint
    pub fn add_cosignatory(&mut self, owner_id: Uuid, shares_pct: f64) -> Result<bool, String> {
        if self.status != AgeRequestStatus::Draft && self.status != AgeRequestStatus::Open {
            return Err(format!(
                "Impossible d'ajouter un cosignataire en statut {:?}",
                self.status
            ));
        }

        // Vérifier si ce copropriétaire a déjà signé
        if self.cosignatories.iter().any(|c| c.owner_id == owner_id) {
            return Err("Ce copropriétaire a déjà signé cette demande".to_string());
        }

        let cosignatory = AgeRequestCosignatory::new(self.id, owner_id, shares_pct)?;
        self.cosignatories.push(cosignatory);

        // Recalcul du total
        self.total_shares_pct = self.cosignatories.iter().map(|c| c.shares_pct).sum();
        self.updated_at = Utc::now();

        // Vérification du seuil
        let newly_reached = !self.threshold_reached && self.total_shares_pct >= self.threshold_pct;

        if newly_reached {
            self.threshold_reached = true;
            self.threshold_reached_at = Some(Utc::now());
            self.status = AgeRequestStatus::Reached;
        }

        Ok(newly_reached)
    }

    /// Retire un cosignataire et recalcule le total
    pub fn remove_cosignatory(&mut self, owner_id: Uuid) -> Result<(), String> {
        if self.status != AgeRequestStatus::Draft
            && self.status != AgeRequestStatus::Open
            && self.status != AgeRequestStatus::Reached
        {
            return Err(format!(
                "Impossible de retirer un cosignataire en statut {:?}",
                self.status
            ));
        }

        let before_len = self.cosignatories.len();
        self.cosignatories.retain(|c| c.owner_id != owner_id);

        if self.cosignatories.len() == before_len {
            return Err("Ce copropriétaire n'a pas signé cette demande".to_string());
        }

        // Recalcul
        self.total_shares_pct = self.cosignatories.iter().map(|c| c.shares_pct).sum();
        self.updated_at = Utc::now();

        // Rétrograder si seuil plus atteint
        if self.threshold_reached && self.total_shares_pct < self.threshold_pct {
            self.threshold_reached = false;
            self.threshold_reached_at = None;
            self.status = AgeRequestStatus::Open; // Retour à Open (était peut-être Reached)
        }

        Ok(())
    }

    /// Soumet formellement la demande au syndic (Reached → Submitted)
    pub fn submit_to_syndic(&mut self) -> Result<(), String> {
        if self.status != AgeRequestStatus::Reached {
            return Err(format!(
                "La demande doit être en statut Reached pour être soumise (statut actuel: {:?}). \
                 Le seuil d'1/5 des quotes-parts doit être atteint.",
                self.status
            ));
        }

        let now = Utc::now();
        self.status = AgeRequestStatus::Submitted;
        self.submitted_to_syndic_at = Some(now);
        self.syndic_deadline_at = Some(now + Duration::days(Self::SYNDIC_DEADLINE_DAYS));
        self.updated_at = now;
        Ok(())
    }

    /// Syndic accepte la demande (Submitted → Accepted)
    pub fn accept_by_syndic(&mut self, notes: Option<String>) -> Result<(), String> {
        if self.status != AgeRequestStatus::Submitted {
            return Err(format!(
                "La demande doit être en statut Submitted pour être acceptée (statut actuel: {:?})",
                self.status
            ));
        }
        let now = Utc::now();
        self.status = AgeRequestStatus::Accepted;
        self.syndic_response_at = Some(now);
        self.syndic_notes = notes;
        self.updated_at = now;
        Ok(())
    }

    /// Syndic rejette la demande avec motif (Submitted → Rejected)
    pub fn reject_by_syndic(&mut self, reason: String) -> Result<(), String> {
        if self.status != AgeRequestStatus::Submitted {
            return Err(format!(
                "La demande doit être en statut Submitted pour être rejetée (statut actuel: {:?})",
                self.status
            ));
        }
        if reason.trim().is_empty() {
            return Err("Un motif de refus est obligatoire".to_string());
        }
        let now = Utc::now();
        self.status = AgeRequestStatus::Rejected;
        self.syndic_response_at = Some(now);
        self.syndic_notes = Some(reason);
        self.updated_at = now;
        Ok(())
    }

    /// Déclenche l'auto-convocation car le syndic n'a pas répondu dans le délai (Submitted → Expired)
    pub fn trigger_auto_convocation(&mut self) -> Result<(), String> {
        if self.status != AgeRequestStatus::Submitted {
            return Err(format!(
                "La demande doit être en statut Submitted (statut actuel: {:?})",
                self.status
            ));
        }

        // Vérifier que le délai est effectivement dépassé
        if let Some(deadline) = self.syndic_deadline_at {
            if Utc::now() < deadline {
                return Err(format!(
                    "Le délai syndic n'est pas encore dépassé (expire le {})",
                    deadline.format("%d/%m/%Y")
                ));
            }
        }

        self.status = AgeRequestStatus::Expired;
        self.auto_convocation_triggered = true;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Retire la demande (tout état non terminal → Withdrawn)
    pub fn withdraw(&mut self, requester_id: Uuid) -> Result<(), String> {
        if self.status.is_terminal() {
            return Err(format!(
                "Impossible de retirer une demande en statut {:?}",
                self.status
            ));
        }
        // Seul l'initiateur peut retirer la demande
        if self.created_by != requester_id {
            return Err("Seul l'initiateur peut retirer cette demande".to_string());
        }
        self.status = AgeRequestStatus::Withdrawn;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Lie une réunion AG à cette demande
    pub fn set_meeting(&mut self, meeting_id: Uuid) {
        self.meeting_id = Some(meeting_id);
        self.updated_at = Utc::now();
    }

    /// Lie un sondage de concertation pré-AGE à cette demande
    pub fn set_concertation_poll(&mut self, poll_id: Uuid) {
        self.concertation_poll_id = Some(poll_id);
        self.updated_at = Utc::now();
    }

    /// Vérifie si le délai syndic est dépassé
    pub fn is_deadline_expired(&self) -> bool {
        self.syndic_deadline_at
            .map(|d| Utc::now() > d)
            .unwrap_or(false)
    }

    /// Retourne le pourcentage manquant pour atteindre le seuil (0.0 si déjà atteint)
    pub fn shares_pct_missing(&self) -> f64 {
        if self.threshold_reached {
            0.0
        } else {
            (self.threshold_pct - self.total_shares_pct).max(0.0)
        }
    }

    /// Calcule le progrès vers le seuil 1/5 en pourcentage (0-100%)
    ///
    /// Exemple : Si 10% des quotes-parts ont signé et le seuil est 20%,
    /// retourne 50.0 (50% du chemin vers le seuil).
    ///
    /// # Arguments
    /// * `building_total_shares` - Total des quotes-parts du bâtiment (normalement 1.0)
    ///
    /// # Returns
    /// Pourcentage de progression : 0.0 (0%) à 100.0 (seuil atteint ou dépassé)
    pub fn calculate_progress_percentage(&self, _building_total_shares: f64) -> f64 {
        // Calcul : (current / threshold) * 100, capped at 100%
        let progress = (self.total_shares_pct / self.threshold_pct) * 100.0;
        progress.min(100.0) // Ne jamais dépasser 100%
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_request() -> AgeRequest {
        AgeRequest::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "Remplacement toiture - AGE urgente".to_string(),
            Some("La toiture présente des infiltrations importantes.".to_string()),
            Uuid::new_v4(),
        )
        .unwrap()
    }

    #[test]
    fn test_new_age_request_is_draft() {
        let req = make_request();
        assert_eq!(req.status, AgeRequestStatus::Draft);
        assert_eq!(req.total_shares_pct, 0.0);
        assert!(!req.threshold_reached);
        assert_eq!(req.threshold_pct, AgeRequest::DEFAULT_THRESHOLD_PCT);
        assert!(req.cosignatories.is_empty());
    }

    #[test]
    fn test_empty_title_rejected() {
        let result = AgeRequest::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "   ".to_string(),
            None,
            Uuid::new_v4(),
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_open_transitions_from_draft() {
        let mut req = make_request();
        req.open().unwrap();
        assert_eq!(req.status, AgeRequestStatus::Open);
    }

    #[test]
    fn test_open_fails_if_not_draft() {
        let mut req = make_request();
        req.status = AgeRequestStatus::Reached;
        assert!(req.open().is_err());
    }

    #[test]
    fn test_add_cosignatory_accumulates_shares() {
        let mut req = make_request();
        req.open().unwrap();

        let owner1 = Uuid::new_v4();
        let newly_reached = req.add_cosignatory(owner1, 0.10).unwrap();
        assert!(!newly_reached);
        assert!((req.total_shares_pct - 0.10).abs() < 1e-9);
        assert_eq!(req.status, AgeRequestStatus::Open);
    }

    #[test]
    fn test_threshold_reached_at_20_percent() {
        let mut req = make_request();
        req.open().unwrap();

        // Premier signataire : 10%
        let o1 = Uuid::new_v4();
        let reached = req.add_cosignatory(o1, 0.10).unwrap();
        assert!(!reached);
        assert_eq!(req.status, AgeRequestStatus::Open);

        // Deuxième signataire : 12% → total 22% ≥ 20%
        let o2 = Uuid::new_v4();
        let reached = req.add_cosignatory(o2, 0.12).unwrap();
        assert!(reached);
        assert_eq!(req.status, AgeRequestStatus::Reached);
        assert!(req.threshold_reached);
        assert!(req.threshold_reached_at.is_some());
        assert!((req.total_shares_pct - 0.22).abs() < 1e-9);
    }

    #[test]
    fn test_duplicate_cosignatory_rejected() {
        let mut req = make_request();
        req.open().unwrap();
        let owner = Uuid::new_v4();
        req.add_cosignatory(owner, 0.10).unwrap();
        let result = req.add_cosignatory(owner, 0.05);
        assert!(result.is_err());
    }

    #[test]
    fn test_remove_cosignatory_reverts_status() {
        let mut req = make_request();
        req.open().unwrap();

        let o1 = Uuid::new_v4();
        let o2 = Uuid::new_v4();
        req.add_cosignatory(o1, 0.15).unwrap();
        req.add_cosignatory(o2, 0.10).unwrap(); // total 25% → Reached

        assert_eq!(req.status, AgeRequestStatus::Reached);

        // Retirer o2 → total 15% < 20% → retour Open
        req.remove_cosignatory(o2).unwrap();
        assert_eq!(req.status, AgeRequestStatus::Open);
        assert!(!req.threshold_reached);
    }

    #[test]
    fn test_submit_to_syndic() {
        let mut req = make_request();
        req.open().unwrap();
        let o1 = Uuid::new_v4();
        req.add_cosignatory(o1, 0.25).unwrap(); // Reached

        req.submit_to_syndic().unwrap();
        assert_eq!(req.status, AgeRequestStatus::Submitted);
        assert!(req.submitted_to_syndic_at.is_some());
        assert!(req.syndic_deadline_at.is_some());

        // Deadline = submitted + 15j
        let diff = req.syndic_deadline_at.unwrap() - req.submitted_to_syndic_at.unwrap();
        assert_eq!(diff.num_days(), AgeRequest::SYNDIC_DEADLINE_DAYS);
    }

    #[test]
    fn test_submit_fails_if_not_reached() {
        let mut req = make_request();
        req.open().unwrap();
        // Sans cosignataires
        assert!(req.submit_to_syndic().is_err());
    }

    #[test]
    fn test_accept_by_syndic() {
        let mut req = make_request();
        req.open().unwrap();
        req.add_cosignatory(Uuid::new_v4(), 0.25).unwrap();
        req.submit_to_syndic().unwrap();
        req.accept_by_syndic(Some("Convocation dans 3 semaines".to_string()))
            .unwrap();
        assert_eq!(req.status, AgeRequestStatus::Accepted);
        assert!(req.syndic_response_at.is_some());
    }

    #[test]
    fn test_reject_requires_reason() {
        let mut req = make_request();
        req.open().unwrap();
        req.add_cosignatory(Uuid::new_v4(), 0.25).unwrap();
        req.submit_to_syndic().unwrap();
        assert!(req.reject_by_syndic("  ".to_string()).is_err());
        req.reject_by_syndic("Demande insuffisamment motivée".to_string())
            .unwrap();
        assert_eq!(req.status, AgeRequestStatus::Rejected);
    }

    #[test]
    fn test_withdraw_by_initiator_only() {
        let mut req = make_request();
        req.open().unwrap();

        let other = Uuid::new_v4();
        assert!(req.withdraw(other).is_err());

        let initiator = req.created_by;
        req.withdraw(initiator).unwrap();
        assert_eq!(req.status, AgeRequestStatus::Withdrawn);
    }

    #[test]
    fn test_shares_pct_missing() {
        let mut req = make_request();
        req.open().unwrap();

        // 0 signatures → 20% manquants
        assert!((req.shares_pct_missing() - 0.20).abs() < 1e-9);

        req.add_cosignatory(Uuid::new_v4(), 0.12).unwrap();
        // 12% → 8% manquants
        assert!((req.shares_pct_missing() - 0.08).abs() < 1e-9);

        req.add_cosignatory(Uuid::new_v4(), 0.10).unwrap();
        // Reached → 0% manquants
        assert_eq!(req.shares_pct_missing(), 0.0);
    }

    #[test]
    fn test_status_is_terminal() {
        assert!(AgeRequestStatus::Accepted.is_terminal());
        assert!(AgeRequestStatus::Expired.is_terminal());
        assert!(AgeRequestStatus::Rejected.is_terminal());
        assert!(AgeRequestStatus::Withdrawn.is_terminal());
        assert!(!AgeRequestStatus::Draft.is_terminal());
        assert!(!AgeRequestStatus::Open.is_terminal());
        assert!(!AgeRequestStatus::Reached.is_terminal());
        assert!(!AgeRequestStatus::Submitted.is_terminal());
    }

    #[test]
    fn test_calculate_progress_percentage() {
        let mut req = make_request();
        req.open().unwrap();

        // 0 signatures : 0% progress
        assert_eq!(req.calculate_progress_percentage(1.0), 0.0);

        // 5% des quotes-parts : 5% / 20% = 25% progress
        req.add_cosignatory(Uuid::new_v4(), 0.05).unwrap();
        assert!((req.calculate_progress_percentage(1.0) - 25.0).abs() < 1e-9);

        // 10% des quotes-parts : 10% / 20% = 50% progress
        req.add_cosignatory(Uuid::new_v4(), 0.05).unwrap();
        assert!((req.calculate_progress_percentage(1.0) - 50.0).abs() < 1e-9);

        // 20% des quotes-parts : 20% / 20% = 100% progress (seuil atteint)
        req.add_cosignatory(Uuid::new_v4(), 0.10).unwrap();
        assert_eq!(req.calculate_progress_percentage(1.0), 100.0);
    }
}
