// E2E tests for unit HTTP endpoints
// Tests focus on HTTP layer: endpoints, auth, JSON serialization
// Covers CRUD operations for units (lots) in Belgian copropriete buildings

mod common;

use actix_web::http::header;
use actix_web::{test, App};
use koprogo_api::application::dto::*;
use koprogo_api::domain::entities::UnitType;
use koprogo_api::infrastructure::web::configure_routes;
use koprogo_api::infrastructure::web::AppState;
use serde_json::json;
use serial_test::serial;
use uuid::Uuid;

/// Helper: Create a building via AppState directly
async fn create_test_building_for_units(
    app_state: &actix_web::web::Data<AppState>,
    org_id: Uuid,
) -> BuildingResponseDto {
    let dto = CreateBuildingDto {
        organization_id: org_id.to_string(),
        name: format!("Unit Test Building {}", Uuid::new_v4()),
        address: "10 Rue du Test".to_string(),
        city: "Ghent".to_string(),
        postal_code: "9000".to_string(),
        country: "Belgium".to_string(),
        total_units: 20,
        construction_year: Some(2010),
        total_tantiemes: Some(1000),
    };
    app_state
        .building_use_cases
        .create_building(dto)
        .await
        .expect("Failed to create test building for units")
}

/// Helper: Create a unit via AppState directly
async fn create_test_unit(
    app_state: &actix_web::web::Data<AppState>,
    org_id: Uuid,
    building_id: &str,
) -> UnitResponseDto {
    let dto = CreateUnitDto {
        organization_id: org_id.to_string(),
        building_id: building_id.to_string(),
        unit_number: format!("A{}", Uuid::new_v4().to_string()[..4].to_uppercase()),
        unit_type: UnitType::Apartment,
        floor: Some(1),
        surface_area: 75.0,
        quota: 100.0,
    };
    app_state
        .unit_use_cases
        .create_unit(dto)
        .await
        .expect("Failed to create test unit")
}

// ==================== Unit CRUD Tests ====================

#[actix_web::test]
#[serial]
async fn test_unit_create_success() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = common::register_and_login(&app_state, org_id).await;
    let building = create_test_building_for_units(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/v1/units")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "organization_id": org_id.to_string(),
            "building_id": building.id,
            "unit_number": "B101",
            "unit_type": "Apartment",
            "floor": 1,
            "surface_area": 85.5,
            "quota": 150.0
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201, "Should create unit successfully");

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["unit_number"], "B101");
    assert_eq!(body["unit_type"], "Apartment");
    assert_eq!(body["building_id"], building.id);
    assert!(body["id"].is_string(), "Should return a UUID id");
}

#[actix_web::test]
#[serial]
async fn test_unit_create_invalid_quota_zero() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = common::register_and_login(&app_state, org_id).await;
    let building = create_test_building_for_units(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/v1/units")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "organization_id": org_id.to_string(),
            "building_id": building.id,
            "unit_number": "C001",
            "unit_type": "Parking",
            "floor": 0,
            "surface_area": 15.0,
            "quota": 0.0
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.status(),
        400,
        "Should reject unit with quota = 0.0 (min is 0.1)"
    );
}

#[actix_web::test]
#[serial]
async fn test_unit_get_success() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = common::register_and_login(&app_state, org_id).await;
    let building = create_test_building_for_units(&app_state, org_id).await;
    let unit = create_test_unit(&app_state, org_id, &building.id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/units/{}", unit.id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200, "Should return the unit");

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["id"], unit.id);
    assert_eq!(body["building_id"], building.id);
}

#[actix_web::test]
#[serial]
async fn test_unit_list_by_building() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = common::register_and_login(&app_state, org_id).await;
    let building = create_test_building_for_units(&app_state, org_id).await;

    // Create two units in the building
    create_test_unit(&app_state, org_id, &building.id).await;
    create_test_unit(&app_state, org_id, &building.id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/buildings/{}/units", building.id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200, "Should list units by building");

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body.is_array(), "Should return an array of units");
    assert!(
        body.as_array().unwrap().len() >= 2,
        "Should contain at least 2 units"
    );
}

#[actix_web::test]
#[serial]
async fn test_unit_update_success() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = common::register_and_login(&app_state, org_id).await;
    let building = create_test_building_for_units(&app_state, org_id).await;
    let unit = create_test_unit(&app_state, org_id, &building.id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::put()
        .uri(&format!("/api/v1/units/{}", unit.id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "unit_number": "D202",
            "unit_type": "Commercial",
            "floor": 2,
            "surface_area": 120.0,
            "quota": 200.0
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200, "Should update unit successfully");

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["unit_number"], "D202");
    assert_eq!(body["unit_type"], "Commercial");
}

#[actix_web::test]
#[serial]
async fn test_unit_create_requires_auth() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let building = create_test_building_for_units(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/v1/units")
        .set_json(json!({
            "organization_id": org_id.to_string(),
            "building_id": building.id,
            "unit_number": "E303",
            "unit_type": "Apartment",
            "floor": 3,
            "surface_area": 60.0,
            "quota": 100.0
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.status(),
        401,
        "Should require authentication to create units"
    );
}
