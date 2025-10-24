use crate::infrastructure::web::{AppState, AuthenticatedUser};
use actix_web::{get, web, HttpResponse, Responder};
use serde::Serialize;

#[derive(Serialize)]
pub struct DashboardStats {
    pub total_organizations: i64,
    pub total_users: i64,
    pub total_buildings: i64,
    pub active_subscriptions: i64,
    pub total_owners: i64,
    pub total_units: i64,
    pub total_expenses: i64,
    pub total_meetings: i64,
}

/// Get dashboard statistics (SuperAdmin only)
#[get("/stats/dashboard")]
pub async fn get_dashboard_stats(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
) -> impl Responder {
    // Only SuperAdmin can access these stats
    if user.role != "superadmin" {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Only SuperAdmin can access dashboard statistics"
        }));
    }

    // Count organizations
    let orgs_result = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM organizations")
        .fetch_one(&state.pool)
        .await;

    // Count users
    let users_result = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users")
        .fetch_one(&state.pool)
        .await;

    // Count buildings
    let buildings_result = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM buildings")
        .fetch_one(&state.pool)
        .await;

    // Count active subscriptions (organizations with is_active = true)
    let active_subs_result =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM organizations WHERE is_active = true")
            .fetch_one(&state.pool)
            .await;

    // Count owners
    let owners_result = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM owners")
        .fetch_one(&state.pool)
        .await;

    // Count units
    let units_result = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM units")
        .fetch_one(&state.pool)
        .await;

    // Count expenses
    let expenses_result = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM expenses")
        .fetch_one(&state.pool)
        .await;

    // Count meetings
    let meetings_result = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM meetings")
        .fetch_one(&state.pool)
        .await;

    // Handle errors
    match (
        orgs_result,
        users_result,
        buildings_result,
        active_subs_result,
        owners_result,
        units_result,
        expenses_result,
        meetings_result,
    ) {
        (
            Ok(total_organizations),
            Ok(total_users),
            Ok(total_buildings),
            Ok(active_subscriptions),
            Ok(total_owners),
            Ok(total_units),
            Ok(total_expenses),
            Ok(total_meetings),
        ) => {
            let stats = DashboardStats {
                total_organizations,
                total_users,
                total_buildings,
                active_subscriptions,
                total_owners,
                total_units,
                total_expenses,
                total_meetings,
            };
            HttpResponse::Ok().json(stats)
        }
        _ => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "Failed to fetch dashboard statistics"
        })),
    }
}
