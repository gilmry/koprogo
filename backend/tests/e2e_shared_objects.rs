// E2E tests for Shared Object Library HTTP endpoints (Issue #49 - Phase 4)
// Tests focus on HTTP layer: endpoints, auth, JSON serialization
// Covers shared object lifecycle: create, get, list, borrow, return, delete

mod common;

use actix_web::http::header;
use actix_web::{test, App};
use koprogo_api::application::dto::RegisterRequest;
use koprogo_api::infrastructure::web::configure_routes;
use serde_json::json;
use serial_test::serial;
use uuid::Uuid;

/// Helper: Register a user, create a linked owner, return (token, user_id)
async fn setup_shared_obj_user_with_owner(
    app_state: &actix_web::web::Data<koprogo_api::infrastructure::web::AppState>,
    org_id: Uuid,
    label: &str,
) -> (String, Uuid) {
    let email = format!("shared-{}-{}@example.com", label, Uuid::new_v4());
    let reg = RegisterRequest {
        email: email.clone(),
        password: "SecurePass123!".to_string(),
        first_name: "Shared".to_string(),
        last_name: label.to_string(),
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
           VALUES ($1, $2, $3, 'Shared', $4, $5, '3 Rue Test', 'Brussels', '1000', 'BE', NOW(), NOW())"#,
    )
    .bind(owner_id)
    .bind(org_id)
    .bind(user_id)
    .bind(label)
    .bind(format!("owner-shared-{}-{}@test.com", label, Uuid::new_v4()))
    .execute(&app_state.pool)
    .await
    .expect("Failed to insert owner");

    (token, user_id)
}

/// Helper: Create a building using use cases directly (not HTTP) and return its id
async fn create_building_for_shared_objects(
    app_state: &actix_web::web::Data<koprogo_api::infrastructure::web::AppState>,
    org_id: Uuid,
    name: &str,
) -> String {
    use koprogo_api::application::dto::CreateBuildingDto;
    let dto = CreateBuildingDto {
        organization_id: org_id.to_string(),
        name: format!("{} {}", name, uuid::Uuid::new_v4()),
        address: "30 Object Street".to_string(),
        city: "Brussels".to_string(),
        postal_code: "1000".to_string(),
        country: "BE".to_string(),
        total_units: 8,
        total_tantiemes: Some(1000),
        construction_year: Some(2020),
    };
    let building = app_state
        .building_use_cases
        .create_building(dto)
        .await
        .expect("Failed to create test building for shared objects");
    building.id
}

// ==================== Shared Object CRUD Tests ====================

#[actix_web::test]
#[serial]
async fn test_shared_objects_create() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, _user_id) = setup_shared_obj_user_with_owner(&app_state, org_id, "owner").await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let building_id =
        create_building_for_shared_objects(&app_state, org_id, "Shared Objects Create Building")
            .await;

    let req = test::TestRequest::post()
        .uri("/api/v1/shared-objects")
        .insert_header(header::ContentType::json())
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "building_id": building_id,
            "object_category": "Tools",
            "object_name": "Electric Drill",
            "description": "Bosch electric drill, good condition.",
            "condition": "Good",
            "is_available": true,
            "borrowing_duration_days": 3
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.status(),
        201,
        "Should create shared object successfully"
    );

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["object_name"], "Electric Drill");
    assert_eq!(body["object_category"], "Tools");
    assert_eq!(body["condition"], "Good");
    assert_eq!(body["is_available"], true);
    assert_eq!(body["is_borrowed"], false);
}

#[actix_web::test]
#[serial]
async fn test_shared_objects_get() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, _user_id) = setup_shared_obj_user_with_owner(&app_state, org_id, "getowner").await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let building_id =
        create_building_for_shared_objects(&app_state, org_id, "Shared Objects Get Building").await;

    // Create shared object
    let create_req = test::TestRequest::post()
        .uri("/api/v1/shared-objects")
        .insert_header(header::ContentType::json())
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "building_id": building_id,
            "object_category": "Sports",
            "object_name": "Bicycle Pump",
            "description": "Standard bicycle pump.",
            "condition": "Good",
            "is_available": true
        }))
        .to_request();
    let create_resp = test::call_service(&app, create_req).await;
    assert_eq!(create_resp.status(), 201);
    let create_body: serde_json::Value = test::read_body_json(create_resp).await;
    let object_id = create_body["id"].as_str().unwrap();

    // Get shared object by ID
    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/shared-objects/{}", object_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200, "Should retrieve shared object by ID");

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["id"], object_id);
    assert_eq!(body["object_name"], "Bicycle Pump");
}

#[actix_web::test]
#[serial]
async fn test_shared_objects_list_by_building() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, _user_id) = setup_shared_obj_user_with_owner(&app_state, org_id, "listowner").await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let building_id =
        create_building_for_shared_objects(&app_state, org_id, "Shared Objects List Building")
            .await;

    // Create two shared objects
    for (name, category) in &[("Ladder", "Tools"), ("Wheelbarrow", "Gardening")] {
        let req = test::TestRequest::post()
            .uri("/api/v1/shared-objects")
            .insert_header(header::ContentType::json())
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
            .set_json(json!({
                "building_id": building_id,
                "object_category": category,
                "object_name": name,
                "description": "Available for borrowing.",
                "condition": "Good",
                "is_available": true
            }))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 201);
    }

    // List shared objects for the building
    let list_req = test::TestRequest::get()
        .uri(&format!("/api/v1/buildings/{}/shared-objects", building_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();
    let list_resp = test::call_service(&app, list_req).await;
    assert_eq!(
        list_resp.status(),
        200,
        "Should list building shared objects"
    );

    let objects: serde_json::Value = test::read_body_json(list_resp).await;
    assert!(objects.is_array(), "Response should be an array");
    assert_eq!(
        objects.as_array().unwrap().len(),
        2,
        "Should have 2 shared objects"
    );
}

#[actix_web::test]
#[serial]
async fn test_shared_objects_borrow() {
    let (app_state, _container, org_id) = common::setup_test_db().await;

    // Create two users: owner of the object and borrower
    let (owner_token, _owner_user_id) =
        setup_shared_obj_user_with_owner(&app_state, org_id, "objectowner").await;
    let (borrower_token, _borrower_user_id) =
        setup_shared_obj_user_with_owner(&app_state, org_id, "borrower").await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let building_id =
        create_building_for_shared_objects(&app_state, org_id, "Shared Objects Borrow Building")
            .await;

    // Create shared object (as owner)
    let create_req = test::TestRequest::post()
        .uri("/api/v1/shared-objects")
        .insert_header(header::ContentType::json())
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", owner_token)))
        .set_json(json!({
            "building_id": building_id,
            "object_category": "Tools",
            "object_name": "Pressure Washer",
            "description": "Karcher pressure washer.",
            "condition": "Excellent",
            "is_available": true,
            "borrowing_duration_days": 2
        }))
        .to_request();
    let create_resp = test::call_service(&app, create_req).await;
    assert_eq!(create_resp.status(), 201);
    let create_body: serde_json::Value = test::read_body_json(create_resp).await;
    let object_id = create_body["id"].as_str().unwrap();

    // Borrow object (as borrower)
    let borrow_req = test::TestRequest::post()
        .uri(&format!("/api/v1/shared-objects/{}/borrow", object_id))
        .insert_header(header::ContentType::json())
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", borrower_token)))
        .set_json(json!({}))
        .to_request();
    let borrow_resp = test::call_service(&app, borrow_req).await;
    let status = borrow_resp.status().as_u16();
    assert!(
        status == 200 || status == 201,
        "Should borrow object successfully (got {})",
        status
    );

    let body: serde_json::Value = test::read_body_json(borrow_resp).await;
    assert_eq!(
        body["is_borrowed"], true,
        "Object should be marked borrowed"
    );
    assert_eq!(
        body["is_available"], false,
        "Object should no longer be available"
    );
}

#[actix_web::test]
#[serial]
async fn test_shared_objects_return() {
    let (app_state, _container, org_id) = common::setup_test_db().await;

    // Create two users: owner and borrower
    let (owner_token, _owner_user_id) =
        setup_shared_obj_user_with_owner(&app_state, org_id, "retowner").await;
    let (borrower_token, _borrower_user_id) =
        setup_shared_obj_user_with_owner(&app_state, org_id, "retborrower").await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let building_id =
        create_building_for_shared_objects(&app_state, org_id, "Shared Objects Return Building")
            .await;

    // Create shared object
    let create_req = test::TestRequest::post()
        .uri("/api/v1/shared-objects")
        .insert_header(header::ContentType::json())
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", owner_token)))
        .set_json(json!({
            "building_id": building_id,
            "object_category": "Gardening",
            "object_name": "Lawn Mower",
            "description": "Electric lawn mower.",
            "condition": "Good",
            "is_available": true,
            "borrowing_duration_days": 1
        }))
        .to_request();
    let create_resp = test::call_service(&app, create_req).await;
    assert_eq!(create_resp.status(), 201);
    let create_body: serde_json::Value = test::read_body_json(create_resp).await;
    let object_id = create_body["id"].as_str().unwrap();

    // Borrow the object first
    let borrow_req = test::TestRequest::post()
        .uri(&format!("/api/v1/shared-objects/{}/borrow", object_id))
        .insert_header(header::ContentType::json())
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", borrower_token)))
        .set_json(json!({}))
        .to_request();
    let borrow_resp = test::call_service(&app, borrow_req).await;
    assert!(
        borrow_resp.status().as_u16() == 200 || borrow_resp.status().as_u16() == 201,
        "Borrow must succeed"
    );

    // Return the object (as borrower)
    let return_req = test::TestRequest::post()
        .uri(&format!("/api/v1/shared-objects/{}/return", object_id))
        .insert_header(header::ContentType::json())
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", borrower_token)))
        .to_request();
    let return_resp = test::call_service(&app, return_req).await;
    assert_eq!(
        return_resp.status(),
        200,
        "Should return object successfully"
    );

    let body: serde_json::Value = test::read_body_json(return_resp).await;
    assert_eq!(
        body["is_borrowed"], false,
        "Object should no longer be borrowed"
    );
    assert_eq!(
        body["is_available"], true,
        "Object should be available again"
    );
}

#[actix_web::test]
#[serial]
async fn test_shared_objects_delete() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, _user_id) = setup_shared_obj_user_with_owner(&app_state, org_id, "delowner").await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let building_id =
        create_building_for_shared_objects(&app_state, org_id, "Shared Objects Delete Building")
            .await;

    // Create shared object
    let create_req = test::TestRequest::post()
        .uri("/api/v1/shared-objects")
        .insert_header(header::ContentType::json())
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "building_id": building_id,
            "object_category": "Books",
            "object_name": "Object to Delete",
            "description": "This object will be deleted.",
            "condition": "Good",
            "is_available": true
        }))
        .to_request();
    let create_resp = test::call_service(&app, create_req).await;
    assert_eq!(create_resp.status(), 201);
    let create_body: serde_json::Value = test::read_body_json(create_resp).await;
    let object_id = create_body["id"].as_str().unwrap();

    // Delete shared object
    let delete_req = test::TestRequest::delete()
        .uri(&format!("/api/v1/shared-objects/{}", object_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();
    let delete_resp = test::call_service(&app, delete_req).await;
    let status = delete_resp.status().as_u16();
    assert!(
        status == 200 || status == 204,
        "Should delete shared object (got {})",
        status
    );

    // Verify object is gone
    let get_req = test::TestRequest::get()
        .uri(&format!("/api/v1/shared-objects/{}", object_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();
    let get_resp = test::call_service(&app, get_req).await;
    assert_eq!(
        get_resp.status(),
        404,
        "Shared object should not exist after delete"
    );
}
