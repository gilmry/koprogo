// E2E tests for User management HTTP endpoints (SuperAdmin only)
// Tests focus on HTTP layer: endpoints, auth, JSON serialization

mod common;

use actix_web::http::header;
use actix_web::{test, App};
use koprogo_api::infrastructure::web::configure_routes;
use serde_json::json;
use serial_test::serial;
use uuid::Uuid;

#[actix_web::test]
#[serial]
async fn test_users_list_superadmin() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = common::register_and_login(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/v1/users")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200, "SuperAdmin should list users");

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(
        body.get("data").is_some(),
        "Response should have data field"
    );
    assert!(body["data"].is_array(), "data field should be an array");
    assert!(
        !body["data"].as_array().unwrap().is_empty(),
        "Should have at least one user (the registered one)"
    );
}

#[actix_web::test]
#[serial]
async fn test_users_list_unauthorized() {
    let (app_state, _container, _org_id) = common::setup_test_db().await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::get().uri("/api/v1/users").to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401, "Should require authentication");
}

#[actix_web::test]
#[serial]
async fn test_users_create_superadmin() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = common::register_and_login(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let unique = Uuid::new_v4();
    let req = test::TestRequest::post()
        .uri("/api/v1/users")
        .insert_header(header::ContentType::json())
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "email": format!("newuser-{}@test.com", unique),
            "password": "SecurePass123!",
            "first_name": "New",
            "last_name": "User",
            "role": "syndic",
            "organization_id": org_id.to_string()
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201, "Should create user");

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["first_name"], "New");
    assert_eq!(body["last_name"], "User");
    assert!(body["id"].as_str().is_some());
}

#[actix_web::test]
#[serial]
async fn test_users_deactivate() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = common::register_and_login(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create a user to deactivate
    let unique = Uuid::new_v4();
    let create_req = test::TestRequest::post()
        .uri("/api/v1/users")
        .insert_header(header::ContentType::json())
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "email": format!("todeactivate-{}@test.com", unique),
            "password": "SecurePass123!",
            "first_name": "To",
            "last_name": "Deactivate",
            "role": "owner",
            "organization_id": org_id.to_string()
        }))
        .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    assert_eq!(create_resp.status(), 201);
    let body: serde_json::Value = test::read_body_json(create_resp).await;
    let user_id = body["id"].as_str().unwrap();

    // Deactivate
    let deactivate_req = test::TestRequest::put()
        .uri(&format!("/api/v1/users/{}/deactivate", user_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let deactivate_resp = test::call_service(&app, deactivate_req).await;
    assert_eq!(
        deactivate_resp.status(),
        200,
        "Should deactivate user successfully"
    );

    let deactivated: serde_json::Value = test::read_body_json(deactivate_resp).await;
    assert_eq!(deactivated["is_active"], false);
}
