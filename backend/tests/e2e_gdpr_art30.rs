// E2E tests for GDPR Article 30 - Processing Register & Sub-Processors (Issue #316)
// Tests focus on HTTP layer: endpoints, auth, JSON serialization
// Covers Art. 30 data processing activities register and DPA sub-processor management

mod common;

use actix_web::http::header;
use actix_web::{test, App};
use koprogo_api::infrastructure::web::configure_routes;
use koprogo_api::infrastructure::web::AppState;
use serde_json::json;
use serial_test::serial;
use uuid::Uuid;

// ==================== Test Setup ====================

async fn create_superadmin_token(
    app_state: &actix_web::web::Data<AppState>,
    org_id: Uuid,
) -> String {
    common::register_and_login(app_state, org_id).await
}

async fn create_non_superadmin_token(
    app_state: &actix_web::web::Data<AppState>,
    org_id: Uuid,
) -> String {
    let email = format!("syndic+{}@test.com", Uuid::new_v4());
    let reg = koprogo_api::application::dto::RegisterRequest {
        email: email.clone(),
        password: "Passw0rd!".to_string(),
        first_name: "Syndic".to_string(),
        last_name: "User".to_string(),
        role: "syndic".to_string(),
        organization_id: Some(org_id),
    };
    let _ = app_state
        .auth_use_cases
        .register(reg)
        .await
        .expect("register syndic");
    let login_req = koprogo_api::application::dto::LoginRequest {
        email,
        password: "Passw0rd!".to_string(),
    };
    app_state
        .auth_use_cases
        .login(login_req)
        .await
        .expect("login syndic")
        .token
}

// ==================== Processing Activities Tests ====================

#[actix_web::test]
#[serial]
async fn test_list_processing_activities_superadmin() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = create_superadmin_token(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/v1/admin/gdpr/processing-register")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.status(),
        200,
        "SuperAdmin should be able to list processing activities"
    );

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body["activities"].is_array());
    assert!(body["total"].is_number());
}

#[actix_web::test]
#[serial]
async fn test_list_processing_activities_non_superadmin_forbidden() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = create_non_superadmin_token(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/v1/admin/gdpr/processing-register")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.status(),
        403,
        "Non-SuperAdmin should be denied access to processing register"
    );

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body["error"].as_str().unwrap().contains("SuperAdmin"));
}

#[actix_web::test]
#[serial]
async fn test_list_processing_activities_without_auth() {
    let (app_state, _container, _org_id) = common::setup_test_db().await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/v1/admin/gdpr/processing-register")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.status(),
        401,
        "Should require authentication"
    );
}

// ==================== Sub-Processors Tests ====================

#[actix_web::test]
#[serial]
async fn test_list_sub_processors_superadmin() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = create_superadmin_token(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/v1/admin/gdpr/processors")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.status(),
        200,
        "SuperAdmin should be able to list sub-processors"
    );

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body["processors"].is_array());
    assert!(body["total"].is_number());
}

#[actix_web::test]
#[serial]
async fn test_list_sub_processors_non_superadmin_forbidden() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = create_non_superadmin_token(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/v1/admin/gdpr/processors")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.status(),
        403,
        "Non-SuperAdmin should be denied access to sub-processors"
    );

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body["error"].as_str().unwrap().contains("SuperAdmin"));
}

#[actix_web::test]
#[serial]
async fn test_list_sub_processors_without_auth() {
    let (app_state, _container, _org_id) = common::setup_test_db().await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/v1/admin/gdpr/processors")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.status(),
        401,
        "Should require authentication"
    );
}

// ==================== Response Structure Tests ====================

#[actix_web::test]
#[serial]
async fn test_processing_activities_response_structure() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = create_superadmin_token(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/v1/admin/gdpr/processing-register")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = test::read_body_json(resp).await;

    // Verify response structure has expected fields
    assert!(body.get("activities").is_some(), "Response should have 'activities' field");
    assert!(body.get("total").is_some(), "Response should have 'total' field");

    // If there are activities, verify each activity has the expected structure
    if let Some(activities) = body["activities"].as_array() {
        for activity in activities {
            assert!(activity.get("id").is_some());
            assert!(activity.get("activity_name").is_some());
            assert!(activity.get("controller_name").is_some());
            assert!(activity.get("purpose").is_some());
            assert!(activity.get("legal_basis").is_some());
            assert!(activity.get("data_categories").is_some());
            assert!(activity.get("retention_period").is_some());
            assert!(activity.get("security_measures").is_some());
        }
    }
}
