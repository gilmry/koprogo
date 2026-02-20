// Shared test setup for all E2E tests
// Extracted from e2e.rs to avoid duplication

use koprogo_api::application::use_cases::*;
use koprogo_api::infrastructure::audit_logger::AuditLogger;
use koprogo_api::infrastructure::database::{
    create_pool, PostgresAccountRepository, PostgresAchievementRepository,
    PostgresAuditLogRepository, PostgresBoardDecisionRepository, PostgresBoardMemberRepository,
    PostgresBudgetRepository, PostgresBuildingRepository, PostgresCallForFundsRepository,
    PostgresChallengeProgressRepository, PostgresChallengeRepository,
    PostgresChargeDistributionRepository, PostgresConvocationRecipientRepository,
    PostgresConvocationRepository, PostgresDocumentRepository, PostgresEnergyBillUploadRepository,
    PostgresEnergyCampaignRepository, PostgresEtatDateRepository, PostgresExpenseRepository,
    PostgresGdprRepository, PostgresIoTRepository, PostgresJournalEntryRepository,
    PostgresLocalExchangeRepository, PostgresNoticeRepository,
    PostgresNotificationPreferenceRepository, PostgresNotificationRepository,
    PostgresOwnerContributionRepository, PostgresOwnerCreditBalanceRepository,
    PostgresOwnerRepository, PostgresPaymentMethodRepository, PostgresPaymentReminderRepository,
    PostgresPaymentRepository, PostgresPollRepository, PostgresPollVoteRepository,
    PostgresQuoteRepository, PostgresRefreshTokenRepository, PostgresResolutionRepository,
    PostgresResourceBookingRepository, PostgresSharedObjectRepository, PostgresSkillRepository,
    PostgresTechnicalInspectionRepository, PostgresTicketRepository, PostgresTwoFactorRepository,
    PostgresUnitOwnerRepository, PostgresUnitRepository, PostgresUserAchievementRepository,
    PostgresUserRepository, PostgresUserRoleRepository, PostgresVoteRepository,
    PostgresWorkReportRepository,
};
use koprogo_api::infrastructure::email::EmailService;
use koprogo_api::infrastructure::storage::{FileStorage, StorageProvider};
use koprogo_api::infrastructure::web::AppState;
use koprogo_api::infrastructure::LinkyApiClientImpl;
use std::sync::Arc;
use testcontainers_modules::postgres::Postgres;
use testcontainers_modules::testcontainers::{runners::AsyncRunner, ContainerAsync};
use uuid::Uuid;

/// Setup a complete test environment with all repositories, use cases, and AppState.
/// Returns (app_state, postgres_container, org_id).
/// The container must be kept alive for the duration of the test.
pub async fn setup_test_db() -> (
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

    let auth_use_cases = AuthUseCases::new(
        user_repo.clone(),
        refresh_token_repo,
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
        unit_owner_repo.clone(),
    );
    let meeting_use_cases = MeetingUseCases::new(meeting_repo.clone());
    let storage_root = std::env::temp_dir().join(format!("koprogo_e2e_{}", Uuid::new_v4()));
    let storage: Arc<dyn StorageProvider> =
        Arc::new(FileStorage::new(&storage_root).expect("storage"));
    let document_use_cases = DocumentUseCases::new(document_repo, storage);
    let pcn_use_cases = PcnUseCases::new(expense_repo.clone());
    let payment_reminder_use_cases = PaymentReminderUseCases::new(
        payment_reminder_repo.clone(),
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

    let budget_repo = Arc::new(PostgresBudgetRepository::new(pool.clone()));
    let convocation_repo = Arc::new(PostgresConvocationRepository::new(pool.clone()));
    let convocation_recipient_repo =
        Arc::new(PostgresConvocationRecipientRepository::new(pool.clone()));
    let resolution_repo = Arc::new(PostgresResolutionRepository::new(pool.clone()));
    let vote_repo = Arc::new(PostgresVoteRepository::new(pool.clone()));
    let ticket_repo = Arc::new(PostgresTicketRepository::new(pool.clone()));
    let two_factor_repo = Arc::new(PostgresTwoFactorRepository::new(pool.clone()));
    let notification_repo = Arc::new(PostgresNotificationRepository::new(pool.clone()));
    let notification_preference_repo =
        Arc::new(PostgresNotificationPreferenceRepository::new(pool.clone()));
    let payment_repo = Arc::new(PostgresPaymentRepository::new(pool.clone()));
    let payment_method_repo = Arc::new(PostgresPaymentMethodRepository::new(pool.clone()));
    let poll_repo = Arc::new(PostgresPollRepository::new(pool.clone()));
    let poll_vote_repo = Arc::new(PostgresPollVoteRepository::new(pool.clone()));
    let quote_repo = Arc::new(PostgresQuoteRepository::new(pool.clone()));
    let local_exchange_repo = Arc::new(PostgresLocalExchangeRepository::new(pool.clone()));
    let owner_credit_balance_repo =
        Arc::new(PostgresOwnerCreditBalanceRepository::new(pool.clone()));
    let notice_repo = Arc::new(PostgresNoticeRepository::new(pool.clone()));
    let resource_booking_repo = Arc::new(PostgresResourceBookingRepository::new(pool.clone()));
    let shared_object_repo = Arc::new(PostgresSharedObjectRepository::new(pool.clone()));
    let skill_repo = Arc::new(PostgresSkillRepository::new(pool.clone()));
    let technical_inspection_repo =
        Arc::new(PostgresTechnicalInspectionRepository::new(pool.clone()));
    let work_report_repo = Arc::new(PostgresWorkReportRepository::new(pool.clone()));
    let energy_campaign_repo = Arc::new(PostgresEnergyCampaignRepository::new(pool.clone()));
    let energy_bill_upload_repo = Arc::new(PostgresEnergyBillUploadRepository::new(pool.clone()));
    let etat_date_repo = Arc::new(PostgresEtatDateRepository::new(pool.clone()));
    let iot_repo = Arc::new(PostgresIoTRepository::new(pool.clone()));
    let owner_contribution_repo = Arc::new(PostgresOwnerContributionRepository::new(pool.clone()));
    let call_for_funds_repo = Arc::new(PostgresCallForFundsRepository::new(pool.clone()));
    let achievement_repo = Arc::new(PostgresAchievementRepository::new(pool.clone()));
    let user_achievement_repo = Arc::new(PostgresUserAchievementRepository::new(pool.clone()));
    let challenge_repo = Arc::new(PostgresChallengeRepository::new(pool.clone()));
    let challenge_progress_repo = Arc::new(PostgresChallengeProgressRepository::new(pool.clone()));

    let budget_use_cases = BudgetUseCases::new(
        budget_repo.clone(),
        building_repo.clone(),
        expense_repo.clone(),
    );
    let convocation_use_cases = ConvocationUseCases::new(
        convocation_repo,
        convocation_recipient_repo,
        owner_repo.clone(),
        building_repo.clone(),
        meeting_repo.clone(),
    );
    let resolution_use_cases = ResolutionUseCases::new(resolution_repo, vote_repo);
    let ticket_use_cases = TicketUseCases::new(ticket_repo);
    let encryption_key: [u8; 32] = *b"test-encryption-key-32bytes!!!!!";
    let two_factor_use_cases =
        TwoFactorUseCases::new(two_factor_repo, user_repo.clone(), encryption_key);
    let notification_use_cases =
        NotificationUseCases::new(notification_repo, notification_preference_repo);
    let payment_use_cases = PaymentUseCases::new(payment_repo.clone(), payment_method_repo.clone());
    let payment_method_use_cases = PaymentMethodUseCases::new(payment_method_repo);
    let quote_use_cases = QuoteUseCases::new(quote_repo);
    let local_exchange_use_cases = LocalExchangeUseCases::new(
        local_exchange_repo,
        owner_credit_balance_repo.clone(),
        owner_repo.clone(),
    );
    let notice_use_cases = NoticeUseCases::new(notice_repo, owner_repo.clone());
    let resource_booking_use_cases =
        ResourceBookingUseCases::new(resource_booking_repo, owner_repo.clone());
    let shared_object_use_cases = SharedObjectUseCases::new(
        shared_object_repo,
        owner_repo.clone(),
        owner_credit_balance_repo.clone(),
    );
    let skill_use_cases = SkillUseCases::new(skill_repo, owner_repo.clone());
    let technical_inspection_use_cases =
        TechnicalInspectionUseCases::new(technical_inspection_repo);
    let work_report_use_cases = WorkReportUseCases::new(work_report_repo);
    let energy_campaign_use_cases = EnergyCampaignUseCases::new(
        energy_campaign_repo.clone(),
        energy_bill_upload_repo.clone(),
        building_repo.clone(),
    );
    let energy_bill_upload_use_cases =
        EnergyBillUploadUseCases::new(energy_bill_upload_repo, energy_campaign_repo);
    let etat_date_use_cases = EtatDateUseCases::new(
        etat_date_repo,
        unit_repo.clone(),
        building_repo.clone(),
        unit_owner_repo.clone(),
    );
    let iot_use_cases = IoTUseCases::new(iot_repo.clone());

    let linky_client = Arc::new(LinkyApiClientImpl::new(
        "https://test.linky-api.fr".to_string(),
        "test-client-id".to_string(),
        "test-client-secret".to_string(),
    ));
    let linky_use_cases = LinkyUseCases::new(
        iot_repo,
        linky_client,
        "http://localhost/callback".to_string(),
    );
    let dashboard_use_cases = DashboardUseCases::new(
        expense_repo.clone(),
        owner_contribution_repo.clone(),
        payment_reminder_repo.clone(),
    );
    let owner_contribution_use_cases =
        OwnerContributionUseCases::new(owner_contribution_repo.clone());
    let call_for_funds_use_cases = CallForFundsUseCases::new(
        call_for_funds_repo,
        owner_contribution_repo,
        unit_owner_repo.clone(),
    );
    let journal_entry_use_cases = JournalEntryUseCases::new(journal_entry_repo.clone());
    let poll_use_cases = PollUseCases::new(
        poll_repo,
        poll_vote_repo,
        owner_repo.clone(),
        unit_owner_repo.clone(),
    );
    let achievement_use_cases = AchievementUseCases::new(
        achievement_repo.clone(),
        user_achievement_repo.clone(),
        user_repo.clone(),
    );
    let challenge_use_cases =
        ChallengeUseCases::new(challenge_repo.clone(), challenge_progress_repo.clone());
    let gamification_stats_use_cases = GamificationStatsUseCases::new(
        achievement_repo,
        user_achievement_repo,
        challenge_repo,
        challenge_progress_repo,
        user_repo.clone(),
    );

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

/// Helper to register a user and get a JWT token
#[allow(dead_code)]
pub async fn register_and_login(
    app_state: &actix_web::web::Data<AppState>,
    org_id: Uuid,
) -> String {
    let email = format!("e2e+{}@test.com", Uuid::new_v4());
    let reg = koprogo_api::application::dto::RegisterRequest {
        email: email.clone(),
        password: "Passw0rd!".to_string(),
        first_name: "E2E".to_string(),
        last_name: "Tester".to_string(),
        role: "superadmin".to_string(),
        organization_id: Some(org_id),
    };
    let _ = app_state
        .auth_use_cases
        .register(reg)
        .await
        .expect("register");
    let login_req = koprogo_api::application::dto::LoginRequest {
        email,
        password: "Passw0rd!".to_string(),
    };
    app_state
        .auth_use_cases
        .login(login_req)
        .await
        .expect("login")
        .token
}
