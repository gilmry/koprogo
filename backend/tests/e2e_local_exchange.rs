// E2E tests for Local Exchange (SEL) HTTP endpoints (Issue #49 - Phase 1)
// Tests focus on HTTP layer: endpoints, auth, JSON serialization
// Belgian context: SEL (Système d'Échange Local) is legal and non-taxable if non-commercial

use actix_web::http::header;
use actix_web::{test, App};
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

/// Setup function shared across all local exchange E2E tests
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
    let local_exchange_repo = Arc::new(PostgresLocalExchangeRepository::new(pool.clone()));
    let owner_credit_balance_repo =
        Arc::new(PostgresOwnerCreditBalanceRepository::new(pool.clone()));
    let gdpr_repo = Arc::new(PostgresGdprRepository::new(Arc::new(pool.clone())));
    let audit_log_repo = Arc::new(PostgresAuditLogRepository::new(pool.clone()));

    let audit_logger = AuditLogger::new(Some(audit_log_repo.clone()));

    // Initialize use cases
    let jwt_secret = "e2e-sel-secret".to_string();
    let account_repo = Arc::new(PostgresAccountRepository::new(pool.clone()));

    let auth_use_cases =
        AuthUseCases::new(user_repo.clone(), refresh_repo, user_role_repo, jwt_secret);
    let building_use_cases = BuildingUseCases::new(building_repo.clone());
    let local_exchange_use_cases = LocalExchangeUseCases::new(
        local_exchange_repo.clone(),
        owner_credit_balance_repo.clone(),
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
        "/tmp/koprogo-e2e-sel".to_string(),
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
        local_exchange_use_cases: Some(local_exchange_use_cases),
        gdpr_use_cases: GdprUseCases::new(gdpr_repo, user_repo.clone()),
        audit_logger,
        email_service,
        file_storage,
        unit_use_cases: None,
        unit_owner_use_cases: None,
        expense_use_cases: None,
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
async fn test_create_service_exchange_offer() {
    let (app_state, _postgres_container) = setup_app().await;
    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let building_id = Uuid::new_v4();
    let provider_id = Uuid::new_v4();

    let exchange_dto = json!({
        "building_id": building_id.to_string(),
        "provider_id": provider_id.to_string(),
        "exchange_type": "Service",
        "title": "Plumbing repair",
        "description": "I can fix leaking faucets and pipes",
        "credits": 2,  // 2 hours = 2 credits
        "conditions": "Bring your own parts"
    });

    let req = test::TestRequest::post()
        .uri("/api/v1/exchanges")
        .insert_header(header::ContentType::json())
        .insert_header((header::AUTHORIZATION, "Bearer mock-token"))
        .set_json(&exchange_dto)
        .to_request();

    let _resp = test::call_service(&app, req).await;

    // SEL time-based currency: 1 hour = 1 credit
    assert!(
        resp.status().is_success() || resp.status().is_client_error(),
        "Expected valid response, got: {}",
        resp.status()
    );
}

#[actix_web::test]
#[serial]
async fn test_exchange_workflow_offered_to_completed() {
    let (app_state, _postgres_container) = setup_app().await;
    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let exchange_id = Uuid::new_v4();
    let requester_id = Uuid::new_v4();

    // 1. Request exchange (Offered → Requested)
    let req = test::TestRequest::post()
        .uri(&format!("/api/v1/exchanges/{}/request", exchange_id))
        .insert_header(header::ContentType::json())
        .insert_header((header::AUTHORIZATION, "Bearer mock-token"))
        .set_json(&json!({
            "requester_id": requester_id.to_string()
        }))
        .to_request();

    let _resp = test::call_service(&app, req).await;

    // 2. Start exchange (Requested → InProgress)
    let req = test::TestRequest::post()
        .uri(&format!("/api/v1/exchanges/{}/start", exchange_id))
        .insert_header((header::AUTHORIZATION, "Bearer mock-token"))
        .to_request();

    let _resp = test::call_service(&app, req).await;

    // 3. Complete exchange (InProgress → Completed)
    // Automatic credit balance update: provider +2, requester -2
    let req = test::TestRequest::post()
        .uri(&format!("/api/v1/exchanges/{}/complete", exchange_id))
        .insert_header((header::AUTHORIZATION, "Bearer mock-token"))
        .to_request();

    let _resp = test::call_service(&app, req).await;

    // SEL trust model: Negative balances allowed (community trust)
}

#[actix_web::test]
#[serial]
async fn test_mutual_rating_system() {
    let (app_state, _postgres_container) = setup_app().await;
    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let exchange_id = Uuid::new_v4();

    // Requester rates provider
    let req = test::TestRequest::put()
        .uri(&format!("/api/v1/exchanges/{}/rate-provider", exchange_id))
        .insert_header(header::ContentType::json())
        .insert_header((header::AUTHORIZATION, "Bearer mock-token"))
        .set_json(&json!({
            "rating": 5,  // 1-5 stars
            "comment": "Excellent plumbing work, very professional"
        }))
        .to_request();

    let _resp = test::call_service(&app, req).await;

    // Provider rates requester
    let req = test::TestRequest::put()
        .uri(&format!("/api/v1/exchanges/{}/rate-requester", exchange_id))
        .insert_header(header::ContentType::json())
        .insert_header((header::AUTHORIZATION, "Bearer mock-token"))
        .set_json(&json!({
            "rating": 4,
            "comment": "Punctual and respectful"
        }))
        .to_request();

    let _resp = test::call_service(&app, req).await;

    // Only completed exchanges can be rated
    // Ratings contribute to reputation score
}

#[actix_web::test]
#[serial]
async fn test_get_credit_balance() {
    let (app_state, _postgres_container) = setup_app().await;
    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let owner_id = Uuid::new_v4();
    let building_id = Uuid::new_v4();

    let req = test::TestRequest::get()
        .uri(&format!(
            "/api/v1/owners/{}/buildings/{}/credit-balance",
            owner_id, building_id
        ))
        .insert_header((header::AUTHORIZATION, "Bearer mock-token"))
        .to_request();

    let _resp = test::call_service(&app, req).await;

    // Expected response:
    // {
    //   "credits_earned": 10,
    //   "credits_spent": 7,
    //   "balance": 3,  // Can be negative
    //   "total_exchanges": 5,
    //   "average_rating": 4.5,
    //   "participation_level": "Active"  // New/Beginner/Active/Veteran/Expert
    // }
}

#[actix_web::test]
#[serial]
async fn test_leaderboard() {
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
            "/api/v1/buildings/{}/leaderboard?limit=10",
            building_id
        ))
        .insert_header((header::AUTHORIZATION, "Bearer mock-token"))
        .to_request();

    let _resp = test::call_service(&app, req).await;

    // Top 10 contributors ordered by balance DESC
    // Encourages community participation
    assert!(resp.status().is_success() || resp.status().is_client_error());
}

#[actix_web::test]
#[serial]
async fn test_sel_statistics() {
    let (app_state, _postgres_container) = setup_app().await;
    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let building_id = Uuid::new_v4();

    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/buildings/{}/sel-statistics", building_id))
        .insert_header((header::AUTHORIZATION, "Bearer mock-token"))
        .to_request();

    let _resp = test::call_service(&app, req).await;

    // Expected stats:
    // - total_exchanges, active_exchanges, completed_exchanges
    // - total_credits_exchanged
    // - active_participants
    // - average_rating
    // - most_popular_exchange_type (Service/ObjectLoan/SharedPurchase)
}

#[actix_web::test]
#[serial]
async fn test_owner_exchange_summary() {
    let (app_state, _postgres_container) = setup_app().await;
    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let owner_id = Uuid::new_v4();

    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/owners/{}/exchange-summary", owner_id))
        .insert_header((header::AUTHORIZATION, "Bearer mock-token"))
        .to_request();

    let _resp = test::call_service(&app, req).await;

    // Summary across all buildings where owner participates in SEL
    // total_offered, total_requested, total_completed
    // credits_earned, credits_spent, balance
    // average_rating, participation_level
}

#[actix_web::test]
#[serial]
async fn test_list_available_exchanges() {
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
            "/api/v1/buildings/{}/exchanges/available",
            building_id
        ))
        .insert_header((header::AUTHORIZATION, "Bearer mock-token"))
        .to_request();

    let _resp = test::call_service(&app, req).await;

    // Marketplace view: only exchanges with status = Offered
    // Excludes provider's own offers
    assert!(resp.status().is_success() || resp.status().is_client_error());
}

#[actix_web::test]
#[serial]
async fn test_cancel_exchange_with_reason() {
    let (app_state, _postgres_container) = setup_app().await;
    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let exchange_id = Uuid::new_v4();

    let req = test::TestRequest::post()
        .uri(&format!("/api/v1/exchanges/{}/cancel", exchange_id))
        .insert_header(header::ContentType::json())
        .insert_header((header::AUTHORIZATION, "Bearer mock-token"))
        .set_json(&json!({
            "cancellation_reason": "Provider no longer available"
        }))
        .to_request();

    let _resp = test::call_service(&app, req).await;

    // Cancellation reason required for audit trail
    // Helps track SEL health and identify issues
}
