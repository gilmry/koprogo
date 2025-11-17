use actix_web::{get, HttpResponse, Responder};
use serde_json::json;

/// Health check endpoint
///
/// Returns system health status. No authentication required.
#[utoipa::path(
    get,
    path = "/api/v1/health",
    tag = "Health",
    responses(
        (status = 200, description = "System is healthy", body = serde_json::Value,
            example = json!({"status": "ok", "service": "koprogo-api"}))
    )
)]
#[get("/health")]
pub async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(json!({
        "status": "ok",
        "service": "koprogo-api"
    }))
}
