use crate::application::dto::{CreateUnitDto, PageRequest, PageResponse};
use crate::infrastructure::audit::{AuditEventType, AuditLogEntry};
use crate::infrastructure::web::{AppState, AuthenticatedUser};
use actix_web::{get, post, put, web, HttpResponse, Responder};
use uuid::Uuid;
use validator::Validate;

#[post("/units")]
pub async fn create_unit(
    state: web::Data<AppState>,
    user: AuthenticatedUser, // JWT-extracted user info (SECURE!)
    mut dto: web::Json<CreateUnitDto>,
) -> impl Responder {
    // Override the organization_id from DTO with the one from JWT token
    // This prevents users from creating units in other organizations
    let organization_id = match user.require_organization() {
        Ok(org_id) => org_id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": e.to_string()
            }))
        }
    };
    dto.organization_id = organization_id.to_string();

    if let Err(errors) = dto.validate() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Validation failed",
            "details": errors.to_string()
        }));
    }

    match state.unit_use_cases.create_unit(dto.into_inner()).await {
        Ok(unit) => {
            // Audit log: successful unit creation
            AuditLogEntry::new(
                AuditEventType::UnitCreated,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_resource("Unit", Uuid::parse_str(&unit.id).unwrap())
            .log();

            HttpResponse::Created().json(unit)
        }
        Err(err) => {
            // Audit log: failed unit creation
            AuditLogEntry::new(
                AuditEventType::UnitCreated,
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

#[get("/units/{id}")]
pub async fn get_unit(state: web::Data<AppState>, id: web::Path<Uuid>) -> impl Responder {
    match state.unit_use_cases.get_unit(*id).await {
        Ok(Some(unit)) => HttpResponse::Ok().json(unit),
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Unit not found"
        })),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": err
        })),
    }
}

#[get("/units")]
pub async fn list_units(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    page_request: web::Query<PageRequest>,
) -> impl Responder {
    let organization_id = user.organization_id;

    match state
        .unit_use_cases
        .list_units_paginated(&page_request, organization_id)
        .await
    {
        Ok((units, total)) => {
            let response =
                PageResponse::new(units, page_request.page, page_request.per_page, total);
            HttpResponse::Ok().json(response)
        }
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": err
        })),
    }
}

#[get("/buildings/{building_id}/units")]
pub async fn list_units_by_building(
    state: web::Data<AppState>,
    building_id: web::Path<Uuid>,
) -> impl Responder {
    match state
        .unit_use_cases
        .list_units_by_building(*building_id)
        .await
    {
        Ok(units) => HttpResponse::Ok().json(units),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": err
        })),
    }
}

#[put("/units/{unit_id}/assign-owner/{owner_id}")]
pub async fn assign_owner(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    path: web::Path<(Uuid, Uuid)>,
) -> impl Responder {
    let (unit_id, owner_id) = path.into_inner();

    match state.unit_use_cases.assign_owner(unit_id, owner_id).await {
        Ok(unit) => {
            // Audit log: successful unit assignment
            AuditLogEntry::new(
                AuditEventType::UnitAssignedToOwner,
                Some(user.user_id),
                user.organization_id,
            )
            .with_resource("Unit", unit_id)
            .log();

            HttpResponse::Ok().json(unit)
        }
        Err(err) => {
            // Audit log: failed unit assignment
            AuditLogEntry::new(
                AuditEventType::UnitAssignedToOwner,
                Some(user.user_id),
                user.organization_id,
            )
            .with_resource("Unit", unit_id)
            .with_error(err.clone())
            .log();

            HttpResponse::BadRequest().json(serde_json::json!({
                "error": err
            }))
        }
    }
}
