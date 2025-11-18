// E2E tests for poll/voting HTTP endpoints (Issue #51)
// Tests focus on HTTP layer: endpoints, auth, JSON serialization
// Covers board decision polls for Belgian copropriété management

use actix_web::http::header;
use actix_web::{test, App};
use chrono::{Duration, Utc};
use koprogo_api::application::dto::*;
use koprogo_api::application::use_cases::*;
use koprogo_api::domain::entities::{PollStatus, PollType};
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

/// Setup function shared across all poll E2E tests
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
    let unit_owner_repo = Arc::new(PostgresUnitOwnerRepository::new(pool.clone()));
    let expense_repo = Arc::new(PostgresExpenseRepository::new(pool.clone()));
    let meeting_repo = Arc::new(PostgresMeetingRepository::new(pool.clone()));
    let document_repo = Arc::new(PostgresDocumentRepository::new(pool.clone()));
    let poll_repo = Arc::new(PostgresPollRepository::new(pool.clone()));
    let poll_vote_repo = Arc::new(PostgresPollVoteRepository::new(pool.clone()));
    let gdpr_repo = Arc::new(PostgresGdprRepository::new(Arc::new(pool.clone())));
    let audit_log_repo = Arc::new(PostgresAuditLogRepository::new(pool.clone()));
    let charge_distribution_repo =
        Arc::new(PostgresChargeDistributionRepository::new(pool.clone()));
    let payment_reminder_repo = Arc::new(PostgresPaymentReminderRepository::new(pool.clone()));
    let board_member_repo = Arc::new(PostgresBoardMemberRepository::new(pool.clone()));
    let board_decision_repo = Arc::new(PostgresBoardDecisionRepository::new(pool.clone()));

    let audit_logger = AuditLogger::new(Some(audit_log_repo.clone()));

    // Initialize use cases
    let jwt_secret = "e2e-poll-secret".to_string();
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
    let document_use_cases = DocumentUseCases::new(document_repo);
    let poll_use_cases = PollUseCases::new(
        poll_repo.clone(),
        poll_vote_repo.clone(),
        owner_repo.clone(),
    );
    let gdpr_use_cases = GdprUseCases::new(gdpr_repo, user_repo.clone());
    let payment_reminder_use_cases = PaymentReminderUseCases::new(payment_reminder_repo);
    let pcn_use_cases = PcnUseCases::new();
    let etat_date_use_cases = EtatDateUseCases::new();
    let board_member_use_cases = BoardMemberUseCases::new(board_member_repo);
    let board_decision_use_cases = BoardDecisionUseCases::new(board_decision_repo);
    let board_dashboard_use_cases = BoardDashboardUseCases::new();

    let email_service = EmailService::from_env().expect("Failed to initialize email service");

    // Initialize storage service
    let storage = FileStorage::new("/tmp/koprogo-test-uploads".to_string());
    let storage_provider = Arc::new(storage) as Arc<dyn StorageProvider>;

    let app_state = AppState::new(
        account_use_cases,
        auth_use_cases,
        building_use_cases,
        unit_use_cases,
        owner_use_cases,
        unit_owner_use_cases,
        expense_use_cases,
        charge_distribution_use_cases,
        meeting_use_cases,
        poll_use_cases,
        document_use_cases,
        etat_date_use_cases,
        pcn_use_cases,
        payment_reminder_use_cases,
        gdpr_use_cases,
        board_member_use_cases,
        board_decision_use_cases,
        board_dashboard_use_cases,
        financial_report_use_cases,
        Arc::new(audit_logger),
        Arc::new(email_service),
        storage_provider,
        pool,
    );

    (actix_web::web::Data::new(app_state), postgres_container)
}

/// Create test user, building, and auth token
async fn create_test_auth(
    app_state: &actix_web::web::Data<AppState>,
) -> (Uuid, Uuid, String) {
    let org_id = Uuid::new_v4();

    // Create user
    let user_dto = CreateUserDto {
        email: "poll-test@example.com".to_string(),
        password: "password123".to_string(),
        first_name: "Poll".to_string(),
        last_name: "Tester".to_string(),
        organization_id: org_id.to_string(),
        role: "Syndic".to_string(),
    };

    let user = app_state
        .auth_use_cases
        .register(user_dto)
        .await
        .expect("Failed to create user");

    // Create building
    let building_dto = CreateBuildingDto {
        name: "Poll Test Building".to_string(),
        address: Some("123 Test St".to_string()),
        city: Some("Brussels".to_string()),
        postal_code: Some("1000".to_string()),
        country: Some("Belgium".to_string()),
        total_units: Some(10),
        organization_id: org_id.to_string(),
        construction_year: Some(2020),
        syndic_name: None,
        syndic_email: None,
        syndic_phone: None,
        syndic_address: None,
        syndic_office_hours: None,
        syndic_emergency_contact: None,
    };

    let building = app_state
        .building_use_cases
        .create_building(building_dto)
        .await
        .expect("Failed to create building");

    // Login to get token
    let login_dto = LoginDto {
        email: "poll-test@example.com".to_string(),
        password: "password123".to_string(),
    };

    let login_response = app_state
        .auth_use_cases
        .login(login_dto)
        .await
        .expect("Failed to login");

    (
        Uuid::parse_str(&building.id).unwrap(),
        user.id,
        login_response.token,
    )
}

#[actix_web::test]
#[serial]
async fn test_create_yesno_poll() {
    let (app_state, _container) = setup_app().await;
    let (building_id, _user_id, token) = create_test_auth(&app_state).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let tomorrow = Utc::now() + Duration::days(1);

    let create_dto = json!({
        "building_id": building_id.to_string(),
        "poll_type": "yes_no",
        "question": "Should we repaint the lobby?",
        "description": "Vote to decide if we should repaint the lobby in blue or keep it white",
        "starts_at": Utc::now().to_rfc3339(),
        "ends_at": tomorrow.to_rfc3339(),
        "is_anonymous": false,
    });

    let req = test::TestRequest::post()
        .uri("/api/v1/polls")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .insert_header((header::CONTENT_TYPE, "application/json"))
        .set_payload(create_dto.to_string())
        .to_request();

    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), 201, "Should create poll");

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["question"], "Should we repaint the lobby?");
    assert_eq!(body["poll_type"], "YesNo");
    assert_eq!(body["status"], "Draft");
}

#[actix_web::test]
#[serial]
async fn test_create_multiple_choice_poll() {
    let (app_state, _container) = setup_app().await;
    let (building_id, _user_id, token) = create_test_auth(&app_state).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let tomorrow = Utc::now() + Duration::days(1);

    let create_dto = json!({
        "building_id": building_id.to_string(),
        "poll_type": "multiple_choice",
        "question": "Which contractor should we hire for roof repairs?",
        "description": "Choose from 3 quotes received",
        "starts_at": Utc::now().to_rfc3339(),
        "ends_at": tomorrow.to_rfc3339(),
        "is_anonymous": false,
        "options": [
            {"option_text": "Contractor A - €15,000", "display_order": 1},
            {"option_text": "Contractor B - €18,500", "display_order": 2},
            {"option_text": "Contractor C - €14,200", "display_order": 3}
        ],
        "allow_multiple_votes": false
    });

    let req = test::TestRequest::post()
        .uri("/api/v1/polls")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .insert_header((header::CONTENT_TYPE, "application/json"))
        .set_payload(create_dto.to_string())
        .to_request();

    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), 201, "Should create multiple choice poll");

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(
        body["question"],
        "Which contractor should we hire for roof repairs?"
    );
    assert_eq!(body["poll_type"], "MultipleChoice");
    assert_eq!(body["options"].as_array().unwrap().len(), 3);
}

#[actix_web::test]
#[serial]
async fn test_publish_poll() {
    let (app_state, _container) = setup_app().await;
    let (building_id, _user_id, token) = create_test_auth(&app_state).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create poll
    let tomorrow = Utc::now() + Duration::days(1);
    let create_dto = json!({
        "building_id": building_id.to_string(),
        "poll_type": "yes_no",
        "question": "Test poll",
        "starts_at": Utc::now().to_rfc3339(),
        "ends_at": tomorrow.to_rfc3339(),
        "is_anonymous": false,
    });

    let req = test::TestRequest::post()
        .uri("/api/v1/polls")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .insert_header((header::CONTENT_TYPE, "application/json"))
        .set_payload(create_dto.to_string())
        .to_request();

    let resp = test::call_service(&app, req).await;
    let poll: serde_json::Value = test::read_body_json(resp).await;
    let poll_id = poll["id"].as_str().unwrap();

    // Publish poll
    let req = test::TestRequest::put()
        .uri(&format!("/api/v1/polls/{}/publish", poll_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), 200, "Should publish poll");

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["status"], "Active");
}

#[actix_web::test]
#[serial]
async fn test_cast_vote_yesno() {
    let (app_state, _container) = setup_app().await;
    let (building_id, user_id, token) = create_test_auth(&app_state).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create and publish poll
    let tomorrow = Utc::now() + Duration::days(1);
    let create_dto = json!({
        "building_id": building_id.to_string(),
        "poll_type": "yes_no",
        "question": "Approve budget?",
        "starts_at": Utc::now().to_rfc3339(),
        "ends_at": tomorrow.to_rfc3339(),
        "is_anonymous": false,
    });

    let req = test::TestRequest::post()
        .uri("/api/v1/polls")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .insert_header((header::CONTENT_TYPE, "application/json"))
        .set_payload(create_dto.to_string())
        .to_request();

    let resp = test::call_service(&app, req).await;
    let poll: serde_json::Value = test::read_body_json(resp).await;
    let poll_id = poll["id"].as_str().unwrap();

    // Publish
    let req = test::TestRequest::put()
        .uri(&format!("/api/v1/polls/{}/publish", poll_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();
    test::call_service(&app, req).await;

    // Cast vote
    let vote_dto = json!({
        "poll_id": poll_id,
        "selected_option_ids": [poll["options"][0]["id"]],
    });

    let req = test::TestRequest::post()
        .uri("/api/v1/polls/vote")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .insert_header((header::CONTENT_TYPE, "application/json"))
        .set_payload(vote_dto.to_string())
        .to_request();

    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), 201, "Should cast vote");
}

#[actix_web::test]
#[serial]
async fn test_get_poll_results() {
    let (app_state, _container) = setup_app().await;
    let (building_id, _user_id, token) = create_test_auth(&app_state).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create, publish, and close poll
    let tomorrow = Utc::now() + Duration::days(1);
    let create_dto = json!({
        "building_id": building_id.to_string(),
        "poll_type": "yes_no",
        "question": "Results test",
        "starts_at": Utc::now().to_rfc3339(),
        "ends_at": tomorrow.to_rfc3339(),
        "is_anonymous": false,
    });

    let req = test::TestRequest::post()
        .uri("/api/v1/polls")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .insert_header((header::CONTENT_TYPE, "application/json"))
        .set_payload(create_dto.to_string())
        .to_request();

    let resp = test::call_service(&app, req).await;
    let poll: serde_json::Value = test::read_body_json(resp).await;
    let poll_id = poll["id"].as_str().unwrap();

    // Get results
    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/polls/{}/results", poll_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), 200, "Should get results");

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["poll_id"], poll_id);
    assert_eq!(body["total_votes_cast"], 0);
}

#[actix_web::test]
#[serial]
async fn test_close_poll() {
    let (app_state, _container) = setup_app().await;
    let (building_id, _user_id, token) = create_test_auth(&app_state).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create and publish poll
    let tomorrow = Utc::now() + Duration::days(1);
    let create_dto = json!({
        "building_id": building_id.to_string(),
        "poll_type": "yes_no",
        "question": "Close test",
        "starts_at": Utc::now().to_rfc3339(),
        "ends_at": tomorrow.to_rfc3339(),
        "is_anonymous": false,
    });

    let req = test::TestRequest::post()
        .uri("/api/v1/polls")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .insert_header((header::CONTENT_TYPE, "application/json"))
        .set_payload(create_dto.to_string())
        .to_request();

    let resp = test::call_service(&app, req).await;
    let poll: serde_json::Value = test::read_body_json(resp).await;
    let poll_id = poll["id"].as_str().unwrap();

    // Publish
    let req = test::TestRequest::put()
        .uri(&format!("/api/v1/polls/{}/publish", poll_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();
    test::call_service(&app, req).await;

    // Close poll
    let req = test::TestRequest::put()
        .uri(&format!("/api/v1/polls/{}/close", poll_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), 200, "Should close poll");

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["status"], "Closed");
}

#[actix_web::test]
#[serial]
async fn test_list_active_polls() {
    let (app_state, _container) = setup_app().await;
    let (building_id, _user_id, token) = create_test_auth(&app_state).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create and publish poll
    let tomorrow = Utc::now() + Duration::days(1);
    let create_dto = json!({
        "building_id": building_id.to_string(),
        "poll_type": "yes_no",
        "question": "Active test",
        "starts_at": Utc::now().to_rfc3339(),
        "ends_at": tomorrow.to_rfc3339(),
        "is_anonymous": false,
    });

    let req = test::TestRequest::post()
        .uri("/api/v1/polls")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .insert_header((header::CONTENT_TYPE, "application/json"))
        .set_payload(create_dto.to_string())
        .to_request();

    let resp = test::call_service(&app, req).await;
    let poll: serde_json::Value = test::read_body_json(resp).await;
    let poll_id = poll["id"].as_str().unwrap();

    // Publish
    let req = test::TestRequest::put()
        .uri(&format!("/api/v1/polls/{}/publish", poll_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();
    test::call_service(&app, req).await;

    // List active polls
    let req = test::TestRequest::get()
        .uri("/api/v1/polls/active")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), 200, "Should list active polls");

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body.is_array());
    assert!(body.as_array().unwrap().len() >= 1);
}

#[actix_web::test]
#[serial]
async fn test_building_poll_statistics() {
    let (app_state, _container) = setup_app().await;
    let (building_id, _user_id, token) = create_test_auth(&app_state).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Get statistics
    let req = test::TestRequest::get()
        .uri(&format!(
            "/api/v1/buildings/{}/polls/statistics",
            building_id
        ))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), 200, "Should get statistics");

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body["total_polls"].is_number());
    assert!(body["active_polls"].is_number());
    assert!(body["closed_polls"].is_number());
    assert!(body["average_participation_rate"].is_number());
}
