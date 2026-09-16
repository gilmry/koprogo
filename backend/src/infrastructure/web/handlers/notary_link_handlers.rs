//! Syndic-facing endpoints for issuing / revoking / renewing / listing
//! `NotaryLink`s (Issue #855 — ADR 0048/0051).
//!
//! The anonymous consultation endpoint
//! (`GET /etats-dates/reference/{reference_number}`) lives in
//! `etat_date_handlers.rs`, next to the use case it now delegates to.

use crate::infrastructure::audit::{AuditEventType, AuditLogEntry};
use crate::infrastructure::web::{AppState, AuthenticatedUser};
use actix_web::{get, post, web, HttpResponse, Responder, ResponseError};
use uuid::Uuid;

/// Cloisonne un lien notaire AVANT de le muter (#864) : résout l'état daté
/// propriétaire du lien et vérifie que l'appelant a accès à son organisation.
async fn verify_org_access_for_etat_date(
    state: &web::Data<AppState>,
    user: &AuthenticatedUser,
    etat_date_id: Uuid,
) -> Option<HttpResponse> {
    match state.etat_date_use_cases.get_etat_date(etat_date_id).await {
        Ok(Some(etat_date)) => match user.verify_org_access(etat_date.organization_id) {
            Ok(()) => None,
            Err(err) => Some(HttpResponse::Forbidden().json(serde_json::json!({ "error": err }))),
        },
        Ok(None) => Some(HttpResponse::NotFound().json(serde_json::json!({
            "error": "État daté not found"
        }))),
        Err(err) => Some(HttpResponse::InternalServerError().json(serde_json::json!({
            "error": err
        }))),
    }
}

/// Syndic émet un lien pour un état daté (#855). Le jeton en clair n'apparaît
/// que dans CETTE réponse — jamais relu ensuite.
#[post("/etats-dates/{id}/notary-links")]
pub async fn issue_notary_link(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    id: web::Path<Uuid>,
) -> impl Responder {
    let etat_date_id = *id;
    if let Some(refus) = verify_org_access_for_etat_date(&state, &user, etat_date_id).await {
        return refus;
    }

    match state
        .notary_link_use_cases
        .issue(etat_date_id, user.user_id)
        .await
    {
        Ok(issued) => {
            AuditLogEntry::new(
                AuditEventType::NotaryLinkIssued,
                Some(user.user_id),
                user.organization_id,
            )
            .with_resource("NotaryLink", issued.id)
            .log();
            HttpResponse::Created().json(issued)
        }
        Err(err) => err.error_response(),
    }
}

/// Syndic révoque un lien avant terme.
#[post("/notary-links/{id}/revoke")]
pub async fn revoke_notary_link(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    id: web::Path<Uuid>,
) -> impl Responder {
    let link_id = *id;
    let link = match state.notary_link_use_cases.get(link_id).await {
        Ok(link) => link,
        Err(err) => return err.error_response(),
    };
    if let Some(refus) = verify_org_access_for_etat_date(&state, &user, link.etat_date_id).await {
        return refus;
    }

    match state.notary_link_use_cases.revoke(link_id).await {
        Ok(()) => {
            AuditLogEntry::new(
                AuditEventType::NotaryLinkRevoked,
                Some(user.user_id),
                user.organization_id,
            )
            .with_resource("NotaryLink", link_id)
            .log();
            HttpResponse::NoContent().finish()
        }
        Err(err) => err.error_response(),
    }
}

/// Syndic renouvelle un lien : repart pour sept jours (ADR 0051), même s'il
/// était déjà expiré. Refusé sur un lien révoqué.
#[post("/notary-links/{id}/renew")]
pub async fn renew_notary_link(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    id: web::Path<Uuid>,
) -> impl Responder {
    let link_id = *id;
    let link = match state.notary_link_use_cases.get(link_id).await {
        Ok(link) => link,
        Err(err) => return err.error_response(),
    };
    if let Some(refus) = verify_org_access_for_etat_date(&state, &user, link.etat_date_id).await {
        return refus;
    }

    match state.notary_link_use_cases.renew(link_id).await {
        Ok(renewed) => {
            AuditLogEntry::new(
                AuditEventType::NotaryLinkRenewed,
                Some(user.user_id),
                user.organization_id,
            )
            .with_resource("NotaryLink", link_id)
            .log();
            HttpResponse::Ok().json(renewed)
        }
        Err(err) => err.error_response(),
    }
}

/// Liste les liens émis pour un état daté — écran d'émission syndic et
/// suivi des demandes en défaut (`releve_notaire`, débouché naturel cité par
/// ADR 0048).
#[get("/etats-dates/{id}/notary-links")]
pub async fn list_notary_links(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    id: web::Path<Uuid>,
) -> impl Responder {
    let etat_date_id = *id;
    if let Some(refus) = verify_org_access_for_etat_date(&state, &user, etat_date_id).await {
        return refus;
    }

    match state
        .notary_link_use_cases
        .list_for_etat_date(etat_date_id)
        .await
    {
        Ok(links) => HttpResponse::Ok().json(links),
        Err(err) => err.error_response(),
    }
}
