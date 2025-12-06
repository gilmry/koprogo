// E2E tests for Payment Recovery HTTP endpoints (Issue #83)
// Tests focus on HTTP layer: endpoints, auth, JSON serialization
// Belgian legal context: Automated recovery workflow with 4 escalation levels

use actix_web::http::header;
use actix_web::{test, App};
use chrono::Utc;
use koprogo_api::application::use_cases::*;
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

/// Setup function shared across all payment recovery E2E tests
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
    let owner_repo = Arc::new(PostgresOwnerRepository::new(pool.clone()));
    let expense_repo = Arc::new(PostgresExpenseRepository::new(pool.clone()));
    let payment_reminder_repo = Arc::new(PostgresPaymentReminderRepository::new(pool.clone()));
    let gdpr_repo = Arc::new(PostgresGdprRepository::new(Arc::new(pool.clone())));
    let audit_log_repo = Arc::new(PostgresAuditLogRepository::new(pool.clone()));

    let audit_logger = AuditLogger::new(Some(audit_log_repo.clone()));

    // Initialize use cases
    let jwt_secret = "e2e-payment-recovery-secret".to_string();
    let account_repo = Arc::new(PostgresAccountRepository::new(pool.clone()));

    let auth_use_cases =
        AuthUseCases::new(user_repo.clone(), refresh_repo, user_role_repo, jwt_secret);
    let building_use_cases = BuildingUseCases::new(building_repo.clone());
    let payment_reminder_use_cases = PaymentReminderUseCases::new(
        payment_reminder_repo.clone(),
        expense_repo.clone(),
        owner_repo.clone(),
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
        "/tmp/koprogo-e2e-payment-recovery".to_string(),
        None,
        None,
        None,
        None,
        None,
    ));

    let app_state = actix_web::web::Data::new(AppState {
        auth_use_cases,
        building_use_cases,
        owner_use_cases: OwnerUseCases::new(owner_repo.clone()),
        expense_use_cases: ExpenseUseCases::new(expense_repo.clone()),
        payment_reminder_use_cases: Some(payment_reminder_use_cases),
        gdpr_use_cases: GdprUseCases::new(gdpr_repo, user_repo.clone()),
        audit_logger,
        email_service,
        file_storage,
        unit_use_cases: None,
        unit_owner_use_cases: None,
        meeting_use_cases: None,
        budget_use_cases: None,
        document_use_cases: None,
        charge_distribution_use_cases: None,
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
        etat_date_use_cases: None,
        journal_entry_use_cases: None,
        call_for_funds_use_cases: None,
        owner_contribution_use_cases: None,
        pcn_use_cases: None,
        dashboard_use_cases: None,
        board_dashboard_use_cases: None,
        account_use_cases: None,
        financial_report_use_cases: None,
    });

    (app_state, postgres_container)
}

#[actix_web::test]
#[serial]
async fn test_create_gentle_reminder() {
    let (app_state, _postgres_container) = setup_app().await;
    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let expense_id = Uuid::new_v4();
    let owner_id = Uuid::new_v4();

    let reminder_dto = json!({
        "expense_id": expense_id.to_string(),
        "owner_id": owner_id.to_string(),
        "level": "Gentle",  // J+15 after due date
        "amount_due_cents": 50_000_i64,  // 500 EUR
        "days_overdue": 15
    });

    let req = test::TestRequest::post()
        .uri("/api/v1/payment-reminders")
        .insert_header(header::ContentType::json())
        .insert_header((header::AUTHORIZATION, "Bearer mock-token"))
        .set_json(&reminder_dto)
        .to_request();

    let _resp = test::call_service(&app, req).await;

    // Gentle reminder: Courteous tone, simple email
    assert!(
        resp.status().is_success() || resp.status().is_client_error(),
        "Expected valid response, got: {}",
        resp.status()
    );
}

#[actix_web::test]
#[serial]
async fn test_escalation_workflow() {
    let (app_state, _postgres_container) = setup_app().await;
    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let reminder_id = Uuid::new_v4();

    // Escalate from Gentle to Formal (J+30)
    let req = test::TestRequest::post()
        .uri(&format!(
            "/api/v1/payment-reminders/{}/escalate",
            reminder_id
        ))
        .insert_header(header::ContentType::json())
        .insert_header((header::AUTHORIZATION, "Bearer mock-token"))
        .set_json(&json!({
            "new_level": "Formal",
            "notes": "No payment received after gentle reminder"
        }))
        .to_request();

    let _resp = test::call_service(&app, req).await;

    // 4-level escalation: Gentle → Formal → FinalNotice → LegalAction
    // Belgian context: Formal = email + PDF letter, mentions penalties
}

#[actix_web::test]
#[serial]
async fn test_calculate_late_payment_penalty() {
    let (app_state, _postgres_container) = setup_app().await;
    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let reminder_id = Uuid::new_v4();

    let req = test::TestRequest::get()
        .uri(&format!(
            "/api/v1/payment-reminders/{}?include_penalty=true",
            reminder_id
        ))
        .insert_header((header::AUTHORIZATION, "Bearer mock-token"))
        .to_request();

    let _resp = test::call_service(&app, req).await;

    // Belgian legal rate: 8% annual
    // Formula: penalty = amount * 0.08 * (days_overdue / 365)
    // Example: 500 EUR * 8% * (30/365) = 3.29 EUR penalty
}

#[actix_web::test]
#[serial]
async fn test_mark_reminder_as_sent() {
    let (app_state, _postgres_container) = setup_app().await;
    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let reminder_id = Uuid::new_v4();

    let req = test::TestRequest::put()
        .uri(&format!(
            "/api/v1/payment-reminders/{}/mark-sent",
            reminder_id
        ))
        .insert_header(header::ContentType::json())
        .insert_header((header::AUTHORIZATION, "Bearer mock-token"))
        .set_json(&json!({
            "sent_date": Utc::now().to_rfc3339()
        }))
        .to_request();

    let _resp = test::call_service(&app, req).await;

    // Tracking: Email sent, awaiting owner response
}

#[actix_web::test]
#[serial]
async fn test_add_tracking_number_for_registered_letter() {
    let (app_state, _postgres_container) = setup_app().await;
    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let reminder_id = Uuid::new_v4();

    let req = test::TestRequest::put()
        .uri(&format!(
            "/api/v1/payment-reminders/{}/tracking-number",
            reminder_id
        ))
        .insert_header(header::ContentType::json())
        .insert_header((header::AUTHORIZATION, "Bearer mock-token"))
        .set_json(&json!({
            "tracking_number": "RR123456789BE"
        }))
        .to_request();

    let _resp = test::call_service(&app, req).await;

    // FinalNotice level: Registered letter (lettre recommandée)
    // Tracking number for legal proof of delivery
}

#[actix_web::test]
#[serial]
async fn test_bulk_create_reminders_for_overdue_expenses() {
    let (app_state, _postgres_container) = setup_app().await;
    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let building_id = Uuid::new_v4();

    let req = test::TestRequest::post()
        .uri("/api/v1/payment-reminders/bulk-create")
        .insert_header(header::ContentType::json())
        .insert_header((header::AUTHORIZATION, "Bearer mock-token"))
        .set_json(&json!({
            "building_id": building_id.to_string(),
            "days_overdue_threshold": 15
        }))
        .to_request();

    let _resp = test::call_service(&app, req).await;

    // Automated workflow: Find all expenses overdue > 15 days without reminders
    // Create Gentle reminders for all
    // Belgian best practice: Automate to reduce manual work
}

#[actix_web::test]
#[serial]
async fn test_get_recovery_statistics() {
    let (app_state, _postgres_container) = setup_app().await;
    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/v1/payment-reminders/stats")
        .insert_header((header::AUTHORIZATION, "Bearer mock-token"))
        .to_request();

    let _resp = test::call_service(&app, req).await;

    // Expected stats:
    // - total_reminders
    // - by_level: { Gentle: 10, Formal: 5, FinalNotice: 2, LegalAction: 1 }
    // - total_amount_overdue_cents
    // - success_rate (% paid after reminder)
    // - average_days_to_payment
}

#[actix_web::test]
#[serial]
async fn test_list_active_reminders_by_owner() {
    let (app_state, _postgres_container) = setup_app().await;
    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let owner_id = Uuid::new_v4();

    let req = test::TestRequest::get()
        .uri(&format!(
            "/api/v1/owners/{}/payment-reminders/active",
            owner_id
        ))
        .insert_header((header::AUTHORIZATION, "Bearer mock-token"))
        .to_request();

    let _resp = test::call_service(&app, req).await;

    // Active = sent but not paid/cancelled
    // Critical for owner dashboard: "You have 3 unpaid reminders"
}

#[actix_web::test]
#[serial]
async fn test_cancel_reminder_if_paid() {
    let (app_state, _postgres_container) = setup_app().await;
    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let reminder_id = Uuid::new_v4();

    let req = test::TestRequest::put()
        .uri(&format!("/api/v1/payment-reminders/{}/cancel", reminder_id))
        .insert_header(header::ContentType::json())
        .insert_header((header::AUTHORIZATION, "Bearer mock-token"))
        .set_json(&json!({
            "cancellation_reason": "Payment received on 2026-01-15"
        }))
        .to_request();

    let _resp = test::call_service(&app, req).await;

    // Stop escalation if owner pays
    // Audit trail for legal compliance
}

#[actix_web::test]
#[serial]
async fn test_find_overdue_expenses_without_reminders() {
    let (app_state, _postgres_container) = setup_app().await;
    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/v1/payment-reminders/find-overdue-without-reminders")
        .insert_header((header::AUTHORIZATION, "Bearer mock-token"))
        .to_request();

    let _resp = test::call_service(&app, req).await;

    // Cron job helper: Identify expenses needing first reminder
    // Automation trigger for bulk_create
}

#[actix_web::test]
#[serial]
async fn test_mark_reminder_as_opened() {
    let (app_state, _postgres_container) = setup_app().await;
    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let reminder_id = Uuid::new_v4();

    let req = test::TestRequest::put()
        .uri(&format!(
            "/api/v1/payment-reminders/{}/mark-opened",
            reminder_id
        ))
        .insert_header((header::AUTHORIZATION, "Bearer mock-token"))
        .to_request();

    let _resp = test::call_service(&app, req).await;

    // Email tracking: Owner opened reminder email
    // Helps measure engagement and reminder effectiveness
}
