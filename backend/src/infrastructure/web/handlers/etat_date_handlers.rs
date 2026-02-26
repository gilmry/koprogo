use crate::application::dto::{
    CreateEtatDateRequest, PageRequest, PageResponse, UpdateEtatDateAdditionalDataRequest,
    UpdateEtatDateFinancialRequest,
};
use crate::domain::entities::EtatDateStatus;
use crate::infrastructure::audit::{AuditEventType, AuditLogEntry};
use crate::infrastructure::web::{AppState, AuthenticatedUser};
use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct EtatDateListQuery {
    #[serde(default = "default_page")]
    pub page: i64,
    #[serde(default = "default_per_page")]
    pub per_page: i64,
    pub status: Option<String>,
}

fn default_page() -> i64 {
    1
}
fn default_per_page() -> i64 {
    10
}

/// Create a new état daté request
#[post("/etats-dates")]
pub async fn create_etat_date(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    mut request: web::Json<CreateEtatDateRequest>,
) -> impl Responder {
    // Override organization_id from JWT token (security)
    let organization_id = match user.require_organization() {
        Ok(org_id) => org_id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": e.to_string()
            }))
        }
    };
    request.organization_id = organization_id;

    match state
        .etat_date_use_cases
        .create_etat_date(request.into_inner())
        .await
    {
        Ok(etat_date) => {
            AuditLogEntry::new(
                AuditEventType::EtatDateCreated,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_resource("EtatDate", etat_date.id)
            .log();

            HttpResponse::Created().json(etat_date)
        }
        Err(err) => {
            AuditLogEntry::new(
                AuditEventType::EtatDateCreated,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_error(err.clone())
            .log();

            HttpResponse::BadRequest().json(serde_json::json!({
                "error": err
            }))
        }
    }
}

/// Get état daté by ID
#[get("/etats-dates/{id}")]
pub async fn get_etat_date(state: web::Data<AppState>, id: web::Path<Uuid>) -> impl Responder {
    match state.etat_date_use_cases.get_etat_date(*id).await {
        Ok(Some(etat_date)) => HttpResponse::Ok().json(etat_date),
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "État daté not found"
        })),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": err
        })),
    }
}

/// Get état daté by reference number
#[get("/etats-dates/reference/{reference_number}")]
pub async fn get_by_reference_number(
    state: web::Data<AppState>,
    reference_number: web::Path<String>,
) -> impl Responder {
    match state
        .etat_date_use_cases
        .get_by_reference_number(&reference_number)
        .await
    {
        Ok(Some(etat_date)) => HttpResponse::Ok().json(etat_date),
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "État daté not found"
        })),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": err
        })),
    }
}

/// List états datés paginated
#[get("/etats-dates")]
pub async fn list_etats_dates(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    query: web::Query<EtatDateListQuery>,
) -> impl Responder {
    let organization_id = user.organization_id;

    // Parse status filter
    let status = query.status.as_ref().and_then(|s| match s.as_str() {
        "requested" => Some(EtatDateStatus::Requested),
        "in_progress" => Some(EtatDateStatus::InProgress),
        "generated" => Some(EtatDateStatus::Generated),
        "delivered" => Some(EtatDateStatus::Delivered),
        "expired" => Some(EtatDateStatus::Expired),
        _ => None,
    });

    let page_request = PageRequest {
        page: query.page,
        per_page: query.per_page,
        sort_by: None,
        order: Default::default(),
    };

    match state
        .etat_date_use_cases
        .list_paginated(&page_request, organization_id, status)
        .await
    {
        Ok((etats, total)) => {
            let response =
                PageResponse::new(etats, page_request.page, page_request.per_page, total);
            HttpResponse::Ok().json(response)
        }
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": err
        })),
    }
}

/// List états datés by unit
#[get("/units/{unit_id}/etats-dates")]
pub async fn list_etats_dates_by_unit(
    state: web::Data<AppState>,
    unit_id: web::Path<Uuid>,
) -> impl Responder {
    match state.etat_date_use_cases.list_by_unit(*unit_id).await {
        Ok(etats) => HttpResponse::Ok().json(etats),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": err
        })),
    }
}

/// List états datés by building
#[get("/buildings/{building_id}/etats-dates")]
pub async fn list_etats_dates_by_building(
    state: web::Data<AppState>,
    building_id: web::Path<Uuid>,
) -> impl Responder {
    match state
        .etat_date_use_cases
        .list_by_building(*building_id)
        .await
    {
        Ok(etats) => HttpResponse::Ok().json(etats),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": err
        })),
    }
}

/// Mark état daté as in progress
#[put("/etats-dates/{id}/mark-in-progress")]
pub async fn mark_in_progress(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    id: web::Path<Uuid>,
) -> impl Responder {
    match state.etat_date_use_cases.mark_in_progress(*id).await {
        Ok(etat_date) => {
            AuditLogEntry::new(
                AuditEventType::EtatDateInProgress,
                Some(user.user_id),
                user.organization_id,
            )
            .with_resource("EtatDate", etat_date.id)
            .log();

            HttpResponse::Ok().json(etat_date)
        }
        Err(err) => HttpResponse::BadRequest().json(serde_json::json!({
            "error": err
        })),
    }
}

/// Mark état daté as generated (with PDF file path)
#[put("/etats-dates/{id}/mark-generated")]
pub async fn mark_generated(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    id: web::Path<Uuid>,
    pdf_path: web::Json<serde_json::Value>,
) -> impl Responder {
    let pdf_file_path = match pdf_path.get("pdf_file_path") {
        Some(serde_json::Value::String(path)) => path.clone(),
        _ => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "pdf_file_path is required as a string"
            }))
        }
    };

    match state
        .etat_date_use_cases
        .mark_generated(*id, pdf_file_path)
        .await
    {
        Ok(etat_date) => {
            AuditLogEntry::new(
                AuditEventType::EtatDateGenerated,
                Some(user.user_id),
                user.organization_id,
            )
            .with_resource("EtatDate", etat_date.id)
            .log();

            HttpResponse::Ok().json(etat_date)
        }
        Err(err) => HttpResponse::BadRequest().json(serde_json::json!({
            "error": err
        })),
    }
}

/// Mark état daté as delivered to notary
#[put("/etats-dates/{id}/mark-delivered")]
pub async fn mark_delivered(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    id: web::Path<Uuid>,
) -> impl Responder {
    match state.etat_date_use_cases.mark_delivered(*id).await {
        Ok(etat_date) => {
            AuditLogEntry::new(
                AuditEventType::EtatDateDelivered,
                Some(user.user_id),
                user.organization_id,
            )
            .with_resource("EtatDate", etat_date.id)
            .log();

            HttpResponse::Ok().json(etat_date)
        }
        Err(err) => HttpResponse::BadRequest().json(serde_json::json!({
            "error": err
        })),
    }
}

/// Update financial data
#[put("/etats-dates/{id}/financial")]
pub async fn update_financial_data(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    id: web::Path<Uuid>,
    request: web::Json<UpdateEtatDateFinancialRequest>,
) -> impl Responder {
    match state
        .etat_date_use_cases
        .update_financial_data(*id, request.into_inner())
        .await
    {
        Ok(etat_date) => {
            AuditLogEntry::new(
                AuditEventType::EtatDateFinancialUpdate,
                Some(user.user_id),
                user.organization_id,
            )
            .with_resource("EtatDate", etat_date.id)
            .log();

            HttpResponse::Ok().json(etat_date)
        }
        Err(err) => HttpResponse::BadRequest().json(serde_json::json!({
            "error": err
        })),
    }
}

/// Update additional data (sections 7-16)
#[put("/etats-dates/{id}/additional-data")]
pub async fn update_additional_data(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    id: web::Path<Uuid>,
    request: web::Json<UpdateEtatDateAdditionalDataRequest>,
) -> impl Responder {
    match state
        .etat_date_use_cases
        .update_additional_data(*id, request.into_inner())
        .await
    {
        Ok(etat_date) => {
            AuditLogEntry::new(
                AuditEventType::EtatDateAdditionalDataUpdate,
                Some(user.user_id),
                user.organization_id,
            )
            .with_resource("EtatDate", etat_date.id)
            .log();

            HttpResponse::Ok().json(etat_date)
        }
        Err(err) => HttpResponse::BadRequest().json(serde_json::json!({
            "error": err
        })),
    }
}

/// List overdue états datés (>10 days, not generated yet)
#[get("/etats-dates/overdue")]
pub async fn list_overdue(state: web::Data<AppState>, user: AuthenticatedUser) -> impl Responder {
    let organization_id = match user.require_organization() {
        Ok(org_id) => org_id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": e.to_string()
            }))
        }
    };

    match state
        .etat_date_use_cases
        .list_overdue(organization_id)
        .await
    {
        Ok(etats) => HttpResponse::Ok().json(etats),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": err
        })),
    }
}

/// List expired états datés (>3 months from reference date)
#[get("/etats-dates/expired")]
pub async fn list_expired(state: web::Data<AppState>, user: AuthenticatedUser) -> impl Responder {
    let organization_id = match user.require_organization() {
        Ok(org_id) => org_id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": e.to_string()
            }))
        }
    };

    match state
        .etat_date_use_cases
        .list_expired(organization_id)
        .await
    {
        Ok(etats) => HttpResponse::Ok().json(etats),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": err
        })),
    }
}

/// Get statistics for dashboard
#[get("/etats-dates/stats")]
pub async fn get_stats(state: web::Data<AppState>, user: AuthenticatedUser) -> impl Responder {
    let organization_id = match user.require_organization() {
        Ok(org_id) => org_id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": e.to_string()
            }))
        }
    };

    match state.etat_date_use_cases.get_stats(organization_id).await {
        Ok(stats) => HttpResponse::Ok().json(stats),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": err
        })),
    }
}

/// Delete état daté
#[delete("/etats-dates/{id}")]
pub async fn delete_etat_date(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    id: web::Path<Uuid>,
) -> impl Responder {
    match state.etat_date_use_cases.delete_etat_date(*id).await {
        Ok(true) => {
            AuditLogEntry::new(
                AuditEventType::EtatDateDeleted,
                Some(user.user_id),
                user.organization_id,
            )
            .with_resource("EtatDate", *id)
            .log();

            HttpResponse::NoContent().finish()
        }
        Ok(false) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "État daté not found"
        })),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": err
        })),
    }
}
