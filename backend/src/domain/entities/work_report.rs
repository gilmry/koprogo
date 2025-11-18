use chrono::{DateTime, Utc};
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
    pub work_date: DateTime<Utc>, // Date of work
    pub completion_date: Option<DateTime<Utc>>, // If different from work_date

    // Financial
    pub cost: f64,
    pub invoice_number: Option<String>,

    // Documentation
    pub photos: Vec<String>, // File paths to photos
    pub documents: Vec<String>, // File paths to related documents
    pub notes: Option<String>,

    // Warranty tracking
    pub warranty_type: WarrantyType,
    pub warranty_expiry: DateTime<Utc>,

    // Metadata
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum WorkType {
    Maintenance,       // Entretien régulier
    Repair,            // Réparation
    Renovation,        // Rénovation
    Emergency,         // Intervention d'urgence
    Inspection,        // Inspection avec travaux
    Installation,      // Installation nouvel équipement
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum WarrantyType {
    None,                          // Pas de garantie
    Standard,                      // 2 ans (vices apparents)
    Decennial,                     // 10 ans (garantie décennale)
    Extended,                      // Garantie étendue (matériel)
    Custom { years: i32 },         // Garantie personnalisée
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
        cost: f64,
        warranty_type: WarrantyType,
    ) -> Self {
        let now = Utc::now();

        // Calculate warranty expiry based on type
        let warranty_expiry = match warranty_type {
            WarrantyType::None => now, // No warranty
            WarrantyType::Standard => work_date + chrono::Duration::days(2 * 365), // 2 years
            WarrantyType::Decennial => work_date + chrono::Duration::days(10 * 365), // 10 years
            WarrantyType::Extended => work_date + chrono::Duration::days(3 * 365), // 3 years default
            WarrantyType::Custom { years } => work_date + chrono::Duration::days(years as i64 * 365),
        };

        Self {
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
        }
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

    #[test]
    fn test_work_report_creation() {
        let report = WorkReport::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "Réparation ascenseur".to_string(),
            "Remplacement câble principal".to_string(),
            WorkType::Repair,
            "Schindler Belgium".to_string(),
            Utc::now(),
            1500.0,
            WarrantyType::Standard,
        );

        assert_eq!(report.title, "Réparation ascenseur");
        assert_eq!(report.cost, 1500.0);
        assert!(report.is_warranty_valid());
        assert!(report.warranty_days_remaining() > 700); // ~2 years
    }

    #[test]
    fn test_decennial_warranty() {
        let report = WorkReport::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "Rénovation façade".to_string(),
            "Réfection complète façade".to_string(),
            WorkType::Renovation,
            "BatiPro SPRL".to_string(),
            Utc::now(),
            50000.0,
            WarrantyType::Decennial,
        );

        assert!(report.warranty_days_remaining() > 3600); // ~10 years
    }

    #[test]
    fn test_add_photos() {
        let mut report = WorkReport::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "Test".to_string(),
            "Test".to_string(),
            WorkType::Maintenance,
            "Test".to_string(),
            Utc::now(),
            100.0,
            WarrantyType::None,
        );

        report.add_photo("/uploads/photo1.jpg".to_string());
        report.add_photo("/uploads/photo2.jpg".to_string());

        assert_eq!(report.photos.len(), 2);
    }
}
