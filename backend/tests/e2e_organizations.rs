// E2E tests for Organization management HTTP endpoints (SuperAdmin only)
// Tests focus on HTTP layer: endpoints, auth, JSON serialization

mod common;

use actix_web::http::header;
use actix_web::{test, App};
use koprogo_api::infrastructure::web::configure_routes;
use serde_json::json;
use serial_test::serial;

#[actix_web::test]
#[serial]
async fn test_organizations_list_requires_superadmin() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = common::register_and_login(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // superadmin can list organizations
    let req = test::TestRequest::get()
        .uri("/api/v1/organizations")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200, "SuperAdmin should list organizations");

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(
        body.get("data").is_some(),
        "Response should have data field"
    );
    assert!(body["data"].is_array(), "data field should be an array");
}

#[actix_web::test]
#[serial]
async fn test_organizations_list_unauthorized() {
    let (app_state, _container, _org_id) = common::setup_test_db().await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/v1/organizations")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401, "Should require authentication");
}

#[actix_web::test]
#[serial]
async fn test_organizations_create() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = common::register_and_login(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/v1/organizations")
        .insert_header(header::ContentType::json())
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "name": "New Test Org",
            "slug": "new-test-org",
            "contact_email": "neworg@test.com",
            "subscription_plan": "starter"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201, "Should create organization");

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["name"], "New Test Org");
    assert_eq!(body["slug"], "new-test-org");
}

#[actix_web::test]
#[serial]
async fn test_organizations_list_contains_created() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = common::register_and_login(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // List — should at least contain the test org
    let req = test::TestRequest::get()
        .uri("/api/v1/organizations")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = test::read_body_json(resp).await;
    let orgs = body["data"].as_array().unwrap();
    assert!(!orgs.is_empty(), "Should have at least one organization");
}
