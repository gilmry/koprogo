use crate::application::dto::CreateUnitDto;
use crate::infrastructure::web::AppState;
use actix_web::{get, post, put, web, HttpResponse, Responder};
use uuid::Uuid;
use validator::Validate;

#[post("/units")]
pub async fn create_unit(
    state: web::Data<AppState>,
    dto: web::Json<CreateUnitDto>,
) -> impl Responder {
    if let Err(errors) = dto.validate() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Validation failed",
            "details": errors.to_string()
        }));
    }

    match state.unit_use_cases.create_unit(dto.into_inner()).await {
        Ok(unit) => HttpResponse::Created().json(unit),
        Err(err) => HttpResponse::BadRequest().json(serde_json::json!({
            "error": err
        })),
    }
}

#[get("/units/{id}")]
pub async fn get_unit(state: web::Data<AppState>, id: web::Path<Uuid>) -> impl Responder {
    match state.unit_use_cases.get_unit(*id).await {
        Ok(Some(unit)) => HttpResponse::Ok().json(unit),
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Unit not found"
        })),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": err
        })),
    }
}

#[get("/buildings/{building_id}/units")]
pub async fn list_units_by_building(
    state: web::Data<AppState>,
    building_id: web::Path<Uuid>,
) -> impl Responder {
    match state
        .unit_use_cases
        .list_units_by_building(*building_id)
        .await
    {
        Ok(units) => HttpResponse::Ok().json(units),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": err
        })),
    }
}

#[put("/units/{unit_id}/assign-owner/{owner_id}")]
pub async fn assign_owner(
    state: web::Data<AppState>,
    path: web::Path<(Uuid, Uuid)>,
) -> impl Responder {
    let (unit_id, owner_id) = path.into_inner();

    match state.unit_use_cases.assign_owner(unit_id, owner_id).await {
        Ok(unit) => HttpResponse::Ok().json(unit),
        Err(err) => HttpResponse::BadRequest().json(serde_json::json!({
            "error": err
        })),
    }
}
