use crate::infrastructure::web::{AppState, AuthenticatedUser};
use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use bcrypt::{hash, DEFAULT_COST};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize)]
pub struct UserResponse {
    pub id: String,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub role: String,
    pub organization_id: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

/// List all users (SuperAdmin only)
#[get("/users")]
pub async fn list_users(state: web::Data<AppState>, user: AuthenticatedUser) -> impl Responder {
    // Only SuperAdmin can access all users
    if user.role != "superadmin" {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Only SuperAdmin can access all users"
        }));
    }

    let result = sqlx::query!(
        r#"
        SELECT id, email, first_name, last_name, role, organization_id, is_active, created_at
        FROM users
        ORDER BY created_at DESC
        "#
    )
    .fetch_all(&state.pool)
    .await;

    match result {
        Ok(rows) => {
            let users: Vec<UserResponse> = rows
                .into_iter()
                .map(|row| UserResponse {
                    id: row.id.to_string(),
                    email: row.email,
                    first_name: row.first_name,
                    last_name: row.last_name,
                    role: row.role,
                    organization_id: row.organization_id.map(|id| id.to_string()),
                    is_active: row.is_active,
                    created_at: row.created_at,
                })
                .collect();

            HttpResponse::Ok().json(serde_json::json!({
                "data": users
            }))
        }
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to fetch users: {}", e)
        })),
    }
}

/// Create user (SuperAdmin only)
#[post("/users")]
pub async fn create_user(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    req: web::Json<CreateUserRequest>,
) -> impl Responder {
    // Only SuperAdmin can create users
    if user.role != "superadmin" {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Only SuperAdmin can create users"
        }));
    }

    // Validate role
    let valid_roles = ["superadmin", "syndic", "accountant", "owner"];
    if !valid_roles.contains(&req.role.as_str()) {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Invalid role"
        }));
    }

    // Validate email format
    if !req.email.contains('@') {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Invalid email format"
        }));
    }

    // Validate name lengths
    if req.first_name.trim().len() < 2 || req.last_name.trim().len() < 2 {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "First and last names must be at least 2 characters"
        }));
    }

    // Validate password length
    if req.password.len() < 6 {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Password must be at least 6 characters"
        }));
    }

    // Hash password
    let hashed_password = match hash(&req.password, DEFAULT_COST) {
        Ok(h) => h,
        Err(e) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to hash password: {}", e)
            }))
        }
    };

    // Generate UUID for the new user
    let user_id = Uuid::new_v4();

    let result = sqlx::query!(
        r#"
        INSERT INTO users (id, email, password_hash, first_name, last_name, role, organization_id, is_active, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, true, NOW(), NOW())
        RETURNING id, email, first_name, last_name, role, organization_id, is_active, created_at
        "#,
        user_id,
        req.email.trim().to_lowercase(),
        hashed_password,
        req.first_name.trim(),
        req.last_name.trim(),
        req.role,
        req.organization_id
    )
    .fetch_one(&state.pool)
    .await;

    match result {
        Ok(row) => HttpResponse::Created().json(UserResponse {
            id: row.id.to_string(),
            email: row.email,
            first_name: row.first_name,
            last_name: row.last_name,
            role: row.role,
            organization_id: row.organization_id.map(|id| id.to_string()),
            is_active: row.is_active,
            created_at: row.created_at,
        }),
        Err(sqlx::Error::Database(db_err)) => {
            if db_err.is_unique_violation() {
                HttpResponse::BadRequest().json(serde_json::json!({
                    "error": "Email already exists"
                }))
            } else {
                HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": format!("Failed to create user: {}", db_err)
                }))
            }
        }
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to create user: {}", e)
        })),
    }
}

#[derive(Deserialize)]
pub struct CreateUserRequest {
    pub email: String,
    pub password: String,
    pub first_name: String,
    pub last_name: String,
    pub role: String,
    pub organization_id: Option<Uuid>,
}

#[derive(Deserialize)]
pub struct UpdateUserRequest {
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub role: String,
    pub organization_id: Option<Uuid>,
}

/// Update user (SuperAdmin only)
#[put("/users/{id}")]
pub async fn update_user(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    path: web::Path<Uuid>,
    req: web::Json<UpdateUserRequest>,
) -> impl Responder {
    // Only SuperAdmin can update users
    if user.role != "superadmin" {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Only SuperAdmin can update users"
        }));
    }

    let user_id = path.into_inner();

    // Validate role
    let valid_roles = ["superadmin", "syndic", "accountant", "owner"];
    if !valid_roles.contains(&req.role.as_str()) {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Invalid role"
        }));
    }

    // Validate email format
    if !req.email.contains('@') {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Invalid email format"
        }));
    }

    // Validate name lengths
    if req.first_name.trim().len() < 2 || req.last_name.trim().len() < 2 {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "First and last names must be at least 2 characters"
        }));
    }

    let result = sqlx::query!(
        r#"
        UPDATE users
        SET email = $1, first_name = $2, last_name = $3, role = $4, organization_id = $5, updated_at = NOW()
        WHERE id = $6
        RETURNING id, email, first_name, last_name, role, organization_id, is_active, created_at
        "#,
        req.email.trim().to_lowercase(),
        req.first_name.trim(),
        req.last_name.trim(),
        req.role,
        req.organization_id,
        user_id
    )
    .fetch_one(&state.pool)
    .await;

    match result {
        Ok(row) => HttpResponse::Ok().json(UserResponse {
            id: row.id.to_string(),
            email: row.email,
            first_name: row.first_name,
            last_name: row.last_name,
            role: row.role,
            organization_id: row.organization_id.map(|id| id.to_string()),
            is_active: row.is_active,
            created_at: row.created_at,
        }),
        Err(sqlx::Error::RowNotFound) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "User not found"
        })),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to update user: {}", e)
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
    // Only SuperAdmin can activate users
    if user.role != "superadmin" {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Only SuperAdmin can activate users"
        }));
    }

    let user_id = path.into_inner();

    let result = sqlx::query!(
        r#"
        UPDATE users
        SET is_active = true, updated_at = NOW()
        WHERE id = $1
        RETURNING id, email, first_name, last_name, role, organization_id, is_active, created_at
        "#,
        user_id
    )
    .fetch_one(&state.pool)
    .await;

    match result {
        Ok(row) => HttpResponse::Ok().json(UserResponse {
            id: row.id.to_string(),
            email: row.email,
            first_name: row.first_name,
            last_name: row.last_name,
            role: row.role,
            organization_id: row.organization_id.map(|id| id.to_string()),
            is_active: row.is_active,
            created_at: row.created_at,
        }),
        Err(sqlx::Error::RowNotFound) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "User not found"
        })),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
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
    // Only SuperAdmin can deactivate users
    if user.role != "superadmin" {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Only SuperAdmin can deactivate users"
        }));
    }

    let user_id = path.into_inner();

    let result = sqlx::query!(
        r#"
        UPDATE users
        SET is_active = false, updated_at = NOW()
        WHERE id = $1
        RETURNING id, email, first_name, last_name, role, organization_id, is_active, created_at
        "#,
        user_id
    )
    .fetch_one(&state.pool)
    .await;

    match result {
        Ok(row) => HttpResponse::Ok().json(UserResponse {
            id: row.id.to_string(),
            email: row.email,
            first_name: row.first_name,
            last_name: row.last_name,
            role: row.role,
            organization_id: row.organization_id.map(|id| id.to_string()),
            is_active: row.is_active,
            created_at: row.created_at,
        }),
        Err(sqlx::Error::RowNotFound) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "User not found"
        })),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
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
    // Only SuperAdmin can delete users
    if user.role != "superadmin" {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Only SuperAdmin can delete users"
        }));
    }

    let user_id = path.into_inner();

    // Check if trying to delete self
    if user.user_id == user_id {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Cannot delete your own account"
        }));
    }

    let result = sqlx::query!(
        r#"
        DELETE FROM users
        WHERE id = $1
        "#,
        user_id
    )
    .execute(&state.pool)
    .await;

    match result {
        Ok(result) => {
            if result.rows_affected() == 0 {
                HttpResponse::NotFound().json(serde_json::json!({
                    "error": "User not found"
                }))
            } else {
                HttpResponse::Ok().json(serde_json::json!({
                    "message": "User deleted successfully"
                }))
            }
        }
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to delete user: {}", e)
        })),
    }
}
