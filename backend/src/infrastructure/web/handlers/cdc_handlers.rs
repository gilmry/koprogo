//! Handlers HTTP du conseil de copropriété — Story 4.7 (#582).

use crate::application::dto::{CreateBoardAlertDto, ElectCdcMembersDto};
use crate::infrastructure::audit::{AuditEventType, AuditLogEntry};
use crate::infrastructure::web::middleware::scope_guard::verify_acp_org_access;
use crate::infrastructure::web::{AppState, AuthenticatedUser};
use actix_web::{get, post, web, HttpResponse, Responder, ResponseError};
use uuid::Uuid;

/// Résout l'`owner_id` de l'utilisateur authentifié — un membre du conseil
/// est nécessairement un copropriétaire (Art. 3.90 §1er CC).
/// `Box<HttpResponse>` : clippy refuse un variant `Err` de 128 octets
/// (`result_large_err`), et il a raison — chaque appel déplacerait la
/// réponse entière sur la pile, y compris sur le chemin nominal où elle
/// n'existe pas. Le coût du déréférencement est sur le chemin d'ERREUR.
async fn owner_id_of(
    state: &AppState,
    user: &AuthenticatedUser,
) -> Result<Uuid, Box<HttpResponse>> {
    match state
        .owner_use_cases
        .find_owner_by_user_id(user.user_id)
        .await
    {
        Ok(Some(owner)) => Uuid::parse_str(&owner.id).map_err(|_| {
            Box::new(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Invalid owner id"
            })))
        }),
        Ok(None) => Err(Box::new(HttpResponse::Forbidden().json(
            serde_json::json!({
                "error": "Cette action est réservée aux membres du conseil de copropriété, \
                          qui sont copropriétaires (Art. 3.90 §1er CC). Votre compte n'est \
                          rattaché à aucune fiche de copropriétaire."
            }),
        ))),
        Err(err) => Err(Box::new(HttpResponse::InternalServerError().json(
            serde_json::json!({ "error": format!("Database error: {}", err) }),
        ))),
    }
}

/// Émet une alerte du conseil de copropriété à destination de la prochaine AG.
#[utoipa::path(
    post,
    path = "/buildings/{building_id}/cdc/alerts",
    tag = "Cdc",
    summary = "Émettre une alerte du conseil de copropriété",
    params(("building_id" = Uuid, Path, description = "UUID de l'immeuble")),
    request_body = CreateBoardAlertDto,
    responses(
        (status = 201, description = "Alerte émise", body = crate::application::dto::BoardAlertResponseDto),
        (status = 403, description = "Hors mandat ou hors portée"),
        (status = 422, description = "Sévérité ou AG cible invalide"),
    ),
    security(("bearer_auth" = []))
)]
#[post("/buildings/{building_id}/cdc/alerts")]
pub async fn create_cdc_alert(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    building_id: web::Path<Uuid>,
    request: web::Json<CreateBoardAlertDto>,
) -> impl Responder {
    let owner_id = match owner_id_of(&state, &user).await {
        Ok(id) => id,
        Err(resp) => return *resp,
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
#[utoipa::path(
    get,
    path = "/meetings/{meeting_id}/cdc/alerts",
    tag = "Cdc",
    summary = "Alertes du conseil rattachées à une AG",
    params(("meeting_id" = Uuid, Path, description = "UUID de l'assemblée")),
    responses(
        (status = 200, description = "Alertes de l'AG", body = Vec<crate::application::dto::BoardAlertResponseDto>),
        (status = 403, description = "Hors portée (cloisonnement #882)"),
        (status = 404, description = "Assemblée introuvable"),
    ),
    security(("bearer_auth" = []))
)]
#[get("/meetings/{meeting_id}/cdc/alerts")]
pub async fn list_cdc_alerts_for_meeting(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    meeting_id: web::Path<Uuid>,
) -> impl Responder {
    let meeting_id = meeting_id.into_inner();

    // Cloisonnement AVANT la lecture (#882).
    //
    // Cette route prenait `_user` — l'identité DÉCLARÉE inutile par le
    // souligné — et rendait les alertes du conseil de n'importe quelle AG à
    // quiconque en connaissait l'UUID.
    //
    // Les alertes du conseil de copropriété ne sont pas des données
    // anodines : le conseil contrôle la gestion du syndic (Art. 3.90 § 1er),
    // et ses alertes disent où il estime que quelque chose cloche. Les lire
    // depuis un autre cabinet, c'est lire un audit interne.
    //
    // La chaîne est celle du reste du dépôt : AG → immeuble → ACP →
    // organisation.
    let meeting = match state.meeting_use_cases.get_meeting(meeting_id).await {
        Ok(Some(m)) => m,
        Ok(None) => {
            return HttpResponse::NotFound().json(serde_json::json!({
                "error": "Meeting not found"
            }))
        }
        Err(err) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": err.to_string()
            }))
        }
    };
    // Un immeuble ou une ACP illisibles REFUSENT : une AG qu'on ne sait pas
    // rattacher est une AG dont on ne peut pas dire qu'elle relève du mandat
    // de l'appelant.
    let acp_id = match state
        .building_use_cases
        .get_building(meeting.building_id)
        .await
    {
        Ok(Some(b)) => match Uuid::parse_str(&b.acp_id) {
            Ok(id) => id,
            Err(_) => {
                return HttpResponse::Forbidden().json(serde_json::json!({
                    "error": "Impossible de rattacher cette assemblée à une ACP"
                }))
            }
        },
        _ => {
            return HttpResponse::Forbidden().json(serde_json::json!({
                "error": "Impossible de rattacher cette assemblée à une ACP"
            }))
        }
    };
    if let Err(err) = verify_acp_org_access(&user, acp_id, &state.acp_use_cases).await {
        return err.error_response();
    }

    match state
        .cdc_use_cases
        .list_alerts_for_meeting(meeting_id)
        .await
    {
        Ok(alerts) => HttpResponse::Ok().json(alerts),
        Err(err) => err.error_response(),
    }
}

/// Élit les membres du conseil de copropriété à l'issue d'une AG clôturée.
#[utoipa::path(
    post,
    path = "/buildings/{building_id}/cdc/elections",
    tag = "Cdc",
    summary = "Élire les membres du conseil de copropriété",
    params(("building_id" = Uuid, Path, description = "UUID de l'immeuble")),
    request_body = ElectCdcMembersDto,
    responses(
        (status = 201, description = "Membres élus", body = Vec<crate::application::dto::BoardMemberResponseDto>),
        (status = 403, description = "Hors portée"),
        (status = 422, description = "Quorum non atteint ou candidature invalide"),
    ),
    security(("bearer_auth" = []))
)]
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
