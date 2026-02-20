// E2E tests for document HTTP endpoints (Issue #76)
// Tests focus on HTTP layer: multipart upload, download, auth, JSON serialization
// Tests document management system with file upload/download functionality

mod common;

use actix_web::http::header;
use actix_web::{test, App};
use koprogo_api::application::dto::CreateBuildingDto;
use koprogo_api::infrastructure::web::configure_routes;
use serial_test::serial;
use uuid::Uuid;

/// Helper: Create organization, user, building for tests
async fn create_test_fixtures(
    app_state: &actix_web::web::Data<koprogo_api::infrastructure::web::AppState>,
    org_id: Uuid,
) -> (String, Uuid, Uuid) {
    // Register + login
    let _token = common::register_and_login(app_state, org_id).await;

    // Get the user_id from the token by doing a register separately
    let email = format!("doc-e2e+{}@test.com", Uuid::new_v4());
    let reg = koprogo_api::application::dto::RegisterRequest {
        email: email.clone(),
        password: "Passw0rd!".to_string(),
        first_name: "E2E".to_string(),
        last_name: "DocUser".to_string(),
        role: "superadmin".to_string(),
        organization_id: Some(org_id),
    };
    let register_result = app_state
        .auth_use_cases
        .register(reg)
        .await
        .expect("Failed to register user");

    let user_id = register_result.user.id;

    let login = koprogo_api::application::dto::LoginRequest {
        email: email.clone(),
        password: "Passw0rd!".to_string(),
    };
    let login_result = app_state
        .auth_use_cases
        .login(login)
        .await
        .expect("Failed to login");
    let token = login_result.token;

    // Create building using DTO
    let building_dto = CreateBuildingDto {
        organization_id: org_id.to_string(),
        name: "Test Building".to_string(),
        address: "123 Main St".to_string(),
        city: "Brussels".to_string(),
        postal_code: "1000".to_string(),
        country: "Belgium".to_string(),
        total_units: 10,
        total_tantiemes: Some(1000),
        construction_year: Some(2020),
    };

    let building = app_state
        .building_use_cases
        .create_building(building_dto)
        .await
        .expect("Failed to create building");

    let building_id = Uuid::parse_str(&building.id).expect("Invalid building ID");

    (token, building_id, user_id)
}

//
// TEST: POST /documents (Upload document with multipart)
//

#[actix_web::test]
#[serial]
async fn test_upload_document_success() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, building_id, user_id) = create_test_fixtures(&app_state, org_id).await;

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
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (_token, building_id, user_id) = create_test_fixtures(&app_state, org_id).await;

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
// TEST: GET /documents/:id (Get document metadata)
//

#[actix_web::test]
#[serial]
async fn test_get_document_metadata() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, building_id, user_id) = create_test_fixtures(&app_state, org_id).await;

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
// TEST: GET /documents (List all documents - paginated)
//

#[actix_web::test]
#[serial]
async fn test_list_documents_paginated() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, building_id, user_id) = create_test_fixtures(&app_state, org_id).await;

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
    assert_eq!(body["pagination"]["current_page"], 1);
    assert_eq!(body["pagination"]["per_page"], 10);
}

//
// TEST: GET /buildings/:id/documents (List building documents)
//

#[actix_web::test]
#[serial]
async fn test_list_building_documents() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, building_id, user_id) = create_test_fixtures(&app_state, org_id).await;

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
// TEST: DELETE /documents/:id (Delete document)
//

#[actix_web::test]
#[serial]
async fn test_delete_document_success() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, building_id, user_id) = create_test_fixtures(&app_state, org_id).await;

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
    assert_eq!(resp.status(), 204, "Should delete document successfully");

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
// TEST: Document types validation
//

#[actix_web::test]
#[serial]
async fn test_document_types_validation() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, building_id, user_id) = create_test_fixtures(&app_state, org_id).await;

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
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, building_id, user_id) = create_test_fixtures(&app_state, org_id).await;

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
// TEST: Complete document lifecycle
//

#[actix_web::test]
#[serial]
async fn test_document_complete_lifecycle() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, building_id, user_id) = create_test_fixtures(&app_state, org_id).await;

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
         Proces-verbal de l'assemblee generale ordinaire\r\n\
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
    assert!(!list["data"].as_array().unwrap().is_empty());

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
    assert_eq!(resp.status(), 204, "Deletion should succeed");

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
