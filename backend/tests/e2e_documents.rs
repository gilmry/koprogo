// E2E tests for document HTTP endpoints (Issue #76)
// Tests focus on HTTP layer: multipart upload, download, auth, JSON serialization
// Tests document management system with file upload/download functionality

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

/// Setup function shared across all document E2E tests
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
    let jwt_secret = "e2e-document-secret".to_string();
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

    // Create unique storage root for this test run
    let test_id = Uuid::new_v4();
    let storage_root = std::env::temp_dir().join(format!("koprogo_e2e_documents_{}", test_id));
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

/// Helper: Create organization, user, building for tests
async fn create_test_fixtures(
    app_state: &actix_web::web::Data<AppState>,
) -> (String, Uuid, Uuid, Uuid) {
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

    let user_id = Uuid::parse_str(&register_result.user_id).expect("Invalid user ID");

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

    (token, org_id, building_id, user_id)
}

//
// ═══════════════════════════════════════════════════════════════════════════
// TEST: POST /documents (Upload document with multipart)
// ═══════════════════════════════════════════════════════════════════════════
//

#[actix_web::test]
#[serial]
async fn test_upload_document_success() {
    let (app_state, _container) = setup_app().await;
    let (token, _org_id, building_id, user_id) = create_test_fixtures(&app_state).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create test file content
    let file_content = b"Test PDF content for document upload test";

    // Create multipart form boundary
    let boundary = "----WebKitFormBoundary7MA4YWxkTrZu0gW";

    // Build multipart/form-data body manually
    let body = format!(
        "--{boundary}\r\n\
         Content-Disposition: form-data; name=\"file\"; filename=\"test-document.pdf\"\r\n\
         Content-Type: application/pdf\r\n\
         \r\n\
         {file_content}\r\n\
         --{boundary}\r\n\
         Content-Disposition: form-data; name=\"building_id\"\r\n\
         \r\n\
         {building_id}\r\n\
         --{boundary}\r\n\
         Content-Disposition: form-data; name=\"document_type\"\r\n\
         \r\n\
         Invoice\r\n\
         --{boundary}\r\n\
         Content-Disposition: form-data; name=\"title\"\r\n\
         \r\n\
         Test Invoice Document\r\n\
         --{boundary}\r\n\
         Content-Disposition: form-data; name=\"description\"\r\n\
         \r\n\
         Test invoice for E2E testing\r\n\
         --{boundary}\r\n\
         Content-Disposition: form-data; name=\"uploaded_by\"\r\n\
         \r\n\
         {user_id}\r\n\
         --{boundary}--\r\n",
        boundary = boundary,
        file_content = String::from_utf8_lossy(file_content),
        building_id = building_id,
        user_id = user_id
    );

    let req = test::TestRequest::post()
        .uri("/api/v1/documents")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .insert_header((
            header::CONTENT_TYPE,
            format!("multipart/form-data; boundary={}", boundary),
        ))
        .set_payload(body)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201, "Should upload document successfully");

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["building_id"], building_id.to_string());
    assert_eq!(body["title"], "Test Invoice Document");
    assert_eq!(body["document_type"], "Invoice");
    assert!(body["id"].is_string());
    assert!(body["file_path"].is_string());
}

#[actix_web::test]
#[serial]
async fn test_upload_document_without_auth_fails() {
    let (app_state, _container) = setup_app().await;
    let (_token, _org_id, building_id, user_id) = create_test_fixtures(&app_state).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let file_content = b"Unauthorized upload attempt";
    let boundary = "----WebKitFormBoundary7MA4YWxkTrZu0gW";

    let body = format!(
        "--{boundary}\r\n\
         Content-Disposition: form-data; name=\"file\"; filename=\"unauthorized.pdf\"\r\n\
         Content-Type: application/pdf\r\n\
         \r\n\
         {file_content}\r\n\
         --{boundary}\r\n\
         Content-Disposition: form-data; name=\"building_id\"\r\n\
         \r\n\
         {building_id}\r\n\
         --{boundary}\r\n\
         Content-Disposition: form-data; name=\"document_type\"\r\n\
         \r\n\
         Invoice\r\n\
         --{boundary}\r\n\
         Content-Disposition: form-data; name=\"title\"\r\n\
         \r\n\
         Unauthorized Doc\r\n\
         --{boundary}\r\n\
         Content-Disposition: form-data; name=\"uploaded_by\"\r\n\
         \r\n\
         {user_id}\r\n\
         --{boundary}--\r\n",
        boundary = boundary,
        file_content = String::from_utf8_lossy(file_content),
        building_id = building_id,
        user_id = user_id
    );

    let req = test::TestRequest::post()
        .uri("/api/v1/documents")
        .insert_header((
            header::CONTENT_TYPE,
            format!("multipart/form-data; boundary={}", boundary),
        ))
        .set_payload(body)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401, "Should reject unauthorized upload");
}

//
// ═══════════════════════════════════════════════════════════════════════════
// TEST: GET /documents/:id (Get document metadata)
// ═══════════════════════════════════════════════════════════════════════════
//

#[actix_web::test]
#[serial]
async fn test_get_document_metadata() {
    let (app_state, _container) = setup_app().await;
    let (token, _org_id, building_id, user_id) = create_test_fixtures(&app_state).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Upload a document first
    let file_content = b"Test content for metadata retrieval";
    let boundary = "----WebKitFormBoundary7MA4YWxkTrZu0gW";

    let body = format!(
        "--{boundary}\r\n\
         Content-Disposition: form-data; name=\"file\"; filename=\"metadata-test.pdf\"\r\n\
         Content-Type: application/pdf\r\n\
         \r\n\
         {file_content}\r\n\
         --{boundary}\r\n\
         Content-Disposition: form-data; name=\"building_id\"\r\n\
         \r\n\
         {building_id}\r\n\
         --{boundary}\r\n\
         Content-Disposition: form-data; name=\"document_type\"\r\n\
         \r\n\
         MeetingMinutes\r\n\
         --{boundary}\r\n\
         Content-Disposition: form-data; name=\"title\"\r\n\
         \r\n\
         Metadata Test Document\r\n\
         --{boundary}\r\n\
         Content-Disposition: form-data; name=\"uploaded_by\"\r\n\
         \r\n\
         {user_id}\r\n\
         --{boundary}--\r\n",
        boundary = boundary,
        file_content = String::from_utf8_lossy(file_content),
        building_id = building_id,
        user_id = user_id
    );

    let req = test::TestRequest::post()
        .uri("/api/v1/documents")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .insert_header((
            header::CONTENT_TYPE,
            format!("multipart/form-data; boundary={}", boundary),
        ))
        .set_payload(body)
        .to_request();

    let upload_resp = test::call_service(&app, req).await;
    assert_eq!(upload_resp.status(), 201);

    let uploaded: serde_json::Value = test::read_body_json(upload_resp).await;
    let document_id = uploaded["id"].as_str().unwrap();

    // Get document metadata
    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/documents/{}", document_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["id"], document_id);
    assert_eq!(body["title"], "Metadata Test Document");
    assert_eq!(body["document_type"], "MeetingMinutes");
    assert!(body["file_size"].as_i64().unwrap() > 0);
    assert_eq!(body["mime_type"], "application/pdf");
}

//
// ═══════════════════════════════════════════════════════════════════════════
// TEST: GET /documents (List all documents - paginated)
// ═══════════════════════════════════════════════════════════════════════════
//

#[actix_web::test]
#[serial]
async fn test_list_documents_paginated() {
    let (app_state, _container) = setup_app().await;
    let (token, _org_id, building_id, user_id) = create_test_fixtures(&app_state).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Upload 3 documents
    for i in 1..=3 {
        let file_content = format!("Test content for document {}", i);
        let boundary = "----WebKitFormBoundary7MA4YWxkTrZu0gW";

        let body = format!(
            "--{boundary}\r\n\
             Content-Disposition: form-data; name=\"file\"; filename=\"doc{i}.pdf\"\r\n\
             Content-Type: application/pdf\r\n\
             \r\n\
             {file_content}\r\n\
             --{boundary}\r\n\
             Content-Disposition: form-data; name=\"building_id\"\r\n\
             \r\n\
             {building_id}\r\n\
             --{boundary}\r\n\
             Content-Disposition: form-data; name=\"document_type\"\r\n\
             \r\n\
             Other\r\n\
             --{boundary}\r\n\
             Content-Disposition: form-data; name=\"title\"\r\n\
             \r\n\
             Document {i}\r\n\
             --{boundary}\r\n\
             Content-Disposition: form-data; name=\"uploaded_by\"\r\n\
             \r\n\
             {user_id}\r\n\
             --{boundary}--\r\n",
            boundary = boundary,
            file_content = file_content,
            building_id = building_id,
            user_id = user_id,
            i = i
        );

        let req = test::TestRequest::post()
            .uri("/api/v1/documents")
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
            .insert_header((
                header::CONTENT_TYPE,
                format!("multipart/form-data; boundary={}", boundary),
            ))
            .set_payload(body)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 201);
    }

    // List documents with pagination
    let req = test::TestRequest::get()
        .uri("/api/v1/documents?page=1&per_page=10")
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
// TEST: GET /buildings/:id/documents (List building documents)
// ═══════════════════════════════════════════════════════════════════════════
//

#[actix_web::test]
#[serial]
async fn test_list_building_documents() {
    let (app_state, _container) = setup_app().await;
    let (token, _org_id, building_id, user_id) = create_test_fixtures(&app_state).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Upload 2 documents for this building
    for i in 1..=2 {
        let file_content = format!("Building document {}", i);
        let boundary = "----WebKitFormBoundary7MA4YWxkTrZu0gW";

        let body = format!(
            "--{boundary}\r\n\
             Content-Disposition: form-data; name=\"file\"; filename=\"building-doc{i}.pdf\"\r\n\
             Content-Type: application/pdf\r\n\
             \r\n\
             {file_content}\r\n\
             --{boundary}\r\n\
             Content-Disposition: form-data; name=\"building_id\"\r\n\
             \r\n\
             {building_id}\r\n\
             --{boundary}\r\n\
             Content-Disposition: form-data; name=\"document_type\"\r\n\
             \r\n\
             Contract\r\n\
             --{boundary}\r\n\
             Content-Disposition: form-data; name=\"title\"\r\n\
             \r\n\
             Building Contract {i}\r\n\
             --{boundary}\r\n\
             Content-Disposition: form-data; name=\"uploaded_by\"\r\n\
             \r\n\
             {user_id}\r\n\
             --{boundary}--\r\n",
            boundary = boundary,
            file_content = file_content,
            building_id = building_id,
            user_id = user_id,
            i = i
        );

        let req = test::TestRequest::post()
            .uri("/api/v1/documents")
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
            .insert_header((
                header::CONTENT_TYPE,
                format!("multipart/form-data; boundary={}", boundary),
            ))
            .set_payload(body)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 201);
    }

    // List documents for building
    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/buildings/{}/documents", building_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body.is_array());
    assert!(body.as_array().unwrap().len() >= 2);

    // Verify all documents belong to this building
    for doc in body.as_array().unwrap() {
        assert_eq!(doc["building_id"], building_id.to_string());
    }
}

//
// ═══════════════════════════════════════════════════════════════════════════
// TEST: DELETE /documents/:id (Delete document)
// ═══════════════════════════════════════════════════════════════════════════
//

#[actix_web::test]
#[serial]
async fn test_delete_document_success() {
    let (app_state, _container) = setup_app().await;
    let (token, _org_id, building_id, user_id) = create_test_fixtures(&app_state).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Upload a document
    let file_content = b"Document to be deleted";
    let boundary = "----WebKitFormBoundary7MA4YWxkTrZu0gW";

    let body = format!(
        "--{boundary}\r\n\
         Content-Disposition: form-data; name=\"file\"; filename=\"to-delete.pdf\"\r\n\
         Content-Type: application/pdf\r\n\
         \r\n\
         {file_content}\r\n\
         --{boundary}\r\n\
         Content-Disposition: form-data; name=\"building_id\"\r\n\
         \r\n\
         {building_id}\r\n\
         --{boundary}\r\n\
         Content-Disposition: form-data; name=\"document_type\"\r\n\
         \r\n\
         Other\r\n\
         --{boundary}\r\n\
         Content-Disposition: form-data; name=\"title\"\r\n\
         \r\n\
         Document to Delete\r\n\
         --{boundary}\r\n\
         Content-Disposition: form-data; name=\"uploaded_by\"\r\n\
         \r\n\
         {user_id}\r\n\
         --{boundary}--\r\n",
        boundary = boundary,
        file_content = String::from_utf8_lossy(file_content),
        building_id = building_id,
        user_id = user_id
    );

    let req = test::TestRequest::post()
        .uri("/api/v1/documents")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .insert_header((
            header::CONTENT_TYPE,
            format!("multipart/form-data; boundary={}", boundary),
        ))
        .set_payload(body)
        .to_request();

    let upload_resp = test::call_service(&app, req).await;
    assert_eq!(upload_resp.status(), 201);

    let uploaded: serde_json::Value = test::read_body_json(upload_resp).await;
    let document_id = uploaded["id"].as_str().unwrap();

    // Delete the document
    let req = test::TestRequest::delete()
        .uri(&format!("/api/v1/documents/{}", document_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200, "Should delete document successfully");

    // Verify document is deleted
    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/documents/{}", document_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.status(),
        404,
        "Deleted document should no longer be found"
    );
}

//
// ═══════════════════════════════════════════════════════════════════════════
// TEST: Document types validation
// ═══════════════════════════════════════════════════════════════════════════
//

#[actix_web::test]
#[serial]
async fn test_document_types_validation() {
    let (app_state, _container) = setup_app().await;
    let (token, _org_id, building_id, user_id) = create_test_fixtures(&app_state).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Test all valid document types
    let valid_types = vec![
        "MeetingMinutes",
        "FinancialStatement",
        "Invoice",
        "Contract",
        "Regulation",
        "WorksQuote",
        "Other",
    ];

    for doc_type in valid_types {
        let file_content = format!("Test content for {}", doc_type);
        let boundary = "----WebKitFormBoundary7MA4YWxkTrZu0gW";

        let body = format!(
            "--{boundary}\r\n\
             Content-Disposition: form-data; name=\"file\"; filename=\"{doc_type}.pdf\"\r\n\
             Content-Type: application/pdf\r\n\
             \r\n\
             {file_content}\r\n\
             --{boundary}\r\n\
             Content-Disposition: form-data; name=\"building_id\"\r\n\
             \r\n\
             {building_id}\r\n\
             --{boundary}\r\n\
             Content-Disposition: form-data; name=\"document_type\"\r\n\
             \r\n\
             {doc_type}\r\n\
             --{boundary}\r\n\
             Content-Disposition: form-data; name=\"title\"\r\n\
             \r\n\
             Test {doc_type}\r\n\
             --{boundary}\r\n\
             Content-Disposition: form-data; name=\"uploaded_by\"\r\n\
             \r\n\
             {user_id}\r\n\
             --{boundary}--\r\n",
            boundary = boundary,
            file_content = file_content,
            building_id = building_id,
            user_id = user_id,
            doc_type = doc_type
        );

        let req = test::TestRequest::post()
            .uri("/api/v1/documents")
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
            .insert_header((
                header::CONTENT_TYPE,
                format!("multipart/form-data; boundary={}", boundary),
            ))
            .set_payload(body)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(
            resp.status(),
            201,
            "Document type {} should be valid",
            doc_type
        );

        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["document_type"], doc_type);
    }
}

#[actix_web::test]
#[serial]
async fn test_invalid_document_type_fails() {
    let (app_state, _container) = setup_app().await;
    let (token, _org_id, building_id, user_id) = create_test_fixtures(&app_state).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let file_content = b"Test content";
    let boundary = "----WebKitFormBoundary7MA4YWxkTrZu0gW";

    let body = format!(
        "--{boundary}\r\n\
         Content-Disposition: form-data; name=\"file\"; filename=\"invalid.pdf\"\r\n\
         Content-Type: application/pdf\r\n\
         \r\n\
         {file_content}\r\n\
         --{boundary}\r\n\
         Content-Disposition: form-data; name=\"building_id\"\r\n\
         \r\n\
         {building_id}\r\n\
         --{boundary}\r\n\
         Content-Disposition: form-data; name=\"document_type\"\r\n\
         \r\n\
         InvalidType\r\n\
         --{boundary}\r\n\
         Content-Disposition: form-data; name=\"title\"\r\n\
         \r\n\
         Invalid Type Test\r\n\
         --{boundary}\r\n\
         Content-Disposition: form-data; name=\"uploaded_by\"\r\n\
         \r\n\
         {user_id}\r\n\
         --{boundary}--\r\n",
        boundary = boundary,
        file_content = String::from_utf8_lossy(file_content),
        building_id = building_id,
        user_id = user_id
    );

    let req = test::TestRequest::post()
        .uri("/api/v1/documents")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .insert_header((
            header::CONTENT_TYPE,
            format!("multipart/form-data; boundary={}", boundary),
        ))
        .set_payload(body)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400, "Should reject invalid document type");
}

//
// ═══════════════════════════════════════════════════════════════════════════
// TEST: Complete document lifecycle
// ═══════════════════════════════════════════════════════════════════════════
//

#[actix_web::test]
#[serial]
async fn test_document_complete_lifecycle() {
    let (app_state, _container) = setup_app().await;
    let (token, _org_id, building_id, user_id) = create_test_fixtures(&app_state).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // 1. Upload document
    let file_content = b"Complete lifecycle test document content with important data";
    let boundary = "----WebKitFormBoundary7MA4YWxkTrZu0gW";

    let body = format!(
        "--{boundary}\r\n\
         Content-Disposition: form-data; name=\"file\"; filename=\"lifecycle-test.pdf\"\r\n\
         Content-Type: application/pdf\r\n\
         \r\n\
         {file_content}\r\n\
         --{boundary}\r\n\
         Content-Disposition: form-data; name=\"building_id\"\r\n\
         \r\n\
         {building_id}\r\n\
         --{boundary}\r\n\
         Content-Disposition: form-data; name=\"document_type\"\r\n\
         \r\n\
         MeetingMinutes\r\n\
         --{boundary}\r\n\
         Content-Disposition: form-data; name=\"title\"\r\n\
         \r\n\
         Lifecycle Test PV AG 2025\r\n\
         --{boundary}\r\n\
         Content-Disposition: form-data; name=\"description\"\r\n\
         \r\n\
         Procès-verbal de l'assemblée générale ordinaire\r\n\
         --{boundary}\r\n\
         Content-Disposition: form-data; name=\"uploaded_by\"\r\n\
         \r\n\
         {user_id}\r\n\
         --{boundary}--\r\n",
        boundary = boundary,
        file_content = String::from_utf8_lossy(file_content),
        building_id = building_id,
        user_id = user_id
    );

    let req = test::TestRequest::post()
        .uri("/api/v1/documents")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .insert_header((
            header::CONTENT_TYPE,
            format!("multipart/form-data; boundary={}", boundary),
        ))
        .set_payload(body)
        .to_request();

    let upload_resp = test::call_service(&app, req).await;
    assert_eq!(upload_resp.status(), 201, "Upload should succeed");

    let uploaded: serde_json::Value = test::read_body_json(upload_resp).await;
    let document_id = uploaded["id"].as_str().unwrap();

    assert_eq!(uploaded["title"], "Lifecycle Test PV AG 2025");
    assert_eq!(uploaded["document_type"], "MeetingMinutes");
    assert_eq!(
        uploaded["description"],
        "Procès-verbal de l'assemblée générale ordinaire"
    );

    // 2. Get document metadata
    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/documents/{}", document_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200, "Should retrieve metadata");

    let metadata: serde_json::Value = test::read_body_json(resp).await;
    assert!(metadata["file_size"].as_i64().unwrap() > 0);
    assert_eq!(metadata["mime_type"], "application/pdf");

    // 3. List documents (should include our document)
    let req = test::TestRequest::get()
        .uri("/api/v1/documents?page=1&per_page=10")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let list: serde_json::Value = test::read_body_json(resp).await;
    assert!(list["data"].as_array().unwrap().len() > 0);

    // Find our document in the list
    let found = list["data"]
        .as_array()
        .unwrap()
        .iter()
        .any(|doc| doc["id"] == document_id);
    assert!(found, "Document should appear in list");

    // 4. List building documents (should include our document)
    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/buildings/{}/documents", building_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let building_docs: serde_json::Value = test::read_body_json(resp).await;
    let found_in_building = building_docs
        .as_array()
        .unwrap()
        .iter()
        .any(|doc| doc["id"] == document_id);
    assert!(
        found_in_building,
        "Document should appear in building documents"
    );

    // 5. Delete document
    let req = test::TestRequest::delete()
        .uri(&format!("/api/v1/documents/{}", document_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200, "Deletion should succeed");

    // 6. Verify deletion
    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/documents/{}", document_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.status(),
        404,
        "Document should no longer exist after deletion"
    );
}
