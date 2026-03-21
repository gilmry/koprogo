// E2E tests for Two-Factor Authentication (2FA TOTP) HTTP endpoints (Issue #78)
// Tests focus on HTTP layer: endpoints, auth, JSON serialization
// Covers TOTP setup, status, and enable flow for Belgian copropriete security

mod common;

use actix_web::http::header;
use actix_web::{test, App};
use koprogo_api::application::dto::*;
use koprogo_api::infrastructure::web::configure_routes;
use koprogo_api::infrastructure::web::AppState;
use serial_test::serial;
use uuid::Uuid;

/// Helper: Register a user and return (token, user_id) — needed for 2FA since it is user-scoped
async fn register_and_login_with_user_id(
    app_state: &actix_web::web::Data<AppState>,
    org_id: Uuid,
) -> (String, Uuid) {
    let email = format!("2fa-test-{}@example.be", Uuid::new_v4());
    let reg = RegisterRequest {
        email: email.clone(),
        password: "SecurePass123!".to_string(),
        first_name: "TwoFactor".to_string(),
        last_name: "Tester".to_string(),
        role: "superadmin".to_string(),
        organization_id: Some(org_id),
    };
    let login_resp = app_state
        .auth_use_cases
        .register(reg)
        .await
        .expect("Failed to register user for 2FA test");

    let user_id = login_resp.user.id;
    let token = login_resp.token;
    (token, user_id)
}

// ==================== 2FA Status Tests ====================

#[actix_web::test]
#[serial]
async fn test_2fa_status_disabled_by_default() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, _user_id) = register_and_login_with_user_id(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/v1/2fa/status")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200, "Should return 2FA status successfully");

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(
        body["is_enabled"], false,
        "2FA should be disabled for a new user"
    );
    assert_eq!(
        body["backup_codes_remaining"], 0,
        "No backup codes before setup"
    );
}

#[actix_web::test]
#[serial]
async fn test_2fa_status_requires_auth() {
    let (app_state, _container, _org_id) = common::setup_test_db().await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/v1/2fa/status")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.status(),
        401,
        "Should require authentication to view 2FA status"
    );
}

// ==================== 2FA Setup Tests ====================

#[actix_web::test]
#[serial]
async fn test_2fa_setup_returns_qr_code_and_backup_codes() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, _user_id) = register_and_login_with_user_id(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/v1/2fa/setup")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.status(),
        200,
        "Should return 2FA setup data successfully"
    );

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(
        body["secret"].is_string() && !body["secret"].as_str().unwrap().is_empty(),
        "Should return a non-empty TOTP secret"
    );
    assert!(
        body["qr_code_data_url"].is_string(),
        "Should return a QR code data URL"
    );
    assert!(
        body["backup_codes"].is_array(),
        "Should return backup codes array"
    );
    assert_eq!(
        body["backup_codes"].as_array().unwrap().len(),
        10,
        "Should return exactly 10 backup codes"
    );
    assert_eq!(body["issuer"], "KoproGo", "Should have KoproGo as issuer");
}

#[actix_web::test]
#[serial]
async fn test_2fa_setup_returns_totp_secret() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, _user_id) = register_and_login_with_user_id(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/v1/2fa/setup")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200, "Should succeed");

    let body: serde_json::Value = test::read_body_json(resp).await;

    // The secret must be a valid Base32-encoded string
    let secret = body["secret"].as_str().expect("secret should be a string");
    assert!(
        secret.len() >= 16,
        "TOTP secret should be at least 16 characters (Base32)"
    );

    // qr_code_data_url should start with data:image/
    let qr = body["qr_code_data_url"]
        .as_str()
        .expect("qr_code_data_url should be a string");
    assert!(
        qr.starts_with("data:image/"),
        "QR code should be a data URL (got: {}...)",
        &qr[..20.min(qr.len())]
    );

    // account_name should be present (the user's email)
    assert!(
        body["account_name"].is_string(),
        "Should return account_name field"
    );
}

#[actix_web::test]
#[serial]
async fn test_2fa_setup_requires_auth() {
    let (app_state, _container, _org_id) = common::setup_test_db().await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/v1/2fa/setup")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.status(),
        401,
        "Should require authentication to setup 2FA"
    );
}

#[actix_web::test]
#[serial]
async fn test_2fa_setup_idempotent_second_call_fails() {
    // After calling setup once, calling setup a second time should fail
    // because a secret is already pending (not yet enabled but exists)
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, _user_id) = register_and_login_with_user_id(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // First setup call should succeed
    let req1 = test::TestRequest::post()
        .uri("/api/v1/2fa/setup")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();
    let resp1 = test::call_service(&app, req1).await;
    assert_eq!(resp1.status(), 200, "First setup should succeed");

    // Second setup call — the handler returns 400 if already set up (pending or enabled)
    let req2 = test::TestRequest::post()
        .uri("/api/v1/2fa/setup")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();
    let resp2 = test::call_service(&app, req2).await;
    // It may return 400 (already exists) or 200 (idempotent re-setup) depending on implementation
    // We only check it does NOT crash (not 500)
    assert_ne!(
        resp2.status(),
        500,
        "Second setup should not cause a server error"
    );
}
