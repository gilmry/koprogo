// E2E tests for API Key Management HTTP endpoints (Issues #111, #232)
// Tests focus on HTTP layer: endpoints, auth, JSON serialization
// Covers API key lifecycle for third-party integrations

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

#[allow(dead_code)]
async fn create_syndic_token(app_state: &actix_web::web::Data<AppState>, org_id: Uuid) -> String {
    let email = format!("syndic+{}@test.com", Uuid::new_v4());
    let reg = koprogo_api::application::dto::RegisterRequest {
        email: email.clone(),
        password: "Passw0rd!".to_string(),
        first_name: "Syndic".to_string(),
        last_name: "Tester".to_string(),
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

// ==================== Create API Key Tests ====================

#[actix_web::test]
#[serial]
async fn test_create_api_key_success() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = create_superadmin_token(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/v1/api-keys")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "name": "PropTech Integration",
            "description": "API key for PropTech partner",
            "permissions": ["read:buildings", "read:expenses"],
            "rate_limit": 500
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201, "Should create API key successfully");

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["name"], "PropTech Integration");
    assert!(body["key"].as_str().unwrap().starts_with("kpg_live_"));
    assert_eq!(body["rate_limit"], 500);
    assert!(body["warning"].as_str().is_some());
}

#[actix_web::test]
#[serial]
async fn test_create_api_key_invalid_permission() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = create_superadmin_token(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/v1/api-keys")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "name": "Invalid Key",
            "permissions": ["invalid:permission"],
            "rate_limit": 100
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400, "Should reject invalid permission");
}

// ==================== List API Keys Tests ====================

#[actix_web::test]
#[serial]
async fn test_list_api_keys() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = create_superadmin_token(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create 2 API keys first
    for i in 1..=2 {
        let create_req = test::TestRequest::post()
            .uri("/api/v1/api-keys")
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
            .set_json(json!({
                "name": format!("Test Key {}", i),
                "permissions": ["read:buildings"],
                "rate_limit": 100
            }))
            .to_request();

        test::call_service(&app, create_req).await;
    }

    // List API keys
    let req = test::TestRequest::get()
        .uri("/api/v1/api-keys")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body["total"].as_i64().unwrap() >= 2);
    assert!(body["data"].as_array().unwrap().len() >= 2);

    // Verify key bodies are hidden (only prefix shown)
    let first_key = &body["data"][0];
    assert!(first_key["key_prefix"].as_str().is_some());
    assert!(first_key.get("key").is_none() || first_key["key"].is_null());
}

// ==================== Get API Key Tests ====================

#[actix_web::test]
#[serial]
async fn test_get_api_key_by_id() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = create_superadmin_token(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create API key
    let create_req = test::TestRequest::post()
        .uri("/api/v1/api-keys")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "name": "Fetch Me Key",
            "permissions": ["read:buildings", "read:expenses"],
            "rate_limit": 200
        }))
        .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    let created: serde_json::Value = test::read_body_json(create_resp).await;
    let key_id = created["id"].as_str().unwrap();

    // Get API key by ID
    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/api-keys/{}", key_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["id"], key_id);
    assert_eq!(body["name"], "Fetch Me Key");
    assert_eq!(body["rate_limit"], 200);
    assert!(body["is_active"].as_bool().unwrap());
}

#[actix_web::test]
#[serial]
async fn test_get_api_key_not_found() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = create_superadmin_token(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let fake_id = Uuid::new_v4();
    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/api-keys/{}", fake_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 404);
}

// ==================== Revoke API Key Tests ====================

#[actix_web::test]
#[serial]
async fn test_revoke_api_key() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = create_superadmin_token(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create API key
    let create_req = test::TestRequest::post()
        .uri("/api/v1/api-keys")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "name": "Key to Revoke",
            "permissions": ["read:buildings"],
            "rate_limit": 100
        }))
        .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    let created: serde_json::Value = test::read_body_json(create_resp).await;
    let key_id = created["id"].as_str().unwrap();

    // Revoke API key
    let revoke_req = test::TestRequest::delete()
        .uri(&format!("/api/v1/api-keys/{}", key_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let revoke_resp = test::call_service(&app, revoke_req).await;
    assert_eq!(revoke_resp.status(), 200);

    let body: serde_json::Value = test::read_body_json(revoke_resp).await;
    assert!(body["success"].as_bool().unwrap());
    assert!(body["message"].as_str().unwrap().contains("revoked"));

    // Verify key is no longer active
    let get_req = test::TestRequest::get()
        .uri(&format!("/api/v1/api-keys/{}", key_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let get_resp = test::call_service(&app, get_req).await;
    if get_resp.status() == 200 {
        let key_data: serde_json::Value = test::read_body_json(get_resp).await;
        assert!(!key_data["is_active"].as_bool().unwrap());
    }
}

// ==================== Authorization Tests ====================

#[actix_web::test]
#[serial]
async fn test_create_api_key_without_auth_fails() {
    let (app_state, _container, _org_id) = common::setup_test_db().await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/v1/api-keys")
        .set_json(json!({
            "name": "Unauthorized Key",
            "permissions": ["read:buildings"],
            "rate_limit": 100
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401, "Should require authentication");
}

#[actix_web::test]
#[serial]
async fn test_create_api_key_with_owner_role_fails() {
    let (app_state, _container, org_id) = common::setup_test_db().await;

    // Register as owner (non-syndic, non-superadmin)
    let email = format!("owner+{}@test.com", Uuid::new_v4());
    let reg = koprogo_api::application::dto::RegisterRequest {
        email: email.clone(),
        password: "Passw0rd!".to_string(),
        first_name: "Owner".to_string(),
        last_name: "Tester".to_string(),
        role: "owner".to_string(),
        organization_id: Some(org_id),
    };
    let _ = app_state
        .auth_use_cases
        .register(reg)
        .await
        .expect("register owner");
    let login_req = koprogo_api::application::dto::LoginRequest {
        email,
        password: "Passw0rd!".to_string(),
    };
    let token = app_state
        .auth_use_cases
        .login(login_req)
        .await
        .expect("login owner")
        .token;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/v1/api-keys")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "name": "Owner Key",
            "permissions": ["read:buildings"],
            "rate_limit": 100
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.status(),
        403,
        "Owners should not be able to create API keys"
    );
}

// ==================== Validation Tests ====================

#[actix_web::test]
#[serial]
async fn test_create_api_key_empty_name_fails() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = create_superadmin_token(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/v1/api-keys")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "name": "",
            "permissions": ["read:buildings"],
            "rate_limit": 100
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400, "Should reject empty API key name");
}

// ==================== Rotate API Key Tests (#339 / WP-A7) ====================
// Taxonomie 4 catégories RED-first (CRITICAL.md #3). Exécutés en CI
// (testcontainers) ; local = compile-check (daemon Docker absent, même
// limite infra que WP-B1/A4/A5, tracée).

async fn create_api_key_returning_id(
    app_state: &actix_web::web::Data<AppState>,
    token: &str,
) -> Uuid {
    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;
    let req = test::TestRequest::post()
        .uri("/api/v1/api-keys")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "name": "Rotatable Key",
            "description": "to be rotated",
            "permissions": ["read:buildings"],
            "rate_limit": 250
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201, "precondition: key created");
    let body: serde_json::Value = test::read_body_json(resp).await;
    Uuid::parse_str(body["id"].as_str().expect("id")).expect("uuid")
}

#[actix_web::test]
#[serial]
async fn happy_rotate_api_key_issues_new_secret() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = create_superadmin_token(&app_state, org_id).await;
    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let key_id = create_api_key_returning_id(&app_state, &token).await;

    let req = test::TestRequest::post()
        .uri(&format!("/api/v1/api-keys/{}/rotate", key_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200, "rotate should succeed");

    let body: serde_json::Value = test::read_body_json(resp).await;
    let new_key = body["key"].as_str().expect("new key");
    assert!(new_key.starts_with("kpg_live_"), "fresh secret issued");
    assert_eq!(body["name"], "Rotatable Key", "metadata inherited");
    assert_eq!(body["rate_limit"], 250, "rate limit inherited");
    assert_ne!(
        body["id"].as_str().unwrap(),
        key_id.to_string(),
        "rotation yields a new key id"
    );
}

#[actix_web::test]
#[serial]
async fn security_old_key_is_invalid_after_rotation() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = create_superadmin_token(&app_state, org_id).await;
    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let key_id = create_api_key_returning_id(&app_state, &token).await;

    let req = test::TestRequest::post()
        .uri(&format!("/api/v1/api-keys/{}/rotate", key_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();
    assert_eq!(test::call_service(&app, req).await.status(), 200);

    // The rotated key MUST be deactivated immediately (replay protection).
    let row = sqlx::query("SELECT is_active FROM api_keys WHERE id = $1")
        .bind(key_id)
        .fetch_one(&app_state.pool)
        .await
        .expect("old key row");
    let is_active: bool = sqlx::Row::get(&row, "is_active");
    assert!(!is_active, "old API key must be inactive post-rotation");

    // A fresh active key exists for the org (the replacement).
    let active: i64 = sqlx::Row::get(
        &sqlx::query(
            "SELECT COUNT(*) AS c FROM api_keys WHERE organization_id = $1 AND is_active = TRUE",
        )
        .bind(org_id)
        .fetch_one(&app_state.pool)
        .await
        .expect("count"),
        "c",
    );
    assert_eq!(active, 1, "exactly one active key (the replacement)");
}

#[actix_web::test]
#[serial]
async fn negative_rotate_requires_privileged_role() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let admin = create_superadmin_token(&app_state, org_id).await;

    // Create a key as admin first.
    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;
    let key_id = create_api_key_returning_id(&app_state, &admin).await;

    // Owner role (non-syndic / non-superadmin) must be refused (403).
    let email = format!("owner+{}@test.com", Uuid::new_v4());
    let reg = koprogo_api::application::dto::RegisterRequest {
        email: email.clone(),
        password: "Passw0rd!".to_string(),
        first_name: "Owner".to_string(),
        last_name: "Tester".to_string(),
        role: "owner".to_string(),
        organization_id: Some(org_id),
    };
    app_state
        .auth_use_cases
        .register(reg)
        .await
        .expect("register owner");
    let owner_token = app_state
        .auth_use_cases
        .login(koprogo_api::application::dto::LoginRequest {
            email,
            password: "Passw0rd!".to_string(),
        })
        .await
        .expect("login owner")
        .token;

    let req = test::TestRequest::post()
        .uri(&format!("/api/v1/api-keys/{}/rotate", key_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", owner_token)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.status(),
        403,
        "owners must not be able to rotate API keys"
    );
}

#[actix_web::test]
#[serial]
async fn edge_rotate_unknown_key_is_404() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = create_superadmin_token(&app_state, org_id).await;
    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::post()
        .uri(&format!("/api/v1/api-keys/{}/rotate", Uuid::new_v4()))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.status(),
        404,
        "rotating an unknown/cross-org key must be 404 (no existence leak)"
    );
}
