use crate::application::dto::{
    DocumentResponse, LinkDocumentToExpenseRequest, LinkDocumentToMeetingRequest,
};
use crate::application::ports::DocumentRepository;
use crate::domain::entities::{Document, DocumentType};
use crate::infrastructure::storage::FileStorage;
use std::sync::Arc;
use uuid::Uuid;

pub struct DocumentUseCases {
    repository: Arc<dyn DocumentRepository>,
    file_storage: FileStorage,
}

impl DocumentUseCases {
    pub fn new(repository: Arc<dyn DocumentRepository>, file_storage: FileStorage) -> Self {
        Self {
            repository,
            file_storage,
        }
    }

    /// Upload a document with file content
    pub async fn upload_document(
        &self,
        building_id: Uuid,
        document_type: DocumentType,
        title: String,
        description: Option<String>,
        filename: String,
        file_content: Vec<u8>,
        mime_type: String,
        uploaded_by: Uuid,
    ) -> Result<DocumentResponse, String> {
        // Validate file size (max 50MB)
        const MAX_FILE_SIZE: usize = 50 * 1024 * 1024; // 50MB
        if file_content.len() > MAX_FILE_SIZE {
            return Err("File size exceeds maximum limit of 50MB".to_string());
        }

        if file_content.is_empty() {
            return Err("File content cannot be empty".to_string());
        }

        // Save file to storage
        let file_path = self
            .file_storage
            .save_file(building_id, &filename, &file_content)
            .await?;

        // Create document entity
        let document = Document::new(
            building_id,
            document_type,
            title,
            description,
            file_path,
            file_content.len() as i64,
            mime_type,
            uploaded_by,
        )?;

        // Save to database
        let created_document = self.repository.create(&document).await?;

        Ok(DocumentResponse::from(created_document))
    }

    /// Get document metadata by ID
    pub async fn get_document(&self, id: Uuid) -> Result<DocumentResponse, String> {
        match self.repository.find_by_id(id).await? {
            Some(document) => Ok(DocumentResponse::from(document)),
            None => Err("Document not found".to_string()),
        }
    }

    /// Download document file content
    pub async fn download_document(&self, id: Uuid) -> Result<(Vec<u8>, String, String), String> {
        let document = match self.repository.find_by_id(id).await? {
            Some(doc) => doc,
            None => return Err("Document not found".to_string()),
        };

        // Read file from storage
        let content = self.file_storage.read_file(&document.file_path).await?;

        // Extract filename from path (last segment)
        let filename = document
            .file_path
            .split('/')
            .last()
            .unwrap_or("download")
            .to_string();

        Ok((content, document.mime_type, filename))
    }

    /// List all documents for a building
    pub async fn list_documents_by_building(
        &self,
        building_id: Uuid,
    ) -> Result<Vec<DocumentResponse>, String> {
        let documents = self.repository.find_by_building(building_id).await?;
        Ok(documents.into_iter().map(DocumentResponse::from).collect())
    }

    /// List all documents for a meeting
    pub async fn list_documents_by_meeting(
        &self,
        meeting_id: Uuid,
    ) -> Result<Vec<DocumentResponse>, String> {
        let documents = self.repository.find_by_meeting(meeting_id).await?;
        Ok(documents.into_iter().map(DocumentResponse::from).collect())
    }

    /// Link a document to a meeting
    pub async fn link_to_meeting(
        &self,
        id: Uuid,
        request: LinkDocumentToMeetingRequest,
    ) -> Result<DocumentResponse, String> {
        let mut document = match self.repository.find_by_id(id).await? {
            Some(doc) => doc,
            None => return Err("Document not found".to_string()),
        };

        document.link_to_meeting(request.meeting_id);

        let updated = self.repository.update(&document).await?;
        Ok(DocumentResponse::from(updated))
    }

    /// Link a document to an expense
    pub async fn link_to_expense(
        &self,
        id: Uuid,
        request: LinkDocumentToExpenseRequest,
    ) -> Result<DocumentResponse, String> {
        let mut document = match self.repository.find_by_id(id).await? {
            Some(doc) => doc,
            None => return Err("Document not found".to_string()),
        };

        document.link_to_expense(request.expense_id);

        let updated = self.repository.update(&document).await?;
        Ok(DocumentResponse::from(updated))
    }

    /// Delete a document (removes from database and file storage)
    pub async fn delete_document(&self, id: Uuid) -> Result<bool, String> {
        // Get document to retrieve file path
        let document = match self.repository.find_by_id(id).await? {
            Some(doc) => doc,
            None => return Err("Document not found".to_string()),
        };

        // Delete from database first
        let deleted = self.repository.delete(id).await?;

        if deleted {
            // Then delete file from storage (ignore errors if file doesn't exist)
            self.file_storage
                .delete_file(&document.file_path)
                .await
                .ok();
        }

        Ok(deleted)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::ports::DocumentRepository;
    use crate::domain::entities::{Document, DocumentType};
    use async_trait::async_trait;
    use std::env;
    use std::sync::Mutex;

    // Mock repository for testing
    struct MockDocumentRepository {
        documents: Mutex<Vec<Document>>,
    }

    impl MockDocumentRepository {
        fn new() -> Self {
            Self {
                documents: Mutex::new(Vec::new()),
            }
        }
    }

    #[async_trait]
    impl DocumentRepository for MockDocumentRepository {
        async fn create(&self, document: &Document) -> Result<Document, String> {
            let mut docs = self.documents.lock().unwrap();
            docs.push(document.clone());
            Ok(document.clone())
        }

        async fn find_by_id(&self, id: Uuid) -> Result<Option<Document>, String> {
            let docs = self.documents.lock().unwrap();
            Ok(docs.iter().find(|d| d.id == id).cloned())
        }

        async fn find_by_building(&self, building_id: Uuid) -> Result<Vec<Document>, String> {
            let docs = self.documents.lock().unwrap();
            Ok(docs
                .iter()
                .filter(|d| d.building_id == building_id)
                .cloned()
                .collect())
        }

        async fn find_by_meeting(&self, meeting_id: Uuid) -> Result<Vec<Document>, String> {
            let docs = self.documents.lock().unwrap();
            Ok(docs
                .iter()
                .filter(|d| d.related_meeting_id == Some(meeting_id))
                .cloned()
                .collect())
        }

        async fn update(&self, document: &Document) -> Result<Document, String> {
            let mut docs = self.documents.lock().unwrap();
            if let Some(pos) = docs.iter().position(|d| d.id == document.id) {
                docs[pos] = document.clone();
                Ok(document.clone())
            } else {
                Err("Document not found".to_string())
            }
        }

        async fn delete(&self, id: Uuid) -> Result<bool, String> {
            let mut docs = self.documents.lock().unwrap();
            if let Some(pos) = docs.iter().position(|d| d.id == id) {
                docs.remove(pos);
                Ok(true)
            } else {
                Ok(false)
            }
        }
    }

    #[tokio::test]
    async fn test_upload_document() {
        let temp_dir = env::temp_dir().join("koprogo_test_upload");
        let storage = FileStorage::new(&temp_dir).unwrap();
        let repo = Arc::new(MockDocumentRepository::new());
        let use_cases = DocumentUseCases::new(repo, storage);

        let building_id = Uuid::new_v4();
        let uploader_id = Uuid::new_v4();
        let content = b"Test PDF content".to_vec();

        let result = use_cases
            .upload_document(
                building_id,
                DocumentType::Invoice,
                "Test Invoice".to_string(),
                Some("Test description".to_string()),
                "invoice.pdf".to_string(),
                content.clone(),
                "application/pdf".to_string(),
                uploader_id,
            )
            .await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.title, "Test Invoice");
        assert_eq!(response.file_size, content.len() as i64);

        // Cleanup
        std::fs::remove_dir_all(&temp_dir).ok();
    }

    #[tokio::test]
    async fn test_upload_document_too_large() {
        let temp_dir = env::temp_dir().join("koprogo_test_large");
        let storage = FileStorage::new(&temp_dir).unwrap();
        let repo = Arc::new(MockDocumentRepository::new());
        let use_cases = DocumentUseCases::new(repo, storage);

        let building_id = Uuid::new_v4();
        let uploader_id = Uuid::new_v4();
        // Create content larger than 50MB
        let large_content = vec![0u8; 51 * 1024 * 1024];

        let result = use_cases
            .upload_document(
                building_id,
                DocumentType::Invoice,
                "Large File".to_string(),
                None,
                "large.pdf".to_string(),
                large_content,
                "application/pdf".to_string(),
                uploader_id,
            )
            .await;

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("File size exceeds maximum limit"));

        // Cleanup
        std::fs::remove_dir_all(&temp_dir).ok();
    }

    #[tokio::test]
    async fn test_link_document_to_meeting() {
        let temp_dir = env::temp_dir().join("koprogo_test_link");
        let storage = FileStorage::new(&temp_dir).unwrap();
        let repo = Arc::new(MockDocumentRepository::new());
        let use_cases = DocumentUseCases::new(repo, storage);

        let building_id = Uuid::new_v4();
        let uploader_id = Uuid::new_v4();
        let content = b"Test content".to_vec();

        // Upload document
        let doc = use_cases
            .upload_document(
                building_id,
                DocumentType::MeetingMinutes,
                "PV AGO".to_string(),
                None,
                "pv.pdf".to_string(),
                content,
                "application/pdf".to_string(),
                uploader_id,
            )
            .await
            .unwrap();

        // Link to meeting
        let meeting_id = Uuid::new_v4();
        let result = use_cases
            .link_to_meeting(doc.id, LinkDocumentToMeetingRequest { meeting_id })
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap().related_meeting_id, Some(meeting_id));

        // Cleanup
        std::fs::remove_dir_all(&temp_dir).ok();
    }
}
