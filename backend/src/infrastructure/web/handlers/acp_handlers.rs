//! Handlers Actix pour `/acps` — Story 1.1.
//!
//! Endpoints :
//! - POST   `/acps`        : create (admin)
//! - GET    `/acps`        : list filtré par rôle
//! - GET    `/acps/{id}`   : get + scope guard
//! - PUT    `/acps/{id}`   : update (admin + scope)
//! - DELETE `/acps/{id}`   : archive (admin + scope)
//!
//! Le mapping `AuthenticatedUser → AcpCaller` se fait ici (couche infra),
//! pour garder le use-case 100% testable en pur Rust.
//!
//! Audit : `infrastructure::audit::AuditLogEntry` consigne chaque mutation
//! réussie ET chaque échec (pattern Building, traçabilité INV-24).

use crate::application::dto::{CreateAcpDto, UpdateAcpDto};
use crate::application::use_cases::acp_use_cases::AcpCaller;
use crate::infrastructure::audit::{AuditEventType, AuditLogEntry};
use crate::infrastructure::web::{AppState, AuthenticatedUser};
use actix_web::{delete, get, post, put, web, HttpResponse, Responder, ResponseError};
use uuid::Uuid;
use validator::Validate;

/// Map `AuthenticatedUser` → `AcpCaller` (story 1.1 — sera enrichi en 3.1
/// quand les sous-rôles accountant.* apparaîtront).
fn caller_from_user(user: &AuthenticatedUser) -> AcpCaller {
    match user.role.to_lowercase().as_str() {
        "superadmin" => AcpCaller::SuperAdmin,
        "admin" => match user.organization_id {
            Some(org) => AcpCaller::Admin {
                organization_id: org,
            },
            None => AcpCaller::SuperAdmin, // admin sans org = traité comme superadmin lecture
        },
        "syndic" | "accountant" => match user.organization_id {
            Some(org) => AcpCaller::Syndic {
                organization_id: org,
            },
            None => AcpCaller::Owner {
                user_id: user.user_id,
            },
        },
        _ => AcpCaller::Owner {
            user_id: user.user_id,
        },
    }
}

#[utoipa::path(
    post,
    path = "/acps",
    tag = "Acps",
    summary = "Create an ACP (Association des Copropriétaires)",
    request_body = CreateAcpDto,
    responses(
        (status = 201, description = "ACP created", body = crate::application::dto::AcpResponseDto),
        (status = 400, description = "Validation error"),
        (status = 403, description = "Forbidden (non-admin)"),
        (status = 422, description = "Domain validation error"),
    ),
    security(("bearer_auth" = []))
)]
#[post("/acps")]
pub async fn create_acp(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    dto: web::Json<CreateAcpDto>,
) -> impl Responder {
    if let Err(errors) = dto.validate() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Validation failed",
            "details": errors.to_string(),
            "kind": "validation",
        }));
    }

    let caller = caller_from_user(&user);
    match state
        .acp_use_cases
        .create_acp(&caller, dto.into_inner())
        .await
    {
        Ok(resp) => {
            // Audit OK
            if let Ok(acp_uuid) = Uuid::parse_str(&resp.id) {
                AuditLogEntry::new(
                    AuditEventType::AcpCreated,
                    Some(user.user_id),
                    user.organization_id,
                )
                .with_resource("Acp", acp_uuid)
                .log();
            }
            HttpResponse::Created().json(resp)
        }
        Err(err) => {
            AuditLogEntry::new(
                AuditEventType::AcpCreated,
                Some(user.user_id),
                user.organization_id,
            )
            .with_error(err.to_string())
            .log();
            err.error_response()
        }
    }
}

#[utoipa::path(
    get,
    path = "/acps",
    tag = "Acps",
    summary = "List ACPs visible to the authenticated user",
    responses(
        (status = 200, description = "List of ACPs", body = Vec<crate::application::dto::AcpResponseDto>),
        (status = 401, description = "Unauthorized"),
    ),
    security(("bearer_auth" = []))
)]
#[get("/acps")]
pub async fn list_acps(state: web::Data<AppState>, user: AuthenticatedUser) -> impl Responder {
    let caller = caller_from_user(&user);
    match state.acp_use_cases.list_acps(&caller).await {
        Ok(list) => HttpResponse::Ok().json(list),
        Err(err) => err.error_response(),
    }
}

#[utoipa::path(
    get,
    path = "/acps/{id}",
    tag = "Acps",
    summary = "Get an ACP by id (scope-guarded)",
    params(("id" = Uuid, Path, description = "ACP UUID")),
    responses(
        (status = 200, description = "ACP found", body = crate::application::dto::AcpResponseDto),
        (status = 403, description = "Out of scope (AcpNotInScope)"),
        (status = 404, description = "Not found"),
    ),
    security(("bearer_auth" = []))
)]
#[get("/acps/{id}")]
pub async fn get_acp(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    id: web::Path<Uuid>,
) -> impl Responder {
    let caller = caller_from_user(&user);
    match state.acp_use_cases.get_acp(&caller, *id).await {
        Ok(resp) => HttpResponse::Ok().json(resp),
        Err(err) => err.error_response(),
    }
}

#[utoipa::path(
    put,
    path = "/acps/{id}",
    tag = "Acps",
    summary = "Update an ACP (admin + scope)",
    params(("id" = Uuid, Path, description = "ACP UUID")),
    request_body = UpdateAcpDto,
    responses(
        (status = 200, description = "ACP updated", body = crate::application::dto::AcpResponseDto),
        (status = 400, description = "Validation error"),
        (status = 403, description = "Forbidden / out of scope"),
        (status = 404, description = "Not found"),
    ),
    security(("bearer_auth" = []))
)]
#[put("/acps/{id}")]
pub async fn update_acp(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    id: web::Path<Uuid>,
    dto: web::Json<UpdateAcpDto>,
) -> impl Responder {
    if let Err(errors) = dto.validate() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Validation failed",
            "details": errors.to_string(),
            "kind": "validation",
        }));
    }

    let acp_id = *id;
    let caller = caller_from_user(&user);
    match state
        .acp_use_cases
        .update_acp(&caller, acp_id, dto.into_inner())
        .await
    {
        Ok(resp) => {
            AuditLogEntry::new(
                AuditEventType::AcpUpdated,
                Some(user.user_id),
                user.organization_id,
            )
            .with_resource("Acp", acp_id)
            .log();
            HttpResponse::Ok().json(resp)
        }
        Err(err) => {
            AuditLogEntry::new(
                AuditEventType::AcpUpdated,
                Some(user.user_id),
                user.organization_id,
            )
            .with_resource("Acp", acp_id)
            .with_error(err.to_string())
            .log();
            err.error_response()
        }
    }
}

#[utoipa::path(
    delete,
    path = "/acps/{id}",
    tag = "Acps",
    summary = "Archive (delete) an ACP (admin + scope)",
    params(("id" = Uuid, Path, description = "ACP UUID")),
    responses(
        (status = 204, description = "ACP archived"),
        (status = 403, description = "Forbidden / out of scope"),
        (status = 404, description = "Not found"),
        (
            status = 409,
            description = "ACP still carries buildings — detach or delete them first"
        ),
    ),
    security(("bearer_auth" = []))
)]
#[delete("/acps/{id}")]
pub async fn archive_acp(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    id: web::Path<Uuid>,
) -> impl Responder {
    let acp_id = *id;
    let caller = caller_from_user(&user);
    match state.acp_use_cases.archive_acp(&caller, acp_id).await {
        Ok(()) => {
            AuditLogEntry::new(
                AuditEventType::AcpArchived,
                Some(user.user_id),
                user.organization_id,
            )
            .with_resource("Acp", acp_id)
            .log();
            HttpResponse::NoContent().finish()
        }
        Err(err) => {
            AuditLogEntry::new(
                AuditEventType::AcpArchived,
                Some(user.user_id),
                user.organization_id,
            )
            .with_resource("Acp", acp_id)
            .with_error(err.to_string())
            .log();
            err.error_response()
        }
    }
}
