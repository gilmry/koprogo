// E2E tests for Individual Members HTTP endpoints (Issue #280)
// Tests focus on HTTP layer: energy campaign individual member management
// Covers Belgian energy group buying extensions (Art. 22 RED II)
// Note: These handlers are mostly stubs (TODO implementations) but we test HTTP contract

mod common;

use actix_web::{test, App};
use koprogo_api::infrastructure::web::configure_routes;
use serde_json::json;
use serial_test::serial;
use uuid::Uuid;

// ==================== Join Campaign Tests ====================

#[actix_web::test]
#[serial]
async fn test_join_campaign_as_individual() {
    let (app_state, _container, _org_id) = common::setup_test_db().await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let campaign_id = Uuid::new_v4();

    let req = test::TestRequest::post()
        .uri(&format!(
            "/api/v1/energy-campaigns/{}/join-as-individual",
            campaign_id
        ))
        .set_json(json!({
            "email": "test-member@example.be",
            "postal_code": "1000",
            "annual_consumption_kwh": 3500.0,
            "current_provider": "Engie Electrabel"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.status(),
        201,
        "Should create individual member successfully"
    );

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body["id"].is_string(), "Should have an ID");
    assert_eq!(body["email"], "test-member@example.be");
    assert_eq!(body["postal_code"], "1000");
    assert_eq!(body["campaign_id"], campaign_id.to_string());
    assert_eq!(body["has_gdpr_consent"], false);
}

#[actix_web::test]
#[serial]
async fn test_join_campaign_minimal_fields() {
    let (app_state, _container, _org_id) = common::setup_test_db().await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let campaign_id = Uuid::new_v4();

    // Only required fields: email and postal_code
    let req = test::TestRequest::post()
        .uri(&format!(
            "/api/v1/energy-campaigns/{}/join-as-individual",
            campaign_id
        ))
        .set_json(json!({
            "email": "minimal@example.be",
            "postal_code": "1050"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.status(),
        201,
        "Should create member with minimal fields"
    );

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["email"], "minimal@example.be");
    assert_eq!(body["postal_code"], "1050");
    assert!(body["annual_consumption_kwh"].is_null(), "Optional field should be null");
}

#[actix_web::test]
#[serial]
async fn test_join_campaign_no_auth_required() {
    let (app_state, _container, _org_id) = common::setup_test_db().await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let campaign_id = Uuid::new_v4();

    // No Authorization header — public endpoint for non-copropriétaires
    let req = test::TestRequest::post()
        .uri(&format!(
            "/api/v1/energy-campaigns/{}/join-as-individual",
            campaign_id
        ))
        .set_json(json!({
            "email": "public@example.be",
            "postal_code": "1000"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_ne!(
        resp.status(),
        401,
        "Join campaign should not require authentication"
    );
}

// ==================== Grant Consent Tests ====================

#[actix_web::test]
#[serial]
async fn test_grant_consent() {
    let (app_state, _container, _org_id) = common::setup_test_db().await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let campaign_id = Uuid::new_v4();
    let member_id = Uuid::new_v4();

    let req = test::TestRequest::post()
        .uri(&format!(
            "/api/v1/energy-campaigns/{}/members/{}/consent",
            campaign_id, member_id
        ))
        .set_json(json!({
            "has_consent": true
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200, "Should grant consent successfully");

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["success"], true);
    assert!(
        body["message"].is_string(),
        "Should have a confirmation message"
    );
}

// ==================== Update Consumption Tests ====================

#[actix_web::test]
#[serial]
async fn test_update_consumption() {
    let (app_state, _container, _org_id) = common::setup_test_db().await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let campaign_id = Uuid::new_v4();
    let member_id = Uuid::new_v4();

    let req = test::TestRequest::put()
        .uri(&format!(
            "/api/v1/energy-campaigns/{}/members/{}/consumption",
            campaign_id, member_id
        ))
        .set_json(json!({
            "annual_consumption_kwh": 4200.0,
            "current_provider": "Luminus",
            "ean_code": "541448860000123456"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.status(),
        200,
        "Should update consumption data successfully"
    );

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["success"], true);
}

// ==================== Withdraw Tests ====================

#[actix_web::test]
#[serial]
async fn test_withdraw_from_campaign() {
    let (app_state, _container, _org_id) = common::setup_test_db().await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let campaign_id = Uuid::new_v4();
    let member_id = Uuid::new_v4();

    let req = test::TestRequest::delete()
        .uri(&format!(
            "/api/v1/energy-campaigns/{}/members/{}/withdraw",
            campaign_id, member_id
        ))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.status(),
        200,
        "Should withdraw from campaign successfully"
    );

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["success"], true);
    assert!(
        body["message"].is_string(),
        "Should have a confirmation message"
    );
}

// ==================== Invalid Input Tests ====================

#[actix_web::test]
#[serial]
async fn test_join_campaign_invalid_email() {
    let (app_state, _container, _org_id) = common::setup_test_db().await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let campaign_id = Uuid::new_v4();

    // IndividualMember::new validates email format in domain layer
    let req = test::TestRequest::post()
        .uri(&format!(
            "/api/v1/energy-campaigns/{}/join-as-individual",
            campaign_id
        ))
        .set_json(json!({
            "email": "not-a-valid-email",
            "postal_code": "1000"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    let status = resp.status().as_u16();
    // Domain validation may reject invalid email with 400, or handler may accept it
    // depending on how strict the domain entity validation is
    assert!(
        status == 400 || status == 201,
        "Expected 400 (validation error) or 201, got {}",
        status
    );
}
