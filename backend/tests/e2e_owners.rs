// E2E tests for owner HTTP endpoints
// Tests focus on HTTP layer: endpoints, auth, JSON serialization
// Covers CRUD operations for co-owners (copropriétaires) in Belgian copropriete management

mod common;

use actix_web::http::header;
use actix_web::{test, App};
use koprogo_api::application::dto::*;
use koprogo_api::infrastructure::web::configure_routes;
use koprogo_api::infrastructure::web::AppState;
use serde_json::json;
use serial_test::serial;
use uuid::Uuid;

/// Helper: Create an owner via AppState directly
async fn create_test_owner(
    app_state: &actix_web::web::Data<AppState>,
    org_id: Uuid,
) -> OwnerResponseDto {
    let unique_suffix = Uuid::new_v4().to_string().replace('-', "")[..8].to_string();
    let dto = CreateOwnerDto {
        organization_id: org_id.to_string(),
        first_name: "Jean".to_string(),
        last_name: format!("Dupont-{}", unique_suffix),
        email: format!("jean.dupont.{}@example.be", unique_suffix),
        phone: Some("+32 2 123 4567".to_string()),
        address: "25 Avenue de la Couronne".to_string(),
        city: "Brussels".to_string(),
        postal_code: "1050".to_string(),
        country: "Belgium".to_string(),
        user_id: None,
    };
    app_state
        .owner_use_cases
        .create_owner(dto)
        .await
        .expect("Failed to create test owner")
}

// ==================== Owner CRUD Tests ====================

#[actix_web::test]
#[serial]
async fn test_owner_create_success() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = common::register_and_login(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let unique_suffix = Uuid::new_v4().to_string().replace('-', "")[..8].to_string();
    let req = test::TestRequest::post()
        .uri("/api/v1/owners")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "organization_id": org_id.to_string(),
            "first_name": "Marie",
            "last_name": "Lecomte",
            "email": format!("marie.lecomte.{}@example.be", unique_suffix),
            "phone": "+32 475 123 456",
            "address": "10 Rue de la Loi",
            "city": "Brussels",
            "postal_code": "1000",
            "country": "Belgium"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201, "Should create owner successfully");

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["first_name"], "Marie");
    assert_eq!(body["last_name"], "Lecomte");
    assert_eq!(body["city"], "Brussels");
    assert!(body["id"].is_string(), "Should return a UUID id");
}

#[actix_web::test]
#[serial]
async fn test_owner_create_invalid_email() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = common::register_and_login(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/v1/owners")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "organization_id": org_id.to_string(),
            "first_name": "Paul",
            "last_name": "Martin",
            "email": "not-a-valid-email",
            "address": "5 Rue du Commerce",
            "city": "Liège",
            "postal_code": "4000",
            "country": "Belgium"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.status(),
        400,
        "Should reject owner with invalid email format"
    );
}

#[actix_web::test]
#[serial]
async fn test_owner_get_success() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = common::register_and_login(&app_state, org_id).await;
    let owner = create_test_owner(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/owners/{}", owner.id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200, "Should return the owner");

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["id"], owner.id);
    assert_eq!(body["first_name"], owner.first_name);
    assert_eq!(body["last_name"], owner.last_name);
}

#[actix_web::test]
#[serial]
async fn test_owner_list_returns_paginated() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = common::register_and_login(&app_state, org_id).await;

    // Create some owners
    create_test_owner(&app_state, org_id).await;
    create_test_owner(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/v1/owners")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200, "Should list owners successfully");

    let body: serde_json::Value = test::read_body_json(resp).await;
    // The list endpoint returns a paginated response with a "data" array
    assert!(
        body["data"].is_array() || body.is_array(),
        "Should return an array or paginated response"
    );
}

#[actix_web::test]
#[serial]
async fn test_owner_get_not_found() {
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
        .uri(&format!("/api/v1/owners/{}", random_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.status(),
        404,
        "Should return 404 for non-existent owner"
    );
}

#[actix_web::test]
#[serial]
async fn test_owner_create_requires_auth() {
    let (app_state, _container, org_id) = common::setup_test_db().await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let unique_suffix = Uuid::new_v4().to_string().replace('-', "")[..8].to_string();
    let req = test::TestRequest::post()
        .uri("/api/v1/owners")
        .set_json(json!({
            "organization_id": org_id.to_string(),
            "first_name": "Unauthorized",
            "last_name": "User",
            "email": format!("unauthorized.{}@example.be", unique_suffix),
            "address": "1 Rue Test",
            "city": "Brussels",
            "postal_code": "1000",
            "country": "Belgium"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.status(),
        401,
        "Should require authentication to create owners"
    );
}
