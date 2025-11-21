use crate::application::dto::{
    AddDocumentDto, AddPhotoDto, CreateWorkReportDto, PageRequest, UpdateWorkReportDto,
    WorkReportFilters,
};
use crate::infrastructure::audit::{AuditEventType, AuditLogEntry};
use crate::infrastructure::web::{AppState, AuthenticatedUser};
use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use uuid::Uuid;

// ==================== Work Report CRUD Endpoints ====================

/// Create a new work report
#[post("/work-reports")]
pub async fn create_work_report(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    request: web::Json<CreateWorkReportDto>,
) -> impl Responder {
    let organization_id = match user.require_organization() {
        Ok(org_id) => org_id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": e.to_string()}))
        }
    };

    match state
        .work_report_use_cases
        .create_work_report(request.into_inner())
        .await
    {
        Ok(work_report) => {
            AuditLogEntry::new(
                AuditEventType::WorkReportCreated,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_resource("WorkReport", work_report.id.parse().unwrap())
            .log();

            HttpResponse::Created().json(work_report)
        }
        Err(err) => HttpResponse::BadRequest().json(serde_json::json!({"error": err})),
    }
}

/// Get work report by ID
#[get("/work-reports/{id}")]
pub async fn get_work_report(state: web::Data<AppState>, id: web::Path<Uuid>) -> impl Responder {
    match state.work_report_use_cases.get_work_report(*id).await {
        Ok(Some(work_report)) => HttpResponse::Ok().json(work_report),
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Work report not found"
        })),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({"error": err})),
    }
}

/// List work reports by building
#[get("/buildings/{building_id}/work-reports")]
pub async fn list_building_work_reports(
    state: web::Data<AppState>,
    building_id: web::Path<Uuid>,
) -> impl Responder {
    match state
        .work_report_use_cases
        .list_work_reports_by_building(*building_id)
        .await
    {
        Ok(work_reports) => HttpResponse::Ok().json(work_reports),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({"error": err})),
    }
}

/// List work reports by organization
#[get("/organizations/{organization_id}/work-reports")]
pub async fn list_organization_work_reports(
    state: web::Data<AppState>,
    organization_id: web::Path<Uuid>,
) -> impl Responder {
    match state
        .work_report_use_cases
        .list_work_reports_by_organization(*organization_id)
        .await
    {
        Ok(work_reports) => HttpResponse::Ok().json(work_reports),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({"error": err})),
    }
}

/// List work reports with pagination and filters
#[get("/work-reports")]
pub async fn list_work_reports_paginated(
    state: web::Data<AppState>,
    page_request: web::Query<PageRequest>,
    filters: web::Query<WorkReportFilters>,
) -> impl Responder {
    match state
        .work_report_use_cases
        .list_work_reports_paginated(&page_request.into_inner(), &filters.into_inner())
        .await
    {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({"error": err})),
    }
}

/// Update work report
#[put("/work-reports/{id}")]
pub async fn update_work_report(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    id: web::Path<Uuid>,
    request: web::Json<UpdateWorkReportDto>,
) -> impl Responder {
    let organization_id = match user.require_organization() {
        Ok(org_id) => org_id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": e.to_string()}))
        }
    };

    match state
        .work_report_use_cases
        .update_work_report(*id, request.into_inner())
        .await
    {
        Ok(work_report) => {
            AuditLogEntry::new(
                AuditEventType::WorkReportUpdated,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_resource("WorkReport", *id)
            .log();

            HttpResponse::Ok().json(work_report)
        }
        Err(err) => HttpResponse::BadRequest().json(serde_json::json!({"error": err})),
    }
}

/// Delete work report
#[delete("/work-reports/{id}")]
pub async fn delete_work_report(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    id: web::Path<Uuid>,
) -> impl Responder {
    let organization_id = match user.require_organization() {
        Ok(org_id) => org_id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": e.to_string()}))
        }
    };

    match state.work_report_use_cases.delete_work_report(*id).await {
        Ok(deleted) => {
            if deleted {
                AuditLogEntry::new(
                    AuditEventType::WorkReportDeleted,
                    Some(user.user_id),
                    Some(organization_id),
                )
                .with_resource("WorkReport", *id)
                .log();

                HttpResponse::NoContent().finish()
            } else {
                HttpResponse::NotFound().json(serde_json::json!({
                    "error": "Work report not found"
                }))
            }
        }
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({"error": err})),
    }
}

// ==================== Warranty Management Endpoints ====================

/// Get active warranties for a building
#[get("/buildings/{building_id}/work-reports/warranties/active")]
pub async fn get_active_warranties(
    state: web::Data<AppState>,
    building_id: web::Path<Uuid>,
) -> impl Responder {
    match state
        .work_report_use_cases
        .get_active_warranties(*building_id)
        .await
    {
        Ok(warranties) => HttpResponse::Ok().json(warranties),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({"error": err})),
    }
}

/// Get expiring warranties for a building (within X days)
#[get("/buildings/{building_id}/work-reports/warranties/expiring")]
pub async fn get_expiring_warranties(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
    query: web::Query<serde_json::Value>,
) -> impl Responder {
    let building_id = path.into_inner();
    let days = query.get("days").and_then(|v| v.as_i64()).unwrap_or(90) as i32;

    match state
        .work_report_use_cases
        .get_expiring_warranties(building_id, days)
        .await
    {
        Ok(warranties) => HttpResponse::Ok().json(warranties),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({"error": err})),
    }
}

// ==================== Photo & Document Management ====================

/// Add photo to work report
#[post("/work-reports/{id}/photos")]
pub async fn add_photo(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    id: web::Path<Uuid>,
    request: web::Json<AddPhotoDto>,
) -> impl Responder {
    let organization_id = match user.require_organization() {
        Ok(org_id) => org_id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": e.to_string()}))
        }
    };

    match state
        .work_report_use_cases
        .add_photo(*id, request.into_inner())
        .await
    {
        Ok(work_report) => {
            AuditLogEntry::new(
                AuditEventType::WorkReportPhotoAdded,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_resource("WorkReport", *id)
            .log();

            HttpResponse::Ok().json(work_report)
        }
        Err(err) => HttpResponse::BadRequest().json(serde_json::json!({"error": err})),
    }
}

/// Add document to work report
#[post("/work-reports/{id}/documents")]
pub async fn add_document(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    id: web::Path<Uuid>,
    request: web::Json<AddDocumentDto>,
) -> impl Responder {
    let organization_id = match user.require_organization() {
        Ok(org_id) => org_id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": e.to_string()}))
        }
    };

    match state
        .work_report_use_cases
        .add_document(*id, request.into_inner())
        .await
    {
        Ok(work_report) => {
            AuditLogEntry::new(
                AuditEventType::WorkReportDocumentAdded,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_resource("WorkReport", *id)
            .log();

            HttpResponse::Ok().json(work_report)
        }
        Err(err) => HttpResponse::BadRequest().json(serde_json::json!({"error": err})),
    }
}
