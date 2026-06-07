//! HTTP handlers for SyndicResponse (Story 3.7 — FR32 INV-23).
//!
//! Routes:
//! - `POST /tickets/{id}/syndic-responses` — syndic / superadmin posts a
//!   structured reply to a ticket.
//! - `GET  /tickets/{id}/syndic-responses` — list responses for a ticket.
//!
//! Both routes are JWT-protected. Phase A keeps the scope checks minimal
//! (syndic / superadmin can post; any authenticated user can read — the
//! ticket detail page already enforces the ACP scope upstream). Tightening
//! to "only members of the building's ACP can read" is a Phase B
//! follow-up tracked in the Story 3.7 acceptance notes.

use crate::application::error::AppError;
use crate::domain::entities::SyndicResponse;
use crate::infrastructure::web::{AppState, AuthenticatedUser};
use actix_web::{get, post, web, HttpResponse};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// DTOs
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateSyndicResponseRequest {
    /// Free text — 10..=5000 chars after trim.
    pub body: String,
    /// One of: `schedule_inspection`, `request_quote`, `closed_no_action`,
    /// `escalated_board`, `other`. Optional.
    #[serde(default)]
    pub action_proposed: Option<String>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct SyndicResponseDto {
    pub id: Uuid,
    pub ticket_id: Uuid,
    pub syndic_user_id: Uuid,
    pub body: String,
    pub action_proposed: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl From<SyndicResponse> for SyndicResponseDto {
    fn from(r: SyndicResponse) -> Self {
        Self {
            id: r.id,
            ticket_id: r.ticket_id,
            syndic_user_id: r.syndic_user_id,
            body: r.body,
            action_proposed: r.action_proposed,
            created_at: r.created_at,
        }
    }
}

// ---------------------------------------------------------------------------
// Guards
// ---------------------------------------------------------------------------

fn require_syndic_or_superadmin(user: &AuthenticatedUser) -> Result<(), AppError> {
    match user.role.as_str() {
        "syndic" | "superadmin" => Ok(()),
        _ => Err(AppError::Forbidden(
            "Only syndic or superadmin can post a SyndicResponse".to_string(),
        )),
    }
}

// ---------------------------------------------------------------------------
// POST /tickets/{id}/syndic-responses
// ---------------------------------------------------------------------------

#[utoipa::path(
    post,
    path = "/tickets/{id}/syndic-responses",
    tag = "SyndicResponse",
    summary = "Post a structured syndic response to a ticket (append-only)",
    responses(
        (status = 201, description = "Response saved", body = SyndicResponseDto),
        (status = 400, description = "Validation error (body too short/long, invalid action)"),
        (status = 403, description = "Forbidden — only syndic / superadmin"),
        (status = 404, description = "Ticket not found"),
    ),
)]
#[post("/tickets/{id}/syndic-responses")]
pub async fn create_syndic_response(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    path: web::Path<Uuid>,
    body: web::Json<CreateSyndicResponseRequest>,
) -> Result<HttpResponse, AppError> {
    require_syndic_or_superadmin(&user)?;
    let ticket_id = path.into_inner();
    let payload = body.into_inner();

    let response = state
        .syndic_response_use_cases
        .respond(
            ticket_id,
            user.user_id,
            payload.body,
            payload.action_proposed,
        )
        .await?;

    Ok(HttpResponse::Created().json(SyndicResponseDto::from(response)))
}

// ---------------------------------------------------------------------------
// GET /tickets/{id}/syndic-responses
// ---------------------------------------------------------------------------

#[utoipa::path(
    get,
    path = "/tickets/{id}/syndic-responses",
    tag = "SyndicResponse",
    summary = "List syndic responses for a ticket (oldest first)",
    responses(
        (status = 200, description = "Responses list", body = Vec<SyndicResponseDto>),
    ),
)]
#[get("/tickets/{id}/syndic-responses")]
pub async fn list_syndic_responses(
    state: web::Data<AppState>,
    _user: AuthenticatedUser,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let ticket_id = path.into_inner();
    let responses = state
        .syndic_response_use_cases
        .list_for_ticket(ticket_id)
        .await?;
    let dtos: Vec<SyndicResponseDto> = responses.into_iter().map(SyndicResponseDto::from).collect();
    Ok(HttpResponse::Ok().json(dtos))
}
