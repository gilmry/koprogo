use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Technical Inspection - Inspection technique obligatoire
///
/// Tracks mandatory technical inspections for building equipment and systems.
/// Belgian law requires regular inspections for safety-critical equipment.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TechnicalInspection {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub building_id: Uuid,

    // Inspection details
    pub inspection_type: InspectionType,
    pub title: String,
    pub description: Option<String>,

    // Inspector info
    pub inspector_name: String,
    pub inspector_company: Option<String>,
    pub inspector_certification: Option<String>, // Certification number

    // Dates
    pub inspection_date: DateTime<Utc>,
    pub next_due_date: DateTime<Utc>, // When next inspection is due

    // Results
    pub status: InspectionStatus,
    pub result_summary: Option<String>,
    pub defects_found: Option<String>,
    pub recommendations: Option<String>,

    // Compliance
    pub compliant: Option<bool>,
    pub compliance_certificate_number: Option<String>,
    pub compliance_valid_until: Option<DateTime<Utc>>,

    // Financial
    /// Coût de l'inspection en EUR. `Decimal` et non `f64` : montant
    /// refacturé via la répartition des charges (Art. 3.86 CC) —
    /// ADR-0007/0008 §A.
    pub cost: Option<Decimal>,
    pub invoice_number: Option<String>,

    // Documentation (JSON arrays of file paths)
    pub reports: Vec<String>,
    pub photos: Vec<String>,
    pub certificates: Vec<String>,
    pub notes: Option<String>,

    // Metadata
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum InspectionType {
    Elevator,               // Ascenseur (annuel)
    Boiler,                 // Chaudière (annuel)
    Electrical,             // Installation électrique (5 ans)
    FireExtinguisher,       // Extincteurs (annuel)
    FireAlarm,              // Système d'alarme incendie (annuel)
    GasInstallation,        // Installation gaz (annuel)
    RoofStructure,          // Structure toiture (5 ans)
    Facade,                 // Façade (quinquennal)
    WaterQuality,           // Qualité eau (annuel)
    Other { name: String }, // Autre type d'inspection
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum InspectionStatus {
    Scheduled,  // Planifiée
    InProgress, // En cours
    Completed,  // Terminée
    Failed,     // Échec (non conforme)
    Overdue,    // En retard
    Cancelled,  // Annulée
}

impl InspectionType {
    /// Get the required inspection frequency in days
    pub fn frequency_days(&self) -> i64 {
        match self {
            InspectionType::Elevator => 365,          // Annual
            InspectionType::Boiler => 365,            // Annual
            InspectionType::Electrical => 365 * 5,    // Every 5 years
            InspectionType::FireExtinguisher => 365,  // Annual
            InspectionType::FireAlarm => 365,         // Annual
            InspectionType::GasInstallation => 365,   // Annual
            InspectionType::RoofStructure => 365 * 5, // Every 5 years
            InspectionType::Facade => 365 * 5,        // Every 5 years
            InspectionType::WaterQuality => 365,      // Annual
            InspectionType::Other { .. } => 365,      // Default annual
        }
    }

    /// Get human-readable name
    pub fn display_name(&self) -> String {
        match self {
            InspectionType::Elevator => "Ascenseur".to_string(),
            InspectionType::Boiler => "Chaudière".to_string(),
            InspectionType::Electrical => "Installation électrique".to_string(),
            InspectionType::FireExtinguisher => "Extincteurs".to_string(),
            InspectionType::FireAlarm => "Alarme incendie".to_string(),
            InspectionType::GasInstallation => "Installation gaz".to_string(),
            InspectionType::RoofStructure => "Structure toiture".to_string(),
            InspectionType::Facade => "Façade".to_string(),
            InspectionType::WaterQuality => "Qualité de l'eau".to_string(),
            InspectionType::Other { name } => name.clone(),
        }
    }
}

/// Erreurs de validation du domaine `TechnicalInspection`.
///
/// Type domaine pur — aucune dépendance infra/application (pureté hexagonale).
#[derive(Debug, Clone, PartialEq)]
pub enum TechnicalInspectionError {
    /// Coût d'inspection strictement négatif.
    NegativeCost,
}

impl std::fmt::Display for TechnicalInspectionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NegativeCost => write!(f, "Technical inspection cost cannot be negative"),
        }
    }
}

impl std::error::Error for TechnicalInspectionError {}

/// Bridge : use-cases/ports `Result<_, String>` inchangés.
impl From<TechnicalInspectionError> for String {
    fn from(e: TechnicalInspectionError) -> String {
        e.to_string()
    }
}

impl TechnicalInspection {
    /// Pose le coût en portant l'invariant de non-négativité.
    ///
    /// `TechnicalInspection::new` ne prend pas de coût (il est renseigné plus
    /// tard, à la facturation) : l'invariant que portait
    /// `#[validate(range(min = 0.0))]` côté DTO se place donc ici, au seul
    /// point d'écriture, plutôt que de disparaître avec l'annotation.
    pub fn set_cost(&mut self, cost: Option<Decimal>) -> Result<(), TechnicalInspectionError> {
        if let Some(value) = cost {
            if value < Decimal::ZERO {
                return Err(TechnicalInspectionError::NegativeCost);
            }
        }
        self.cost = cost;
        self.updated_at = Utc::now();
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new(
        organization_id: Uuid,
        building_id: Uuid,
        title: String,
        description: Option<String>,
        inspection_type: InspectionType,
        inspector_name: String,
        inspection_date: DateTime<Utc>,
    ) -> Self {
        let now = Utc::now();

        // Calculate next due date based on inspection type
        let next_due_date =
            inspection_date + chrono::Duration::days(inspection_type.frequency_days());

        Self {
            id: Uuid::new_v4(),
            organization_id,
            building_id,
            inspection_type,
            title,
            description,
            inspector_name,
            inspector_company: None,
            inspector_certification: None,
            inspection_date,
            next_due_date,
            status: InspectionStatus::Scheduled,
            result_summary: None,
            defects_found: None,
            recommendations: None,
            compliant: None,
            compliance_certificate_number: None,
            compliance_valid_until: None,
            cost: None,
            invoice_number: None,
            reports: Vec::new(),
            photos: Vec::new(),
            certificates: Vec::new(),
            notes: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Calculate next due date based on inspection type
    pub fn calculate_next_due_date(&self) -> DateTime<Utc> {
        self.inspection_date + chrono::Duration::days(self.inspection_type.frequency_days())
    }

    /// Check if inspection is overdue
    pub fn is_overdue(&self) -> bool {
        Utc::now() > self.next_due_date
    }

    /// Get days until next inspection is due (negative if overdue)
    pub fn days_until_due(&self) -> i64 {
        (self.next_due_date - Utc::now()).num_days()
    }

    /// Mark as overdue
    pub fn mark_overdue(&mut self) {
        if self.is_overdue() && self.status == InspectionStatus::Scheduled {
            self.status = InspectionStatus::Overdue;
            self.updated_at = Utc::now();
        }
    }

    /// Add report to inspection
    pub fn add_report(&mut self, report_path: String) {
        self.reports.push(report_path);
        self.updated_at = Utc::now();
    }

    /// Add photo to inspection
    pub fn add_photo(&mut self, photo_path: String) {
        self.photos.push(photo_path);
        self.updated_at = Utc::now();
    }

    /// Add certificate to inspection
    pub fn add_certificate(&mut self, certificate_path: String) {
        self.certificates.push(certificate_path);
        self.updated_at = Utc::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_inspection_creation() {
        let inspection = TechnicalInspection::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "Inspection annuelle ascenseur".to_string(),
            Some("Vérification complète".to_string()),
            InspectionType::Elevator,
            "Schindler Belgium".to_string(),
            Utc::now(),
        );

        assert_eq!(inspection.title, "Inspection annuelle ascenseur");
        assert_eq!(inspection.status, InspectionStatus::Scheduled);
        assert!(!inspection.is_overdue());
    }

    #[test]
    fn test_inspection_frequencies() {
        assert_eq!(InspectionType::Elevator.frequency_days(), 365);
        assert_eq!(InspectionType::Electrical.frequency_days(), 365 * 5);
        assert_eq!(InspectionType::Facade.frequency_days(), 365 * 5);
    }

    #[test]
    fn test_inspection_completion() {
        let mut inspection = TechnicalInspection::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "Inspection chaudière".to_string(),
            None,
            InspectionType::Boiler,
            "Test Inspector".to_string(),
            Utc::now(),
        );

        inspection.status = InspectionStatus::Completed;
        inspection.compliant = Some(true);
        assert_eq!(inspection.status, InspectionStatus::Completed);
        assert_eq!(inspection.compliant, Some(true));
    }

    #[test]
    fn test_overdue_detection() {
        let past_date = Utc::now() - chrono::Duration::days(400); // Over a year ago
        let mut inspection = TechnicalInspection::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "Test".to_string(),
            None,
            InspectionType::FireExtinguisher,
            "Test".to_string(),
            past_date,
        );

        assert!(inspection.is_overdue());
        assert!(inspection.days_until_due() < 0);

        inspection.mark_overdue();
        assert_eq!(inspection.status, InspectionStatus::Overdue);
    }

    // ----- set_cost : ADR-0008, tests 4-cat -------------------------------

    fn make_inspection() -> TechnicalInspection {
        TechnicalInspection::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "Inspection annuelle ascenseur".to_string(),
            None,
            InspectionType::Elevator,
            "Schindler Belgium".to_string(),
            Utc::now(),
        )
    }

    /// @happy — le coût se pose et l'horodatage suit.
    #[test]
    fn happy_set_cost_records_the_amount() {
        let mut inspection = make_inspection();
        let before = inspection.updated_at;

        inspection
            .set_cost(Some(dec!(450.00)))
            .expect("coût valide");

        assert_eq!(inspection.cost, Some(dec!(450.00)));
        assert!(inspection.updated_at >= before);
    }

    /// @happy — `None` est légitime : inspection planifiée, pas encore facturée.
    #[test]
    fn happy_set_cost_none_is_accepted() {
        let mut inspection = make_inspection();
        inspection.set_cost(Some(dec!(10.00))).expect("coût valide");

        inspection.set_cost(None).expect("absence de coût valide");

        assert_eq!(inspection.cost, None);
    }

    /// @edge — zéro accepté (inspection sous contrat déjà réglé), le centime
    /// négatif refusé : la borne est à zéro exclu du côté négatif.
    #[test]
    fn edge_zero_accepted_minus_one_cent_rejected() {
        let mut inspection = make_inspection();

        inspection
            .set_cost(Some(Decimal::ZERO))
            .expect("zéro est un coût valide");
        assert_eq!(inspection.cost, Some(Decimal::ZERO));

        assert_eq!(
            inspection.set_cost(Some(dec!(-0.01))).unwrap_err(),
            TechnicalInspectionError::NegativeCost
        );
    }

    /// @edge — exactitude décimale, raison d'être de la conversion : en
    /// binary64 cette égalité est fausse.
    #[test]
    fn edge_decimal_arithmetic_is_exact() {
        let mut inspection = make_inspection();
        inspection.set_cost(Some(dec!(0.10))).expect("coût valide");

        let cumulated = inspection.cost.expect("coût posé") + dec!(0.20);
        inspection.set_cost(Some(cumulated)).expect("coût valide");

        assert_eq!(inspection.cost, Some(dec!(0.30)));
    }

    /// @negative — un refus ne laisse aucune écriture partielle derrière lui.
    #[test]
    fn negative_rejected_set_cost_leaves_the_entity_untouched() {
        let mut inspection = make_inspection();
        inspection
            .set_cost(Some(dec!(120.00)))
            .expect("coût valide");
        let before = inspection.updated_at;

        let _ = inspection.set_cost(Some(dec!(-5.00)));

        assert_eq!(inspection.cost, Some(dec!(120.00)));
        assert_eq!(inspection.updated_at, before);
    }

    /// @security — un coût négatif refacturé via la répartition des charges
    /// (Art. 3.86 CC) produirait un avoir au profit des copropriétaires depuis
    /// une simple fiche d'inspection. L'invariant tient dans le domaine, donc
    /// hors d'atteinte d'un contournement de la route HTTP.
    #[test]
    fn security_negative_cost_cannot_bypass_the_domain() {
        let mut inspection = make_inspection();

        assert!(inspection.set_cost(Some(dec!(-999999.99))).is_err());
        assert_eq!(inspection.cost, None);
    }
}
