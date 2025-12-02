// E2E tests for meeting HTTP endpoints (Issue #75)
// Tests focus on HTTP layer: endpoints, auth, JSON serialization
// BDD tests (meetings.feature, meetings_manage.feature) cover business scenarios

use actix_web::http::header;
use actix_web::{test, App};
use chrono::{Duration, Utc};
use koprogo_api::application::dto::*;
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

/// Setup function shared across all meeting E2E tests
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
    let gdpr_repo = Arc::new(PostgresGdprRepository::new(Arc::new(pool.clone())));
    let audit_log_repo = Arc::new(PostgresAuditLogRepository::new(pool.clone()));
    let charge_distribution_repo =
        Arc::new(PostgresChargeDistributionRepository::new(pool.clone()));
    let payment_reminder_repo = Arc::new(PostgresPaymentReminderRepository::new(pool.clone()));
    let board_member_repo = Arc::new(PostgresBoardMemberRepository::new(pool.clone()));
    let board_decision_repo = Arc::new(PostgresBoardDecisionRepository::new(pool.clone()));

    let audit_logger = AuditLogger::new(Some(audit_log_repo.clone()));

    // Initialize use cases
    let jwt_secret = "e2e-meeting-secret".to_string();
    let account_repo = Arc::new(PostgresAccountRepository::new(pool.clone()));
    let account_use_cases = AccountUseCases::new(account_repo.clone());
    let financial_report_use_cases =
        FinancialReportUseCases::new(account_repo, expense_repo.clone());

    let auth_use_cases =
        AuthUseCases::new(user_repo.clone(), refresh_repo, user_role_repo, jwt_secret);
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
    let storage_root = std::env::temp_dir().join("koprogo_e2e_meetings_uploads");
    let storage: Arc<dyn StorageProvider> =
        Arc::new(FileStorage::new(&storage_root).expect("storage"));
    let document_use_cases = DocumentUseCases::new(document_repo, storage.clone());
    let pcn_use_cases = PcnUseCases::new(expense_repo.clone());
    let payment_reminder_use_cases =
        PaymentReminderUseCases::new(payment_reminder_repo, expense_repo);
    let gdpr_use_cases = GdprUseCases::new(gdpr_repo);
    let board_member_use_cases =
        BoardMemberUseCases::new(board_member_repo.clone(), building_repo.clone());
    let board_decision_use_cases = BoardDecisionUseCases::new(
        board_decision_repo.clone(),
        building_repo.clone(),
        meeting_repo.clone(),
    );
    let board_dashboard_use_cases = BoardDashboardUseCases::new(
        board_member_repo.clone(),
        board_decision_repo.clone(),
        building_repo.clone(),
    );

    let app_state = actix_web::web::Data::new(AppState::new(
        account_use_cases,
        auth_use_cases,
        building_use_cases,
        unit_use_cases,
        owner_use_cases,
        unit_owner_use_cases,
        expense_use_cases,
        charge_distribution_use_cases,
        meeting_use_cases,
        document_use_cases,
        pcn_use_cases,
        payment_reminder_use_cases,
        gdpr_use_cases,
        board_member_use_cases,
        board_decision_use_cases,
        board_dashboard_use_cases,
        financial_report_use_cases,
        audit_logger,
        EmailService::from_env().expect("email service"),
        pool.clone(),
    ));

    (app_state, postgres_container)
}

/// Helper: Create organization, user, and building for tests
async fn create_test_fixtures(app_state: &actix_web::web::Data<AppState>) -> (String, Uuid, Uuid) {
    let pool = &app_state.pool;

    // Create organization
    let org_id = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO organizations (id, name, slug, contact_email, subscription_plan, max_buildings, max_users, is_active, created_at, updated_at)
           VALUES ($1, 'E2E Org', 'e2e-org', 'e2e@org.com', 'starter', 10, 10, true, NOW(), NOW())"#
    )
    .bind(org_id)
    .execute(pool)
    .await
    .expect("Failed to insert organization");

    // Register user
    let email = format!("e2e+{}@test.com", Uuid::new_v4());
    let reg = RegisterRequest {
        email: email.clone(),
        password: "Passw0rd!".to_string(),
        first_name: "E2E".to_string(),
        last_name: "User".to_string(),
        role: "syndic".to_string(),
        organization_id: Some(org_id),
    };

    let _register_result = app_state
        .auth_use_cases
        .register(reg)
        .await
        .expect("Failed to register user");

    // Login to get JWT token
    let login = LoginRequest {
        email: email.clone(),
        password: "Passw0rd!".to_string(),
    };

    let login_result = app_state
        .auth_use_cases
        .login(login)
        .await
        .expect("Failed to login");

    let token = login_result.access_token;

    // Create building using DTO
    let building_dto = CreateBuildingDto {
        organization_id: org_id,
        name: "Test Building".to_string(),
        address: "123 Main St".to_string(),
        city: "Brussels".to_string(),
        postal_code: "1000".to_string(),
        country: "Belgium".to_string(),
        total_units: 10,
        construction_year: Some(2020),
    };

    let building = app_state
        .building_use_cases
        .create_building(building_dto)
        .await
        .expect("Failed to create building");

    let building_id = Uuid::parse_str(&building.id).expect("Invalid building ID");

    (token, org_id, building_id)
}

//
// ═══════════════════════════════════════════════════════════════════════════
// TEST: POST /meetings (Create meeting)
// ═══════════════════════════════════════════════════════════════════════════
//

#[actix_web::test]
#[serial]
async fn test_create_meeting_success() {
    let (app_state, _container) = setup_app().await;
    let (token, org_id, building_id) = create_test_fixtures(&app_state).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let scheduled_date = Utc::now() + Duration::days(30);

    let req = test::TestRequest::post()
        .uri("/api/v1/meetings")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "organization_id": org_id.to_string(),
            "building_id": building_id.to_string(),
            "meeting_type": "Ordinary",
            "title": "AG Ordinaire 2025",
            "description": "Assemblée générale ordinaire annuelle",
            "scheduled_date": scheduled_date.to_rfc3339(),
            "location": "Salle communautaire"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201, "Should create meeting successfully");

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["building_id"], building_id.to_string());
    assert_eq!(body["title"], "AG Ordinaire 2025");
    assert_eq!(body["meeting_type"], "Ordinary");
    assert_eq!(body["status"], "Planned");
    assert!(body["agenda"].is_array());
}

#[actix_web::test]
#[serial]
async fn test_create_meeting_without_auth_fails() {
    let (app_state, _container) = setup_app().await;
    let (_token, org_id, building_id) = create_test_fixtures(&app_state).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let scheduled_date = Utc::now() + Duration::days(30);

    let req = test::TestRequest::post()
        .uri("/api/v1/meetings")
        .set_json(json!({
            "organization_id": org_id.to_string(),
            "building_id": building_id.to_string(),
            "meeting_type": "Ordinary",
            "title": "Unauthorized Meeting",
            "scheduled_date": scheduled_date.to_rfc3339(),
            "location": "Somewhere"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401, "Should reject unauthorized request");
}

//
// ═══════════════════════════════════════════════════════════════════════════
// TEST: GET /meetings/:id (Get meeting details)
// ═══════════════════════════════════════════════════════════════════════════
//

#[actix_web::test]
#[serial]
async fn test_get_meeting_by_id_success() {
    let (app_state, _container) = setup_app().await;
    let (token, org_id, building_id) = create_test_fixtures(&app_state).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create a meeting first
    let scheduled_date = Utc::now() + Duration::days(30);

    let req = test::TestRequest::post()
        .uri("/api/v1/meetings")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "organization_id": org_id.to_string(),
            "building_id": building_id.to_string(),
            "meeting_type": "Ordinary",
            "title": "Test Meeting",
            "scheduled_date": scheduled_date.to_rfc3339(),
            "location": "Test Location"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    let created: serde_json::Value = test::read_body_json(resp).await;
    let meeting_id = created["id"].as_str().unwrap();

    // Get the meeting
    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/meetings/{}", meeting_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["id"], meeting_id);
    assert_eq!(body["title"], "Test Meeting");
}

#[actix_web::test]
#[serial]
async fn test_get_meeting_not_found() {
    let (app_state, _container) = setup_app().await;
    let (token, _org_id, _building_id) = create_test_fixtures(&app_state).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let fake_id = Uuid::new_v4();

    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/meetings/{}", fake_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.status(),
        404,
        "Should return 404 for non-existent meeting"
    );
}

//
// ═══════════════════════════════════════════════════════════════════════════
// TEST: GET /meetings (List all meetings - paginated)
// ═══════════════════════════════════════════════════════════════════════════
//

#[actix_web::test]
#[serial]
async fn test_list_meetings_paginated() {
    let (app_state, _container) = setup_app().await;
    let (token, org_id, building_id) = create_test_fixtures(&app_state).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create 3 meetings
    for i in 1..=3 {
        let scheduled_date = Utc::now() + Duration::days(30 + i);

        let req = test::TestRequest::post()
            .uri("/api/v1/meetings")
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
            .set_json(json!({
                "organization_id": org_id.to_string(),
                "building_id": building_id.to_string(),
                "meeting_type": "Ordinary",
                "title": format!("Meeting {}", i),
                "scheduled_date": scheduled_date.to_rfc3339(),
                "location": "Location"
            }))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 201);
    }

    // List meetings with pagination
    let req = test::TestRequest::get()
        .uri("/api/v1/meetings?page=1&per_page=10")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body["data"].is_array());
    assert!(body["data"].as_array().unwrap().len() >= 3);
    assert_eq!(body["page"], 1);
    assert_eq!(body["per_page"], 10);
}

//
// ═══════════════════════════════════════════════════════════════════════════
// TEST: GET /buildings/:id/meetings (List meetings by building)
// ═══════════════════════════════════════════════════════════════════════════
//

#[actix_web::test]
#[serial]
async fn test_list_meetings_by_building() {
    let (app_state, _container) = setup_app().await;
    let (token, org_id, building_id) = create_test_fixtures(&app_state).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create 2 meetings for this building
    for i in 1..=2 {
        let scheduled_date = Utc::now() + Duration::days(30 + i);

        let req = test::TestRequest::post()
            .uri("/api/v1/meetings")
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
            .set_json(json!({
                "organization_id": org_id.to_string(),
                "building_id": building_id.to_string(),
                "meeting_type": "Ordinary",
                "title": format!("Building Meeting {}", i),
                "scheduled_date": scheduled_date.to_rfc3339(),
                "location": "Building Hall"
            }))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 201);
    }

    // List meetings for building
    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/buildings/{}/meetings", building_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body.is_array());
    assert!(body.as_array().unwrap().len() >= 2);

    // Verify all meetings belong to this building
    for meeting in body.as_array().unwrap() {
        assert_eq!(meeting["building_id"], building_id.to_string());
    }
}

//
// ═══════════════════════════════════════════════════════════════════════════
// TEST: PUT /meetings/:id (Update meeting)
// ═══════════════════════════════════════════════════════════════════════════
//

#[actix_web::test]
#[serial]
async fn test_update_meeting_success() {
    let (app_state, _container) = setup_app().await;
    let (token, org_id, building_id) = create_test_fixtures(&app_state).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create meeting
    let scheduled_date = Utc::now() + Duration::days(30);

    let req = test::TestRequest::post()
        .uri("/api/v1/meetings")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "organization_id": org_id.to_string(),
            "building_id": building_id.to_string(),
            "meeting_type": "Ordinary",
            "title": "Original Title",
            "scheduled_date": scheduled_date.to_rfc3339(),
            "location": "Original Location"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    let created: serde_json::Value = test::read_body_json(resp).await;
    let meeting_id = created["id"].as_str().unwrap();

    // Update meeting
    let req = test::TestRequest::put()
        .uri(&format!("/api/v1/meetings/{}", meeting_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "title": "Updated Title",
            "location": "Updated Location"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200, "Should update meeting successfully");

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["title"], "Updated Title");
    assert_eq!(body["location"], "Updated Location");
}

//
// ═══════════════════════════════════════════════════════════════════════════
// TEST: POST /meetings/:id/agenda (Add agenda item)
// ═══════════════════════════════════════════════════════════════════════════
//

#[actix_web::test]
#[serial]
async fn test_add_agenda_item_success() {
    let (app_state, _container) = setup_app().await;
    let (token, org_id, building_id) = create_test_fixtures(&app_state).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create meeting
    let scheduled_date = Utc::now() + Duration::days(30);

    let req = test::TestRequest::post()
        .uri("/api/v1/meetings")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "organization_id": org_id.to_string(),
            "building_id": building_id.to_string(),
            "meeting_type": "Ordinary",
            "title": "Meeting with Agenda",
            "scheduled_date": scheduled_date.to_rfc3339(),
            "location": "Hall"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    let created: serde_json::Value = test::read_body_json(resp).await;
    let meeting_id = created["id"].as_str().unwrap();

    // Add agenda item
    let req = test::TestRequest::post()
        .uri(&format!("/api/v1/meetings/{}/agenda", meeting_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "item": "1. Approbation du budget"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200, "Should add agenda item successfully");

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body["agenda"].is_array());
    assert!(body["agenda"].as_array().unwrap().len() >= 1);
    assert!(body["agenda"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item.as_str().unwrap().contains("Approbation du budget")));
}

//
// ═══════════════════════════════════════════════════════════════════════════
// TEST: POST /meetings/:id/complete (Complete meeting)
// ═══════════════════════════════════════════════════════════════════════════
//

#[actix_web::test]
#[serial]
async fn test_complete_meeting_success() {
    let (app_state, _container) = setup_app().await;
    let (token, org_id, building_id) = create_test_fixtures(&app_state).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create meeting
    let scheduled_date = Utc::now() + Duration::days(30);

    let req = test::TestRequest::post()
        .uri("/api/v1/meetings")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "organization_id": org_id.to_string(),
            "building_id": building_id.to_string(),
            "meeting_type": "Ordinary",
            "title": "Meeting to Complete",
            "scheduled_date": scheduled_date.to_rfc3339(),
            "location": "Hall"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    let created: serde_json::Value = test::read_body_json(resp).await;
    let meeting_id = created["id"].as_str().unwrap();

    // Complete meeting
    let req = test::TestRequest::post()
        .uri(&format!("/api/v1/meetings/{}/complete", meeting_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "attendees_count": 42
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200, "Should complete meeting successfully");

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["status"], "Completed");
    assert_eq!(body["attendees_count"], 42);
}

//
// ═══════════════════════════════════════════════════════════════════════════
// TEST: POST /meetings/:id/cancel (Cancel meeting)
// ═══════════════════════════════════════════════════════════════════════════
//

#[actix_web::test]
#[serial]
async fn test_cancel_meeting_success() {
    let (app_state, _container) = setup_app().await;
    let (token, org_id, building_id) = create_test_fixtures(&app_state).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create meeting
    let scheduled_date = Utc::now() + Duration::days(30);

    let req = test::TestRequest::post()
        .uri("/api/v1/meetings")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "organization_id": org_id.to_string(),
            "building_id": building_id.to_string(),
            "meeting_type": "Ordinary",
            "title": "Meeting to Cancel",
            "scheduled_date": scheduled_date.to_rfc3339(),
            "location": "Hall"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    let created: serde_json::Value = test::read_body_json(resp).await;
    let meeting_id = created["id"].as_str().unwrap();

    // Cancel meeting
    let req = test::TestRequest::post()
        .uri(&format!("/api/v1/meetings/{}/cancel", meeting_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200, "Should cancel meeting successfully");

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["status"], "Cancelled");
}

//
// ═══════════════════════════════════════════════════════════════════════════
// TEST: POST /meetings/:id/reschedule (Reschedule meeting)
// ═══════════════════════════════════════════════════════════════════════════
//

#[actix_web::test]
#[serial]
async fn test_reschedule_meeting_success() {
    let (app_state, _container) = setup_app().await;
    let (token, org_id, building_id) = create_test_fixtures(&app_state).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create meeting
    let original_date = Utc::now() + Duration::days(30);

    let req = test::TestRequest::post()
        .uri("/api/v1/meetings")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "organization_id": org_id.to_string(),
            "building_id": building_id.to_string(),
            "meeting_type": "Ordinary",
            "title": "Meeting to Reschedule",
            "scheduled_date": original_date.to_rfc3339(),
            "location": "Hall"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    let created: serde_json::Value = test::read_body_json(resp).await;
    let meeting_id = created["id"].as_str().unwrap();

    // Reschedule meeting
    let new_date = Utc::now() + Duration::days(45);

    let req = test::TestRequest::post()
        .uri(&format!("/api/v1/meetings/{}/reschedule", meeting_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "scheduled_date": new_date.to_rfc3339()
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200, "Should reschedule meeting successfully");

    let body: serde_json::Value = test::read_body_json(resp).await;
    // Verify new date is different from original
    assert_ne!(
        body["scheduled_date"],
        original_date.to_rfc3339(),
        "Scheduled date should be updated"
    );
}

//
// ═══════════════════════════════════════════════════════════════════════════
// TEST: DELETE /meetings/:id (Delete meeting)
// ═══════════════════════════════════════════════════════════════════════════
//

#[actix_web::test]
#[serial]
async fn test_delete_meeting_success() {
    let (app_state, _container) = setup_app().await;
    let (token, org_id, building_id) = create_test_fixtures(&app_state).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create meeting
    let scheduled_date = Utc::now() + Duration::days(30);

    let req = test::TestRequest::post()
        .uri("/api/v1/meetings")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "organization_id": org_id.to_string(),
            "building_id": building_id.to_string(),
            "meeting_type": "Ordinary",
            "title": "Meeting to Delete",
            "scheduled_date": scheduled_date.to_rfc3339(),
            "location": "Hall"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    let created: serde_json::Value = test::read_body_json(resp).await;
    let meeting_id = created["id"].as_str().unwrap();

    // Delete meeting
    let req = test::TestRequest::delete()
        .uri(&format!("/api/v1/meetings/{}", meeting_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 204, "Should delete meeting successfully");

    // Verify meeting is deleted
    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/meetings/{}", meeting_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.status(),
        404,
        "Deleted meeting should no longer be found"
    );
}

//
// ═══════════════════════════════════════════════════════════════════════════
// TEST: Complete meeting lifecycle workflow
// ═══════════════════════════════════════════════════════════════════════════
//

#[actix_web::test]
#[serial]
async fn test_meeting_complete_lifecycle() {
    let (app_state, _container) = setup_app().await;
    let (token, org_id, building_id) = create_test_fixtures(&app_state).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let scheduled_date = Utc::now() + Duration::days(30);

    // 1. Create meeting
    let req = test::TestRequest::post()
        .uri("/api/v1/meetings")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "organization_id": org_id.to_string(),
            "building_id": building_id.to_string(),
            "meeting_type": "Ordinary",
            "title": "Lifecycle Test Meeting",
            "description": "Testing complete lifecycle",
            "scheduled_date": scheduled_date.to_rfc3339(),
            "location": "Community Hall"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    let created: serde_json::Value = test::read_body_json(resp).await;
    let meeting_id = created["id"].as_str().unwrap();
    assert_eq!(created["status"], "Planned");

    // 2. Add agenda items
    let req = test::TestRequest::post()
        .uri(&format!("/api/v1/meetings/{}/agenda", meeting_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "item": "1. Approbation des comptes"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let req = test::TestRequest::post()
        .uri(&format!("/api/v1/meetings/{}/agenda", meeting_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "item": "2. Vote du budget"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["agenda"].as_array().unwrap().len(), 2);

    // 3. Update meeting details
    let req = test::TestRequest::put()
        .uri(&format!("/api/v1/meetings/{}", meeting_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "title": "Lifecycle Test Meeting - Updated",
            "location": "Updated Hall"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    // 4. Complete meeting
    let req = test::TestRequest::post()
        .uri(&format!("/api/v1/meetings/{}/complete", meeting_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "attendees_count": 35
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let completed: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(completed["status"], "Completed");
    assert_eq!(completed["attendees_count"], 35);
    assert_eq!(
        completed["title"], "Lifecycle Test Meeting - Updated",
        "Title should remain updated"
    );
    assert_eq!(
        completed["location"], "Updated Hall",
        "Location should remain updated"
    );
}
