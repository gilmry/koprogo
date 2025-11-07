use actix_web::{test, App};
use koprogo_api::application::dto::{CreateBuildingDto, LoginRequest, RegisterRequest};
use koprogo_api::application::use_cases::*;
use koprogo_api::infrastructure::audit_logger::AuditLogger;
use koprogo_api::infrastructure::database::repositories::*;
use koprogo_api::infrastructure::database::*;
use koprogo_api::infrastructure::email::EmailService;
use koprogo_api::infrastructure::storage::{FileStorage, StorageProvider};
use koprogo_api::infrastructure::web::{configure_routes, AppState};
use serial_test::serial;
use std::sync::Arc;
use testcontainers_modules::postgres::Postgres;
use testcontainers_modules::testcontainers::{runners::AsyncRunner, ContainerAsync};
use uuid::Uuid;

async fn setup_test_db() -> (
    actix_web::web::Data<AppState>,
    ContainerAsync<Postgres>,
    Uuid,
) {
    let postgres_container = Postgres::default()
        .start()
        .await
        .expect("Failed to start Postgres container");
    let port = postgres_container
        .get_host_port_ipv4(5432)
        .await
        .expect("Failed to get port");

    let database_url = format!("postgres://postgres:postgres@localhost:{}/postgres", port);

    let pool = create_pool(&database_url)
        .await
        .expect("Failed to create pool");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    let org_id = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO organizations (id, name, slug, contact_email, subscription_plan, max_buildings, max_users, is_active, created_at, updated_at)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NOW(), NOW())"#
    )
    .bind(org_id)
    .bind("Test Organization E2E Board Dashboard")
    .bind(format!("test-org-dashboard-{}", Uuid::new_v4()))
    .bind("dashboard@test.com")
    .bind("professional")
    .bind(100i32)
    .bind(1000i32)
    .bind(true)
    .execute(&pool)
    .await
    .expect("insert org");

    // Initialize repositories
    let user_repo = Arc::new(PostgresUserRepository::new(pool.clone()));
    let user_role_repo = Arc::new(PostgresUserRoleRepository::new(pool.clone()));
    let refresh_token_repo = Arc::new(PostgresRefreshTokenRepository::new(pool.clone()));
    let building_repo = Arc::new(PostgresBuildingRepository::new(pool.clone()));
    let unit_repo = Arc::new(PostgresUnitRepository::new(pool.clone()));
    let owner_repo = Arc::new(PostgresOwnerRepository::new(pool.clone()));
    let unit_owner_repo = Arc::new(PostgresUnitOwnerRepository::new(pool.clone()));
    let expense_repo = Arc::new(PostgresExpenseRepository::new(pool.clone()));
    let meeting_repo = Arc::new(PostgresMeetingRepository::new(pool.clone()));
    let document_repo = Arc::new(PostgresDocumentRepository::new(pool.clone()));
    let board_member_repo = Arc::new(PostgresBoardMemberRepository::new(pool.clone()));
    let board_decision_repo = Arc::new(PostgresBoardDecisionRepository::new(pool.clone()));
    let gdpr_repo = Arc::new(PostgresGdprRepository::new(Arc::new(pool.clone())));
    let audit_log_repo = Arc::new(PostgresAuditLogRepository::new(pool.clone()));

    let file_storage: Arc<dyn StorageProvider> =
        Arc::new(FileStorage::new("./test_uploads").expect("file storage"));
    let audit_logger = AuditLogger::new(Some(audit_log_repo.clone()));

    // Initialize use cases
    let jwt_secret = "test-secret-key".to_string();
    let auth_use_cases =
        AuthUseCases::new(user_repo, refresh_token_repo, user_role_repo, jwt_secret);
    let building_use_cases = BuildingUseCases::new(building_repo.clone());
    let unit_use_cases = UnitUseCases::new(unit_repo.clone());
    let owner_use_cases = OwnerUseCases::new(owner_repo.clone());
    let unit_owner_use_cases =
        UnitOwnerUseCases::new(unit_owner_repo, unit_repo.clone(), owner_repo.clone());
    let expense_use_cases = ExpenseUseCases::new(expense_repo.clone());
    let meeting_use_cases = MeetingUseCases::new(meeting_repo.clone());
    let document_use_cases = DocumentUseCases::new(document_repo, file_storage.clone());
    let pcn_use_cases = PcnUseCases::new(expense_repo);
    let gdpr_use_cases = GdprUseCases::new(gdpr_repo);
    let board_member_use_cases =
        BoardMemberUseCases::new(board_member_repo.clone(), building_repo.clone());
    let board_decision_use_cases = BoardDecisionUseCases::new(
        board_decision_repo.clone(),
        building_repo.clone(),
        meeting_repo.clone(),
    );
    let board_dashboard_use_cases = BoardDashboardUseCases::new(
        board_member_repo,
        board_decision_repo,
        building_repo.clone(),
    );

    let app_state = actix_web::web::Data::new(AppState::new(
        auth_use_cases,
        building_use_cases,
        unit_use_cases,
        owner_use_cases,
        unit_owner_use_cases,
        expense_use_cases,
        meeting_use_cases,
        document_use_cases,
        pcn_use_cases,
        gdpr_use_cases,
        board_member_use_cases,
        board_decision_use_cases,
        board_dashboard_use_cases,
        audit_logger,
        EmailService::from_env().expect("email service"),
        pool.clone(),
    ));

    (app_state, postgres_container, org_id)
}

#[actix_web::test]
#[serial]
async fn test_board_dashboard_returns_complete_data() {
    let (app_state, _container, org_id) = setup_test_db().await;

    let state = app_state.clone();
    let app = test::init_service(
        App::new()
            .app_data(state.clone())
            .configure(configure_routes),
    )
    .await;

    // Register + login as superadmin
    let email = format!("board-dashboard-{}@test.com", Uuid::new_v4());
    let reg = RegisterRequest {
        email: email.clone(),
        password: "Passw0rd!".to_string(),
        first_name: "Board".to_string(),
        last_name: "Member".to_string(),
        role: "superadmin".to_string(),
        organization_id: Some(org_id),
    };
    let _ = state.auth_use_cases.register(reg).await.expect("register");

    let login = LoginRequest {
        email: email.clone(),
        password: "Passw0rd!".to_string(),
    };
    let login_response = state.auth_use_cases.login(login).await.expect("login");
    let token = login_response.token;

    // Create building (>20 units for board requirement)
    let building_dto = CreateBuildingDto {
        organization_id: org_id.to_string(),
        name: "Test Building Dashboard".to_string(),
        address: "123 Dashboard St".to_string(),
        city: "Brussels".to_string(),
        postal_code: "1000".to_string(),
        country: "Belgium".to_string(),
        total_units: 25,
        total_tantiemes: Some(1000),
        construction_year: Some(2020),
    };

    let req = test::TestRequest::post()
        .uri("/api/v1/buildings")
        .insert_header((
            actix_web::http::header::AUTHORIZATION,
            format!("Bearer {}", token),
        ))
        .set_json(&building_dto)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);
    let building: serde_json::Value = test::read_body_json(resp).await;
    let building_id = building["id"].as_str().unwrap();

    // Create owner for board member
    let owner_dto = serde_json::json!({
        "organization_id": org_id,
        "first_name": "John",
        "last_name": "President",
        "email": "president@example.com",
        "phone": "+32123456789",
        "address": "456 Owner St",
        "city": "Brussels",
        "postal_code": "1000",
        "country": "Belgium"
    });

    let req = test::TestRequest::post()
        .uri("/api/v1/owners")
        .insert_header((
            actix_web::http::header::AUTHORIZATION,
            format!("Bearer {}", token),
        ))
        .set_json(&owner_dto)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);
    let owner: serde_json::Value = test::read_body_json(resp).await;
    let owner_id_str = owner["id"].as_str().unwrap();
    let owner_id_uuid = Uuid::parse_str(owner_id_str).unwrap();

    // Link the user to this owner directly in the database (E2E test shortcut)
    // In production, this would be done via the API after the user has an 'owner' role
    sqlx::query("UPDATE owners SET user_id = $1 WHERE id = $2")
        .bind(login_response.user.id)
        .bind(owner_id_uuid)
        .execute(&state.pool)
        .await
        .expect("link user to owner");

    // Create meeting for board election
    let meeting_dto = serde_json::json!({
        "organization_id": org_id,
        "building_id": building_id,
        "meeting_type": "Ordinary",
        "title": "Board Election",
        "description": "Elect board president",
        "scheduled_date": chrono::Utc::now().to_rfc3339(),
        "location": "Hall"
    });

    let req = test::TestRequest::post()
        .uri("/api/v1/meetings")
        .insert_header((
            actix_web::http::header::AUTHORIZATION,
            format!("Bearer {}", token),
        ))
        .set_json(&meeting_dto)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);
    let meeting: serde_json::Value = test::read_body_json(resp).await;
    let meeting_id = meeting["id"].as_str().unwrap();

    // Elect board member with mandate expiring in 45 days
    let mandate_start = chrono::Utc::now() - chrono::Duration::days(320);
    let mandate_end = mandate_start + chrono::Duration::days(365); // Expires in 45 days

    let board_member_dto = serde_json::json!({
        "owner_id": owner_id_str,
        "building_id": building_id,
        "position": "president",
        "mandate_start": mandate_start.to_rfc3339(),
        "mandate_end": mandate_end.to_rfc3339(),
        "elected_by_meeting_id": meeting_id,
    });

    let req = test::TestRequest::post()
        .uri("/api/v1/board-members")
        .insert_header((
            actix_web::http::header::AUTHORIZATION,
            format!("Bearer {}", token),
        ))
        .set_json(&board_member_dto)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    // Create some board decisions
    // 1. Pending decision
    let decision1 = serde_json::json!({
        "building_id": building_id,
        "meeting_id": meeting_id,
        "subject": "Pending Task 1",
        "decision_text": "This is pending",
        "deadline": (chrono::Utc::now() + chrono::Duration::days(30)).to_rfc3339(),
    });

    let req = test::TestRequest::post()
        .uri("/api/v1/board-decisions")
        .insert_header((
            actix_web::http::header::AUTHORIZATION,
            format!("Bearer {}", token),
        ))
        .set_json(&decision1)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    // 2. Another pending decision
    let decision2 = serde_json::json!({
        "building_id": building_id,
        "meeting_id": meeting_id,
        "subject": "Pending Task 2",
        "decision_text": "This is also pending",
        "deadline": (chrono::Utc::now() + chrono::Duration::days(60)).to_rfc3339(),
    });

    let req = test::TestRequest::post()
        .uri("/api/v1/board-decisions")
        .insert_header((
            actix_web::http::header::AUTHORIZATION,
            format!("Bearer {}", token),
        ))
        .set_json(&decision2)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    // 3. Third pending decision
    let decision3 = serde_json::json!({
        "building_id": building_id,
        "meeting_id": meeting_id,
        "subject": "Pending Task 3",
        "decision_text": "This is pending too",
        "deadline": (chrono::Utc::now() + chrono::Duration::days(15)).to_rfc3339(),
    });

    let req = test::TestRequest::post()
        .uri("/api/v1/board-decisions")
        .insert_header((
            actix_web::http::header::AUTHORIZATION,
            format!("Bearer {}", token),
        ))
        .set_json(&decision3)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    // NOW TEST THE DASHBOARD ENDPOINT (this will fail initially - RED phase)
    let req = test::TestRequest::get()
        .uri(&format!(
            "/api/v1/board-members/dashboard?building_id={}",
            building_id
        ))
        .insert_header((
            actix_web::http::header::AUTHORIZATION,
            format!("Bearer {}", token),
        ))
        .to_request();

    let resp = test::call_service(&app, req).await;
    let status = resp.status();
    assert_eq!(status, 200, "Dashboard endpoint should return 200");

    let body = test::read_body(resp).await;

    let dashboard: serde_json::Value = serde_json::from_slice(&body).expect("parse json");

    // Verify dashboard structure
    assert!(
        dashboard.get("my_mandate").is_some(),
        "Should have my_mandate"
    );
    assert!(
        dashboard.get("decisions_stats").is_some(),
        "Should have decisions_stats"
    );
    assert!(
        dashboard.get("overdue_decisions").is_some(),
        "Should have overdue_decisions"
    );
    assert!(
        dashboard.get("upcoming_deadlines").is_some(),
        "Should have upcoming_deadlines"
    );

    // Verify data
    let stats = &dashboard["decisions_stats"];
    assert_eq!(
        stats["pending"].as_i64().unwrap(),
        3,
        "Should have 3 pending decisions"
    );

    let my_mandate = &dashboard["my_mandate"];
    assert!(
        my_mandate["expires_soon"].as_bool().unwrap(),
        "Mandate should expire soon"
    );
}

#[actix_web::test]
#[serial]
async fn test_superadmin_has_global_dashboard_access() {
    let (app_state, _container, org_id) = setup_test_db().await;

    let state = app_state.clone();
    let app = test::init_service(
        App::new()
            .app_data(state.clone())
            .configure(configure_routes),
    )
    .await;

    // Register + login as board member
    let email = format!("board-member-{}@test.com", Uuid::new_v4());
    let reg = RegisterRequest {
        email: email.clone(),
        password: "Passw0rd!".to_string(),
        first_name: "Board".to_string(),
        last_name: "MemberA".to_string(),
        role: "superadmin".to_string(), // Use superadmin to create resources
        organization_id: Some(org_id),
    };
    let _ = state.auth_use_cases.register(reg).await.expect("register");

    let login = LoginRequest {
        email: email.clone(),
        password: "Passw0rd!".to_string(),
    };
    let login_response = state.auth_use_cases.login(login).await.expect("login");
    let token = login_response.token;

    // Create Building A (>20 units)
    let building_a_dto = CreateBuildingDto {
        organization_id: org_id.to_string(),
        name: "Building A".to_string(),
        address: "123 A St".to_string(),
        city: "Brussels".to_string(),
        postal_code: "1000".to_string(),
        country: "Belgium".to_string(),
        total_units: 25,
        total_tantiemes: Some(1000),
        construction_year: Some(2020),
    };

    let req = test::TestRequest::post()
        .uri("/api/v1/buildings")
        .insert_header((
            actix_web::http::header::AUTHORIZATION,
            format!("Bearer {}", token),
        ))
        .set_json(&building_a_dto)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);
    let building_a: serde_json::Value = test::read_body_json(resp).await;
    let building_a_id = building_a["id"].as_str().unwrap();

    // Create Building B (>20 units)
    let building_b_dto = CreateBuildingDto {
        organization_id: org_id.to_string(),
        name: "Building B".to_string(),
        address: "456 B St".to_string(),
        city: "Brussels".to_string(),
        postal_code: "1001".to_string(),
        country: "Belgium".to_string(),
        total_units: 30,
        total_tantiemes: Some(1500),
        construction_year: Some(2021),
    };

    let req = test::TestRequest::post()
        .uri("/api/v1/buildings")
        .insert_header((
            actix_web::http::header::AUTHORIZATION,
            format!("Bearer {}", token),
        ))
        .set_json(&building_b_dto)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);
    let building_b: serde_json::Value = test::read_body_json(resp).await;
    let building_b_id = building_b["id"].as_str().unwrap();

    // Create owner for board member
    let owner_dto = serde_json::json!({
        "organization_id": org_id,
        "first_name": "John",
        "last_name": "BoardMember",
        "email": "john@example.com",
        "phone": "+32123456789",
        "address": "789 Owner St",
        "city": "Brussels",
        "postal_code": "1000",
        "country": "Belgium"
    });

    let req = test::TestRequest::post()
        .uri("/api/v1/owners")
        .insert_header((
            actix_web::http::header::AUTHORIZATION,
            format!("Bearer {}", token),
        ))
        .set_json(&owner_dto)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);
    let owner: serde_json::Value = test::read_body_json(resp).await;
    let owner_id_str = owner["id"].as_str().unwrap();
    let owner_id_uuid = Uuid::parse_str(owner_id_str).unwrap();

    // Link user to owner
    sqlx::query("UPDATE owners SET user_id = $1 WHERE id = $2")
        .bind(login_response.user.id)
        .bind(owner_id_uuid)
        .execute(&state.pool)
        .await
        .expect("link user to owner");

    // Create meeting for Building A
    let meeting_dto = serde_json::json!({
        "organization_id": org_id,
        "building_id": building_a_id,
        "meeting_type": "Ordinary",
        "title": "Board Election A",
        "description": "Elect board for Building A",
        "scheduled_date": chrono::Utc::now().to_rfc3339(),
        "location": "Hall A"
    });

    let req = test::TestRequest::post()
        .uri("/api/v1/meetings")
        .insert_header((
            actix_web::http::header::AUTHORIZATION,
            format!("Bearer {}", token),
        ))
        .set_json(&meeting_dto)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);
    let meeting: serde_json::Value = test::read_body_json(resp).await;
    let meeting_id = meeting["id"].as_str().unwrap();

    // Elect as board member for Building A ONLY
    let board_member_dto = serde_json::json!({
        "owner_id": owner_id_str,
        "building_id": building_a_id,
        "position": "president",
        "mandate_start": chrono::Utc::now().to_rfc3339(),
        "mandate_end": (chrono::Utc::now() + chrono::Duration::days(365)).to_rfc3339(),
        "elected_by_meeting_id": meeting_id,
    });

    let req = test::TestRequest::post()
        .uri("/api/v1/board-members")
        .insert_header((
            actix_web::http::header::AUTHORIZATION,
            format!("Bearer {}", token),
        ))
        .set_json(&board_member_dto)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    // TEST 1: Can access Building A dashboard (authorized)
    let req = test::TestRequest::get()
        .uri(&format!(
            "/api/v1/board-members/dashboard?building_id={}",
            building_a_id
        ))
        .insert_header((
            actix_web::http::header::AUTHORIZATION,
            format!("Bearer {}", token),
        ))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.status(),
        200,
        "Should access Building A dashboard as board member"
    );

    // TEST 2: Superadmin CAN access Building B dashboard (even though not a board member)
    // This verifies that superadmins have global access
    let req = test::TestRequest::get()
        .uri(&format!(
            "/api/v1/board-members/dashboard?building_id={}",
            building_b_id
        ))
        .insert_header((
            actix_web::http::header::AUTHORIZATION,
            format!("Bearer {}", token),
        ))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.status(),
        200,
        "Superadmin should access ANY building dashboard (global access)"
    );
}

#[actix_web::test]
#[serial]
async fn test_regular_board_member_cannot_access_other_building() {
    let (app_state, _container, org_id) = setup_test_db().await;

    let state = app_state.clone();
    let app = test::init_service(
        App::new()
            .app_data(state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create Building A and Building B
    let building_a_id = {
        let dto = CreateBuildingDto {
            organization_id: org_id.to_string(),
            name: "Building A".to_string(),
            address: "123 A Street".to_string(),
            city: "Brussels".to_string(),
            postal_code: "1000".to_string(),
            country: "Belgium".to_string(),
            total_units: 25,
            total_tantiemes: Some(2500),
            construction_year: Some(2015),
        };
        state
            .building_use_cases
            .create_building(dto)
            .await
            .expect("create building A")
            .id
    };

    let building_b_id = {
        let dto = CreateBuildingDto {
            organization_id: org_id.to_string(),
            name: "Building B".to_string(),
            address: "456 B Avenue".to_string(),
            city: "Brussels".to_string(),
            postal_code: "1000".to_string(),
            country: "Belgium".to_string(),
            total_units: 30,
            total_tantiemes: Some(3000),
            construction_year: Some(2018),
        };
        state
            .building_use_cases
            .create_building(dto)
            .await
            .expect("create building B")
            .id
    };

    // Register a regular user with board_member role (NOT superadmin)
    let email = format!("boardmember-{}@test.com", Uuid::new_v4());
    let password = "Passw0rd!";

    let reg = RegisterRequest {
        email: email.clone(),
        password: password.to_string(),
        first_name: "Board".to_string(),
        last_name: "Member".to_string(),
        role: "owner".to_string(), // Base role - will become board member via election
        organization_id: Some(org_id),
    };
    let login_response = state
        .auth_use_cases
        .register(reg)
        .await
        .expect("register board member");
    let user_id = login_response.user.id;

    // Create an owner and link to the user
    let owner_id = {
        use koprogo_api::application::dto::CreateOwnerDto;
        let owner_dto = CreateOwnerDto {
            organization_id: org_id.to_string(),
            first_name: "Board".to_string(),
            last_name: "Member".to_string(),
            email: email.clone(),
            phone: Some("+32123456789".to_string()),
            address: "123 Owner St".to_string(),
            city: "Brussels".to_string(),
            postal_code: "1000".to_string(),
            country: "Belgium".to_string(),
        };
        let created_owner = state
            .owner_use_cases
            .create_owner(owner_dto)
            .await
            .expect("create owner");

        // Link owner to user
        sqlx::query("UPDATE owners SET user_id = $1 WHERE id = $2")
            .bind(user_id)
            .bind(Uuid::parse_str(&created_owner.id).unwrap())
            .execute(&state.pool)
            .await
            .expect("link owner to user");

        Uuid::parse_str(&created_owner.id).unwrap()
    };

    // Create a meeting for Building A to elect board member
    let meeting_a_id = {
        sqlx::query_scalar::<_, Uuid>(
            "INSERT INTO meetings (id, organization_id, building_id, meeting_type, title, location, scheduled_date, created_at, updated_at)
             VALUES ($1, $2, $3, $4::meeting_type, $5, $6, $7, NOW(), NOW())
             RETURNING id"
        )
        .bind(Uuid::new_v4())
        .bind(org_id)
        .bind(Uuid::parse_str(&building_a_id).unwrap())
        .bind("ordinary")
        .bind("Board Election - Building A")
        .bind("Building A Meeting Room")
        .bind(chrono::Utc::now())
        .fetch_one(&state.pool)
        .await
        .expect("create meeting A")
    };

    // Elect user as board member for Building A ONLY
    {
        use koprogo_api::application::dto::CreateBoardMemberDto;
        let dto = CreateBoardMemberDto {
            owner_id: owner_id.to_string(),
            building_id: building_a_id.clone(),
            position: "president".to_string(),
            mandate_start: chrono::Utc::now().to_rfc3339(),
            mandate_end: (chrono::Utc::now() + chrono::Duration::days(365)).to_rfc3339(),
            elected_by_meeting_id: meeting_a_id.to_string(),
        };
        state
            .board_member_use_cases
            .elect_board_member(dto)
            .await
            .expect("elect board member for building A");
    }

    // Use the token from registration (already logged in)
    let token = login_response.token;

    // TEST 1: Can access Building A dashboard (authorized)
    let building_a_uuid = Uuid::parse_str(&building_a_id).unwrap();
    let req = test::TestRequest::get()
        .uri(&format!(
            "/api/v1/board-members/dashboard?building_id={}",
            building_a_uuid
        ))
        .insert_header((
            actix_web::http::header::AUTHORIZATION,
            format!("Bearer {}", token),
        ))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.status(),
        200,
        "Board member should access their own building dashboard"
    );

    // TEST 2: CANNOT access Building B dashboard (403 Forbidden)
    let building_b_uuid = Uuid::parse_str(&building_b_id).unwrap();
    let req = test::TestRequest::get()
        .uri(&format!(
            "/api/v1/board-members/dashboard?building_id={}",
            building_b_uuid
        ))
        .insert_header((
            actix_web::http::header::AUTHORIZATION,
            format!("Bearer {}", token),
        ))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.status(),
        403,
        "Regular board member should NOT access other building's dashboard (only superadmin has global access)"
    );

    // Verify error message
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(
        body["error"].as_str().unwrap().contains("Access denied"),
        "Should return access denied error"
    );
}

#[actix_web::test]
#[serial]
async fn test_only_board_member_or_superadmin_can_access_dashboard() {
    let (app_state, _container, org_id) = setup_test_db().await;

    let state = app_state.clone();
    let app = test::init_service(
        App::new()
            .app_data(state.clone())
            .configure(configure_routes),
    )
    .await;

    // Register + login as regular owner (not board member)
    let email = format!("owner-{}@test.com", Uuid::new_v4());
    let reg = RegisterRequest {
        email: email.clone(),
        password: "Passw0rd!".to_string(),
        first_name: "Regular".to_string(),
        last_name: "Owner".to_string(),
        role: "owner".to_string(),
        organization_id: Some(org_id),
    };
    let _ = state.auth_use_cases.register(reg).await.expect("register");

    let login = LoginRequest {
        email: email.clone(),
        password: "Passw0rd!".to_string(),
    };
    let login_response = state.auth_use_cases.login(login).await.expect("login");
    let token = login_response.token;

    // Try to access dashboard as non-board-member
    let req = test::TestRequest::get()
        .uri("/api/v1/board-members/dashboard?building_id=00000000-0000-0000-0000-000000000000")
        .insert_header((
            actix_web::http::header::AUTHORIZATION,
            format!("Bearer {}", token),
        ))
        .to_request();

    let resp = test::call_service(&app, req).await;

    // This assertion will FAIL in RED phase - that's expected!
    assert_eq!(
        resp.status(),
        403,
        "Owner (non-board-member) should get 403 Forbidden"
    );
}
