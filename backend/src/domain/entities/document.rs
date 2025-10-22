use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Type de document
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DocumentType {
    MeetingMinutes,     // Procès-verbal
    FinancialStatement, // Bilan financier
    Invoice,            // Facture
    Contract,           // Contrat
    Regulation,         // Règlement
    WorksQuote,         // Devis travaux
    Other,
}

/// Représente un document de copropriété
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Document {
    pub id: Uuid,
    pub building_id: Uuid,
    pub document_type: DocumentType,
    pub title: String,
    pub description: Option<String>,
    pub file_path: String,
    pub file_size: i64, // en bytes
    pub mime_type: String,
    pub uploaded_by: Uuid, // ID de l'utilisateur qui a uploadé
    pub related_meeting_id: Option<Uuid>,
    pub related_expense_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Document {
    pub fn new(
        building_id: Uuid,
        document_type: DocumentType,
        title: String,
        description: Option<String>,
        file_path: String,
        file_size: i64,
        mime_type: String,
        uploaded_by: Uuid,
    ) -> Result<Self, String> {
        if title.is_empty() {
            return Err("Title cannot be empty".to_string());
        }
        if file_path.is_empty() {
            return Err("File path cannot be empty".to_string());
        }
        if file_size <= 0 {
            return Err("File size must be greater than 0".to_string());
        }

        let now = Utc::now();
        Ok(Self {
            id: Uuid::new_v4(),
            building_id,
            document_type,
            title,
            description,
            file_path,
            file_size,
            mime_type,
            uploaded_by,
            related_meeting_id: None,
            related_expense_id: None,
            created_at: now,
            updated_at: now,
        })
    }

    pub fn link_to_meeting(&mut self, meeting_id: Uuid) {
        self.related_meeting_id = Some(meeting_id);
        self.updated_at = Utc::now();
    }

    pub fn link_to_expense(&mut self, expense_id: Uuid) {
        self.related_expense_id = Some(expense_id);
        self.updated_at = Utc::now();
    }

    pub fn file_size_mb(&self) -> f64 {
        self.file_size as f64 / 1_048_576.0 // Convertir bytes en MB
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_document_success() {
        let building_id = Uuid::new_v4();
        let uploader_id = Uuid::new_v4();

        let document = Document::new(
            building_id,
            DocumentType::MeetingMinutes,
            "PV AGO 2024".to_string(),
            Some("Procès-verbal de l'assemblée générale ordinaire 2024".to_string()),
            "/documents/pv-ago-2024.pdf".to_string(),
            1048576, // 1 MB
            "application/pdf".to_string(),
            uploader_id,
        );

        assert!(document.is_ok());
        let document = document.unwrap();
        assert_eq!(document.file_size_mb(), 1.0);
    }

    #[test]
    fn test_create_document_empty_title_fails() {
        let building_id = Uuid::new_v4();
        let uploader_id = Uuid::new_v4();

        let document = Document::new(
            building_id,
            DocumentType::Invoice,
            "".to_string(),
            None,
            "/documents/test.pdf".to_string(),
            1024,
            "application/pdf".to_string(),
            uploader_id,
        );

        assert!(document.is_err());
    }

    #[test]
    fn test_link_document_to_meeting() {
        let building_id = Uuid::new_v4();
        let uploader_id = Uuid::new_v4();

        let mut document = Document::new(
            building_id,
            DocumentType::MeetingMinutes,
            "Test".to_string(),
            None,
            "/test.pdf".to_string(),
            1024,
            "application/pdf".to_string(),
            uploader_id,
        ).unwrap();

        let meeting_id = Uuid::new_v4();
        document.link_to_meeting(meeting_id);

        assert_eq!(document.related_meeting_id, Some(meeting_id));
    }
}
