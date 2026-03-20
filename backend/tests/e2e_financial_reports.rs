// E2E tests for Financial Report HTTP endpoints (Belgian PCMN)
// Tests focus on HTTP layer: endpoints, auth, JSON serialization
// Belgian legal context: Balance sheet (Bilan) and Income Statement (Compte de résultats)
// Inspired by Noalyss (GPL-2.0-or-later) — see financial_report_handlers.rs

mod common;

use actix_web::http::header;
use actix_web::{test, App};
use koprogo_api::infrastructure::web::configure_routes;
use serial_test::serial;

#[actix_web::test]
#[serial]
async fn test_financial_reports_balance_sheet() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    // financial reports require accountant or superadmin — register_and_login creates superadmin
    let token = common::register_and_login(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // GET /api/v1/reports/balance-sheet — no query params required
    let req = test::TestRequest::get()
        .uri("/api/v1/reports/balance-sheet")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;

    // The endpoint should return 200 OK (even if org has no accounts yet)
    assert!(
        resp.status().is_success(),
        "Expected 200 OK for balance sheet, got: {}",
        resp.status()
    );
}

#[actix_web::test]
#[serial]
async fn test_financial_reports_income_statement() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = common::register_and_login(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // GET /api/v1/reports/income-statement with required period query params (ISO 8601)
    let req = test::TestRequest::get()
        .uri("/api/v1/reports/income-statement?period_start=2026-01-01T00:00:00Z&period_end=2026-12-31T23:59:59Z")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;

    assert!(
        resp.status().is_success(),
        "Expected 200 OK for income statement, got: {}",
        resp.status()
    );
}

#[actix_web::test]
#[serial]
async fn test_financial_reports_income_statement_invalid_dates() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = common::register_and_login(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // period_start is AFTER period_end — should return 400
    let req = test::TestRequest::get()
        .uri("/api/v1/reports/income-statement?period_start=2026-12-31T00:00:00Z&period_end=2026-01-01T00:00:00Z")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;

    assert_eq!(
        resp.status().as_u16(),
        400,
        "Expected 400 for inverted date range, got: {}",
        resp.status()
    );
}

#[actix_web::test]
#[serial]
async fn test_financial_reports_income_statement_missing_params() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = common::register_and_login(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Missing required query params — should return 400
    let req = test::TestRequest::get()
        .uri("/api/v1/reports/income-statement")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;

    assert!(
        resp.status().is_client_error(),
        "Expected 4xx when query params are missing, got: {}",
        resp.status()
    );
}

#[actix_web::test]
#[serial]
async fn test_financial_reports_balance_sheet_unauthorized() {
    let (app_state, _container, _org_id) = common::setup_test_db().await;
    // No token — access should be denied

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/v1/reports/balance-sheet")
        .to_request();

    let resp = test::call_service(&app, req).await;

    assert_eq!(
        resp.status().as_u16(),
        401,
        "Expected 401 Unauthorized without token, got: {}",
        resp.status()
    );
}

#[actix_web::test]
#[serial]
async fn test_financial_reports_income_statement_unauthorized() {
    let (app_state, _container, _org_id) = common::setup_test_db().await;
    // No token — access should be denied

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/v1/reports/income-statement?period_start=2026-01-01T00:00:00Z&period_end=2026-12-31T23:59:59Z")
        .to_request();

    let resp = test::call_service(&app, req).await;

    assert_eq!(
        resp.status().as_u16(),
        401,
        "Expected 401 Unauthorized without token, got: {}",
        resp.status()
    );
}
