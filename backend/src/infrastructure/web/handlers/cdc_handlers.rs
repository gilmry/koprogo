//! Handlers HTTP du conseil de copropriété — Story 4.7 (#582).

use crate::application::dto::{CreateBoardAlertDto, ElectCdcMembersDto};
use crate::infrastructure::audit::{AuditEventType, AuditLogEntry};
use crate::infrastructure::web::{AppState, AuthenticatedUser};
use actix_web::{get, post, web, HttpResponse, Responder, ResponseError};
use uuid::Uuid;

/// Résout l'`owner_id` de l'utilisateur authentifié — un membre du conseil
/// est nécessairement un copropriétaire (Art. 3.90 §1er CC).
async fn owner_id_of(state: &AppState, user: &AuthenticatedUser) -> Result<Uuid, HttpResponse> {
    match state
        .owner_use_cases
        .find_owner_by_user_id(user.user_id)
        .await
    {
        Ok(Some(owner)) => Uuid::parse_str(&owner.id).map_err(|_| {
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Invalid owner id"
            }))
        }),
        Ok(None) => Err(HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Cette action est réservée aux membres du conseil de copropriété, \
                      qui sont copropriétaires (Art. 3.90 §1er CC). Votre compte n'est \
                      rattaché à aucune fiche de copropriétaire."
        }))),
        Err(err) => Err(HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Database error: {}", err)
        }))),
    }
}

/// Émet une alerte du conseil de copropriété à destination de la prochaine AG.
#[post("/buildings/{building_id}/cdc/alerts")]
pub async fn create_cdc_alert(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    building_id: web::Path<Uuid>,
    request: web::Json<CreateBoardAlertDto>,
) -> impl Responder {
    let owner_id = match owner_id_of(&state, &user).await {
        Ok(id) => id,
        Err(resp) => return resp,
    };
    let building_id = building_id.into_inner();

    match state
        .cdc_use_cases
        .create_alert(owner_id, building_id, request.into_inner())
        .await
    {
        Ok(alert) => {
            if let Ok(alert_id) = Uuid::parse_str(&alert.id) {
                AuditLogEntry::new(
                    AuditEventType::CdcAlertCreated,
                    Some(user.user_id),
                    user.organization_id,
                )
                .with_resource("BoardAlert", alert_id)
                .log();
            }
            HttpResponse::Created().json(alert)
        }
        Err(err) => {
            AuditLogEntry::new(
                AuditEventType::CdcAlertCreated,
                Some(user.user_id),
                user.organization_id,
            )
            .with_error(err.to_string())
            .log();
            err.error_response()
        }
    }
}

/// Liste les alertes du conseil visibles à une AG donnée.
#[get("/meetings/{meeting_id}/cdc/alerts")]
pub async fn list_cdc_alerts_for_meeting(
    state: web::Data<AppState>,
    _user: AuthenticatedUser,
    meeting_id: web::Path<Uuid>,
) -> impl Responder {
    match state
        .cdc_use_cases
        .list_alerts_for_meeting(meeting_id.into_inner())
        .await
    {
        Ok(alerts) => HttpResponse::Ok().json(alerts),
        Err(err) => err.error_response(),
    }
}

/// Élit les membres du conseil de copropriété à l'issue d'une AG clôturée.
#[post("/buildings/{building_id}/cdc/elections")]
pub async fn elect_cdc_members(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    building_id: web::Path<Uuid>,
    request: web::Json<ElectCdcMembersDto>,
) -> impl Responder {
    match state
        .cdc_use_cases
        .elect_members(building_id.into_inner(), request.into_inner())
        .await
    {
        Ok(members) => {
            AuditLogEntry::new(
                AuditEventType::CdcMembersElected,
                Some(user.user_id),
                user.organization_id,
            )
            .log();
            HttpResponse::Created().json(members)
        }
        Err(err) => {
            AuditLogEntry::new(
                AuditEventType::CdcMembersElected,
                Some(user.user_id),
                user.organization_id,
            )
            .with_error(err.to_string())
            .log();
            err.error_response()
        }
    }
}
