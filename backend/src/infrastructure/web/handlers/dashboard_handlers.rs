// Infrastructure Web Handlers: Dashboard
//
// HTTP handlers for dashboard endpoints

use crate::infrastructure::web::{AppState, AuthenticatedUser};
use actix_web::{get, web, HttpResponse, Responder};

/// GET /api/v1/dashboard/accountant/stats
/// Get accountant dashboard statistics
#[get("/dashboard/accountant/stats")]
pub async fn get_accountant_stats(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
) -> impl Responder {
    let organization_id = match user.organization_id {
        Some(org_id) => org_id,
        None => {
            return HttpResponse::BadRequest().body("User does not belong to an organization");
        }
    };

    match state
        .dashboard_use_cases
        .get_accountant_stats(organization_id)
        .await
    {
        Ok(stats) => HttpResponse::Ok().json(stats),
        Err(e) => HttpResponse::InternalServerError().body(e),
    }
}

/// GET /api/v1/dashboard/accountant/transactions?limit=10
/// Get recent transactions for dashboard
#[get("/dashboard/accountant/transactions")]
pub async fn get_recent_transactions(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    query: web::Query<RecentTransactionsQuery>,
) -> impl Responder {
    let organization_id = match user.organization_id {
        Some(org_id) => org_id,
        None => {
            return HttpResponse::BadRequest().body("User does not belong to an organization");
        }
    };

    let limit = query.limit.unwrap_or(10).min(50); // Max 50 transactions

    match state
        .dashboard_use_cases
        .get_recent_transactions(organization_id, limit)
        .await
    {
        Ok(transactions) => HttpResponse::Ok().json(transactions),
        Err(e) => HttpResponse::InternalServerError().body(e),
    }
}

#[derive(serde::Deserialize)]
pub struct RecentTransactionsQuery {
    pub limit: Option<usize>,
}
