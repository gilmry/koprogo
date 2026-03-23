use actix_web::{delete, get, post, put, web, HttpResponse};
use uuid::Uuid;

use crate::application::dto::{
    CreateServiceProviderDto, SearchServiceProvidersQuery, ServiceProviderResponseDto,
    CreateContractEvaluationDto, ContractEvaluationResponseDto, ContractEvaluationsAnnualReportDto,
};
use crate::domain::entities::{ServiceProvider, TradeCategory};
use crate::infrastructure::web::middleware::AuthenticatedUser;
use crate::infrastructure::web::AppState;

/// GET /api/v1/marketplace/providers
/// Search for service providers (public - no authentication required)
/// Query params: trade_category, postal_code, min_rating, is_verified_only
#[get("/marketplace/providers")]
pub async fn search_service_providers(
    query: web::Query<SearchServiceProvidersQuery>,
) -> Result<HttpResponse, actix_web::Error> {
    // TODO: Implement search with filters
    // For now, return empty list as placeholder
    let providers: Vec<ServiceProviderResponseDto> = vec![];
    Ok(HttpResponse::Ok().json(providers))
}

/// GET /api/v1/marketplace/providers/{slug}
/// Get public service provider profile (no authentication required)
#[get("/marketplace/providers/{slug}")]
pub async fn get_provider_by_slug(
    slug: web::Path<String>,
) -> Result<HttpResponse, actix_web::Error> {
    let slug_str = slug.into_inner();
    // TODO: Query provider by slug, return public profile
    Ok(HttpResponse::NotFound().json(serde_json::json!({
        "error": format!("Provider not found: {}", slug_str)
    })))
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

    // Parse trade category
    let trade_category = TradeCategory::from_sql(&request.trade_category)
        .map_err(actix_web::error::ErrorBadRequest)?;

    // Create service provider
    let provider = ServiceProvider::new(
        org_id,
        request.company_name.clone(),
        trade_category,
        request.bce_number.clone(),
    )
    .map_err(actix_web::error::ErrorBadRequest)?;

    // TODO: Save to database using repository
    // For now, just return the created provider
    let response = ServiceProviderResponseDto::from(provider);
    Ok(HttpResponse::Created().json(response))
}

/// GET /api/v1/buildings/{building_id}/reports/contract-evaluations/annual
/// Get annual contract evaluations report (L13 legal report)
/// Query param: year (default: current year)
#[get("/buildings/{building_id}/reports/contract-evaluations/annual")]
pub async fn get_contract_evaluations_annual(
    state: web::Data<AppState>,
    building_id: web::Path<Uuid>,
    web::Query(params): web::Query<std::collections::HashMap<String, String>>,
    user: AuthenticatedUser,
) -> Result<HttpResponse, actix_web::Error> {
    let building_id = building_id.into_inner();
    let year = params
        .get("year")
        .and_then(|y| y.parse::<i32>().ok())
        .unwrap_or_else(|| chrono::Local::now().year());

    // TODO: Query contract evaluations for building and year
    // Filter: is_legal_evaluation = true
    // Calculate: average score, recommendation rate
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

#[cfg(test)]
mod tests {
    #[test]
    fn test_marketplace_handlers_compile() {
        // Placeholder test to verify handler structure
    }
}
