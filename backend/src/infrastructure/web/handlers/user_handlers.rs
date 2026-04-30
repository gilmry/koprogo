use crate::domain::entities::UserRole;
use crate::domain::entities::UserRoleAssignment;
use crate::infrastructure::web::{AppState, AuthenticatedUser};
use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use bcrypt::{hash, DEFAULT_COST};
use serde::Deserialize;
use serde_json::json;
use std::collections::HashSet;
use uuid::Uuid;

const ALLOWED_ROLES: [&str; 4] = ["superadmin", "syndic", "accountant", "owner"];

#[derive(Deserialize, Clone)]
pub struct RoleAssignmentRequest {
    pub role: String,
    pub organization_id: Option<Uuid>,
    pub is_primary: Option<bool>,
}

#[derive(Deserialize)]
pub struct CreateUserRequest {
    pub email: String,
    pub password: String,
    pub first_name: String,
    pub last_name: String,
    pub roles: Option<Vec<RoleAssignmentRequest>>,
    pub role: Option<String>,          // backward compatibility
    pub organization_id: Option<Uuid>, // backward compatibility
}

#[derive(Deserialize)]
pub struct UpdateUserRequest {
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub roles: Option<Vec<RoleAssignmentRequest>>,
    pub role: Option<String>,          // backward compatibility
    pub organization_id: Option<Uuid>, // backward compatibility
    pub password: Option<String>,
}

#[derive(Clone, Debug)]
struct NormalizedRole {
    id: Uuid,
    role: String,
    organization_id: Option<Uuid>,
    is_primary: bool,
}

impl NormalizedRole {
    fn to_assignment(&self, user_id: Uuid) -> UserRoleAssignment {
        let domain_role = self.role.parse::<UserRole>().expect("already validated");
        let mut a =
            UserRoleAssignment::new(user_id, domain_role, self.organization_id, self.is_primary);
        a.id = self.id;
        a
    }
}

/// GET /api/v1/users — list all users (SuperAdmin only)
#[get("/users")]
pub async fn list_users(state: web::Data<AppState>, user: AuthenticatedUser) -> impl Responder {
    if user.role != "superadmin" {
        return HttpResponse::Forbidden().json(json!({
            "error": "Only SuperAdmin can access all users"
        }));
    }

    match state.user_use_cases.list_all().await {
        Ok(users) => HttpResponse::Ok().json(json!({ "data": users })),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "error": format!("Failed to fetch users: {}", e)
        })),
    }
}

/// POST /api/v1/users — create user (SuperAdmin only)
#[post("/users")]
pub async fn create_user(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    req: web::Json<CreateUserRequest>,
) -> impl Responder {
    if user.role != "superadmin" {
        return HttpResponse::Forbidden().json(json!({
            "error": "Only SuperAdmin can create users"
        }));
    }

    if !req.email.contains('@') {
        return HttpResponse::BadRequest().json(json!({ "error": "Invalid email format" }));
    }
    if req.first_name.trim().len() < 2 || req.last_name.trim().len() < 2 {
        return HttpResponse::BadRequest().json(json!({
            "error": "First and last names must be at least 2 characters"
        }));
    }
    if req.password.trim().len() < 6 {
        return HttpResponse::BadRequest().json(json!({
            "error": "Password must be at least 6 characters"
        }));
    }

    let roles = match normalize_roles(req.roles.clone(), req.role.clone(), req.organization_id) {
        Ok(r) => r,
        Err(resp) => return resp,
    };

    let primary = roles
        .iter()
        .find(|r| r.is_primary)
        .cloned()
        .expect("normalized roles always have a primary");

    let hashed_password = match hash(req.password.trim(), DEFAULT_COST) {
        Ok(h) => h,
        Err(e) => {
            return HttpResponse::InternalServerError().json(json!({
                "error": format!("Failed to hash password: {}", e)
            }))
        }
    };

    let assignments: Vec<UserRoleAssignment> = roles
        .iter()
        .map(|r| r.to_assignment(Uuid::nil())) // user_id filled by use case
        .collect();

    let primary_role = primary.role.parse::<UserRole>().expect("already validated");

    match state
        .user_use_cases
        .create(
            req.email.trim().to_lowercase(),
            hashed_password,
            req.first_name.trim().to_string(),
            req.last_name.trim().to_string(),
            primary_role,
            primary.organization_id,
            assignments,
        )
        .await
    {
        Ok(resp) => HttpResponse::Created().json(resp),
        Err(e) if e == "email_exists" => HttpResponse::BadRequest().json(json!({
            "error": "Email already exists"
        })),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "error": format!("Failed to create user: {}", e)
        })),
    }
}

/// PUT /api/v1/users/{id} — update user (SuperAdmin only)
#[put("/users/{id}")]
pub async fn update_user(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    path: web::Path<Uuid>,
    req: web::Json<UpdateUserRequest>,
) -> impl Responder {
    if user.role != "superadmin" {
        return HttpResponse::Forbidden().json(json!({
            "error": "Only SuperAdmin can update users"
        }));
    }

    if !req.email.contains('@') {
        return HttpResponse::BadRequest().json(json!({ "error": "Invalid email format" }));
    }
    if req.first_name.trim().len() < 2 || req.last_name.trim().len() < 2 {
        return HttpResponse::BadRequest().json(json!({
            "error": "First and last names must be at least 2 characters"
        }));
    }
    if let Some(password) = &req.password {
        if !password.trim().is_empty() && password.trim().len() < 6 {
            return HttpResponse::BadRequest().json(json!({
                "error": "Password must be at least 6 characters"
            }));
        }
    }

    let roles = match normalize_roles(req.roles.clone(), req.role.clone(), req.organization_id) {
        Ok(r) => r,
        Err(resp) => return resp,
    };

    let primary = roles
        .iter()
        .find(|r| r.is_primary)
        .cloned()
        .expect("normalized roles always have a primary");

    let user_id = path.into_inner();

    let password_hash = if let Some(pw) = &req.password {
        if !pw.trim().is_empty() {
            match hash(pw.trim(), DEFAULT_COST) {
                Ok(h) => Some(h),
                Err(e) => {
                    return HttpResponse::InternalServerError().json(json!({
                        "error": format!("Failed to hash password: {}", e)
                    }))
                }
            }
        } else {
            None
        }
    } else {
        None
    };

    let assignments: Vec<UserRoleAssignment> =
        roles.iter().map(|r| r.to_assignment(user_id)).collect();

    let primary_role = primary.role.parse::<UserRole>().expect("already validated");

    match state
        .user_use_cases
        .update(
            user_id,
            req.email.trim().to_lowercase(),
            req.first_name.trim().to_string(),
            req.last_name.trim().to_string(),
            primary_role,
            primary.organization_id,
            password_hash,
            assignments,
        )
        .await
    {
        Ok(Some(resp)) => HttpResponse::Ok().json(resp),
        Ok(None) => HttpResponse::NotFound().json(json!({ "error": "User not found" })),
        Err(e) if e == "email_exists" => HttpResponse::BadRequest().json(json!({
            "error": "Email already exists"
        })),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "error": format!("Failed to update user: {}", e)
        })),
    }
}

/// PUT /api/v1/users/{id}/activate — activate user (SuperAdmin only)
#[put("/users/{id}/activate")]
pub async fn activate_user(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    path: web::Path<Uuid>,
) -> impl Responder {
    if user.role != "superadmin" {
        return HttpResponse::Forbidden().json(json!({
            "error": "Only SuperAdmin can activate users"
        }));
    }

    let user_id = path.into_inner();
    match state.user_use_cases.activate(user_id).await {
        Ok(Some(resp)) => HttpResponse::Ok().json(resp),
        Ok(None) => HttpResponse::NotFound().json(json!({ "error": "User not found" })),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "error": format!("Failed to activate user: {}", e)
        })),
    }
}

/// PUT /api/v1/users/{id}/deactivate — deactivate user (SuperAdmin only)
#[put("/users/{id}/deactivate")]
pub async fn deactivate_user(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    path: web::Path<Uuid>,
) -> impl Responder {
    if user.role != "superadmin" {
        return HttpResponse::Forbidden().json(json!({
            "error": "Only SuperAdmin can deactivate users"
        }));
    }

    let user_id = path.into_inner();
    match state.user_use_cases.deactivate(user_id).await {
        Ok(Some(resp)) => HttpResponse::Ok().json(resp),
        Ok(None) => HttpResponse::NotFound().json(json!({ "error": "User not found" })),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "error": format!("Failed to deactivate user: {}", e)
        })),
    }
}

/// DELETE /api/v1/users/{id} — delete user (SuperAdmin only)
#[delete("/users/{id}")]
pub async fn delete_user(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    path: web::Path<Uuid>,
) -> impl Responder {
    if user.role != "superadmin" {
        return HttpResponse::Forbidden().json(json!({
            "error": "Only SuperAdmin can delete users"
        }));
    }

    let user_id = path.into_inner();

    if user.user_id == user_id {
        return HttpResponse::BadRequest().json(json!({
            "error": "Cannot delete your own account"
        }));
    }

    match state.user_use_cases.delete(user_id).await {
        Ok(true) => HttpResponse::Ok().json(json!({ "message": "User deleted successfully" })),
        Ok(false) => HttpResponse::NotFound().json(json!({ "error": "User not found" })),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "error": format!("Failed to delete user: {}", e)
        })),
    }
}

// ---------------------------------------------------------------------------
// Pure helper functions — no DB access, kept here because they have tests
// ---------------------------------------------------------------------------

fn normalize_roles(
    roles: Option<Vec<RoleAssignmentRequest>>,
    fallback_role: Option<String>,
    fallback_org: Option<Uuid>,
) -> Result<Vec<NormalizedRole>, HttpResponse> {
    let mut entries = roles.unwrap_or_else(|| {
        fallback_role
            .map(|role| {
                vec![RoleAssignmentRequest {
                    role,
                    organization_id: fallback_org,
                    is_primary: Some(true),
                }]
            })
            .unwrap_or_default()
    });

    if entries.is_empty() {
        return Err(HttpResponse::BadRequest().json(json!({
            "error": "At least one role must be specified"
        })));
    }

    let mut normalized = Vec::with_capacity(entries.len());
    let mut seen = HashSet::new();
    let mut primary_count = 0;

    for entry in entries.drain(..) {
        let RoleAssignmentRequest {
            role,
            organization_id,
            is_primary,
        } = entry;
        let normalized_role = role.trim().to_lowercase();
        if !ALLOWED_ROLES.contains(&normalized_role.as_str()) {
            return Err(HttpResponse::BadRequest().json(json!({
                "error": format!("Invalid role: {}", role)
            })));
        }

        let mut organization_id = organization_id;
        if normalized_role != "superadmin" {
            if organization_id.is_none() {
                return Err(HttpResponse::BadRequest().json(json!({
                    "error": format!("Organization is required for role {}", normalized_role)
                })));
            }
        } else {
            organization_id = None;
        }

        let is_primary = is_primary.unwrap_or(false);
        if is_primary {
            primary_count += 1;
            if primary_count > 1 {
                return Err(HttpResponse::BadRequest().json(json!({
                    "error": "Only one primary role can be specified"
                })));
            }
        }

        let key = (normalized_role.clone(), organization_id);
        if !seen.insert(key) {
            return Err(HttpResponse::BadRequest().json(json!({
                "error": "Duplicate role assignment detected"
            })));
        }

        normalized.push(NormalizedRole {
            id: Uuid::new_v4(),
            role: normalized_role,
            organization_id,
            is_primary,
        });
    }

    if primary_count == 0 {
        if let Some(first) = normalized.first_mut() {
            first.is_primary = true;
        }
    }

    normalized.sort_by_key(|r| std::cmp::Reverse(r.is_primary));
    Ok(normalized)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::use_cases::user_use_cases::RoleResponse;
    use actix_web::http::StatusCode;

    fn normalize_primary_role(roles: &mut [RoleResponse]) {
        if roles.is_empty() {
            return;
        }
        if roles.iter().filter(|r| r.is_primary).count() == 0 {
            roles[0].is_primary = true;
        }
        roles.sort_by_key(|r| std::cmp::Reverse(r.is_primary));
    }

    #[test]
    fn normalize_roles_marks_first_as_primary_when_none_provided() {
        let primary_org = Uuid::new_v4();
        let secondary_org = Uuid::new_v4();
        let input = vec![
            RoleAssignmentRequest {
                role: "syndic".to_string(),
                organization_id: Some(primary_org),
                is_primary: None,
            },
            RoleAssignmentRequest {
                role: "accountant".to_string(),
                organization_id: Some(secondary_org),
                is_primary: Some(false),
            },
        ];

        let normalized = normalize_roles(Some(input), None, None).expect("normalized roles");
        assert_eq!(normalized.len(), 2);
        assert!(
            normalized.first().unwrap().is_primary,
            "first role should become primary"
        );
        assert_eq!(
            normalized.first().unwrap().organization_id,
            Some(primary_org)
        );
        assert_eq!(normalized.first().unwrap().role, "syndic");
    }

    #[test]
    fn normalize_roles_rejects_invalid_role() {
        let res = normalize_roles(
            Some(vec![RoleAssignmentRequest {
                role: "invalid-role".to_string(),
                organization_id: None,
                is_primary: None,
            }]),
            None,
            None,
        );

        let err = res.expect_err("invalid role should fail");
        assert_eq!(err.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn normalize_roles_requires_org_for_non_superadmin() {
        let res = normalize_roles(
            Some(vec![RoleAssignmentRequest {
                role: "syndic".to_string(),
                organization_id: None,
                is_primary: Some(true),
            }]),
            None,
            None,
        );

        let err = res.expect_err("organization required");
        assert_eq!(err.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn normalize_roles_uses_fallback_when_no_roles_provided() {
        let fallback_org = Uuid::new_v4();
        let roles = normalize_roles(None, Some("syndic".to_string()), Some(fallback_org))
            .expect("fallback role");

        assert_eq!(roles.len(), 1);
        let role = roles.first().unwrap();
        assert_eq!(role.role, "syndic");
        assert_eq!(role.organization_id, Some(fallback_org));
        assert!(role.is_primary);
    }

    #[test]
    fn normalize_primary_role_sets_first_when_none_primary() {
        let mut roles = vec![
            RoleResponse {
                id: Uuid::new_v4().to_string(),
                role: "syndic".to_string(),
                organization_id: None,
                is_primary: false,
            },
            RoleResponse {
                id: Uuid::new_v4().to_string(),
                role: "accountant".to_string(),
                organization_id: None,
                is_primary: false,
            },
        ];
        normalize_primary_role(&mut roles);
        assert!(roles[0].is_primary);
    }
}
