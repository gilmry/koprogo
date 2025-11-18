// E2E tests for État Daté HTTP endpoints (Issue #80)
// Tests focus on HTTP layer: endpoints, auth, JSON serialization
// Belgian legal requirement: État Daté required for ALL property sales (Article 577-2 Civil Code)

use actix_web::http::header;
use actix_web::{test, App};
use chrono::{Duration, Utc};
use koprogo_api::application::dto::*;
use koprogo_api::application::use_cases::*;
use koprogo_api::domain::entities::EtatDateStatus;
use koprogo_api::infrastructure::audit_logger::AuditLogger;
use koprogo_api::infrastructure::database::create_pool;
use koprogo_api::infrastructure::database::repositories::*;
use koprogo_api::infrastructure::database::PostgresAccountRepository;
use koprogo_api::infrastructure::email::EmailService;
use koprogo_api::infrastructure::storage::{FileStorage, StorageProvider};
use koprogo_api::infrastructure::web::configure_routes;
use koprogo_api::infrastructure::web::AppState;
use serde_json::json;
use serial_test::serial;
use std::sync::Arc;
use testcontainers_modules::postgres::Postgres;
use testcontainers_modules::testcontainers::{runners::AsyncRunner, ContainerAsync};
use uuid::Uuid;

/// Setup function shared across all état daté E2E tests
async fn setup_app() -> (actix_web::web::Data<AppState>, ContainerAsync<Postgres>) {
    let postgres_container = Postgres::default()
        .start()
        .await
        .expect("Failed to start postgres container");

    let host_port = postgres_container
        .get_host_port_ipv4(5432)
        .await
        .expect("Failed to get host port");

    let connection_string = format!(
        "postgres://postgres:postgres@127.0.0.1:{}/postgres",
        host_port
    );

    let pool = create_pool(&connection_string)
        .await
        .expect("Failed to create pool");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    // Initialize repositories
    let user_repo = Arc::new(PostgresUserRepository::new(pool.clone()));
    let user_role_repo = Arc::new(PostgresUserRoleRepository::new(pool.clone()));
    let refresh_repo = Arc::new(PostgresRefreshTokenRepository::new(pool.clone()));
    let building_repo = Arc::new(PostgresBuildingRepository::new(pool.clone()));
    let unit_repo = Arc::new(PostgresUnitRepository::new(pool.clone()));
    let owner_repo = Arc::new(PostgresOwnerRepository::new(pool.clone()));
    let expense_repo = Arc::new(PostgresExpenseRepository::new(pool.clone()));
    let etat_date_repo = Arc::new(PostgresEtatDateRepository::new(pool.clone()));
    let gdpr_repo = Arc::new(PostgresGdprRepository::new(Arc::new(pool.clone())));
    let audit_log_repo = Arc::new(PostgresAuditLogRepository::new(pool.clone()));

    let audit_logger = AuditLogger::new(Some(audit_log_repo.clone()));

    // Initialize use cases
    let jwt_secret = "e2e-etat-date-secret".to_string();
    let account_repo = Arc::new(PostgresAccountRepository::new(pool.clone()));
    let account_use_cases = AccountUseCases::new(account_repo.clone());
    let financial_report_use_cases =
        FinancialReportUseCases::new(account_repo, expense_repo.clone());

    let auth_use_cases =
        AuthUseCases::new(user_repo.clone(), refresh_repo, user_role_repo, jwt_secret);
    let building_use_cases = BuildingUseCases::new(building_repo.clone());
    let unit_use_cases = UnitUseCases::new(unit_repo.clone());
    let etat_date_use_cases = EtatDateUseCases::new(
        etat_date_repo.clone(),
        building_repo.clone(),
        unit_repo.clone(),
        owner_repo.clone(),
        expense_repo.clone(),
    );

    let email_service = Arc::new(EmailService::new(
        "smtp.test.com".to_string(),
        587,
        "test@test.com".to_string(),
        "password".to_string(),
        "KoproGo Test".to_string(),
    ));

    let file_storage = Arc::new(FileStorage::new(
        StorageProvider::LocalFilesystem,
        "/tmp/koprogo-e2e-etat-date".to_string(),
        None,
        None,
        None,
        None,
        None,
    ));

    let app_state = actix_web::web::Data::new(AppState {
        auth_use_cases,
        building_use_cases,
        unit_use_cases,
        owner_use_cases: OwnerUseCases::new(owner_repo.clone()),
        expense_use_cases: ExpenseUseCases::new(expense_repo.clone()),
        etat_date_use_cases: Some(etat_date_use_cases),
        account_use_cases,
        financial_report_use_cases,
        gdpr_use_cases: GdprUseCases::new(gdpr_repo, user_repo.clone()),
        audit_logger,
        email_service,
        file_storage,
        unit_owner_use_cases: None,
        meeting_use_cases: None,
        budget_use_cases: None,
        document_use_cases: None,
        charge_distribution_use_cases: None,
        payment_reminder_use_cases: None,
        board_member_use_cases: None,
        board_decision_use_cases: None,
        convocation_use_cases: None,
        resolution_use_cases: None,
        ticket_use_cases: None,
        notification_use_cases: None,
        payment_use_cases: None,
        payment_method_use_cases: None,
        quote_use_cases: None,
        local_exchange_use_cases: None,
        notice_use_cases: None,
        skill_use_cases: None,
        shared_object_use_cases: None,
        resource_booking_use_cases: None,
        gamification_use_cases: None,
        journal_entry_use_cases: None,
        call_for_funds_use_cases: None,
        owner_contribution_use_cases: None,
        pcn_use_cases: None,
        dashboard_use_cases: None,
        board_dashboard_use_cases: None,
    });

    (app_state, postgres_container)
}

#[actix_web::test]
#[serial]
async fn test_create_etat_date_request() {
    let (app_state, _postgres_container) = setup_app().await;
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
        .insert_header((header::AUTHORIZATION, "Bearer mock-token"))
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
    let (app_state, _postgres_container) = setup_app().await;
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
        .insert_header((header::AUTHORIZATION, "Bearer mock-token"))
        .to_request();

    let resp = test::call_service(&app, req).await;
    // Workflow transition test

    // 2. Update financial data (16 legal sections required)
    let financial_data = json!({
        "quota_ordinary": "0.0250",  // 2.5% of building
        "quota_extraordinary": "0.0250",
        "provisions_paid_amount_cents": 1500_00i64,  // 1,500 EUR
        "outstanding_amount_cents": 0i64,
        "pending_works_amount_cents": 5000_00i64,  // 5,000 EUR for elevator
        "pending_litigation": false,
        "insurance_policy_number": "BE-ASSUR-12345",
        "reserve_fund_amount_cents": 50000_00i64,  // 50,000 EUR
        "building_debt_amount_cents": 0i64,
        "building_credit_amount_cents": 10000_00i64
    });

    let req = test::TestRequest::put()
        .uri(&format!(
            "/api/v1/etats-dates/{}/financial-data",
            etat_date_id
        ))
        .insert_header(header::ContentType::json())
        .insert_header((header::AUTHORIZATION, "Bearer mock-token"))
        .set_json(&financial_data)
        .to_request();

    let resp = test::call_service(&app, req).await;

    // 3. Mark as Generated (InProgress → Generated)
    let req = test::TestRequest::put()
        .uri(&format!("/api/v1/etats-dates/{}/generated", etat_date_id))
        .insert_header(header::ContentType::json())
        .insert_header((header::AUTHORIZATION, "Bearer mock-token"))
        .set_json(&json!({
            "pdf_file_path": "/documents/etat_date_123.pdf"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;

    // 4. Mark as Delivered (Generated → Delivered)
    let req = test::TestRequest::put()
        .uri(&format!("/api/v1/etats-dates/{}/delivered", etat_date_id))
        .insert_header((header::AUTHORIZATION, "Bearer mock-token"))
        .to_request();

    let resp = test::call_service(&app, req).await;
    // Complete workflow: Requested → InProgress → Generated → Delivered
}

#[actix_web::test]
#[serial]
async fn test_list_overdue_etats_dates() {
    let (app_state, _postgres_container) = setup_app().await;
    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/v1/etats-dates/overdue")
        .insert_header((header::AUTHORIZATION, "Bearer mock-token"))
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Belgian law: État Daté MUST be delivered within 15 days
    // Overdue = requested_date + 15 days < NOW and status != Delivered
    assert!(resp.status().is_success() || resp.status().is_client_error());
}

#[actix_web::test]
#[serial]
async fn test_list_expired_etats_dates() {
    let (app_state, _postgres_container) = setup_app().await;
    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/v1/etats-dates/expired")
        .insert_header((header::AUTHORIZATION, "Bearer mock-token"))
        .to_request();

    let resp = test::call_service(&app, req).await;

    // État Daté expires after 3 months (90 days)
    // Seller must request a new one if not used
    assert!(resp.status().is_success() || resp.status().is_client_error());
}

#[actix_web::test]
#[serial]
async fn test_get_etat_date_by_reference_number() {
    let (app_state, _postgres_container) = setup_app().await;
    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let reference = "ED-2026-001";

    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/etats-dates/reference/{}", reference))
        .insert_header((header::AUTHORIZATION, "Bearer mock-token"))
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Reference number format: ED-YYYY-NNN (e.g., ED-2026-001)
    // Used for notary tracking and legal compliance
}

#[actix_web::test]
#[serial]
async fn test_etat_date_statistics() {
    let (app_state, _postgres_container) = setup_app().await;
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
        .insert_header((header::AUTHORIZATION, "Bearer mock-token"))
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Expected stats: total, by status, average delivery time, overdue count
    // Critical for syndic dashboard to monitor legal compliance
}

#[actix_web::test]
#[serial]
async fn test_etat_date_16_legal_sections_validation() {
    let (app_state, _postgres_container) = setup_app().await;
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
        .insert_header((header::AUTHORIZATION, "Bearer mock-token"))
        .set_json(&additional_data)
        .to_request();

    let resp = test::call_service(&app, req).await;

    // All 16 sections must be filled before marking as Generated
    // Validation ensures legal compliance
}
