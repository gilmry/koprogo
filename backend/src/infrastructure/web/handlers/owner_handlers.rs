use crate::application::dto::CreateOwnerDto;
use crate::infrastructure::web::AppState;
use actix_web::{get, post, web, HttpResponse, Responder};
use uuid::Uuid;
use validator::Validate;

#[post("/owners")]
pub async fn create_owner(
    state: web::Data<AppState>,
    dto: web::Json<CreateOwnerDto>,
) -> impl Responder {
    if let Err(errors) = dto.validate() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Validation failed",
            "details": errors.to_string()
        }));
    }

    match state.owner_use_cases.create_owner(dto.into_inner()).await {
        Ok(owner) => HttpResponse::Created().json(owner),
        Err(err) => HttpResponse::BadRequest().json(serde_json::json!({
            "error": err
        })),
    }
}

#[get("/owners")]
pub async fn list_owners(state: web::Data<AppState>) -> impl Responder {
    match state.owner_use_cases.list_owners().await {
        Ok(owners) => HttpResponse::Ok().json(owners),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": err
        })),
    }
}

#[get("/owners/{id}")]
pub async fn get_owner(state: web::Data<AppState>, id: web::Path<Uuid>) -> impl Responder {
    match state.owner_use_cases.get_owner(*id).await {
        Ok(Some(owner)) => HttpResponse::Ok().json(owner),
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Owner not found"
        })),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": err
        })),
    }
}
