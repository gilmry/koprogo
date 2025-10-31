use crate::application::dto::{CreateUnitDto, PageRequest, PageResponse, UpdateUnitDto};
use crate::infrastructure::audit::{AuditEventType, AuditLogEntry};
use crate::infrastructure::web::{AppState, AuthenticatedUser};
use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use uuid::Uuid;
use validator::Validate;

#[post("/units")]
pub async fn create_unit(
    state: web::Data<AppState>,
    user: AuthenticatedUser, // JWT-extracted user info (SECURE!)
    dto: web::Json<CreateUnitDto>,
) -> impl Responder {
    // Only SuperAdmin can create units (structural data)
    if user.role != "superadmin" {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Only SuperAdmin can create units (structural data cannot be modified after creation)"
        }));
    }

    // SuperAdmin must specify organization_id and building_id in the request body
    // Validate that both are provided
    if dto.organization_id.is_empty() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "SuperAdmin must specify organization_id"
        }));
    }

    if dto.building_id.is_empty() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "SuperAdmin must specify building_id"
        }));
    }

    // Parse organization_id for audit logging
    let organization_id = match Uuid::parse_str(&dto.organization_id) {
        Ok(org_id) => org_id,
        Err(_) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid organization_id format"
            }))
        }
    };

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

#[put("/units/{id}")]
pub async fn update_unit(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    id: web::Path<Uuid>,
    dto: web::Json<UpdateUnitDto>,
) -> impl Responder {
    // Only SuperAdmin can update units (structural data including quotités)
    if user.role != "superadmin" {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Only SuperAdmin can update units (structural data including quotités)"
        }));
    }

    if let Err(errors) = dto.validate() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Validation failed",
            "details": errors.to_string()
        }));
    }

    // Verify the user owns the unit (via building organization check)
    if user.role != "superadmin" {
        match state.unit_use_cases.get_unit(*id).await {
            Ok(Some(unit)) => {
                // Get the building to check organization
                let building_id = match Uuid::parse_str(&unit.building_id) {
                    Ok(id) => id,
                    Err(_) => {
                        return HttpResponse::InternalServerError().json(serde_json::json!({
                            "error": "Invalid building_id"
                        }));
                    }
                };

                match state.building_use_cases.get_building(building_id).await {
                    Ok(Some(building)) => {
                        let building_org_id = match Uuid::parse_str(&building.organization_id) {
                            Ok(id) => id,
                            Err(_) => {
                                return HttpResponse::InternalServerError().json(
                                    serde_json::json!({
                                        "error": "Invalid building organization_id"
                                    }),
                                );
                            }
                        };

                        let user_org_id = match user.require_organization() {
                            Ok(id) => id,
                            Err(e) => {
                                return HttpResponse::Unauthorized().json(serde_json::json!({
                                    "error": e.to_string()
                                }));
                            }
                        };

                        if building_org_id != user_org_id {
                            return HttpResponse::Forbidden().json(serde_json::json!({
                                "error": "You can only update units in your own organization"
                            }));
                        }
                    }
                    Ok(None) => {
                        return HttpResponse::NotFound().json(serde_json::json!({
                            "error": "Building not found"
                        }));
                    }
                    Err(err) => {
                        return HttpResponse::InternalServerError().json(serde_json::json!({
                            "error": err
                        }));
                    }
                }
            }
            Ok(None) => {
                return HttpResponse::NotFound().json(serde_json::json!({
                    "error": "Unit not found"
                }));
            }
            Err(err) => {
                return HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": err
                }));
            }
        }
    }

    match state
        .unit_use_cases
        .update_unit(*id, dto.into_inner())
        .await
    {
        Ok(unit) => {
            // Audit log: successful unit update
            AuditLogEntry::new(
                AuditEventType::UnitUpdated,
                Some(user.user_id),
                user.organization_id,
            )
            .with_resource("Unit", *id)
            .log();

            HttpResponse::Ok().json(unit)
        }
        Err(err) => {
            // Audit log: failed unit update
            AuditLogEntry::new(
                AuditEventType::UnitUpdated,
                Some(user.user_id),
                user.organization_id,
            )
            .with_resource("Unit", *id)
            .with_error(err.clone())
            .log();

            HttpResponse::BadRequest().json(serde_json::json!({
                "error": err
            }))
        }
    }
}

#[delete("/units/{id}")]
pub async fn delete_unit(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    id: web::Path<Uuid>,
) -> impl Responder {
    // Only SuperAdmin can delete units (structural data)
    if user.role != "superadmin" {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Only SuperAdmin can delete units (structural data)"
        }));
    }

    match state.unit_use_cases.delete_unit(*id).await {
        Ok(true) => {
            // Audit log: successful unit deletion
            AuditLogEntry::new(
                AuditEventType::UnitDeleted,
                Some(user.user_id),
                user.organization_id,
            )
            .with_resource("Unit", *id)
            .log();

            HttpResponse::Ok().json(serde_json::json!({
                "message": "Unit deleted successfully"
            }))
        }
        Ok(false) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Unit not found"
        })),
        Err(err) => {
            // Audit log: failed unit deletion
            AuditLogEntry::new(
                AuditEventType::UnitDeleted,
                Some(user.user_id),
                user.organization_id,
            )
            .with_resource("Unit", *id)
            .with_error(err.clone())
            .log();

            HttpResponse::BadRequest().json(serde_json::json!({
                "error": err
            }))
        }
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
