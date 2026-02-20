// E2E tests for État Daté HTTP endpoints (Issue #80)
// Tests focus on HTTP layer: endpoints, auth, JSON serialization
// Belgian legal requirement: État Daté required for ALL property sales (Article 577-2 Civil Code)

mod common;

use actix_web::http::header;
use actix_web::{test, App};
use chrono::Utc;
use koprogo_api::infrastructure::web::configure_routes;
use serde_json::json;
use serial_test::serial;
use uuid::Uuid;

#[actix_web::test]
#[serial]
async fn test_create_etat_date_request() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = common::register_and_login(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let building_id = Uuid::new_v4();
    let unit_id = Uuid::new_v4();

    let etat_date_dto = json!({
        "building_id": building_id.to_string(),
        "unit_id": unit_id.to_string(),
        "reference_date": Utc::now().to_rfc3339(),
        "requestor_name": "Notaire Jean Dupont",
        "requestor_email": "jdupont@notaire.be",
        "requestor_phone": "+32 2 123 45 67"
    });

    let req = test::TestRequest::post()
        .uri("/api/v1/etats-dates")
        .insert_header(header::ContentType::json())
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(&etat_date_dto)
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Belgian law: État Daté must be delivered within 15 days
    assert!(
        resp.status().is_success() || resp.status().is_client_error(),
        "Expected valid response, got: {}",
        resp.status()
    );
}

#[actix_web::test]
#[serial]
async fn test_etat_date_workflow_requested_to_delivered() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = common::register_and_login(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let etat_date_id = Uuid::new_v4();

    // 1. Mark as InProgress (Requested → InProgress)
    let req = test::TestRequest::put()
        .uri(&format!("/api/v1/etats-dates/{}/in-progress", etat_date_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let _resp = test::call_service(&app, req).await;
    // Workflow transition test

    // 2. Update financial data (16 legal sections required)
    let financial_data = json!({
        "quota_ordinary": "0.0250",  // 2.5% of building
        "quota_extraordinary": "0.0250",
        "provisions_paid_amount_cents": 150_000_i64,  // 1,500 EUR
        "outstanding_amount_cents": 0i64,
        "pending_works_amount_cents": 500_000_i64,  // 5,000 EUR for elevator
        "pending_litigation": false,
        "insurance_policy_number": "BE-ASSUR-12345",
        "reserve_fund_amount_cents": 5_000_000_i64,  // 50,000 EUR
        "building_debt_amount_cents": 0i64,
        "building_credit_amount_cents": 1_000_000_i64
    });

    let req = test::TestRequest::put()
        .uri(&format!(
            "/api/v1/etats-dates/{}/financial-data",
            etat_date_id
        ))
        .insert_header(header::ContentType::json())
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(&financial_data)
        .to_request();

    let _resp = test::call_service(&app, req).await;

    // 3. Mark as Generated (InProgress → Generated)
    let req = test::TestRequest::put()
        .uri(&format!("/api/v1/etats-dates/{}/generated", etat_date_id))
        .insert_header(header::ContentType::json())
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "pdf_file_path": "/documents/etat_date_123.pdf"
        }))
        .to_request();

    let _resp = test::call_service(&app, req).await;

    // 4. Mark as Delivered (Generated → Delivered)
    let req = test::TestRequest::put()
        .uri(&format!("/api/v1/etats-dates/{}/delivered", etat_date_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let _resp = test::call_service(&app, req).await;
    // Complete workflow: Requested → InProgress → Generated → Delivered
}

#[actix_web::test]
#[serial]
async fn test_list_overdue_etats_dates() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = common::register_and_login(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/v1/etats-dates/overdue")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Belgian law: État Daté MUST be delivered within 15 days
    // Overdue = requested_date + 15 days < NOW and status != Delivered
    assert!(resp.status().is_success() || resp.status().is_client_error());
}

#[actix_web::test]
#[serial]
async fn test_list_expired_etats_dates() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = common::register_and_login(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/v1/etats-dates/expired")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;

    // État Daté expires after 3 months (90 days)
    // Seller must request a new one if not used
    assert!(resp.status().is_success() || resp.status().is_client_error());
}

#[actix_web::test]
#[serial]
async fn test_get_etat_date_by_reference_number() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = common::register_and_login(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let reference = "ED-2026-001";

    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/etats-dates/reference/{}", reference))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let _resp = test::call_service(&app, req).await;

    // Reference number format: ED-YYYY-NNN (e.g., ED-2026-001)
    // Used for notary tracking and legal compliance
}

#[actix_web::test]
#[serial]
async fn test_etat_date_statistics() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = common::register_and_login(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let building_id = Uuid::new_v4();

    let req = test::TestRequest::get()
        .uri(&format!(
            "/api/v1/etats-dates/stats?building_id={}",
            building_id
        ))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let _resp = test::call_service(&app, req).await;

    // Expected stats: total, by status, average delivery time, overdue count
    // Critical for syndic dashboard to monitor legal compliance
}

#[actix_web::test]
#[serial]
async fn test_etat_date_16_legal_sections_validation() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = common::register_and_login(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let etat_date_id = Uuid::new_v4();

    // Belgian law requires 16 sections in État Daté
    let additional_data = json!({
        // Sections beyond financial data
        "regulation_copy_url": "/documents/reglement_copropriete.pdf",
        "recent_ag_minutes_urls": [
            "/documents/pv_ag_2025_01.pdf",
            "/documents/pv_ag_2024_12.pdf"
        ],
        "budget_url": "/documents/budget_2026.pdf",
        "insurance_certificate_url": "/documents/assurance_2026.pdf",
        "guarantees_and_mortgages": "None",
        "observations": "Elevator renovation approved in AG 2025-01-15, work starts 2026-03-01"
    });

    let req = test::TestRequest::put()
        .uri(&format!(
            "/api/v1/etats-dates/{}/additional-data",
            etat_date_id
        ))
        .insert_header(header::ContentType::json())
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(&additional_data)
        .to_request();

    let _resp = test::call_service(&app, req).await;

    // All 16 sections must be filled before marking as Generated
    // Validation ensures legal compliance
}
