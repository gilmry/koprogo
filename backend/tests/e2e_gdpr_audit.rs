mod common;

use actix_web::{http::header, test, App};
use koprogo_api::application::dto::{LoginRequest, RegisterRequest};
use koprogo_api::infrastructure::web::configure_routes;
use serial_test::serial;
use uuid::Uuid;

#[actix_web::test]
#[serial]
async fn test_gdpr_export_creates_audit_log() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let pool = app_state.pool.clone();

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Register + login to obtain JWT
    let email = format!("gdpr+{}@test.com", Uuid::new_v4());
    let reg = RegisterRequest {
        email: email.clone(),
        password: "Passw0rd!".to_string(),
        first_name: "GDPR".to_string(),
        last_name: "Test".to_string(),
        role: "syndic".to_string(),
        organization_id: Some(org_id),
    };
    let _ = app_state
        .auth_use_cases
        .register(reg)
        .await
        .expect("register");
    let login = LoginRequest {
        email: email.clone(),
        password: "Passw0rd!".to_string(),
    };
    let token = app_state
        .auth_use_cases
        .login(login)
        .await
        .expect("login")
        .token;

    // Call GDPR export endpoint
    let req = test::TestRequest::get()
        .uri("/api/v1/gdpr/export")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    // Wait a bit for async audit log to be written
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    // Check that an audit log was created in the database
    let count: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM audit_logs WHERE event_type = 'GdprDataExported'")
            .fetch_one(&pool)
            .await
            .expect("count audit logs");

    assert_eq!(count.0, 1, "Expected 1 audit log for GdprDataExported");

    // Verify the audit log has correct data
    let log: (String, bool, Option<String>) = sqlx::query_as(
        "SELECT event_type, success, metadata::text FROM audit_logs WHERE event_type = 'GdprDataExported' LIMIT 1"
    )
    .fetch_one(&pool)
    .await
    .expect("fetch audit log");

    assert_eq!(log.0, "GdprDataExported");
    assert!(log.1, "Expected success to be true");
    assert!(log.2.is_some(), "Metadata should be present");

    // Check retention_until is set (7 years in the future)
    let retention: (chrono::DateTime<chrono::Utc>,) = sqlx::query_as(
        "SELECT retention_until FROM audit_logs WHERE event_type = 'GdprDataExported' LIMIT 1",
    )
    .fetch_one(&pool)
    .await
    .expect("fetch retention_until");

    let now = chrono::Utc::now();
    let min_retention = now + chrono::Duration::days(365 * 6 + 360); // ~6.99 years
    let max_retention = now + chrono::Duration::days(365 * 7 + 5); // ~7.01 years

    assert!(
        retention.0 > min_retention && retention.0 < max_retention,
        "Retention should be approximately 7 years in the future. Got: {}, Expected between {} and {}",
        retention.0, min_retention, max_retention
    );
}

#[actix_web::test]
#[serial]
async fn test_gdpr_can_erase_creates_audit_log() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let pool = app_state.pool.clone();

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Register + login
    let email = format!("gdpr+{}@test.com", Uuid::new_v4());
    let reg = RegisterRequest {
        email: email.clone(),
        password: "Passw0rd!".to_string(),
        first_name: "GDPR".to_string(),
        last_name: "Test".to_string(),
        role: "syndic".to_string(),
        organization_id: Some(org_id),
    };
    let _ = app_state
        .auth_use_cases
        .register(reg)
        .await
        .expect("register");
    let login = LoginRequest {
        email: email.clone(),
        password: "Passw0rd!".to_string(),
    };
    let token = app_state
        .auth_use_cases
        .login(login)
        .await
        .expect("login")
        .token;

    // Call can-erase endpoint
    let req = test::TestRequest::get()
        .uri("/api/v1/gdpr/can-erase")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    // Wait for async audit log
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    // Check audit log
    let count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM audit_logs WHERE event_type = 'GdprErasureCheckRequested'",
    )
    .fetch_one(&pool)
    .await
    .expect("count audit logs");

    assert_eq!(
        count.0, 1,
        "Expected 1 audit log for GdprErasureCheckRequested"
    );
}
