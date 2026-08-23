//! HTTP handlers for the MagicLink feature (Story 3.2).
//!
//! Two endpoints:
//! - `POST /magic-links` — syndic / superadmin issues a link for a recipient.
//! - `GET  /c/{token}`   — PUBLIC: validate, consume, resolve scope. The route
//!   is intentionally outside `/api/v1` so the public-facing URL stays short
//!   (`/c/<token>`). IP-based rate-limiting is enforced by Traefik for all
//!   routes — no extra guard needed here.

use crate::application::error::AppError;
use crate::domain::entities::MagicLinkScopeKind;
use crate::infrastructure::web::{AppState, AuthenticatedUser};
use actix_web::{get, post, web, HttpResponse};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// DTOs
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct IssueMagicLinkRequest {
    pub subject_user_id: Uuid,
    pub scope_kind: String,
    pub scope_id: Uuid,
    pub expires_in_seconds: i64,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct IssueMagicLinkResponse {
    pub id: Uuid,
    pub token: String,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub scope_kind: String,
    pub scope_id: Uuid,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct PublicScopePayload {
    pub scope_kind: String,
    pub scope_id: Uuid,
    #[schema(value_type = serde_json::Value)]
    pub scope: serde_json::Value,
}

// ---------------------------------------------------------------------------
// Guards
// ---------------------------------------------------------------------------

fn require_syndic_or_superadmin(user: &AuthenticatedUser) -> Result<(), AppError> {
    match user.role.as_str() {
        "syndic" | "superadmin" => Ok(()),
        _ => Err(AppError::Forbidden(
            "Only syndic or superadmin can issue magic links".to_string(),
        )),
    }
}

// ---------------------------------------------------------------------------
// POST /magic-links — syndic / superadmin only
// ---------------------------------------------------------------------------

#[utoipa::path(
    post,
    path = "/magic-links",
    tag = "MagicLink",
    summary = "Issue a magic link (syndic / superadmin only)",
    responses(
        (status = 201, description = "MagicLink issued"),
        (status = 400, description = "Validation error"),
        (status = 403, description = "Forbidden — only syndic or superadmin"),
    ),
)]
#[post("/magic-links")]
pub async fn issue_magic_link(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    body: web::Json<IssueMagicLinkRequest>,
) -> Result<HttpResponse, AppError> {
    require_syndic_or_superadmin(&user)?;

    let req = body.into_inner();
    let scope_kind = MagicLinkScopeKind::from_str(&req.scope_kind)?;

    let issued = state
        .magic_link_use_cases
        .issue(
            req.subject_user_id,
            scope_kind,
            req.scope_id,
            user.user_id,
            req.expires_in_seconds,
        )
        .await?;

    Ok(HttpResponse::Created().json(IssueMagicLinkResponse {
        id: issued.id,
        token: issued.token,
        expires_at: issued.expires_at,
        scope_kind: issued.scope_kind.to_string(),
        scope_id: issued.scope_id,
    }))
}

// ---------------------------------------------------------------------------
// GET /c/{token} — PUBLIC (no auth) — validate, consume, resolve.
// ---------------------------------------------------------------------------

#[utoipa::path(
    get,
    path = "/c/{token}",
    tag = "MagicLink",
    summary = "Public access via magic link",
    responses(
        (status = 200, description = "Scope payload"),
        (status = 403, description = "Invalid / expired / already consumed"),
    ),
)]
#[get("/c/{token}")]
pub async fn consume_magic_link(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> Result<HttpResponse, AppError> {
    let token = path.into_inner();
    let link = state
        .magic_link_use_cases
        .validate_and_consume(&token)
        .await?;

    // Resolve the underlying resource. For scopes that don't yet have a
    // public-friendly DTO we return a minimal placeholder — the front-end
    // page will render the scope_kind specific UI and may call additional
    // public endpoints if needed (follow-up).
    let scope_json = match link.scope_kind {
        MagicLinkScopeKind::Ticket => {
            match state
                .ticket_use_cases
                .get_ticket(link.scope_id)
                .await
                .map_err(AppError::Internal)?
            {
                Some(ticket) => {
                    serde_json::to_value(&ticket).map_err(|e| AppError::Internal(e.to_string()))?
                }
                None => return Err(AppError::NotFound(format!("ticket {}", link.scope_id))),
            }
        }
        MagicLinkScopeKind::Quote
        | MagicLinkScopeKind::Invoice
        | MagicLinkScopeKind::ContractorEvaluation => {
            // Follow-up: wire dedicated public DTOs for these scopes.
            // For now return the scope identifier so the front-end can
            // render a minimal "received" view.
            serde_json::json!({
                "scope_id": link.scope_id,
                "note": "Scope payload resolution pending follow-up",
            })
        }
    };

    Ok(HttpResponse::Ok().json(PublicScopePayload {
        scope_kind: link.scope_kind.to_string(),
        scope_id: link.scope_id,
        scope: scope_json,
    }))
}
