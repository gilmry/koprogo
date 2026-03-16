// E2E tests for Owner Contributions HTTP endpoints
// Tests focus on HTTP layer: endpoints, auth, JSON serialization
// Belgian legal context: Individual owner payment contributions (appels de fonds)

mod common;

use actix_web::http::header;
use actix_web::{test, App};
use koprogo_api::application::dto::{CreateBuildingDto, CreateOwnerDto, CreateUnitDto};
use koprogo_api::domain::entities::UnitType;
use koprogo_api::infrastructure::web::configure_routes;
use serde_json::json;
use serial_test::serial;
use uuid::Uuid;
// chrono used via fully qualified chrono::Utc::now() calls below

/// Helper: Create building, owner, and unit fixtures for contribution tests.
/// Returns (token, owner_id, unit_id, building_id).
async fn create_contribution_fixtures(
    app_state: &actix_web::web::Data<koprogo_api::infrastructure::web::AppState>,
    org_id: Uuid,
) -> (String, Uuid, Uuid, Uuid) {
    // 1. Register user (superadmin)
    let email = format!("contrib-test-{}@example.com", Uuid::new_v4());
    let register_req = koprogo_api::application::dto::RegisterRequest {
        email: email.clone(),
        password: "SecurePass123!".to_string(),
        first_name: "Contrib".to_string(),
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
        name: format!("Contrib Building {}", Uuid::new_v4()),
        address: "15 Avenue Louise".to_string(),
        city: "Brussels".to_string(),
        postal_code: "1050".to_string(),
        country: "Belgium".to_string(),
        total_units: 6,
        total_tantiemes: Some(1000),
        construction_year: Some(1990),
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
        first_name: "Marie".to_string(),
        last_name: "Martin".to_string(),
        email: format!("marie-{}@example.com", Uuid::new_v4()),
        phone: Some("+32 2 555 12 34".to_string()),
        address: "15 Avenue Louise".to_string(),
        city: "Brussels".to_string(),
        postal_code: "1050".to_string(),
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
        unit_number: "B2".to_string(),
        unit_type: UnitType::Apartment,
        floor: Some(2),
        surface_area: 95.0,
        quota: 0.5,
    };
    let unit = app_state
        .unit_use_cases
        .create_unit(unit_dto)
        .await
        .expect("Failed to create unit");
    let unit_id = Uuid::parse_str(&unit.id).expect("parse unit_id");

    (token, owner_id, unit_id, building_id)
}

#[actix_web::test]
#[serial]
async fn test_owner_contributions_create() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, owner_id, unit_id, _building_id) =
        create_contribution_fixtures(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let contribution_date = chrono::Utc::now().to_rfc3339();

    let req = test::TestRequest::post()
        .uri("/api/v1/owner-contributions")
        .insert_header(header::ContentType::json())
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "owner_id": owner_id.to_string(),
            "unit_id": unit_id.to_string(),
            "description": "Q1 2026 quarterly contribution — regular maintenance fund",
            "amount": 250.00,
            "contribution_type": "regular",
            "contribution_date": contribution_date
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;

    assert!(
        resp.status().is_success(),
        "Expected 201 Created for owner contribution, got: {}",
        resp.status()
    );
    assert_eq!(resp.status().as_u16(), 201);
}

#[actix_web::test]
#[serial]
async fn test_owner_contributions_get() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, owner_id, unit_id, _building_id) =
        create_contribution_fixtures(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let contribution_date = chrono::Utc::now().to_rfc3339();

    // Create a contribution
    let create_resp = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/api/v1/owner-contributions")
            .insert_header(header::ContentType::json())
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
            .set_json(json!({
                "owner_id": owner_id.to_string(),
                "unit_id": unit_id.to_string(),
                "description": "Get test contribution",
                "amount": 180.50,
                "contribution_type": "regular",
                "contribution_date": contribution_date
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
    let contribution_id = created["id"].as_str().unwrap();

    // Retrieve by ID
    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/owner-contributions/{}", contribution_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;

    assert!(
        resp.status().is_success(),
        "Expected 200 OK for get contribution, got: {}",
        resp.status()
    );
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["id"], contribution_id);
    assert_eq!(body["owner_id"], owner_id.to_string());
}

#[actix_web::test]
#[serial]
async fn test_owner_contributions_list_by_owner() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, owner_id, unit_id, _building_id) =
        create_contribution_fixtures(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let contribution_date = chrono::Utc::now().to_rfc3339();

    // Create a contribution so the list is non-empty
    test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/api/v1/owner-contributions")
            .insert_header(header::ContentType::json())
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
            .set_json(json!({
                "owner_id": owner_id.to_string(),
                "unit_id": unit_id.to_string(),
                "description": "List test contribution",
                "amount": 300.00,
                "contribution_type": "regular",
                "contribution_date": contribution_date
            }))
            .to_request(),
    )
    .await;

    // List by owner_id query parameter
    let req = test::TestRequest::get()
        .uri(&format!(
            "/api/v1/owner-contributions?owner_id={}",
            owner_id
        ))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;

    assert!(
        resp.status().is_success(),
        "Expected 200 OK for list contributions by owner, got: {}",
        resp.status()
    );
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body.is_array(), "Expected JSON array response");
    // At least one contribution should exist
    assert!(
        !body.as_array().unwrap().is_empty(),
        "Expected at least 1 contribution"
    );
}

#[actix_web::test]
#[serial]
async fn test_owner_contributions_mark_paid() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, owner_id, unit_id, _building_id) =
        create_contribution_fixtures(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let contribution_date = chrono::Utc::now().to_rfc3339();

    // Create a contribution (default status: Pending/Unpaid)
    let create_resp = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/api/v1/owner-contributions")
            .insert_header(header::ContentType::json())
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
            .set_json(json!({
                "owner_id": owner_id.to_string(),
                "unit_id": unit_id.to_string(),
                "description": "Contribution to be marked paid",
                "amount": 425.00,
                "contribution_type": "regular",
                "contribution_date": contribution_date
            }))
            .to_request(),
    )
    .await;

    let created: serde_json::Value = test::read_body_json(create_resp).await;
    let contribution_id = created["id"].as_str().unwrap();

    let payment_date = chrono::Utc::now().to_rfc3339();

    // Mark as paid via PUT /api/v1/owner-contributions/{id}/mark-paid
    let req = test::TestRequest::put()
        .uri(&format!(
            "/api/v1/owner-contributions/{}/mark-paid",
            contribution_id
        ))
        .insert_header(header::ContentType::json())
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "payment_date": payment_date,
            "payment_method": "bank_transfer",
            "payment_reference": "VIREMENT-2026-001"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;

    assert!(
        resp.status().is_success(),
        "Expected 200 OK for mark paid, got: {}",
        resp.status()
    );
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["payment_status"], "paid");
}

#[actix_web::test]
#[serial]
async fn test_owner_contributions_outstanding() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, owner_id, unit_id, _building_id) =
        create_contribution_fixtures(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let contribution_date = chrono::Utc::now().to_rfc3339();

    // Create an unpaid contribution
    test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/api/v1/owner-contributions")
            .insert_header(header::ContentType::json())
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
            .set_json(json!({
                "owner_id": owner_id.to_string(),
                "unit_id": unit_id.to_string(),
                "description": "Outstanding contribution for testing",
                "amount": 150.00,
                "contribution_type": "extraordinary",
                "contribution_date": contribution_date
            }))
            .to_request(),
    )
    .await;

    // Query outstanding contributions for this owner
    // GET /api/v1/owner-contributions/outstanding?owner_id={id}
    let req = test::TestRequest::get()
        .uri(&format!(
            "/api/v1/owner-contributions/outstanding?owner_id={}",
            owner_id
        ))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;

    assert!(
        resp.status().is_success(),
        "Expected 200 OK for outstanding contributions, got: {}",
        resp.status()
    );
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(
        body.is_array(),
        "Expected JSON array of outstanding contributions"
    );
}

#[actix_web::test]
#[serial]
async fn test_owner_contributions_outstanding_missing_owner_id() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = common::register_and_login(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Outstanding endpoint requires owner_id — missing should give 400
    let req = test::TestRequest::get()
        .uri("/api/v1/owner-contributions/outstanding")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;

    assert_eq!(
        resp.status().as_u16(),
        400,
        "Expected 400 when owner_id is missing, got: {}",
        resp.status()
    );
}

#[actix_web::test]
#[serial]
async fn test_owner_contributions_unauthorized() {
    let (app_state, _container, _org_id) = common::setup_test_db().await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Attempt without token
    let req = test::TestRequest::post()
        .uri("/api/v1/owner-contributions")
        .insert_header(header::ContentType::json())
        .set_json(json!({
            "owner_id": Uuid::new_v4().to_string(),
            "description": "Unauthorized attempt",
            "amount": 100.00,
            "contribution_type": "regular",
            "contribution_date": chrono::Utc::now().to_rfc3339()
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;

    assert_eq!(
        resp.status().as_u16(),
        401,
        "Expected 401 Unauthorized without token, got: {}",
        resp.status()
    );
}
