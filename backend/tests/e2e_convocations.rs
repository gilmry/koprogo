use actix_web::{http::header, test, web, App};
use chrono::{DateTime, Duration, Utc};
use serde_json::json;
use serial_test::serial;
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use testcontainers::core::IntoContainerPort;
use testcontainers::{runners::AsyncRunner, ContainerAsync, GenericImage};
use testcontainers_modules::postgres::Postgres;
use uuid::Uuid;

use koprogo::application::dto::{
    ConvocationRecipientResponse, ConvocationResponse, RecipientTrackingSummaryResponse,
};
use koprogo::application::ports::*;
use koprogo::application::use_cases::*;
use koprogo::domain::entities::{AttendanceStatus, ConvocationStatus, ConvocationType};
use koprogo::infrastructure::database::repositories::*;
use koprogo::infrastructure::email::mock_email_service::MockEmailService;
use koprogo::infrastructure::storage::mock_storage_provider::MockStorageProvider;
use koprogo::infrastructure::web::{create_authenticated_app, AppState};

// ==================== Test Setup ====================

async fn setup_app() -> (web::Data<AppState>, ContainerAsync<Postgres>) {
    let postgres_container = Postgres::default()
        .start()
        .await
        .expect("Failed to start PostgreSQL container");

    let host_port = postgres_container
        .get_host_port_ipv4(5432.tcp())
        .await
        .expect("Failed to get PostgreSQL host port");

    let connection_string = format!(
        "postgresql://postgres:postgres@127.0.0.1:{}/postgres",
        host_port
    );

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&connection_string)
        .await
        .expect("Failed to create PostgreSQL connection pool");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    // Initialize all repositories
    let account_repo = Arc::new(PostgresAccountRepository::new(pool.clone()));
    let auth_repo = Arc::new(PostgresAuthRepository::new(pool.clone()));
    let building_repo = Arc::new(PostgresBuildingRepository::new(pool.clone()));
    let unit_repo = Arc::new(PostgresUnitRepository::new(pool.clone()));
    let unit_owner_repo = Arc::new(PostgresUnitOwnerRepository::new(pool.clone()));
    let owner_repo = Arc::new(PostgresOwnerRepository::new(pool.clone()));
    let expense_repo = Arc::new(PostgresExpenseRepository::new(pool.clone()));
    let invoice_line_item_repo = Arc::new(PostgresInvoiceLineItemRepository::new(pool.clone()));
    let payment_reminder_repo = Arc::new(PostgresPaymentReminderRepository::new(pool.clone()));
    let meeting_repo = Arc::new(PostgresMeetingRepository::new(pool.clone()));
    let resolution_repo = Arc::new(PostgresResolutionRepository::new(pool.clone()));
    let vote_repo = Arc::new(PostgresVoteRepository::new(pool.clone()));
    let ticket_repo = Arc::new(PostgresTicketRepository::new(pool.clone()));
    let notification_repo = Arc::new(PostgresNotificationRepository::new(pool.clone()));
    let notification_preference_repo =
        Arc::new(PostgresNotificationPreferenceRepository::new(pool.clone()));
    let payment_repo = Arc::new(PostgresPaymentRepository::new(pool.clone()));
    let payment_method_repo = Arc::new(PostgresPaymentMethodRepository::new(pool.clone()));
    let quote_repo = Arc::new(PostgresQuoteRepository::new(pool.clone()));
    let convocation_repo = Arc::new(PostgresConvocationRepository::new(pool.clone()));
    let convocation_recipient_repo =
        Arc::new(PostgresConvocationRecipientRepository::new(pool.clone()));
    let user_role_repo = Arc::new(PostgresUserRoleRepository::new(pool.clone()));
    let gdpr_repo = Arc::new(PostgresGdprRepository::new(pool.clone()));
    let local_exchange_repo = Arc::new(PostgresLocalExchangeRepository::new(pool.clone()));
    let owner_credit_balance_repo =
        Arc::new(PostgresOwnerCreditBalanceRepository::new(pool.clone()));
    let community_notice_repo = Arc::new(PostgresCommunityNoticeRepository::new(pool.clone()));
    let skills_directory_repo = Arc::new(PostgresSkillsDirectoryRepository::new(pool.clone()));
    let object_sharing_repo = Arc::new(PostgresObjectSharingRepository::new(pool.clone()));
    let resource_booking_repo = Arc::new(PostgresResourceBookingRepository::new(pool.clone()));
    let achievement_repo = Arc::new(PostgresAchievementRepository::new(pool.clone()));
    let user_achievement_repo = Arc::new(PostgresUserAchievementRepository::new(pool.clone()));
    let challenge_repo = Arc::new(PostgresChallengeRepository::new(pool.clone()));
    let challenge_progress_repo = Arc::new(PostgresChallengeProgressRepository::new(pool.clone()));

    // Initialize email service and storage provider
    let email_service = Arc::new(MockEmailService::new());
    let storage_provider = Arc::new(MockStorageProvider::new("/tmp/e2e_convocations_test"));

    // Initialize use cases
    let account_use_cases = AccountUseCases::new(account_repo.clone());
    let auth_use_cases = AuthUseCases::new(
        auth_repo.clone(),
        user_role_repo.clone(),
        "test_jwt_secret_key_minimum_32_chars".to_string(),
    );
    let building_use_cases = BuildingUseCases::new(building_repo.clone());
    let unit_use_cases = UnitUseCases::new(unit_repo.clone(), building_repo.clone());
    let unit_owner_use_cases = UnitOwnerUseCases::new(
        unit_owner_repo.clone(),
        unit_repo.clone(),
        owner_repo.clone(),
    );
    let owner_use_cases = OwnerUseCases::new(
        owner_repo.clone(),
        unit_repo.clone(),
        unit_owner_repo.clone(),
    );
    let expense_use_cases = ExpenseUseCases::new(
        expense_repo.clone(),
        invoice_line_item_repo.clone(),
        building_repo.clone(),
    );
    let payment_reminder_use_cases =
        PaymentReminderUseCases::new(payment_reminder_repo.clone(), expense_repo.clone());
    let financial_report_use_cases = FinancialReportUseCases::new(
        account_repo.clone(),
        expense_repo.clone(),
        invoice_line_item_repo.clone(),
    );
    let meeting_use_cases = MeetingUseCases::new(meeting_repo.clone(), building_repo.clone());
    let resolution_use_cases = ResolutionUseCases::new(
        resolution_repo.clone(),
        vote_repo.clone(),
        meeting_repo.clone(),
    );
    let ticket_use_cases = TicketUseCases::new(ticket_repo.clone(), building_repo.clone());
    let notification_use_cases = NotificationUseCases::new(
        notification_repo.clone(),
        notification_preference_repo.clone(),
    );
    let payment_use_cases = PaymentUseCases::new(
        payment_repo.clone(),
        expense_repo.clone(),
        owner_repo.clone(),
    );
    let payment_method_use_cases = PaymentMethodUseCases::new(payment_method_repo.clone());
    let quote_use_cases = QuoteUseCases::new(quote_repo.clone(), building_repo.clone());
    let convocation_use_cases = ConvocationUseCases::new(
        convocation_repo.clone(),
        convocation_recipient_repo.clone(),
        owner_repo.clone(),
        meeting_repo.clone(),
        email_service.clone(),
        storage_provider.clone(),
    );
    let gdpr_use_cases = GdprUseCases::new(gdpr_repo.clone(), auth_repo.clone());
    let local_exchange_use_cases = LocalExchangeUseCases::new(
        local_exchange_repo.clone(),
        owner_credit_balance_repo.clone(),
        owner_repo.clone(),
    );
    let community_notice_use_cases = CommunityNoticeUseCases::new(
        community_notice_repo.clone(),
        building_repo.clone(),
        auth_repo.clone(),
    );
    let skills_directory_use_cases = SkillsDirectoryUseCases::new(
        skills_directory_repo.clone(),
        building_repo.clone(),
        auth_repo.clone(),
    );
    let object_sharing_use_cases = ObjectSharingUseCases::new(
        object_sharing_repo.clone(),
        building_repo.clone(),
        auth_repo.clone(),
    );
    let resource_booking_use_cases = ResourceBookingUseCases::new(
        resource_booking_repo.clone(),
        building_repo.clone(),
        auth_repo.clone(),
    );
    let achievement_use_cases =
        AchievementUseCases::new(achievement_repo.clone(), user_achievement_repo.clone());
    let challenge_use_cases =
        ChallengeUseCases::new(challenge_repo.clone(), challenge_progress_repo.clone());
    let gamification_stats_use_cases = GamificationStatsUseCases::new(
        user_achievement_repo.clone(),
        achievement_repo.clone(),
        challenge_progress_repo.clone(),
        challenge_repo.clone(),
        auth_repo.clone(),
    );

    let app_state = web::Data::new(AppState::new(
        account_use_cases,
        auth_use_cases,
        building_use_cases,
        unit_use_cases,
        unit_owner_use_cases,
        owner_use_cases,
        expense_use_cases,
        payment_reminder_use_cases,
        financial_report_use_cases,
        meeting_use_cases,
        resolution_use_cases,
        ticket_use_cases,
        notification_use_cases,
        payment_use_cases,
        payment_method_use_cases,
        quote_use_cases,
        convocation_use_cases,
        gdpr_use_cases,
        local_exchange_use_cases,
        community_notice_use_cases,
        skills_directory_use_cases,
        object_sharing_use_cases,
        resource_booking_use_cases,
        achievement_use_cases,
        challenge_use_cases,
        gamification_stats_use_cases,
    ));

    (app_state, postgres_container)
}

async fn create_test_user(app_state: &web::Data<AppState>) -> (Uuid, String) {
    let email = format!("convocation_test_{}@example.com", Uuid::new_v4());
    let register_result = app_state
        .auth_use_cases
        .register(
            email.clone(),
            "TestPassword123!".to_string(),
            "Test".to_string(),
            "User".to_string(),
        )
        .await
        .expect("Failed to register test user");

    let login_result = app_state
        .auth_use_cases
        .login(email, "TestPassword123!".to_string())
        .await
        .expect("Failed to login test user");

    (register_result.id, login_result.token)
}

async fn create_test_building(app_state: &web::Data<AppState>, organization_id: Uuid) -> Uuid {
    let building_name = format!("Test Building {}", Uuid::new_v4());
    let building = app_state
        .building_use_cases
        .create_building(
            organization_id,
            building_name,
            "123 Test Street".to_string(),
            Some("Test City".to_string()),
            Some("12345".to_string()),
            Some("BE".to_string()),
            10,
            Some(2020),
        )
        .await
        .expect("Failed to create test building");

    building.id
}

async fn create_test_meeting(
    app_state: &web::Data<AppState>,
    organization_id: Uuid,
    building_id: Uuid,
    meeting_date: DateTime<Utc>,
) -> Uuid {
    let meeting = app_state
        .meeting_use_cases
        .create_meeting(
            organization_id,
            building_id,
            "Ordinary".to_string(),
            format!("Test Meeting {}", Uuid::new_v4()),
            Some("Annual general assembly".to_string()),
            meeting_date,
            Some("Building main hall".to_string()),
        )
        .await
        .expect("Failed to create test meeting");

    meeting.id
}

async fn create_test_owner(app_state: &web::Data<AppState>, organization_id: Uuid) -> Uuid {
    let email = format!("owner_{}@example.com", Uuid::new_v4());
    let owner = app_state
        .owner_use_cases
        .create_owner(
            organization_id,
            "Test".to_string(),
            "Owner".to_string(),
            email,
            Some("+32123456789".to_string()),
        )
        .await
        .expect("Failed to create test owner");

    owner.id
}

// ==================== Convocation CRUD Tests ====================

#[actix_web::test]
#[serial]
async fn test_create_convocation_success() {
    let (app_state, _container) = setup_app().await;
    let (user_id, token) = create_test_user(&app_state).await;
    let organization_id = user_id;
    let building_id = create_test_building(&app_state, organization_id).await;

    // Create meeting 30 days in future (well beyond 15d requirement)
    let meeting_date = Utc::now() + Duration::days(30);
    let meeting_id =
        create_test_meeting(&app_state, organization_id, building_id, meeting_date).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(create_authenticated_app),
    )
    .await;

    let create_req = test::TestRequest::post()
        .uri("/api/v1/convocations")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "building_id": building_id.to_string(),
            "meeting_id": meeting_id.to_string(),
            "meeting_type": "Ordinary",
            "meeting_date": meeting_date.to_rfc3339(),
            "language": "FR"
        }))
        .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    assert_eq!(
        create_resp.status(),
        201,
        "Expected 201 Created for convocation creation"
    );

    let convocation: ConvocationResponse = test::read_body_json(create_resp).await;
    assert_eq!(convocation.building_id, building_id);
    assert_eq!(convocation.meeting_id, meeting_id);
    assert_eq!(convocation.language, "FR");
    assert_eq!(convocation.status, ConvocationStatus::Draft);
    assert!(
        convocation.respects_legal_deadline,
        "Should respect 15-day deadline for Ordinary AG"
    );
}

#[actix_web::test]
#[serial]
async fn test_create_convocation_without_auth() {
    let (app_state, _container) = setup_app().await;
    let (user_id, _token) = create_test_user(&app_state).await;
    let organization_id = user_id;
    let building_id = create_test_building(&app_state, organization_id).await;

    let meeting_date = Utc::now() + Duration::days(30);
    let meeting_id =
        create_test_meeting(&app_state, organization_id, building_id, meeting_date).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(create_authenticated_app),
    )
    .await;

    let create_req = test::TestRequest::post()
        .uri("/api/v1/convocations")
        .set_json(json!({
            "building_id": building_id.to_string(),
            "meeting_id": meeting_id.to_string(),
            "meeting_type": "Ordinary",
            "meeting_date": meeting_date.to_rfc3339(),
            "language": "FR"
        }))
        .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    assert_eq!(
        create_resp.status(),
        401,
        "Expected 401 Unauthorized without authentication"
    );
}

#[actix_web::test]
#[serial]
async fn test_create_convocation_all_meeting_types() {
    let (app_state, _container) = setup_app().await;
    let (user_id, token) = create_test_user(&app_state).await;
    let organization_id = user_id;
    let building_id = create_test_building(&app_state, organization_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(create_authenticated_app),
    )
    .await;

    let meeting_types = vec![
        ("Ordinary", 30),          // 15-day requirement
        ("Extraordinary", 20),     // 8-day requirement
        ("SecondConvocation", 20), // 8-day requirement
    ];

    for (meeting_type, days_ahead) in meeting_types {
        let meeting_date = Utc::now() + Duration::days(days_ahead);
        let meeting_id =
            create_test_meeting(&app_state, organization_id, building_id, meeting_date).await;

        let create_req = test::TestRequest::post()
            .uri("/api/v1/convocations")
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
            .set_json(json!({
                "building_id": building_id.to_string(),
                "meeting_id": meeting_id.to_string(),
                "meeting_type": meeting_type,
                "meeting_date": meeting_date.to_rfc3339(),
                "language": "FR"
            }))
            .to_request();

        let create_resp = test::call_service(&app, create_req).await;
        assert_eq!(
            create_resp.status(),
            201,
            "Expected 201 Created for {} meeting type",
            meeting_type
        );

        let convocation: ConvocationResponse = test::read_body_json(create_resp).await;
        assert!(
            convocation.respects_legal_deadline,
            "{} meeting should respect legal deadline",
            meeting_type
        );
    }
}

#[actix_web::test]
#[serial]
async fn test_create_convocation_all_languages() {
    let (app_state, _container) = setup_app().await;
    let (user_id, token) = create_test_user(&app_state).await;
    let organization_id = user_id;
    let building_id = create_test_building(&app_state, organization_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(create_authenticated_app),
    )
    .await;

    let languages = vec!["FR", "NL", "DE", "EN"];

    for language in languages {
        let meeting_date = Utc::now() + Duration::days(30);
        let meeting_id =
            create_test_meeting(&app_state, organization_id, building_id, meeting_date).await;

        let create_req = test::TestRequest::post()
            .uri("/api/v1/convocations")
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
            .set_json(json!({
                "building_id": building_id.to_string(),
                "meeting_id": meeting_id.to_string(),
                "meeting_type": "Ordinary",
                "meeting_date": meeting_date.to_rfc3339(),
                "language": language
            }))
            .to_request();

        let create_resp = test::call_service(&app, create_req).await;
        assert_eq!(
            create_resp.status(),
            201,
            "Expected 201 Created for language {}",
            language
        );

        let convocation: ConvocationResponse = test::read_body_json(create_resp).await;
        assert_eq!(convocation.language, language);
    }
}

#[actix_web::test]
#[serial]
async fn test_get_convocation_by_id() {
    let (app_state, _container) = setup_app().await;
    let (user_id, token) = create_test_user(&app_state).await;
    let organization_id = user_id;
    let building_id = create_test_building(&app_state, organization_id).await;

    let meeting_date = Utc::now() + Duration::days(30);
    let meeting_id =
        create_test_meeting(&app_state, organization_id, building_id, meeting_date).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(create_authenticated_app),
    )
    .await;

    // Create convocation
    let create_req = test::TestRequest::post()
        .uri("/api/v1/convocations")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "building_id": building_id.to_string(),
            "meeting_id": meeting_id.to_string(),
            "meeting_type": "Ordinary",
            "meeting_date": meeting_date.to_rfc3339(),
            "language": "FR"
        }))
        .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    let created_convocation: ConvocationResponse = test::read_body_json(create_resp).await;

    // Get by ID
    let get_req = test::TestRequest::get()
        .uri(&format!("/api/v1/convocations/{}", created_convocation.id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let get_resp = test::call_service(&app, get_req).await;
    assert_eq!(get_resp.status(), 200);

    let fetched_convocation: ConvocationResponse = test::read_body_json(get_resp).await;
    assert_eq!(fetched_convocation.id, created_convocation.id);
    assert_eq!(fetched_convocation.building_id, building_id);
}

#[actix_web::test]
#[serial]
async fn test_get_convocation_not_found() {
    let (app_state, _container) = setup_app().await;
    let (_user_id, token) = create_test_user(&app_state).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(create_authenticated_app),
    )
    .await;

    let non_existent_id = Uuid::new_v4();
    let get_req = test::TestRequest::get()
        .uri(&format!("/api/v1/convocations/{}", non_existent_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let get_resp = test::call_service(&app, get_req).await;
    assert_eq!(get_resp.status(), 404);
}

#[actix_web::test]
#[serial]
async fn test_get_convocation_by_meeting() {
    let (app_state, _container) = setup_app().await;
    let (user_id, token) = create_test_user(&app_state).await;
    let organization_id = user_id;
    let building_id = create_test_building(&app_state, organization_id).await;

    let meeting_date = Utc::now() + Duration::days(30);
    let meeting_id =
        create_test_meeting(&app_state, organization_id, building_id, meeting_date).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(create_authenticated_app),
    )
    .await;

    // Create convocation
    let create_req = test::TestRequest::post()
        .uri("/api/v1/convocations")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "building_id": building_id.to_string(),
            "meeting_id": meeting_id.to_string(),
            "meeting_type": "Ordinary",
            "meeting_date": meeting_date.to_rfc3339(),
            "language": "FR"
        }))
        .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    let created_convocation: ConvocationResponse = test::read_body_json(create_resp).await;

    // Get by meeting ID
    let get_req = test::TestRequest::get()
        .uri(&format!("/api/v1/meetings/{}/convocation", meeting_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let get_resp = test::call_service(&app, get_req).await;
    assert_eq!(get_resp.status(), 200);

    let fetched_convocation: ConvocationResponse = test::read_body_json(get_resp).await;
    assert_eq!(fetched_convocation.id, created_convocation.id);
    assert_eq!(fetched_convocation.meeting_id, meeting_id);
}

#[actix_web::test]
#[serial]
async fn test_list_building_convocations() {
    let (app_state, _container) = setup_app().await;
    let (user_id, token) = create_test_user(&app_state).await;
    let organization_id = user_id;
    let building_id = create_test_building(&app_state, organization_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(create_authenticated_app),
    )
    .await;

    // Create 3 convocations for the same building
    for i in 0..3 {
        let meeting_date = Utc::now() + Duration::days(30 + i);
        let meeting_id =
            create_test_meeting(&app_state, organization_id, building_id, meeting_date).await;

        let create_req = test::TestRequest::post()
            .uri("/api/v1/convocations")
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
            .set_json(json!({
                "building_id": building_id.to_string(),
                "meeting_id": meeting_id.to_string(),
                "meeting_type": "Ordinary",
                "meeting_date": meeting_date.to_rfc3339(),
                "language": "FR"
            }))
            .to_request();

        test::call_service(&app, create_req).await;
    }

    // List building convocations
    let list_req = test::TestRequest::get()
        .uri(&format!("/api/v1/buildings/{}/convocations", building_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let list_resp = test::call_service(&app, list_req).await;
    assert_eq!(list_resp.status(), 200);

    let convocations: Vec<ConvocationResponse> = test::read_body_json(list_resp).await;
    assert_eq!(
        convocations.len(),
        3,
        "Expected 3 convocations for building"
    );
}

#[actix_web::test]
#[serial]
async fn test_list_organization_convocations() {
    let (app_state, _container) = setup_app().await;
    let (user_id, token) = create_test_user(&app_state).await;
    let organization_id = user_id;
    let building1_id = create_test_building(&app_state, organization_id).await;
    let building2_id = create_test_building(&app_state, organization_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(create_authenticated_app),
    )
    .await;

    // Create 2 convocations for building1
    for _ in 0..2 {
        let meeting_date = Utc::now() + Duration::days(30);
        let meeting_id =
            create_test_meeting(&app_state, organization_id, building1_id, meeting_date).await;

        let create_req = test::TestRequest::post()
            .uri("/api/v1/convocations")
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
            .set_json(json!({
                "building_id": building1_id.to_string(),
                "meeting_id": meeting_id.to_string(),
                "meeting_type": "Ordinary",
                "meeting_date": meeting_date.to_rfc3339(),
                "language": "FR"
            }))
            .to_request();

        test::call_service(&app, create_req).await;
    }

    // Create 1 convocation for building2
    let meeting_date = Utc::now() + Duration::days(30);
    let meeting_id =
        create_test_meeting(&app_state, organization_id, building2_id, meeting_date).await;

    let create_req = test::TestRequest::post()
        .uri("/api/v1/convocations")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "building_id": building2_id.to_string(),
            "meeting_id": meeting_id.to_string(),
            "meeting_type": "Ordinary",
            "meeting_date": meeting_date.to_rfc3339(),
            "language": "FR"
        }))
        .to_request();

    test::call_service(&app, create_req).await;

    // List organization convocations
    let list_req = test::TestRequest::get()
        .uri(&format!(
            "/api/v1/organizations/{}/convocations",
            organization_id
        ))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let list_resp = test::call_service(&app, list_req).await;
    assert_eq!(list_resp.status(), 200);

    let convocations: Vec<ConvocationResponse> = test::read_body_json(list_resp).await;
    assert!(
        convocations.len() >= 3,
        "Expected at least 3 convocations for organization"
    );
}

#[actix_web::test]
#[serial]
async fn test_delete_convocation() {
    let (app_state, _container) = setup_app().await;
    let (user_id, token) = create_test_user(&app_state).await;
    let organization_id = user_id;
    let building_id = create_test_building(&app_state, organization_id).await;

    let meeting_date = Utc::now() + Duration::days(30);
    let meeting_id =
        create_test_meeting(&app_state, organization_id, building_id, meeting_date).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(create_authenticated_app),
    )
    .await;

    // Create convocation
    let create_req = test::TestRequest::post()
        .uri("/api/v1/convocations")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "building_id": building_id.to_string(),
            "meeting_id": meeting_id.to_string(),
            "meeting_type": "Ordinary",
            "meeting_date": meeting_date.to_rfc3339(),
            "language": "FR"
        }))
        .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    let created_convocation: ConvocationResponse = test::read_body_json(create_resp).await;

    // Delete convocation
    let delete_req = test::TestRequest::delete()
        .uri(&format!("/api/v1/convocations/{}", created_convocation.id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let delete_resp = test::call_service(&app, delete_req).await;
    assert_eq!(delete_resp.status(), 204);

    // Verify deletion
    let get_req = test::TestRequest::get()
        .uri(&format!("/api/v1/convocations/{}", created_convocation.id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let get_resp = test::call_service(&app, get_req).await;
    assert_eq!(get_resp.status(), 404, "Expected 404 after deletion");
}

// ==================== Workflow Tests ====================

#[actix_web::test]
#[serial]
async fn test_schedule_convocation() {
    let (app_state, _container) = setup_app().await;
    let (user_id, token) = create_test_user(&app_state).await;
    let organization_id = user_id;
    let building_id = create_test_building(&app_state, organization_id).await;

    let meeting_date = Utc::now() + Duration::days(30);
    let meeting_id =
        create_test_meeting(&app_state, organization_id, building_id, meeting_date).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(create_authenticated_app),
    )
    .await;

    // Create convocation
    let create_req = test::TestRequest::post()
        .uri("/api/v1/convocations")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "building_id": building_id.to_string(),
            "meeting_id": meeting_id.to_string(),
            "meeting_type": "Ordinary",
            "meeting_date": meeting_date.to_rfc3339(),
            "language": "FR"
        }))
        .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    let convocation: ConvocationResponse = test::read_body_json(create_resp).await;

    // Schedule send date (20 days from now, respects 15-day deadline)
    let send_date = Utc::now() + Duration::days(10);

    let schedule_req = test::TestRequest::put()
        .uri(&format!("/api/v1/convocations/{}/schedule", convocation.id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "send_date": send_date.to_rfc3339()
        }))
        .to_request();

    let schedule_resp = test::call_service(&app, schedule_req).await;
    assert_eq!(schedule_resp.status(), 200);

    let scheduled_convocation: ConvocationResponse = test::read_body_json(schedule_resp).await;
    assert_eq!(scheduled_convocation.status, ConvocationStatus::Scheduled);
    assert!(scheduled_convocation.scheduled_send_date.is_some());
}

#[actix_web::test]
#[serial]
async fn test_send_convocation() {
    let (app_state, _container) = setup_app().await;
    let (user_id, token) = create_test_user(&app_state).await;
    let organization_id = user_id;
    let building_id = create_test_building(&app_state, organization_id).await;

    // Create 2 owners as recipients
    let owner1_id = create_test_owner(&app_state, organization_id).await;
    let owner2_id = create_test_owner(&app_state, organization_id).await;

    let meeting_date = Utc::now() + Duration::days(30);
    let meeting_id =
        create_test_meeting(&app_state, organization_id, building_id, meeting_date).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(create_authenticated_app),
    )
    .await;

    // Create convocation
    let create_req = test::TestRequest::post()
        .uri("/api/v1/convocations")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "building_id": building_id.to_string(),
            "meeting_id": meeting_id.to_string(),
            "meeting_type": "Ordinary",
            "meeting_date": meeting_date.to_rfc3339(),
            "language": "FR"
        }))
        .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    let convocation: ConvocationResponse = test::read_body_json(create_resp).await;

    // Send convocation
    let send_req = test::TestRequest::post()
        .uri(&format!("/api/v1/convocations/{}/send", convocation.id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "recipient_owner_ids": [owner1_id.to_string(), owner2_id.to_string()]
        }))
        .to_request();

    let send_resp = test::call_service(&app, send_req).await;
    assert_eq!(send_resp.status(), 200);

    let sent_convocation: ConvocationResponse = test::read_body_json(send_resp).await;
    assert_eq!(sent_convocation.status, ConvocationStatus::Sent);
    assert_eq!(sent_convocation.total_recipients, 2);
    assert!(sent_convocation.actual_send_date.is_some());
}

#[actix_web::test]
#[serial]
async fn test_cancel_convocation() {
    let (app_state, _container) = setup_app().await;
    let (user_id, token) = create_test_user(&app_state).await;
    let organization_id = user_id;
    let building_id = create_test_building(&app_state, organization_id).await;

    let meeting_date = Utc::now() + Duration::days(30);
    let meeting_id =
        create_test_meeting(&app_state, organization_id, building_id, meeting_date).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(create_authenticated_app),
    )
    .await;

    // Create convocation
    let create_req = test::TestRequest::post()
        .uri("/api/v1/convocations")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "building_id": building_id.to_string(),
            "meeting_id": meeting_id.to_string(),
            "meeting_type": "Ordinary",
            "meeting_date": meeting_date.to_rfc3339(),
            "language": "FR"
        }))
        .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    let convocation: ConvocationResponse = test::read_body_json(create_resp).await;

    // Cancel convocation
    let cancel_req = test::TestRequest::put()
        .uri(&format!("/api/v1/convocations/{}/cancel", convocation.id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let cancel_resp = test::call_service(&app, cancel_req).await;
    assert_eq!(cancel_resp.status(), 200);

    let cancelled_convocation: ConvocationResponse = test::read_body_json(cancel_resp).await;
    assert_eq!(cancelled_convocation.status, ConvocationStatus::Cancelled);
}

// ==================== Recipient Tracking Tests ====================

#[actix_web::test]
#[serial]
async fn test_list_convocation_recipients() {
    let (app_state, _container) = setup_app().await;
    let (user_id, token) = create_test_user(&app_state).await;
    let organization_id = user_id;
    let building_id = create_test_building(&app_state, organization_id).await;

    let owner1_id = create_test_owner(&app_state, organization_id).await;
    let owner2_id = create_test_owner(&app_state, organization_id).await;

    let meeting_date = Utc::now() + Duration::days(30);
    let meeting_id =
        create_test_meeting(&app_state, organization_id, building_id, meeting_date).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(create_authenticated_app),
    )
    .await;

    // Create and send convocation
    let create_req = test::TestRequest::post()
        .uri("/api/v1/convocations")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "building_id": building_id.to_string(),
            "meeting_id": meeting_id.to_string(),
            "meeting_type": "Ordinary",
            "meeting_date": meeting_date.to_rfc3339(),
            "language": "FR"
        }))
        .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    let convocation: ConvocationResponse = test::read_body_json(create_resp).await;

    let send_req = test::TestRequest::post()
        .uri(&format!("/api/v1/convocations/{}/send", convocation.id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "recipient_owner_ids": [owner1_id.to_string(), owner2_id.to_string()]
        }))
        .to_request();

    test::call_service(&app, send_req).await;

    // List recipients
    let list_req = test::TestRequest::get()
        .uri(&format!(
            "/api/v1/convocations/{}/recipients",
            convocation.id
        ))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let list_resp = test::call_service(&app, list_req).await;
    assert_eq!(list_resp.status(), 200);

    let recipients: Vec<ConvocationRecipientResponse> = test::read_body_json(list_resp).await;
    assert_eq!(recipients.len(), 2, "Expected 2 recipients");
}

#[actix_web::test]
#[serial]
async fn test_get_tracking_summary() {
    let (app_state, _container) = setup_app().await;
    let (user_id, token) = create_test_user(&app_state).await;
    let organization_id = user_id;
    let building_id = create_test_building(&app_state, organization_id).await;

    let owner1_id = create_test_owner(&app_state, organization_id).await;
    let owner2_id = create_test_owner(&app_state, organization_id).await;

    let meeting_date = Utc::now() + Duration::days(30);
    let meeting_id =
        create_test_meeting(&app_state, organization_id, building_id, meeting_date).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(create_authenticated_app),
    )
    .await;

    // Create and send convocation
    let create_req = test::TestRequest::post()
        .uri("/api/v1/convocations")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "building_id": building_id.to_string(),
            "meeting_id": meeting_id.to_string(),
            "meeting_type": "Ordinary",
            "meeting_date": meeting_date.to_rfc3339(),
            "language": "FR"
        }))
        .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    let convocation: ConvocationResponse = test::read_body_json(create_resp).await;

    let send_req = test::TestRequest::post()
        .uri(&format!("/api/v1/convocations/{}/send", convocation.id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "recipient_owner_ids": [owner1_id.to_string(), owner2_id.to_string()]
        }))
        .to_request();

    test::call_service(&app, send_req).await;

    // Get tracking summary
    let summary_req = test::TestRequest::get()
        .uri(&format!(
            "/api/v1/convocations/{}/tracking-summary",
            convocation.id
        ))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let summary_resp = test::call_service(&app, summary_req).await;
    assert_eq!(summary_resp.status(), 200);

    let summary: RecipientTrackingSummaryResponse = test::read_body_json(summary_resp).await;
    assert_eq!(summary.total_count, 2);
    assert_eq!(summary.pending_count, 2); // No actions yet
}

#[actix_web::test]
#[serial]
async fn test_mark_recipient_email_opened() {
    let (app_state, _container) = setup_app().await;
    let (user_id, token) = create_test_user(&app_state).await;
    let organization_id = user_id;
    let building_id = create_test_building(&app_state, organization_id).await;

    let owner_id = create_test_owner(&app_state, organization_id).await;

    let meeting_date = Utc::now() + Duration::days(30);
    let meeting_id =
        create_test_meeting(&app_state, organization_id, building_id, meeting_date).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(create_authenticated_app),
    )
    .await;

    // Create and send convocation
    let create_req = test::TestRequest::post()
        .uri("/api/v1/convocations")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "building_id": building_id.to_string(),
            "meeting_id": meeting_id.to_string(),
            "meeting_type": "Ordinary",
            "meeting_date": meeting_date.to_rfc3339(),
            "language": "FR"
        }))
        .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    let convocation: ConvocationResponse = test::read_body_json(create_resp).await;

    let send_req = test::TestRequest::post()
        .uri(&format!("/api/v1/convocations/{}/send", convocation.id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "recipient_owner_ids": [owner_id.to_string()]
        }))
        .to_request();

    test::call_service(&app, send_req).await;

    // Get recipient
    let list_req = test::TestRequest::get()
        .uri(&format!(
            "/api/v1/convocations/{}/recipients",
            convocation.id
        ))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let list_resp = test::call_service(&app, list_req).await;
    let recipients: Vec<ConvocationRecipientResponse> = test::read_body_json(list_resp).await;
    let recipient = &recipients[0];

    // Mark email opened
    let mark_opened_req = test::TestRequest::put()
        .uri(&format!(
            "/api/v1/convocation-recipients/{}/email-opened",
            recipient.id
        ))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let mark_opened_resp = test::call_service(&app, mark_opened_req).await;
    assert_eq!(mark_opened_resp.status(), 200);

    let updated_recipient: ConvocationRecipientResponse =
        test::read_body_json(mark_opened_resp).await;
    assert!(updated_recipient.has_opened_email);
    assert!(updated_recipient.email_opened_at.is_some());
}

#[actix_web::test]
#[serial]
async fn test_update_recipient_attendance() {
    let (app_state, _container) = setup_app().await;
    let (user_id, token) = create_test_user(&app_state).await;
    let organization_id = user_id;
    let building_id = create_test_building(&app_state, organization_id).await;

    let owner_id = create_test_owner(&app_state, organization_id).await;

    let meeting_date = Utc::now() + Duration::days(30);
    let meeting_id =
        create_test_meeting(&app_state, organization_id, building_id, meeting_date).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(create_authenticated_app),
    )
    .await;

    // Create and send convocation
    let create_req = test::TestRequest::post()
        .uri("/api/v1/convocations")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "building_id": building_id.to_string(),
            "meeting_id": meeting_id.to_string(),
            "meeting_type": "Ordinary",
            "meeting_date": meeting_date.to_rfc3339(),
            "language": "FR"
        }))
        .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    let convocation: ConvocationResponse = test::read_body_json(create_resp).await;

    let send_req = test::TestRequest::post()
        .uri(&format!("/api/v1/convocations/{}/send", convocation.id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "recipient_owner_ids": [owner_id.to_string()]
        }))
        .to_request();

    test::call_service(&app, send_req).await;

    // Get recipient
    let list_req = test::TestRequest::get()
        .uri(&format!(
            "/api/v1/convocations/{}/recipients",
            convocation.id
        ))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let list_resp = test::call_service(&app, list_req).await;
    let recipients: Vec<ConvocationRecipientResponse> = test::read_body_json(list_resp).await;
    let recipient = &recipients[0];

    // Update attendance to WillAttend
    let update_req = test::TestRequest::put()
        .uri(&format!(
            "/api/v1/convocation-recipients/{}/attendance",
            recipient.id
        ))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "attendance_status": "WillAttend"
        }))
        .to_request();

    let update_resp = test::call_service(&app, update_req).await;
    assert_eq!(update_resp.status(), 200);

    let updated_recipient: ConvocationRecipientResponse = test::read_body_json(update_resp).await;
    assert_eq!(
        updated_recipient.attendance_status,
        AttendanceStatus::WillAttend
    );
    assert!(updated_recipient.has_confirmed_attendance);
}

#[actix_web::test]
#[serial]
async fn test_update_recipient_attendance_all_statuses() {
    let (app_state, _container) = setup_app().await;
    let (user_id, token) = create_test_user(&app_state).await;
    let organization_id = user_id;
    let building_id = create_test_building(&app_state, organization_id).await;

    let meeting_date = Utc::now() + Duration::days(30);
    let meeting_id =
        create_test_meeting(&app_state, organization_id, building_id, meeting_date).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(create_authenticated_app),
    )
    .await;

    let statuses = vec![
        "Pending",
        "WillAttend",
        "WillNotAttend",
        "Attended",
        "DidNotAttend",
    ];

    for status in statuses {
        let owner_id = create_test_owner(&app_state, organization_id).await;

        // Create and send convocation
        let create_req = test::TestRequest::post()
            .uri("/api/v1/convocations")
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
            .set_json(json!({
                "building_id": building_id.to_string(),
                "meeting_id": meeting_id.to_string(),
                "meeting_type": "Ordinary",
                "meeting_date": meeting_date.to_rfc3339(),
                "language": "FR"
            }))
            .to_request();

        let create_resp = test::call_service(&app, create_req).await;
        let convocation: ConvocationResponse = test::read_body_json(create_resp).await;

        let send_req = test::TestRequest::post()
            .uri(&format!("/api/v1/convocations/{}/send", convocation.id))
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
            .set_json(json!({
                "recipient_owner_ids": [owner_id.to_string()]
            }))
            .to_request();

        test::call_service(&app, send_req).await;

        // Get recipient
        let list_req = test::TestRequest::get()
            .uri(&format!(
                "/api/v1/convocations/{}/recipients",
                convocation.id
            ))
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
            .to_request();

        let list_resp = test::call_service(&app, list_req).await;
        let recipients: Vec<ConvocationRecipientResponse> = test::read_body_json(list_resp).await;
        let recipient = &recipients[0];

        // Update attendance
        let update_req = test::TestRequest::put()
            .uri(&format!(
                "/api/v1/convocation-recipients/{}/attendance",
                recipient.id
            ))
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
            .set_json(json!({
                "attendance_status": status
            }))
            .to_request();

        let update_resp = test::call_service(&app, update_req).await;
        assert_eq!(
            update_resp.status(),
            200,
            "Expected 200 for attendance status {}",
            status
        );
    }
}

#[actix_web::test]
#[serial]
async fn test_set_recipient_proxy() {
    let (app_state, _container) = setup_app().await;
    let (user_id, token) = create_test_user(&app_state).await;
    let organization_id = user_id;
    let building_id = create_test_building(&app_state, organization_id).await;

    let owner1_id = create_test_owner(&app_state, organization_id).await;
    let owner2_id = create_test_owner(&app_state, organization_id).await; // Proxy

    let meeting_date = Utc::now() + Duration::days(30);
    let meeting_id =
        create_test_meeting(&app_state, organization_id, building_id, meeting_date).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(create_authenticated_app),
    )
    .await;

    // Create and send convocation
    let create_req = test::TestRequest::post()
        .uri("/api/v1/convocations")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "building_id": building_id.to_string(),
            "meeting_id": meeting_id.to_string(),
            "meeting_type": "Ordinary",
            "meeting_date": meeting_date.to_rfc3339(),
            "language": "FR"
        }))
        .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    let convocation: ConvocationResponse = test::read_body_json(create_resp).await;

    let send_req = test::TestRequest::post()
        .uri(&format!("/api/v1/convocations/{}/send", convocation.id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "recipient_owner_ids": [owner1_id.to_string()]
        }))
        .to_request();

    test::call_service(&app, send_req).await;

    // Get recipient
    let list_req = test::TestRequest::get()
        .uri(&format!(
            "/api/v1/convocations/{}/recipients",
            convocation.id
        ))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let list_resp = test::call_service(&app, list_req).await;
    let recipients: Vec<ConvocationRecipientResponse> = test::read_body_json(list_resp).await;
    let recipient = &recipients[0];

    // Set proxy
    let proxy_req = test::TestRequest::put()
        .uri(&format!(
            "/api/v1/convocation-recipients/{}/proxy",
            recipient.id
        ))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "proxy_owner_id": owner2_id.to_string()
        }))
        .to_request();

    let proxy_resp = test::call_service(&app, proxy_req).await;
    assert_eq!(proxy_resp.status(), 200);

    let updated_recipient: ConvocationRecipientResponse = test::read_body_json(proxy_resp).await;
    assert_eq!(updated_recipient.proxy_owner_id, Some(owner2_id));
}

#[actix_web::test]
#[serial]
async fn test_send_convocation_reminders() {
    let (app_state, _container) = setup_app().await;
    let (user_id, token) = create_test_user(&app_state).await;
    let organization_id = user_id;
    let building_id = create_test_building(&app_state, organization_id).await;

    let owner_id = create_test_owner(&app_state, organization_id).await;

    let meeting_date = Utc::now() + Duration::days(30);
    let meeting_id =
        create_test_meeting(&app_state, organization_id, building_id, meeting_date).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(create_authenticated_app),
    )
    .await;

    // Create and send convocation
    let create_req = test::TestRequest::post()
        .uri("/api/v1/convocations")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "building_id": building_id.to_string(),
            "meeting_id": meeting_id.to_string(),
            "meeting_type": "Ordinary",
            "meeting_date": meeting_date.to_rfc3339(),
            "language": "FR"
        }))
        .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    let convocation: ConvocationResponse = test::read_body_json(create_resp).await;

    let send_req = test::TestRequest::post()
        .uri(&format!("/api/v1/convocations/{}/send", convocation.id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "recipient_owner_ids": [owner_id.to_string()]
        }))
        .to_request();

    test::call_service(&app, send_req).await;

    // Send reminders
    let reminder_req = test::TestRequest::post()
        .uri(&format!(
            "/api/v1/convocations/{}/reminders",
            convocation.id
        ))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let reminder_resp = test::call_service(&app, reminder_req).await;
    assert_eq!(reminder_resp.status(), 200);
}

// ==================== Complete Lifecycle Test ====================

#[actix_web::test]
#[serial]
async fn test_complete_convocation_lifecycle() {
    let (app_state, _container) = setup_app().await;
    let (user_id, token) = create_test_user(&app_state).await;
    let organization_id = user_id;
    let building_id = create_test_building(&app_state, organization_id).await;

    let owner1_id = create_test_owner(&app_state, organization_id).await;
    let owner2_id = create_test_owner(&app_state, organization_id).await;

    let meeting_date = Utc::now() + Duration::days(30);
    let meeting_id =
        create_test_meeting(&app_state, organization_id, building_id, meeting_date).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(create_authenticated_app),
    )
    .await;

    // 1. Create convocation (Draft)
    let create_req = test::TestRequest::post()
        .uri("/api/v1/convocations")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "building_id": building_id.to_string(),
            "meeting_id": meeting_id.to_string(),
            "meeting_type": "Ordinary",
            "meeting_date": meeting_date.to_rfc3339(),
            "language": "FR"
        }))
        .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    let convocation: ConvocationResponse = test::read_body_json(create_resp).await;
    assert_eq!(convocation.status, ConvocationStatus::Draft);

    // 2. Schedule convocation (Draft → Scheduled)
    let send_date = Utc::now() + Duration::days(10);
    let schedule_req = test::TestRequest::put()
        .uri(&format!("/api/v1/convocations/{}/schedule", convocation.id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "send_date": send_date.to_rfc3339()
        }))
        .to_request();

    let schedule_resp = test::call_service(&app, schedule_req).await;
    let scheduled_convocation: ConvocationResponse = test::read_body_json(schedule_resp).await;
    assert_eq!(scheduled_convocation.status, ConvocationStatus::Scheduled);

    // 3. Send convocation (Scheduled → Sent)
    let send_req = test::TestRequest::post()
        .uri(&format!("/api/v1/convocations/{}/send", convocation.id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "recipient_owner_ids": [owner1_id.to_string(), owner2_id.to_string()]
        }))
        .to_request();

    let send_resp = test::call_service(&app, send_req).await;
    let sent_convocation: ConvocationResponse = test::read_body_json(send_resp).await;
    assert_eq!(sent_convocation.status, ConvocationStatus::Sent);
    assert_eq!(sent_convocation.total_recipients, 2);

    // 4. Get recipients
    let list_req = test::TestRequest::get()
        .uri(&format!(
            "/api/v1/convocations/{}/recipients",
            convocation.id
        ))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let list_resp = test::call_service(&app, list_req).await;
    let recipients: Vec<ConvocationRecipientResponse> = test::read_body_json(list_resp).await;
    assert_eq!(recipients.len(), 2);

    // 5. Owner 1 opens email
    let mark_opened_req = test::TestRequest::put()
        .uri(&format!(
            "/api/v1/convocation-recipients/{}/email-opened",
            recipients[0].id
        ))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    test::call_service(&app, mark_opened_req).await;

    // 6. Owner 1 confirms attendance
    let attendance_req = test::TestRequest::put()
        .uri(&format!(
            "/api/v1/convocation-recipients/{}/attendance",
            recipients[0].id
        ))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "attendance_status": "WillAttend"
        }))
        .to_request();

    test::call_service(&app, attendance_req).await;

    // 7. Get tracking summary
    let summary_req = test::TestRequest::get()
        .uri(&format!(
            "/api/v1/convocations/{}/tracking-summary",
            convocation.id
        ))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let summary_resp = test::call_service(&app, summary_req).await;
    let summary: RecipientTrackingSummaryResponse = test::read_body_json(summary_resp).await;

    assert_eq!(summary.total_count, 2);
    assert_eq!(summary.opened_count, 1);
    assert_eq!(summary.will_attend_count, 1);
    assert_eq!(summary.opening_rate, 50.0); // 1/2 * 100
    assert_eq!(summary.attendance_rate, 50.0); // 1/2 * 100

    // 8. Send reminders to unopened emails
    let reminder_req = test::TestRequest::post()
        .uri(&format!(
            "/api/v1/convocations/{}/reminders",
            convocation.id
        ))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let reminder_resp = test::call_service(&app, reminder_req).await;
    assert_eq!(reminder_resp.status(), 200);
}

// ==================== Belgian Legal Deadline Tests ====================

#[actix_web::test]
#[serial]
async fn test_legal_deadline_ordinary_ag() {
    let (app_state, _container) = setup_app().await;
    let (user_id, token) = create_test_user(&app_state).await;
    let organization_id = user_id;
    let building_id = create_test_building(&app_state, organization_id).await;

    // Ordinary AG requires 15 days minimum notice
    // Meeting 16 days from now = respects deadline
    let meeting_date = Utc::now() + Duration::days(16);
    let meeting_id =
        create_test_meeting(&app_state, organization_id, building_id, meeting_date).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(create_authenticated_app),
    )
    .await;

    let create_req = test::TestRequest::post()
        .uri("/api/v1/convocations")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "building_id": building_id.to_string(),
            "meeting_id": meeting_id.to_string(),
            "meeting_type": "Ordinary",
            "meeting_date": meeting_date.to_rfc3339(),
            "language": "FR"
        }))
        .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    let convocation: ConvocationResponse = test::read_body_json(create_resp).await;

    // minimum_send_date should be meeting_date - 15 days
    let expected_minimum_send = meeting_date - Duration::days(15);
    assert!(
        convocation.minimum_send_date <= expected_minimum_send + Duration::seconds(5),
        "Minimum send date should be at least 15 days before meeting"
    );

    assert!(
        convocation.respects_legal_deadline,
        "Ordinary AG with 16 days notice should respect legal deadline"
    );
}

#[actix_web::test]
#[serial]
async fn test_legal_deadline_extraordinary_ag() {
    let (app_state, _container) = setup_app().await;
    let (user_id, token) = create_test_user(&app_state).await;
    let organization_id = user_id;
    let building_id = create_test_building(&app_state, organization_id).await;

    // Extraordinary AG requires 8 days minimum notice
    // Meeting 10 days from now = respects deadline
    let meeting_date = Utc::now() + Duration::days(10);
    let meeting_id =
        create_test_meeting(&app_state, organization_id, building_id, meeting_date).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(create_authenticated_app),
    )
    .await;

    let create_req = test::TestRequest::post()
        .uri("/api/v1/convocations")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "building_id": building_id.to_string(),
            "meeting_id": meeting_id.to_string(),
            "meeting_type": "Extraordinary",
            "meeting_date": meeting_date.to_rfc3339(),
            "language": "FR"
        }))
        .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    let convocation: ConvocationResponse = test::read_body_json(create_resp).await;

    // minimum_send_date should be meeting_date - 8 days
    let expected_minimum_send = meeting_date - Duration::days(8);
    assert!(
        convocation.minimum_send_date <= expected_minimum_send + Duration::seconds(5),
        "Minimum send date should be at least 8 days before meeting"
    );

    assert!(
        convocation.respects_legal_deadline,
        "Extraordinary AG with 10 days notice should respect legal deadline"
    );
}

#[actix_web::test]
#[serial]
async fn test_legal_deadline_second_convocation() {
    let (app_state, _container) = setup_app().await;
    let (user_id, token) = create_test_user(&app_state).await;
    let organization_id = user_id;
    let building_id = create_test_building(&app_state, organization_id).await;

    // Second convocation requires 8 days minimum notice (after quorum not reached)
    // Meeting 9 days from now = respects deadline
    let meeting_date = Utc::now() + Duration::days(9);
    let meeting_id =
        create_test_meeting(&app_state, organization_id, building_id, meeting_date).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(create_authenticated_app),
    )
    .await;

    let create_req = test::TestRequest::post()
        .uri("/api/v1/convocations")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "building_id": building_id.to_string(),
            "meeting_id": meeting_id.to_string(),
            "meeting_type": "SecondConvocation",
            "meeting_date": meeting_date.to_rfc3339(),
            "language": "FR"
        }))
        .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    let convocation: ConvocationResponse = test::read_body_json(create_resp).await;

    assert!(
        convocation.respects_legal_deadline,
        "Second convocation with 9 days notice should respect legal deadline"
    );
}
