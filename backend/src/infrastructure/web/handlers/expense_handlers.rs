use crate::application::dto::CreateExpenseDto;
use crate::infrastructure::web::AppState;
use actix_web::{get, post, put, web, HttpResponse, Responder};
use uuid::Uuid;
use validator::Validate;

#[post("/expenses")]
pub async fn create_expense(
    state: web::Data<AppState>,
    dto: web::Json<CreateExpenseDto>,
) -> impl Responder {
    if let Err(errors) = dto.validate() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Validation failed",
            "details": errors.to_string()
        }));
    }

    match state
        .expense_use_cases
        .create_expense(dto.into_inner())
        .await
    {
        Ok(expense) => HttpResponse::Created().json(expense),
        Err(err) => HttpResponse::BadRequest().json(serde_json::json!({
            "error": err
        })),
    }
}

#[get("/expenses/{id}")]
pub async fn get_expense(state: web::Data<AppState>, id: web::Path<Uuid>) -> impl Responder {
    match state.expense_use_cases.get_expense(*id).await {
        Ok(Some(expense)) => HttpResponse::Ok().json(expense),
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Expense not found"
        })),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": err
        })),
    }
}

#[get("/buildings/{building_id}/expenses")]
pub async fn list_expenses_by_building(
    state: web::Data<AppState>,
    building_id: web::Path<Uuid>,
) -> impl Responder {
    match state
        .expense_use_cases
        .list_expenses_by_building(*building_id)
        .await
    {
        Ok(expenses) => HttpResponse::Ok().json(expenses),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": err
        })),
    }
}

#[put("/expenses/{id}/mark-paid")]
pub async fn mark_expense_paid(state: web::Data<AppState>, id: web::Path<Uuid>) -> impl Responder {
    match state.expense_use_cases.mark_as_paid(*id).await {
        Ok(expense) => HttpResponse::Ok().json(expense),
        Err(err) => HttpResponse::BadRequest().json(serde_json::json!({
            "error": err
        })),
    }
}
