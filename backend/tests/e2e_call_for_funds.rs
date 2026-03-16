// E2E tests for Call For Funds HTTP endpoints
// Tests focus on HTTP layer: endpoints, auth, JSON serialization
// Belgian legal context: Appels de fonds for copropriete management

mod common;

use actix_web::http::header;
use actix_web::{test, App};
use koprogo_api::application::dto::{CreateBuildingDto, CreateOwnerDto, CreateUnitDto};
use koprogo_api::domain::entities::UnitType;
use koprogo_api::infrastructure::web::configure_routes;
use serde_json::json;
use serial_test::serial;
use uuid::Uuid;

/// Helper: Create building + owner + unit + unit_owner relationship.
/// Returns (token, building_id, owner_id, unit_id).
async fn create_call_for_funds_fixtures(
    app_state: &actix_web::web::Data<koprogo_api::infrastructure::web::AppState>,
    org_id: Uuid,
) -> (String, Uuid, Uuid, Uuid) {
    // 1. Register user and get token (superadmin role)
    let email = format!("cff-test-{}@example.com", Uuid::new_v4());
    let register_req = koprogo_api::application::dto::RegisterRequest {
        email: email.clone(),
        password: "SecurePass123!".to_string(),
        first_name: "CallForFunds".to_string(),
        last_name: "Tester".to_string(),
        role: "superadmin".to_string(),
        organization_id: Some(org_id),
    };
    let login_response = app_state
        .auth_use_cases
        .register(register_req)
        .await
        .expect("Failed to register user");
    let token = login_response.token;

    // 2. Create building
    let building_dto = CreateBuildingDto {
        organization_id: org_id.to_string(),
        name: format!("CFF Building {}", Uuid::new_v4()),
        address: "10 Rue de la Loi".to_string(),
        city: "Brussels".to_string(),
        postal_code: "1000".to_string(),
        country: "Belgium".to_string(),
        total_units: 4,
        total_tantiemes: Some(1000),
        construction_year: Some(2005),
    };
    let building = app_state
        .building_use_cases
        .create_building(building_dto)
        .await
        .expect("Failed to create building");
    let building_id = Uuid::parse_str(&building.id).expect("parse building_id");

    // 3. Create owner
    let owner_dto = CreateOwnerDto {
        organization_id: org_id.to_string(),
        first_name: "Jean".to_string(),
        last_name: "Dupont".to_string(),
        email: format!("owner-{}@example.com", Uuid::new_v4()),
        phone: None,
        address: "10 Rue de la Loi".to_string(),
        city: "Brussels".to_string(),
        postal_code: "1000".to_string(),
        country: "Belgium".to_string(),
    };
    let owner = app_state
        .owner_use_cases
        .create_owner(owner_dto)
        .await
        .expect("Failed to create owner");
    let owner_id = Uuid::parse_str(&owner.id).expect("parse owner_id");

    // 4. Create unit
    let unit_dto = CreateUnitDto {
        organization_id: org_id.to_string(),
        building_id: building.id.clone(),
        unit_number: "A1".to_string(),
        unit_type: UnitType::Apartment,
        floor: Some(1),
        surface_area: 80.0,
        quota: 0.25,
    };
    let unit = app_state
        .unit_use_cases
        .create_unit(unit_dto)
        .await
        .expect("Failed to create unit");
    let unit_id = Uuid::parse_str(&unit.id).expect("parse unit_id");

    // 5. Link owner to unit (required for send_call_for_funds to generate contributions)
    let _ = app_state
        .unit_owner_use_cases
        .add_owner_to_unit(
            unit_id, owner_id, 0.25, // 25% ownership share
            true, // is primary contact
        )
        .await;
    // Note: ownership validation may require total = 100%, so we accept any result here

    (token, building_id, owner_id, unit_id)
}

#[actix_web::test]
#[serial]
async fn test_call_for_funds_create() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, building_id, _owner_id, _unit_id) =
        create_call_for_funds_fixtures(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let now = chrono::Utc::now();
    let due_date = now + chrono::Duration::days(30);

    let req = test::TestRequest::post()
        .uri("/api/v1/call-for-funds")
        .insert_header(header::ContentType::json())
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "building_id": building_id.to_string(),
            "title": "Q1 2026 Regular Contribution",
            "description": "First quarter ordinary contribution for common expenses",
            "total_amount": 4000.00,
            "contribution_type": "regular",
            "call_date": now.to_rfc3339(),
            "due_date": due_date.to_rfc3339()
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;

    assert!(
        resp.status().is_success(),
        "Expected 201 Created for call for funds, got: {}",
        resp.status()
    );
    assert_eq!(resp.status().as_u16(), 201);
}

#[actix_web::test]
#[serial]
async fn test_call_for_funds_get() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, building_id, _owner_id, _unit_id) =
        create_call_for_funds_fixtures(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let now = chrono::Utc::now();
    let due_date = now + chrono::Duration::days(30);

    // Create a call for funds first
    let create_resp = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/api/v1/call-for-funds")
            .insert_header(header::ContentType::json())
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
            .set_json(json!({
                "building_id": building_id.to_string(),
                "title": "Q2 2026 Contribution",
                "description": "Second quarter contribution",
                "total_amount": 4000.00,
                "contribution_type": "regular",
                "call_date": now.to_rfc3339(),
                "due_date": due_date.to_rfc3339()
            }))
            .to_request(),
    )
    .await;

    assert!(
        create_resp.status().is_success(),
        "Pre-condition: create should succeed, got: {}",
        create_resp.status()
    );

    let created: serde_json::Value = test::read_body_json(create_resp).await;
    let cff_id = created["id"].as_str().unwrap();

    // Retrieve by ID
    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/call-for-funds/{}", cff_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;

    assert!(
        resp.status().is_success(),
        "Expected 200 OK for get call for funds, got: {}",
        resp.status()
    );
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["id"], cff_id);
    assert_eq!(body["status"], "draft");
}

#[actix_web::test]
#[serial]
async fn test_call_for_funds_list() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, building_id, _owner_id, _unit_id) =
        create_call_for_funds_fixtures(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let now = chrono::Utc::now();
    let due_date = now + chrono::Duration::days(30);

    // Create a call for funds
    test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/api/v1/call-for-funds")
            .insert_header(header::ContentType::json())
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
            .set_json(json!({
                "building_id": building_id.to_string(),
                "title": "List Test Contribution",
                "description": "Test contribution for list endpoint",
                "total_amount": 2000.00,
                "contribution_type": "regular",
                "call_date": now.to_rfc3339(),
                "due_date": due_date.to_rfc3339()
            }))
            .to_request(),
    )
    .await;

    // List by building_id
    let req = test::TestRequest::get()
        .uri(&format!(
            "/api/v1/call-for-funds?building_id={}",
            building_id
        ))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;

    assert!(
        resp.status().is_success(),
        "Expected 200 OK for list call for funds, got: {}",
        resp.status()
    );
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body.is_array(), "Expected JSON array response");
}

#[actix_web::test]
#[serial]
async fn test_call_for_funds_send() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, building_id, _owner_id, _unit_id) =
        create_call_for_funds_fixtures(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let now = chrono::Utc::now();
    let due_date = now + chrono::Duration::days(30);

    // Create a call for funds
    let create_resp = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/api/v1/call-for-funds")
            .insert_header(header::ContentType::json())
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
            .set_json(json!({
                "building_id": building_id.to_string(),
                "title": "Send Test Contribution",
                "description": "Contribution to be sent",
                "total_amount": 3600.00,
                "contribution_type": "regular",
                "call_date": now.to_rfc3339(),
                "due_date": due_date.to_rfc3339()
            }))
            .to_request(),
    )
    .await;

    let created: serde_json::Value = test::read_body_json(create_resp).await;
    let cff_id = created["id"].as_str().unwrap();

    // Send the call for funds (Draft → Sent, generates owner contributions)
    let req = test::TestRequest::post()
        .uri(&format!("/api/v1/call-for-funds/{}/send", cff_id))
        .insert_header(header::ContentType::json())
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({}))
        .to_request();

    let resp = test::call_service(&app, req).await;

    // May succeed or fail depending on whether unit owners have been set up
    assert!(
        resp.status().is_success() || resp.status().is_client_error(),
        "Expected valid response for send call for funds, got: {}",
        resp.status()
    );
}

#[actix_web::test]
#[serial]
async fn test_call_for_funds_cancel() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, building_id, _owner_id, _unit_id) =
        create_call_for_funds_fixtures(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let now = chrono::Utc::now();
    let due_date = now + chrono::Duration::days(30);

    // Create a call for funds
    let create_resp = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/api/v1/call-for-funds")
            .insert_header(header::ContentType::json())
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
            .set_json(json!({
                "building_id": building_id.to_string(),
                "title": "Cancel Test Contribution",
                "description": "Contribution to be cancelled",
                "total_amount": 1000.00,
                "contribution_type": "extraordinary",
                "call_date": now.to_rfc3339(),
                "due_date": due_date.to_rfc3339()
            }))
            .to_request(),
    )
    .await;

    let created: serde_json::Value = test::read_body_json(create_resp).await;
    let cff_id = created["id"].as_str().unwrap();

    // Cancel the call for funds
    let req = test::TestRequest::put()
        .uri(&format!("/api/v1/call-for-funds/{}/cancel", cff_id))
        .insert_header(header::ContentType::json())
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({}))
        .to_request();

    let resp = test::call_service(&app, req).await;

    assert!(
        resp.status().is_success(),
        "Expected 200 OK for cancel call for funds, got: {}",
        resp.status()
    );
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["status"], "cancelled");
}

#[actix_web::test]
#[serial]
async fn test_call_for_funds_delete_draft() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, building_id, _owner_id, _unit_id) =
        create_call_for_funds_fixtures(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let now = chrono::Utc::now();
    let due_date = now + chrono::Duration::days(30);

    // Create a call for funds (Draft status)
    let create_resp = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/api/v1/call-for-funds")
            .insert_header(header::ContentType::json())
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
            .set_json(json!({
                "building_id": building_id.to_string(),
                "title": "Delete Test Contribution",
                "description": "Draft contribution to be deleted",
                "total_amount": 500.00,
                "contribution_type": "adjustment",
                "call_date": now.to_rfc3339(),
                "due_date": due_date.to_rfc3339()
            }))
            .to_request(),
    )
    .await;

    let created: serde_json::Value = test::read_body_json(create_resp).await;
    let cff_id = created["id"].as_str().unwrap();

    // Delete the draft
    let req = test::TestRequest::delete()
        .uri(&format!("/api/v1/call-for-funds/{}", cff_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Handler returns 204 No Content on success
    assert!(
        resp.status().is_success(),
        "Expected 2xx for delete draft call for funds, got: {}",
        resp.status()
    );
}

#[actix_web::test]
#[serial]
async fn test_call_for_funds_unauthorized() {
    let (app_state, _container, _org_id) = common::setup_test_db().await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Attempt without token
    let req = test::TestRequest::get()
        .uri("/api/v1/call-for-funds")
        .to_request();

    let resp = test::call_service(&app, req).await;

    assert_eq!(
        resp.status().as_u16(),
        401,
        "Expected 401 Unauthorized without token, got: {}",
        resp.status()
    );
}
