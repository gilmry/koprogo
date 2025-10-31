use crate::application::dto::{
    AddOwnerToUnitDto, TransferOwnershipDto, UnitOwnerResponseDto, UpdateOwnershipDto,
};
use crate::domain::entities::UnitOwner;
use crate::infrastructure::audit::{AuditEventType, AuditLogEntry};
use crate::infrastructure::web::{AppState, AuthenticatedUser};
use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use uuid::Uuid;
use validator::Validate;

/// Helper function to check if user role can modify unit ownership
/// Only SuperAdmin and Syndic can modify unit ownership (who owns what)
fn check_unit_ownership_permission(user: &AuthenticatedUser) -> Option<HttpResponse> {
    if user.role == "owner" || user.role == "accountant" {
        Some(HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Only SuperAdmin and Syndic can modify unit ownership"
        })))
    } else {
        None
    }
}

/// Add an owner to a unit
#[post("/units/{unit_id}/owners")]
pub async fn add_owner_to_unit(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    unit_id: web::Path<String>,
    dto: web::Json<AddOwnerToUnitDto>,
) -> impl Responder {
    if let Some(response) = check_unit_ownership_permission(&user) {
        return response;
    }

    // Validate DTO
    if let Err(errors) = dto.validate() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Validation failed",
            "details": errors.to_string()
        }));
    }

    // Parse UUIDs
    let unit_id = match Uuid::parse_str(&unit_id) {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid unit_id format"
            }))
        }
    };

    let owner_id = match Uuid::parse_str(&dto.owner_id) {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid owner_id format"
            }))
        }
    };

    // Call use case
    match state
        .unit_owner_use_cases
        .add_owner_to_unit(
            unit_id,
            owner_id,
            dto.ownership_percentage,
            dto.is_primary_contact,
        )
        .await
    {
        Ok(unit_owner) => {
            // Audit log
            if let Some(org_id) = user.organization_id {
                AuditLogEntry::new(
                    AuditEventType::UnitOwnerCreated,
                    Some(user.user_id),
                    Some(org_id),
                )
                .with_resource("UnitOwner", unit_owner.id)
                .log();
            }

            HttpResponse::Created().json(to_response_dto(&unit_owner))
        }
        Err(err) => {
            if let Some(org_id) = user.organization_id {
                AuditLogEntry::new(
                    AuditEventType::UnitOwnerCreated,
                    Some(user.user_id),
                    Some(org_id),
                )
                .with_error(err.clone())
                .log();
            }

            HttpResponse::BadRequest().json(serde_json::json!({
                "error": err
            }))
        }
    }
}

/// Remove an owner from a unit
#[delete("/units/{unit_id}/owners/{owner_id}")]
pub async fn remove_owner_from_unit(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    path: web::Path<(String, String)>,
) -> impl Responder {
    if let Some(response) = check_unit_ownership_permission(&user) {
        return response;
    }

    let (unit_id_str, owner_id_str) = path.into_inner();

    // Parse UUIDs
    let unit_id = match Uuid::parse_str(&unit_id_str) {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid unit_id format"
            }))
        }
    };

    let owner_id = match Uuid::parse_str(&owner_id_str) {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid owner_id format"
            }))
        }
    };

    // Call use case
    match state
        .unit_owner_use_cases
        .remove_owner_from_unit(unit_id, owner_id)
        .await
    {
        Ok(unit_owner) => {
            // Audit log
            if let Some(org_id) = user.organization_id {
                AuditLogEntry::new(
                    AuditEventType::UnitOwnerDeleted,
                    Some(user.user_id),
                    Some(org_id),
                )
                .with_resource("UnitOwner", unit_owner.id)
                .log();
            }

            HttpResponse::Ok().json(to_response_dto(&unit_owner))
        }
        Err(err) => HttpResponse::BadRequest().json(serde_json::json!({
            "error": err
        })),
    }
}

/// Update a unit-owner relationship (ownership percentage or primary contact)
#[put("/unit-owners/{id}")]
pub async fn update_unit_owner(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    id: web::Path<String>,
    dto: web::Json<UpdateOwnershipDto>,
) -> impl Responder {
    if let Some(response) = check_unit_ownership_permission(&user) {
        return response;
    }

    // Validate DTO
    if let Err(errors) = dto.validate() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Validation failed",
            "details": errors.to_string()
        }));
    }

    // Parse UUID
    let unit_owner_id = match Uuid::parse_str(&id) {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid unit_owner_id format"
            }))
        }
    };

    // Update ownership percentage if provided
    let result = if let Some(percentage) = dto.ownership_percentage {
        state
            .unit_owner_use_cases
            .update_ownership_percentage(unit_owner_id, percentage)
            .await
    } else if let Some(is_primary) = dto.is_primary_contact {
        if is_primary {
            state
                .unit_owner_use_cases
                .set_primary_contact(unit_owner_id)
                .await
        } else {
            // If unsetting primary, just get the current one and update it
            match state
                .unit_owner_use_cases
                .get_unit_owner(unit_owner_id)
                .await
            {
                Ok(Some(mut unit_owner)) => {
                    unit_owner.set_primary_contact(false);
                    // We'd need an update method in the repository, but for now return error
                    return HttpResponse::BadRequest().json(serde_json::json!({
                        "error": "Cannot unset primary contact directly. Set another owner as primary instead."
                    }));
                }
                Ok(None) => {
                    return HttpResponse::NotFound().json(serde_json::json!({
                        "error": "Unit-owner relationship not found"
                    }))
                }
                Err(err) => {
                    return HttpResponse::InternalServerError().json(serde_json::json!({
                        "error": err
                    }))
                }
            }
        }
    } else {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Must provide either ownership_percentage or is_primary_contact"
        }));
    };

    match result {
        Ok(unit_owner) => {
            // Audit log
            if let Some(org_id) = user.organization_id {
                AuditLogEntry::new(
                    AuditEventType::UnitOwnerUpdated,
                    Some(user.user_id),
                    Some(org_id),
                )
                .with_resource("UnitOwner", unit_owner.id)
                .log();
            }

            HttpResponse::Ok().json(to_response_dto(&unit_owner))
        }
        Err(err) => HttpResponse::BadRequest().json(serde_json::json!({
            "error": err
        })),
    }
}

/// Get all current owners of a unit
#[get("/units/{unit_id}/owners")]
pub async fn get_unit_owners(
    state: web::Data<AppState>,
    _user: AuthenticatedUser,
    unit_id: web::Path<String>,
) -> impl Responder {
    // Parse UUID
    let unit_id = match Uuid::parse_str(&unit_id) {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid unit_id format"
            }))
        }
    };

    // Call use case
    match state.unit_owner_use_cases.get_unit_owners(unit_id).await {
        Ok(unit_owners) => {
            let dtos: Vec<UnitOwnerResponseDto> = unit_owners.iter().map(to_response_dto).collect();
            HttpResponse::Ok().json(dtos)
        }
        Err(err) => HttpResponse::BadRequest().json(serde_json::json!({
            "error": err
        })),
    }
}

/// Get all current units owned by an owner
#[get("/owners/{owner_id}/units")]
pub async fn get_owner_units(
    state: web::Data<AppState>,
    _user: AuthenticatedUser,
    owner_id: web::Path<String>,
) -> impl Responder {
    // Parse UUID
    let owner_id = match Uuid::parse_str(&owner_id) {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid owner_id format"
            }))
        }
    };

    // Call use case
    match state.unit_owner_use_cases.get_owner_units(owner_id).await {
        Ok(unit_owners) => {
            let dtos: Vec<UnitOwnerResponseDto> = unit_owners.iter().map(to_response_dto).collect();
            HttpResponse::Ok().json(dtos)
        }
        Err(err) => HttpResponse::BadRequest().json(serde_json::json!({
            "error": err
        })),
    }
}

/// Get ownership history for a unit
#[get("/units/{unit_id}/owners/history")]
pub async fn get_unit_ownership_history(
    state: web::Data<AppState>,
    _user: AuthenticatedUser,
    unit_id: web::Path<String>,
) -> impl Responder {
    // Parse UUID
    let unit_id = match Uuid::parse_str(&unit_id) {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid unit_id format"
            }))
        }
    };

    // Call use case
    match state
        .unit_owner_use_cases
        .get_unit_ownership_history(unit_id)
        .await
    {
        Ok(unit_owners) => {
            let dtos: Vec<UnitOwnerResponseDto> = unit_owners.iter().map(to_response_dto).collect();
            HttpResponse::Ok().json(dtos)
        }
        Err(err) => HttpResponse::BadRequest().json(serde_json::json!({
            "error": err
        })),
    }
}

/// Get ownership history for an owner
#[get("/owners/{owner_id}/units/history")]
pub async fn get_owner_ownership_history(
    state: web::Data<AppState>,
    _user: AuthenticatedUser,
    owner_id: web::Path<String>,
) -> impl Responder {
    // Parse UUID
    let owner_id = match Uuid::parse_str(&owner_id) {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid owner_id format"
            }))
        }
    };

    // Call use case
    match state
        .unit_owner_use_cases
        .get_owner_ownership_history(owner_id)
        .await
    {
        Ok(unit_owners) => {
            let dtos: Vec<UnitOwnerResponseDto> = unit_owners.iter().map(to_response_dto).collect();
            HttpResponse::Ok().json(dtos)
        }
        Err(err) => HttpResponse::BadRequest().json(serde_json::json!({
            "error": err
        })),
    }
}

/// Transfer ownership from one owner to another
#[post("/units/{unit_id}/owners/transfer")]
pub async fn transfer_ownership(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    unit_id: web::Path<String>,
    dto: web::Json<TransferOwnershipDto>,
) -> impl Responder {
    if let Some(response) = check_unit_ownership_permission(&user) {
        return response;
    }

    // Validate DTO
    if let Err(errors) = dto.validate() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Validation failed",
            "details": errors.to_string()
        }));
    }

    // Parse UUIDs
    let unit_id = match Uuid::parse_str(&unit_id) {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid unit_id format"
            }))
        }
    };

    let from_owner_id = match Uuid::parse_str(&dto.from_owner_id) {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid from_owner_id format"
            }))
        }
    };

    let to_owner_id = match Uuid::parse_str(&dto.to_owner_id) {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid to_owner_id format"
            }))
        }
    };

    // Call use case
    match state
        .unit_owner_use_cases
        .transfer_ownership(from_owner_id, to_owner_id, unit_id)
        .await
    {
        Ok((ended, created)) => {
            // Audit log
            if let Some(org_id) = user.organization_id {
                AuditLogEntry::new(
                    AuditEventType::UnitOwnerUpdated,
                    Some(user.user_id),
                    Some(org_id),
                )
                .with_resource("UnitOwner", ended.id)
                .with_metadata(serde_json::json!({
                    "transferred_to": created.id.to_string(),
                    "new_unit_owner_id": created.id.to_string()
                }))
                .log();
            }

            HttpResponse::Ok().json(serde_json::json!({
                "ended_relationship": to_response_dto(&ended),
                "new_relationship": to_response_dto(&created)
            }))
        }
        Err(err) => HttpResponse::BadRequest().json(serde_json::json!({
            "error": err
        })),
    }
}

/// Get the total ownership percentage for a unit
#[get("/units/{unit_id}/owners/total-percentage")]
pub async fn get_total_ownership_percentage(
    state: web::Data<AppState>,
    _user: AuthenticatedUser,
    unit_id: web::Path<String>,
) -> impl Responder {
    // Parse UUID
    let unit_id = match Uuid::parse_str(&unit_id) {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid unit_id format"
            }))
        }
    };

    // Call use case
    match state
        .unit_owner_use_cases
        .get_total_ownership_percentage(unit_id)
        .await
    {
        Ok(total) => HttpResponse::Ok().json(serde_json::json!({
            "unit_id": unit_id.to_string(),
            "total_ownership_percentage": total,
            "percentage_display": format!("{:.2}%", total * 100.0)
        })),
        Err(err) => HttpResponse::BadRequest().json(serde_json::json!({
            "error": err
        })),
    }
}

// Helper function to convert entity to response DTO
fn to_response_dto(unit_owner: &UnitOwner) -> UnitOwnerResponseDto {
    UnitOwnerResponseDto {
        id: unit_owner.id.to_string(),
        unit_id: unit_owner.unit_id.to_string(),
        owner_id: unit_owner.owner_id.to_string(),
        ownership_percentage: unit_owner.ownership_percentage,
        start_date: unit_owner.start_date,
        end_date: unit_owner.end_date,
        is_primary_contact: unit_owner.is_primary_contact,
        is_active: unit_owner.is_active(),
        created_at: unit_owner.created_at,
        updated_at: unit_owner.updated_at,
    }
}
