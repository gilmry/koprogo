use crate::application::dto::{
    LinkDocumentToExpenseRequest, LinkDocumentToMeetingRequest, PageRequest, PageResponse,
};
use crate::domain::entities::DocumentType;
use crate::infrastructure::audit::{AuditEventType, AuditLogEntry};
use crate::infrastructure::web::{app_state::AppState, AuthenticatedUser};
use actix_multipart::form::{tempfile::TempFile, text::Text, MultipartForm};
use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use uuid::Uuid;

#[derive(Debug, MultipartForm)]
pub struct UploadForm {
    #[multipart(limit = "50MB")]
    file: TempFile,
    building_id: Text<String>,
    document_type: Text<String>,
    title: Text<String>,
    description: Option<Text<String>>,
    uploaded_by: Text<String>,
}

/// Upload a document with multipart/form-data
#[post("/documents")]
pub async fn upload_document(
    app_state: web::Data<AppState>,
    user: AuthenticatedUser, // JWT-extracted user info (SECURE!)
    MultipartForm(form): MultipartForm<UploadForm>,
) -> impl Responder {
    // Use organization_id from JWT token (SECURE - cannot be forged!)
    let organization_id = match user.require_organization() {
        Ok(org_id) => org_id,
        Err(e) => return HttpResponse::Unauthorized().json(serde_json::json!({
            "error": e.to_string()
        })),
    };

    // Parse building_id
    let building_id = match Uuid::parse_str(&form.building_id.0) {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().json("Invalid building_id"),
    };

    // Parse document_type
    let document_type = match form.document_type.0.as_str() {
        "meeting_minutes" | "MeetingMinutes" => DocumentType::MeetingMinutes,
        "financial_statement" | "FinancialStatement" => DocumentType::FinancialStatement,
        "invoice" | "Invoice" => DocumentType::Invoice,
        "contract" | "Contract" => DocumentType::Contract,
        "regulation" | "Regulation" => DocumentType::Regulation,
        "works_quote" | "WorksQuote" => DocumentType::WorksQuote,
        "other" | "Other" => DocumentType::Other,
        _ => return HttpResponse::BadRequest().json("Invalid document_type"),
    };

    // Parse uploaded_by
    let uploaded_by = match Uuid::parse_str(&form.uploaded_by.0) {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().json("Invalid uploaded_by"),
    };

    // Get file metadata
    let filename = form
        .file
        .file_name
        .clone()
        .unwrap_or_else(|| "unnamed".to_string());
    let mime_type = form
        .file
        .content_type
        .as_ref()
        .map(|ct| ct.to_string())
        .unwrap_or_else(|| "application/octet-stream".to_string());

    // Read file content
    let file_content = match std::fs::read(form.file.file.path()) {
        Ok(content) => content,
        Err(e) => {
            return HttpResponse::InternalServerError().json(format!("Failed to read file: {}", e))
        }
    };

    // Upload document
    match app_state
        .document_use_cases
        .upload_document(
            organization_id,
            building_id,
            document_type,
            form.title.0.clone(),
            form.description.map(|d| d.0),
            filename,
            file_content,
            mime_type,
            uploaded_by,
        )
        .await
    {
        Ok(document) => {
            // Audit log: successful document upload
            AuditLogEntry::new(
                AuditEventType::DocumentUploaded,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_resource("Document", document.id)
            .log();

            HttpResponse::Created().json(document)
        }
        Err(e) => {
            // Audit log: failed document upload
            AuditLogEntry::new(
                AuditEventType::DocumentUploaded,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_error(e.clone())
            .log();

            HttpResponse::InternalServerError().json(e)
        }
    }
}

/// Get document metadata by ID
#[get("/documents/{id}")]
pub async fn get_document(app_state: web::Data<AppState>, path: web::Path<Uuid>) -> impl Responder {
    let id = path.into_inner();

    match app_state.document_use_cases.get_document(id).await {
        Ok(document) => HttpResponse::Ok().json(document),
        Err(e) => HttpResponse::NotFound().json(e),
    }
}

/// List all documents with pagination
#[get("/documents")]
pub async fn list_documents(
    app_state: web::Data<AppState>,
    user: AuthenticatedUser,
    page_request: web::Query<PageRequest>,
) -> impl Responder {
    let organization_id = user.organization_id;

    match app_state
        .document_use_cases
        .list_documents_paginated(&page_request, organization_id)
        .await
    {
        Ok((documents, total)) => {
            let response = PageResponse::new(
                documents,
                page_request.page,
                page_request.per_page,
                total,
            );
            HttpResponse::Ok().json(response)
        }
        Err(e) => HttpResponse::InternalServerError().json(e),
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
            .insert_header((
                "Content-Disposition",
                format!("attachment; filename=\"{}\"", filename),
            ))
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
    user: AuthenticatedUser,
    path: web::Path<Uuid>,
) -> impl Responder {
    let id = path.into_inner();

    match app_state.document_use_cases.delete_document(id).await {
        Ok(true) => {
            // Audit log: successful document deletion
            AuditLogEntry::new(
                AuditEventType::DocumentDeleted,
                Some(user.user_id),
                user.organization_id,
            )
            .with_resource("Document", id)
            .log();

            HttpResponse::NoContent().finish()
        }
        Ok(false) => HttpResponse::NotFound().json("Document not found"),
        Err(e) => {
            // Audit log: failed document deletion
            AuditLogEntry::new(
                AuditEventType::DocumentDeleted,
                Some(user.user_id),
                user.organization_id,
            )
            .with_resource("Document", id)
            .with_error(e.clone())
            .log();

            HttpResponse::InternalServerError().json(e)
        }
    }
}
