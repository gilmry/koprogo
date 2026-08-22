//! Handlers Actix pour `/portfolios` — Story 2.1.
//!
//! Endpoints :
//! - POST   `/portfolios`                                : create
//! - GET    `/portfolios`                                : list (owner + shared)
//! - GET    `/portfolios/{id}`                           : get (scope-guarded)
//! - PUT    `/portfolios/{id}`                           : update (owner OR shared can_edit)
//! - DELETE `/portfolios/{id}`                           : delete (owner)
//! - POST   `/portfolios/{id}/buildings`                 : add building (owner OR can_edit)
//! - GET    `/portfolios/{id}/buildings`                 : list buildings (owner OR shared)
//! - DELETE `/portfolios/{id}/buildings/{building_id}`   : remove building
//! - POST   `/portfolios/{id}/shares`                    : share (owner)
//! - GET    `/portfolios/{id}/shares`                    : list shares (owner)
//! - DELETE `/portfolios/{id}/shares/{user_id}`          : unshare (owner)
//!
//! Audit : `infrastructure::audit::AuditLogEntry` consigne chaque mutation
//! réussie ET chaque échec (traçabilité INV-24, pattern ACP Story 1.1).

use crate::application::dto::{
    AddBuildingDto, CreatePortfolioDto, SharePortfolioDto, UpdatePortfolioDto,
};
use crate::application::use_cases::PortfolioCaller;
use crate::infrastructure::audit::{AuditEventType, AuditLogEntry};
use crate::infrastructure::web::{AppState, AuthenticatedUser};
use actix_web::{delete, get, post, put, web, HttpResponse, Responder, ResponseError};
use uuid::Uuid;
use validator::Validate;

fn caller_from_user(user: &AuthenticatedUser) -> PortfolioCaller {
    PortfolioCaller {
        user_id: user.user_id,
    }
}

#[utoipa::path(
    post,
    path = "/portfolios",
    tag = "Portfolios",
    summary = "Create a portfolio",
    request_body = CreatePortfolioDto,
    responses(
        (status = 201, description = "Portfolio created", body = crate::application::dto::PortfolioResponseDto),
        (status = 400, description = "Validation error"),
        (status = 401, description = "Unauthorized"),
    ),
    security(("bearer_auth" = []))
)]
#[post("/portfolios")]
pub async fn create_portfolio(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    dto: web::Json<CreatePortfolioDto>,
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
        .portfolio_use_cases
        .create_portfolio(&caller, dto.into_inner())
        .await
    {
        Ok(resp) => {
            if let Ok(pid) = Uuid::parse_str(&resp.id) {
                AuditLogEntry::new(
                    AuditEventType::PortfolioCreated,
                    Some(user.user_id),
                    user.organization_id,
                )
                .with_resource("Portfolio", pid)
                .log();
            }
            HttpResponse::Created().json(resp)
        }
        Err(err) => {
            AuditLogEntry::new(
                AuditEventType::PortfolioCreated,
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
    path = "/portfolios",
    tag = "Portfolios",
    summary = "List portfolios visible to the authenticated user (owned + shared)",
    responses(
        (status = 200, description = "List of portfolios", body = Vec<crate::application::dto::PortfolioResponseDto>),
        (status = 401, description = "Unauthorized"),
    ),
    security(("bearer_auth" = []))
)]
#[get("/portfolios")]
pub async fn list_portfolios(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
) -> impl Responder {
    let caller = caller_from_user(&user);
    match state.portfolio_use_cases.list_portfolios(&caller).await {
        Ok(list) => HttpResponse::Ok().json(list),
        Err(err) => err.error_response(),
    }
}

#[utoipa::path(
    get,
    path = "/portfolios/{id}",
    tag = "Portfolios",
    summary = "Get a portfolio by id (owner OR shared)",
    params(("id" = Uuid, Path, description = "Portfolio UUID")),
    responses(
        (status = 200, description = "Portfolio found", body = crate::application::dto::PortfolioResponseDto),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found"),
    ),
    security(("bearer_auth" = []))
)]
#[get("/portfolios/{id}")]
pub async fn get_portfolio(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    id: web::Path<Uuid>,
) -> impl Responder {
    let caller = caller_from_user(&user);
    match state.portfolio_use_cases.get_portfolio(&caller, *id).await {
        Ok(resp) => HttpResponse::Ok().json(resp),
        Err(err) => err.error_response(),
    }
}

#[utoipa::path(
    put,
    path = "/portfolios/{id}",
    tag = "Portfolios",
    summary = "Update a portfolio (owner OR shared can_edit)",
    params(("id" = Uuid, Path, description = "Portfolio UUID")),
    request_body = UpdatePortfolioDto,
    responses(
        (status = 200, description = "Portfolio updated", body = crate::application::dto::PortfolioResponseDto),
        (status = 400, description = "Validation error"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found"),
    ),
    security(("bearer_auth" = []))
)]
#[put("/portfolios/{id}")]
pub async fn update_portfolio(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    id: web::Path<Uuid>,
    dto: web::Json<UpdatePortfolioDto>,
) -> impl Responder {
    if let Err(errors) = dto.validate() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Validation failed",
            "details": errors.to_string(),
            "kind": "validation",
        }));
    }
    let pid = *id;
    let caller = caller_from_user(&user);
    match state
        .portfolio_use_cases
        .update_portfolio(&caller, pid, dto.into_inner())
        .await
    {
        Ok(resp) => {
            AuditLogEntry::new(
                AuditEventType::PortfolioUpdated,
                Some(user.user_id),
                user.organization_id,
            )
            .with_resource("Portfolio", pid)
            .log();
            HttpResponse::Ok().json(resp)
        }
        Err(err) => {
            AuditLogEntry::new(
                AuditEventType::PortfolioUpdated,
                Some(user.user_id),
                user.organization_id,
            )
            .with_resource("Portfolio", pid)
            .with_error(err.to_string())
            .log();
            err.error_response()
        }
    }
}

#[utoipa::path(
    delete,
    path = "/portfolios/{id}",
    tag = "Portfolios",
    summary = "Delete a portfolio (owner only)",
    params(("id" = Uuid, Path, description = "Portfolio UUID")),
    responses(
        (status = 204, description = "Portfolio deleted"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found"),
    ),
    security(("bearer_auth" = []))
)]
#[delete("/portfolios/{id}")]
pub async fn delete_portfolio(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    id: web::Path<Uuid>,
) -> impl Responder {
    let pid = *id;
    let caller = caller_from_user(&user);
    match state
        .portfolio_use_cases
        .delete_portfolio(&caller, pid)
        .await
    {
        Ok(()) => {
            AuditLogEntry::new(
                AuditEventType::PortfolioDeleted,
                Some(user.user_id),
                user.organization_id,
            )
            .with_resource("Portfolio", pid)
            .log();
            HttpResponse::NoContent().finish()
        }
        Err(err) => {
            AuditLogEntry::new(
                AuditEventType::PortfolioDeleted,
                Some(user.user_id),
                user.organization_id,
            )
            .with_resource("Portfolio", pid)
            .with_error(err.to_string())
            .log();
            err.error_response()
        }
    }
}

// ============================================================================
// Buildings
// ============================================================================

#[utoipa::path(
    post,
    path = "/portfolios/{id}/buildings",
    tag = "Portfolios",
    summary = "Add a building to a portfolio (owner OR shared can_edit)",
    params(("id" = Uuid, Path, description = "Portfolio UUID")),
    request_body = AddBuildingDto,
    responses(
        (status = 201, description = "Building added", body = crate::application::dto::PortfolioBuildingResponseDto),
        (status = 400, description = "Validation error"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Portfolio or Building not found"),
    ),
    security(("bearer_auth" = []))
)]
#[post("/portfolios/{id}/buildings")]
pub async fn add_portfolio_building(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    id: web::Path<Uuid>,
    dto: web::Json<AddBuildingDto>,
) -> impl Responder {
    let pid = *id;
    let caller = caller_from_user(&user);
    match state
        .portfolio_use_cases
        .add_building(&caller, pid, dto.into_inner())
        .await
    {
        Ok(resp) => {
            AuditLogEntry::new(
                AuditEventType::PortfolioBuildingAdded,
                Some(user.user_id),
                user.organization_id,
            )
            .with_resource("Portfolio", pid)
            .log();
            HttpResponse::Created().json(resp)
        }
        Err(err) => {
            AuditLogEntry::new(
                AuditEventType::PortfolioBuildingAdded,
                Some(user.user_id),
                user.organization_id,
            )
            .with_resource("Portfolio", pid)
            .with_error(err.to_string())
            .log();
            err.error_response()
        }
    }
}

#[utoipa::path(
    get,
    path = "/portfolios/{id}/buildings",
    tag = "Portfolios",
    summary = "List buildings of a portfolio (favorites first)",
    params(("id" = Uuid, Path, description = "Portfolio UUID")),
    responses(
        (status = 200, description = "List of buildings", body = Vec<crate::application::dto::PortfolioBuildingResponseDto>),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Portfolio not found"),
    ),
    security(("bearer_auth" = []))
)]
#[get("/portfolios/{id}/buildings")]
pub async fn list_portfolio_buildings(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    id: web::Path<Uuid>,
) -> impl Responder {
    let pid = *id;
    let caller = caller_from_user(&user);
    match state.portfolio_use_cases.list_buildings(&caller, pid).await {
        Ok(list) => HttpResponse::Ok().json(list),
        Err(err) => err.error_response(),
    }
}

#[utoipa::path(
    delete,
    path = "/portfolios/{id}/buildings/{building_id}",
    tag = "Portfolios",
    summary = "Remove a building from a portfolio",
    params(
        ("id" = Uuid, Path, description = "Portfolio UUID"),
        ("building_id" = Uuid, Path, description = "Building UUID"),
    ),
    responses(
        (status = 204, description = "Building removed"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found"),
    ),
    security(("bearer_auth" = []))
)]
#[delete("/portfolios/{id}/buildings/{building_id}")]
pub async fn remove_portfolio_building(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    path: web::Path<(Uuid, Uuid)>,
) -> impl Responder {
    let (pid, bid) = path.into_inner();
    let caller = caller_from_user(&user);
    match state
        .portfolio_use_cases
        .remove_building(&caller, pid, bid)
        .await
    {
        Ok(()) => {
            AuditLogEntry::new(
                AuditEventType::PortfolioBuildingRemoved,
                Some(user.user_id),
                user.organization_id,
            )
            .with_resource("Portfolio", pid)
            .log();
            HttpResponse::NoContent().finish()
        }
        Err(err) => {
            AuditLogEntry::new(
                AuditEventType::PortfolioBuildingRemoved,
                Some(user.user_id),
                user.organization_id,
            )
            .with_resource("Portfolio", pid)
            .with_error(err.to_string())
            .log();
            err.error_response()
        }
    }
}

// ============================================================================
// Shares
// ============================================================================

#[utoipa::path(
    post,
    path = "/portfolios/{id}/shares",
    tag = "Portfolios",
    summary = "Share a portfolio with another user (owner only)",
    params(("id" = Uuid, Path, description = "Portfolio UUID")),
    request_body = SharePortfolioDto,
    responses(
        (status = 201, description = "Portfolio shared", body = crate::application::dto::PortfolioShareResponseDto),
        (status = 400, description = "Validation error"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Portfolio or User not found"),
    ),
    security(("bearer_auth" = []))
)]
#[post("/portfolios/{id}/shares")]
pub async fn share_portfolio(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    id: web::Path<Uuid>,
    dto: web::Json<SharePortfolioDto>,
) -> impl Responder {
    let pid = *id;
    let caller = caller_from_user(&user);
    match state
        .portfolio_use_cases
        .share_with(&caller, pid, dto.into_inner())
        .await
    {
        Ok(resp) => {
            AuditLogEntry::new(
                AuditEventType::PortfolioShared,
                Some(user.user_id),
                user.organization_id,
            )
            .with_resource("Portfolio", pid)
            .log();
            HttpResponse::Created().json(resp)
        }
        Err(err) => {
            AuditLogEntry::new(
                AuditEventType::PortfolioShared,
                Some(user.user_id),
                user.organization_id,
            )
            .with_resource("Portfolio", pid)
            .with_error(err.to_string())
            .log();
            err.error_response()
        }
    }
}

#[utoipa::path(
    get,
    path = "/portfolios/{id}/shares",
    tag = "Portfolios",
    summary = "List shares of a portfolio (owner only)",
    params(("id" = Uuid, Path, description = "Portfolio UUID")),
    responses(
        (status = 200, description = "List of shares", body = Vec<crate::application::dto::PortfolioShareResponseDto>),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found"),
    ),
    security(("bearer_auth" = []))
)]
#[get("/portfolios/{id}/shares")]
pub async fn list_portfolio_shares(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    id: web::Path<Uuid>,
) -> impl Responder {
    let pid = *id;
    let caller = caller_from_user(&user);
    match state.portfolio_use_cases.list_shares(&caller, pid).await {
        Ok(list) => HttpResponse::Ok().json(list),
        Err(err) => err.error_response(),
    }
}

#[utoipa::path(
    delete,
    path = "/portfolios/{id}/shares/{user_id}",
    tag = "Portfolios",
    summary = "Unshare a portfolio (owner only)",
    params(
        ("id" = Uuid, Path, description = "Portfolio UUID"),
        ("user_id" = Uuid, Path, description = "Shared user UUID"),
    ),
    responses(
        (status = 204, description = "Unshared"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Share not found"),
    ),
    security(("bearer_auth" = []))
)]
#[delete("/portfolios/{id}/shares/{user_id}")]
pub async fn unshare_portfolio(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    path: web::Path<(Uuid, Uuid)>,
) -> impl Responder {
    let (pid, uid) = path.into_inner();
    let caller = caller_from_user(&user);
    match state.portfolio_use_cases.unshare(&caller, pid, uid).await {
        Ok(()) => {
            AuditLogEntry::new(
                AuditEventType::PortfolioUnshared,
                Some(user.user_id),
                user.organization_id,
            )
            .with_resource("Portfolio", pid)
            .log();
            HttpResponse::NoContent().finish()
        }
        Err(err) => {
            AuditLogEntry::new(
                AuditEventType::PortfolioUnshared,
                Some(user.user_id),
                user.organization_id,
            )
            .with_resource("Portfolio", pid)
            .with_error(err.to_string())
            .log();
            err.error_response()
        }
    }
}
