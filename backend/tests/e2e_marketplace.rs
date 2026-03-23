// E2E tests for Marketplace HTTP endpoints (Issue #276)
// Tests focus on HTTP layer: service provider search, public profiles, contract evaluations
// Covers Belgian contractor marketplace (BCE validation, trade categories)

mod common;

use actix_web::http::header;
use actix_web::{test, App};
use koprogo_api::application::dto::*;
use koprogo_api::infrastructure::web::configure_routes;
use serde_json::json;
use serial_test::serial;
use uuid::Uuid;

// ==================== Helper ====================

/// Create a building for contract evaluation tests (requires auth)
async fn create_test_building_for_marketplace(
    app_state: &actix_web::web::Data<koprogo_api::infrastructure::web::AppState>,
    org_id: Uuid,
) -> String {
    let dto = CreateBuildingDto {
        organization_id: org_id.to_string(),
        name: format!("Marketplace Test Building {}", Uuid::new_v4()),
        address: "100 Marketplace Ave".to_string(),
        city: "Brussels".to_string(),
        postal_code: "1000".to_string(),
        country: "Belgium".to_string(),
        total_units: 4,
        total_tantiemes: Some(1000),
        construction_year: Some(2020),
    };
    let building = app_state
        .building_use_cases
        .create_building(dto)
        .await
        .expect("Failed to create building for marketplace tests");
    building.id
}

// ==================== Search Providers Tests (Public) ====================

#[actix_web::test]
#[serial]
async fn test_marketplace_search_providers_empty() {
    let (app_state, _container, _org_id) = common::setup_test_db().await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Search with no filters - should return empty list (no providers yet)
    let req = test::TestRequest::get()
        .uri("/api/v1/marketplace/providers")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200, "Search should return 200");

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body.is_array(), "Response should be an array");
    assert_eq!(
        body.as_array().unwrap().len(),
        0,
        "Should return empty list when no providers exist"
    );
}

#[actix_web::test]
#[serial]
async fn test_marketplace_search_providers_with_filters() {
    let (app_state, _container, _org_id) = common::setup_test_db().await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Search with category filter
    let req = test::TestRequest::get()
        .uri("/api/v1/marketplace/providers?trade_category=Plombier&postal_code=1000")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.status(),
        200,
        "Filtered search should return 200"
    );

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body.is_array(), "Response should be an array");
}

#[actix_web::test]
#[serial]
async fn test_marketplace_search_no_auth_required() {
    let (app_state, _container, _org_id) = common::setup_test_db().await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // No Authorization header — public endpoint
    let req = test::TestRequest::get()
        .uri("/api/v1/marketplace/providers")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_ne!(
        resp.status(),
        401,
        "Provider search should not require authentication"
    );
}

// ==================== Get Provider by Slug Tests (Public) ====================

#[actix_web::test]
#[serial]
async fn test_marketplace_get_provider_by_slug_not_found() {
    let (app_state, _container, _org_id) = common::setup_test_db().await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/v1/marketplace/providers/nonexistent-provider-slug")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.status(),
        404,
        "Nonexistent provider slug should return 404"
    );

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(
        body["error"].is_string(),
        "Should have error message for not found"
    );
}

#[actix_web::test]
#[serial]
async fn test_marketplace_get_provider_slug_no_auth_required() {
    let (app_state, _container, _org_id) = common::setup_test_db().await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // No Authorization header — public endpoint
    let req = test::TestRequest::get()
        .uri("/api/v1/marketplace/providers/some-slug")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_ne!(
        resp.status(),
        401,
        "Provider slug endpoint should not require authentication"
    );
}

// ==================== Create Service Provider Tests (Auth Required) ====================

#[actix_web::test]
#[serial]
async fn test_marketplace_create_provider() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = common::register_and_login(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/v1/service-providers")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "company_name": "Plomberie Belge SPRL",
            "trade_category": "Plombier",
            "bce_number": "0123.456.789"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.status(),
        201,
        "Should create service provider successfully"
    );

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["company_name"], "Plomberie Belge SPRL");
    assert_eq!(body["trade_category"], "Plombier");
    assert!(body["id"].is_string(), "Should have an ID");
}

#[actix_web::test]
#[serial]
async fn test_marketplace_create_provider_unauthenticated() {
    let (app_state, _container, _org_id) = common::setup_test_db().await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // No auth token
    let req = test::TestRequest::post()
        .uri("/api/v1/service-providers")
        .set_json(json!({
            "company_name": "Test SPRL",
            "trade_category": "Plombier"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.status(),
        401,
        "Creating a provider should require authentication"
    );
}

// ==================== Contract Evaluations Annual Report Tests ====================

#[actix_web::test]
#[serial]
async fn test_marketplace_get_evaluations_report() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = common::register_and_login(&app_state, org_id).await;
    let building_id = create_test_building_for_marketplace(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::get()
        .uri(&format!(
            "/api/v1/buildings/{}/reports/contract-evaluations/annual",
            building_id
        ))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.status(),
        200,
        "Should return annual evaluations report"
    );

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["building_id"], building_id);
    assert!(
        body["report_year"].is_number(),
        "Should have a report year"
    );
    assert!(
        body["total_evaluations"].is_number(),
        "Should have total_evaluations field"
    );
    assert!(
        body["evaluations"].is_array(),
        "Should have evaluations array"
    );
}

#[actix_web::test]
#[serial]
async fn test_marketplace_get_evaluations_report_with_year() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = common::register_and_login(&app_state, org_id).await;
    let building_id = create_test_building_for_marketplace(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::get()
        .uri(&format!(
            "/api/v1/buildings/{}/reports/contract-evaluations/annual?year=2025",
            building_id
        ))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200, "Should return report for specific year");

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["report_year"], 2025);
}

#[actix_web::test]
#[serial]
async fn test_marketplace_evaluations_report_unauthenticated() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let building_id = create_test_building_for_marketplace(
        &app_state,
        org_id,
    )
    .await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::get()
        .uri(&format!(
            "/api/v1/buildings/{}/reports/contract-evaluations/annual",
            building_id
        ))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.status(),
        401,
        "Evaluations report should require authentication"
    );
}
