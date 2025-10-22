use crate::application::dto::{CreateBuildingDto, UpdateBuildingDto};
use crate::infrastructure::web::AppState;
use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use uuid::Uuid;
use validator::Validate;

#[post("/buildings")]
pub async fn create_building(
    state: web::Data<AppState>,
    dto: web::Json<CreateBuildingDto>,
) -> impl Responder {
    if let Err(errors) = dto.validate() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Validation failed",
            "details": errors.to_string()
        }));
    }

    match state
        .building_use_cases
        .create_building(dto.into_inner())
        .await
    {
        Ok(building) => HttpResponse::Created().json(building),
        Err(err) => HttpResponse::BadRequest().json(serde_json::json!({
            "error": err
        })),
    }
}

#[get("/buildings")]
pub async fn list_buildings(state: web::Data<AppState>) -> impl Responder {
    match state.building_use_cases.list_buildings().await {
        Ok(buildings) => HttpResponse::Ok().json(buildings),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": err
        })),
    }
}

#[get("/buildings/{id}")]
pub async fn get_building(state: web::Data<AppState>, id: web::Path<Uuid>) -> impl Responder {
    match state.building_use_cases.get_building(*id).await {
        Ok(Some(building)) => HttpResponse::Ok().json(building),
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Building not found"
        })),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": err
        })),
    }
}

#[put("/buildings/{id}")]
pub async fn update_building(
    state: web::Data<AppState>,
    id: web::Path<Uuid>,
    dto: web::Json<UpdateBuildingDto>,
) -> impl Responder {
    if let Err(errors) = dto.validate() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Validation failed",
            "details": errors.to_string()
        }));
    }

    match state
        .building_use_cases
        .update_building(*id, dto.into_inner())
        .await
    {
        Ok(building) => HttpResponse::Ok().json(building),
        Err(err) => HttpResponse::BadRequest().json(serde_json::json!({
            "error": err
        })),
    }
}

#[delete("/buildings/{id}")]
pub async fn delete_building(state: web::Data<AppState>, id: web::Path<Uuid>) -> impl Responder {
    match state.building_use_cases.delete_building(*id).await {
        Ok(true) => HttpResponse::NoContent().finish(),
        Ok(false) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Building not found"
        })),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": err
        })),
    }
}
