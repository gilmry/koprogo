use crate::application::dto::{
    AddAgendaItemRequest, CompleteMeetingRequest, CreateMeetingRequest, PageRequest, PageResponse,
    UpdateMeetingRequest,
};
use crate::infrastructure::web::{AppState, AuthenticatedUser, OrganizationId};
use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use uuid::Uuid;

#[post("/meetings")]
pub async fn create_meeting(
    state: web::Data<AppState>,
    organization: OrganizationId, // JWT-extracted organization_id (SECURE!)
    mut request: web::Json<CreateMeetingRequest>,
) -> impl Responder {
    // Override the organization_id from request with the one from JWT token
    request.organization_id = organization.0;

    match state
        .meeting_use_cases
        .create_meeting(request.into_inner())
        .await
    {
        Ok(meeting) => HttpResponse::Created().json(meeting),
        Err(err) => HttpResponse::BadRequest().json(serde_json::json!({
            "error": err
        })),
    }
}

#[get("/meetings/{id}")]
pub async fn get_meeting(state: web::Data<AppState>, id: web::Path<Uuid>) -> impl Responder {
    match state.meeting_use_cases.get_meeting(*id).await {
        Ok(Some(meeting)) => HttpResponse::Ok().json(meeting),
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Meeting not found"
        })),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": err
        })),
    }
}

#[get("/meetings")]
pub async fn list_meetings(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    page_request: web::Query<PageRequest>,
) -> impl Responder {
    let organization_id = user.organization_id;

    match state
        .meeting_use_cases
        .list_meetings_paginated(&page_request, organization_id)
        .await
    {
        Ok((meetings, total)) => {
            let response = PageResponse::new(
                meetings,
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

#[get("/buildings/{building_id}/meetings")]
pub async fn list_meetings_by_building(
    state: web::Data<AppState>,
    building_id: web::Path<Uuid>,
) -> impl Responder {
    match state
        .meeting_use_cases
        .list_meetings_by_building(*building_id)
        .await
    {
        Ok(meetings) => HttpResponse::Ok().json(meetings),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": err
        })),
    }
}

#[put("/meetings/{id}")]
pub async fn update_meeting(
    state: web::Data<AppState>,
    id: web::Path<Uuid>,
    request: web::Json<UpdateMeetingRequest>,
) -> impl Responder {
    match state
        .meeting_use_cases
        .update_meeting(*id, request.into_inner())
        .await
    {
        Ok(meeting) => HttpResponse::Ok().json(meeting),
        Err(err) => HttpResponse::BadRequest().json(serde_json::json!({
            "error": err
        })),
    }
}

#[post("/meetings/{id}/agenda")]
pub async fn add_agenda_item(
    state: web::Data<AppState>,
    id: web::Path<Uuid>,
    request: web::Json<AddAgendaItemRequest>,
) -> impl Responder {
    match state
        .meeting_use_cases
        .add_agenda_item(*id, request.into_inner())
        .await
    {
        Ok(meeting) => HttpResponse::Ok().json(meeting),
        Err(err) => HttpResponse::BadRequest().json(serde_json::json!({
            "error": err
        })),
    }
}

#[post("/meetings/{id}/complete")]
pub async fn complete_meeting(
    state: web::Data<AppState>,
    id: web::Path<Uuid>,
    request: web::Json<CompleteMeetingRequest>,
) -> impl Responder {
    match state
        .meeting_use_cases
        .complete_meeting(*id, request.into_inner())
        .await
    {
        Ok(meeting) => HttpResponse::Ok().json(meeting),
        Err(err) => HttpResponse::BadRequest().json(serde_json::json!({
            "error": err
        })),
    }
}

#[post("/meetings/{id}/cancel")]
pub async fn cancel_meeting(state: web::Data<AppState>, id: web::Path<Uuid>) -> impl Responder {
    match state.meeting_use_cases.cancel_meeting(*id).await {
        Ok(meeting) => HttpResponse::Ok().json(meeting),
        Err(err) => HttpResponse::BadRequest().json(serde_json::json!({
            "error": err
        })),
    }
}

#[delete("/meetings/{id}")]
pub async fn delete_meeting(state: web::Data<AppState>, id: web::Path<Uuid>) -> impl Responder {
    match state.meeting_use_cases.delete_meeting(*id).await {
        Ok(true) => HttpResponse::NoContent().finish(),
        Ok(false) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Meeting not found"
        })),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": err
        })),
    }
}
