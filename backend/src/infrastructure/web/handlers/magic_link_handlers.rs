//! HTTP handlers for the MagicLink feature (Story 3.2).
//!
//! Two endpoints:
//! - `POST /magic-links` — syndic / superadmin issues a link for a recipient.
//! - `GET  /c/{token}`   — PUBLIC: validate, consume, resolve scope. The route
//!   is intentionally outside `/api/v1` so the public-facing URL stays short
//!   (`/c/<token>`). IP-based rate-limiting is enforced by Traefik for all
//!   routes — no extra guard needed here.

use crate::application::dto::contractor_report_dto::UpdateContractorReportDto;
use crate::application::error::AppError;
use crate::application::use_cases::MagicLinkUseCases;
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
        MagicLinkScopeKind::Quote | MagicLinkScopeKind::Invoice => {
            // Follow-up: wire dedicated public DTOs for these scopes.
            // For now return the scope identifier so the front-end can
            // render a minimal "received" view.
            serde_json::json!({
                "scope_id": link.scope_id,
                "note": "Scope payload resolution pending follow-up",
            })
        }
        MagicLinkScopeKind::ContractorEvaluation => {
            serde_json::json!({
                "scope_id": link.scope_id,
                "note": "Scope payload resolution pending follow-up",
            })
        }
        MagicLinkScopeKind::ContractorReport => {
            // #835 — le rapport d'intervention rejoint l'écran unifié : même
            // page, même paramètre `t`, plus de second système parallèle.
            let dto = state
                .contractor_report_use_cases
                .get_via_magic_link(link.scope_id)
                .await?;
            serde_json::to_value(&dto).map_err(|e| AppError::Internal(e.to_string()))?
        }
    };

    Ok(HttpResponse::Ok().json(PublicScopePayload {
        scope_kind: link.scope_kind.to_string(),
        scope_id: link.scope_id,
        scope: scope_json,
    }))
}

// ---------------------------------------------------------------------------
// POST /c/{token}/respond — PUBLIC (no auth) — write action bound to the same
// token as the GET above. #835.
// ---------------------------------------------------------------------------

/// Only `ContractorReport` has a real "respond" action today: view the ticket
/// via the other four scopes is already implemented (`GET /c/{token}`), but
/// writing back for them has no use case yet — that's an existing gap this
/// route doesn't attempt to close, only to name explicitly rather than 404.
#[utoipa::path(
    post,
    path = "/c/{token}/respond",
    tag = "MagicLink",
    summary = "Public write action for a magic link (currently: ContractorReport submit)",
    responses(
        (status = 200, description = "Report updated and submitted"),
        (status = 400, description = "Unsupported scope for this link, or validation error"),
        (status = 403, description = "Invalid / expired token"),
    ),
)]
#[post("/c/{token}/respond")]
pub async fn respond_magic_link(
    state: web::Data<AppState>,
    path: web::Path<String>,
    body: web::Json<UpdateContractorReportDto>,
) -> Result<HttpResponse, AppError> {
    let token = path.into_inner();

    // Non-consuming lookup — the initial `GET /c/{token}` already consumed
    // the token as an audit marker; this write may happen much later
    // (offline draft, cf. #835 @edge), so it must not be re-gated on
    // `consumed_at`. Cloisonnement is enforced right below via `ensure_scope`.
    let link = state.magic_link_use_cases.peek(&token).await?;
    MagicLinkUseCases::ensure_scope(&link, MagicLinkScopeKind::ContractorReport)?;

    let updated = state
        .contractor_report_use_cases
        .respond_via_magic_link(link.scope_id, body.into_inner())
        .await?;

    Ok(HttpResponse::Ok().json(updated))
}
