use actix_web::{http::header, test, App};
use koprogo_api::application::dto::{LoginRequest, RegisterRequest};
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
    let account_use_cases = AccountUseCases::new(account_repo.clone());
    let journal_entry_repo = Arc::new(PostgresJournalEntryRepository::new(pool.clone()));
    let financial_report_use_cases =
        FinancialReportUseCases::new(account_repo, expense_repo.clone(), journal_entry_repo);

    let auth_use_cases =
        AuthUseCases::new(user_repo, refresh_token_repo, user_role_repo, jwt_secret);
    let building_use_cases = BuildingUseCases::new(building_repo.clone());
    let unit_use_cases = UnitUseCases::new(unit_repo.clone());
    let owner_use_cases = OwnerUseCases::new(owner_repo.clone());
    let unit_owner_use_cases =
        UnitOwnerUseCases::new(unit_owner_repo.clone(), unit_repo, owner_repo);
    let expense_use_cases = ExpenseUseCases::new(expense_repo.clone());
    let charge_distribution_use_cases = ChargeDistributionUseCases::new(
        charge_distribution_repo,
        expense_repo.clone(),
        unit_owner_repo,
    );
    let meeting_use_cases = MeetingUseCases::new(meeting_repo.clone());
    let storage_root = std::env::temp_dir().join("koprogo_e2e_gdpr_uploads");
    let storage: Arc<dyn StorageProvider> =
        Arc::new(FileStorage::new(&storage_root).expect("storage"));
    let document_use_cases = DocumentUseCases::new(document_repo, storage.clone());
    let pcn_use_cases = PcnUseCases::new(expense_repo.clone());
    let payment_reminder_use_cases = PaymentReminderUseCases::new(
        payment_reminder_repo,
        expense_repo.clone(),
        owner_repo.clone(),
    );
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

    (app_state, postgres_container, org_id)
}

#[actix_web::test]
#[serial]
async fn test_gdpr_export_creates_audit_log() {
    let (app_state, _container, org_id) = setup_test_db().await;
    let pool = app_state.pool.clone();

    let state = app_state.clone();
    let app = test::init_service(
        App::new()
            .app_data(state.clone())
            .configure(configure_routes),
    )
    .await;

    // Register + login to obtain JWT
    let email = format!("gdpr+{}@test.com", Uuid::new_v4());
    let reg = RegisterRequest {
        email: email.clone(),
        password: "Passw0rd!".to_string(),
        first_name: "GDPR".to_string(),
        last_name: "Test".to_string(),
        role: "syndic".to_string(),
        organization_id: Some(org_id),
    };
    let _ = state.auth_use_cases.register(reg).await.expect("register");
    let login = LoginRequest {
        email: email.clone(),
        password: "Passw0rd!".to_string(),
    };
    let token = state
        .auth_use_cases
        .login(login)
        .await
        .expect("login")
        .token;

    // Call GDPR export endpoint
    let req = test::TestRequest::get()
        .uri("/api/v1/gdpr/export")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    // Wait a bit for async audit log to be written
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    // Check that an audit log was created in the database
    let count: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM audit_logs WHERE event_type = 'GdprDataExported'")
            .fetch_one(&pool)
            .await
            .expect("count audit logs");

    assert_eq!(count.0, 1, "Expected 1 audit log for GdprDataExported");

    // Verify the audit log has correct data
    let log: (String, bool, Option<String>) = sqlx::query_as(
        "SELECT event_type, success, metadata::text FROM audit_logs WHERE event_type = 'GdprDataExported' LIMIT 1"
    )
    .fetch_one(&pool)
    .await
    .expect("fetch audit log");

    assert_eq!(log.0, "GdprDataExported");
    assert!(log.1, "Expected success to be true");
    assert!(log.2.is_some(), "Metadata should be present");

    // Check retention_until is set (7 years in the future)
    let retention: (chrono::DateTime<chrono::Utc>,) = sqlx::query_as(
        "SELECT retention_until FROM audit_logs WHERE event_type = 'GdprDataExported' LIMIT 1",
    )
    .fetch_one(&pool)
    .await
    .expect("fetch retention_until");

    let now = chrono::Utc::now();
    let min_retention = now + chrono::Duration::days(365 * 6 + 360); // ~6.99 years
    let max_retention = now + chrono::Duration::days(365 * 7 + 5); // ~7.01 years

    assert!(
        retention.0 > min_retention && retention.0 < max_retention,
        "Retention should be approximately 7 years in the future. Got: {}, Expected between {} and {}",
        retention.0, min_retention, max_retention
    );
}

#[actix_web::test]
#[serial]
async fn test_gdpr_can_erase_creates_audit_log() {
    let (app_state, _container, org_id) = setup_test_db().await;
    let pool = app_state.pool.clone();

    let state = app_state.clone();
    let app = test::init_service(
        App::new()
            .app_data(state.clone())
            .configure(configure_routes),
    )
    .await;

    // Register + login
    let email = format!("gdpr+{}@test.com", Uuid::new_v4());
    let reg = RegisterRequest {
        email: email.clone(),
        password: "Passw0rd!".to_string(),
        first_name: "GDPR".to_string(),
        last_name: "Test".to_string(),
        role: "syndic".to_string(),
        organization_id: Some(org_id),
    };
    let _ = state.auth_use_cases.register(reg).await.expect("register");
    let login = LoginRequest {
        email: email.clone(),
        password: "Passw0rd!".to_string(),
    };
    let token = state
        .auth_use_cases
        .login(login)
        .await
        .expect("login")
        .token;

    // Call can-erase endpoint
    let req = test::TestRequest::get()
        .uri("/api/v1/gdpr/can-erase")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    // Wait for async audit log
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    // Check audit log
    let count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM audit_logs WHERE event_type = 'GdprErasureCheckRequested'",
    )
    .fetch_one(&pool)
    .await
    .expect("count audit logs");

    assert_eq!(
        count.0, 1,
        "Expected 1 audit log for GdprErasureCheckRequested"
    );
}
