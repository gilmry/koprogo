// E2E tests for notification system HTTP endpoints (Issue #86)
// Tests focus on HTTP layer: endpoints, auth, JSON serialization
// Covers multi-channel notification system (Email, InApp, Push) for Belgian copropriété

use actix_web::http::header;
use actix_web::{test, App};
use chrono::{Duration, Utc};
use koprogo_api::application::dto::*;
use koprogo_api::application::use_cases::*;
use koprogo_api::domain::entities::{
    NotificationChannel, NotificationPriority, NotificationStatus, NotificationType,
};
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

/// Setup function shared across all notification E2E tests
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
    let notification_repo = Arc::new(PostgresNotificationRepository::new(pool.clone()));
    let notification_preference_repo =
        Arc::new(PostgresNotificationPreferenceRepository::new(pool.clone()));
    let gdpr_repo = Arc::new(PostgresGdprRepository::new(Arc::new(pool.clone())));
    let audit_log_repo = Arc::new(PostgresAuditLogRepository::new(pool.clone()));
    let charge_distribution_repo =
        Arc::new(PostgresChargeDistributionRepository::new(pool.clone()));
    let payment_reminder_repo = Arc::new(PostgresPaymentReminderRepository::new(pool.clone()));
    let board_member_repo = Arc::new(PostgresBoardMemberRepository::new(pool.clone()));
    let board_decision_repo = Arc::new(PostgresBoardDecisionRepository::new(pool.clone()));
    let organization_repo = Arc::new(PostgresOrganizationRepository::new(pool.clone()));
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
    let jwt_secret = "e2e-notification-secret".to_string();
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
    let ticket_use_cases = TicketUseCases::new(ticket_repo, building_repo.clone());
    let notification_use_cases =
        NotificationUseCases::new(notification_repo, notification_preference_repo);
    let gdpr_use_cases = GdprUseCases::new(gdpr_repo, user_repo.clone());
    let payment_reminder_use_cases = PaymentReminderUseCases::new(
        payment_reminder_repo,
        expense_repo,
        owner_repo.clone(),
    );
    let board_member_use_cases = BoardMemberUseCases::new(board_member_repo);
    let board_decision_use_cases =
        BoardDecisionUseCases::new(board_decision_repo, user_repo.clone());
    let board_dashboard_use_cases =
        BoardDashboardUseCases::new(building_repo.clone(), meeting_repo);
    let organization_use_cases = OrganizationUseCases::new(organization_repo);
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
    let storage_root = std::env::temp_dir().join(format!("koprogo_e2e_notifications_{}", test_id));
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

/// Helper: Create test fixtures (organization, user)
async fn create_test_fixtures(app_state: &actix_web::web::Data<AppState>) -> (String, Uuid, Uuid) {
    // 1. Register user and get token
    let register_dto = RegisterUserDto {
        email: format!("notification-test-{}@example.com", Uuid::new_v4()),
        password: "SecurePass123!".to_string(),
        first_name: "Notification".to_string(),
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
        name: format!("Test Org Notification {}", Uuid::new_v4()),
        registration_number: format!("REG-NOTIF-{}", Uuid::new_v4()),
        address: "123 Notification St".to_string(),
        city: "Brussels".to_string(),
        postal_code: "1000".to_string(),
        country: "Belgium".to_string(),
        phone: "+32 2 123 4567".to_string(),
        email: format!("org-notif-{}@example.com", Uuid::new_v4()),
    };

    let organization = app_state
        .organization_use_cases
        .create_organization(org_dto)
        .await
        .expect("Failed to create organization");

    (token, organization.id, user.id)
}

// ==================== Notification CRUD Tests ====================

#[actix_web::test]
#[serial]
async fn test_create_notification_success() {
    let (app_state, _container) = setup_app().await;
    let (token, _org_id, user_id) = create_test_fixtures(&app_state).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/v1/notifications")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "user_id": user_id.to_string(),
            "notification_type": "ExpenseCreated",
            "channel": "Email",
            "priority": "Medium",
            "title": "New expense created",
            "message": "A new maintenance expense has been added to your account."
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201, "Should create notification successfully");

    let notification: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(notification["title"], "New expense created");
    assert_eq!(notification["notification_type"], "ExpenseCreated");
    assert_eq!(notification["channel"], "Email");
    assert_eq!(notification["priority"], "Medium");
    assert_eq!(notification["status"], "Pending");
}

#[actix_web::test]
#[serial]
async fn test_create_notification_without_auth_fails() {
    let (app_state, _container) = setup_app().await;
    let (_token, _org_id, user_id) = create_test_fixtures(&app_state).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/v1/notifications")
        .set_json(json!({
            "user_id": user_id.to_string(),
            "notification_type": "System",
            "channel": "InApp",
            "priority": "Low",
            "title": "Unauthorized",
            "message": "This should fail"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401, "Should require authentication");
}

#[actix_web::test]
#[serial]
async fn test_create_notification_all_types() {
    let (app_state, _container) = setup_app().await;
    let (token, _org_id, user_id) = create_test_fixtures(&app_state).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let notification_types = vec![
        "ExpenseCreated",
        "MeetingConvocation",
        "PaymentReceived",
        "TicketResolved",
        "DocumentAdded",
        "BoardMessage",
        "PaymentReminder",
        "BudgetApproved",
        "ResolutionVote",
        "System",
    ];

    for notif_type in notification_types {
        let req = test::TestRequest::post()
            .uri("/api/v1/notifications")
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
            .set_json(json!({
                "user_id": user_id.to_string(),
                "notification_type": notif_type,
                "channel": "InApp",
                "priority": "Low",
                "title": format!("Test {} notification", notif_type),
                "message": format!("Testing {} type", notif_type)
            }))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(
            resp.status(),
            201,
            "Should create notification for type {}",
            notif_type
        );
    }
}

#[actix_web::test]
#[serial]
async fn test_create_notification_all_channels() {
    let (app_state, _container) = setup_app().await;
    let (token, _org_id, user_id) = create_test_fixtures(&app_state).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let channels = vec!["Email", "InApp", "Push"];

    for channel in channels {
        let req = test::TestRequest::post()
            .uri("/api/v1/notifications")
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
            .set_json(json!({
                "user_id": user_id.to_string(),
                "notification_type": "System",
                "channel": channel,
                "priority": "Low",
                "title": format!("Test {} channel", channel),
                "message": format!("Testing {} channel", channel)
            }))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(
            resp.status(),
            201,
            "Should create notification for channel {}",
            channel
        );
    }
}

#[actix_web::test]
#[serial]
async fn test_create_notification_all_priorities() {
    let (app_state, _container) = setup_app().await;
    let (token, _org_id, user_id) = create_test_fixtures(&app_state).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let priorities = vec!["Low", "Medium", "High", "Critical"];

    for priority in priorities {
        let req = test::TestRequest::post()
            .uri("/api/v1/notifications")
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
            .set_json(json!({
                "user_id": user_id.to_string(),
                "notification_type": "System",
                "channel": "InApp",
                "priority": priority,
                "title": format!("Test {} priority", priority),
                "message": format!("Testing {} priority", priority)
            }))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(
            resp.status(),
            201,
            "Should create notification with priority {}",
            priority
        );
    }
}

#[actix_web::test]
#[serial]
async fn test_get_notification_by_id() {
    let (app_state, _container) = setup_app().await;
    let (token, _org_id, user_id) = create_test_fixtures(&app_state).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create notification
    let create_req = test::TestRequest::post()
        .uri("/api/v1/notifications")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "user_id": user_id.to_string(),
            "notification_type": "PaymentReceived",
            "channel": "Email",
            "priority": "High",
            "title": "Payment received",
            "message": "Your payment of 500 EUR has been received."
        }))
        .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    let notification: serde_json::Value = test::read_body_json(create_resp).await;
    let notification_id = notification["id"].as_str().unwrap();

    // Get notification
    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/notifications/{}", notification_id))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let fetched: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(fetched["id"], notification_id);
    assert_eq!(fetched["title"], "Payment received");
}

#[actix_web::test]
#[serial]
async fn test_get_notification_not_found() {
    let (app_state, _container) = setup_app().await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let fake_id = Uuid::new_v4();
    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/notifications/{}", fake_id))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 404);
}

#[actix_web::test]
#[serial]
async fn test_list_my_notifications() {
    let (app_state, _container) = setup_app().await;
    let (token, _org_id, user_id) = create_test_fixtures(&app_state).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create 3 notifications for the user
    for i in 1..=3 {
        let req = test::TestRequest::post()
            .uri("/api/v1/notifications")
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
            .set_json(json!({
                "user_id": user_id.to_string(),
                "notification_type": "System",
                "channel": "InApp",
                "priority": "Low",
                "title": format!("Notification #{}", i),
                "message": format!("Message {}", i)
            }))
            .to_request();

        test::call_service(&app, req).await;
    }

    // List all notifications for the user
    let req = test::TestRequest::get()
        .uri("/api/v1/notifications/my-notifications")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let notifications: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(notifications.as_array().unwrap().len(), 3);
}

#[actix_web::test]
#[serial]
async fn test_list_unread_notifications() {
    let (app_state, _container) = setup_app().await;
    let (token, _org_id, user_id) = create_test_fixtures(&app_state).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create unread notification
    let req = test::TestRequest::post()
        .uri("/api/v1/notifications")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "user_id": user_id.to_string(),
            "notification_type": "MeetingConvocation",
            "channel": "InApp",
            "priority": "High",
            "title": "AG Convocation",
            "message": "You are invited to the general assembly on Dec 15"
        }))
        .to_request();

    test::call_service(&app, req).await;

    // List unread notifications
    let list_req = test::TestRequest::get()
        .uri("/api/v1/notifications/unread")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, list_req).await;
    assert_eq!(resp.status(), 200);

    let notifications: serde_json::Value = test::read_body_json(resp).await;
    assert!(notifications.as_array().unwrap().len() >= 1);
}

#[actix_web::test]
#[serial]
async fn test_mark_notification_read() {
    let (app_state, _container) = setup_app().await;
    let (token, _org_id, user_id) = create_test_fixtures(&app_state).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create notification
    let create_req = test::TestRequest::post()
        .uri("/api/v1/notifications")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "user_id": user_id.to_string(),
            "notification_type": "DocumentAdded",
            "channel": "InApp",
            "priority": "Low",
            "title": "New document",
            "message": "A new document has been uploaded"
        }))
        .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    let notification: serde_json::Value = test::read_body_json(create_resp).await;
    let notification_id = notification["id"].as_str().unwrap();
    assert!(notification["read_at"].is_null());

    // Mark as read
    let mark_read_req = test::TestRequest::put()
        .uri(&format!("/api/v1/notifications/{}/mark-read", notification_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let mark_read_resp = test::call_service(&app, mark_read_req).await;
    assert_eq!(mark_read_resp.status(), 200);

    let marked: serde_json::Value = test::read_body_json(mark_read_resp).await;
    assert!(marked["read_at"].is_string(), "Should have read_at timestamp");
}

#[actix_web::test]
#[serial]
async fn test_mark_all_notifications_read() {
    let (app_state, _container) = setup_app().await;
    let (token, _org_id, user_id) = create_test_fixtures(&app_state).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create 3 unread notifications
    for i in 1..=3 {
        let req = test::TestRequest::post()
            .uri("/api/v1/notifications")
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
            .set_json(json!({
                "user_id": user_id.to_string(),
                "notification_type": "System",
                "channel": "InApp",
                "priority": "Low",
                "title": format!("Unread notification {}", i),
                "message": format!("Message {}", i)
            }))
            .to_request();

        test::call_service(&app, req).await;
    }

    // Mark all as read
    let mark_all_req = test::TestRequest::put()
        .uri("/api/v1/notifications/mark-all-read")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let mark_all_resp = test::call_service(&app, mark_all_req).await;
    assert_eq!(mark_all_resp.status(), 200);

    // Verify all notifications are now read
    let list_unread_req = test::TestRequest::get()
        .uri("/api/v1/notifications/unread")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, list_unread_req).await;
    let unread: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(
        unread.as_array().unwrap().len(),
        0,
        "Should have no unread notifications"
    );
}

#[actix_web::test]
#[serial]
async fn test_delete_notification() {
    let (app_state, _container) = setup_app().await;
    let (token, _org_id, user_id) = create_test_fixtures(&app_state).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create notification
    let create_req = test::TestRequest::post()
        .uri("/api/v1/notifications")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "user_id": user_id.to_string(),
            "notification_type": "System",
            "channel": "InApp",
            "priority": "Low",
            "title": "To delete",
            "message": "This will be deleted"
        }))
        .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    let notification: serde_json::Value = test::read_body_json(create_resp).await;
    let notification_id = notification["id"].as_str().unwrap();

    // Delete notification
    let delete_req = test::TestRequest::delete()
        .uri(&format!("/api/v1/notifications/{}", notification_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let delete_resp = test::call_service(&app, delete_req).await;
    assert_eq!(delete_resp.status(), 204);

    // Verify deletion
    let get_req = test::TestRequest::get()
        .uri(&format!("/api/v1/notifications/{}", notification_id))
        .to_request();

    let get_resp = test::call_service(&app, get_req).await;
    assert_eq!(get_resp.status(), 404);
}

#[actix_web::test]
#[serial]
async fn test_get_notification_stats() {
    let (app_state, _container) = setup_app().await;
    let (token, _org_id, user_id) = create_test_fixtures(&app_state).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create notifications with different statuses
    for i in 1..=3 {
        let req = test::TestRequest::post()
            .uri("/api/v1/notifications")
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
            .set_json(json!({
                "user_id": user_id.to_string(),
                "notification_type": "System",
                "channel": "InApp",
                "priority": "Low",
                "title": format!("Stats notification {}", i),
                "message": format!("For statistics test {}", i)
            }))
            .to_request();

        test::call_service(&app, req).await;
    }

    // Get statistics
    let stats_req = test::TestRequest::get()
        .uri("/api/v1/notifications/stats")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let stats_resp = test::call_service(&app, stats_req).await;
    assert_eq!(stats_resp.status(), 200);

    let stats: serde_json::Value = test::read_body_json(stats_resp).await;
    assert!(stats["total"].as_i64().unwrap() >= 3);
}

// ==================== Notification Preference Tests ====================

#[actix_web::test]
#[serial]
async fn test_get_user_preferences() {
    let (app_state, _container) = setup_app().await;
    let (token, _org_id, _user_id) = create_test_fixtures(&app_state).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/v1/notification-preferences")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let preferences: serde_json::Value = test::read_body_json(resp).await;
    assert!(preferences.is_array());
}

#[actix_web::test]
#[serial]
async fn test_update_preference() {
    let (app_state, _container) = setup_app().await;
    let (token, _org_id, _user_id) = create_test_fixtures(&app_state).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Update preference for ExpenseCreated notifications
    let update_req = test::TestRequest::put()
        .uri("/api/v1/notification-preferences/ExpenseCreated")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "email_enabled": false,
            "in_app_enabled": true,
            "push_enabled": false
        }))
        .to_request();

    let update_resp = test::call_service(&app, update_req).await;
    assert_eq!(update_resp.status(), 200);

    let preference: serde_json::Value = test::read_body_json(update_resp).await;
    assert_eq!(preference["email_enabled"], false);
    assert_eq!(preference["in_app_enabled"], true);
    assert_eq!(preference["push_enabled"], false);
}

#[actix_web::test]
#[serial]
async fn test_get_specific_preference() {
    let (app_state, _container) = setup_app().await;
    let (token, _org_id, _user_id) = create_test_fixtures(&app_state).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // First update the preference
    let update_req = test::TestRequest::put()
        .uri("/api/v1/notification-preferences/PaymentReceived")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "email_enabled": true,
            "in_app_enabled": true,
            "push_enabled": true
        }))
        .to_request();

    test::call_service(&app, update_req).await;

    // Get specific preference
    let get_req = test::TestRequest::get()
        .uri("/api/v1/notification-preferences/PaymentReceived")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let get_resp = test::call_service(&app, get_req).await;
    assert_eq!(get_resp.status(), 200);

    let preference: serde_json::Value = test::read_body_json(get_resp).await;
    assert_eq!(preference["notification_type"], "PaymentReceived");
    assert_eq!(preference["email_enabled"], true);
}

#[actix_web::test]
#[serial]
async fn test_complete_notification_lifecycle() {
    let (app_state, _container) = setup_app().await;
    let (token, _org_id, user_id) = create_test_fixtures(&app_state).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // 1. Create notification
    let create_req = test::TestRequest::post()
        .uri("/api/v1/notifications")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "user_id": user_id.to_string(),
            "notification_type": "TicketResolved",
            "channel": "InApp",
            "priority": "High",
            "title": "Ticket #123 resolved",
            "message": "Your plumbing ticket has been resolved by the contractor.",
            "link_url": "/tickets/123",
            "metadata": json!({"ticket_id": "123", "category": "Plumbing"}).to_string()
        }))
        .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    let notification: serde_json::Value = test::read_body_json(create_resp).await;
    let notification_id = notification["id"].as_str().unwrap();
    assert_eq!(notification["status"], "Pending");
    assert!(notification["read_at"].is_null());

    // 2. List unread notifications
    let unread_req = test::TestRequest::get()
        .uri("/api/v1/notifications/unread")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let unread_resp = test::call_service(&app, unread_req).await;
    let unread: serde_json::Value = test::read_body_json(unread_resp).await;
    assert!(unread.as_array().unwrap().len() >= 1);

    // 3. Mark notification as read
    let mark_read_req = test::TestRequest::put()
        .uri(&format!("/api/v1/notifications/{}/mark-read", notification_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let mark_read_resp = test::call_service(&app, mark_read_req).await;
    let marked: serde_json::Value = test::read_body_json(mark_read_resp).await;
    assert!(marked["read_at"].is_string());

    // 4. Verify in my notifications list
    let my_notif_req = test::TestRequest::get()
        .uri("/api/v1/notifications/my-notifications")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let my_notif_resp = test::call_service(&app, my_notif_req).await;
    let my_notifications: serde_json::Value = test::read_body_json(my_notif_resp).await;
    assert!(my_notifications.as_array().unwrap().len() >= 1);

    // 5. Get notification statistics
    let stats_req = test::TestRequest::get()
        .uri("/api/v1/notifications/stats")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let stats_resp = test::call_service(&app, stats_req).await;
    let stats: serde_json::Value = test::read_body_json(stats_resp).await;
    assert!(stats["total"].as_i64().unwrap() >= 1);

    // 6. Update notification preferences for this type
    let pref_req = test::TestRequest::put()
        .uri("/api/v1/notification-preferences/TicketResolved")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "email_enabled": true,
            "in_app_enabled": true,
            "push_enabled": false
        }))
        .to_request();

    let pref_resp = test::call_service(&app, pref_req).await;
    assert_eq!(pref_resp.status(), 200);
}
