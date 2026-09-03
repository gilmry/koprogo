use crate::application::dto::{
    CreateUnitDto, PageRequest, PageResponse, UnitResponseDto, UpdateUnitDto,
};
use crate::infrastructure::audit::{AuditEventType, AuditLogEntry};
use crate::infrastructure::web::middleware::scope_guard::verify_acp_org_access;
use crate::infrastructure::web::{AppState, AuthenticatedUser};
use actix_web::{delete, get, post, put, web, HttpResponse, Responder, ResponseError};
use uuid::Uuid;
use validator::Validate;
use crate::infrastructure::web::middleware::scope_guard::verify_building_org_access;

#[utoipa::path(
    post,
    path = "/units",
    tag = "Units",
    summary = "Create a unit (lot) inside a building",
    request_body = CreateUnitDto,
    responses(
        (status = 201, description = "Unit created", body = UnitResponseDto),
        (status = 400, description = "Validation error, or unknown field in the body"),
        (status = 403, description = "Forbidden (superadmin only — structural data)"),
        (status = 404, description = "Building not found"),
    ),
    security(("bearer_auth" = []))
)]
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

    if dto.building_id.is_empty() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "SuperAdmin must specify building_id"
        }));
    }

    let building_uuid = match Uuid::parse_str(&dto.building_id) {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid building ID format"
            }));
        }
    };

    // Story H15 — l'ACP d'un lot est celle de son building parent (#602).
    // Elle reste acceptée dans le corps pour ne rien casser chez les
    // appelants existants, mais elle n'est plus exigée : absente ou vide,
    // on la lit sur le building, qui en est la source de vérité.
    let acp_id = match dto
        .acp_id
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        Some(raw) => match Uuid::parse_str(raw) {
            Ok(id) => id,
            Err(_) => {
                return HttpResponse::BadRequest().json(serde_json::json!({
                    "error": "Invalid acp_id format"
                }));
            }
        },
        None => match state.building_use_cases.get_building(building_uuid).await {
            Ok(Some(building)) => match Uuid::parse_str(&building.acp_id) {
                Ok(id) => id,
                Err(_) => {
                    return HttpResponse::InternalServerError().json(serde_json::json!({
                        "error": "Invalid building.acp_id format"
                    }));
                }
            },
            Ok(None) => {
                return HttpResponse::NotFound().json(serde_json::json!({
                    "error": "Building not found"
                }));
            }
            Err(err) => {
                return HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": format!("Failed to resolve building ACP: {}", err)
                }));
            }
        },
    };

    // Story H15 — audit org context : units.organization_id ayant été DROP,
    // le scope org de l'audit vient du contexte utilisateur (cf. buildings).
    let organization_id = user.organization_id;

    if let Err(errors) = dto.validate() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Validation failed",
            "details": errors.to_string()
        }));
    }

    // L'ACP resolue ci-dessus fait foi : on la reinjecte dans le DTO pour que
    // le use case travaille sur une valeur toujours presente et bien formee.
    let mut dto = dto.into_inner();
    dto.acp_id = Some(acp_id.to_string());

    match state.unit_use_cases.create_unit(dto).await {
        Ok(unit) => {
            // Audit log: successful unit creation
            AuditLogEntry::new(
                AuditEventType::UnitCreated,
                Some(user.user_id),
                organization_id,
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
                organization_id,
            )
            .with_error(err.clone())
            .log();

            HttpResponse::BadRequest().json(serde_json::json!({
                "error": err
            }))
        }
    }
}

#[utoipa::path(
    get,
    path = "/units/{id}",
    tag = "Units",
    summary = "Get a single unit",
    params(("id" = Uuid, Path, description = "Unit identifier")),
    responses(
        (status = 200, description = "Unit", body = UnitResponseDto),
        (status = 404, description = "Unit not found"),
    ),
    security(("bearer_auth" = []))
)]
#[get("/units/{id}")]
pub async fn get_unit(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    id: web::Path<Uuid>,
) -> impl Responder {
    match state.unit_use_cases.get_unit(*id).await {
        Ok(Some(unit)) => {
            // Hotfix #603 — multi-tenant isolation via ACP→organization resolution.
            if let Ok(building_id) = Uuid::parse_str(&unit.building_id) {
                if let Ok(Some(building)) = state.building_use_cases.get_building(building_id).await
                {
                    let acp_id = match Uuid::parse_str(&building.acp_id) {
                        Ok(id) => id,
                        Err(_) => {
                            return HttpResponse::InternalServerError().json(serde_json::json!({
                                "error": "Invalid building.acp_id format"
                            }));
                        }
                    };
                    if let Err(err) =
                        verify_acp_org_access(&user, acp_id, &state.acp_use_cases).await
                    {
                        return err.error_response();
                    }
                }
            }
            HttpResponse::Ok().json(unit)
        }
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Unit not found"
        })),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": err
        })),
    }
}

#[utoipa::path(
    get,
    path = "/units",
    tag = "Units",
    summary = "List units visible to the authenticated user (paginated)",
    responses(
        (status = 200, description = "Units page"),
        (status = 401, description = "User does not belong to an organization"),
    ),
    security(("bearer_auth" = []))
)]
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

#[utoipa::path(
    get,
    path = "/buildings/{building_id}/units",
    tag = "Units",
    summary = "List the units of a building",
    params(("building_id" = Uuid, Path, description = "Building identifier")),
    responses(
        (status = 200, description = "Units", body = Vec<UnitResponseDto>),
        (status = 403, description = "Forbidden (building outside the user scope)"),
    ),
    security(("bearer_auth" = []))
)]
#[get("/buildings/{building_id}/units")]
pub async fn list_units_by_building(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    building_id: web::Path<Uuid>,
) -> impl Responder {
    // Hotfix #603 — multi-tenant isolation via ACP→organization resolution.
    match state.building_use_cases.get_building(*building_id).await {
        Ok(Some(building)) => {
            let acp_id = match Uuid::parse_str(&building.acp_id) {
                Ok(id) => id,
                Err(_) => {
                    return HttpResponse::InternalServerError().json(serde_json::json!({
                        "error": "Invalid building.acp_id format"
                    }));
                }
            };
            if let Err(err) = verify_acp_org_access(&user, acp_id, &state.acp_use_cases).await {
                return err.error_response();
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

#[utoipa::path(
    put,
    path = "/units/{id}",
    tag = "Units",
    summary = "Update a unit",
    description = "N'accepte PAS `owner_id` : la relation lot/proprietaire vit dans \
`unit_owners` (routes `/unit-owners`), qui porte les quotites et les dates de \
detention. `units.owner_id` est deprecie depuis la migration \
`20250127000000_refactor_owners_multitenancy`. Un corps portant `owner_id` \
recevait auparavant un 200 en jetant le champ ; il recoit desormais un 400.",
    params(("id" = Uuid, Path, description = "Unit identifier")),
    request_body = UpdateUnitDto,
    responses(
        (status = 200, description = "Unit updated", body = UnitResponseDto),
        (status = 400, description = "Validation error, or unknown field (e.g. `owner_id`)"),
        (status = 403, description = "Forbidden (superadmin only — quotites are structural)"),
        (status = 404, description = "Unit not found"),
    ),
    security(("bearer_auth" = []))
)]
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
                        // Hotfix #603 — branch is unreachable (superadmin-only guard
                        // above) but defensive verify_acp_org_access in case the
                        // guard is relaxed in the future.
                        let acp_id = match Uuid::parse_str(&building.acp_id) {
                            Ok(id) => id,
                            Err(_) => {
                                return HttpResponse::InternalServerError().json(
                                    serde_json::json!({
                                        "error": "Invalid building.acp_id format"
                                    }),
                                );
                            }
                        };
                        if let Err(err) =
                            verify_acp_org_access(&user, acp_id, &state.acp_use_cases).await
                        {
                            return err.error_response();
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

#[utoipa::path(
    delete,
    path = "/units/{id}",
    tag = "Units",
    summary = "Delete a unit",
    params(("id" = Uuid, Path, description = "Unit identifier")),
    responses(
        (status = 204, description = "Unit deleted"),
        (status = 403, description = "Forbidden (superadmin only)"),
        (status = 404, description = "Unit not found"),
    ),
    security(("bearer_auth" = []))
)]
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

#[utoipa::path(
    put,
    path = "/units/{unit_id}/assign-owner/{owner_id}",
    tag = "Units",
    summary = "Assign an owner to a unit",
    params(
        ("unit_id" = Uuid, Path, description = "Unit identifier"),
        ("owner_id" = Uuid, Path, description = "Owner identifier"),
    ),
    responses(
        (status = 200, description = "Owner assigned", body = UnitResponseDto),
        (status = 400, description = "Assignment refused by the domain"),
    ),
    security(("bearer_auth" = []))
)]
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
