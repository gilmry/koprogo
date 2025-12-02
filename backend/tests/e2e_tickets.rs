// E2E tests for ticket management HTTP endpoints (Issue #85)
// Tests focus on HTTP layer: endpoints, auth, JSON serialization
// Covers complete maintenance request workflow for Belgian copropriété

use actix_web::http::header;
use actix_web::{test, App};
use chrono::{Duration, Utc};
use koprogo_api::application::dto::*;
use koprogo_api::application::use_cases::*;
use koprogo_api::domain::entities::{TicketCategory, TicketPriority, TicketStatus};
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

/// Setup function shared across all ticket E2E tests
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

    // Initialize ALL repositories
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
    let ticket_repo = Arc::new(PostgresTicketRepository::new(pool.clone()));
    let gdpr_repo = Arc::new(PostgresGdprRepository::new(Arc::new(pool.clone())));
    let audit_log_repo = Arc::new(PostgresAuditLogRepository::new(pool.clone()));
    let charge_distribution_repo =
        Arc::new(PostgresChargeDistributionRepository::new(pool.clone()));
    let payment_reminder_repo = Arc::new(PostgresPaymentReminderRepository::new(pool.clone()));
    let board_member_repo = Arc::new(PostgresBoardMemberRepository::new(pool.clone()));
    let board_decision_repo = Arc::new(PostgresBoardDecisionRepository::new(pool.clone()));
    let organization_repo = Arc::new(PostgresOrganizationRepository::new(pool.clone()));
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
    let challenge_progress_repo = Arc::new(PostgresChallengeProgressRepository::new(pool.clone()));

    let audit_logger = AuditLogger::new(Some(audit_log_repo.clone()));

    // Initialize use cases
    let jwt_secret = "e2e-ticket-secret".to_string();
    let account_repo = Arc::new(PostgresAccountRepository::new(pool.clone()));
    let account_use_cases = AccountUseCases::new(account_repo.clone());
    let financial_report_use_cases =
        FinancialReportUseCases::new(account_repo, expense_repo.clone());

    let auth_use_cases =
        AuthUseCases::new(user_repo.clone(), refresh_repo, user_role_repo, jwt_secret);
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
    let ticket_use_cases = TicketUseCases::new(ticket_repo, building_repo.clone());
    let gdpr_use_cases = GdprUseCases::new(gdpr_repo, user_repo.clone());
    let payment_reminder_use_cases =
        PaymentReminderUseCases::new(payment_reminder_repo, expense_repo, owner_repo.clone());
    let board_member_use_cases = BoardMemberUseCases::new(board_member_repo);
    let board_decision_use_cases =
        BoardDecisionUseCases::new(board_decision_repo, user_repo.clone());
    let board_dashboard_use_cases =
        BoardDashboardUseCases::new(building_repo.clone(), meeting_repo);
    let organization_use_cases = OrganizationUseCases::new(organization_repo);
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
    let storage_root = std::env::temp_dir().join(format!("koprogo_e2e_tickets_{}", test_id));
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

/// Helper: Create test fixtures (organization, building, unit, user)
async fn create_test_fixtures(
    app_state: &actix_web::web::Data<AppState>,
) -> (String, Uuid, Uuid, Uuid, Uuid) {
    // 1. Register user and get token
    let register_dto = RegisterUserDto {
        email: format!("ticket-test-{}@example.com", Uuid::new_v4()),
        password: "SecurePass123!".to_string(),
        first_name: "Ticket".to_string(),
        last_name: "Tester".to_string(),
    };

    let user = app_state
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
        name: format!("Test Org Ticket {}", Uuid::new_v4()),
        registration_number: format!("REG-TICKET-{}", Uuid::new_v4()),
        address: "123 Ticket St".to_string(),
        city: "Brussels".to_string(),
        postal_code: "1000".to_string(),
        country: "Belgium".to_string(),
        phone: "+32 2 123 4567".to_string(),
        email: format!("org-ticket-{}@example.com", Uuid::new_v4()),
    };

    let organization = app_state
        .organization_use_cases
        .create_organization(org_dto)
        .await
        .expect("Failed to create organization");

    // 3. Create building
    let building_dto = CreateBuildingDto {
        organization_id: organization.id,
        name: format!("Test Building Ticket {}", Uuid::new_v4()),
        address: "456 Maintenance Ave".to_string(),
        city: "Brussels".to_string(),
        postal_code: "1000".to_string(),
        country: "Belgium".to_string(),
        total_units: 10,
        construction_year: Some(2015),
    };

    let building = app_state
        .building_use_cases
        .create_building(building_dto)
        .await
        .expect("Failed to create building");

    // 4. Create unit
    let unit_dto = CreateUnitDto {
        building_id: building.id,
        unit_number: "A101".to_string(),
        floor: 1,
        surface_area: Some(75.0),
        unit_type: koprogo_api::domain::entities::UnitType::Apartment,
    };

    let unit = app_state
        .unit_use_cases
        .create_unit(unit_dto)
        .await
        .expect("Failed to create unit");

    (token, organization.id, building.id, unit.id, user.id)
}

// ==================== Ticket CRUD Tests ====================

#[actix_web::test]
#[serial]
async fn test_create_ticket_success() {
    let (app_state, _container) = setup_app().await;
    let (token, _org_id, building_id, unit_id, _user_id) = create_test_fixtures(&app_state).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/v1/tickets")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "building_id": building_id.to_string(),
            "unit_id": unit_id.to_string(),
            "title": "Leaking faucet in bathroom",
            "description": "The bathroom faucet has been leaking for 2 days. Water is dripping constantly.",
            "category": "Plumbing",
            "priority": "Medium"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201, "Should create ticket successfully");

    let ticket: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(ticket["title"], "Leaking faucet in bathroom");
    assert_eq!(ticket["category"], "Plumbing");
    assert_eq!(ticket["priority"], "Medium");
    assert_eq!(ticket["status"], "Open");
    assert!(ticket["assigned_to"].is_null());
}

#[actix_web::test]
#[serial]
async fn test_create_ticket_without_auth_fails() {
    let (app_state, _container) = setup_app().await;
    let (_token, _org_id, building_id, unit_id, _user_id) = create_test_fixtures(&app_state).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/v1/tickets")
        .set_json(json!({
            "building_id": building_id.to_string(),
            "unit_id": unit_id.to_string(),
            "title": "Unauthorized ticket",
            "description": "This should fail",
            "category": "Electrical",
            "priority": "Low"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401, "Should require authentication");
}

#[actix_web::test]
#[serial]
async fn test_create_ticket_all_categories() {
    let (app_state, _container) = setup_app().await;
    let (token, _org_id, building_id, unit_id, _user_id) = create_test_fixtures(&app_state).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let categories = vec![
        "Plumbing",
        "Electrical",
        "Heating",
        "CommonAreas",
        "Elevator",
        "Security",
        "Cleaning",
        "Landscaping",
        "Other",
    ];

    for category in categories {
        let req = test::TestRequest::post()
            .uri("/api/v1/tickets")
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
            .set_json(json!({
                "building_id": building_id.to_string(),
                "unit_id": unit_id.to_string(),
                "title": format!("Test {} ticket", category),
                "description": format!("Testing {} category", category),
                "category": category,
                "priority": "Low"
            }))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(
            resp.status(),
            201,
            "Should create ticket for category {}",
            category
        );
    }
}

#[actix_web::test]
#[serial]
async fn test_create_ticket_all_priorities() {
    let (app_state, _container) = setup_app().await;
    let (token, _org_id, building_id, unit_id, _user_id) = create_test_fixtures(&app_state).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let priorities = vec!["Low", "Medium", "High", "Critical"];

    for priority in priorities {
        let req = test::TestRequest::post()
            .uri("/api/v1/tickets")
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
            .set_json(json!({
                "building_id": building_id.to_string(),
                "unit_id": unit_id.to_string(),
                "title": format!("Test {} priority ticket", priority),
                "description": format!("Testing {} priority", priority),
                "category": "Other",
                "priority": priority
            }))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(
            resp.status(),
            201,
            "Should create ticket with priority {}",
            priority
        );
    }
}

#[actix_web::test]
#[serial]
async fn test_get_ticket_by_id() {
    let (app_state, _container) = setup_app().await;
    let (token, _org_id, building_id, unit_id, _user_id) = create_test_fixtures(&app_state).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create ticket
    let create_req = test::TestRequest::post()
        .uri("/api/v1/tickets")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "building_id": building_id.to_string(),
            "unit_id": unit_id.to_string(),
            "title": "Broken window",
            "description": "Window in living room is broken",
            "category": "CommonAreas",
            "priority": "High"
        }))
        .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    let ticket: serde_json::Value = test::read_body_json(create_resp).await;
    let ticket_id = ticket["id"].as_str().unwrap();

    // Get ticket
    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/tickets/{}", ticket_id))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let fetched: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(fetched["id"], ticket_id);
    assert_eq!(fetched["title"], "Broken window");
}

#[actix_web::test]
#[serial]
async fn test_get_ticket_not_found() {
    let (app_state, _container) = setup_app().await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let fake_id = Uuid::new_v4();
    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/tickets/{}", fake_id))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 404);
}

#[actix_web::test]
#[serial]
async fn test_list_building_tickets() {
    let (app_state, _container) = setup_app().await;
    let (token, _org_id, building_id, unit_id, _user_id) = create_test_fixtures(&app_state).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create 3 tickets for the building
    for i in 1..=3 {
        let req = test::TestRequest::post()
            .uri("/api/v1/tickets")
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
            .set_json(json!({
                "building_id": building_id.to_string(),
                "unit_id": unit_id.to_string(),
                "title": format!("Ticket #{}", i),
                "description": format!("Description for ticket {}", i),
                "category": "Other",
                "priority": "Low"
            }))
            .to_request();

        test::call_service(&app, req).await;
    }

    // List all tickets for the building
    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/buildings/{}/tickets", building_id))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let tickets: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(tickets.as_array().unwrap().len(), 3);
}

#[actix_web::test]
#[serial]
async fn test_list_tickets_by_status() {
    let (app_state, _container) = setup_app().await;
    let (token, _org_id, building_id, unit_id, _user_id) = create_test_fixtures(&app_state).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create ticket
    let create_req = test::TestRequest::post()
        .uri("/api/v1/tickets")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "building_id": building_id.to_string(),
            "unit_id": unit_id.to_string(),
            "title": "Status test ticket",
            "description": "Testing status filtering",
            "category": "Other",
            "priority": "Low"
        }))
        .to_request();

    test::call_service(&app, create_req).await;

    // List Open tickets
    let req = test::TestRequest::get()
        .uri(&format!(
            "/api/v1/buildings/{}/tickets/status/Open",
            building_id
        ))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let tickets: serde_json::Value = test::read_body_json(resp).await;
    assert!(tickets.as_array().unwrap().len() >= 1);
}

#[actix_web::test]
#[serial]
async fn test_delete_ticket() {
    let (app_state, _container) = setup_app().await;
    let (token, _org_id, building_id, unit_id, _user_id) = create_test_fixtures(&app_state).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create ticket
    let create_req = test::TestRequest::post()
        .uri("/api/v1/tickets")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "building_id": building_id.to_string(),
            "unit_id": unit_id.to_string(),
            "title": "Ticket to delete",
            "description": "This will be deleted",
            "category": "Other",
            "priority": "Low"
        }))
        .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    let ticket: serde_json::Value = test::read_body_json(create_resp).await;
    let ticket_id = ticket["id"].as_str().unwrap();

    // Delete ticket
    let delete_req = test::TestRequest::delete()
        .uri(&format!("/api/v1/tickets/{}", ticket_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let delete_resp = test::call_service(&app, delete_req).await;
    assert_eq!(delete_resp.status(), 204);

    // Verify deletion
    let get_req = test::TestRequest::get()
        .uri(&format!("/api/v1/tickets/{}", ticket_id))
        .to_request();

    let get_resp = test::call_service(&app, get_req).await;
    assert_eq!(get_resp.status(), 404);
}

// ==================== Ticket Workflow Tests ====================

#[actix_web::test]
#[serial]
async fn test_assign_ticket() {
    let (app_state, _container) = setup_app().await;
    let (token, _org_id, building_id, unit_id, user_id) = create_test_fixtures(&app_state).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create ticket
    let create_req = test::TestRequest::post()
        .uri("/api/v1/tickets")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "building_id": building_id.to_string(),
            "unit_id": unit_id.to_string(),
            "title": "Ticket for assignment",
            "description": "Will be assigned",
            "category": "Plumbing",
            "priority": "Medium"
        }))
        .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    let ticket: serde_json::Value = test::read_body_json(create_resp).await;
    let ticket_id = ticket["id"].as_str().unwrap();
    assert_eq!(ticket["status"], "Open");
    assert!(ticket["assigned_to"].is_null());

    // Assign ticket
    let assign_req = test::TestRequest::put()
        .uri(&format!("/api/v1/tickets/{}/assign", ticket_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "assigned_to": user_id.to_string()
        }))
        .to_request();

    let assign_resp = test::call_service(&app, assign_req).await;
    assert_eq!(assign_resp.status(), 200);

    let assigned: serde_json::Value = test::read_body_json(assign_resp).await;
    assert_eq!(assigned["status"], "InProgress");
    assert_eq!(assigned["assigned_to"], user_id.to_string());
}

#[actix_web::test]
#[serial]
async fn test_start_work_on_ticket() {
    let (app_state, _container) = setup_app().await;
    let (token, _org_id, building_id, unit_id, _user_id) = create_test_fixtures(&app_state).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create ticket
    let create_req = test::TestRequest::post()
        .uri("/api/v1/tickets")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "building_id": building_id.to_string(),
            "unit_id": unit_id.to_string(),
            "title": "Work start test",
            "description": "Testing start work",
            "category": "Electrical",
            "priority": "High"
        }))
        .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    let ticket: serde_json::Value = test::read_body_json(create_resp).await;
    let ticket_id = ticket["id"].as_str().unwrap();

    // Start work
    let start_req = test::TestRequest::put()
        .uri(&format!("/api/v1/tickets/{}/start-work", ticket_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let start_resp = test::call_service(&app, start_req).await;
    assert_eq!(start_resp.status(), 200);

    let started: serde_json::Value = test::read_body_json(start_resp).await;
    assert_eq!(started["status"], "InProgress");
}

#[actix_web::test]
#[serial]
async fn test_resolve_ticket() {
    let (app_state, _container) = setup_app().await;
    let (token, _org_id, building_id, unit_id, _user_id) = create_test_fixtures(&app_state).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create and start work on ticket
    let create_req = test::TestRequest::post()
        .uri("/api/v1/tickets")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "building_id": building_id.to_string(),
            "unit_id": unit_id.to_string(),
            "title": "Resolve test",
            "description": "Will be resolved",
            "category": "Heating",
            "priority": "Medium"
        }))
        .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    let ticket: serde_json::Value = test::read_body_json(create_resp).await;
    let ticket_id = ticket["id"].as_str().unwrap();

    // Start work first
    let start_req = test::TestRequest::put()
        .uri(&format!("/api/v1/tickets/{}/start-work", ticket_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    test::call_service(&app, start_req).await;

    // Resolve ticket
    let resolve_req = test::TestRequest::put()
        .uri(&format!("/api/v1/tickets/{}/resolve", ticket_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "resolution_notes": "Replaced the heating valve. System is now working properly."
        }))
        .to_request();

    let resolve_resp = test::call_service(&app, resolve_req).await;
    assert_eq!(resolve_resp.status(), 200);

    let resolved: serde_json::Value = test::read_body_json(resolve_resp).await;
    assert_eq!(resolved["status"], "Resolved");
    assert!(resolved["resolved_at"].is_string());
    assert_eq!(
        resolved["resolution_notes"],
        "Replaced the heating valve. System is now working properly."
    );
}

#[actix_web::test]
#[serial]
async fn test_close_ticket() {
    let (app_state, _container) = setup_app().await;
    let (token, _org_id, building_id, unit_id, _user_id) = create_test_fixtures(&app_state).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create, start, resolve ticket
    let create_req = test::TestRequest::post()
        .uri("/api/v1/tickets")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "building_id": building_id.to_string(),
            "unit_id": unit_id.to_string(),
            "title": "Close test",
            "description": "Will be closed",
            "category": "Cleaning",
            "priority": "Low"
        }))
        .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    let ticket: serde_json::Value = test::read_body_json(create_resp).await;
    let ticket_id = ticket["id"].as_str().unwrap();

    // Start work
    let start_req = test::TestRequest::put()
        .uri(&format!("/api/v1/tickets/{}/start-work", ticket_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    test::call_service(&app, start_req).await;

    // Resolve
    let resolve_req = test::TestRequest::put()
        .uri(&format!("/api/v1/tickets/{}/resolve", ticket_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "resolution_notes": "Cleaned the area thoroughly"
        }))
        .to_request();

    test::call_service(&app, resolve_req).await;

    // Close ticket
    let close_req = test::TestRequest::put()
        .uri(&format!("/api/v1/tickets/{}/close", ticket_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let close_resp = test::call_service(&app, close_req).await;
    assert_eq!(close_resp.status(), 200);

    let closed: serde_json::Value = test::read_body_json(close_resp).await;
    assert_eq!(closed["status"], "Closed");
    assert!(closed["closed_at"].is_string());
}

#[actix_web::test]
#[serial]
async fn test_cancel_ticket() {
    let (app_state, _container) = setup_app().await;
    let (token, _org_id, building_id, unit_id, _user_id) = create_test_fixtures(&app_state).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create ticket
    let create_req = test::TestRequest::post()
        .uri("/api/v1/tickets")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "building_id": building_id.to_string(),
            "unit_id": unit_id.to_string(),
            "title": "Cancel test",
            "description": "Will be cancelled",
            "category": "Other",
            "priority": "Low"
        }))
        .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    let ticket: serde_json::Value = test::read_body_json(create_resp).await;
    let ticket_id = ticket["id"].as_str().unwrap();

    // Cancel ticket
    let cancel_req = test::TestRequest::put()
        .uri(&format!("/api/v1/tickets/{}/cancel", ticket_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "reason": "Duplicate ticket, already reported"
        }))
        .to_request();

    let cancel_resp = test::call_service(&app, cancel_req).await;
    assert_eq!(cancel_resp.status(), 200);

    let cancelled: serde_json::Value = test::read_body_json(cancel_resp).await;
    assert_eq!(cancelled["status"], "Cancelled");
}

#[actix_web::test]
#[serial]
async fn test_reopen_ticket() {
    let (app_state, _container) = setup_app().await;
    let (token, _org_id, building_id, unit_id, _user_id) = create_test_fixtures(&app_state).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create, start, resolve, close ticket
    let create_req = test::TestRequest::post()
        .uri("/api/v1/tickets")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "building_id": building_id.to_string(),
            "unit_id": unit_id.to_string(),
            "title": "Reopen test",
            "description": "Will be reopened",
            "category": "Security",
            "priority": "High"
        }))
        .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    let ticket: serde_json::Value = test::read_body_json(create_resp).await;
    let ticket_id = ticket["id"].as_str().unwrap();

    // Start work
    let start_req = test::TestRequest::put()
        .uri(&format!("/api/v1/tickets/{}/start-work", ticket_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    test::call_service(&app, start_req).await;

    // Resolve
    let resolve_req = test::TestRequest::put()
        .uri(&format!("/api/v1/tickets/{}/resolve", ticket_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "resolution_notes": "Fixed the security issue"
        }))
        .to_request();

    test::call_service(&app, resolve_req).await;

    // Close
    let close_req = test::TestRequest::put()
        .uri(&format!("/api/v1/tickets/{}/close", ticket_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    test::call_service(&app, close_req).await;

    // Reopen ticket
    let reopen_req = test::TestRequest::put()
        .uri(&format!("/api/v1/tickets/{}/reopen", ticket_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "reason": "Issue returned, needs further investigation"
        }))
        .to_request();

    let reopen_resp = test::call_service(&app, reopen_req).await;
    assert_eq!(reopen_resp.status(), 200);

    let reopened: serde_json::Value = test::read_body_json(reopen_resp).await;
    assert_eq!(reopened["status"], "Open");
}

#[actix_web::test]
#[serial]
async fn test_get_ticket_statistics() {
    let (app_state, _container) = setup_app().await;
    let (token, _org_id, building_id, unit_id, _user_id) = create_test_fixtures(&app_state).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create tickets with different statuses
    for i in 1..=3 {
        let req = test::TestRequest::post()
            .uri("/api/v1/tickets")
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
            .set_json(json!({
                "building_id": building_id.to_string(),
                "unit_id": unit_id.to_string(),
                "title": format!("Stats ticket {}", i),
                "description": format!("For statistics test {}", i),
                "category": "Other",
                "priority": "Low"
            }))
            .to_request();

        test::call_service(&app, req).await;
    }

    // Get statistics
    let stats_req = test::TestRequest::get()
        .uri(&format!(
            "/api/v1/buildings/{}/tickets/statistics",
            building_id
        ))
        .to_request();

    let stats_resp = test::call_service(&app, stats_req).await;
    assert_eq!(stats_resp.status(), 200);

    let stats: serde_json::Value = test::read_body_json(stats_resp).await;
    assert!(stats["total_tickets"].as_i64().unwrap() >= 3);
}

#[actix_web::test]
#[serial]
async fn test_complete_ticket_lifecycle() {
    let (app_state, _container) = setup_app().await;
    let (token, _org_id, building_id, unit_id, user_id) = create_test_fixtures(&app_state).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // 1. Create ticket
    let create_req = test::TestRequest::post()
        .uri("/api/v1/tickets")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "building_id": building_id.to_string(),
            "unit_id": unit_id.to_string(),
            "title": "Complete lifecycle test",
            "description": "Testing full ticket workflow from creation to closure",
            "category": "Plumbing",
            "priority": "Critical"
        }))
        .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    let ticket: serde_json::Value = test::read_body_json(create_resp).await;
    let ticket_id = ticket["id"].as_str().unwrap();
    assert_eq!(ticket["status"], "Open");
    assert_eq!(ticket["priority"], "Critical");

    // 2. Assign ticket
    let assign_req = test::TestRequest::put()
        .uri(&format!("/api/v1/tickets/{}/assign", ticket_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "assigned_to": user_id.to_string()
        }))
        .to_request();

    let assign_resp = test::call_service(&app, assign_req).await;
    let assigned: serde_json::Value = test::read_body_json(assign_resp).await;
    assert_eq!(assigned["status"], "InProgress");
    assert_eq!(assigned["assigned_to"], user_id.to_string());

    // 3. Resolve ticket
    let resolve_req = test::TestRequest::put()
        .uri(&format!("/api/v1/tickets/{}/resolve", ticket_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "resolution_notes": "Emergency plumbing fixed. Replaced burst pipe and checked water pressure. No further issues detected."
        }))
        .to_request();

    let resolve_resp = test::call_service(&app, resolve_req).await;
    let resolved: serde_json::Value = test::read_body_json(resolve_resp).await;
    assert_eq!(resolved["status"], "Resolved");
    assert!(resolved["resolved_at"].is_string());

    // 4. Close ticket
    let close_req = test::TestRequest::put()
        .uri(&format!("/api/v1/tickets/{}/close", ticket_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let close_resp = test::call_service(&app, close_req).await;
    let closed: serde_json::Value = test::read_body_json(close_resp).await;
    assert_eq!(closed["status"], "Closed");
    assert!(closed["closed_at"].is_string());

    // 5. Verify in building tickets list
    let list_req = test::TestRequest::get()
        .uri(&format!("/api/v1/buildings/{}/tickets", building_id))
        .to_request();

    let list_resp = test::call_service(&app, list_req).await;
    let tickets: serde_json::Value = test::read_body_json(list_resp).await;
    assert!(tickets.as_array().unwrap().len() >= 1);

    // 6. Verify in closed status list
    let closed_list_req = test::TestRequest::get()
        .uri(&format!(
            "/api/v1/buildings/{}/tickets/status/Closed",
            building_id
        ))
        .to_request();

    let closed_list_resp = test::call_service(&app, closed_list_req).await;
    let closed_tickets: serde_json::Value = test::read_body_json(closed_list_resp).await;
    assert!(closed_tickets.as_array().unwrap().len() >= 1);
}
