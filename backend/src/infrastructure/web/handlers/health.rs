use actix_web::{get, HttpResponse, Responder};
use serde_json::json;

#[get("/health")]
pub async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(json!({
        "status": "ok",
        "service": "koprogo-api"
    }))
}
