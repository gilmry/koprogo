use crate::application::dto::{CreateOwnerDto, PageRequest, PageResponse};
use crate::infrastructure::audit::{AuditEventType, AuditLogEntry};
use crate::infrastructure::web::{AppState, AuthenticatedUser};
use actix_web::{get, post, web, HttpResponse, Responder};
use uuid::Uuid;
use validator::Validate;

#[post("/owners")]
pub async fn create_owner(
    state: web::Data<AppState>,
    user: AuthenticatedUser, // JWT-extracted user info (SECURE!)
    mut dto: web::Json<CreateOwnerDto>,
) -> impl Responder {
    // Override the organization_id from DTO with the one from JWT token
    // This prevents users from creating owners in other organizations
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
    let organization_id = user.organization_id;

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
