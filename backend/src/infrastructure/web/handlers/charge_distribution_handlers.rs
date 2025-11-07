use crate::infrastructure::web::{AppState, AuthenticatedUser};
use actix_web::{get, post, web, HttpResponse, Responder};
use uuid::Uuid;

/// POST /invoices/{id}/calculate-distribution - Calculate and save charge distribution
/// Automatically called after invoice approval, or can be triggered manually
/// Only accountant, syndic, or superadmin can calculate distribution
#[post("/invoices/{expense_id}/calculate-distribution")]
pub async fn calculate_and_save_distribution(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    expense_id: web::Path<Uuid>,
) -> impl Responder {
    // Check permissions
    if user.role != "accountant" && user.role != "syndic" && user.role != "superadmin" {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Only accountant, syndic, or superadmin can calculate charge distributions"
        }));
    }

    match state
        .charge_distribution_use_cases
        .calculate_and_save_distribution(*expense_id)
        .await
    {
        Ok(distributions) => HttpResponse::Ok().json(serde_json::json!({
            "message": "Charge distribution calculated successfully",
            "count": distributions.len(),
            "distributions": distributions
        })),
        Err(err) => HttpResponse::BadRequest().json(serde_json::json!({
            "error": err
        })),
    }
}

/// GET /invoices/{id}/distribution - Get charge distribution for an invoice
#[get("/invoices/{expense_id}/distribution")]
pub async fn get_distribution_by_expense(
    state: web::Data<AppState>,
    expense_id: web::Path<Uuid>,
) -> impl Responder {
    match state
        .charge_distribution_use_cases
        .get_distribution_by_expense(*expense_id)
        .await
    {
        Ok(distributions) => HttpResponse::Ok().json(distributions),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": err
        })),
    }
}

/// GET /owners/{id}/distributions - Get all charge distributions for an owner
#[get("/owners/{owner_id}/distributions")]
pub async fn get_distributions_by_owner(
    state: web::Data<AppState>,
    owner_id: web::Path<Uuid>,
) -> impl Responder {
    match state
        .charge_distribution_use_cases
        .get_distributions_by_owner(*owner_id)
        .await
    {
        Ok(distributions) => HttpResponse::Ok().json(distributions),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": err
        })),
    }
}

/// GET /owners/{id}/total-due - Get total amount due for an owner
#[get("/owners/{owner_id}/total-due")]
pub async fn get_total_due_by_owner(
    state: web::Data<AppState>,
    owner_id: web::Path<Uuid>,
) -> impl Responder {
    match state
        .charge_distribution_use_cases
        .get_total_due_by_owner(*owner_id)
        .await
    {
        Ok(total_due) => HttpResponse::Ok().json(serde_json::json!({
            "owner_id": owner_id.to_string(),
            "total_due": total_due
        })),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": err
        })),
    }
}
