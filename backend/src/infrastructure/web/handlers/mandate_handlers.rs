//! HTTP handlers for the Mandate feature (Story 3.4 — FR7 INV-14).
//!
//! Routes:
//! - `POST   /mandates`             — syndic / superadmin issues a mandate.
//! - `GET    /mandates?subject={u}` — list active mandates for a user.
//! - `GET    /mandates/{id}`        — details of a mandate.
//! - `POST   /mandates/{id}/revoke` — early revocation.
//!
//! Auth: syndic / superadmin for write paths; the subject can also read
//! their own mandates (`?subject=<self>`).

use crate::application::error::AppError;
use crate::domain::entities::{Mandate, MandateKind, MandateScope};
use crate::infrastructure::web::{AppState, AuthenticatedUser};
use actix_web::{get, post, web, HttpResponse};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// DTOs
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct IssueMandateRequest {
    pub subject_user_id: Uuid,
    pub kind: String,
    pub scope_kind: String,
    pub scope_id: Uuid,
    pub reason: String,
    /// Optional — defaults to `now()` server-side.
    pub valid_from: Option<DateTime<Utc>>,
    /// Mandatory. Returning 422-like validation if missing.
    pub valid_until: DateTime<Utc>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct MandateResponse {
    pub id: Uuid,
    pub subject_user_id: Uuid,
    pub kind: String,
    pub scope_kind: String,
    pub scope_id: Uuid,
    pub issued_by: Uuid,
    pub reason: String,
    pub valid_from: DateTime<Utc>,
    pub valid_until: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Mandate> for MandateResponse {
    fn from(m: Mandate) -> Self {
        Self {
            id: m.id,
            subject_user_id: m.subject_user_id,
            kind: m.kind.to_string(),
            scope_kind: m.scope.kind_str().to_string(),
            scope_id: m.scope.id(),
            issued_by: m.issued_by,
            reason: m.reason,
            valid_from: m.valid_from,
            valid_until: m.valid_until,
            revoked_at: m.revoked_at,
            created_at: m.created_at,
            updated_at: m.updated_at,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListMandatesQuery {
    pub subject: Option<Uuid>,
}

// ---------------------------------------------------------------------------
// Guards
// ---------------------------------------------------------------------------

fn require_syndic_or_superadmin(user: &AuthenticatedUser) -> Result<(), AppError> {
    match user.role.as_str() {
        "syndic" | "superadmin" => Ok(()),
        _ => Err(AppError::Forbidden(
            "Only syndic or superadmin can manage mandates".to_string(),
        )),
    }
}

// ---------------------------------------------------------------------------
// POST /mandates — issue
// ---------------------------------------------------------------------------

#[utoipa::path(
    post,
    path = "/mandates",
    tag = "Mandate",
    summary = "Issue a mandate (syndic / superadmin only)",
    responses(
        (status = 201, description = "Mandate issued", body = MandateResponse),
        (status = 400, description = "Validation error"),
        (status = 403, description = "Forbidden — only syndic or superadmin"),
    ),
)]
#[post("/mandates")]
pub async fn issue_mandate(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    body: web::Json<IssueMandateRequest>,
) -> Result<HttpResponse, AppError> {
    require_syndic_or_superadmin(&user)?;
    let req = body.into_inner();

    let kind = MandateKind::from_str(&req.kind)?;
    let scope = MandateScope::from_parts(&req.scope_kind, req.scope_id)?;
    let valid_from = req.valid_from.unwrap_or_else(Utc::now);

    let mandate = state
        .mandate_use_cases
        .issue(
            req.subject_user_id,
            kind,
            scope,
            user.user_id,
            req.reason,
            valid_from,
            req.valid_until,
        )
        .await?;

    Ok(HttpResponse::Created().json(MandateResponse::from(mandate)))
}

// ---------------------------------------------------------------------------
// GET /mandates?subject={uuid} — list active mandates
// ---------------------------------------------------------------------------

#[utoipa::path(
    get,
    path = "/mandates",
    tag = "Mandate",
    summary = "List active mandates for a subject user",
    params(
        ("subject" = Option<Uuid>, Query, description = "Subject user id. Defaults to the caller.")
    ),
    responses(
        (status = 200, description = "Active mandates", body = Vec<MandateResponse>),
        (status = 403, description = "Forbidden — caller cannot view this subject's mandates"),
    ),
)]
#[get("/mandates")]
pub async fn list_mandates(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    query: web::Query<ListMandatesQuery>,
) -> Result<HttpResponse, AppError> {
    let subject = query.into_inner().subject.unwrap_or(user.user_id);
    let is_self = subject == user.user_id;
    let is_admin = matches!(user.role.as_str(), "syndic" | "superadmin");

    if !is_self && !is_admin {
        return Err(AppError::Forbidden(
            "Only syndic / superadmin can view another user's mandates".to_string(),
        ));
    }

    let mandates = state
        .mandate_use_cases
        .list_active_for_subject(subject)
        .await?;
    let response: Vec<MandateResponse> = mandates.into_iter().map(MandateResponse::from).collect();
    Ok(HttpResponse::Ok().json(response))
}

// ---------------------------------------------------------------------------
// GET /mandates/{id} — details
// ---------------------------------------------------------------------------

#[utoipa::path(
    get,
    path = "/mandates/{id}",
    tag = "Mandate",
    summary = "Get mandate details",
    responses(
        (status = 200, description = "Mandate details", body = MandateResponse),
        (status = 403, description = "Forbidden — not allowed to view this mandate"),
        (status = 404, description = "Mandate not found"),
    ),
)]
#[get("/mandates/{id}")]
pub async fn get_mandate(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let id = path.into_inner();
    let mandate = state.mandate_use_cases.get(id).await?;

    let is_subject = mandate.subject_user_id == user.user_id;
    let is_admin = matches!(user.role.as_str(), "syndic" | "superadmin");
    if !is_subject && !is_admin {
        return Err(AppError::Forbidden(
            "Only syndic / superadmin or the mandated user can view this mandate".to_string(),
        ));
    }

    Ok(HttpResponse::Ok().json(MandateResponse::from(mandate)))
}

// ---------------------------------------------------------------------------
// POST /mandates/{id}/revoke — early revocation
// ---------------------------------------------------------------------------

#[utoipa::path(
    post,
    path = "/mandates/{id}/revoke",
    tag = "Mandate",
    summary = "Revoke a mandate before its natural expiry (syndic / superadmin)",
    responses(
        (status = 204, description = "Mandate revoked"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Mandate not found"),
    ),
)]
#[post("/mandates/{id}/revoke")]
pub async fn revoke_mandate(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    require_syndic_or_superadmin(&user)?;
    let id = path.into_inner();
    state.mandate_use_cases.revoke(id).await?;
    Ok(HttpResponse::NoContent().finish())
}
