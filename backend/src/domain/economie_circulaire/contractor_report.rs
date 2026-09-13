use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Statut d'un rapport de travaux corps de métier
///
/// Machine d'état BC16 (Backoffice Prestataires PWA) :
/// Draft → Submitted → UnderReview → Validated/Rejected/RequiresCorrection
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ContractorReportStatus {
    /// Corps de métier rédige le rapport (photos, pièces, compte-rendu)
    Draft,
    /// Soumis au conseil de copropriété pour validation
    Submitted,
    /// CdC examine le rapport
    UnderReview,
    /// CdC validé → déclenche le paiement automatique
    Validated,
    /// CdC refuse le rapport (avec motif)
    Rejected,
    /// CdC demande des corrections (motif + délai)
    RequiresCorrection,
}

impl ContractorReportStatus {
    pub fn from_db_string(s: &str) -> Result<Self, String> {
        match s {
            "draft" => Ok(Self::Draft),
            "submitted" => Ok(Self::Submitted),
            "under_review" => Ok(Self::UnderReview),
            "validated" => Ok(Self::Validated),
            "rejected" => Ok(Self::Rejected),
            "requires_correction" => Ok(Self::RequiresCorrection),
            _ => Err(format!("Unknown contractor_report_status: {}", s)),
        }
    }

    pub fn to_db_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Submitted => "submitted",
            Self::UnderReview => "under_review",
            Self::Validated => "validated",
            Self::Rejected => "rejected",
            Self::RequiresCorrection => "requires_correction",
        }
    }

    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Validated | Self::Rejected)
    }
}

/// Pièce remplacée lors des travaux
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReplacedPart {
    pub name: String,
    pub reference: Option<String>,
    pub quantity: u32,
    pub photo_document_id: Option<Uuid>,
}

/// Rapport de travaux soumis par le corps de métier via magic link PWA
///
/// Workflow :
/// 1. Ticket/Quote assigné → magic link JWT 72h envoyé au corps de métier
/// 2. Corps de métier : photos avant/après + pièces + compte-rendu → submit
/// 3. CdC : valide (→ paiement auto) ou demande corrections ou rejette
#[derive(Debug, Clone)]
pub struct ContractorReport {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub building_id: Uuid,

    /// Ticket ou devis associé (au moins l'un des deux doit être présent)
    pub ticket_id: Option<Uuid>,
    pub quote_id: Option<Uuid>,

    /// Id du prestataire (service_provider futur) ou nom libre
    pub contractor_user_id: Option<Uuid>,
    pub contractor_name: String,

    /// Date d'intervention
    pub work_date: Option<DateTime<Utc>>,

    /// Compte-rendu libre du corps de métier
    pub compte_rendu: Option<String>,

    /// Photos avant travaux (document_ids)
    pub photos_before: Vec<Uuid>,
    /// Photos après travaux (document_ids)
    pub photos_after: Vec<Uuid>,
    /// Pièces remplacées
    pub parts_replaced: Vec<ReplacedPart>,

    /// Statut de la machine d'état
    pub status: ContractorReportStatus,

    /// Magic link token (JWT hashé) pour accès sans auth classique
    pub magic_token_hash: Option<String>,
    pub magic_token_expires_at: Option<DateTime<Utc>>,

    /// Horodatage de soumission
    pub submitted_at: Option<DateTime<Utc>>,
    /// Validation CdC
    pub validated_at: Option<DateTime<Utc>>,
    pub validated_by: Option<Uuid>,
    /// Commentaires CdC (corrections ou refus)
    pub review_comments: Option<String>,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl ContractorReport {
    pub fn new(
        organization_id: Uuid,
        building_id: Uuid,
        contractor_name: String,
        ticket_id: Option<Uuid>,
        quote_id: Option<Uuid>,
        contractor_user_id: Option<Uuid>,
    ) -> Result<Self, String> {
        if contractor_name.trim().is_empty() {
            return Err("Le nom du prestataire est obligatoire".to_string());
        }
        if ticket_id.is_none() && quote_id.is_none() {
            return Err("Un rapport doit être lié à un ticket ou à un devis".to_string());
        }
        let now = Utc::now();
        Ok(Self {
            id: Uuid::new_v4(),
            organization_id,
            building_id,
            ticket_id,
            quote_id,
            contractor_user_id,
            contractor_name,
            work_date: None,
            compte_rendu: None,
            photos_before: vec![],
            photos_after: vec![],
            parts_replaced: vec![],
            status: ContractorReportStatus::Draft,
            magic_token_hash: None,
            magic_token_expires_at: None,
            submitted_at: None,
            validated_at: None,
            validated_by: None,
            review_comments: None,
            created_at: now,
            updated_at: now,
        })
    }

    /// Corps de métier soumet le rapport (Draft → Submitted)
    pub fn submit(&mut self) -> Result<(), String> {
        if self.status != ContractorReportStatus::Draft
            && self.status != ContractorReportStatus::RequiresCorrection
        {
            return Err(format!(
                "Impossible de soumettre depuis l'état {:?}",
                self.status
            ));
        }
        if self.compte_rendu.as_deref().unwrap_or("").trim().is_empty() {
            return Err(
                "Le champ compte_rendu est obligatoire pour soumettre le rapport".to_string(),
            );
        }
        self.status = ContractorReportStatus::Submitted;
        self.submitted_at = Some(Utc::now());
        self.updated_at = Utc::now();
        Ok(())
    }

    /// CdC commence l'examen (Submitted → UnderReview)
    pub fn start_review(&mut self) -> Result<(), String> {
        if self.status != ContractorReportStatus::Submitted {
            return Err(format!(
                "Impossible de mettre en révision depuis l'état {:?}",
                self.status
            ));
        }
        self.status = ContractorReportStatus::UnderReview;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// CdC valide le rapport (→ Validated, déclenche paiement)
    pub fn validate(&mut self, validated_by: Uuid) -> Result<(), String> {
        if self.status != ContractorReportStatus::Submitted
            && self.status != ContractorReportStatus::UnderReview
        {
            return Err(format!(
                "Impossible de valider depuis l'état {:?}",
                self.status
            ));
        }
        self.status = ContractorReportStatus::Validated;
        self.validated_at = Some(Utc::now());
        self.validated_by = Some(validated_by);
        self.updated_at = Utc::now();
        Ok(())
    }

    /// CdC demande des corrections (→ RequiresCorrection)
    pub fn request_corrections(&mut self, comments: String) -> Result<(), String> {
        if self.status != ContractorReportStatus::Submitted
            && self.status != ContractorReportStatus::UnderReview
        {
            return Err(format!(
                "Impossible de demander des corrections depuis l'état {:?}",
                self.status
            ));
        }
        if comments.trim().is_empty() {
            return Err("Les commentaires de correction sont obligatoires".to_string());
        }
        self.status = ContractorReportStatus::RequiresCorrection;
        self.review_comments = Some(comments);
        self.updated_at = Utc::now();
        Ok(())
    }

    /// CdC rejette le rapport
    pub fn reject(&mut self, comments: String, rejected_by: Uuid) -> Result<(), String> {
        if self.status.is_terminal() {
            return Err(format!(
                "Impossible de rejeter depuis l'état terminal {:?}",
                self.status
            ));
        }
        self.status = ContractorReportStatus::Rejected;
        self.review_comments = Some(comments);
        self.validated_by = Some(rejected_by);
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Vérifie si le magic token est encore valide
    pub fn is_magic_token_valid(&self) -> bool {
        match &self.magic_token_expires_at {
            Some(exp) => *exp > Utc::now(),
            None => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_report() -> ContractorReport {
        let mut r = ContractorReport::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "Martin Plomberie SPRL".to_string(),
            Some(Uuid::new_v4()),
            None,
            None,
        )
        .unwrap();
        r.compte_rendu = Some("Travaux effectués conformément au devis".to_string());
        r
    }

    #[test]
    fn test_new_report_success() {
        let r = make_report();
        assert_eq!(r.status, ContractorReportStatus::Draft);
        assert!(r.photos_before.is_empty());
    }

    #[test]
    fn test_new_requires_ticket_or_quote() {
        let err = ContractorReport::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "Test".to_string(),
            None,
            None,
            None,
        );
        assert!(err.is_err());
    }

    #[test]
    fn test_new_requires_contractor_name() {
        let err = ContractorReport::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "  ".to_string(),
            Some(Uuid::new_v4()),
            None,
            None,
        );
        assert!(err.is_err());
    }

    #[test]
    fn test_submit_from_draft() {
        let mut r = make_report();
        r.submit().unwrap();
        assert_eq!(r.status, ContractorReportStatus::Submitted);
        assert!(r.submitted_at.is_some());
    }

    #[test]
    fn test_submit_from_requires_correction() {
        let mut r = make_report();
        r.submit().unwrap();
        r.request_corrections("Manque photos avant".to_string())
            .unwrap();
        r.submit().unwrap();
        assert_eq!(r.status, ContractorReportStatus::Submitted);
    }

    #[test]
    fn test_validate_from_submitted() {
        let mut r = make_report();
        let cdc_id = Uuid::new_v4();
        r.submit().unwrap();
        r.validate(cdc_id).unwrap();
        assert_eq!(r.status, ContractorReportStatus::Validated);
        assert_eq!(r.validated_by, Some(cdc_id));
    }

    #[test]
    fn test_validate_from_under_review() {
        let mut r = make_report();
        let cdc_id = Uuid::new_v4();
        r.submit().unwrap();
        r.start_review().unwrap();
        r.validate(cdc_id).unwrap();
        assert_eq!(r.status, ContractorReportStatus::Validated);
    }

    #[test]
    fn test_cannot_validate_from_draft() {
        let mut r = make_report();
        assert!(r.validate(Uuid::new_v4()).is_err());
    }

    #[test]
    fn test_request_corrections_requires_comment() {
        let mut r = make_report();
        r.submit().unwrap();
        assert!(r.request_corrections("  ".to_string()).is_err());
    }

    #[test]
    fn test_request_corrections_ok() {
        let mut r = make_report();
        r.submit().unwrap();
        r.request_corrections("Ajoutez les photos après travaux".to_string())
            .unwrap();
        assert_eq!(r.status, ContractorReportStatus::RequiresCorrection);
        assert!(r.review_comments.is_some());
    }

    #[test]
    fn test_reject_from_submitted() {
        let mut r = make_report();
        r.submit().unwrap();
        r.reject("Travaux non conformes".to_string(), Uuid::new_v4())
            .unwrap();
        assert_eq!(r.status, ContractorReportStatus::Rejected);
    }

    #[test]
    fn test_cannot_reject_validated() {
        let mut r = make_report();
        r.submit().unwrap();
        r.validate(Uuid::new_v4()).unwrap();
        assert!(r.reject("Non".to_string(), Uuid::new_v4()).is_err());
    }

    #[test]
    fn test_magic_token_expired() {
        let mut r = make_report();
        r.magic_token_expires_at = Some(Utc::now() - chrono::Duration::hours(1));
        assert!(!r.is_magic_token_valid());
    }

    #[test]
    fn test_status_db_roundtrip() {
        let statuses = [
            ContractorReportStatus::Draft,
            ContractorReportStatus::Submitted,
            ContractorReportStatus::UnderReview,
            ContractorReportStatus::Validated,
            ContractorReportStatus::Rejected,
            ContractorReportStatus::RequiresCorrection,
        ];
        for s in &statuses {
            let db = s.to_db_str();
            let back = ContractorReportStatus::from_db_string(db).unwrap();
            assert_eq!(s, &back);
        }
    }
}
