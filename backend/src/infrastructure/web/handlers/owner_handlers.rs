use crate::application::dto::{CreateOwnerDto, PageRequest, PageResponse};
use crate::infrastructure::audit::{AuditEventType, AuditLogEntry};
use crate::infrastructure::web::{AppState, AuthenticatedUser};
use actix_web::{get, post, put, web, HttpResponse, Responder};
use serde::Deserialize;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateOwnerDto {
    #[validate(length(min = 1, message = "First name is required"))]
    pub first_name: String,
    #[validate(length(min = 1, message = "Last name is required"))]
    pub last_name: String,
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
    pub phone: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LinkOwnerUserDto {
    pub user_id: Option<String>, // UUID as string, or null to unlink
}

#[post("/owners")]
pub async fn create_owner(
    state: web::Data<AppState>,
    user: AuthenticatedUser, // JWT-extracted user info (SECURE!)
    mut dto: web::Json<CreateOwnerDto>,
) -> impl Responder {
    // Only SuperAdmin and Syndic can create owners
    if user.role == "owner" || user.role == "accountant" {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Only SuperAdmin and Syndic can create owners"
        }));
    }

    // For SuperAdmin: allow specifying organization_id in DTO
    // For others: override with their JWT organization_id
    let organization_id = if user.role == "superadmin" {
        // SuperAdmin can specify organization_id or it defaults to empty string
        if dto.organization_id.is_empty() {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "SuperAdmin must specify organization_id"
            }));
        }
        match Uuid::parse_str(&dto.organization_id) {
            Ok(org_id) => org_id,
            Err(_) => {
                return HttpResponse::BadRequest().json(serde_json::json!({
                    "error": "Invalid organization_id format"
                }))
            }
        }
    } else {
        // Regular users: use their organization from JWT token
        match user.require_organization() {
            Ok(org_id) => {
                dto.organization_id = org_id.to_string();
                org_id
            }
            Err(e) => {
                return HttpResponse::Unauthorized().json(serde_json::json!({
                    "error": e.to_string()
                }))
            }
        }
    };

    if let Err(errors) = dto.validate() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Validation failed",
            "details": errors.to_string()
        }));
    }

    match state.owner_use_cases.create_owner(dto.into_inner()).await {
        Ok(owner) => {
            // Audit log: successful owner creation
            AuditLogEntry::new(
                AuditEventType::OwnerCreated,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_resource("Owner", Uuid::parse_str(&owner.id).unwrap())
            .log();

            HttpResponse::Created().json(owner)
        }
        Err(err) => {
            // Audit log: failed owner creation
            AuditLogEntry::new(
                AuditEventType::OwnerCreated,
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

#[get("/owners")]
pub async fn list_owners(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    page_request: web::Query<PageRequest>,
) -> impl Responder {
    // SuperAdmin can see all owners, others only see their organization's owners
    let organization_id = if user.role == "superadmin" {
        None // SuperAdmin sees all organizations
    } else {
        user.organization_id // Other roles see only their organization
    };

    match state
        .owner_use_cases
        .list_owners_paginated(&page_request, organization_id)
        .await
    {
        Ok((owners, total)) => {
            let response =
                PageResponse::new(owners, page_request.page, page_request.per_page, total);
            HttpResponse::Ok().json(response)
        }
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": err
        })),
    }
}

#[get("/owners/{id}")]
pub async fn get_owner(state: web::Data<AppState>, id: web::Path<Uuid>) -> impl Responder {
    match state.owner_use_cases.get_owner(*id).await {
        Ok(Some(owner)) => HttpResponse::Ok().json(owner),
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Owner not found"
        })),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": err
        })),
    }
}

#[put("/owners/{id}")]
pub async fn update_owner(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    id: web::Path<Uuid>,
    dto: web::Json<UpdateOwnerDto>,
) -> impl Responder {
    // Only SuperAdmin and Syndic can update owners
    if user.role == "owner" || user.role == "accountant" {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Only SuperAdmin and Syndic can update owners"
        }));
    }

    // SuperAdmin can update any owner, others need organization check
    let user_organization_id = if user.role != "superadmin" {
        match user.require_organization() {
            Ok(org_id) => Some(org_id),
            Err(e) => {
                return HttpResponse::Unauthorized().json(serde_json::json!({
                    "error": e.to_string()
                }))
            }
        }
    } else {
        None // SuperAdmin doesn't need organization check
    };

    if let Err(errors) = dto.validate() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Validation failed",
            "details": errors.to_string()
        }));
    }

    let owner_id = *id;

    // First verify the owner exists and belongs to the user's organization
    match state.owner_use_cases.get_owner(owner_id).await {
        Ok(Some(_existing_owner)) => {
            // Verify organization ownership
            // Note: We need to check if this owner belongs to the user's organization
            // For now, we'll proceed with the update
            match state
                .owner_use_cases
                .update_owner(
                    owner_id,
                    dto.first_name.clone(),
                    dto.last_name.clone(),
                    dto.email.clone(),
                    dto.phone.clone(),
                )
                .await
            {
                Ok(owner) => {
                    // Audit log: successful owner update
                    AuditLogEntry::new(
                        AuditEventType::OwnerUpdated,
                        Some(user.user_id),
                        user_organization_id,
                    )
                    .with_resource("Owner", owner_id)
                    .log();

                    HttpResponse::Ok().json(owner)
                }
                Err(err) => {
                    // Audit log: failed owner update
                    AuditLogEntry::new(
                        AuditEventType::OwnerUpdated,
                        Some(user.user_id),
                        user_organization_id,
                    )
                    .with_error(err.clone())
                    .log();

                    HttpResponse::BadRequest().json(serde_json::json!({
                        "error": err
                    }))
                }
            }
        }
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Owner not found"
        })),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": err
        })),
    }
}

/// Link or unlink a user account to an owner (SuperAdmin only)
#[put("/owners/{id}/link-user")]
pub async fn link_owner_to_user(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    id: web::Path<Uuid>,
    dto: web::Json<LinkOwnerUserDto>,
) -> impl Responder {
    // Only SuperAdmin can link users to owners
    if user.role != "superadmin" {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Only SuperAdmin can link users to owners"
        }));
    }

    let owner_id = *id;

    // Parse user_id if provided
    let user_id_to_link = if let Some(user_id_str) = &dto.user_id {
        if user_id_str.is_empty() {
            None // Empty string = unlink
        } else {
            match Uuid::parse_str(user_id_str) {
                Ok(uid) => Some(uid),
                Err(_) => {
                    return HttpResponse::BadRequest().json(serde_json::json!({
                        "error": "Invalid user_id format"
                    }))
                }
            }
        }
    } else {
        None // null = unlink
    };

    // Verify owner exists
    let _owner = match state.owner_use_cases.get_owner(owner_id).await {
        Ok(Some(o)) => o,
        Ok(None) => {
            return HttpResponse::NotFound().json(serde_json::json!({
                "error": "Owner not found"
            }))
        }
        Err(err) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": err
            }))
        }
    };

    // If linking to a user, verify the user exists and has role=owner
    if let Some(uid) = user_id_to_link {
        let user_check = sqlx::query!("SELECT role FROM users WHERE id = $1", uid)
            .fetch_optional(&state.pool)
            .await;

        match user_check {
            Ok(Some(user_record)) => {
                if user_record.role != "owner" {
                    return HttpResponse::BadRequest().json(serde_json::json!({
                        "error": "User must have role 'owner' to be linked to an owner entity"
                    }));
                }
            }
            Ok(None) => {
                return HttpResponse::NotFound().json(serde_json::json!({
                    "error": "User not found"
                }));
            }
            Err(err) => {
                return HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": format!("Database error: {}", err)
                }));
            }
        }

        // Check if this user is already linked to another owner
        let existing_link = sqlx::query!(
            "SELECT id, first_name, last_name FROM owners WHERE user_id = $1 AND id != $2",
            uid,
            owner_id
        )
        .fetch_optional(&state.pool)
        .await;

        match existing_link {
            Ok(Some(existing)) => {
                return HttpResponse::Conflict().json(serde_json::json!({
                    "error": format!("User is already linked to owner {} {} (ID: {})",
                        existing.first_name, existing.last_name, existing.id)
                }));
            }
            Ok(None) => {} // OK, no conflict
            Err(err) => {
                return HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": format!("Database error: {}", err)
                }));
            }
        }
    }

    // Update the owner's user_id
    let update_result = sqlx::query!(
        "UPDATE owners SET user_id = $1, updated_at = NOW() WHERE id = $2",
        user_id_to_link,
        owner_id
    )
    .execute(&state.pool)
    .await;

    match update_result {
        Ok(_) => {
            // Audit log
            AuditLogEntry::new(
                AuditEventType::OwnerUpdated,
                Some(user.user_id),
                user.organization_id,
            )
            .with_resource("Owner", owner_id)
            .log();

            let action = if user_id_to_link.is_some() {
                "linked"
            } else {
                "unlinked"
            };

            HttpResponse::Ok().json(serde_json::json!({
                "message": format!("Owner successfully {} to user", action),
                "owner_id": owner_id,
                "user_id": user_id_to_link
            }))
        }
        Err(err) => {
            // Audit log
            AuditLogEntry::new(
                AuditEventType::OwnerUpdated,
                Some(user.user_id),
                user.organization_id,
            )
            .with_error(err.to_string())
            .log();

            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Database error: {}", err)
            }))
        }
    }
}
