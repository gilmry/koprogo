use actix_web::{get, post, web, HttpResponse};
use chrono::Datelike;
use uuid::Uuid;

use crate::application::dto::{
    ContractEvaluationsAnnualReportDto, CreateServiceProviderDto, SearchServiceProvidersQuery,
};
use crate::infrastructure::web::middleware::AuthenticatedUser;
use crate::infrastructure::web::AppState;

/// GET /api/v1/marketplace/providers
/// Search for service providers (public - no authentication required)
#[get("/marketplace/providers")]
pub async fn search_service_providers(
    state: web::Data<AppState>,
    query: web::Query<SearchServiceProvidersQuery>,
) -> Result<HttpResponse, actix_web::Error> {
    let results = state
        .service_provider_use_cases
        .search(&query.into_inner())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().json(results))
}

/// GET /api/v1/marketplace/providers/{slug}
/// Get public service provider profile (no authentication required)
#[get("/marketplace/providers/{slug}")]
pub async fn get_provider_by_slug(
    state: web::Data<AppState>,
    slug: web::Path<String>,
) -> Result<HttpResponse, actix_web::Error> {
    let slug_str = slug.into_inner();
    match state
        .service_provider_use_cases
        .find_by_slug(&slug_str)
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?
    {
        Some(provider) => Ok(HttpResponse::Ok().json(provider)),
        None => Ok(HttpResponse::NotFound().json(serde_json::json!({
            "error": format!("Provider not found: {}", slug_str)
        }))),
    }
}

/// POST /api/v1/service-providers
/// Create a new service provider (authenticated - syndic/admin only)
#[post("/service-providers")]
pub async fn create_service_provider(
    state: web::Data<AppState>,
    request: web::Json<CreateServiceProviderDto>,
    user: AuthenticatedUser,
) -> Result<HttpResponse, actix_web::Error> {
    let org_id = user
        .organization_id
        .ok_or_else(|| actix_web::error::ErrorBadRequest("Organization ID required"))?;

    let response = state
        .service_provider_use_cases
        .create(org_id, request.into_inner())
        .await
        .map_err(actix_web::error::ErrorBadRequest)?;

    Ok(HttpResponse::Created().json(response))
}

/// GET /api/v1/buildings/{building_id}/reports/contract-evaluations/annual
/// Get annual contract evaluations report (L13 legal report)
#[get("/buildings/{building_id}/reports/contract-evaluations/annual")]
pub async fn get_contract_evaluations_annual(
    _state: web::Data<AppState>,
    building_id: web::Path<Uuid>,
    web::Query(params): web::Query<std::collections::HashMap<String, String>>,
    _user: AuthenticatedUser,
) -> Result<HttpResponse, actix_web::Error> {
    let building_id = building_id.into_inner();
    let year = params
        .get("year")
        .and_then(|y| y.parse::<i32>().ok())
        .unwrap_or_else(|| chrono::Local::now().year());

    // TODO: Implement ContractEvaluationRepository for real data
    // For now, return empty report (evaluations are a separate entity not yet persisted)
    let report = ContractEvaluationsAnnualReportDto {
        building_id: building_id.to_string(),
        report_year: year,
        total_evaluations: 0,
        total_providers_evaluated: 0,
        average_global_score: 0.0,
        recommendation_rate: 0.0,
        evaluations: vec![],
    };

    Ok(HttpResponse::Ok().json(report))
}
