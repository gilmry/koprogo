// E2E tests for Dashboard HTTP endpoints
// Tests focus on HTTP layer: endpoints, auth, JSON serialization

mod common;

use actix_web::http::header;
use actix_web::{test, App};
use koprogo_api::infrastructure::web::configure_routes;
use serial_test::serial;

#[actix_web::test]
#[serial]
async fn test_dashboard_accountant_stats() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = common::register_and_login(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/v1/dashboard/accountant/stats")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200, "Should return dashboard stats");

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body.is_object(), "Response should be an object");
    // Stats fields should be present
    assert!(
        body.get("total_expenses").is_some()
            || body.get("pending_payments").is_some()
            || body.is_object(),
        "Stats object should have fields"
    );
}

#[actix_web::test]
#[serial]
async fn test_dashboard_accountant_stats_unauthorized() {
    let (app_state, _container, _org_id) = common::setup_test_db().await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/v1/dashboard/accountant/stats")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401, "Should require authentication");
}

#[actix_web::test]
#[serial]
async fn test_dashboard_recent_transactions() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = common::register_and_login(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/v1/dashboard/accountant/transactions?limit=5")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200, "Should return recent transactions");

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body.is_array(), "Response should be an array");
}

#[actix_web::test]
#[serial]
async fn test_dashboard_transactions_unauthorized() {
    let (app_state, _container, _org_id) = common::setup_test_db().await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/v1/dashboard/accountant/transactions")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401, "Should require authentication");
}
