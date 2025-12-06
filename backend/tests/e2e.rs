use actix_web::{http::header, test, App};
use koprogo_api::application::dto::CreateBuildingDto;
use koprogo_api::application::use_cases::*;
use koprogo_api::infrastructure::audit_logger::AuditLogger;
use koprogo_api::infrastructure::database::{
    create_pool, PostgresAccountRepository, PostgresAuditLogRepository,
    PostgresBoardDecisionRepository, PostgresBoardMemberRepository, PostgresBuildingRepository,
    PostgresChargeDistributionRepository, PostgresDocumentRepository, PostgresExpenseRepository,
    PostgresGdprRepository, PostgresJournalEntryRepository, PostgresOwnerRepository,
    PostgresPaymentReminderRepository, PostgresRefreshTokenRepository, PostgresUnitOwnerRepository,
    PostgresUnitRepository, PostgresUserRepository, PostgresUserRoleRepository,
};
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

    let building_repo = Arc::new(PostgresBuildingRepository::new(pool.clone()));
    let unit_repo = Arc::new(PostgresUnitRepository::new(pool.clone()));
    let owner_repo = Arc::new(PostgresOwnerRepository::new(pool.clone()));
    let unit_owner_repo = Arc::new(PostgresUnitOwnerRepository::new(pool.clone()));
    let expense_repo = Arc::new(PostgresExpenseRepository::new(pool.clone()));
    let user_repo = Arc::new(PostgresUserRepository::new(pool.clone()));
    let user_role_repo = Arc::new(PostgresUserRoleRepository::new(pool.clone()));
    let refresh_token_repo = Arc::new(PostgresRefreshTokenRepository::new(pool.clone()));
    let meeting_repo = Arc::new(
        koprogo_api::infrastructure::database::repositories::PostgresMeetingRepository::new(
            pool.clone(),
        ),
    );
    let document_repo = Arc::new(PostgresDocumentRepository::new(pool.clone()));
    let gdpr_repo = Arc::new(PostgresGdprRepository::new(Arc::new(pool.clone())));
    let audit_log_repo = Arc::new(PostgresAuditLogRepository::new(pool.clone()));
    let charge_distribution_repo =
        Arc::new(PostgresChargeDistributionRepository::new(pool.clone()));
    let payment_reminder_repo = Arc::new(PostgresPaymentReminderRepository::new(pool.clone()));

    let audit_logger = AuditLogger::new(Some(audit_log_repo.clone()));
    let jwt_secret = "test-secret-key".to_string();

    let account_repo = Arc::new(PostgresAccountRepository::new(pool.clone()));
    let journal_entry_repo = Arc::new(PostgresJournalEntryRepository::new(pool.clone()));
    let account_use_cases = AccountUseCases::new(account_repo.clone());
    let financial_report_use_cases = FinancialReportUseCases::new(
        account_repo,
        expense_repo.clone(),
        journal_entry_repo.clone(),
    );

    let auth_use_cases =
        AuthUseCases::new(user_repo.clone(), refresh_token_repo, user_role_repo, jwt_secret);
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
    let storage_root = std::env::temp_dir().join("koprogo_e2e_uploads");
    let storage: Arc<dyn StorageProvider> =
        Arc::new(FileStorage::new(&storage_root).expect("storage"));
    let document_use_cases = DocumentUseCases::new(document_repo, storage.clone());
    let pcn_use_cases = PcnUseCases::new(expense_repo.clone());
    let payment_reminder_use_cases =
        PaymentReminderUseCases::new(payment_reminder_repo, expense_repo.clone(), owner_repo.clone());
    let gdpr_use_cases = GdprUseCases::new(gdpr_repo, user_repo.clone());

    // Create an organization for FK references
    let org_id = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO organizations (id, name, slug, contact_email, subscription_plan, max_buildings, max_users, is_active, created_at, updated_at)
           VALUES ($1, 'Org Test', 'org-test', 'org@test.com', 'starter', 10, 10, true, NOW(), NOW())"#
    )
    .bind(org_id)
    .execute(&pool)
    .await
    .expect("insert org");

    let board_member_repo = Arc::new(PostgresBoardMemberRepository::new(pool.clone()));
    let board_decision_repo = Arc::new(PostgresBoardDecisionRepository::new(pool.clone()));
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

    // E2E tests focus on basic CRUD operations (Buildings, Units, Owners, Expenses)
    // Most features are tested separately in BDD tests - we create stub use cases here
    // to satisfy AppState::new() signature without implementing full repository dependencies

    // Use a macro to create stub use cases - these won't be used in E2E tests
    macro_rules! stub_use_case {
        ($use_case_type:ty) => {
            std::mem::zeroed::<$use_case_type>()
        };
    }

    // SAFETY: These use cases are never called in E2E tests - they exist only to satisfy
    // AppState::new() signature. E2E tests only exercise Buildings/Units/Owners/Expenses.
    // All other features have dedicated BDD test coverage.
    let stub_use_cases = unsafe {
        (
            stub_use_case!(BudgetUseCases),
            stub_use_case!(ConvocationUseCases),
            stub_use_case!(ResolutionUseCases),
            stub_use_case!(TicketUseCases),
            stub_use_case!(TwoFactorUseCases),
            stub_use_case!(NotificationUseCases),
            stub_use_case!(PaymentUseCases),
            stub_use_case!(PaymentMethodUseCases),
            stub_use_case!(PollUseCases),
            stub_use_case!(QuoteUseCases),
            stub_use_case!(LocalExchangeUseCases),
            stub_use_case!(NoticeUseCases),
            stub_use_case!(ResourceBookingUseCases),
            stub_use_case!(SharedObjectUseCases),
            stub_use_case!(SkillUseCases),
            stub_use_case!(TechnicalInspectionUseCases),
            stub_use_case!(WorkReportUseCases),
            stub_use_case!(EnergyCampaignUseCases),
            stub_use_case!(EnergyBillUploadUseCases),
            stub_use_case!(EtatDateUseCases),
            stub_use_case!(IoTUseCases),
            stub_use_case!(LinkyUseCases),
            stub_use_case!(DashboardUseCases),
            stub_use_case!(OwnerContributionUseCases),
            stub_use_case!(CallForFundsUseCases),
            stub_use_case!(JournalEntryUseCases),
            stub_use_case!(AchievementUseCases),
            stub_use_case!(ChallengeUseCases),
            stub_use_case!(GamificationStatsUseCases),
        )
    };

    let (
        budget_use_cases,
        convocation_use_cases,
        resolution_use_cases,
        ticket_use_cases,
        two_factor_use_cases,
        notification_use_cases,
        payment_use_cases,
        payment_method_use_cases,
        poll_use_cases,
        quote_use_cases,
        local_exchange_use_cases,
        notice_use_cases,
        resource_booking_use_cases,
        shared_object_use_cases,
        skill_use_cases,
        technical_inspection_use_cases,
        work_report_use_cases,
        energy_campaign_use_cases,
        energy_bill_upload_use_cases,
        etat_date_use_cases,
        iot_use_cases,
        linky_use_cases,
        dashboard_use_cases,
        owner_contribution_use_cases,
        call_for_funds_use_cases,
        journal_entry_use_cases,
        achievement_use_cases,
        challenge_use_cases,
        gamification_stats_use_cases,
    ) = stub_use_cases;

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
        two_factor_use_cases,
        notification_use_cases,
        payment_use_cases,
        payment_method_use_cases,
        poll_use_cases,
        quote_use_cases,
        local_exchange_use_cases,
        notice_use_cases,
        resource_booking_use_cases,
        shared_object_use_cases,
        skill_use_cases,
        technical_inspection_use_cases,
        work_report_use_cases,
        document_use_cases,
        energy_campaign_use_cases,
        energy_bill_upload_use_cases,
        etat_date_use_cases,
        pcn_use_cases,
        payment_reminder_use_cases,
        gdpr_use_cases,
        iot_use_cases,
        linky_use_cases,
        board_member_use_cases,
        board_decision_use_cases,
        board_dashboard_use_cases,
        dashboard_use_cases,
        financial_report_use_cases,
        owner_contribution_use_cases,
        call_for_funds_use_cases,
        journal_entry_use_cases,
        achievement_use_cases,
        challenge_use_cases,
        gamification_stats_use_cases,
        audit_logger,
        EmailService::from_env().expect("email service"),
        pool.clone(),
    ));

    (app_state, postgres_container, org_id)
}

#[actix_web::test]
#[serial]
async fn test_health_endpoint() {
    let (app_state, _container, _org_id) = setup_test_db().await;

    let state = app_state.clone();
    let app = test::init_service(
        App::new()
            .app_data(state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::get().uri("/api/v1/health").to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}

#[actix_web::test]
#[serial]
async fn test_create_building_endpoint() {
    let (app_state, _container, org_id) = setup_test_db().await;

    let state = app_state.clone();
    let app = test::init_service(
        App::new()
            .app_data(state.clone())
            .configure(configure_routes),
    )
    .await;

    // Register + login to obtain JWT tied to org_id
    // Use superadmin role as only superadmin can create buildings (structural data)
    let email = format!("e2e+{}@test.com", Uuid::new_v4());
    let reg = koprogo_api::application::dto::RegisterRequest {
        email: email.clone(),
        password: "Passw0rd!".to_string(),
        first_name: "E2E".to_string(),
        last_name: "SuperAdmin".to_string(),
        role: "superadmin".to_string(),
        organization_id: Some(org_id),
    };
    let _ = state.auth_use_cases.register(reg).await.expect("register");
    let login = koprogo_api::application::dto::LoginRequest {
        email: email.clone(),
        password: "Passw0rd!".to_string(),
    };
    let token = state
        .auth_use_cases
        .login(login)
        .await
        .expect("login")
        .token;

    let dto = CreateBuildingDto {
        organization_id: org_id.to_string(),
        name: "Test Building".to_string(),
        address: "123 Test St".to_string(),
        city: "Paris".to_string(),
        postal_code: "75001".to_string(),
        country: "France".to_string(),
        total_units: 10,
        total_tantiemes: Some(1000),
        construction_year: Some(2000),
    };

    let req = test::TestRequest::post()
        .uri("/api/v1/buildings")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(&dto)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);
}

#[actix_web::test]
#[serial]
async fn test_list_buildings_endpoint() {
    let (app_state, _container, org_id) = setup_test_db().await;

    let state = app_state.clone();
    let app = test::init_service(
        App::new()
            .app_data(state.clone())
            .configure(configure_routes),
    )
    .await;

    // Register + login for auth
    let email = format!("e2e+{}@test.com", Uuid::new_v4());
    let reg = koprogo_api::application::dto::RegisterRequest {
        email: email.clone(),
        password: "Passw0rd!".to_string(),
        first_name: "E2E".to_string(),
        last_name: "User".to_string(),
        role: "syndic".to_string(),
        organization_id: Some(org_id),
    };
    let _ = state.auth_use_cases.register(reg).await.expect("register");
    let login = koprogo_api::application::dto::LoginRequest {
        email: email.clone(),
        password: "Passw0rd!".to_string(),
    };
    let token = state
        .auth_use_cases
        .login(login)
        .await
        .expect("login")
        .token;

    let req = test::TestRequest::get()
        .uri("/api/v1/buildings")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}

#[actix_web::test]
#[serial]
async fn test_create_building_validation_fails() {
    let (app_state, _container, org_id) = setup_test_db().await;

    let state = app_state.clone();
    let app = test::init_service(
        App::new()
            .app_data(state.clone())
            .configure(configure_routes),
    )
    .await;

    // Auth - Use superadmin role as only superadmin can create buildings
    let email = format!("e2e+{}@test.com", Uuid::new_v4());
    let reg = koprogo_api::application::dto::RegisterRequest {
        email: email.clone(),
        password: "Passw0rd!".to_string(),
        first_name: "E2E".to_string(),
        last_name: "SuperAdmin".to_string(),
        role: "superadmin".to_string(),
        organization_id: Some(org_id),
    };
    let _ = state.auth_use_cases.register(reg).await.expect("register");
    let login = koprogo_api::application::dto::LoginRequest {
        email: email.clone(),
        password: "Passw0rd!".to_string(),
    };
    let token = state
        .auth_use_cases
        .login(login)
        .await
        .expect("login")
        .token;

    let dto = CreateBuildingDto {
        organization_id: org_id.to_string(),
        name: "".to_string(), // Invalid: empty name
        address: "123 Test St".to_string(),
        city: "Paris".to_string(),
        postal_code: "75001".to_string(),
        country: "France".to_string(),
        total_units: 10,
        total_tantiemes: Some(1000),
        construction_year: Some(2000),
    };

    let req = test::TestRequest::post()
        .uri("/api/v1/buildings")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(&dto)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);
}
