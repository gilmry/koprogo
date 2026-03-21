// E2E tests for Community Notice Board HTTP endpoints (Issue #49 - Phase 2)
// Tests focus on HTTP layer: endpoints, auth, JSON serialization
// Covers notice lifecycle: create, get, list, delete

mod common;

use actix_web::http::header;
use actix_web::{test, App};
use koprogo_api::application::dto::RegisterRequest;
use koprogo_api::infrastructure::web::configure_routes;
use serde_json::json;
use serial_test::serial;
use uuid::Uuid;

/// Helper: Register a user, create a linked owner, return (token, user_id, owner_id)
async fn setup_user_with_owner(
    app_state: &actix_web::web::Data<koprogo_api::infrastructure::web::AppState>,
    org_id: Uuid,
) -> (String, Uuid) {
    let email = format!("notices-test-{}@example.com", Uuid::new_v4());
    let reg = RegisterRequest {
        email: email.clone(),
        password: "SecurePass123!".to_string(),
        first_name: "Notice".to_string(),
        last_name: "Tester".to_string(),
        role: "superadmin".to_string(),
        organization_id: Some(org_id),
    };
    let login_resp = app_state
        .auth_use_cases
        .register(reg)
        .await
        .expect("Failed to register user");
    let user_id = login_resp.user.id;
    let token = login_resp.token;

    // Create an owner in the DB and link to this user
    let owner_id = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO owners (id, organization_id, user_id, first_name, last_name, email,
           address, city, postal_code, country, created_at, updated_at)
           VALUES ($1, $2, $3, 'Notice', 'Tester', $4, '1 Rue Test', 'Brussels', '1000', 'BE', NOW(), NOW())"#,
    )
    .bind(owner_id)
    .bind(org_id)
    .bind(user_id)
    .bind(format!("owner-notices-{}@test.com", Uuid::new_v4()))
    .execute(&app_state.pool)
    .await
    .expect("Failed to insert owner");

    (token, user_id)
}

// ==================== Notice CRUD Tests ====================

#[actix_web::test]
#[serial]
async fn test_notices_create() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, _user_id) = setup_user_with_owner(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create a building
    let building_req = test::TestRequest::post()
        .uri("/api/v1/buildings")
        .insert_header(header::ContentType::json())
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "organization_id": org_id.to_string(),
            "name": "Notice Test Building",
            "address": "10 Notice Street",
            "city": "Brussels",
            "postal_code": "1000",
            "country": "BE",
            "total_units": 8,
            "total_tantiemes": 1000
        }))
        .to_request();
    let building_resp = test::call_service(&app, building_req).await;
    let building_body: serde_json::Value = test::read_body_json(building_resp).await;
    let building_id = building_body["id"].as_str().unwrap();

    let req = test::TestRequest::post()
        .uri("/api/v1/notices")
        .insert_header(header::ContentType::json())
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "building_id": building_id,
            "notice_type": "Announcement",
            "category": "General",
            "title": "Welcome to the building",
            "content": "This is a test notice for the building community."
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201, "Should create notice successfully");

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["title"], "Welcome to the building");
    assert_eq!(body["notice_type"], "Announcement");
    assert_eq!(body["category"], "General");
    assert_eq!(body["status"], "Draft");
}

#[actix_web::test]
#[serial]
async fn test_notices_get() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, _user_id) = setup_user_with_owner(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create a building
    let building_req = test::TestRequest::post()
        .uri("/api/v1/buildings")
        .insert_header(header::ContentType::json())
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "organization_id": org_id.to_string(),
            "name": "Notice Get Building",
            "address": "11 Notice Street",
            "city": "Brussels",
            "postal_code": "1000",
            "country": "BE",
            "total_units": 8,
            "total_tantiemes": 1000
        }))
        .to_request();
    let building_resp = test::call_service(&app, building_req).await;
    let building_body: serde_json::Value = test::read_body_json(building_resp).await;
    let building_id = building_body["id"].as_str().unwrap();

    // Create notice
    let create_req = test::TestRequest::post()
        .uri("/api/v1/notices")
        .insert_header(header::ContentType::json())
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "building_id": building_id,
            "notice_type": "Announcement",
            "category": "General",
            "title": "Notice to Retrieve",
            "content": "Content of the notice to retrieve."
        }))
        .to_request();
    let create_resp = test::call_service(&app, create_req).await;
    assert_eq!(create_resp.status(), 201);
    let create_body: serde_json::Value = test::read_body_json(create_resp).await;
    let notice_id = create_body["id"].as_str().unwrap();

    // Get notice by ID (no auth required per handler)
    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/notices/{}", notice_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200, "Should retrieve notice by ID");

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["id"], notice_id);
    assert_eq!(body["title"], "Notice to Retrieve");
}

#[actix_web::test]
#[serial]
async fn test_notices_list_by_building() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, _user_id) = setup_user_with_owner(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create a building
    let building_req = test::TestRequest::post()
        .uri("/api/v1/buildings")
        .insert_header(header::ContentType::json())
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "organization_id": org_id.to_string(),
            "name": "Notice List Building",
            "address": "12 Notice Street",
            "city": "Brussels",
            "postal_code": "1000",
            "country": "BE",
            "total_units": 8,
            "total_tantiemes": 1000
        }))
        .to_request();
    let building_resp = test::call_service(&app, building_req).await;
    let building_body: serde_json::Value = test::read_body_json(building_resp).await;
    let building_id = building_body["id"].as_str().unwrap();

    // Create two notices
    for title in &["First Notice", "Second Notice"] {
        let req = test::TestRequest::post()
            .uri("/api/v1/notices")
            .insert_header(header::ContentType::json())
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
            .set_json(json!({
                "building_id": building_id,
                "notice_type": "Announcement",
                "category": "General",
                "title": title,
                "content": "Notice content for listing test."
            }))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 201);
    }

    // List all notices for the building
    let list_req = test::TestRequest::get()
        .uri(&format!("/api/v1/buildings/{}/notices", building_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();
    let list_resp = test::call_service(&app, list_req).await;
    assert_eq!(list_resp.status(), 200, "Should list building notices");

    let notices: serde_json::Value = test::read_body_json(list_resp).await;
    assert!(notices.is_array(), "Response should be an array");
    assert_eq!(
        notices.as_array().unwrap().len(),
        2,
        "Should have 2 notices"
    );
}

#[actix_web::test]
#[serial]
async fn test_notices_delete() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, _user_id) = setup_user_with_owner(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create a building
    let building_req = test::TestRequest::post()
        .uri("/api/v1/buildings")
        .insert_header(header::ContentType::json())
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "organization_id": org_id.to_string(),
            "name": "Notice Delete Building",
            "address": "13 Notice Street",
            "city": "Brussels",
            "postal_code": "1000",
            "country": "BE",
            "total_units": 8,
            "total_tantiemes": 1000
        }))
        .to_request();
    let building_resp = test::call_service(&app, building_req).await;
    let building_body: serde_json::Value = test::read_body_json(building_resp).await;
    let building_id = building_body["id"].as_str().unwrap();

    // Create notice
    let create_req = test::TestRequest::post()
        .uri("/api/v1/notices")
        .insert_header(header::ContentType::json())
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "building_id": building_id,
            "notice_type": "Announcement",
            "category": "General",
            "title": "Notice to Delete",
            "content": "This notice will be deleted."
        }))
        .to_request();
    let create_resp = test::call_service(&app, create_req).await;
    assert_eq!(create_resp.status(), 201);
    let create_body: serde_json::Value = test::read_body_json(create_resp).await;
    let notice_id = create_body["id"].as_str().unwrap();

    // Delete notice
    let delete_req = test::TestRequest::delete()
        .uri(&format!("/api/v1/notices/{}", notice_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();
    let delete_resp = test::call_service(&app, delete_req).await;
    let status = delete_resp.status().as_u16();
    assert!(
        status == 200 || status == 204,
        "Should delete notice (got {})",
        status
    );

    // Verify notice is gone
    let get_req = test::TestRequest::get()
        .uri(&format!("/api/v1/notices/{}", notice_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();
    let get_resp = test::call_service(&app, get_req).await;
    assert_eq!(
        get_resp.status(),
        404,
        "Notice should not exist after delete"
    );
}

#[actix_web::test]
#[serial]
async fn test_notices_unauthorized() {
    let (app_state, _container, _org_id) = common::setup_test_db().await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Attempt to create a notice without a token
    let req = test::TestRequest::post()
        .uri("/api/v1/notices")
        .insert_header(header::ContentType::json())
        .set_json(json!({
            "building_id": Uuid::new_v4().to_string(),
            "notice_type": "Announcement",
            "category": "General",
            "title": "Unauthorized Notice",
            "content": "Should not be created."
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401, "Should return 401 without auth token");
}
