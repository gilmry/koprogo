use crate::application::dto::{CreateOwnerDto, PageRequest, PageResponse};
use crate::infrastructure::web::{AppState, AuthenticatedUser, OrganizationId};
use actix_web::{get, post, web, HttpResponse, Responder};
use uuid::Uuid;
use validator::Validate;

#[post("/owners")]
pub async fn create_owner(
    state: web::Data<AppState>,
    organization: OrganizationId, // JWT-extracted organization_id (SECURE!)
    mut dto: web::Json<CreateOwnerDto>,
) -> impl Responder {
    // Override the organization_id from DTO with the one from JWT token
    dto.organization_id = organization.0.to_string();

    if let Err(errors) = dto.validate() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Validation failed",
            "details": errors.to_string()
        }));
    }

    match state.owner_use_cases.create_owner(dto.into_inner()).await {
        Ok(owner) => HttpResponse::Created().json(owner),
        Err(err) => HttpResponse::BadRequest().json(serde_json::json!({
            "error": err
        })),
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
            let response = PageResponse::new(
                owners,
                page_request.page,
                page_request.per_page,
                total,
            );
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
