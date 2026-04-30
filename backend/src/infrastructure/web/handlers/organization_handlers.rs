use crate::domain::entities::Organization;
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

fn to_response(org: Organization) -> OrganizationResponse {
    OrganizationResponse {
        id: org.id.to_string(),
        name: org.name,
        slug: org.slug,
        contact_email: org.contact_email,
        contact_phone: org.contact_phone,
        subscription_plan: org.subscription_plan.to_string(),
        max_buildings: org.max_buildings,
        max_users: org.max_users,
        is_active: org.is_active,
        created_at: org.created_at,
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

/// GET /api/v1/organizations
/// List all organizations (SuperAdmin only)
#[get("/organizations")]
pub async fn list_organizations(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
) -> impl Responder {
    if user.role != "superadmin" {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Only SuperAdmin can access all organizations"
        }));
    }

    match state.organization_use_cases.list_all().await {
        Ok(orgs) => HttpResponse::Ok().json(serde_json::json!({
            "data": orgs.into_iter().map(to_response).collect::<Vec<_>>()
        })),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to fetch organizations: {}", e)
        })),
    }
}

/// POST /api/v1/organizations
/// Create organization (SuperAdmin only)
#[post("/organizations")]
pub async fn create_organization(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    req: web::Json<CreateOrganizationRequest>,
) -> impl Responder {
    if user.role != "superadmin" {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Only SuperAdmin can create organizations"
        }));
    }

    match state
        .organization_use_cases
        .create(
            req.name.clone(),
            req.slug.clone(),
            req.contact_email.clone(),
            req.contact_phone.clone(),
            req.subscription_plan.clone(),
        )
        .await
    {
        Ok(org) => HttpResponse::Created().json(to_response(org)),
        Err(e) if e == "invalid_plan" => HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Invalid subscription plan"
        })),
        Err(e) if e.starts_with("validation_error:") => {
            HttpResponse::BadRequest().json(serde_json::json!({
                "error": e.trim_start_matches("validation_error:")
            }))
        }
        Err(e) if e.contains("unique") || e.contains("duplicate") => HttpResponse::BadRequest()
            .json(serde_json::json!({
                "error": "Slug already exists"
            })),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to create organization: {}", e)
        })),
    }
}

/// PUT /api/v1/organizations/{id}
/// Update organization (SuperAdmin only)
#[put("/organizations/{id}")]
pub async fn update_organization(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    path: web::Path<Uuid>,
    req: web::Json<UpdateOrganizationRequest>,
) -> impl Responder {
    if user.role != "superadmin" {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Only SuperAdmin can update organizations"
        }));
    }

    let org_id = path.into_inner();

    match state
        .organization_use_cases
        .update(
            org_id,
            req.name.clone(),
            req.slug.clone(),
            req.contact_email.clone(),
            req.contact_phone.clone(),
            req.subscription_plan.clone(),
        )
        .await
    {
        Ok(org) => HttpResponse::Ok().json(to_response(org)),
        Err(e) if e == "not_found" => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Organization not found"
        })),
        Err(e) if e == "invalid_plan" => HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Invalid subscription plan"
        })),
        Err(e) if e.starts_with("validation_error:") => {
            HttpResponse::BadRequest().json(serde_json::json!({
                "error": e.trim_start_matches("validation_error:")
            }))
        }
        Err(e) if e.contains("unique") || e.contains("duplicate") => HttpResponse::BadRequest()
            .json(serde_json::json!({
                "error": "Slug already exists"
            })),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to update organization: {}", e)
        })),
    }
}

/// PUT /api/v1/organizations/{id}/activate
/// Activate organization (SuperAdmin only)
#[put("/organizations/{id}/activate")]
pub async fn activate_organization(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    path: web::Path<Uuid>,
) -> impl Responder {
    if user.role != "superadmin" {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Only SuperAdmin can activate organizations"
        }));
    }

    let org_id = path.into_inner();

    match state.organization_use_cases.activate(org_id).await {
        Ok(org) => HttpResponse::Ok().json(to_response(org)),
        Err(e) if e == "not_found" => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Organization not found"
        })),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to activate organization: {}", e)
        })),
    }
}

/// PUT /api/v1/organizations/{id}/suspend
/// Suspend organization (SuperAdmin only)
#[put("/organizations/{id}/suspend")]
pub async fn suspend_organization(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    path: web::Path<Uuid>,
) -> impl Responder {
    if user.role != "superadmin" {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Only SuperAdmin can suspend organizations"
        }));
    }

    let org_id = path.into_inner();

    match state.organization_use_cases.suspend(org_id).await {
        Ok(org) => HttpResponse::Ok().json(to_response(org)),
        Err(e) if e == "not_found" => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Organization not found"
        })),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to suspend organization: {}", e)
        })),
    }
}

/// DELETE /api/v1/organizations/{id}
/// Delete organization (SuperAdmin only)
#[delete("/organizations/{id}")]
pub async fn delete_organization(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    path: web::Path<Uuid>,
) -> impl Responder {
    if user.role != "superadmin" {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Only SuperAdmin can delete organizations"
        }));
    }

    let org_id = path.into_inner();

    match state.organization_use_cases.delete(org_id).await {
        Ok(true) => HttpResponse::Ok().json(serde_json::json!({
            "message": "Organization deleted successfully"
        })),
        Ok(false) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Organization not found"
        })),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to delete organization: {}", e)
        })),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_response_maps_all_fields() {
        let org = Organization {
            id: Uuid::new_v4(),
            name: "Test Org".to_string(),
            slug: "test-org".to_string(),
            contact_email: "admin@test.com".to_string(),
            contact_phone: Some("+32123456789".to_string()),
            subscription_plan: crate::domain::entities::SubscriptionPlan::Professional,
            max_buildings: 20,
            max_users: 50,
            is_active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let resp = to_response(org);
        assert_eq!(resp.name, "Test Org");
        assert_eq!(resp.slug, "test-org");
        assert_eq!(resp.subscription_plan, "professional");
        assert_eq!(resp.max_buildings, 20);
    }

    #[test]
    fn test_to_response_inactive_org() {
        let org = Organization {
            id: Uuid::new_v4(),
            name: "Inactive".to_string(),
            slug: "inactive".to_string(),
            contact_email: "x@x.com".to_string(),
            contact_phone: None,
            subscription_plan: crate::domain::entities::SubscriptionPlan::Free,
            max_buildings: 1,
            max_users: 3,
            is_active: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let resp = to_response(org);
        assert!(!resp.is_active);
        assert!(resp.contact_phone.is_none());
    }
}
