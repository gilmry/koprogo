use crate::infrastructure::web::{AppState, AuthenticatedUser};
use actix_web::{get, web, HttpResponse, Responder};
use chrono::{DateTime, Utc};
use serde::Serialize;

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
