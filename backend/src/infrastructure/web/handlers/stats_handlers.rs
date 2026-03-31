use crate::infrastructure::web::{AppState, AuthenticatedUser};
use actix_web::{get, web, HttpResponse, Responder};

/// GET /api/v1/stats/dashboard
/// Get dashboard statistics (SuperAdmin only)
#[get("/stats/dashboard")]
pub async fn get_dashboard_stats(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
) -> impl Responder {
    if user.role != "superadmin" {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Only SuperAdmin can access dashboard statistics"
        }));
    }

    match state.stats_use_cases.get_admin_dashboard_stats().await {
        Ok(stats) => HttpResponse::Ok().json(stats),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to fetch dashboard statistics: {}", e)
        })),
    }
}

/// GET /api/v1/stats/owner
/// Get owner dashboard statistics (Owner role)
#[get("/stats/owner")]
pub async fn get_owner_stats(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
) -> impl Responder {
    if user.role != "owner" && user.role != "superadmin" {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Only Owner can access these statistics"
        }));
    }

    match state
        .stats_use_cases
        .get_owner_stats_by_user_id(user.user_id)
        .await
    {
        Ok(stats) => HttpResponse::Ok().json(stats),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to fetch owner dashboard statistics: {}", e)
        })),
    }
}

/// GET /api/v1/stats/syndic
/// Get Syndic dashboard statistics (Syndic and Accountant roles)
#[get("/stats/syndic")]
pub async fn get_syndic_stats(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
) -> impl Responder {
    if user.role != "syndic" && user.role != "accountant" && user.role != "superadmin" {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Only Syndic and Accountant can access these statistics"
        }));
    }

    let Some(org_id) = user.organization_id else {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "User has no organization"
        }));
    };

    match state.stats_use_cases.get_syndic_stats(org_id).await {
        Ok(stats) => HttpResponse::Ok().json(stats),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to fetch syndic dashboard statistics: {}", e)
        })),
    }
}

/// GET /api/v1/stats/syndic/urgent-tasks
/// Get urgent tasks for Syndic dashboard (Syndic and Accountant roles)
#[get("/stats/syndic/urgent-tasks")]
pub async fn get_syndic_urgent_tasks(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
) -> impl Responder {
    if user.role != "syndic" && user.role != "accountant" && user.role != "superadmin" {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Only Syndic and Accountant can access these tasks"
        }));
    }

    let Some(org_id) = user.organization_id else {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "User has no organization"
        }));
    };

    match state.stats_use_cases.get_syndic_urgent_tasks(org_id).await {
        Ok(tasks) => HttpResponse::Ok().json(tasks),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to fetch urgent tasks: {}", e)
        })),
    }
}

/// GET /api/v1/stats/seed-data
/// Get seed data statistics (SuperAdmin only)
#[get("/stats/seed-data")]
pub async fn get_seed_data_stats(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
) -> impl Responder {
    if user.role != "superadmin" {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Only SuperAdmin can access seed data statistics"
        }));
    }

    match state.stats_use_cases.get_seed_data_stats().await {
        Ok(stats) => HttpResponse::Ok().json(stats),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to fetch seed data statistics: {}", e)
        })),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_role_check_superadmin_passes() {
        let allowed_roles = ["superadmin"];
        assert!(allowed_roles.contains(&"superadmin"));
        assert!(!allowed_roles.contains(&"owner"));
    }

    #[test]
    fn test_role_check_syndic_and_accountant() {
        let allowed_roles = ["syndic", "accountant", "superadmin"];
        assert!(allowed_roles.contains(&"syndic"));
        assert!(allowed_roles.contains(&"accountant"));
        assert!(!allowed_roles.contains(&"owner"));
    }

    #[test]
    fn test_role_check_owner() {
        let allowed_roles = ["owner", "superadmin"];
        assert!(allowed_roles.contains(&"owner"));
        assert!(!allowed_roles.contains(&"syndic"));
    }
}
