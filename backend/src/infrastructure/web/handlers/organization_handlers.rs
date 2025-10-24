use crate::infrastructure::web::{AppState, AuthenticatedUser};
use actix_web::{get, web, HttpResponse, Responder};
use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Serialize)]
pub struct OrganizationResponse {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub contact_email: String,
    pub contact_phone: Option<String>,
    pub subscription_plan: String,
    pub max_buildings: i32,
    pub max_users: i32,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

/// List all organizations (SuperAdmin only)
#[get("/organizations")]
pub async fn list_organizations(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
) -> impl Responder {
    // Only SuperAdmin can access all organizations
    if user.role != "superadmin" {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Only SuperAdmin can access all organizations"
        }));
    }

    let result = sqlx::query!(
        r#"
        SELECT id, name, slug, contact_email, contact_phone, subscription_plan, max_buildings, max_users, is_active, created_at
        FROM organizations
        ORDER BY created_at DESC
        "#
    )
    .fetch_all(&state.pool)
    .await;

    match result {
        Ok(rows) => {
            let organizations: Vec<OrganizationResponse> = rows
                .into_iter()
                .map(|row| OrganizationResponse {
                    id: row.id.to_string(),
                    name: row.name,
                    slug: row.slug,
                    contact_email: row.contact_email,
                    contact_phone: row.contact_phone,
                    subscription_plan: row.subscription_plan,
                    max_buildings: row.max_buildings,
                    max_users: row.max_users,
                    is_active: row.is_active,
                    created_at: row.created_at,
                })
                .collect();

            HttpResponse::Ok().json(serde_json::json!({
                "data": organizations
            }))
        }
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to fetch organizations: {}", e)
        })),
    }
}
