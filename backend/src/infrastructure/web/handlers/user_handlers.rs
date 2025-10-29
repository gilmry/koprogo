use crate::infrastructure::web::{AppState, AuthenticatedUser};
use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use bcrypt::{hash, DEFAULT_COST};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{Executor, Postgres, Transaction};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

const ALLOWED_ROLES: [&str; 4] = ["superadmin", "syndic", "accountant", "owner"];

#[derive(Serialize, Clone)]
pub struct RoleResponse {
    pub id: String,
    pub role: String,
    pub organization_id: Option<String>,
    pub is_primary: bool,
}

#[derive(Serialize, Clone)]
pub struct UserResponse {
    pub id: String,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub role: String,
    pub organization_id: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub roles: Vec<RoleResponse>,
    pub active_role: Option<RoleResponse>,
}

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
struct NormalizedRoleAssignment {
    id: Uuid,
    role: String,
    organization_id: Option<Uuid>,
    is_primary: bool,
}

/// List all users (SuperAdmin only)
#[get("/users")]
pub async fn list_users(state: web::Data<AppState>, user: AuthenticatedUser) -> impl Responder {
    if user.role != "superadmin" {
        return HttpResponse::Forbidden().json(json!({
            "error": "Only SuperAdmin can access all users"
        }));
    }

    let rows = match sqlx::query!(
        r#"
        SELECT id, email, first_name, last_name, role, organization_id, is_active, created_at
        FROM users
        ORDER BY created_at DESC
        "#
    )
    .fetch_all(&state.pool)
    .await
    {
        Ok(rows) => rows,
        Err(e) => {
            return HttpResponse::InternalServerError().json(json!({
                "error": format!("Failed to fetch users: {}", e)
            }))
        }
    };

    let user_ids: Vec<Uuid> = rows.iter().map(|row| row.id).collect();
    let roles_map = match load_roles_for_users(&state.pool, &user_ids).await {
        Ok(map) => map,
        Err(e) => {
            return HttpResponse::InternalServerError().json(json!({
                "error": format!("Failed to fetch user roles: {}", e)
            }))
        }
    };

    let mut users: Vec<UserResponse> = Vec::with_capacity(user_ids.len());
    for row in rows {
        let fallback_role = row.role.clone();
        let fallback_org = row.organization_id;
        let mut roles = roles_map
            .get(&row.id)
            .cloned()
            .unwrap_or_else(|| vec![fallback_role_response(fallback_role.clone(), fallback_org)]);

        normalize_primary_role(&mut roles);
        let active_role = roles
            .iter()
            .find(|role| role.is_primary)
            .cloned()
            .or_else(|| roles.first().cloned());

        users.push(UserResponse {
            id: row.id.to_string(),
            email: row.email,
            first_name: row.first_name,
            last_name: row.last_name,
            role: active_role
                .as_ref()
                .map(|r| r.role.clone())
                .unwrap_or(fallback_role),
            organization_id: active_role
                .as_ref()
                .and_then(|r| r.organization_id.clone())
                .or_else(|| fallback_org.map(|id| id.to_string())),
            is_active: row.is_active,
            created_at: row.created_at,
            roles,
            active_role,
        });
    }

    HttpResponse::Ok().json(json!({ "data": users }))
}

/// Create user (SuperAdmin only)
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
        return HttpResponse::BadRequest().json(json!({
            "error": "Invalid email format"
        }));
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
        Ok(roles) => roles,
        Err(resp) => return resp,
    };

    let primary_role = roles
        .iter()
        .find(|role| role.is_primary)
        .cloned()
        .expect("normalized roles always have a primary role");

    let hashed_password = match hash(req.password.trim(), DEFAULT_COST) {
        Ok(hash) => hash,
        Err(e) => {
            return HttpResponse::InternalServerError().json(json!({
                "error": format!("Failed to hash password: {}", e)
            }))
        }
    };

    let mut tx = match state.pool.begin().await {
        Ok(tx) => tx,
        Err(e) => {
            return HttpResponse::InternalServerError().json(json!({
                "error": format!("Failed to begin transaction: {}", e)
            }))
        }
    };

    let user_row = match sqlx::query!(
        r#"
        INSERT INTO users (id, email, password_hash, first_name, last_name, role, organization_id, is_active, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, true, NOW(), NOW())
        RETURNING id
        "#,
        Uuid::new_v4(),
        req.email.trim().to_lowercase(),
        hashed_password,
        req.first_name.trim(),
        req.last_name.trim(),
        primary_role.role.clone(),
        primary_role.organization_id
    )
    .fetch_one(&mut *tx)
    .await
    {
        Ok(row) => row,
        Err(sqlx::Error::Database(db_err)) if db_err.is_unique_violation() => {
            return HttpResponse::BadRequest().json(json!({
                "error": "Email already exists"
            }))
        }
        Err(e) => {
            return HttpResponse::InternalServerError().json(json!({
                "error": format!("Failed to create user: {}", e)
            }))
        }
    };

    if let Err(e) = replace_user_roles(&mut tx, user_row.id, &roles).await {
        return HttpResponse::InternalServerError().json(json!({
            "error": format!("Failed to assign roles: {}", e)
        }));
    }

    if let Err(e) = tx.commit().await {
        return HttpResponse::InternalServerError().json(json!({
            "error": format!("Failed to commit transaction: {}", e)
        }));
    }

    match load_user_response(&state.pool, user_row.id).await {
        Ok(response) => HttpResponse::Created().json(response),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "error": format!("Failed to load created user: {}", e)
        })),
    }
}

/// Update user (SuperAdmin only)
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
        return HttpResponse::BadRequest().json(json!({
            "error": "Invalid email format"
        }));
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
        Ok(roles) => roles,
        Err(resp) => return resp,
    };

    let primary_role = roles
        .iter()
        .find(|role| role.is_primary)
        .cloned()
        .expect("normalized roles always have a primary role");

    let user_id = path.into_inner();

    let mut tx = match state.pool.begin().await {
        Ok(tx) => tx,
        Err(e) => {
            return HttpResponse::InternalServerError().json(json!({
                "error": format!("Failed to begin transaction: {}", e)
            }))
        }
    };

    if let Some(password) = &req.password {
        if !password.trim().is_empty() {
            let hashed = match hash(password.trim(), DEFAULT_COST) {
                Ok(hash) => hash,
                Err(e) => {
                    return HttpResponse::InternalServerError().json(json!({
                        "error": format!("Failed to hash password: {}", e)
                    }));
                }
            };

            if let Err(e) = tx
                .execute(sqlx::query!(
                    r#"
                    UPDATE users
                    SET password_hash = $1, updated_at = NOW()
                    WHERE id = $2
                    "#,
                    hashed,
                    user_id
                ))
                .await
            {
                return HttpResponse::InternalServerError().json(json!({
                    "error": format!("Failed to update password: {}", e)
                }));
            }
        }
    }

    let updated = match sqlx::query!(
        r#"
        UPDATE users
        SET email = $1,
            first_name = $2,
            last_name = $3,
            role = $4,
            organization_id = $5,
            updated_at = NOW()
        WHERE id = $6
        RETURNING id
        "#,
        req.email.trim().to_lowercase(),
        req.first_name.trim(),
        req.last_name.trim(),
        primary_role.role.clone(),
        primary_role.organization_id,
        user_id
    )
    .fetch_optional(&mut *tx)
    .await
    {
        Ok(row) => row,
        Err(sqlx::Error::Database(db_err)) if db_err.is_unique_violation() => {
            return HttpResponse::BadRequest().json(json!({
                "error": "Email already exists"
            }))
        }
        Err(e) => {
            return HttpResponse::InternalServerError().json(json!({
                "error": format!("Failed to update user: {}", e)
            }))
        }
    };

    if updated.is_none() {
        return HttpResponse::NotFound().json(json!({
            "error": "User not found"
        }));
    }

    if let Err(e) = replace_user_roles(&mut tx, user_id, &roles).await {
        return HttpResponse::InternalServerError().json(json!({
            "error": format!("Failed to update user roles: {}", e)
        }));
    }

    if let Err(e) = tx.commit().await {
        return HttpResponse::InternalServerError().json(json!({
            "error": format!("Failed to commit transaction: {}", e)
        }));
    }

    match load_user_response(&state.pool, user_id).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "error": format!("Failed to load updated user: {}", e)
        })),
    }
}

/// Activate user (SuperAdmin only)
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

    let updated = sqlx::query!(
        r#"
        UPDATE users
        SET is_active = true, updated_at = NOW()
        WHERE id = $1
        RETURNING id
        "#,
        user_id
    )
    .fetch_optional(&state.pool)
    .await;

    match updated {
        Ok(Some(_)) => match load_user_response(&state.pool, user_id).await {
            Ok(response) => HttpResponse::Ok().json(response),
            Err(e) => HttpResponse::InternalServerError().json(json!({
                "error": format!("Failed to load user: {}", e)
            })),
        },
        Ok(None) => HttpResponse::NotFound().json(json!({
            "error": "User not found"
        })),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "error": format!("Failed to activate user: {}", e)
        })),
    }
}

/// Deactivate user (SuperAdmin only)
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

    let updated = sqlx::query!(
        r#"
        UPDATE users
        SET is_active = false, updated_at = NOW()
        WHERE id = $1
        RETURNING id
        "#,
        user_id
    )
    .fetch_optional(&state.pool)
    .await;

    match updated {
        Ok(Some(_)) => match load_user_response(&state.pool, user_id).await {
            Ok(response) => HttpResponse::Ok().json(response),
            Err(e) => HttpResponse::InternalServerError().json(json!({
                "error": format!("Failed to load user: {}", e)
            })),
        },
        Ok(None) => HttpResponse::NotFound().json(json!({
            "error": "User not found"
        })),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "error": format!("Failed to deactivate user: {}", e)
        })),
    }
}

/// Delete user (SuperAdmin only)
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

    match sqlx::query!(
        r#"
        DELETE FROM users
        WHERE id = $1
        "#,
        user_id
    )
    .execute(&state.pool)
    .await
    {
        Ok(result) => {
            if result.rows_affected() == 0 {
                HttpResponse::NotFound().json(json!({
                    "error": "User not found"
                }))
            } else {
                HttpResponse::Ok().json(json!({
                    "message": "User deleted successfully"
                }))
            }
        }
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "error": format!("Failed to delete user: {}", e)
        })),
    }
}

fn normalize_roles(
    roles: Option<Vec<RoleAssignmentRequest>>,
    fallback_role: Option<String>,
    fallback_org: Option<Uuid>,
) -> Result<Vec<NormalizedRoleAssignment>, HttpResponse> {
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

        normalized.push(NormalizedRoleAssignment {
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

    normalized.sort_by(|a, b| b.is_primary.cmp(&a.is_primary));
    Ok(normalized)
}

async fn replace_user_roles(
    tx: &mut Transaction<'_, Postgres>,
    user_id: Uuid,
    roles: &[NormalizedRoleAssignment],
) -> Result<(), sqlx::Error> {
    tx.execute(sqlx::query!(
        "DELETE FROM user_roles WHERE user_id = $1",
        user_id
    ))
    .await?;

    for assignment in roles {
        tx.execute(sqlx::query!(
            r#"
            INSERT INTO user_roles (id, user_id, role, organization_id, is_primary, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, NOW(), NOW())
            "#,
            assignment.id,
            user_id,
            assignment.role,
            assignment.organization_id,
            assignment.is_primary
        ))
        .await?;
    }

    Ok(())
}

async fn load_roles_for_users(
    pool: &crate::infrastructure::pool::DbPool,
    user_ids: &[Uuid],
) -> Result<HashMap<Uuid, Vec<RoleResponse>>, sqlx::Error> {
    if user_ids.is_empty() {
        return Ok(HashMap::new());
    }

    let rows = sqlx::query!(
        r#"
        SELECT id, user_id, role, organization_id, is_primary, created_at
        FROM user_roles
        WHERE user_id = ANY($1)
        ORDER BY user_id, is_primary DESC, created_at ASC
        "#,
        user_ids
    )
    .fetch_all(pool)
    .await?;

    let mut map: HashMap<Uuid, Vec<RoleResponse>> = HashMap::new();

    for row in rows {
        let entry = RoleResponse {
            id: row.id.to_string(),
            role: row.role,
            organization_id: row.organization_id.map(|id| id.to_string()),
            is_primary: row.is_primary,
        };
        map.entry(row.user_id).or_default().push(entry);
    }

    for roles in map.values_mut() {
        normalize_primary_role(roles);
    }

    Ok(map)
}

async fn load_user_response(
    pool: &crate::infrastructure::pool::DbPool,
    user_id: Uuid,
) -> Result<UserResponse, sqlx::Error> {
    let row = sqlx::query!(
        r#"
        SELECT id, email, first_name, last_name, role, organization_id, is_active, created_at
        FROM users
        WHERE id = $1
        "#,
        user_id
    )
    .fetch_one(pool)
    .await?;

    let roles_map = load_roles_for_users(pool, &[user_id]).await?;
    let mut roles = roles_map.get(&user_id).cloned().unwrap_or_else(|| {
        vec![fallback_role_response(
            row.role.clone(),
            row.organization_id,
        )]
    });

    normalize_primary_role(&mut roles);
    let active_role = roles
        .iter()
        .find(|role| role.is_primary)
        .cloned()
        .or_else(|| roles.first().cloned());

    Ok(UserResponse {
        id: row.id.to_string(),
        email: row.email,
        first_name: row.first_name,
        last_name: row.last_name,
        role: active_role
            .as_ref()
            .map(|r| r.role.clone())
            .unwrap_or(row.role),
        organization_id: active_role
            .as_ref()
            .and_then(|r| r.organization_id.clone())
            .or_else(|| row.organization_id.map(|id| id.to_string())),
        is_active: row.is_active,
        created_at: row.created_at,
        roles,
        active_role,
    })
}

fn fallback_role_response(role: String, organization_id: Option<Uuid>) -> RoleResponse {
    RoleResponse {
        id: Uuid::new_v4().to_string(),
        role,
        organization_id: organization_id.map(|id| id.to_string()),
        is_primary: true,
    }
}

fn normalize_primary_role(roles: &mut [RoleResponse]) {
    if roles.is_empty() {
        return;
    }

    if roles.iter().filter(|r| r.is_primary).count() == 0 {
        roles[0].is_primary = true;
    }

    roles.sort_by(|a, b| b.is_primary.cmp(&a.is_primary));
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::http::StatusCode;

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
}
