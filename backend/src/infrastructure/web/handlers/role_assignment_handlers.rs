//! HTTP handlers for Story B0bis — CRUD REST `role-assignments` (gap Story 3.1).
//!
//! Story 3.1 a livré l'entité `UserRoleAssignment` + `UserRoleRepository` +
//! helpers RBAC, mais n'avait **jamais** exposé un endpoint REST pour
//! assigner / lister / révoquer un sous-rôle. Cela bloquait toute UI
//! d'administration (Phase B FE — `RoleAssignmentForm` / `RoleAssignmentList`).
//!
//! Routes:
//! - `POST   /users/{user_id}/role-assignments`
//! - `GET    /users/{user_id}/role-assignments`
//! - `DELETE /users/{user_id}/role-assignments/{assignment_id}`
//! - `GET    /role-assignments?organization_id=&role=` — listing admin filtré.
//!
//! Auth (cohérent avec mandate_handlers / role_delegation_handlers) :
//! - POST    : superadmin OU syndic dans son organization.
//! - GET     : superadmin OU syndic OU le user lui-même (self).
//! - DELETE  : superadmin OU syndic-de-l-org.
//! - GET admin list filtré : superadmin uniquement.

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
pub struct AssignRoleRequest {
    /// Role canonique (ex. "accountant.encodeur", "community.moderator").
    pub role: String,
    /// Organization scope (None = role global).
    pub organization_id: Option<Uuid>,
    /// Si Some, assignment temporaire (delegated) qui expire à cette date.
    /// Si None, assignment native permanente.
    pub valid_until: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct UserRoleAssignmentResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub role: String,
    pub organization_id: Option<Uuid>,
    pub is_primary: bool,
    pub valid_until: Option<DateTime<Utc>>,
    pub delegated_from_user_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<UserRoleAssignment> for UserRoleAssignmentResponse {
    fn from(a: UserRoleAssignment) -> Self {
        Self {
            id: a.id,
            user_id: a.user_id,
            role: a.role.to_string(),
            organization_id: a.organization_id,
            is_primary: a.is_primary,
            valid_until: a.valid_until,
            delegated_from_user_id: a.delegated_from_user_id,
            created_at: a.created_at,
            updated_at: a.updated_at,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListRoleAssignmentsAdminQuery {
    pub organization_id: Option<Uuid>,
    pub role: Option<String>,
}

// ---------------------------------------------------------------------------
// Guards
// ---------------------------------------------------------------------------

/// Le caller peut administrer les role-assignments d'une target :
/// - superadmin global,
/// - OU syndic dans l'organization de la target user (si org alignée).
async fn ensure_can_admin_target(
    state: &web::Data<AppState>,
    caller: &AuthenticatedUser,
    target_user_id: Uuid,
) -> Result<(), AppError> {
    if caller.is_superadmin() {
        return Ok(());
    }
    if caller.role != "syndic" {
        return Err(AppError::Forbidden(
            "Only superadmin or syndic can administer role assignments".to_string(),
        ));
    }
    // Syndic : vérifier que la target est dans la MÊME organization.
    let target = state
        .user_use_cases
        .list_assignments_for_user(target_user_id)
        .await?;
    // On accepte si au moins une assignment partage l'org du caller.
    let same_org = target
        .iter()
        .any(|a| a.organization_id.is_some() && a.organization_id == caller.organization_id);
    // Cas bootstrap : si la target n'a encore AUCUNE assignment, on autorise le
    // syndic à créer la première assignment dans sa propre organization.
    let bootstrap = target.is_empty() && caller.organization_id.is_some();
    if same_org || bootstrap {
        Ok(())
    } else {
        Err(AppError::Forbidden(
            "Syndic can only administer role assignments inside their own organization".to_string(),
        ))
    }
}

// ---------------------------------------------------------------------------
// POST /users/{user_id}/role-assignments
// ---------------------------------------------------------------------------

#[utoipa::path(
    post,
    path = "/users/{user_id}/role-assignments",
    tag = "RoleAssignment",
    summary = "Assign a sub-role to a user (Story B0bis — gap Story 3.1)",
    request_body = AssignRoleRequest,
    params(("user_id" = Uuid, Path, description = "Target user UUID")),
    responses(
        (status = 201, description = "Role assigned", body = UserRoleAssignmentResponse),
        (status = 400, description = "Validation error (unknown role, past valid_until)"),
        (status = 403, description = "Forbidden — not superadmin/syndic"),
        (status = 404, description = "Target user not found"),
        (status = 409, description = "Role already actively assigned to this user"),
    ),
)]
#[post("/users/{user_id}/role-assignments")]
pub async fn assign_role(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    path: web::Path<Uuid>,
    body: web::Json<AssignRoleRequest>,
) -> Result<HttpResponse, AppError> {
    let target_user_id = path.into_inner();
    ensure_can_admin_target(&state, &user, target_user_id).await?;

    let req = body.into_inner();
    let role = UserRole::from_str(&req.role).map_err(AppError::Validation)?;

    let saved = state
        .user_use_cases
        .assign_role(
            target_user_id,
            role,
            req.organization_id,
            req.valid_until,
            user.user_id,
        )
        .await?;
    Ok(HttpResponse::Created().json(UserRoleAssignmentResponse::from(saved)))
}

// ---------------------------------------------------------------------------
// GET /users/{user_id}/role-assignments
// ---------------------------------------------------------------------------

#[utoipa::path(
    get,
    path = "/users/{user_id}/role-assignments",
    tag = "RoleAssignment",
    summary = "List role assignments for a user",
    params(("user_id" = Uuid, Path, description = "Target user UUID")),
    responses(
        (status = 200, description = "Role assignments", body = Vec<UserRoleAssignmentResponse>),
        (status = 403, description = "Forbidden"),
    ),
)]
#[get("/users/{user_id}/role-assignments")]
pub async fn list_role_assignments_for_user(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let target_user_id = path.into_inner();
    let is_self = target_user_id == user.user_id;
    if !is_self {
        ensure_can_admin_target(&state, &user, target_user_id).await?;
    }
    let rows = state
        .user_use_cases
        .list_assignments_for_user(target_user_id)
        .await?;
    let response: Vec<UserRoleAssignmentResponse> = rows
        .into_iter()
        .map(UserRoleAssignmentResponse::from)
        .collect();
    Ok(HttpResponse::Ok().json(response))
}

// ---------------------------------------------------------------------------
// DELETE /users/{user_id}/role-assignments/{assignment_id}
// ---------------------------------------------------------------------------

#[utoipa::path(
    delete,
    path = "/users/{user_id}/role-assignments/{assignment_id}",
    tag = "RoleAssignment",
    summary = "Revoke a role assignment",
    params(
        ("user_id" = Uuid, Path, description = "Target user UUID"),
        ("assignment_id" = Uuid, Path, description = "Assignment UUID to revoke"),
    ),
    responses(
        (status = 204, description = "Assignment revoked"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Assignment not found"),
    ),
)]
#[delete("/users/{user_id}/role-assignments/{assignment_id}")]
pub async fn revoke_role_assignment(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    path: web::Path<(Uuid, Uuid)>,
) -> Result<HttpResponse, AppError> {
    let (target_user_id, assignment_id) = path.into_inner();
    ensure_can_admin_target(&state, &user, target_user_id).await?;
    state
        .user_use_cases
        .revoke_assignment(assignment_id)
        .await?;
    Ok(HttpResponse::NoContent().finish())
}

// ---------------------------------------------------------------------------
// GET /role-assignments?organization_id=&role= — admin filtered list
// ---------------------------------------------------------------------------

#[utoipa::path(
    get,
    path = "/role-assignments",
    tag = "RoleAssignment",
    summary = "List role assignments filtered by organization and/or role (superadmin only)",
    params(
        ("organization_id" = Option<Uuid>, Query, description = "Filter by organization"),
        ("role" = Option<String>, Query, description = "Filter by role string (whitelist)"),
    ),
    responses(
        (status = 200, description = "Filtered role assignments", body = Vec<UserRoleAssignmentResponse>),
        (status = 403, description = "Forbidden — superadmin only"),
    ),
)]
#[get("/role-assignments")]
pub async fn list_role_assignments_admin(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    query: web::Query<ListRoleAssignmentsAdminQuery>,
) -> Result<HttpResponse, AppError> {
    if !user.is_superadmin() {
        return Err(AppError::Forbidden(
            "Only superadmin can list cross-user role assignments".to_string(),
        ));
    }
    let q = query.into_inner();
    // Validate role if provided (refuse arbitrary strings — INV cohérent
    // avec UserRole::from_str whitelist).
    let role_filter = match q.role.as_deref() {
        None => None,
        Some(s) => Some(UserRole::from_str(s).map_err(AppError::Validation)?),
    };

    // No bulk admin query exists in the trait — we list per-user via
    // `list_all` then filter. SuperAdmin is the only caller; the cost is
    // acceptable for an admin tool. A dedicated repo method can be added
    // later if perf becomes a concern.
    let all_users = state
        .user_use_cases
        .list_all()
        .await
        .map_err(AppError::from)?;
    let mut rows: Vec<UserRoleAssignmentResponse> = Vec::new();
    for u in all_users {
        let uid = Uuid::parse_str(&u.id).map_err(|e| AppError::Internal(e.to_string()))?;
        let assignments = state.user_use_cases.list_assignments_for_user(uid).await?;
        for a in assignments {
            let role_match = match &role_filter {
                None => true,
                Some(r) => &a.role == r,
            };
            let org_match = match q.organization_id {
                None => true,
                Some(org) => a.organization_id == Some(org),
            };
            if role_match && org_match {
                rows.push(UserRoleAssignmentResponse::from(a));
            }
        }
    }
    Ok(HttpResponse::Ok().json(rows))
}
