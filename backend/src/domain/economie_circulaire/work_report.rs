use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Work Report - Rapport de travaux effectués
///
/// Tracks maintenance work, repairs, and renovations performed on the building.
/// Part of the digital maintenance logbook (Carnet d'Entretien).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorkReport {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub building_id: Uuid,

    // Work details
    pub title: String,
    pub description: String,
    pub work_type: WorkType,
    pub contractor_name: String,
    pub contractor_contact: Option<String>,

    // Dates
    pub work_date: DateTime<Utc>,               // Date of work
    pub completion_date: Option<DateTime<Utc>>, // If different from work_date

    // Financial
    /// Coût des travaux en EUR. `Decimal` et non `f64` : ce montant est
    /// refacturé aux copropriétaires via la répartition des charges
    /// (Art. 3.86 CC) et alimente le fonds de réserve — ADR-0007/0008 §A.
    pub cost: Decimal,
    pub invoice_number: Option<String>,

    // Documentation
    pub photos: Vec<String>,    // File paths to photos
    pub documents: Vec<String>, // File paths to related documents
    pub notes: Option<String>,

    // Warranty tracking
    pub warranty_type: WarrantyType,
    pub warranty_expiry: DateTime<Utc>,

    // Metadata
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum WorkType {
    Maintenance,  // Entretien régulier
    Repair,       // Réparation
    Renovation,   // Rénovation
    Emergency,    // Intervention d'urgence
    Inspection,   // Inspection avec travaux
    Installation, // Installation nouvel équipement
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum WarrantyType {
    None,                  // Pas de garantie
    Standard,              // 2 ans (vices apparents)
    Decennial,             // 10 ans (garantie décennale)
    Extended,              // Garantie étendue (matériel)
    Custom { years: i32 }, // Garantie personnalisée
}

/// Erreurs de validation du domaine `WorkReport`.
///
/// Type domaine pur — aucune dépendance infra/application (pureté hexagonale).
/// Précédent `CallForFundsError` / `OwnerContributionError` → 400 validation,
/// jamais 500 Internal.
#[derive(Debug, Clone, PartialEq)]
pub enum WorkReportError {
    /// Coût de travaux strictement négatif.
    NegativeCost,
}

impl std::fmt::Display for WorkReportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NegativeCost => write!(f, "Work report cost cannot be negative"),
        }
    }
}

impl std::error::Error for WorkReportError {}

/// Bridge : use-cases/ports `Result<_, String>` inchangés (cascade
/// String→AppError = slice large différée, précédent WP-A3/A4/A5).
impl From<WorkReportError> for String {
    fn from(e: WorkReportError) -> String {
        e.to_string()
    }
}

impl WorkReport {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        organization_id: Uuid,
        building_id: Uuid,
        title: String,
        description: String,
        work_type: WorkType,
        contractor_name: String,
        work_date: DateTime<Utc>,
        cost: Decimal,
        warranty_type: WarrantyType,
    ) -> Result<Self, WorkReportError> {
        // Reprise de l'invariant que portait `#[validate(range(min = 0.0))]`
        // côté DTO avant cette conversion : `validator` ne sait pas borner un
        // `Decimal`, la règle descend donc dans le domaine plutôt que de
        // disparaître. Elle s'applique désormais à tous les appelants, pas
        // seulement à la route HTTP — précédent `PaymentReminder::new`.
        if cost < Decimal::ZERO {
            return Err(WorkReportError::NegativeCost);
        }

        let now = Utc::now();

        // Calculate warranty expiry based on type
        let warranty_expiry = match warranty_type {
            WarrantyType::None => now, // No warranty
            WarrantyType::Standard => work_date + chrono::Duration::days(2 * 365), // 2 years
            WarrantyType::Decennial => work_date + chrono::Duration::days(10 * 365), // 10 years
            WarrantyType::Extended => work_date + chrono::Duration::days(3 * 365), // 3 years default
            WarrantyType::Custom { years } => {
                work_date + chrono::Duration::days(years as i64 * 365)
            }
        };

        Ok(Self {
            id: Uuid::new_v4(),
            organization_id,
            building_id,
            title,
            description,
            work_type,
            contractor_name,
            contractor_contact: None,
            work_date,
            completion_date: None,
            cost,
            invoice_number: None,
            photos: Vec::new(),
            documents: Vec::new(),
            notes: None,
            warranty_type,
            warranty_expiry,
            created_at: now,
            updated_at: now,
        })
    }

    /// Modifie le coût en portant l'invariant de non-négativité.
    ///
    /// Le chemin de mise à jour écrivait `work_report.cost = cost` directement :
    /// l'invariant du constructeur ne s'y appliquait pas. Il s'applique
    /// désormais aux deux points d'écriture.
    pub fn set_cost(&mut self, cost: Decimal) -> Result<(), WorkReportError> {
        if cost < Decimal::ZERO {
            return Err(WorkReportError::NegativeCost);
        }
        self.cost = cost;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Check if warranty is still valid
    pub fn is_warranty_valid(&self) -> bool {
        Utc::now() < self.warranty_expiry
    }

    /// Get remaining warranty days
    pub fn warranty_days_remaining(&self) -> i64 {
        let now = Utc::now();
        if now >= self.warranty_expiry {
            0
        } else {
            (self.warranty_expiry - now).num_days()
        }
    }

    /// Add photo to work report
    pub fn add_photo(&mut self, photo_path: String) {
        self.photos.push(photo_path);
        self.updated_at = Utc::now();
    }

    /// Add document to work report
    pub fn add_document(&mut self, document_path: String) {
        self.documents.push(document_path);
        self.updated_at = Utc::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn make(cost: Decimal, warranty: WarrantyType) -> Result<WorkReport, WorkReportError> {
        WorkReport::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "Réparation ascenseur".to_string(),
            "Remplacement câble principal".to_string(),
            WorkType::Repair,
            "Schindler Belgium".to_string(),
            Utc::now(),
            cost,
            warranty,
        )
    }

    // ----- @happy ---------------------------------------------------------

    #[test]
    fn happy_work_report_creation() {
        let report = make(dec!(1500.00), WarrantyType::Standard).expect("coût valide");

        assert_eq!(report.title, "Réparation ascenseur");
        assert_eq!(report.cost, dec!(1500.00));
        assert!(report.is_warranty_valid());
        assert!(report.warranty_days_remaining() > 700); // ~2 ans
    }

    #[test]
    fn happy_decennial_warranty() {
        let report = WorkReport::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "Rénovation façade".to_string(),
            "Réfection complète façade".to_string(),
            WorkType::Renovation,
            "BatiPro SPRL".to_string(),
            Utc::now(),
            dec!(50000.00),
            WarrantyType::Decennial,
        )
        .expect("coût valide");

        assert!(report.warranty_days_remaining() > 3600); // ~10 ans
    }

    #[test]
    fn happy_add_photos() {
        let mut report = make(dec!(100.00), WarrantyType::None).expect("coût valide");

        report.add_photo("/uploads/photo1.jpg".to_string());
        report.add_photo("/uploads/photo2.jpg".to_string());

        assert_eq!(report.photos.len(), 2);
    }

    #[test]
    fn happy_set_cost_replaces_the_amount() {
        let mut report = make(dec!(100.00), WarrantyType::None).expect("coût valide");

        report.set_cost(dec!(250.75)).expect("coût valide");

        assert_eq!(report.cost, dec!(250.75));
    }

    // ----- @edge ----------------------------------------------------------

    /// Borne inférieure : zéro est accepté (travaux sous garantie, refacturés
    /// à zéro), seul le strictement négatif est refusé.
    #[test]
    fn edge_zero_cost_is_accepted() {
        let report = make(Decimal::ZERO, WarrantyType::None).expect("zéro est un coût valide");
        assert_eq!(report.cost, Decimal::ZERO);
    }

    #[test]
    fn edge_set_cost_to_zero_is_accepted() {
        let mut report = make(dec!(10.00), WarrantyType::None).expect("coût valide");
        report
            .set_cost(Decimal::ZERO)
            .expect("zéro est un coût valide");
        assert_eq!(report.cost, Decimal::ZERO);
    }

    /// Le centime le plus proche de zéro par le bas reste refusé — la borne
    /// est bien à zéro exclu du côté négatif, pas « autour de zéro ».
    #[test]
    fn edge_minus_one_cent_is_rejected() {
        assert_eq!(
            make(dec!(-0.01), WarrantyType::None).unwrap_err(),
            WorkReportError::NegativeCost
        );
    }

    /// Exactitude décimale : c'est la raison d'être de la conversion. En
    /// binary64 cette égalité est fausse.
    #[test]
    fn edge_decimal_arithmetic_is_exact() {
        let mut report = make(dec!(0.10), WarrantyType::None).expect("coût valide");
        report
            .set_cost(report.cost + dec!(0.20))
            .expect("coût valide");

        assert_eq!(report.cost, dec!(0.30));
    }

    // ----- @negative ------------------------------------------------------

    #[test]
    fn negative_new_rejects_negative_cost() {
        assert_eq!(
            make(dec!(-1.00), WarrantyType::Standard).unwrap_err(),
            WorkReportError::NegativeCost
        );
    }

    #[test]
    fn negative_set_cost_rejects_negative_cost() {
        let mut report = make(dec!(100.00), WarrantyType::None).expect("coût valide");

        assert_eq!(
            report.set_cost(dec!(-0.01)).unwrap_err(),
            WorkReportError::NegativeCost
        );
    }

    /// Un `set_cost` refusé ne doit rien modifier — pas d'écriture partielle.
    #[test]
    fn negative_rejected_set_cost_leaves_the_entity_untouched() {
        let mut report = make(dec!(100.00), WarrantyType::None).expect("coût valide");
        let before = report.updated_at;

        let _ = report.set_cost(dec!(-5.00));

        assert_eq!(report.cost, dec!(100.00));
        assert_eq!(report.updated_at, before);
    }

    // ----- @security ------------------------------------------------------

    /// Un coût négatif est un vecteur d'abus : refacturé via la répartition
    /// des charges (Art. 3.86 CC), il produirait un AVOIR au profit des
    /// copropriétaires depuis un simple rapport de travaux. L'invariant est
    /// porté par le domaine, donc inaccessible en contournant la route HTTP.
    #[test]
    fn security_negative_cost_cannot_bypass_the_domain() {
        assert!(make(dec!(-999999.99), WarrantyType::Decennial).is_err());

        let mut report = make(dec!(1.00), WarrantyType::None).expect("coût valide");
        assert!(report.set_cost(dec!(-999999.99)).is_err());
        assert_eq!(report.cost, dec!(1.00));
    }
}
