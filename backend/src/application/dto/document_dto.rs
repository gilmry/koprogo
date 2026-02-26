use crate::domain::entities::{Document, DocumentType};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Response DTO for Document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentResponse {
    pub id: Uuid,
    pub building_id: Uuid,
    pub document_type: DocumentType,
    pub title: String,
    pub description: Option<String>,
    pub file_path: String,
    pub file_size: i64,
    pub file_size_mb: f64,
    pub mime_type: String,
    pub uploaded_by: Uuid,
    pub related_meeting_id: Option<Uuid>,
    pub related_expense_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Document> for DocumentResponse {
    fn from(doc: Document) -> Self {
        let file_size_mb = doc.file_size_mb();
        Self {
            id: doc.id,
            building_id: doc.building_id,
            document_type: doc.document_type,
            title: doc.title,
            description: doc.description,
            file_path: doc.file_path,
            file_size: doc.file_size,
            file_size_mb,
            mime_type: doc.mime_type,
            uploaded_by: doc.uploaded_by,
            related_meeting_id: doc.related_meeting_id,
            related_expense_id: doc.related_expense_id,
            created_at: doc.created_at,
            updated_at: doc.updated_at,
        }
    }
}

/// Request to upload a document
/// Note: File upload is handled via multipart/form-data, this is for metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadDocumentRequest {
    #[serde(default)]
    pub organization_id: Uuid, // Will be overridden by JWT token
    pub building_id: Uuid,
    pub document_type: DocumentType,
    pub title: String,
    pub description: Option<String>,
    pub uploaded_by: Uuid,
    pub related_meeting_id: Option<Uuid>,
    pub related_expense_id: Option<Uuid>,
}

/// Request to link document to meeting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkDocumentToMeetingRequest {
    pub meeting_id: Uuid,
}

/// Request to link document to expense
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkDocumentToExpenseRequest {
    pub expense_id: Uuid,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_document_response_from_entity() {
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let uploader_id = Uuid::new_v4();

        let document = Document::new(
            org_id,
            building_id,
            DocumentType::Invoice,
            "Test Invoice".to_string(),
            Some("Test description".to_string()),
            "/documents/test.pdf".to_string(),
            1048576, // 1 MB
            "application/pdf".to_string(),
            uploader_id,
        )
        .unwrap();

        let response = DocumentResponse::from(document.clone());

        assert_eq!(response.id, document.id);
        assert_eq!(response.title, "Test Invoice");
        assert_eq!(response.file_size, 1048576);
        assert_eq!(response.file_size_mb, 1.0);
    }

    #[test]
    fn test_upload_request_serialization() {
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let uploader_id = Uuid::new_v4();

        let request = UploadDocumentRequest {
            organization_id: org_id,
            building_id,
            document_type: DocumentType::Contract,
            title: "Contrat de syndic".to_string(),
            description: Some("Contrat annuel".to_string()),
            uploaded_by: uploader_id,
            related_meeting_id: None,
            related_expense_id: None,
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("Contrat de syndic"));

        let deserialized: UploadDocumentRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.title, request.title);
    }
}
