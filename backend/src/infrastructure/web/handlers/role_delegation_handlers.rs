//! HTTP handlers for Story 3.5 — Temporary role delegation (FR8 INV-8).
//!
//! Routes:
//! - `POST   /role-delegations`        — delegate a role to another user.
//! - `DELETE /role-delegations/{id}`   — revoke an active delegation.
//! - `GET    /role-delegations?subject={u}` — list delegations of `subject`
//!   (admin only OR the subject themselves).
//!
//! Auth: the caller MUST hold the role they want to delegate (checked via
//! their JWT primary role string; the use-case re-checks the *native*
//! invariant by inspecting persisted assignments).

use crate::application::error::AppError;
use crate::domain::entities::{UserRole, UserRoleAssignment};
use crate::infrastructure::web::{AppState, AuthenticatedUser};
use actix_web::{delete, get, post, web, HttpResponse};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// DTOs
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct DelegateRoleRequest {
    pub target_user_id: Uuid,
    pub role: String,
    pub organization_id: Option<Uuid>,
    pub valid_until: DateTime<Utc>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct RoleDelegationResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub role: String,
    pub organization_id: Option<Uuid>,
    pub valid_until: Option<DateTime<Utc>>,
    pub delegated_from_user_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<UserRoleAssignment> for RoleDelegationResponse {
    fn from(a: UserRoleAssignment) -> Self {
        Self {
            id: a.id,
            user_id: a.user_id,
            role: a.role.to_string(),
            organization_id: a.organization_id,
            valid_until: a.valid_until,
            delegated_from_user_id: a.delegated_from_user_id,
            created_at: a.created_at,
            updated_at: a.updated_at,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListDelegationsQuery {
    pub subject: Option<Uuid>,
}

// ---------------------------------------------------------------------------
// Guards
// ---------------------------------------------------------------------------

/// The caller must have the role they want to delegate as their primary role.
/// The use-case will additionally verify they hold it *natively* (not via a
/// prior delegation — INV-8 non-transitive).
fn caller_must_hold_role(user: &AuthenticatedUser, role: &UserRole) -> Result<(), AppError> {
    if user.role == role.to_string() {
        return Ok(());
    }
    // Superadmin shortcut — Story 3.1 helpers grant blanket authority.
    if user.role == "superadmin" {
        return Ok(());
    }
    Err(AppError::Forbidden(format!(
        "Caller does not hold role '{}'",
        role
    )))
}

// ---------------------------------------------------------------------------
// POST /role-delegations
// ---------------------------------------------------------------------------

#[utoipa::path(
    post,
    path = "/role-delegations",
    tag = "RoleDelegation",
    summary = "Delegate a role to another user for a bounded duration",
    responses(
        (status = 201, description = "Delegation created", body = RoleDelegationResponse),
        (status = 400, description = "Validation error"),
        (status = 403, description = "Forbidden — caller cannot delegate this role"),
        (status = 409, description = "Target already holds the role"),
    ),
)]
#[post("/role-delegations")]
pub async fn create_role_delegation(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    body: web::Json<DelegateRoleRequest>,
) -> Result<HttpResponse, AppError> {
    let req = body.into_inner();
    let role = UserRole::from_str(&req.role).map_err(AppError::Validation)?;
    caller_must_hold_role(&user, &role)?;

    let delegation = state
        .role_delegation_use_cases
        .delegate_role(
            user.user_id,
            req.target_user_id,
            role,
            req.organization_id,
            req.valid_until,
        )
        .await?;

    Ok(HttpResponse::Created().json(RoleDelegationResponse::from(delegation)))
}

// ---------------------------------------------------------------------------
// DELETE /role-delegations/{id}
// ---------------------------------------------------------------------------

#[utoipa::path(
    delete,
    path = "/role-delegations/{id}",
    tag = "RoleDelegation",
    summary = "Revoke a delegation before its natural expiry",
    responses(
        (status = 204, description = "Delegation revoked"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Delegation not found"),
    ),
)]
#[delete("/role-delegations/{id}")]
pub async fn revoke_role_delegation(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let id = path.into_inner();
    // Only the original delegator (or a superadmin) may revoke. We resolve
    // the row to check the delegator field.
    let existing = state
        .role_delegation_use_cases
        .list_delegations_of(user.user_id)
        .await?;
    let is_admin = user.role == "superadmin";
    let is_owner_of_delegation = existing.iter().any(|a| {
        a.id == id && (a.delegated_from_user_id == Some(user.user_id) || a.user_id == user.user_id)
    });
    if !is_admin && !is_owner_of_delegation {
        return Err(AppError::Forbidden(
            "Only the delegator, the subject or a superadmin can revoke".to_string(),
        ));
    }
    state
        .role_delegation_use_cases
        .revoke_delegation(id)
        .await?;
    Ok(HttpResponse::NoContent().finish())
}

// ---------------------------------------------------------------------------
// GET /role-delegations?subject={uuid}
// ---------------------------------------------------------------------------

#[utoipa::path(
    get,
    path = "/role-delegations",
    tag = "RoleDelegation",
    summary = "List active delegations involving a subject user",
    params(
        ("subject" = Option<Uuid>, Query, description = "Subject user id. Defaults to the caller.")
    ),
    responses(
        (status = 200, description = "Active delegations", body = Vec<RoleDelegationResponse>),
        (status = 403, description = "Forbidden — caller cannot view this subject's delegations"),
    ),
)]
#[get("/role-delegations")]
pub async fn list_role_delegations(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    query: web::Query<ListDelegationsQuery>,
) -> Result<HttpResponse, AppError> {
    let subject = query.into_inner().subject.unwrap_or(user.user_id);
    let is_self = subject == user.user_id;
    let is_admin = matches!(user.role.as_str(), "syndic" | "superadmin");
    if !is_self && !is_admin {
        return Err(AppError::Forbidden(
            "Only the subject themselves or a syndic/superadmin can list delegations".to_string(),
        ));
    }
    let delegations = state
        .role_delegation_use_cases
        .list_delegations_of(subject)
        .await?;
    let response: Vec<RoleDelegationResponse> = delegations
        .into_iter()
        .map(RoleDelegationResponse::from)
        .collect();
    Ok(HttpResponse::Ok().json(response))
}
