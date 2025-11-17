use crate::application::dto::{
    CreateQuoteDto, QuoteComparisonRequestDto, QuoteDecisionDto, UpdateQuoteDto,
};
use crate::infrastructure::web::middleware::AuthenticatedUser;
use crate::infrastructure::web::AppState;
use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use uuid::Uuid;

/// POST /api/v1/quotes
/// Create new quote request (Syndic action)
#[post("/quotes")]
pub async fn create_quote(
    data: web::Data<AppState>,
    _auth: AuthenticatedUser,
    request: web::Json<CreateQuoteDto>,
) -> impl Responder {
    match data.quote_use_cases.create_quote(request.into_inner()).await {
        Ok(quote) => HttpResponse::Created().json(quote),
        Err(e) => HttpResponse::BadRequest().json(serde_json::json!({
            "error": e
        })),
    }
}

/// GET /api/v1/quotes/:id
/// Get quote by ID
#[get("/quotes/{id}")]
pub async fn get_quote(
    data: web::Data<AppState>,
    _auth: AuthenticatedUser,
    id: web::Path<Uuid>,
) -> impl Responder {
    match data.quote_use_cases.get_quote(id.into_inner()).await {
        Ok(Some(quote)) => HttpResponse::Ok().json(quote),
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Quote not found"
        })),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e
        })),
    }
}

/// GET /api/v1/buildings/:building_id/quotes
/// List all quotes for a building
#[get("/buildings/{building_id}/quotes")]
pub async fn list_building_quotes(
    data: web::Data<AppState>,
    _auth: AuthenticatedUser,
    building_id: web::Path<Uuid>,
) -> impl Responder {
    match data
        .quote_use_cases
        .list_by_building(building_id.into_inner())
        .await
    {
        Ok(quotes) => HttpResponse::Ok().json(quotes),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e
        })),
    }
}

/// GET /api/v1/contractors/:contractor_id/quotes
/// List all quotes for a contractor
#[get("/contractors/{contractor_id}/quotes")]
pub async fn list_contractor_quotes(
    data: web::Data<AppState>,
    _auth: AuthenticatedUser,
    contractor_id: web::Path<Uuid>,
) -> impl Responder {
    match data
        .quote_use_cases
        .list_by_contractor(contractor_id.into_inner())
        .await
    {
        Ok(quotes) => HttpResponse::Ok().json(quotes),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e
        })),
    }
}

/// GET /api/v1/buildings/:building_id/quotes/status/:status
/// List quotes by status
#[get("/buildings/{building_id}/quotes/status/{status}")]
pub async fn list_quotes_by_status(
    data: web::Data<AppState>,
    _auth: AuthenticatedUser,
    path: web::Path<(Uuid, String)>,
) -> impl Responder {
    let (building_id, status) = path.into_inner();

    match data
        .quote_use_cases
        .list_by_status(building_id, &status)
        .await
    {
        Ok(quotes) => HttpResponse::Ok().json(quotes),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e
        })),
    }
}

/// POST /api/v1/quotes/:id/submit
/// Submit quote (Contractor action)
#[post("/quotes/{id}/submit")]
pub async fn submit_quote(
    data: web::Data<AppState>,
    _auth: AuthenticatedUser,
    id: web::Path<Uuid>,
) -> impl Responder {
    match data.quote_use_cases.submit_quote(id.into_inner()).await {
        Ok(quote) => HttpResponse::Ok().json(quote),
        Err(e) => HttpResponse::BadRequest().json(serde_json::json!({
            "error": e
        })),
    }
}

/// POST /api/v1/quotes/:id/review
/// Start quote review (Syndic action)
#[post("/quotes/{id}/review")]
pub async fn start_review(
    data: web::Data<AppState>,
    _auth: AuthenticatedUser,
    id: web::Path<Uuid>,
) -> impl Responder {
    match data.quote_use_cases.start_review(id.into_inner()).await {
        Ok(quote) => HttpResponse::Ok().json(quote),
        Err(e) => HttpResponse::BadRequest().json(serde_json::json!({
            "error": e
        })),
    }
}

/// POST /api/v1/quotes/:id/accept
/// Accept quote (Syndic action - winner)
#[post("/quotes/{id}/accept")]
pub async fn accept_quote(
    data: web::Data<AppState>,
    _auth: AuthenticatedUser,
    id: web::Path<Uuid>,
    request: web::Json<QuoteDecisionDto>,
) -> impl Responder {
    match data
        .quote_use_cases
        .accept_quote(id.into_inner(), auth.user_id, request.into_inner())
        .await
    {
        Ok(quote) => HttpResponse::Ok().json(quote),
        Err(e) => HttpResponse::BadRequest().json(serde_json::json!({
            "error": e
        })),
    }
}

/// POST /api/v1/quotes/:id/reject
/// Reject quote (Syndic action)
#[post("/quotes/{id}/reject")]
pub async fn reject_quote(
    data: web::Data<AppState>,
    _auth: AuthenticatedUser,
    id: web::Path<Uuid>,
    request: web::Json<QuoteDecisionDto>,
) -> impl Responder {
    match data
        .quote_use_cases
        .reject_quote(id.into_inner(), auth.user_id, request.into_inner())
        .await
    {
        Ok(quote) => HttpResponse::Ok().json(quote),
        Err(e) => HttpResponse::BadRequest().json(serde_json::json!({
            "error": e
        })),
    }
}

/// POST /api/v1/quotes/:id/withdraw
/// Withdraw quote (Contractor action)
#[post("/quotes/{id}/withdraw")]
pub async fn withdraw_quote(
    data: web::Data<AppState>,
    _auth: AuthenticatedUser,
    id: web::Path<Uuid>,
) -> impl Responder {
    match data.quote_use_cases.withdraw_quote(id.into_inner()).await {
        Ok(quote) => HttpResponse::Ok().json(quote),
        Err(e) => HttpResponse::BadRequest().json(serde_json::json!({
            "error": e
        })),
    }
}

/// POST /api/v1/quotes/compare
/// Compare multiple quotes (Belgian legal requirement: 3 quotes minimum)
/// Returns quotes sorted by automatic score (best first)
#[post("/quotes/compare")]
pub async fn compare_quotes(
    data: web::Data<AppState>,
    _auth: AuthenticatedUser,
    request: web::Json<QuoteComparisonRequestDto>,
) -> impl Responder {
    match data
        .quote_use_cases
        .compare_quotes(request.into_inner())
        .await
    {
        Ok(comparison) => HttpResponse::Ok().json(comparison),
        Err(e) => HttpResponse::BadRequest().json(serde_json::json!({
            "error": e
        })),
    }
}

/// PUT /api/v1/quotes/:id/contractor-rating
/// Update contractor rating (for scoring algorithm)
#[put("/quotes/{id}/contractor-rating")]
pub async fn update_contractor_rating(
    data: web::Data<AppState>,
    _auth: AuthenticatedUser,
    id: web::Path<Uuid>,
    request: web::Json<serde_json::Value>,
) -> impl Responder {
    let rating = match request.get("rating").and_then(|v| v.as_i64()) {
        Some(r) => r as i32,
        None => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Rating field is required and must be an integer (0-100)"
            }))
        }
    };

    match data
        .quote_use_cases
        .update_contractor_rating(id.into_inner(), rating)
        .await
    {
        Ok(quote) => HttpResponse::Ok().json(quote),
        Err(e) => HttpResponse::BadRequest().json(serde_json::json!({
            "error": e
        })),
    }
}

/// DELETE /api/v1/quotes/:id
/// Delete quote
#[delete("/quotes/{id}")]
pub async fn delete_quote(
    data: web::Data<AppState>,
    _auth: AuthenticatedUser,
    id: web::Path<Uuid>,
) -> impl Responder {
    match data.quote_use_cases.delete_quote(id.into_inner()).await {
        Ok(true) => HttpResponse::NoContent().finish(),
        Ok(false) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Quote not found"
        })),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e
        })),
    }
}

/// GET /api/v1/buildings/:building_id/quotes/count
/// Count total quotes for building
#[get("/buildings/{building_id}/quotes/count")]
pub async fn count_building_quotes(
    data: web::Data<AppState>,
    _auth: AuthenticatedUser,
    building_id: web::Path<Uuid>,
) -> impl Responder {
    match data
        .quote_use_cases
        .count_by_building(building_id.into_inner())
        .await
    {
        Ok(count) => HttpResponse::Ok().json(serde_json::json!({
            "count": count
        })),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e
        })),
    }
}

/// GET /api/v1/buildings/:building_id/quotes/status/:status/count
/// Count quotes by status for building
#[get("/buildings/{building_id}/quotes/status/{status}/count")]
pub async fn count_quotes_by_status(
    data: web::Data<AppState>,
    _auth: AuthenticatedUser,
    path: web::Path<(Uuid, String)>,
) -> impl Responder {
    let (building_id, status) = path.into_inner();

    match data
        .quote_use_cases
        .count_by_status(building_id, &status)
        .await
    {
        Ok(count) => HttpResponse::Ok().json(serde_json::json!({
            "count": count
        })),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e
        })),
    }
}

#[cfg(test)]
mod tests {
    // Handler tests are covered by E2E tests in tests/e2e/

    #[test]
    fn test_handler_structure_quotes() {
        // This test verifies handler function signatures compile
        // Real testing happens in E2E tests with testcontainers
    }
}
