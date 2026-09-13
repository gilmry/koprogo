use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Attendance status for recipient
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, utoipa::ToSchema)]
pub enum AttendanceStatus {
    /// No response yet
    Pending,
    /// Will attend the meeting
    WillAttend,
    /// Will not attend
    WillNotAttend,
    /// Attended (marked after meeting)
    Attended,
    /// Did not attend (marked after meeting)
    DidNotAttend,
}

impl AttendanceStatus {
    pub fn to_db_string(&self) -> &'static str {
        match self {
            AttendanceStatus::Pending => "pending",
            AttendanceStatus::WillAttend => "will_attend",
            AttendanceStatus::WillNotAttend => "will_not_attend",
            AttendanceStatus::Attended => "attended",
            AttendanceStatus::DidNotAttend => "did_not_attend",
        }
    }

    pub fn from_db_string(s: &str) -> Result<Self, String> {
        match s {
            "pending" => Ok(AttendanceStatus::Pending),
            "will_attend" => Ok(AttendanceStatus::WillAttend),
            "will_not_attend" => Ok(AttendanceStatus::WillNotAttend),
            "attended" => Ok(AttendanceStatus::Attended),
            "did_not_attend" => Ok(AttendanceStatus::DidNotAttend),
            _ => Err(format!("Invalid attendance status: {}", s)),
        }
    }
}

/// Individual recipient of a convocation
///
/// Tracks delivery, opening, and attendance for each owner
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvocationRecipient {
    pub id: Uuid,
    pub convocation_id: Uuid,
    pub owner_id: Uuid,
    pub email: String,

    // Email tracking
    pub email_sent_at: Option<DateTime<Utc>>,
    pub email_opened_at: Option<DateTime<Utc>>, // Email read receipt
    pub email_failed: bool,
    pub email_failure_reason: Option<String>,

    // Reminder tracking
    pub reminder_sent_at: Option<DateTime<Utc>>,
    pub reminder_opened_at: Option<DateTime<Utc>>,

    // Attendance tracking
    pub attendance_status: AttendanceStatus,
    pub attendance_updated_at: Option<DateTime<Utc>>,

    // Proxy delegation (if owner delegates voting power)
    pub proxy_owner_id: Option<Uuid>, // Delegated to this owner

    // Audit
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Ce qu'est le mandataire pressenti, du point de vue de l'ACP.
///
/// Un booléen nu au site d'appel — `set_proxy(id, true)` — ne dit pas de quoi
/// il parle. Cette énumération force l'appelant à nommer ce qu'il a résolu, et
/// laissera place aux autres qualités si le besoin vient (le conjoint non
/// copropriétaire, par exemple, que l'Art. 3.87 § 7 traite différemment).
///
/// Le domaine ne sait pas *comment* on l'établit : c'est au cas d'usage de
/// remonter `owners.user_id` puis le rôle de cet utilisateur. La règle, elle,
/// reste ici.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QualiteDuMandataire {
    /// Un copropriétaire, ou toute personne que rien n'écarte.
    Coproprietaire,
    /// Le syndic en fonction de cette ACP.
    Syndic,
}

impl ConvocationRecipient {
    /// Create a new convocation recipient
    pub fn new(convocation_id: Uuid, owner_id: Uuid, email: String) -> Result<Self, String> {
        // Validate email
        if email.is_empty() || !email.contains('@') {
            return Err(format!("Invalid email address: {}", email));
        }

        let now = Utc::now();

        Ok(Self {
            id: Uuid::new_v4(),
            convocation_id,
            owner_id,
            email,
            email_sent_at: None,
            email_opened_at: None,
            email_failed: false,
            email_failure_reason: None,
            reminder_sent_at: None,
            reminder_opened_at: None,
            attendance_status: AttendanceStatus::Pending,
            attendance_updated_at: None,
            proxy_owner_id: None,
            created_at: now,
            updated_at: now,
        })
    }

    /// Mark email as sent
    pub fn mark_email_sent(&mut self) {
        self.email_sent_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }

    /// Mark email as failed
    pub fn mark_email_failed(&mut self, reason: String) {
        self.email_failed = true;
        self.email_failure_reason = Some(reason);
        self.updated_at = Utc::now();
    }

    /// Mark email as opened (read receipt)
    pub fn mark_email_opened(&mut self) -> Result<(), String> {
        if self.email_sent_at.is_none() {
            return Err("Cannot mark email as opened before it's sent".to_string());
        }

        if self.email_opened_at.is_some() {
            return Ok(()); // Already marked as opened, idempotent
        }

        self.email_opened_at = Some(Utc::now());
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Mark reminder as sent
    pub fn mark_reminder_sent(&mut self) -> Result<(), String> {
        if self.email_sent_at.is_none() {
            return Err("Cannot send reminder before initial email".to_string());
        }

        self.reminder_sent_at = Some(Utc::now());
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Mark reminder as opened
    pub fn mark_reminder_opened(&mut self) -> Result<(), String> {
        if self.reminder_sent_at.is_none() {
            return Err("Cannot mark reminder as opened before it's sent".to_string());
        }

        self.reminder_opened_at = Some(Utc::now());
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Update attendance status
    pub fn update_attendance_status(&mut self, status: AttendanceStatus) -> Result<(), String> {
        // Cannot change attendance after meeting (Attended/DidNotAttend is final)
        if matches!(
            self.attendance_status,
            AttendanceStatus::Attended | AttendanceStatus::DidNotAttend
        ) {
            return Err(format!(
                "Cannot change attendance after meeting. Current status: {:?}",
                self.attendance_status
            ));
        }

        self.attendance_status = status;
        self.attendance_updated_at = Some(Utc::now());
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Enregistre une procuration.
    ///
    /// # Art. 3.87 § 7, dernier alinéa
    ///
    /// > « Le syndic ne peut intervenir comme mandataire d'un copropriétaire à
    /// > l'assemblée générale, nonobstant le droit pour lui, s'il est
    /// > copropriétaire, de participer à ce titre aux délibérations. »
    ///
    /// La règle porte sur le **mandat**, pas sur le vote qui en découle : le
    /// texte interdit d'« intervenir comme mandataire ». On refuse donc ici, à
    /// l'enregistrement.
    ///
    /// C'est ce qui la distingue des deux autres règles du § 7 — le plafond de
    /// trois procurations et celui des voix. Celles-là ne *peuvent pas* se
    /// vérifier au moment du mandat : on ignore encore combien de procurations
    /// le mandataire recevra et quel poids elles pèseront. Elles restent donc
    /// dans `procurations.rs`, sur l'ensemble des voix d'une séance.
    ///
    /// L'enjeu est pratique. Une assemblée tenue sur un mandat prohibé est
    /// attaquable, et ce sont ses décisions — travaux, budgets, mandats — qui
    /// tombent avec elle. Refuser à l'enregistrement évite la séance entière ;
    /// refuser au dépouillement ne fait que constater les dégâts.
    ///
    /// La seconde partie de l'alinéa est respectée : rien n'empêche le syndic
    /// copropriétaire d'être **destinataire** et de voter pour lui-même. Ce
    /// qu'on refuse, c'est qu'il soit le mandataire d'un autre.
    pub fn set_proxy(
        &mut self,
        proxy_owner_id: Uuid,
        qualite: QualiteDuMandataire,
    ) -> Result<(), String> {
        if proxy_owner_id == self.owner_id {
            return Err("Cannot delegate to self".to_string());
        }

        if qualite == QualiteDuMandataire::Syndic {
            return Err(
                "Art. 3.87 § 7 : le syndic ne peut intervenir comme mandataire d'un copropriétaire"
                    .to_string(),
            );
        }

        self.proxy_owner_id = Some(proxy_owner_id);
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Remove proxy delegation
    pub fn remove_proxy(&mut self) {
        self.proxy_owner_id = None;
        self.updated_at = Utc::now();
    }

    /// Check if email was opened
    pub fn has_opened_email(&self) -> bool {
        self.email_opened_at.is_some()
    }

    /// Check if reminder was opened
    pub fn has_opened_reminder(&self) -> bool {
        self.reminder_opened_at.is_some()
    }

    /// Check if recipient needs reminder (email sent but not opened, no reminder sent yet)
    pub fn needs_reminder(&self) -> bool {
        self.email_sent_at.is_some()
            && self.email_opened_at.is_none()
            && self.reminder_sent_at.is_none()
            && !self.email_failed
    }

    /// Check if owner has confirmed attendance (either will attend or will not attend)
    pub fn has_confirmed_attendance(&self) -> bool {
        matches!(
            self.attendance_status,
            AttendanceStatus::WillAttend | AttendanceStatus::WillNotAttend
        )
    }

    /// Get days since email sent (if sent)
    pub fn days_since_email_sent(&self) -> Option<i64> {
        self.email_sent_at.map(|sent_at| {
            let now = Utc::now();
            now.signed_duration_since(sent_at).num_days()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_recipient_success() {
        let conv_id = Uuid::new_v4();
        let owner_id = Uuid::new_v4();

        let recipient =
            ConvocationRecipient::new(conv_id, owner_id, "owner@example.com".to_string());

        assert!(recipient.is_ok());
        let r = recipient.unwrap();
        assert_eq!(r.convocation_id, conv_id);
        assert_eq!(r.owner_id, owner_id);
        assert_eq!(r.email, "owner@example.com");
        assert_eq!(r.attendance_status, AttendanceStatus::Pending);
        assert!(!r.email_failed);
    }

    #[test]
    fn test_create_recipient_invalid_email() {
        let result =
            ConvocationRecipient::new(Uuid::new_v4(), Uuid::new_v4(), "invalid-email".to_string());

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid email"));
    }

    #[test]
    fn test_mark_email_opened() {
        let mut recipient = ConvocationRecipient::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "owner@example.com".to_string(),
        )
        .unwrap();

        // Cannot mark opened before sent
        assert!(recipient.mark_email_opened().is_err());

        // Mark sent first
        recipient.mark_email_sent();
        assert!(recipient.email_sent_at.is_some());

        // Now can mark opened
        assert!(recipient.mark_email_opened().is_ok());
        assert!(recipient.has_opened_email());

        // Idempotent
        assert!(recipient.mark_email_opened().is_ok());
    }

    #[test]
    fn test_mark_email_failed() {
        let mut recipient = ConvocationRecipient::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "owner@example.com".to_string(),
        )
        .unwrap();

        recipient.mark_email_failed("Invalid email address".to_string());

        assert!(recipient.email_failed);
        assert_eq!(
            recipient.email_failure_reason,
            Some("Invalid email address".to_string())
        );
    }

    #[test]
    fn test_needs_reminder() {
        let mut recipient = ConvocationRecipient::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "owner@example.com".to_string(),
        )
        .unwrap();

        // Not sent yet
        assert!(!recipient.needs_reminder());

        // Sent but not opened
        recipient.mark_email_sent();
        assert!(recipient.needs_reminder());

        // Opened
        recipient.mark_email_opened().unwrap();
        assert!(!recipient.needs_reminder());
    }

    #[test]
    fn test_update_attendance_status() {
        let mut recipient = ConvocationRecipient::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "owner@example.com".to_string(),
        )
        .unwrap();

        // Update to will attend
        assert!(recipient
            .update_attendance_status(AttendanceStatus::WillAttend)
            .is_ok());
        assert_eq!(recipient.attendance_status, AttendanceStatus::WillAttend);
        assert!(recipient.has_confirmed_attendance());

        // Change mind to will not attend
        assert!(recipient
            .update_attendance_status(AttendanceStatus::WillNotAttend)
            .is_ok());
        assert_eq!(recipient.attendance_status, AttendanceStatus::WillNotAttend);

        // Mark as attended (final)
        assert!(recipient
            .update_attendance_status(AttendanceStatus::Attended)
            .is_ok());

        // Cannot change after meeting
        assert!(recipient
            .update_attendance_status(AttendanceStatus::DidNotAttend)
            .is_err());
    }

    #[test]
    fn test_set_proxy() {
        let mut recipient = ConvocationRecipient::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "owner@example.com".to_string(),
        )
        .unwrap();

        let proxy_owner = Uuid::new_v4();

        // Set proxy
        assert!(recipient
            .set_proxy(proxy_owner, QualiteDuMandataire::Coproprietaire)
            .is_ok());
        assert_eq!(recipient.proxy_owner_id, Some(proxy_owner));

        // Cannot delegate to self
        assert!(recipient
            .set_proxy(recipient.owner_id, QualiteDuMandataire::Coproprietaire)
            .is_err());

        // Remove proxy
        recipient.remove_proxy();
        assert_eq!(recipient.proxy_owner_id, None);
    }

    /// Art. 3.87 § 7, dernier alinéa : le syndic ne peut intervenir comme
    /// mandataire d'un copropriétaire à l'assemblée générale.
    ///
    /// La règle porte sur le mandat lui-même, donc sur cet appel, et non
    /// seulement sur le vote qui en découlerait (#829).
    #[test]
    fn le_syndic_ne_peut_pas_recevoir_de_procuration() {
        let mut destinataire = ConvocationRecipient::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "coproprietaire@example.be".to_string(),
        )
        .unwrap();

        let syndic = Uuid::new_v4();
        let erreur = destinataire
            .set_proxy(syndic, QualiteDuMandataire::Syndic)
            .expect_err("le mandat au syndic doit être refusé");

        assert!(
            erreur.contains("3.87"),
            "le message doit citer l'article qui fonde le refus, reçu : {erreur}"
        );
        assert_eq!(
            destinataire.proxy_owner_id, None,
            "un mandat refusé ne doit rien laisser derrière lui"
        );
    }

    /// Le § 7 réserve expressément au syndic copropriétaire le droit de
    /// « participer à ce titre aux délibérations ». Être destinataire et voter
    /// pour soi-même reste donc permis : ce qu'on refuse, c'est qu'il soit le
    /// mandataire d'un AUTRE.
    ///
    /// Sans ce cas, la règle précédente passerait aussi avec une implémentation
    /// qui écarterait le syndic de toute la convocation.
    #[test]
    fn le_syndic_coproprietaire_reste_destinataire_a_part_entiere() {
        let syndic_coproprietaire = Uuid::new_v4();
        let mut destinataire = ConvocationRecipient::new(
            Uuid::new_v4(),
            syndic_coproprietaire,
            "syndic@example.be".to_string(),
        )
        .unwrap();

        assert!(destinataire
            .update_attendance_status(AttendanceStatus::WillAttend)
            .is_ok());

        // Et il peut donner procuration à un copropriétaire, comme n'importe
        // qui : l'interdiction porte sur le fait de la RECEVOIR.
        assert!(destinataire
            .set_proxy(Uuid::new_v4(), QualiteDuMandataire::Coproprietaire)
            .is_ok());
    }

    #[test]
    fn test_mark_reminder_sent() {
        let mut recipient = ConvocationRecipient::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "owner@example.com".to_string(),
        )
        .unwrap();

        // Cannot send reminder before initial email
        assert!(recipient.mark_reminder_sent().is_err());

        // Send initial email first
        recipient.mark_email_sent();

        // Now can send reminder
        assert!(recipient.mark_reminder_sent().is_ok());
        assert!(recipient.reminder_sent_at.is_some());
    }
}
