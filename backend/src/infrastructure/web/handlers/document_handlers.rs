use crate::application::dto::{DocumentResponse, LinkDocumentToExpenseRequest, LinkDocumentToMeetingRequest};
use crate::domain::entities::DocumentType;
use crate::infrastructure::web::app_state::AppState;
use actix_multipart::Multipart;
use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use futures_util::StreamExt;
use serde::Deserialize;
use uuid::Uuid;

/// Upload a document with multipart/form-data
/// Expected fields:
/// - file: the file to upload
/// - building_id: UUID
/// - document_type: string (meeting_minutes, invoice, contract, etc.)
/// - title: string
/// - description: optional string
/// - uploaded_by: UUID
/// - related_meeting_id: optional UUID
/// - related_expense_id: optional UUID
#[post("/documents")]
pub async fn upload_document(
    app_state: web::Data<AppState>,
    mut payload: Multipart,
) -> impl Responder {
    let mut file_content: Option<Vec<u8>> = None;
    let mut filename: Option<String> = None;
    let mut mime_type: Option<String> = None;
    let mut building_id: Option<Uuid> = None;
    let mut document_type: Option<DocumentType> = None;
    let mut title: Option<String> = None;
    let mut description: Option<String> = None;
    let mut uploaded_by: Option<Uuid> = None;

    // Process multipart fields
    while let Some(item) = payload.next().await {
        let mut field = match item {
            Ok(field) => field,
            Err(e) => return HttpResponse::BadRequest().json(format!("Multipart error: {}", e)),
        };

        let field_name = field.name().to_string();

        match field_name.as_str() {
            "file" => {
                // Get filename and content type
                filename = field
                    .content_disposition()
                    .get_filename()
                    .map(|s| s.to_string());
                mime_type = field.content_type().map(|ct| ct.to_string());

                // Read file content
                let mut content = Vec::new();
                while let Some(chunk) = field.next().await {
                    match chunk {
                        Ok(data) => content.extend_from_slice(&data),
                        Err(e) => {
                            return HttpResponse::BadRequest()
                                .json(format!("Error reading file: {}", e))
                        }
                    }
                }
                file_content = Some(content);
            }
            "building_id" => {
                let mut value = Vec::new();
                while let Some(chunk) = field.next().await {
                    if let Ok(data) = chunk {
                        value.extend_from_slice(&data);
                    }
                }
                if let Ok(str_val) = String::from_utf8(value) {
                    building_id = Uuid::parse_str(str_val.trim()).ok();
                }
            }
            "document_type" => {
                let mut value = Vec::new();
                while let Some(chunk) = field.next().await {
                    if let Ok(data) = chunk {
                        value.extend_from_slice(&data);
                    }
                }
                if let Ok(str_val) = String::from_utf8(value) {
                    document_type = match str_val.trim() {
                        "meeting_minutes" | "MeetingMinutes" => Some(DocumentType::MeetingMinutes),
                        "financial_statement" | "FinancialStatement" => {
                            Some(DocumentType::FinancialStatement)
                        }
                        "invoice" | "Invoice" => Some(DocumentType::Invoice),
                        "contract" | "Contract" => Some(DocumentType::Contract),
                        "regulation" | "Regulation" => Some(DocumentType::Regulation),
                        "works_quote" | "WorksQuote" => Some(DocumentType::WorksQuote),
                        "other" | "Other" => Some(DocumentType::Other),
                        _ => None,
                    };
                }
            }
            "title" => {
                let mut value = Vec::new();
                while let Some(chunk) = field.next().await {
                    if let Ok(data) = chunk {
                        value.extend_from_slice(&data);
                    }
                }
                if let Ok(str_val) = String::from_utf8(value) {
                    title = Some(str_val.trim().to_string());
                }
            }
            "description" => {
                let mut value = Vec::new();
                while let Some(chunk) = field.next().await {
                    if let Ok(data) = chunk {
                        value.extend_from_slice(&data);
                    }
                }
                if let Ok(str_val) = String::from_utf8(value) {
                    let trimmed = str_val.trim();
                    if !trimmed.is_empty() {
                        description = Some(trimmed.to_string());
                    }
                }
            }
            "uploaded_by" => {
                let mut value = Vec::new();
                while let Some(chunk) = field.next().await {
                    if let Ok(data) = chunk {
                        value.extend_from_slice(&data);
                    }
                }
                if let Ok(str_val) = String::from_utf8(value) {
                    uploaded_by = Uuid::parse_str(str_val.trim()).ok();
                }
            }
            _ => {} // Ignore unknown fields
        }
    }

    // Validate required fields
    let file_content = match file_content {
        Some(content) => content,
        None => return HttpResponse::BadRequest().json("Missing file field"),
    };

    let filename = match filename {
        Some(name) => name,
        None => return HttpResponse::BadRequest().json("Missing filename"),
    };

    let mime_type = mime_type.unwrap_or_else(|| "application/octet-stream".to_string());

    let building_id = match building_id {
        Some(id) => id,
        None => return HttpResponse::BadRequest().json("Missing or invalid building_id"),
    };

    let document_type = match document_type {
        Some(dt) => dt,
        None => return HttpResponse::BadRequest().json("Missing or invalid document_type"),
    };

    let title = match title {
        Some(t) => t,
        None => return HttpResponse::BadRequest().json("Missing title"),
    };

    let uploaded_by = match uploaded_by {
        Some(id) => id,
        None => return HttpResponse::BadRequest().json("Missing or invalid uploaded_by"),
    };

    // Upload document
    match app_state
        .document_use_cases
        .upload_document(
            building_id,
            document_type,
            title,
            description,
            filename,
            file_content,
            mime_type,
            uploaded_by,
        )
        .await
    {
        Ok(document) => HttpResponse::Created().json(document),
        Err(e) => HttpResponse::InternalServerError().json(e),
    }
}

/// Get document metadata by ID
#[get("/documents/{id}")]
pub async fn get_document(
    app_state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let id = path.into_inner();

    match app_state.document_use_cases.get_document(id).await {
        Ok(document) => HttpResponse::Ok().json(document),
        Err(e) => HttpResponse::NotFound().json(e),
    }
}

/// Download document file
#[get("/documents/{id}/download")]
pub async fn download_document(
    app_state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let id = path.into_inner();

    match app_state.document_use_cases.download_document(id).await {
        Ok((content, mime_type, filename)) => HttpResponse::Ok()
            .content_type(mime_type)
            .insert_header(("Content-Disposition", format!("attachment; filename=\"{}\"", filename)))
            .body(content),
        Err(e) => HttpResponse::NotFound().json(e),
    }
}

/// List all documents for a building
#[get("/buildings/{building_id}/documents")]
pub async fn list_documents_by_building(
    app_state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let building_id = path.into_inner();

    match app_state
        .document_use_cases
        .list_documents_by_building(building_id)
        .await
    {
        Ok(documents) => HttpResponse::Ok().json(documents),
        Err(e) => HttpResponse::InternalServerError().json(e),
    }
}

/// List all documents for a meeting
#[get("/meetings/{meeting_id}/documents")]
pub async fn list_documents_by_meeting(
    app_state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let meeting_id = path.into_inner();

    match app_state
        .document_use_cases
        .list_documents_by_meeting(meeting_id)
        .await
    {
        Ok(documents) => HttpResponse::Ok().json(documents),
        Err(e) => HttpResponse::InternalServerError().json(e),
    }
}

/// Link document to a meeting
#[put("/documents/{id}/link-meeting")]
pub async fn link_document_to_meeting(
    app_state: web::Data<AppState>,
    path: web::Path<Uuid>,
    request: web::Json<LinkDocumentToMeetingRequest>,
) -> impl Responder {
    let id = path.into_inner();

    match app_state
        .document_use_cases
        .link_to_meeting(id, request.into_inner())
        .await
    {
        Ok(document) => HttpResponse::Ok().json(document),
        Err(e) => HttpResponse::NotFound().json(e),
    }
}

/// Link document to an expense
#[put("/documents/{id}/link-expense")]
pub async fn link_document_to_expense(
    app_state: web::Data<AppState>,
    path: web::Path<Uuid>,
    request: web::Json<LinkDocumentToExpenseRequest>,
) -> impl Responder {
    let id = path.into_inner();

    match app_state
        .document_use_cases
        .link_to_expense(id, request.into_inner())
        .await
    {
        Ok(document) => HttpResponse::Ok().json(document),
        Err(e) => HttpResponse::NotFound().json(e),
    }
}

/// Delete a document
#[delete("/documents/{id}")]
pub async fn delete_document(
    app_state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let id = path.into_inner();

    match app_state.document_use_cases.delete_document(id).await {
        Ok(true) => HttpResponse::NoContent().finish(),
        Ok(false) => HttpResponse::NotFound().json("Document not found"),
        Err(e) => HttpResponse::InternalServerError().json(e),
    }
}
