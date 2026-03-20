// E2E tests for building HTTP endpoints
// Tests focus on HTTP layer: endpoints, auth, JSON serialization
// Covers CRUD operations for buildings in Belgian copropriete management

mod common;

use actix_web::http::header;
use actix_web::{test, App};
use koprogo_api::application::dto::*;
use koprogo_api::infrastructure::web::configure_routes;
use koprogo_api::infrastructure::web::AppState;
use serde_json::json;
use serial_test::serial;
use uuid::Uuid;

/// Helper: Create a building via AppState directly (bypasses HTTP)
async fn create_test_building(
    app_state: &actix_web::web::Data<AppState>,
    org_id: Uuid,
) -> BuildingResponseDto {
    let dto = CreateBuildingDto {
        organization_id: org_id.to_string(),
        name: format!("Test Building {}", Uuid::new_v4()),
        address: "123 Rue de la Paix".to_string(),
        city: "Brussels".to_string(),
        postal_code: "1000".to_string(),
        country: "Belgium".to_string(),
        total_units: 10,
        construction_year: Some(2020),
        total_tantiemes: Some(1000),
    };
    app_state
        .building_use_cases
        .create_building(dto)
        .await
        .expect("Failed to create test building")
}

// ==================== Building CRUD Tests ====================

#[actix_web::test]
#[serial]
async fn test_building_create_success() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = common::register_and_login(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/v1/buildings")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "organization_id": org_id.to_string(),
            "name": "Résidence Les Jardins",
            "address": "45 Avenue Louise",
            "city": "Brussels",
            "postal_code": "1050",
            "country": "Belgium",
            "total_units": 24
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201, "Should create building successfully");

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["name"], "Résidence Les Jardins");
    assert_eq!(body["city"], "Brussels");
    assert_eq!(body["total_units"], 24);
    assert!(body["id"].is_string(), "Should return a UUID id");
}

#[actix_web::test]
#[serial]
async fn test_building_create_missing_name() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = common::register_and_login(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/v1/buildings")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "organization_id": org_id.to_string(),
            "name": "",
            "address": "45 Avenue Louise",
            "city": "Brussels",
            "postal_code": "1050",
            "country": "Belgium",
            "total_units": 10
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400, "Should reject building with empty name");
}

#[actix_web::test]
#[serial]
async fn test_building_get_success() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = common::register_and_login(&app_state, org_id).await;
    let building = create_test_building(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/buildings/{}", building.id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200, "Should return the building");

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["id"], building.id);
    assert_eq!(body["name"], building.name);
}

#[actix_web::test]
#[serial]
async fn test_building_get_not_found() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = common::register_and_login(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let random_id = Uuid::new_v4();
    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/buildings/{}", random_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.status(),
        404,
        "Should return 404 for non-existent building"
    );
}

#[actix_web::test]
#[serial]
async fn test_building_list_returns_array() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = common::register_and_login(&app_state, org_id).await;

    // Create a couple of buildings first
    create_test_building(&app_state, org_id).await;
    create_test_building(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/v1/buildings")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200, "Should list buildings successfully");

    let body: serde_json::Value = test::read_body_json(resp).await;
    // The list endpoint returns a paginated response with a "data" array
    assert!(
        body["data"].is_array() || body.is_array(),
        "Should return an array or paginated response"
    );
}

#[actix_web::test]
#[serial]
async fn test_building_update_success() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = common::register_and_login(&app_state, org_id).await;
    let building = create_test_building(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::put()
        .uri(&format!("/api/v1/buildings/{}", building.id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "name": "Updated Building Name",
            "address": "999 Rue Neuve",
            "city": "Liège",
            "postal_code": "4000",
            "country": "Belgium",
            "total_units": 12
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200, "Should update building successfully");

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["name"], "Updated Building Name");
    assert_eq!(body["city"], "Liège");
}

#[actix_web::test]
#[serial]
async fn test_building_delete_success() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = common::register_and_login(&app_state, org_id).await;
    let building = create_test_building(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::delete()
        .uri(&format!("/api/v1/buildings/{}", building.id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.status(),
        204,
        "Should delete building with 204 NoContent"
    );

    // Verify the building is gone
    let get_req = test::TestRequest::get()
        .uri(&format!("/api/v1/buildings/{}", building.id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();
    let get_resp = test::call_service(&app, get_req).await;
    assert_eq!(
        get_resp.status(),
        404,
        "Building should be 404 after deletion"
    );
}

#[actix_web::test]
#[serial]
async fn test_building_create_requires_auth() {
    let (app_state, _container, org_id) = common::setup_test_db().await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/v1/buildings")
        .set_json(json!({
            "organization_id": org_id.to_string(),
            "name": "Unauthorized Building",
            "address": "1 Rue Test",
            "city": "Brussels",
            "postal_code": "1000",
            "country": "Belgium",
            "total_units": 5
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.status(),
        401,
        "Should require authentication to create buildings"
    );
}
