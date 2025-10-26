use crate::infrastructure::web::{AppState, AuthenticatedUser};
use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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

#[derive(Deserialize)]
pub struct CreateOrganizationRequest {
    pub name: String,
    pub slug: String,
    pub contact_email: String,
    pub contact_phone: Option<String>,
    pub subscription_plan: String,
}

#[derive(Deserialize)]
pub struct UpdateOrganizationRequest {
    pub name: String,
    pub slug: String,
    pub contact_email: String,
    pub contact_phone: Option<String>,
    pub subscription_plan: String,
}

/// Create organization (SuperAdmin only)
#[post("/organizations")]
pub async fn create_organization(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    req: web::Json<CreateOrganizationRequest>,
) -> impl Responder {
    // Only SuperAdmin can create organizations
    if user.role != "superadmin" {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Only SuperAdmin can create organizations"
        }));
    }

    // Validate subscription plan
    let valid_plans = ["free", "starter", "professional", "enterprise"];
    if !valid_plans.contains(&req.subscription_plan.to_lowercase().as_str()) {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Invalid subscription plan"
        }));
    }

    // Validate email format
    if !req.contact_email.contains('@') {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Invalid email format"
        }));
    }

    // Validate name and slug lengths
    if req.name.trim().len() < 2 || req.slug.trim().len() < 2 {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Name and slug must be at least 2 characters"
        }));
    }

    // Validate slug format (lowercase alphanumeric and hyphens only)
    if !req.slug.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-') {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Slug must contain only lowercase letters, numbers, and hyphens"
        }));
    }

    // Determine limits based on plan
    let (max_buildings, max_users) = match req.subscription_plan.to_lowercase().as_str() {
        "free" => (1, 3),
        "starter" => (5, 10),
        "professional" => (20, 50),
        "enterprise" => (999, 999),
        _ => (1, 3), // Default to free
    };

    // Generate UUID
    let org_id = Uuid::new_v4();

    let result = sqlx::query!(
        r#"
        INSERT INTO organizations (id, name, slug, contact_email, contact_phone, subscription_plan, max_buildings, max_users, is_active, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, true, NOW(), NOW())
        RETURNING id, name, slug, contact_email, contact_phone, subscription_plan, max_buildings, max_users, is_active, created_at
        "#,
        org_id,
        req.name.trim(),
        req.slug.trim().to_lowercase(),
        req.contact_email.trim().to_lowercase(),
        req.contact_phone.as_ref().map(|s| s.trim()),
        req.subscription_plan.to_lowercase(),
        max_buildings,
        max_users
    )
    .fetch_one(&state.pool)
    .await;

    match result {
        Ok(row) => HttpResponse::Created().json(OrganizationResponse {
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
        }),
        Err(sqlx::Error::Database(db_err)) => {
            if db_err.is_unique_violation() {
                HttpResponse::BadRequest().json(serde_json::json!({
                    "error": "Slug already exists"
                }))
            } else {
                HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": format!("Failed to create organization: {}", db_err)
                }))
            }
        }
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to create organization: {}", e)
        })),
    }
}

/// Update organization (SuperAdmin only)
#[put("/organizations/{id}")]
pub async fn update_organization(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    path: web::Path<Uuid>,
    req: web::Json<UpdateOrganizationRequest>,
) -> impl Responder {
    // Only SuperAdmin can update organizations
    if user.role != "superadmin" {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Only SuperAdmin can update organizations"
        }));
    }

    let org_id = path.into_inner();

    // Validate subscription plan
    let valid_plans = ["free", "starter", "professional", "enterprise"];
    if !valid_plans.contains(&req.subscription_plan.to_lowercase().as_str()) {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Invalid subscription plan"
        }));
    }

    // Validate email format
    if !req.contact_email.contains('@') {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Invalid email format"
        }));
    }

    // Validate name and slug lengths
    if req.name.trim().len() < 2 || req.slug.trim().len() < 2 {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Name and slug must be at least 2 characters"
        }));
    }

    // Validate slug format
    if !req.slug.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-') {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Slug must contain only lowercase letters, numbers, and hyphens"
        }));
    }

    // Determine limits based on plan
    let (max_buildings, max_users) = match req.subscription_plan.to_lowercase().as_str() {
        "free" => (1, 3),
        "starter" => (5, 10),
        "professional" => (20, 50),
        "enterprise" => (999, 999),
        _ => (1, 3),
    };

    let result = sqlx::query!(
        r#"
        UPDATE organizations
        SET name = $1, slug = $2, contact_email = $3, contact_phone = $4, subscription_plan = $5, max_buildings = $6, max_users = $7, updated_at = NOW()
        WHERE id = $8
        RETURNING id, name, slug, contact_email, contact_phone, subscription_plan, max_buildings, max_users, is_active, created_at
        "#,
        req.name.trim(),
        req.slug.trim().to_lowercase(),
        req.contact_email.trim().to_lowercase(),
        req.contact_phone.as_ref().map(|s| s.trim()),
        req.subscription_plan.to_lowercase(),
        max_buildings,
        max_users,
        org_id
    )
    .fetch_one(&state.pool)
    .await;

    match result {
        Ok(row) => HttpResponse::Ok().json(OrganizationResponse {
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
        }),
        Err(sqlx::Error::RowNotFound) => {
            HttpResponse::NotFound().json(serde_json::json!({
                "error": "Organization not found"
            }))
        }
        Err(sqlx::Error::Database(db_err)) => {
            if db_err.is_unique_violation() {
                HttpResponse::BadRequest().json(serde_json::json!({
                    "error": "Slug already exists"
                }))
            } else {
                HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": format!("Failed to update organization: {}", db_err)
                }))
            }
        }
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to update organization: {}", e)
        })),
    }
}

/// Activate organization (SuperAdmin only)
#[put("/organizations/{id}/activate")]
pub async fn activate_organization(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    path: web::Path<Uuid>,
) -> impl Responder {
    // Only SuperAdmin can activate organizations
    if user.role != "superadmin" {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Only SuperAdmin can activate organizations"
        }));
    }

    let org_id = path.into_inner();

    let result = sqlx::query!(
        r#"
        UPDATE organizations
        SET is_active = true, updated_at = NOW()
        WHERE id = $1
        RETURNING id, name, slug, contact_email, contact_phone, subscription_plan, max_buildings, max_users, is_active, created_at
        "#,
        org_id
    )
    .fetch_one(&state.pool)
    .await;

    match result {
        Ok(row) => HttpResponse::Ok().json(OrganizationResponse {
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
        }),
        Err(sqlx::Error::RowNotFound) => {
            HttpResponse::NotFound().json(serde_json::json!({
                "error": "Organization not found"
            }))
        }
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to activate organization: {}", e)
        })),
    }
}

/// Suspend organization (SuperAdmin only)
#[put("/organizations/{id}/suspend")]
pub async fn suspend_organization(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    path: web::Path<Uuid>,
) -> impl Responder {
    // Only SuperAdmin can suspend organizations
    if user.role != "superadmin" {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Only SuperAdmin can suspend organizations"
        }));
    }

    let org_id = path.into_inner();

    let result = sqlx::query!(
        r#"
        UPDATE organizations
        SET is_active = false, updated_at = NOW()
        WHERE id = $1
        RETURNING id, name, slug, contact_email, contact_phone, subscription_plan, max_buildings, max_users, is_active, created_at
        "#,
        org_id
    )
    .fetch_one(&state.pool)
    .await;

    match result {
        Ok(row) => HttpResponse::Ok().json(OrganizationResponse {
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
        }),
        Err(sqlx::Error::RowNotFound) => {
            HttpResponse::NotFound().json(serde_json::json!({
                "error": "Organization not found"
            }))
        }
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to suspend organization: {}", e)
        })),
    }
}

/// Delete organization (SuperAdmin only)
#[delete("/organizations/{id}")]
pub async fn delete_organization(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    path: web::Path<Uuid>,
) -> impl Responder {
    // Only SuperAdmin can delete organizations
    if user.role != "superadmin" {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Only SuperAdmin can delete organizations"
        }));
    }

    let org_id = path.into_inner();

    let result = sqlx::query!(
        r#"
        DELETE FROM organizations
        WHERE id = $1
        "#,
        org_id
    )
    .execute(&state.pool)
    .await;

    match result {
        Ok(result) => {
            if result.rows_affected() == 0 {
                HttpResponse::NotFound().json(serde_json::json!({
                    "error": "Organization not found"
                }))
            } else {
                HttpResponse::Ok().json(serde_json::json!({
                    "message": "Organization deleted successfully"
                }))
            }
        }
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to delete organization: {}", e)
        })),
    }
}
