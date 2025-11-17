use crate::application::dto::PublicSyndicInfoResponse;
use crate::infrastructure::web::AppState;
use actix_web::{get, web, HttpResponse, Responder};

/// GET /api/v1/public/buildings/:slug/syndic
/// Get public syndic information for a building (no authentication required)
///
/// This endpoint is publicly accessible to comply with Belgian law requiring
/// syndics to display contact information publicly.
///
/// # Path Parameters
/// * `slug` - URL-friendly building identifier (e.g., "residence-les-jardins-brussels")
///
/// # Returns
/// * `200 OK` - Public syndic information
/// * `404 Not Found` - Building not found or slug invalid
/// * `500 Internal Server Error` - Database error
///
/// # Example
/// ```
/// GET /api/v1/public/buildings/residence-les-jardins-brussels/syndic
///
/// Response 200 OK:
/// {
///   "building_name": "Résidence Les Jardins",
///   "building_address": "123 Rue de la Paix",
///   "building_city": "Brussels",
///   "building_postal_code": "1000",
///   "building_country": "Belgium",
///   "slug": "residence-les-jardins-brussels",
///   "syndic_name": "Syndic ASBL",
///   "syndic_email": "contact@syndic.be",
///   "syndic_phone": "+32 2 123 4567",
///   "syndic_address": "Avenue Louise 123, 1000 Brussels",
///   "syndic_office_hours": "Mon-Fri 9h-17h",
///   "syndic_emergency_contact": "+32 475 123 456",
///   "has_syndic_info": true
/// }
/// ```
#[get("/public/buildings/{slug}/syndic")]
pub async fn get_public_syndic_info(
    data: web::Data<AppState>,
    slug: web::Path<String>,
) -> impl Responder {
    let slug = slug.into_inner();

    // Find building by slug using BuildingUseCases
    match data.building_use_cases.find_by_slug(&slug).await {
        Ok(Some(building)) => {
            let response = PublicSyndicInfoResponse::from(building);
            HttpResponse::Ok().json(response)
        }
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({
            "error": format!("Building not found with slug: {}", slug)
        })),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to fetch building information: {}", e)
        })),
    }
}

#[cfg(test)]
mod tests {
    // Note: Full integration tests with actual AppState would require proper initialization
    // of all use cases. These handler tests are covered by E2E tests in tests/e2e/

    #[test]
    fn test_handler_structure_public_syndic() {
        // This test just verifies the handler function signature compiles
        // Real testing happens in E2E tests with testcontainers
    }
}
