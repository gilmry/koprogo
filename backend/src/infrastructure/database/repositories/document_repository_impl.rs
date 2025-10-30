use crate::application::ports::DocumentRepository;
use crate::domain::entities::{Document, DocumentType};
use crate::infrastructure::database::pool::DbPool;
use async_trait::async_trait;
use sqlx::Row;
use uuid::Uuid;

pub struct PostgresDocumentRepository {
    pool: DbPool,
}

impl PostgresDocumentRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl DocumentRepository for PostgresDocumentRepository {
    async fn create(&self, document: &Document) -> Result<Document, String> {
        let document_type_str = match document.document_type {
            DocumentType::MeetingMinutes => "meeting_minutes",
            DocumentType::FinancialStatement => "financial_statement",
            DocumentType::Invoice => "invoice",
            DocumentType::Contract => "contract",
            DocumentType::Regulation => "regulation",
            DocumentType::WorksQuote => "works_quote",
            DocumentType::Other => "other",
        };

        sqlx::query(
            r#"
            INSERT INTO documents (id, organization_id, building_id, document_type, title, description, file_path, file_size, mime_type, uploaded_by, related_meeting_id, related_expense_id, created_at, updated_at)
            VALUES ($1, $2, $3, CAST($4 AS document_type), $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            "#,
        )
        .bind(document.id)
        .bind(document.organization_id)
        .bind(document.building_id)
        .bind(document_type_str)
        .bind(&document.title)
        .bind(&document.description)
        .bind(&document.file_path)
        .bind(document.file_size)
        .bind(&document.mime_type)
        .bind(document.uploaded_by)
        .bind(document.related_meeting_id)
        .bind(document.related_expense_id)
        .bind(document.created_at)
        .bind(document.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(document.clone())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Document>, String> {
        let row = sqlx::query(
            r#"
            SELECT id, organization_id, building_id, document_type::text AS document_type, title, description, file_path, file_size, mime_type, uploaded_by, related_meeting_id, related_expense_id, created_at, updated_at
            FROM documents
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(row.map(|row| {
            let document_type_str: String = row.get("document_type");
            let document_type = match document_type_str.as_str() {
                "meeting_minutes" => DocumentType::MeetingMinutes,
                "financial_statement" => DocumentType::FinancialStatement,
                "invoice" => DocumentType::Invoice,
                "contract" => DocumentType::Contract,
                "regulation" => DocumentType::Regulation,
                "works_quote" => DocumentType::WorksQuote,
                _ => DocumentType::Other,
            };

            Document {
                id: row.get("id"),
                organization_id: row.get("organization_id"),
                building_id: row.get("building_id"),
                document_type,
                title: row.get("title"),
                description: row.get("description"),
                file_path: row.get("file_path"),
                file_size: row.get("file_size"),
                mime_type: row.get("mime_type"),
                uploaded_by: row.get("uploaded_by"),
                related_meeting_id: row.get("related_meeting_id"),
                related_expense_id: row.get("related_expense_id"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            }
        }))
    }

    async fn find_by_building(&self, building_id: Uuid) -> Result<Vec<Document>, String> {
        let rows = sqlx::query(
            r#"
            SELECT id, organization_id, building_id, document_type::text AS document_type, title, description, file_path, file_size, mime_type, uploaded_by, related_meeting_id, related_expense_id, created_at, updated_at
            FROM documents
            WHERE building_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(building_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(rows
            .iter()
            .map(|row| {
                let document_type_str: String = row.get("document_type");
                let document_type = match document_type_str.as_str() {
                    "meeting_minutes" => DocumentType::MeetingMinutes,
                    "financial_statement" => DocumentType::FinancialStatement,
                    "invoice" => DocumentType::Invoice,
                    "contract" => DocumentType::Contract,
                    "regulation" => DocumentType::Regulation,
                    "works_quote" => DocumentType::WorksQuote,
                    _ => DocumentType::Other,
                };

                Document {
                    id: row.get("id"),
                    organization_id: row.get("organization_id"),
                    building_id: row.get("building_id"),
                    document_type,
                    title: row.get("title"),
                    description: row.get("description"),
                    file_path: row.get("file_path"),
                    file_size: row.get("file_size"),
                    mime_type: row.get("mime_type"),
                    uploaded_by: row.get("uploaded_by"),
                    related_meeting_id: row.get("related_meeting_id"),
                    related_expense_id: row.get("related_expense_id"),
                    created_at: row.get("created_at"),
                    updated_at: row.get("updated_at"),
                }
            })
            .collect())
    }

    async fn find_by_meeting(&self, meeting_id: Uuid) -> Result<Vec<Document>, String> {
        let rows = sqlx::query(
            r#"
            SELECT id, organization_id, building_id, document_type::text AS document_type, title, description, file_path, file_size, mime_type, uploaded_by, related_meeting_id, related_expense_id, created_at, updated_at
            FROM documents
            WHERE related_meeting_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(meeting_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(rows
            .iter()
            .map(|row| {
                let document_type_str: String = row.get("document_type");
                let document_type = match document_type_str.as_str() {
                    "meeting_minutes" => DocumentType::MeetingMinutes,
                    "financial_statement" => DocumentType::FinancialStatement,
                    "invoice" => DocumentType::Invoice,
                    "contract" => DocumentType::Contract,
                    "regulation" => DocumentType::Regulation,
                    "works_quote" => DocumentType::WorksQuote,
                    _ => DocumentType::Other,
                };

                Document {
                    id: row.get("id"),
                    organization_id: row.get("organization_id"),
                    building_id: row.get("building_id"),
                    document_type,
                    title: row.get("title"),
                    description: row.get("description"),
                    file_path: row.get("file_path"),
                    file_size: row.get("file_size"),
                    mime_type: row.get("mime_type"),
                    uploaded_by: row.get("uploaded_by"),
                    related_meeting_id: row.get("related_meeting_id"),
                    related_expense_id: row.get("related_expense_id"),
                    created_at: row.get("created_at"),
                    updated_at: row.get("updated_at"),
                }
            })
            .collect())
    }

    async fn find_by_expense(&self, expense_id: Uuid) -> Result<Vec<Document>, String> {
        let rows = sqlx::query(
            r#"
            SELECT id, organization_id, building_id, document_type::text AS document_type, title, description, file_path, file_size, mime_type, uploaded_by, related_meeting_id, related_expense_id, created_at, updated_at
            FROM documents
            WHERE related_expense_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(expense_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(rows
            .iter()
            .map(|row| {
                let document_type_str: String = row.get("document_type");
                let document_type = match document_type_str.as_str() {
                    "meeting_minutes" => DocumentType::MeetingMinutes,
                    "financial_statement" => DocumentType::FinancialStatement,
                    "invoice" => DocumentType::Invoice,
                    "contract" => DocumentType::Contract,
                    "regulation" => DocumentType::Regulation,
                    "works_quote" => DocumentType::WorksQuote,
                    _ => DocumentType::Other,
                };

                Document {
                    id: row.get("id"),
                    organization_id: row.get("organization_id"),
                    building_id: row.get("building_id"),
                    document_type,
                    title: row.get("title"),
                    description: row.get("description"),
                    file_path: row.get("file_path"),
                    file_size: row.get("file_size"),
                    mime_type: row.get("mime_type"),
                    uploaded_by: row.get("uploaded_by"),
                    related_meeting_id: row.get("related_meeting_id"),
                    related_expense_id: row.get("related_expense_id"),
                    created_at: row.get("created_at"),
                    updated_at: row.get("updated_at"),
                }
            })
            .collect())
    }

    async fn update(&self, document: &Document) -> Result<Document, String> {
        sqlx::query(
            r#"
            UPDATE documents
            SET related_meeting_id = $2, related_expense_id = $3, updated_at = $4
            WHERE id = $1
            "#,
        )
        .bind(document.id)
        .bind(document.related_meeting_id)
        .bind(document.related_expense_id)
        .bind(document.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(document.clone())
    }

    async fn delete(&self, id: Uuid) -> Result<bool, String> {
        let result = sqlx::query("DELETE FROM documents WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("Database error: {}", e))?;

        Ok(result.rows_affected() > 0)
    }

    async fn find_all_paginated(
        &self,
        page_request: &crate::application::dto::PageRequest,
        organization_id: Option<Uuid>,
    ) -> Result<(Vec<Document>, i64), String> {
        // Validate page request
        page_request.validate()?;

        // Build WHERE clause
        let where_clause = if let Some(org_id) = organization_id {
            format!("WHERE organization_id = '{}'", org_id)
        } else {
            String::new()
        };

        // Count total items
        let count_query = format!("SELECT COUNT(*) FROM documents {}", where_clause);
        let total_items = sqlx::query_scalar::<_, i64>(&count_query)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| format!("Database error: {}", e))?;

        // Fetch paginated data
        let data_query = format!(
            "SELECT id, organization_id, building_id, document_type, title, description, file_path, file_size, mime_type, uploaded_by, related_meeting_id, related_expense_id, created_at, updated_at \
             FROM documents {} ORDER BY created_at DESC LIMIT {} OFFSET {}",
            where_clause,
            page_request.limit(),
            page_request.offset()
        );

        let rows = sqlx::query(&data_query)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| format!("Database error: {}", e))?;

        let documents: Vec<Document> = rows
            .iter()
            .map(|row| {
                let document_type_str: String = row
                    .try_get("document_type")
                    .unwrap_or_else(|_| "other".to_string());
                let document_type = match document_type_str.as_str() {
                    "meeting_minutes" => DocumentType::MeetingMinutes,
                    "financial_statement" => DocumentType::FinancialStatement,
                    "invoice" => DocumentType::Invoice,
                    "contract" => DocumentType::Contract,
                    "regulation" => DocumentType::Regulation,
                    "works_quote" => DocumentType::WorksQuote,
                    _ => DocumentType::Other,
                };

                Document {
                    id: row.get("id"),
                    organization_id: row.get("organization_id"),
                    building_id: row.get("building_id"),
                    document_type,
                    title: row.get("title"),
                    description: row.get("description"),
                    file_path: row.get("file_path"),
                    file_size: row.get("file_size"),
                    mime_type: row.get("mime_type"),
                    uploaded_by: row.get("uploaded_by"),
                    related_meeting_id: row.get("related_meeting_id"),
                    related_expense_id: row.get("related_expense_id"),
                    created_at: row.get("created_at"),
                    updated_at: row.get("updated_at"),
                }
            })
            .collect();

        Ok((documents, total_items))
    }
}
