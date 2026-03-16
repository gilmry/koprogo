// E2E tests for Account Management HTTP endpoints (Belgian PCMN - Issue #79)
// Tests focus on HTTP layer: endpoints, auth, JSON serialization
// Covers Belgian Plan Comptable Minimum Normalisé (AR 12/07/2012)

mod common;

use actix_web::http::header;
use actix_web::{test, App};
use koprogo_api::infrastructure::web::configure_routes;
use serde_json::json;
use serial_test::serial;
use uuid::Uuid;

// ==================== Account CRUD Tests ====================

#[actix_web::test]
#[serial]
async fn test_accounts_seed_belgian_pcmn() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = common::register_and_login(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/v1/accounts/seed/belgian-pcmn")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "organization_id": org_id.to_string()
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.status(),
        200,
        "Should seed Belgian PCMN accounts successfully"
    );

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(
        body["accounts_created"].as_i64().unwrap_or(0) > 0,
        "Should have created at least one account"
    );
}

#[actix_web::test]
#[serial]
async fn test_accounts_list_after_seed() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = common::register_and_login(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Seed first
    let seed_req = test::TestRequest::post()
        .uri("/api/v1/accounts/seed/belgian-pcmn")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "organization_id": org_id.to_string()
        }))
        .to_request();
    let _ = test::call_service(&app, seed_req).await;

    // List accounts
    let req = test::TestRequest::get()
        .uri("/api/v1/accounts")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200, "Should list accounts successfully");

    let accounts: serde_json::Value = test::read_body_json(resp).await;
    let count = accounts.as_array().unwrap_or(&vec![]).len();
    assert!(
        count >= 80,
        "Should have at least 80 Belgian PCMN accounts, got {}",
        count
    );
}

#[actix_web::test]
#[serial]
async fn test_accounts_get_by_code() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = common::register_and_login(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Seed PCMN first
    let seed_req = test::TestRequest::post()
        .uri("/api/v1/accounts/seed/belgian-pcmn")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "organization_id": org_id.to_string()
        }))
        .to_request();
    let _ = test::call_service(&app, seed_req).await;

    // Get account by code 604001 (Électricité - Expense account in Belgian PCMN)
    let req = test::TestRequest::get()
        .uri("/api/v1/accounts/code/604001")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.status(),
        200,
        "Should get account by code 604001 successfully"
    );

    let account: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(account["code"], "604001", "Account code should be 604001");
    assert_eq!(
        account["account_type"], "EXPENSE",
        "Account type should be EXPENSE (Charge)"
    );
}

#[actix_web::test]
#[serial]
async fn test_accounts_create_custom() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = common::register_and_login(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/v1/accounts")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "organization_id": org_id.to_string(),
            "code": "99001",
            "label": "Custom Test Account",
            "account_type": "EXPENSE",
            "direct_use": true
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.status(),
        201,
        "Should create custom account successfully"
    );

    let account: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(account["code"], "99001", "Account code should be 99001");
    assert_eq!(account["label"], "Custom Test Account");
    assert_eq!(account["account_type"], "EXPENSE");
}

#[actix_web::test]
#[serial]
async fn test_accounts_create_duplicate_code_fails() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = common::register_and_login(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let unique_code = format!("9{}", &Uuid::new_v4().to_string()[..4].replace('-', "0"));

    let first_req = test::TestRequest::post()
        .uri("/api/v1/accounts")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "organization_id": org_id.to_string(),
            "code": unique_code,
            "label": "First Account",
            "account_type": "EXPENSE",
            "direct_use": true
        }))
        .to_request();

    let first_resp = test::call_service(&app, first_req).await;
    assert_eq!(first_resp.status(), 201, "First creation should succeed");

    // Try to create account with same code again
    let dup_req = test::TestRequest::post()
        .uri("/api/v1/accounts")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "organization_id": org_id.to_string(),
            "code": unique_code,
            "label": "Duplicate Account",
            "account_type": "EXPENSE",
            "direct_use": true
        }))
        .to_request();

    let dup_resp = test::call_service(&app, dup_req).await;
    let status = dup_resp.status().as_u16();
    assert!(
        status == 409 || status == 400,
        "Duplicate code should return 409 or 400, got {}",
        status
    );
}

#[actix_web::test]
#[serial]
async fn test_accounts_update() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = common::register_and_login(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create an account first
    let create_req = test::TestRequest::post()
        .uri("/api/v1/accounts")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "organization_id": org_id.to_string(),
            "code": "98001",
            "label": "Original Label",
            "account_type": "EXPENSE",
            "direct_use": false
        }))
        .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    assert_eq!(create_resp.status(), 201);
    let created: serde_json::Value = test::read_body_json(create_resp).await;
    let account_id = created["id"].as_str().unwrap();

    // Update the account
    let update_req = test::TestRequest::put()
        .uri(&format!("/api/v1/accounts/{}", account_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "label": "Updated Label",
            "direct_use": true
        }))
        .to_request();

    let update_resp = test::call_service(&app, update_req).await;
    assert_eq!(
        update_resp.status(),
        200,
        "Should update account successfully"
    );

    let updated: serde_json::Value = test::read_body_json(update_resp).await;
    assert_eq!(updated["label"], "Updated Label");
    assert_eq!(updated["direct_use"], true);
}

#[actix_web::test]
#[serial]
async fn test_accounts_delete() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = common::register_and_login(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create an account to delete
    let create_req = test::TestRequest::post()
        .uri("/api/v1/accounts")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "organization_id": org_id.to_string(),
            "code": "97001",
            "label": "Account To Delete",
            "account_type": "EXPENSE",
            "direct_use": true
        }))
        .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    assert_eq!(create_resp.status(), 201);
    let created: serde_json::Value = test::read_body_json(create_resp).await;
    let account_id = created["id"].as_str().unwrap();

    // Delete the account
    let delete_req = test::TestRequest::delete()
        .uri(&format!("/api/v1/accounts/{}", account_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let delete_resp = test::call_service(&app, delete_req).await;
    let status = delete_resp.status().as_u16();
    assert!(
        status == 204 || status == 200,
        "Should delete account with 204 or 200, got {}",
        status
    );
}

#[actix_web::test]
#[serial]
async fn test_accounts_unauthorized() {
    let (app_state, _container, _org_id) = common::setup_test_db().await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Request without token
    let req = test::TestRequest::get()
        .uri("/api/v1/accounts")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.status(),
        401,
        "Should return 401 when no auth token provided"
    );
}
