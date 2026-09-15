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

/// `require_role` est l'idiome que ce dépôt reconnaît pour un contrôle de
/// rôle qui DÉCIDE (cf. `garde_identite_sans_decision`, liste `DECISION`) —
/// pas `match user.role.as_str() { ... }`, textuellement invisible pour ce
/// cliquet malgré une logique identique.
fn require_role(user: &AuthenticatedUser, allowed: &[&str]) -> Result<(), AppError> {
    if allowed.contains(&user.role.as_str()) {
        Ok(())
    } else {
        Err(AppError::Forbidden(format!(
            "Requires one of roles: {}",
            allowed.join(", ")
        )))
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
    require_role(&user, &["syndic", "superadmin"])?;

    let req = body.into_inner();
    let scope_kind = MagicLinkScopeKind::from_str(&req.scope_kind)?;

    // La portée `etat_date` ne passe PAS par ce endpoint générique (issue
    // #855) : lui seul vérifie le rôle, jamais que l'appelant a la main sur
    // l'organisation propriétaire de `scope_id`. `POST
    // /etats-dates/{id}/notary-access` fait ce cloisonnement
    // (`verify_etat_date_org_access`) avant d'émettre — l'accepter ici le
    // contournerait complètement : un syndic du cabinet A pourrait émettre un
    // lien pour l'état daté du cabinet B et le lire via
    // `GET /etats-dates/reference/{ref}?token=`, qui ne vérifie que la
    // correspondance jeton↔ressource, pas l'organisation. Même refus que
    // `consume_magic_link` applique déjà en sens inverse (ne résout jamais
    // `EtatDate`, cf. plus bas).
    if scope_kind == MagicLinkScopeKind::EtatDate {
        return Err(AppError::Validation(
            "scope_kind 'etat_date' : utilisez POST /etats-dates/{id}/notary-access, \
             qui vérifie l'organisation avant d'émettre"
                .to_string(),
        ));
    }

    let issued = state
        .magic_link_use_cases
        .issue(
            Some(req.subject_user_id),
            None,
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
// POST /magic-links/{id}/revoke — syndic / superadmin only (issue #855)
// ---------------------------------------------------------------------------

/// Révoque un lien avant son terme naturel (issue #855 : « révocable »).
///
/// Restreint au rôle syndic/superadmin, comme l'émission — pas à
/// l'émetteur précis : le port `MagicLinkRepository` n'expose pas de lecture
/// par id, et l'ajouter pour ce seul contrôle a été jugé disproportionné vis-
/// à-vis du gain (un syndic malveillant a de toute façon accès à la donnée
/// sous-jacente). À resserrer si ce besoin se confirme.
#[utoipa::path(
    post,
    path = "/magic-links/{id}/revoke",
    tag = "MagicLink",
    summary = "Revoke a magic link before its natural expiry (syndic / superadmin only)",
    responses(
        (status = 204, description = "MagicLink revoked"),
        (status = 403, description = "Forbidden or already revoked/consumed"),
    ),
)]
#[post("/magic-links/{id}/revoke")]
pub async fn revoke_magic_link(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    id: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    require_role(&user, &["syndic", "superadmin"])?;

    state.magic_link_use_cases.revoke(*id).await?;

    Ok(HttpResponse::NoContent().finish())
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
        MagicLinkScopeKind::EtatDate => {
            // Les liens `EtatDate` (issue #855) ne se résolvent jamais ici :
            // ce endpoint générique consomme le lien sans vérifier qu'il
            // correspond à la ressource demandée. La route dédiée
            // `GET /etats-dates/reference/{reference_number}?token=`
            // (`MagicLinkUseCases::verify_token`) fait cette vérification de
            // portée ET laisse le lien relisible jusqu'à expiration — deux
            // garanties que cet endpoint ne peut pas offrir.
            return Err(AppError::MagicLinkInvalid);
        }
    };

    Ok(HttpResponse::Ok().json(PublicScopePayload {
        scope_kind: link.scope_kind.to_string(),
        scope_id: link.scope_id,
        scope: scope_json,
    }))
}
