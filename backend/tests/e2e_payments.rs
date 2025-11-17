// E2E tests for payment & payment method HTTP endpoints (Issue #84)
// Tests focus on HTTP layer: endpoints, auth, JSON serialization
// Covers Stripe Payment Integration + SEPA Direct Debit workflows

use actix_web::http::header;
use actix_web::{test, App};
use chrono::{Duration, Utc};
use koprogo_api::application::dto::*;
use koprogo_api::application::use_cases::*;
use koprogo_api::domain::entities::{PaymentMethodType, TransactionStatus};
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

/// Setup function shared across all payment E2E tests
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

    // Initialize ALL repositories (AppState requires all of them)
    let user_repo = Arc::new(PostgresUserRepository::new(pool.clone()));
    let user_role_repo = Arc::new(PostgresUserRoleRepository::new(pool.clone()));
    let refresh_repo = Arc::new(PostgresRefreshTokenRepository::new(pool.clone()));
    let building_repo = Arc::new(PostgresBuildingRepository::new(pool.clone()));
    let unit_repo = Arc::new(PostgresUnitRepository::new(pool.clone()));
    let owner_repo = Arc::new(PostgresOwnerRepository::new(pool.clone()));
    let unit_owner_repo = Arc::new(PostgresUnitOwnerRepository::new(pool.clone()));
    let expense_repo = Arc::new(PostgresExpenseRepository::new(pool.clone()));
    let meeting_repo = Arc::new(PostgresMeetingRepository::new(pool.clone()));
    let document_repo = Arc::new(PostgresDocumentRepository::new(pool.clone()));
    let resolution_repo = Arc::new(PostgresResolutionRepository::new(pool.clone()));
    let vote_repo = Arc::new(PostgresVoteRepository::new(pool.clone()));
    let payment_repo = Arc::new(PostgresPaymentRepository::new(pool.clone()));
    let payment_method_repo = Arc::new(PostgresPaymentMethodRepository::new(pool.clone()));
    let gdpr_repo = Arc::new(PostgresGdprRepository::new(Arc::new(pool.clone())));
    let audit_log_repo = Arc::new(PostgresAuditLogRepository::new(pool.clone()));
    let charge_distribution_repo =
        Arc::new(PostgresChargeDistributionRepository::new(pool.clone()));
    let payment_reminder_repo = Arc::new(PostgresPaymentReminderRepository::new(pool.clone()));
    let board_member_repo = Arc::new(PostgresBoardMemberRepository::new(pool.clone()));
    let board_decision_repo = Arc::new(PostgresBoardDecisionRepository::new(pool.clone()));
    let organization_repo = Arc::new(PostgresOrganizationRepository::new(pool.clone()));
    let ticket_repo = Arc::new(PostgresTicketRepository::new(pool.clone()));
    let notification_repo = Arc::new(PostgresNotificationRepository::new(pool.clone()));
    let notification_preference_repo =
        Arc::new(PostgresNotificationPreferenceRepository::new(pool.clone()));
    let quote_repo = Arc::new(PostgresQuoteRepository::new(pool.clone()));
    let local_exchange_repo = Arc::new(PostgresLocalExchangeRepository::new(pool.clone()));
    let owner_credit_balance_repo =
        Arc::new(PostgresOwnerCreditBalanceRepository::new(pool.clone()));
    let notice_repo = Arc::new(PostgresNoticeRepository::new(pool.clone()));
    let skill_repo = Arc::new(PostgresSkillRepository::new(pool.clone()));
    let shared_object_repo = Arc::new(PostgresSharedObjectRepository::new(pool.clone()));
    let resource_booking_repo = Arc::new(PostgresResourceBookingRepository::new(pool.clone()));
    let convocation_repo = Arc::new(PostgresConvocationRepository::new(pool.clone()));
    let convocation_recipient_repo =
        Arc::new(PostgresConvocationRecipientRepository::new(pool.clone()));
    let budget_repo = Arc::new(PostgresBudgetRepository::new(pool.clone()));
    let etat_date_repo = Arc::new(PostgresEtatDateRepository::new(pool.clone()));
    let achievement_repo = Arc::new(PostgresAchievementRepository::new(pool.clone()));
    let user_achievement_repo = Arc::new(PostgresUserAchievementRepository::new(pool.clone()));
    let challenge_repo = Arc::new(PostgresChallengeRepository::new(pool.clone()));
    let challenge_progress_repo =
        Arc::new(PostgresChallengeProgressRepository::new(pool.clone()));

    let audit_logger = AuditLogger::new(Some(audit_log_repo.clone()));

    // Initialize use cases
    let jwt_secret = "e2e-payment-secret".to_string();
    let account_repo = Arc::new(PostgresAccountRepository::new(pool.clone()));
    let account_use_cases = AccountUseCases::new(account_repo.clone());
    let financial_report_use_cases =
        FinancialReportUseCases::new(account_repo, expense_repo.clone());

    let auth_use_cases = AuthUseCases::new(
        user_repo.clone(),
        refresh_repo,
        user_role_repo,
        jwt_secret,
    );
    let building_use_cases = BuildingUseCases::new(building_repo.clone());
    let budget_use_cases = BudgetUseCases::new(budget_repo, building_repo.clone());
    let unit_use_cases = UnitUseCases::new(unit_repo.clone());
    let owner_use_cases = OwnerUseCases::new(owner_repo.clone());
    let unit_owner_use_cases = UnitOwnerUseCases::new(
        unit_owner_repo.clone(),
        unit_repo.clone(),
        owner_repo.clone(),
    );
    let expense_use_cases = ExpenseUseCases::new(expense_repo.clone());
    let charge_distribution_use_cases = ChargeDistributionUseCases::new(
        charge_distribution_repo,
        expense_repo.clone(),
        unit_owner_repo,
    );
    let meeting_use_cases = MeetingUseCases::new(meeting_repo.clone());
    let convocation_use_cases = ConvocationUseCases::new(
        convocation_repo,
        convocation_recipient_repo,
        meeting_repo.clone(),
        owner_repo.clone(),
    );
    let resolution_use_cases = ResolutionUseCases::new(resolution_repo, vote_repo);
    let payment_use_cases = PaymentUseCases::new(payment_repo, payment_method_repo.clone());
    let payment_method_use_cases = PaymentMethodUseCases::new(payment_method_repo);
    let gdpr_use_cases = GdprUseCases::new(gdpr_repo, user_repo.clone());
    let payment_reminder_use_cases = PaymentReminderUseCases::new(
        payment_reminder_repo,
        expense_repo,
        owner_repo.clone(),
    );
    let board_member_use_cases = BoardMemberUseCases::new(board_member_repo);
    let board_decision_use_cases = BoardDecisionUseCases::new(board_decision_repo, user_repo.clone());
    let board_dashboard_use_cases =
        BoardDashboardUseCases::new(building_repo.clone(), meeting_repo);
    let organization_use_cases = OrganizationUseCases::new(organization_repo);
    let ticket_use_cases = TicketUseCases::new(ticket_repo, building_repo.clone());
    let notification_use_cases =
        NotificationUseCases::new(notification_repo, notification_preference_repo);
    let quote_use_cases = QuoteUseCases::new(quote_repo, building_repo);
    let local_exchange_use_cases = LocalExchangeUseCases::new(
        local_exchange_repo,
        owner_credit_balance_repo,
        owner_repo.clone(),
    );
    let notice_use_cases = NoticeUseCases::new(notice_repo, owner_repo.clone());
    let skill_use_cases = SkillUseCases::new(skill_repo, owner_repo.clone());
    let shared_object_use_cases = SharedObjectUseCases::new(shared_object_repo, owner_repo.clone());
    let resource_booking_use_cases =
        ResourceBookingUseCases::new(resource_booking_repo, owner_repo.clone());
    let etat_date_use_cases = EtatDateUseCases::new(etat_date_repo, unit_repo, owner_repo);
    let pcn_use_cases = PcnUseCases::new();
    let achievement_use_cases = AchievementUseCases::new(achievement_repo, user_achievement_repo);
    let challenge_use_cases = ChallengeUseCases::new(challenge_repo, challenge_progress_repo);
    let gamification_stats_use_cases = GamificationStatsUseCases::new(
        achievement_use_cases.clone(),
        challenge_use_cases.clone(),
        user_repo,
    );

    let email_service = Arc::new(EmailService::new());

    let test_id = Uuid::new_v4();
    let storage_root = std::env::temp_dir().join(format!("koprogo_e2e_payments_{}", test_id));
    let storage: Arc<dyn StorageProvider> =
        Arc::new(FileStorage::new(&storage_root).expect("Failed to create file storage"));

    let document_use_cases = DocumentUseCases::new(document_repo, storage.clone());

    let app_state = actix_web::web::Data::new(AppState::new(
        account_use_cases,
        auth_use_cases,
        building_use_cases,
        budget_use_cases,
        unit_use_cases,
        owner_use_cases,
        unit_owner_use_cases,
        expense_use_cases,
        charge_distribution_use_cases,
        meeting_use_cases,
        convocation_use_cases,
        resolution_use_cases,
        ticket_use_cases,
        notification_use_cases,
        payment_use_cases,
        payment_method_use_cases,
        quote_use_cases,
        local_exchange_use_cases,
        notice_use_cases,
        resource_booking_use_cases,
        shared_object_use_cases,
        skill_use_cases,
        document_use_cases,
        etat_date_use_cases,
        pcn_use_cases,
        payment_reminder_use_cases,
        gdpr_use_cases,
        board_member_use_cases,
        board_decision_use_cases,
        board_dashboard_use_cases,
        financial_report_use_cases,
        achievement_use_cases,
        challenge_use_cases,
        gamification_stats_use_cases,
        audit_logger,
        email_service,
        pool,
    ));

    (app_state, postgres_container)
}

/// Helper: Create test fixtures (organization, building, owner, expense)
async fn create_test_fixtures(
    app_state: &actix_web::web::Data<AppState>,
) -> (String, Uuid, Uuid, Uuid, Uuid) {
    // 1. Register user and get token
    let register_dto = RegisterUserDto {
        email: format!("payment-test-{}@example.com", Uuid::new_v4()),
        password: "SecurePass123!".to_string(),
        first_name: "Payment".to_string(),
        last_name: "Tester".to_string(),
    };

    let _user = app_state
        .auth_use_cases
        .register(register_dto.clone())
        .await
        .expect("Failed to register user");

    let login = app_state
        .auth_use_cases
        .login(register_dto.email, register_dto.password)
        .await
        .expect("Failed to login");

    let token = login.access_token;

    // 2. Create organization
    let org_dto = CreateOrganizationDto {
        name: format!("Test Org Payment {}", Uuid::new_v4()),
        registration_number: format!("REG-PAY-{}", Uuid::new_v4()),
        address: "123 Payment St".to_string(),
        city: "Brussels".to_string(),
        postal_code: "1000".to_string(),
        country: "Belgium".to_string(),
        phone: "+32 2 123 4567".to_string(),
        email: format!("org-pay-{}@example.com", Uuid::new_v4()),
    };

    let organization = app_state
        .organization_use_cases
        .create_organization(org_dto)
        .await
        .expect("Failed to create organization");

    // 3. Create building
    let building_dto = CreateBuildingDto {
        organization_id: organization.id,
        name: format!("Test Building Payment {}", Uuid::new_v4()),
        address: "456 Stripe Ave".to_string(),
        city: "Brussels".to_string(),
        postal_code: "1000".to_string(),
        country: "Belgium".to_string(),
        total_units: 5,
        construction_year: Some(2020),
    };

    let building = app_state
        .building_use_cases
        .create_building(building_dto)
        .await
        .expect("Failed to create building");

    // 4. Create owner
    let owner_dto = CreateOwnerDto {
        organization_id: organization.id,
        first_name: "Payment".to_string(),
        last_name: "Owner".to_string(),
        email: format!("payment-owner-{}@example.com", Uuid::new_v4()),
        phone: Some("+32 2 999 9999".to_string()),
    };

    let owner = app_state
        .owner_use_cases
        .create_owner(owner_dto)
        .await
        .expect("Failed to create owner");

    // 5. Create expense
    let expense_dto = CreateExpenseDto {
        organization_id: organization.id,
        building_id: building.id,
        category: koprogo_api::domain::entities::ExpenseCategory::Maintenance,
        amount: rust_decimal::Decimal::new(50000, 2), // 500.00 EUR
        description: "Test expense for payment".to_string(),
        expense_date: Utc::now().date_naive(),
        vendor: Some("Test Vendor".to_string()),
    };

    let expense = app_state
        .expense_use_cases
        .create_expense(expense_dto)
        .await
        .expect("Failed to create expense");

    (token, organization.id, building.id, owner.id, expense.id)
}

// ==================== Payment Tests ====================

#[actix_web::test]
#[serial]
async fn test_create_payment_success() {
    let (app_state, _container) = setup_app().await;
    let (token, _org_id, building_id, owner_id, expense_id) =
        create_test_fixtures(&app_state).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/v1/payments")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "building_id": building_id.to_string(),
            "owner_id": owner_id.to_string(),
            "expense_id": expense_id.to_string(),
            "amount_cents": 50000, // 500.00 EUR
            "payment_method_type": "Card",
            "description": "Payment for maintenance expense"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201, "Should create payment successfully");

    let payment: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(payment["amount_cents"], 50000);
    assert_eq!(payment["currency"], "EUR");
    assert_eq!(payment["status"], "Pending");
    assert_eq!(payment["payment_method_type"], "Card");
    assert_eq!(payment["refunded_amount_cents"], 0);
    assert_eq!(payment["net_amount_cents"], 50000);
}

#[actix_web::test]
#[serial]
async fn test_create_payment_without_auth_fails() {
    let (app_state, _container) = setup_app().await;
    let (_token, _org_id, building_id, owner_id, expense_id) =
        create_test_fixtures(&app_state).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/v1/payments")
        .set_json(json!({
            "building_id": building_id.to_string(),
            "owner_id": owner_id.to_string(),
            "expense_id": expense_id.to_string(),
            "amount_cents": 50000,
            "payment_method_type": "Card"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401, "Should require authentication");
}

#[actix_web::test]
#[serial]
async fn test_get_payment_by_id() {
    let (app_state, _container) = setup_app().await;
    let (token, _org_id, building_id, owner_id, expense_id) =
        create_test_fixtures(&app_state).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create payment first
    let create_req = test::TestRequest::post()
        .uri("/api/v1/payments")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "building_id": building_id.to_string(),
            "owner_id": owner_id.to_string(),
            "expense_id": expense_id.to_string(),
            "amount_cents": 75000,
            "payment_method_type": "SepaDebit",
            "description": "SEPA payment test"
        }))
        .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    let payment: serde_json::Value = test::read_body_json(create_resp).await;
    let payment_id = payment["id"].as_str().unwrap();

    // Get payment by ID
    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/payments/{}", payment_id))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let fetched: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(fetched["id"], payment_id);
    assert_eq!(fetched["amount_cents"], 75000);
    assert_eq!(fetched["payment_method_type"], "SepaDebit");
}

#[actix_web::test]
#[serial]
async fn test_get_payment_not_found() {
    let (app_state, _container) = setup_app().await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let fake_id = Uuid::new_v4();
    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/payments/{}", fake_id))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 404);
}

#[actix_web::test]
#[serial]
async fn test_list_owner_payments() {
    let (app_state, _container) = setup_app().await;
    let (token, _org_id, building_id, owner_id, expense_id) =
        create_test_fixtures(&app_state).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create 3 payments for the same owner
    for i in 1..=3 {
        let req = test::TestRequest::post()
            .uri("/api/v1/payments")
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
            .set_json(json!({
                "building_id": building_id.to_string(),
                "owner_id": owner_id.to_string(),
                "expense_id": expense_id.to_string(),
                "amount_cents": 10000 * i,
                "payment_method_type": "Card"
            }))
            .to_request();

        test::call_service(&app, req).await;
    }

    // List all payments for the owner
    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/owners/{}/payments", owner_id))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let payments: serde_json::Value = test::read_body_json(resp).await;
    let payments_array = payments.as_array().unwrap();
    assert_eq!(payments_array.len(), 3, "Should return all 3 payments");
}

#[actix_web::test]
#[serial]
async fn test_list_building_payments() {
    let (app_state, _container) = setup_app().await;
    let (token, _org_id, building_id, owner_id, expense_id) =
        create_test_fixtures(&app_state).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create 2 payments for the building
    for i in 1..=2 {
        let req = test::TestRequest::post()
            .uri("/api/v1/payments")
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
            .set_json(json!({
                "building_id": building_id.to_string(),
                "owner_id": owner_id.to_string(),
                "expense_id": expense_id.to_string(),
                "amount_cents": 20000 * i,
                "payment_method_type": "BankTransfer"
            }))
            .to_request();

        test::call_service(&app, req).await;
    }

    // List all payments for the building
    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/buildings/{}/payments", building_id))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let payments: serde_json::Value = test::read_body_json(resp).await;
    assert!(payments.as_array().unwrap().len() >= 2);
}

#[actix_web::test]
#[serial]
async fn test_list_expense_payments() {
    let (app_state, _container) = setup_app().await;
    let (token, _org_id, building_id, owner_id, expense_id) =
        create_test_fixtures(&app_state).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create payment for specific expense
    let req = test::TestRequest::post()
        .uri("/api/v1/payments")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "building_id": building_id.to_string(),
            "owner_id": owner_id.to_string(),
            "expense_id": expense_id.to_string(),
            "amount_cents": 30000,
            "payment_method_type": "Card"
        }))
        .to_request();

    test::call_service(&app, req).await;

    // List payments for the expense
    let list_req = test::TestRequest::get()
        .uri(&format!("/api/v1/expenses/{}/payments", expense_id))
        .to_request();

    let resp = test::call_service(&app, list_req).await;
    assert_eq!(resp.status(), 200);

    let payments: serde_json::Value = test::read_body_json(resp).await;
    assert!(payments.as_array().unwrap().len() >= 1);
}

#[actix_web::test]
#[serial]
async fn test_payment_status_transitions() {
    let (app_state, _container) = setup_app().await;
    let (token, _org_id, building_id, owner_id, expense_id) =
        create_test_fixtures(&app_state).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create payment
    let create_req = test::TestRequest::post()
        .uri("/api/v1/payments")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "building_id": building_id.to_string(),
            "owner_id": owner_id.to_string(),
            "expense_id": expense_id.to_string(),
            "amount_cents": 100000,
            "payment_method_type": "Card",
            "description": "Payment lifecycle test"
        }))
        .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    let payment: serde_json::Value = test::read_body_json(create_resp).await;
    let payment_id = payment["id"].as_str().unwrap();
    assert_eq!(payment["status"], "Pending");

    // Mark as Processing
    let processing_req = test::TestRequest::put()
        .uri(&format!("/api/v1/payments/{}/processing", payment_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let processing_resp = test::call_service(&app, processing_req).await;
    assert_eq!(processing_resp.status(), 200);
    let updated: serde_json::Value = test::read_body_json(processing_resp).await;
    assert_eq!(updated["status"], "Processing");

    // Mark as Succeeded
    let succeeded_req = test::TestRequest::put()
        .uri(&format!("/api/v1/payments/{}/succeeded", payment_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let succeeded_resp = test::call_service(&app, succeeded_req).await;
    assert_eq!(succeeded_resp.status(), 200);
    let succeeded: serde_json::Value = test::read_body_json(succeeded_resp).await;
    assert_eq!(succeeded["status"], "Succeeded");
    assert!(succeeded["succeeded_at"].is_string());
}

#[actix_web::test]
#[serial]
async fn test_payment_failed_status() {
    let (app_state, _container) = setup_app().await;
    let (token, _org_id, building_id, owner_id, expense_id) =
        create_test_fixtures(&app_state).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create payment
    let create_req = test::TestRequest::post()
        .uri("/api/v1/payments")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "building_id": building_id.to_string(),
            "owner_id": owner_id.to_string(),
            "expense_id": expense_id.to_string(),
            "amount_cents": 25000,
            "payment_method_type": "Card"
        }))
        .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    let payment: serde_json::Value = test::read_body_json(create_resp).await;
    let payment_id = payment["id"].as_str().unwrap();

    // Mark as Processing
    let processing_req = test::TestRequest::put()
        .uri(&format!("/api/v1/payments/{}/processing", payment_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    test::call_service(&app, processing_req).await;

    // Mark as Failed
    let failed_req = test::TestRequest::put()
        .uri(&format!("/api/v1/payments/{}/failed", payment_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "failure_reason": "Insufficient funds"
        }))
        .to_request();

    let failed_resp = test::call_service(&app, failed_req).await;
    assert_eq!(failed_resp.status(), 200);
    let failed: serde_json::Value = test::read_body_json(failed_resp).await;
    assert_eq!(failed["status"], "Failed");
    assert!(failed["failed_at"].is_string());
}

#[actix_web::test]
#[serial]
async fn test_payment_cancelled() {
    let (app_state, _container) = setup_app().await;
    let (token, _org_id, building_id, owner_id, expense_id) =
        create_test_fixtures(&app_state).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create payment
    let create_req = test::TestRequest::post()
        .uri("/api/v1/payments")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "building_id": building_id.to_string(),
            "owner_id": owner_id.to_string(),
            "expense_id": expense_id.to_string(),
            "amount_cents": 35000,
            "payment_method_type": "Card"
        }))
        .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    let payment: serde_json::Value = test::read_body_json(create_resp).await;
    let payment_id = payment["id"].as_str().unwrap();

    // Cancel payment
    let cancel_req = test::TestRequest::put()
        .uri(&format!("/api/v1/payments/{}/cancelled", payment_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let cancel_resp = test::call_service(&app, cancel_req).await;
    assert_eq!(cancel_resp.status(), 200);
    let cancelled: serde_json::Value = test::read_body_json(cancel_resp).await;
    assert_eq!(cancelled["status"], "Cancelled");
    assert!(cancelled["cancelled_at"].is_string());
}

#[actix_web::test]
#[serial]
async fn test_refund_payment() {
    let (app_state, _container) = setup_app().await;
    let (token, _org_id, building_id, owner_id, expense_id) =
        create_test_fixtures(&app_state).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create and succeed payment
    let create_req = test::TestRequest::post()
        .uri("/api/v1/payments")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "building_id": building_id.to_string(),
            "owner_id": owner_id.to_string(),
            "expense_id": expense_id.to_string(),
            "amount_cents": 120000, // 1200.00 EUR
            "payment_method_type": "Card"
        }))
        .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    let payment: serde_json::Value = test::read_body_json(create_resp).await;
    let payment_id = payment["id"].as_str().unwrap();

    // Mark as succeeded
    let succeeded_req = test::TestRequest::put()
        .uri(&format!("/api/v1/payments/{}/succeeded", payment_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    test::call_service(&app, succeeded_req).await;

    // Refund partial amount
    let refund_req = test::TestRequest::post()
        .uri(&format!("/api/v1/payments/{}/refund", payment_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "amount_cents": 30000, // Refund 300.00 EUR
            "reason": "Partial refund test"
        }))
        .to_request();

    let refund_resp = test::call_service(&app, refund_req).await;
    assert_eq!(refund_resp.status(), 200);

    let refunded: serde_json::Value = test::read_body_json(refund_resp).await;
    assert_eq!(refunded["status"], "Refunded");
    assert_eq!(refunded["refunded_amount_cents"], 30000);
    assert_eq!(
        refunded["net_amount_cents"], 90000,
        "Net amount should be original minus refund"
    );
}

#[actix_web::test]
#[serial]
async fn test_list_payments_by_status() {
    let (app_state, _container) = setup_app().await;
    let (token, _org_id, building_id, owner_id, expense_id) =
        create_test_fixtures(&app_state).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create payment and mark as succeeded
    let create_req = test::TestRequest::post()
        .uri("/api/v1/payments")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "building_id": building_id.to_string(),
            "owner_id": owner_id.to_string(),
            "expense_id": expense_id.to_string(),
            "amount_cents": 50000,
            "payment_method_type": "Card"
        }))
        .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    let payment: serde_json::Value = test::read_body_json(create_resp).await;
    let payment_id = payment["id"].as_str().unwrap();

    let succeeded_req = test::TestRequest::put()
        .uri(&format!("/api/v1/payments/{}/succeeded", payment_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    test::call_service(&app, succeeded_req).await;

    // List succeeded payments
    let list_req = test::TestRequest::get()
        .uri("/api/v1/payments/status/Succeeded")
        .to_request();

    let resp = test::call_service(&app, list_req).await;
    assert_eq!(resp.status(), 200);

    let payments: serde_json::Value = test::read_body_json(resp).await;
    assert!(payments.as_array().unwrap().len() >= 1);
}

#[actix_web::test]
#[serial]
async fn test_list_pending_payments() {
    let (app_state, _container) = setup_app().await;
    let (token, _org_id, building_id, owner_id, expense_id) =
        create_test_fixtures(&app_state).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create pending payment
    let req = test::TestRequest::post()
        .uri("/api/v1/payments")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "building_id": building_id.to_string(),
            "owner_id": owner_id.to_string(),
            "expense_id": expense_id.to_string(),
            "amount_cents": 15000,
            "payment_method_type": "SepaDebit"
        }))
        .to_request();

    test::call_service(&app, req).await;

    // List pending payments
    let list_req = test::TestRequest::get()
        .uri("/api/v1/payments/pending")
        .to_request();

    let resp = test::call_service(&app, list_req).await;
    assert_eq!(resp.status(), 200);

    let payments: serde_json::Value = test::read_body_json(resp).await;
    assert!(payments.as_array().unwrap().len() >= 1);
}

#[actix_web::test]
#[serial]
async fn test_delete_payment() {
    let (app_state, _container) = setup_app().await;
    let (token, _org_id, building_id, owner_id, expense_id) =
        create_test_fixtures(&app_state).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create payment
    let create_req = test::TestRequest::post()
        .uri("/api/v1/payments")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "building_id": building_id.to_string(),
            "owner_id": owner_id.to_string(),
            "expense_id": expense_id.to_string(),
            "amount_cents": 40000,
            "payment_method_type": "Card"
        }))
        .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    let payment: serde_json::Value = test::read_body_json(create_resp).await;
    let payment_id = payment["id"].as_str().unwrap();

    // Delete payment
    let delete_req = test::TestRequest::delete()
        .uri(&format!("/api/v1/payments/{}", payment_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let delete_resp = test::call_service(&app, delete_req).await;
    assert_eq!(delete_resp.status(), 204);

    // Verify deletion
    let get_req = test::TestRequest::get()
        .uri(&format!("/api/v1/payments/{}", payment_id))
        .to_request();

    let get_resp = test::call_service(&app, get_req).await;
    assert_eq!(get_resp.status(), 404);
}

#[actix_web::test]
#[serial]
async fn test_get_owner_payment_stats() {
    let (app_state, _container) = setup_app().await;
    let (token, _org_id, building_id, owner_id, expense_id) =
        create_test_fixtures(&app_state).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create payments with different statuses
    let amounts = vec![10000, 20000, 30000];
    let mut payment_ids = Vec::new();

    for amount in amounts {
        let req = test::TestRequest::post()
            .uri("/api/v1/payments")
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
            .set_json(json!({
                "building_id": building_id.to_string(),
                "owner_id": owner_id.to_string(),
                "expense_id": expense_id.to_string(),
                "amount_cents": amount,
                "payment_method_type": "Card"
            }))
            .to_request();

        let resp = test::call_service(&app, req).await;
        let payment: serde_json::Value = test::read_body_json(resp).await;
        payment_ids.push(payment["id"].as_str().unwrap().to_string());
    }

    // Mark first as succeeded, second as failed
    let succeeded_req = test::TestRequest::put()
        .uri(&format!("/api/v1/payments/{}/succeeded", payment_ids[0]))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    test::call_service(&app, succeeded_req).await;

    // Get owner payment stats
    let stats_req = test::TestRequest::get()
        .uri(&format!("/api/v1/owners/{}/payments/stats", owner_id))
        .to_request();

    let stats_resp = test::call_service(&app, stats_req).await;
    assert_eq!(stats_resp.status(), 200);

    let stats: serde_json::Value = test::read_body_json(stats_resp).await;
    assert_eq!(stats["total_count"], 3);
    assert_eq!(stats["succeeded_count"], 1);
    assert_eq!(stats["pending_count"], 2);
}

// ==================== Payment Method Tests ====================

#[actix_web::test]
#[serial]
async fn test_create_payment_method() {
    let (app_state, _container) = setup_app().await;
    let (token, _org_id, _building_id, owner_id, _expense_id) =
        create_test_fixtures(&app_state).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/v1/payment-methods")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "owner_id": owner_id.to_string(),
            "method_type": "Card",
            "stripe_payment_method_id": "pm_test_visa_4242",
            "stripe_customer_id": "cus_test_12345",
            "display_label": "Visa ****4242",
            "is_default": true,
            "expires_at": (Utc::now() + Duration::days(365 * 3)).to_rfc3339()
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201, "Should create payment method");

    let method: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(method["method_type"], "Card");
    assert_eq!(method["display_label"], "Visa ****4242");
    assert_eq!(method["is_default"], true);
    assert_eq!(method["is_active"], true);
    assert_eq!(method["is_usable"], true);
}

#[actix_web::test]
#[serial]
async fn test_create_sepa_payment_method() {
    let (app_state, _container) = setup_app().await;
    let (token, _org_id, _building_id, owner_id, _expense_id) =
        create_test_fixtures(&app_state).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/v1/payment-methods")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "owner_id": owner_id.to_string(),
            "method_type": "SepaDebit",
            "stripe_payment_method_id": "pm_sepa_BE68539007547034",
            "stripe_customer_id": "cus_test_54321",
            "display_label": "SEPA Debit ****7034",
            "is_default": false
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    let method: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(method["method_type"], "SepaDebit");
    assert_eq!(method["is_expired"], false);
}

#[actix_web::test]
#[serial]
async fn test_list_owner_payment_methods() {
    let (app_state, _container) = setup_app().await;
    let (token, _org_id, _building_id, owner_id, _expense_id) =
        create_test_fixtures(&app_state).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create 2 payment methods
    for i in 1..=2 {
        let req = test::TestRequest::post()
            .uri("/api/v1/payment-methods")
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
            .set_json(json!({
                "owner_id": owner_id.to_string(),
                "method_type": "Card",
                "stripe_payment_method_id": format!("pm_test_{}", i),
                "stripe_customer_id": format!("cus_test_{}", i),
                "display_label": format!("Card {}", i),
                "is_default": i == 1
            }))
            .to_request();

        test::call_service(&app, req).await;
    }

    // List all payment methods for owner
    let list_req = test::TestRequest::get()
        .uri(&format!("/api/v1/owners/{}/payment-methods", owner_id))
        .to_request();

    let resp = test::call_service(&app, list_req).await;
    assert_eq!(resp.status(), 200);

    let methods: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(methods.as_array().unwrap().len(), 2);
}

#[actix_web::test]
#[serial]
async fn test_set_payment_method_as_default() {
    let (app_state, _container) = setup_app().await;
    let (token, _org_id, _building_id, owner_id, _expense_id) =
        create_test_fixtures(&app_state).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create first method as default
    let req1 = test::TestRequest::post()
        .uri("/api/v1/payment-methods")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "owner_id": owner_id.to_string(),
            "method_type": "Card",
            "stripe_payment_method_id": "pm_test_1",
            "stripe_customer_id": "cus_test_1",
            "display_label": "Card 1",
            "is_default": true
        }))
        .to_request();

    test::call_service(&app, req1).await;

    // Create second method
    let req2 = test::TestRequest::post()
        .uri("/api/v1/payment-methods")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "owner_id": owner_id.to_string(),
            "method_type": "Card",
            "stripe_payment_method_id": "pm_test_2",
            "stripe_customer_id": "cus_test_2",
            "display_label": "Card 2",
            "is_default": false
        }))
        .to_request();

    let resp2 = test::call_service(&app, req2).await;
    let method2: serde_json::Value = test::read_body_json(resp2).await;
    let method2_id = method2["id"].as_str().unwrap();

    // Set second method as default
    let set_default_req = test::TestRequest::put()
        .uri(&format!("/api/v1/payment-methods/{}/set-default", method2_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let set_default_resp = test::call_service(&app, set_default_req).await;
    assert_eq!(set_default_resp.status(), 200);

    let updated: serde_json::Value = test::read_body_json(set_default_resp).await;
    assert_eq!(updated["is_default"], true);
}

#[actix_web::test]
#[serial]
async fn test_deactivate_payment_method() {
    let (app_state, _container) = setup_app().await;
    let (token, _org_id, _building_id, owner_id, _expense_id) =
        create_test_fixtures(&app_state).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create payment method
    let create_req = test::TestRequest::post()
        .uri("/api/v1/payment-methods")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "owner_id": owner_id.to_string(),
            "method_type": "Card",
            "stripe_payment_method_id": "pm_test_deactivate",
            "stripe_customer_id": "cus_test_deactivate",
            "display_label": "Card to Deactivate",
            "is_default": false
        }))
        .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    let method: serde_json::Value = test::read_body_json(create_resp).await;
    let method_id = method["id"].as_str().unwrap();
    assert_eq!(method["is_active"], true);

    // Deactivate method
    let deactivate_req = test::TestRequest::put()
        .uri(&format!("/api/v1/payment-methods/{}/deactivate", method_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let deactivate_resp = test::call_service(&app, deactivate_req).await;
    assert_eq!(deactivate_resp.status(), 200);

    let deactivated: serde_json::Value = test::read_body_json(deactivate_resp).await;
    assert_eq!(deactivated["is_active"], false);
    assert_eq!(deactivated["is_usable"], false);
}

#[actix_web::test]
#[serial]
async fn test_reactivate_payment_method() {
    let (app_state, _container) = setup_app().await;
    let (token, _org_id, _building_id, owner_id, _expense_id) =
        create_test_fixtures(&app_state).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create and deactivate payment method
    let create_req = test::TestRequest::post()
        .uri("/api/v1/payment-methods")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "owner_id": owner_id.to_string(),
            "method_type": "SepaDebit",
            "stripe_payment_method_id": "pm_sepa_test",
            "stripe_customer_id": "cus_sepa_test",
            "display_label": "SEPA Test",
            "is_default": false
        }))
        .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    let method: serde_json::Value = test::read_body_json(create_resp).await;
    let method_id = method["id"].as_str().unwrap();

    // Deactivate
    let deactivate_req = test::TestRequest::put()
        .uri(&format!("/api/v1/payment-methods/{}/deactivate", method_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    test::call_service(&app, deactivate_req).await;

    // Reactivate
    let reactivate_req = test::TestRequest::put()
        .uri(&format!("/api/v1/payment-methods/{}/reactivate", method_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let reactivate_resp = test::call_service(&app, reactivate_req).await;
    assert_eq!(reactivate_resp.status(), 200);

    let reactivated: serde_json::Value = test::read_body_json(reactivate_resp).await;
    assert_eq!(reactivated["is_active"], true);
    assert_eq!(reactivated["is_usable"], true);
}

#[actix_web::test]
#[serial]
async fn test_delete_payment_method() {
    let (app_state, _container) = setup_app().await;
    let (token, _org_id, _building_id, owner_id, _expense_id) =
        create_test_fixtures(&app_state).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create payment method
    let create_req = test::TestRequest::post()
        .uri("/api/v1/payment-methods")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "owner_id": owner_id.to_string(),
            "method_type": "Card",
            "stripe_payment_method_id": "pm_test_delete",
            "stripe_customer_id": "cus_test_delete",
            "display_label": "Card to Delete",
            "is_default": false
        }))
        .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    let method: serde_json::Value = test::read_body_json(create_resp).await;
    let method_id = method["id"].as_str().unwrap();

    // Delete method
    let delete_req = test::TestRequest::delete()
        .uri(&format!("/api/v1/payment-methods/{}", method_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let delete_resp = test::call_service(&app, delete_req).await;
    assert_eq!(delete_resp.status(), 204);

    // Verify deletion
    let get_req = test::TestRequest::get()
        .uri(&format!("/api/v1/payment-methods/{}", method_id))
        .to_request();

    let get_resp = test::call_service(&app, get_req).await;
    assert_eq!(get_resp.status(), 404);
}

#[actix_web::test]
#[serial]
async fn test_complete_payment_lifecycle() {
    let (app_state, _container) = setup_app().await;
    let (token, _org_id, building_id, owner_id, expense_id) =
        create_test_fixtures(&app_state).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // 1. Create payment method
    let method_req = test::TestRequest::post()
        .uri("/api/v1/payment-methods")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "owner_id": owner_id.to_string(),
            "method_type": "Card",
            "stripe_payment_method_id": "pm_lifecycle_test",
            "stripe_customer_id": "cus_lifecycle_test",
            "display_label": "Lifecycle Test Card",
            "is_default": true
        }))
        .to_request();

    let method_resp = test::call_service(&app, method_req).await;
    let method: serde_json::Value = test::read_body_json(method_resp).await;
    let method_id = method["id"].as_str().unwrap();

    // 2. Create payment using saved payment method
    let payment_req = test::TestRequest::post()
        .uri("/api/v1/payments")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "building_id": building_id.to_string(),
            "owner_id": owner_id.to_string(),
            "expense_id": expense_id.to_string(),
            "amount_cents": 200000, // 2000.00 EUR
            "payment_method_type": "Card",
            "payment_method_id": method_id,
            "description": "Complete lifecycle payment"
        }))
        .to_request();

    let payment_resp = test::call_service(&app, payment_req).await;
    let payment: serde_json::Value = test::read_body_json(payment_resp).await;
    let payment_id = payment["id"].as_str().unwrap();
    assert_eq!(payment["status"], "Pending");
    assert_eq!(payment["amount_cents"], 200000);

    // 3. Process payment (Pending → Processing)
    let processing_req = test::TestRequest::put()
        .uri(&format!("/api/v1/payments/{}/processing", payment_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    test::call_service(&app, processing_req).await;

    // 4. Mark as succeeded
    let succeeded_req = test::TestRequest::put()
        .uri(&format!("/api/v1/payments/{}/succeeded", payment_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let succeeded_resp = test::call_service(&app, succeeded_req).await;
    let succeeded: serde_json::Value = test::read_body_json(succeeded_resp).await;
    assert_eq!(succeeded["status"], "Succeeded");
    assert!(succeeded["succeeded_at"].is_string());

    // 5. Partial refund
    let refund_req = test::TestRequest::post()
        .uri(&format!("/api/v1/payments/{}/refund", payment_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "amount_cents": 50000, // Refund 500.00 EUR
            "reason": "Customer request"
        }))
        .to_request();

    let refund_resp = test::call_service(&app, refund_req).await;
    let refunded: serde_json::Value = test::read_body_json(refund_resp).await;
    assert_eq!(refunded["status"], "Refunded");
    assert_eq!(refunded["refunded_amount_cents"], 50000);
    assert_eq!(refunded["net_amount_cents"], 150000); // 2000 - 500

    // 6. Verify payment in lists
    let owner_payments_req = test::TestRequest::get()
        .uri(&format!("/api/v1/owners/{}/payments", owner_id))
        .to_request();

    let owner_payments_resp = test::call_service(&app, owner_payments_req).await;
    let owner_payments: serde_json::Value = test::read_body_json(owner_payments_resp).await;
    assert!(owner_payments.as_array().unwrap().len() >= 1);

    // 7. Get payment stats
    let stats_req = test::TestRequest::get()
        .uri(&format!("/api/v1/owners/{}/payments/stats", owner_id))
        .to_request();

    let stats_resp = test::call_service(&app, stats_req).await;
    let stats: serde_json::Value = test::read_body_json(stats_resp).await;
    assert!(stats["total_count"].as_i64().unwrap() >= 1);
    assert_eq!(stats["total_refunded_cents"], 50000);
}
