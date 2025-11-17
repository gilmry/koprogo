use crate::application::dto::{
    CancelExchangeDto, CompleteExchangeDto, CreateLocalExchangeDto,
    LocalExchangeResponseDto, OwnerCreditBalanceDto, OwnerExchangeSummaryDto,
    RateExchangeDto, RequestExchangeDto, SelStatisticsDto,
};
use crate::domain::entities::ExchangeType;
use crate::infrastructure::web::{AppState, AuthenticatedUser};
use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use uuid::Uuid;

/// POST /api/v1/exchanges
/// Create a new exchange offer
#[post("/exchanges")]
pub async fn create_exchange(
    data: web::Data<AppState>,
    auth: AuthenticatedUser,
    request: web::Json<CreateLocalExchangeDto>,
) -> impl Responder {
    match data
        .local_exchange_use_cases
        .create_exchange(auth.user_id, request.into_inner())
        .await
    {
        Ok(exchange) => HttpResponse::Created().json(exchange),
        Err(e) => HttpResponse::BadRequest().json(serde_json::json!({"error": e})),
    }
}

/// GET /api/v1/exchanges/:id
/// Get exchange by ID
#[get("/exchanges/{id}")]
pub async fn get_exchange(
    data: web::Data<AppState>,
    auth: AuthenticatedUser,
    id: web::Path<Uuid>,
) -> impl Responder {
    match data.local_exchange_use_cases.get_exchange(id.into_inner()).await {
        Ok(exchange) => HttpResponse::Ok().json(exchange),
        Err(e) => HttpResponse::NotFound().json(serde_json::json!({"error": e})),
    }
}

/// GET /api/v1/buildings/:building_id/exchanges
/// List all exchanges for a building
#[get("/buildings/{building_id}/exchanges")]
pub async fn list_building_exchanges(
    data: web::Data<AppState>,
    auth: AuthenticatedUser,
    building_id: web::Path<Uuid>,
) -> impl Responder {
    match data
        .local_exchange_use_cases
        .list_building_exchanges(building_id.into_inner())
        .await
    {
        Ok(exchanges) => HttpResponse::Ok().json(exchanges),
        Err(e) => HttpResponse::BadRequest().json(serde_json::json!({"error": e})),
    }
}

/// GET /api/v1/buildings/:building_id/exchanges/available
/// List available exchanges (status = Offered)
#[get("/buildings/{building_id}/exchanges/available")]
pub async fn list_available_exchanges(
    data: web::Data<AppState>,
    auth: AuthenticatedUser,
    building_id: web::Path<Uuid>,
) -> impl Responder {
    match data
        .local_exchange_use_cases
        .list_available_exchanges(building_id.into_inner())
        .await
    {
        Ok(exchanges) => HttpResponse::Ok().json(exchanges),
        Err(e) => HttpResponse::BadRequest().json(serde_json::json!({"error": e})),
    }
}

/// GET /api/v1/owners/:owner_id/exchanges
/// List exchanges for an owner (as provider OR requester)
#[get("/owners/{owner_id}/exchanges")]
pub async fn list_owner_exchanges(
    data: web::Data<AppState>,
    _auth: AuthenticatedUser,
    owner_id: web::Path<Uuid>,
) -> impl Responder {
    let owner_id = owner_id.into_inner();

    // Authorization: users can only see their own exchanges (owner_id = user_id for simplicity)
    // TODO: Implement proper owner-to-user mapping via UnitOwner relationships
    // For now, we allow any authenticated user to query

    match data
        .local_exchange_use_cases
        .list_owner_exchanges(owner_id)
        .await
    {
        Ok(exchanges) => HttpResponse::Ok().json(exchanges),
        Err(e) => HttpResponse::BadRequest().json(serde_json::json!({"error": e})),
    }
}

/// GET /api/v1/buildings/:building_id/exchanges/type/:exchange_type
/// List exchanges by type (Service, ObjectLoan, SharedPurchase)
#[get("/buildings/{building_id}/exchanges/type/{exchange_type}")]
pub async fn list_exchanges_by_type(
    data: web::Data<AppState>,
    _auth: AuthenticatedUser,
    path: web::Path<(Uuid, String)>,
) -> impl Responder {
    let (building_id, exchange_type_str) = path.into_inner();

    // Parse exchange type
    let exchange_type = match exchange_type_str.as_str() {
        "Service" => ExchangeType::Service,
        "ObjectLoan" => ExchangeType::ObjectLoan,
        "SharedPurchase" => ExchangeType::SharedPurchase,
        _ => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid exchange type. Must be Service, ObjectLoan, or SharedPurchase"
            }));
        }
    };

    match data
        .local_exchange_use_cases
        .list_exchanges_by_type(building_id, exchange_type)
        .await
    {
        Ok(exchanges) => HttpResponse::Ok().json(exchanges),
        Err(e) => HttpResponse::BadRequest().json(serde_json::json!({"error": e})),
    }
}

/// POST /api/v1/exchanges/:id/request
/// Request an exchange (Offered → Requested)
#[post("/exchanges/{id}/request")]
pub async fn request_exchange(
    data: web::Data<AppState>,
    auth: AuthenticatedUser,
    id: web::Path<Uuid>,
    request: web::Json<RequestExchangeDto>,
) -> impl Responder {
    match data
        .local_exchange_use_cases
        .request_exchange(id.into_inner(), auth.user_id, request.into_inner())
        .await
    {
        Ok(exchange) => HttpResponse::Ok().json(exchange),
        Err(e) => HttpResponse::BadRequest().json(serde_json::json!({"error": e})),
    }
}

/// POST /api/v1/exchanges/:id/start
/// Start an exchange (Requested → InProgress)
/// Only provider can start
#[post("/exchanges/{id}/start")]
pub async fn start_exchange(
    data: web::Data<AppState>,
    auth: AuthenticatedUser,
    id: web::Path<Uuid>,
) -> impl Responder {
    match data
        .local_exchange_use_cases
        .start_exchange(id.into_inner(), auth.user_id)
        .await
    {
        Ok(exchange) => HttpResponse::Ok().json(exchange),
        Err(e) => HttpResponse::BadRequest().json(serde_json::json!({"error": e})),
    }
}

/// POST /api/v1/exchanges/:id/complete
/// Complete an exchange (InProgress → Completed)
/// Updates credit balances automatically
#[post("/exchanges/{id}/complete")]
pub async fn complete_exchange(
    data: web::Data<AppState>,
    auth: AuthenticatedUser,
    id: web::Path<Uuid>,
    request: web::Json<CompleteExchangeDto>,
) -> impl Responder {
    match data
        .local_exchange_use_cases
        .complete_exchange(id.into_inner(), auth.user_id, request.into_inner())
        .await
    {
        Ok(exchange) => HttpResponse::Ok().json(exchange),
        Err(e) => HttpResponse::BadRequest().json(serde_json::json!({"error": e})),
    }
}

/// POST /api/v1/exchanges/:id/cancel
/// Cancel an exchange
#[post("/exchanges/{id}/cancel")]
pub async fn cancel_exchange(
    data: web::Data<AppState>,
    auth: AuthenticatedUser,
    id: web::Path<Uuid>,
    request: web::Json<CancelExchangeDto>,
) -> impl Responder {
    match data
        .local_exchange_use_cases
        .cancel_exchange(id.into_inner(), auth.user_id, request.into_inner())
        .await
    {
        Ok(exchange) => HttpResponse::Ok().json(exchange),
        Err(e) => HttpResponse::BadRequest().json(serde_json::json!({"error": e})),
    }
}

/// PUT /api/v1/exchanges/:id/rate-provider
/// Rate the provider (by requester)
#[put("/exchanges/{id}/rate-provider")]
pub async fn rate_provider(
    data: web::Data<AppState>,
    auth: AuthenticatedUser,
    id: web::Path<Uuid>,
    request: web::Json<RateExchangeDto>,
) -> impl Responder {
    match data
        .local_exchange_use_cases
        .rate_provider(id.into_inner(), auth.user_id, request.into_inner())
        .await
    {
        Ok(exchange) => HttpResponse::Ok().json(exchange),
        Err(e) => HttpResponse::BadRequest().json(serde_json::json!({"error": e})),
    }
}

/// PUT /api/v1/exchanges/:id/rate-requester
/// Rate the requester (by provider)
#[put("/exchanges/{id}/rate-requester")]
pub async fn rate_requester(
    data: web::Data<AppState>,
    auth: AuthenticatedUser,
    id: web::Path<Uuid>,
    request: web::Json<RateExchangeDto>,
) -> impl Responder {
    match data
        .local_exchange_use_cases
        .rate_requester(id.into_inner(), auth.user_id, request.into_inner())
        .await
    {
        Ok(exchange) => HttpResponse::Ok().json(exchange),
        Err(e) => HttpResponse::BadRequest().json(serde_json::json!({"error": e})),
    }
}

/// DELETE /api/v1/exchanges/:id
/// Delete an exchange (only provider, not completed)
#[delete("/exchanges/{id}")]
pub async fn delete_exchange(
    data: web::Data<AppState>,
    auth: AuthenticatedUser,
    id: web::Path<Uuid>,
) -> impl Responder {
    match data
        .local_exchange_use_cases
        .delete_exchange(id.into_inner(), auth.user_id)
        .await
    {
        Ok(_) => HttpResponse::NoContent().finish(),
        Err(e) => HttpResponse::BadRequest().json(serde_json::json!({"error": e})),
    }
}

/// GET /api/v1/owners/:owner_id/buildings/:building_id/credit-balance
/// Get credit balance for an owner in a building
#[get("/owners/{owner_id}/buildings/{building_id}/credit-balance")]
pub async fn get_credit_balance(
    data: web::Data<AppState>,
    _auth: AuthenticatedUser,
    path: web::Path<(Uuid, Uuid)>,
) -> impl Responder {
    let (owner_id, building_id) = path.into_inner();

    match data
        .local_exchange_use_cases
        .get_credit_balance(owner_id, building_id)
        .await
    {
        Ok(balance) => HttpResponse::Ok().json(balance),
        Err(e) => HttpResponse::BadRequest().json(serde_json::json!({"error": e})),
    }
}

/// GET /api/v1/buildings/:building_id/leaderboard
/// Get leaderboard (top contributors)
#[get("/buildings/{building_id}/leaderboard")]
pub async fn get_leaderboard(
    data: web::Data<AppState>,
    _auth: AuthenticatedUser,
    building_id: web::Path<Uuid>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> impl Responder {
    let limit = query
        .get("limit")
        .and_then(|l| l.parse::<i32>().ok())
        .unwrap_or(10);

    match data
        .local_exchange_use_cases
        .get_leaderboard(building_id.into_inner(), limit)
        .await
    {
        Ok(leaderboard) => HttpResponse::Ok().json(leaderboard),
        Err(e) => HttpResponse::BadRequest().json(serde_json::json!({"error": e})),
    }
}

/// GET /api/v1/buildings/:building_id/sel-statistics
/// Get SEL statistics for a building
#[get("/buildings/{building_id}/sel-statistics")]
pub async fn get_sel_statistics(
    data: web::Data<AppState>,
    _auth: AuthenticatedUser,
    building_id: web::Path<Uuid>,
) -> impl Responder {
    match data
        .local_exchange_use_cases
        .get_statistics(building_id.into_inner())
        .await
    {
        Ok(stats) => HttpResponse::Ok().json(stats),
        Err(e) => HttpResponse::BadRequest().json(serde_json::json!({"error": e})),
    }
}

/// GET /api/v1/owners/:owner_id/exchange-summary
/// Get owner exchange summary
#[get("/owners/{owner_id}/exchange-summary")]
pub async fn get_owner_summary(
    data: web::Data<AppState>,
    _auth: AuthenticatedUser,
    owner_id: web::Path<Uuid>,
) -> impl Responder {
    match data
        .local_exchange_use_cases
        .get_owner_summary(owner_id.into_inner())
        .await
    {
        Ok(summary) => HttpResponse::Ok().json(summary),
        Err(e) => HttpResponse::BadRequest().json(serde_json::json!({"error": e})),
    }
}
