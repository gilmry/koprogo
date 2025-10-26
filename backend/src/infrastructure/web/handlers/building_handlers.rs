use crate::application::dto::{CreateBuildingDto, PageRequest, PageResponse, UpdateBuildingDto};
use crate::infrastructure::audit::{AuditEventType, AuditLogEntry};
use crate::infrastructure::web::{AppState, AuthenticatedUser};
use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use uuid::Uuid;
use validator::Validate;

#[post("/buildings")]
pub async fn create_building(
    state: web::Data<AppState>,
    user: AuthenticatedUser, // JWT-extracted user info (SECURE!)
    mut dto: web::Json<CreateBuildingDto>,
) -> impl Responder {
    // SuperAdmin can create buildings for any organization
    // Regular users can only create for their own organization
    let organization_id: Uuid;

    if user.role == "superadmin" {
        // SuperAdmin: organization_id must be provided in DTO
        if dto.organization_id.is_empty() {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "SuperAdmin must specify organization_id"
            }));
        }
        // Parse the organization_id from DTO
        organization_id = match Uuid::parse_str(&dto.organization_id) {
            Ok(id) => id,
            Err(_) => {
                return HttpResponse::BadRequest().json(serde_json::json!({
                    "error": "Invalid organization_id format"
                }));
            }
        };
    } else {
        // Regular user: override organization_id from JWT token
        // This prevents users from creating buildings in other organizations
        organization_id = match user.require_organization() {
            Ok(org_id) => org_id,
            Err(e) => {
                return HttpResponse::Unauthorized().json(serde_json::json!({
                    "error": e.to_string()
                }))
            }
        };
        dto.organization_id = organization_id.to_string();
    }

    if let Err(errors) = dto.validate() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Validation failed",
            "details": errors.to_string()
        }));
    }

    match state
        .building_use_cases
        .create_building(dto.into_inner())
        .await
    {
        Ok(building) => {
            // Audit log: successful building creation
            AuditLogEntry::new(
                AuditEventType::BuildingCreated,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_resource("Building", Uuid::parse_str(&building.id).unwrap())
            .log();

            HttpResponse::Created().json(building)
        }
        Err(err) => {
            // Audit log: failed building creation
            AuditLogEntry::new(
                AuditEventType::BuildingCreated,
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

#[get("/buildings")]
pub async fn list_buildings(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    page_request: web::Query<PageRequest>,
) -> impl Responder {
    // Extract organization_id from authenticated user (secure!)
    let organization_id = user.organization_id;

    match state
        .building_use_cases
        .list_buildings_paginated(&page_request, organization_id)
        .await
    {
        Ok((buildings, total)) => {
            let response =
                PageResponse::new(buildings, page_request.page, page_request.per_page, total);
            HttpResponse::Ok().json(response)
        }
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": err
        })),
    }
}

#[get("/buildings/{id}")]
pub async fn get_building(state: web::Data<AppState>, id: web::Path<Uuid>) -> impl Responder {
    match state.building_use_cases.get_building(*id).await {
        Ok(Some(building)) => HttpResponse::Ok().json(building),
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Building not found"
        })),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": err
        })),
    }
}

#[put("/buildings/{id}")]
pub async fn update_building(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    id: web::Path<Uuid>,
    dto: web::Json<UpdateBuildingDto>,
) -> impl Responder {
    if let Err(errors) = dto.validate() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Validation failed",
            "details": errors.to_string()
        }));
    }

    // Only SuperAdmin can change organization_id
    if dto.organization_id.is_some() && user.role != "superadmin" {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Only SuperAdmins can change building organization"
        }));
    }

    // For non-SuperAdmins, verify they own the building
    if user.role != "superadmin" {
        match state.building_use_cases.get_building(*id).await {
            Ok(Some(building)) => {
                let building_org_id = match Uuid::parse_str(&building.organization_id) {
                    Ok(id) => id,
                    Err(_) => {
                        return HttpResponse::InternalServerError().json(serde_json::json!({
                            "error": "Invalid building organization_id"
                        }));
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
                        "error": "You can only update buildings in your own organization"
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

    match state
        .building_use_cases
        .update_building(*id, dto.into_inner())
        .await
    {
        Ok(building) => {
            // Audit log: successful building update
            AuditLogEntry::new(
                AuditEventType::BuildingUpdated,
                Some(user.user_id),
                user.organization_id,
            )
            .with_resource("Building", *id)
            .log();

            HttpResponse::Ok().json(building)
        }
        Err(err) => {
            // Audit log: failed building update
            AuditLogEntry::new(
                AuditEventType::BuildingUpdated,
                Some(user.user_id),
                user.organization_id,
            )
            .with_resource("Building", *id)
            .with_error(err.clone())
            .log();

            HttpResponse::BadRequest().json(serde_json::json!({
                "error": err
            }))
        }
    }
}

#[delete("/buildings/{id}")]
pub async fn delete_building(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    id: web::Path<Uuid>,
) -> impl Responder {
    match state.building_use_cases.delete_building(*id).await {
        Ok(true) => {
            // Audit log: successful building deletion
            AuditLogEntry::new(
                AuditEventType::BuildingDeleted,
                Some(user.user_id),
                user.organization_id,
            )
            .with_resource("Building", *id)
            .log();

            HttpResponse::NoContent().finish()
        }
        Ok(false) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Building not found"
        })),
        Err(err) => {
            // Audit log: failed building deletion
            AuditLogEntry::new(
                AuditEventType::BuildingDeleted,
                Some(user.user_id),
                user.organization_id,
            )
            .with_resource("Building", *id)
            .with_error(err.clone())
            .log();

            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": err
            }))
        }
    }
}
