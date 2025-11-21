use chrono::{DateTime, Utc};
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
    pub cost: Option<f64>,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum InspectionType {
    Elevator,           // Ascenseur (annuel)
    Boiler,             // Chaudière (annuel)
    Electrical,         // Installation électrique (5 ans)
    FireExtinguisher,   // Extincteurs (annuel)
    FireAlarm,          // Système d'alarme incendie (annuel)
    GasInstallation,    // Installation gaz (annuel)
    RoofStructure,      // Structure toiture (5 ans)
    Facade,             // Façade (quinquennal)
    WaterQuality,       // Qualité eau (annuel)
    Other { name: String }, // Autre type d'inspection
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum InspectionStatus {
    Scheduled,          // Planifiée
    InProgress,         // En cours
    Completed,          // Terminée
    Failed,             // Échec (non conforme)
    Overdue,            // En retard
    Cancelled,          // Annulée
}

impl InspectionType {
    /// Get the required inspection frequency in days
    pub fn frequency_days(&self) -> i64 {
        match self {
            InspectionType::Elevator => 365,           // Annual
            InspectionType::Boiler => 365,             // Annual
            InspectionType::Electrical => 365 * 5,     // Every 5 years
            InspectionType::FireExtinguisher => 365,   // Annual
            InspectionType::FireAlarm => 365,          // Annual
            InspectionType::GasInstallation => 365,    // Annual
            InspectionType::RoofStructure => 365 * 5,  // Every 5 years
            InspectionType::Facade => 365 * 5,         // Every 5 years
            InspectionType::WaterQuality => 365,       // Annual
            InspectionType::Other { .. } => 365,       // Default annual
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

impl TechnicalInspection {
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
        let next_due_date = inspection_date + chrono::Duration::days(inspection_type.frequency_days());

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
}
