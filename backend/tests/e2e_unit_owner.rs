// E2E tests for unit_owner HTTP endpoints (Issue #32)
// Tests focus on HTTP layer: endpoints, auth, JSON serialization
// Integration tests (integration_unit_owner.rs) cover business logic

use actix_web::http::header;
use actix_web::{test, App};
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

/// Setup function shared across all unit_owner E2E tests
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
    let jwt_secret = "e2e-unit-owner-secret".to_string();
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
    let storage_root = std::env::temp_dir().join("koprogo_e2e_unit_owner_uploads");
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

/// Helper: Create organization, user, building, unit, and owner for tests
async fn create_test_fixtures(
    app_state: &actix_web::web::Data<AppState>,
) -> (String, Uuid, Uuid, Uuid, Uuid) {
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

    let register_result = app_state
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

    // Create unit using DTO
    let unit_dto = CreateUnitDto {
        organization_id: org_id,
        building_id,
        unit_number: "101".to_string(),
        unit_type: "Apartment".to_string(),
        floor: Some(1),
        surface_area: 75.5,
        tax_value: 100.0,
    };

    let unit = app_state
        .unit_use_cases
        .create_unit(unit_dto)
        .await
        .expect("Failed to create unit");

    let unit_id = Uuid::parse_str(&unit.id).expect("Invalid unit ID");

    // Create owner using DTO
    let owner_dto = CreateOwnerDto {
        organization_id: org_id,
        first_name: "John".to_string(),
        last_name: "Doe".to_string(),
        email: "john.doe@example.com".to_string(),
        phone: Some("+32123456789".to_string()),
        address: Some("456 Oak St".to_string()),
        city: Some("Brussels".to_string()),
        postal_code: Some("1000".to_string()),
        country: Some("Belgium".to_string()),
        language: Some("fr".to_string()),
    };

    let owner = app_state
        .owner_use_cases
        .create_owner(owner_dto)
        .await
        .expect("Failed to create owner");

    let owner_id = Uuid::parse_str(&owner.id).expect("Invalid owner ID");

    (token, org_id, building_id, unit_id, owner_id)
}

//
// ═══════════════════════════════════════════════════════════════════════════
// TEST: POST /units/:unit_id/owners (Add owner to unit)
// ═══════════════════════════════════════════════════════════════════════════
//

#[actix_web::test]
#[serial]
async fn test_add_owner_to_unit_success() {
    let (app_state, _container) = setup_app().await;
    let (token, _org_id, _building_id, unit_id, owner_id) = create_test_fixtures(&app_state).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Add owner with 100% ownership
    let req = test::TestRequest::post()
        .uri(&format!("/api/v1/units/{}/owners", unit_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "owner_id": owner_id.to_string(),
            "ownership_percentage": 1.0,
            "is_primary_contact": true
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201, "Should create unit-owner relationship");

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["unit_id"], unit_id.to_string());
    assert_eq!(body["owner_id"], owner_id.to_string());
    assert_eq!(body["ownership_percentage"], 1.0);
    assert_eq!(body["is_primary_contact"], true);
    assert_eq!(body["is_active"], true);
}

#[actix_web::test]
#[serial]
async fn test_add_owner_to_unit_exceeds_100_percent() {
    let (app_state, _container) = setup_app().await;
    let (token, _org_id, _building_id, unit_id, owner_id) = create_test_fixtures(&app_state).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Add first owner with 60%
    let req = test::TestRequest::post()
        .uri(&format!("/api/v1/units/{}/owners", unit_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "owner_id": owner_id.to_string(),
            "ownership_percentage": 0.6,
            "is_primary_contact": true
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    // Get organization_id from database
    let org_id: Uuid = sqlx::query_scalar("SELECT id FROM organizations LIMIT 1")
        .fetch_one(&app_state.pool)
        .await
        .expect("Failed to get organization ID");

    // Create second owner
    let owner_dto2 = CreateOwnerDto {
        organization_id: org_id,
        first_name: "Jane".to_string(),
        last_name: "Smith".to_string(),
        email: "jane.smith@example.com".to_string(),
        phone: Some("+32987654321".to_string()),
        address: None,
        city: None,
        postal_code: None,
        country: None,
        language: Some("en".to_string()),
    };

    let owner2 = app_state
        .owner_use_cases
        .create_owner(owner_dto2)
        .await
        .expect("Failed to create second owner");

    let owner2_id = Uuid::parse_str(&owner2.id).expect("Invalid owner2 ID");

    // Try to add second owner with 50% (total would be 110%)
    let req = test::TestRequest::post()
        .uri(&format!("/api/v1/units/{}/owners", unit_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "owner_id": owner2_id.to_string(),
            "ownership_percentage": 0.5,
            "is_primary_contact": false
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.status(),
        400,
        "Should fail when total ownership exceeds 100%"
    );

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(
        body["error"].as_str().unwrap().contains("exceed 100%"),
        "Error message should mention exceeding 100%"
    );
}

//
// ═══════════════════════════════════════════════════════════════════════════
// TEST: GET /units/:unit_id/owners (List owners for unit)
// ═══════════════════════════════════════════════════════════════════════════
//

#[actix_web::test]
#[serial]
async fn test_list_owners_for_unit_success() {
    let (app_state, _container) = setup_app().await;
    let (token, _org_id, _building_id, unit_id, owner_id) = create_test_fixtures(&app_state).await;

    // Add owner first
    let req = test::TestRequest::post()
        .uri(&format!("/api/v1/units/{}/owners", unit_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "owner_id": owner_id.to_string(),
            "ownership_percentage": 1.0,
            "is_primary_contact": true
        }))
        .to_request();

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    // List owners
    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/units/{}/owners", unit_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["unit_id"], unit_id.to_string());
    assert!(body["owners"].is_array());
    assert_eq!(body["owners"].as_array().unwrap().len(), 1);
    assert_eq!(body["total_ownership_percentage"], 1.0);
}

//
// ═══════════════════════════════════════════════════════════════════════════
// TEST: DELETE /units/:unit_id/owners/:owner_id (Remove owner from unit)
// ═══════════════════════════════════════════════════════════════════════════
//

#[actix_web::test]
#[serial]
async fn test_remove_owner_from_unit_success() {
    let (app_state, _container) = setup_app().await;
    let (token, _org_id, _building_id, unit_id, owner_id) = create_test_fixtures(&app_state).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Add owner
    let req = test::TestRequest::post()
        .uri(&format!("/api/v1/units/{}/owners", unit_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "owner_id": owner_id.to_string(),
            "ownership_percentage": 1.0,
            "is_primary_contact": true
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    // Remove owner
    let req = test::TestRequest::delete()
        .uri(&format!("/api/v1/units/{}/owners/{}", unit_id, owner_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200, "Should remove owner successfully");

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(
        body["is_active"], false,
        "Ownership should be marked inactive"
    );
    assert!(body["end_date"].is_string(), "end_date should be set");
}

//
// ═══════════════════════════════════════════════════════════════════════════
// TEST: PUT /unit-owners/:id (Update ownership details)
// ═══════════════════════════════════════════════════════════════════════════
//

#[actix_web::test]
#[serial]
async fn test_update_unit_owner_percentage_success() {
    let (app_state, _container) = setup_app().await;
    let (token, _org_id, _building_id, unit_id, owner_id) = create_test_fixtures(&app_state).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Add owner with 100%
    let req = test::TestRequest::post()
        .uri(&format!("/api/v1/units/{}/owners", unit_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "owner_id": owner_id.to_string(),
            "ownership_percentage": 1.0,
            "is_primary_contact": true
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    let created: serde_json::Value = test::read_body_json(resp).await;
    let relationship_id = created["id"].as_str().unwrap();

    // Update to 80%
    let req = test::TestRequest::put()
        .uri(&format!("/api/v1/unit-owners/{}", relationship_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "ownership_percentage": 0.8,
            "is_primary_contact": true
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200, "Should update ownership percentage");

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["ownership_percentage"], 0.8);
}

//
// ═══════════════════════════════════════════════════════════════════════════
// TEST: GET /owners/:owner_id/units (Get owner's units)
// ═══════════════════════════════════════════════════════════════════════════
//

#[actix_web::test]
#[serial]
async fn test_get_owner_units_success() {
    let (app_state, _container) = setup_app().await;
    let (token, _org_id, _building_id, unit_id, owner_id) = create_test_fixtures(&app_state).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Add owner to unit
    let req = test::TestRequest::post()
        .uri(&format!("/api/v1/units/{}/owners", unit_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "owner_id": owner_id.to_string(),
            "ownership_percentage": 1.0,
            "is_primary_contact": true
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    // Get owner's units
    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/owners/{}/units", owner_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["owner_id"], owner_id.to_string());
    assert!(body["units"].is_array());
    assert_eq!(body["units"].as_array().unwrap().len(), 1);
}

//
// ═══════════════════════════════════════════════════════════════════════════
// TEST: GET /units/:unit_id/ownership-history (Get ownership history)
// ═══════════════════════════════════════════════════════════════════════════
//

#[actix_web::test]
#[serial]
async fn test_get_unit_ownership_history_success() {
    let (app_state, _container) = setup_app().await;
    let (token, _org_id, _building_id, unit_id, owner_id) = create_test_fixtures(&app_state).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Add owner
    let req = test::TestRequest::post()
        .uri(&format!("/api/v1/units/{}/owners", unit_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "owner_id": owner_id.to_string(),
            "ownership_percentage": 1.0,
            "is_primary_contact": true
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    // Remove owner (creates history)
    let req = test::TestRequest::delete()
        .uri(&format!("/api/v1/units/{}/owners/{}", unit_id, owner_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    // Get history
    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/units/{}/owners/history", unit_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body.is_array());
    assert!(body.as_array().unwrap().len() > 0, "Should have history");
}

//
// ═══════════════════════════════════════════════════════════════════════════
// TEST: POST /units/:unit_id/owners/transfer (Transfer ownership)
// ═══════════════════════════════════════════════════════════════════════════
//

#[actix_web::test]
#[serial]
async fn test_transfer_ownership_success() {
    let (app_state, _container) = setup_app().await;
    let (token, org_id, _building_id, unit_id, owner1_id) = create_test_fixtures(&app_state).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Add first owner with 100%
    let req = test::TestRequest::post()
        .uri(&format!("/api/v1/units/{}/owners", unit_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "owner_id": owner1_id.to_string(),
            "ownership_percentage": 1.0,
            "is_primary_contact": true
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    // Create second owner
    let owner_dto2 = CreateOwnerDto {
        organization_id: org_id,
        first_name: "Jane".to_string(),
        last_name: "Smith".to_string(),
        email: "jane.transfer@example.com".to_string(),
        phone: Some("+32111111111".to_string()),
        address: None,
        city: None,
        postal_code: None,
        country: None,
        language: Some("en".to_string()),
    };

    let owner2 = app_state
        .owner_use_cases
        .create_owner(owner_dto2)
        .await
        .expect("Failed to create second owner");

    let owner2_id = Uuid::parse_str(&owner2.id).expect("Invalid owner2 ID");

    // Transfer ownership
    let req = test::TestRequest::post()
        .uri(&format!("/api/v1/units/{}/owners/transfer", unit_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "from_owner_id": owner1_id.to_string(),
            "to_owner_id": owner2_id.to_string()
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200, "Should transfer ownership successfully");

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["owner_id"], owner2_id.to_string());
    assert_eq!(body["ownership_percentage"], 1.0);
    assert_eq!(body["is_active"], true);
}

//
// ═══════════════════════════════════════════════════════════════════════════
// TEST: GET /units/:unit_id/owners/total-percentage (Get total ownership %)
// ═══════════════════════════════════════════════════════════════════════════
//

#[actix_web::test]
#[serial]
async fn test_get_total_ownership_percentage_success() {
    let (app_state, _container) = setup_app().await;
    let (token, _org_id, _building_id, unit_id, owner_id) = create_test_fixtures(&app_state).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Add owner with 60%
    let req = test::TestRequest::post()
        .uri(&format!("/api/v1/units/{}/owners", unit_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "owner_id": owner_id.to_string(),
            "ownership_percentage": 0.6,
            "is_primary_contact": true
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    // Get total percentage
    let req = test::TestRequest::get()
        .uri(&format!(
            "/api/v1/units/{}/owners/total-percentage",
            unit_id
        ))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["total_percentage"], 0.6);
}

//
// ═══════════════════════════════════════════════════════════════════════════
// TEST: Authentication & Authorization
// ═══════════════════════════════════════════════════════════════════════════
//

#[actix_web::test]
#[serial]
async fn test_unit_owner_endpoints_require_auth() {
    let (app_state, _container) = setup_app().await;
    let (_, _org_id, _building_id, unit_id, owner_id) = create_test_fixtures(&app_state).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Try POST without auth
    let req = test::TestRequest::post()
        .uri(&format!("/api/v1/units/{}/owners", unit_id))
        .set_json(json!({
            "owner_id": owner_id.to_string(),
            "ownership_percentage": 1.0,
            "is_primary_contact": true
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401, "Should reject unauthorized request");

    // Try GET without auth
    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/units/{}/owners", unit_id))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401, "Should reject unauthorized request");
}
