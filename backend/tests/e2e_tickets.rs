// E2E tests for ticket management HTTP endpoints (Issue #85)
// Tests focus on HTTP layer: endpoints, auth, JSON serialization
// Covers complete maintenance request workflow for Belgian copropriete

mod common;

use actix_web::http::header;
use actix_web::{test, App};
use koprogo_api::application::dto::*;
use koprogo_api::domain::entities::UnitType;
use koprogo_api::infrastructure::web::configure_routes;
use koprogo_api::infrastructure::web::AppState;
use serde_json::json;
use serial_test::serial;
use uuid::Uuid;

/// Helper: Create test fixtures (building, unit, user) using the shared org from setup_test_db.
/// Returns (token, org_id, building_id, unit_id, user_id).
async fn create_test_fixtures(
    app_state: &actix_web::web::Data<AppState>,
    org_id: Uuid,
) -> (String, Uuid, Uuid, Uuid, Uuid) {
    // 1. Register user and get token
    let email = format!("ticket-test-{}@example.com", Uuid::new_v4());
    let register_req = RegisterRequest {
        email: email.clone(),
        password: "SecurePass123!".to_string(),
        first_name: "Ticket".to_string(),
        last_name: "Tester".to_string(),
        role: "superadmin".to_string(),
        organization_id: Some(org_id),
    };

    let login_response = app_state
        .auth_use_cases
        .register(register_req)
        .await
        .expect("Failed to register user");

    let user_id = login_response.user.id;
    let token = login_response.token;

    // 2. Create building (org already created by setup_test_db)
    let building_dto = CreateBuildingDto {
        organization_id: org_id.to_string(),
        name: format!("Test Building Ticket {}", Uuid::new_v4()),
        address: "456 Maintenance Ave".to_string(),
        city: "Brussels".to_string(),
        postal_code: "1000".to_string(),
        country: "Belgium".to_string(),
        total_units: 10,
        total_tantiemes: Some(1000),
        construction_year: Some(2015),
    };

    let building = app_state
        .building_use_cases
        .create_building(building_dto)
        .await
        .expect("Failed to create building");

    let building_id = Uuid::parse_str(&building.id).expect("Failed to parse building_id");

    // 3. Create unit
    let unit_dto = CreateUnitDto {
        organization_id: org_id.to_string(),
        building_id: building.id.clone(),
        unit_number: "A101".to_string(),
        unit_type: UnitType::Apartment,
        floor: Some(1),
        surface_area: 75.0,
        quota: 0.5,
    };

    let unit = app_state
        .unit_use_cases
        .create_unit(unit_dto)
        .await
        .expect("Failed to create unit");

    let unit_id = Uuid::parse_str(&unit.id).expect("Failed to parse unit_id");

    (token, org_id, building_id, unit_id, user_id)
}

// ==================== Ticket CRUD Tests ====================

#[actix_web::test]
#[serial]
async fn test_create_ticket_success() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, _org_id, building_id, unit_id, _user_id) =
        create_test_fixtures(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/v1/tickets")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "building_id": building_id.to_string(),
            "unit_id": unit_id.to_string(),
            "title": "Leaking faucet in bathroom",
            "description": "The bathroom faucet has been leaking for 2 days. Water is dripping constantly.",
            "category": "Plumbing",
            "priority": "Medium"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201, "Should create ticket successfully");

    let ticket: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(ticket["title"], "Leaking faucet in bathroom");
    assert_eq!(ticket["category"], "Plumbing");
    assert_eq!(ticket["priority"], "Medium");
    assert_eq!(ticket["status"], "Open");
    assert!(ticket["assigned_to"].is_null());
}

#[actix_web::test]
#[serial]
async fn test_create_ticket_without_auth_fails() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (_token, _org_id, building_id, unit_id, _user_id) =
        create_test_fixtures(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/v1/tickets")
        .set_json(json!({
            "building_id": building_id.to_string(),
            "unit_id": unit_id.to_string(),
            "title": "Unauthorized ticket",
            "description": "This should fail",
            "category": "Electrical",
            "priority": "Low"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401, "Should require authentication");
}

#[actix_web::test]
#[serial]
async fn test_create_ticket_all_categories() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, _org_id, building_id, unit_id, _user_id) =
        create_test_fixtures(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let categories = vec![
        "Plumbing",
        "Electrical",
        "Heating",
        "CommonAreas",
        "Elevator",
        "Security",
        "Cleaning",
        "Landscaping",
        "Other",
    ];

    for category in categories {
        let req = test::TestRequest::post()
            .uri("/api/v1/tickets")
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
            .set_json(json!({
                "building_id": building_id.to_string(),
                "unit_id": unit_id.to_string(),
                "title": format!("Test {} ticket", category),
                "description": format!("Testing {} category", category),
                "category": category,
                "priority": "Low"
            }))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(
            resp.status(),
            201,
            "Should create ticket for category {}",
            category
        );
    }
}

#[actix_web::test]
#[serial]
async fn test_create_ticket_all_priorities() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, _org_id, building_id, unit_id, _user_id) =
        create_test_fixtures(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let priorities = vec!["Low", "Medium", "High", "Critical"];

    for priority in priorities {
        let req = test::TestRequest::post()
            .uri("/api/v1/tickets")
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
            .set_json(json!({
                "building_id": building_id.to_string(),
                "unit_id": unit_id.to_string(),
                "title": format!("Test {} priority ticket", priority),
                "description": format!("Testing {} priority", priority),
                "category": "Other",
                "priority": priority
            }))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(
            resp.status(),
            201,
            "Should create ticket with priority {}",
            priority
        );
    }
}

#[actix_web::test]
#[serial]
async fn test_get_ticket_by_id() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, _org_id, building_id, unit_id, _user_id) =
        create_test_fixtures(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create ticket
    let create_req = test::TestRequest::post()
        .uri("/api/v1/tickets")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "building_id": building_id.to_string(),
            "unit_id": unit_id.to_string(),
            "title": "Broken window",
            "description": "Window in living room is broken",
            "category": "CommonAreas",
            "priority": "High"
        }))
        .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    let ticket: serde_json::Value = test::read_body_json(create_resp).await;
    let ticket_id = ticket["id"].as_str().unwrap();

    // Get ticket
    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/tickets/{}", ticket_id))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let fetched: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(fetched["id"], ticket_id);
    assert_eq!(fetched["title"], "Broken window");
}

#[actix_web::test]
#[serial]
async fn test_get_ticket_not_found() {
    let (app_state, _container, _org_id) = common::setup_test_db().await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let fake_id = Uuid::new_v4();
    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/tickets/{}", fake_id))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 404);
}

#[actix_web::test]
#[serial]
async fn test_list_building_tickets() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, _org_id, building_id, unit_id, _user_id) =
        create_test_fixtures(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create 3 tickets for the building
    for i in 1..=3 {
        let req = test::TestRequest::post()
            .uri("/api/v1/tickets")
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
            .set_json(json!({
                "building_id": building_id.to_string(),
                "unit_id": unit_id.to_string(),
                "title": format!("Ticket #{}", i),
                "description": format!("Description for ticket {}", i),
                "category": "Other",
                "priority": "Low"
            }))
            .to_request();

        test::call_service(&app, req).await;
    }

    // List all tickets for the building
    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/buildings/{}/tickets", building_id))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let tickets: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(tickets.as_array().unwrap().len(), 3);
}

#[actix_web::test]
#[serial]
async fn test_list_tickets_by_status() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, _org_id, building_id, unit_id, _user_id) =
        create_test_fixtures(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create ticket
    let create_req = test::TestRequest::post()
        .uri("/api/v1/tickets")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "building_id": building_id.to_string(),
            "unit_id": unit_id.to_string(),
            "title": "Status test ticket",
            "description": "Testing status filtering",
            "category": "Other",
            "priority": "Low"
        }))
        .to_request();

    test::call_service(&app, create_req).await;

    // List Open tickets
    let req = test::TestRequest::get()
        .uri(&format!(
            "/api/v1/buildings/{}/tickets/status/Open",
            building_id
        ))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let tickets: serde_json::Value = test::read_body_json(resp).await;
    assert!(!tickets.as_array().unwrap().is_empty());
}

#[actix_web::test]
#[serial]
async fn test_delete_ticket() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, _org_id, building_id, unit_id, _user_id) =
        create_test_fixtures(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create ticket
    let create_req = test::TestRequest::post()
        .uri("/api/v1/tickets")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "building_id": building_id.to_string(),
            "unit_id": unit_id.to_string(),
            "title": "Ticket to delete",
            "description": "This will be deleted",
            "category": "Other",
            "priority": "Low"
        }))
        .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    let ticket: serde_json::Value = test::read_body_json(create_resp).await;
    let ticket_id = ticket["id"].as_str().unwrap();

    // Delete ticket
    let delete_req = test::TestRequest::delete()
        .uri(&format!("/api/v1/tickets/{}", ticket_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let delete_resp = test::call_service(&app, delete_req).await;
    assert_eq!(delete_resp.status(), 204);

    // Verify deletion
    let get_req = test::TestRequest::get()
        .uri(&format!("/api/v1/tickets/{}", ticket_id))
        .to_request();

    let get_resp = test::call_service(&app, get_req).await;
    assert_eq!(get_resp.status(), 404);
}

// ==================== Ticket Workflow Tests ====================

#[actix_web::test]
#[serial]
async fn test_assign_ticket() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, _org_id, building_id, unit_id, user_id) =
        create_test_fixtures(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create ticket
    let create_req = test::TestRequest::post()
        .uri("/api/v1/tickets")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "building_id": building_id.to_string(),
            "unit_id": unit_id.to_string(),
            "title": "Ticket for assignment",
            "description": "Will be assigned",
            "category": "Plumbing",
            "priority": "Medium"
        }))
        .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    let ticket: serde_json::Value = test::read_body_json(create_resp).await;
    let ticket_id = ticket["id"].as_str().unwrap();
    assert_eq!(ticket["status"], "Open");
    assert!(ticket["assigned_to"].is_null());

    // Assign ticket
    let assign_req = test::TestRequest::put()
        .uri(&format!("/api/v1/tickets/{}/assign", ticket_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "assigned_to": user_id.to_string()
        }))
        .to_request();

    let assign_resp = test::call_service(&app, assign_req).await;
    assert_eq!(assign_resp.status(), 200);

    let assigned: serde_json::Value = test::read_body_json(assign_resp).await;
    assert_eq!(assigned["status"], "InProgress");
    assert_eq!(assigned["assigned_to"], user_id.to_string());
}

#[actix_web::test]
#[serial]
async fn test_start_work_on_ticket() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, _org_id, building_id, unit_id, _user_id) =
        create_test_fixtures(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create ticket
    let create_req = test::TestRequest::post()
        .uri("/api/v1/tickets")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "building_id": building_id.to_string(),
            "unit_id": unit_id.to_string(),
            "title": "Work start test",
            "description": "Testing start work",
            "category": "Electrical",
            "priority": "High"
        }))
        .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    let ticket: serde_json::Value = test::read_body_json(create_resp).await;
    let ticket_id = ticket["id"].as_str().unwrap();

    // Start work
    let start_req = test::TestRequest::put()
        .uri(&format!("/api/v1/tickets/{}/start-work", ticket_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let start_resp = test::call_service(&app, start_req).await;
    assert_eq!(start_resp.status(), 200);

    let started: serde_json::Value = test::read_body_json(start_resp).await;
    assert_eq!(started["status"], "InProgress");
}

#[actix_web::test]
#[serial]
async fn test_resolve_ticket() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, _org_id, building_id, unit_id, _user_id) =
        create_test_fixtures(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create and start work on ticket
    let create_req = test::TestRequest::post()
        .uri("/api/v1/tickets")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "building_id": building_id.to_string(),
            "unit_id": unit_id.to_string(),
            "title": "Resolve test",
            "description": "Will be resolved",
            "category": "Heating",
            "priority": "Medium"
        }))
        .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    let ticket: serde_json::Value = test::read_body_json(create_resp).await;
    let ticket_id = ticket["id"].as_str().unwrap();

    // Start work first
    let start_req = test::TestRequest::put()
        .uri(&format!("/api/v1/tickets/{}/start-work", ticket_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    test::call_service(&app, start_req).await;

    // Resolve ticket
    let resolve_req = test::TestRequest::put()
        .uri(&format!("/api/v1/tickets/{}/resolve", ticket_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "resolution_notes": "Replaced the heating valve. System is now working properly."
        }))
        .to_request();

    let resolve_resp = test::call_service(&app, resolve_req).await;
    assert_eq!(resolve_resp.status(), 200);

    let resolved: serde_json::Value = test::read_body_json(resolve_resp).await;
    assert_eq!(resolved["status"], "Resolved");
    assert!(resolved["resolved_at"].is_string());
    assert_eq!(
        resolved["resolution_notes"],
        "Replaced the heating valve. System is now working properly."
    );
}

#[actix_web::test]
#[serial]
async fn test_close_ticket() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, _org_id, building_id, unit_id, _user_id) =
        create_test_fixtures(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create, start, resolve ticket
    let create_req = test::TestRequest::post()
        .uri("/api/v1/tickets")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "building_id": building_id.to_string(),
            "unit_id": unit_id.to_string(),
            "title": "Close test",
            "description": "Will be closed",
            "category": "Cleaning",
            "priority": "Low"
        }))
        .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    let ticket: serde_json::Value = test::read_body_json(create_resp).await;
    let ticket_id = ticket["id"].as_str().unwrap();

    // Start work
    let start_req = test::TestRequest::put()
        .uri(&format!("/api/v1/tickets/{}/start-work", ticket_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    test::call_service(&app, start_req).await;

    // Resolve
    let resolve_req = test::TestRequest::put()
        .uri(&format!("/api/v1/tickets/{}/resolve", ticket_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "resolution_notes": "Cleaned the area thoroughly"
        }))
        .to_request();

    test::call_service(&app, resolve_req).await;

    // Close ticket
    let close_req = test::TestRequest::put()
        .uri(&format!("/api/v1/tickets/{}/close", ticket_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let close_resp = test::call_service(&app, close_req).await;
    assert_eq!(close_resp.status(), 200);

    let closed: serde_json::Value = test::read_body_json(close_resp).await;
    assert_eq!(closed["status"], "Closed");
    assert!(closed["closed_at"].is_string());
}

#[actix_web::test]
#[serial]
async fn test_cancel_ticket() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, _org_id, building_id, unit_id, _user_id) =
        create_test_fixtures(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create ticket
    let create_req = test::TestRequest::post()
        .uri("/api/v1/tickets")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "building_id": building_id.to_string(),
            "unit_id": unit_id.to_string(),
            "title": "Cancel test",
            "description": "Will be cancelled",
            "category": "Other",
            "priority": "Low"
        }))
        .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    let ticket: serde_json::Value = test::read_body_json(create_resp).await;
    let ticket_id = ticket["id"].as_str().unwrap();

    // Cancel ticket
    let cancel_req = test::TestRequest::put()
        .uri(&format!("/api/v1/tickets/{}/cancel", ticket_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "reason": "Duplicate ticket, already reported"
        }))
        .to_request();

    let cancel_resp = test::call_service(&app, cancel_req).await;
    assert_eq!(cancel_resp.status(), 200);

    let cancelled: serde_json::Value = test::read_body_json(cancel_resp).await;
    assert_eq!(cancelled["status"], "Cancelled");
}

#[actix_web::test]
#[serial]
async fn test_reopen_ticket() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, _org_id, building_id, unit_id, _user_id) =
        create_test_fixtures(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create, start, resolve, close ticket
    let create_req = test::TestRequest::post()
        .uri("/api/v1/tickets")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "building_id": building_id.to_string(),
            "unit_id": unit_id.to_string(),
            "title": "Reopen test",
            "description": "Will be reopened",
            "category": "Security",
            "priority": "High"
        }))
        .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    let ticket: serde_json::Value = test::read_body_json(create_resp).await;
    let ticket_id = ticket["id"].as_str().unwrap();

    // Start work
    let start_req = test::TestRequest::put()
        .uri(&format!("/api/v1/tickets/{}/start-work", ticket_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    test::call_service(&app, start_req).await;

    // Resolve
    let resolve_req = test::TestRequest::put()
        .uri(&format!("/api/v1/tickets/{}/resolve", ticket_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "resolution_notes": "Fixed the security issue"
        }))
        .to_request();

    test::call_service(&app, resolve_req).await;

    // Close
    let close_req = test::TestRequest::put()
        .uri(&format!("/api/v1/tickets/{}/close", ticket_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    test::call_service(&app, close_req).await;

    // Reopen ticket
    let reopen_req = test::TestRequest::put()
        .uri(&format!("/api/v1/tickets/{}/reopen", ticket_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "reason": "Issue returned, needs further investigation"
        }))
        .to_request();

    let reopen_resp = test::call_service(&app, reopen_req).await;
    assert_eq!(reopen_resp.status(), 200);

    let reopened: serde_json::Value = test::read_body_json(reopen_resp).await;
    assert_eq!(reopened["status"], "InProgress");
}

#[actix_web::test]
#[serial]
async fn test_get_ticket_statistics() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, _org_id, building_id, unit_id, _user_id) =
        create_test_fixtures(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create tickets with different statuses
    for i in 1..=3 {
        let req = test::TestRequest::post()
            .uri("/api/v1/tickets")
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
            .set_json(json!({
                "building_id": building_id.to_string(),
                "unit_id": unit_id.to_string(),
                "title": format!("Stats ticket {}", i),
                "description": format!("For statistics test {}", i),
                "category": "Other",
                "priority": "Low"
            }))
            .to_request();

        test::call_service(&app, req).await;
    }

    // Get statistics
    let stats_req = test::TestRequest::get()
        .uri(&format!(
            "/api/v1/buildings/{}/tickets/statistics",
            building_id
        ))
        .to_request();

    let stats_resp = test::call_service(&app, stats_req).await;
    assert_eq!(stats_resp.status(), 200);

    let stats: serde_json::Value = test::read_body_json(stats_resp).await;
    assert!(stats["total"].as_i64().unwrap() >= 3);
}

#[actix_web::test]
#[serial]
async fn test_complete_ticket_lifecycle() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, _org_id, building_id, unit_id, user_id) =
        create_test_fixtures(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // 1. Create ticket
    let create_req = test::TestRequest::post()
        .uri("/api/v1/tickets")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "building_id": building_id.to_string(),
            "unit_id": unit_id.to_string(),
            "title": "Complete lifecycle test",
            "description": "Testing full ticket workflow from creation to closure",
            "category": "Plumbing",
            "priority": "Critical"
        }))
        .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    let ticket: serde_json::Value = test::read_body_json(create_resp).await;
    let ticket_id = ticket["id"].as_str().unwrap();
    assert_eq!(ticket["status"], "Open");
    assert_eq!(ticket["priority"], "Critical");

    // 2. Assign ticket
    let assign_req = test::TestRequest::put()
        .uri(&format!("/api/v1/tickets/{}/assign", ticket_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "assigned_to": user_id.to_string()
        }))
        .to_request();

    let assign_resp = test::call_service(&app, assign_req).await;
    let assigned: serde_json::Value = test::read_body_json(assign_resp).await;
    assert_eq!(assigned["status"], "InProgress");
    assert_eq!(assigned["assigned_to"], user_id.to_string());

    // 3. Resolve ticket
    let resolve_req = test::TestRequest::put()
        .uri(&format!("/api/v1/tickets/{}/resolve", ticket_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "resolution_notes": "Emergency plumbing fixed. Replaced burst pipe and checked water pressure. No further issues detected."
        }))
        .to_request();

    let resolve_resp = test::call_service(&app, resolve_req).await;
    let resolved: serde_json::Value = test::read_body_json(resolve_resp).await;
    assert_eq!(resolved["status"], "Resolved");
    assert!(resolved["resolved_at"].is_string());

    // 4. Close ticket
    let close_req = test::TestRequest::put()
        .uri(&format!("/api/v1/tickets/{}/close", ticket_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let close_resp = test::call_service(&app, close_req).await;
    let closed: serde_json::Value = test::read_body_json(close_resp).await;
    assert_eq!(closed["status"], "Closed");
    assert!(closed["closed_at"].is_string());

    // 5. Verify in building tickets list
    let list_req = test::TestRequest::get()
        .uri(&format!("/api/v1/buildings/{}/tickets", building_id))
        .to_request();

    let list_resp = test::call_service(&app, list_req).await;
    let tickets: serde_json::Value = test::read_body_json(list_resp).await;
    assert!(!tickets.as_array().unwrap().is_empty());

    // 6. Verify in closed status list
    let closed_list_req = test::TestRequest::get()
        .uri(&format!(
            "/api/v1/buildings/{}/tickets/status/Closed",
            building_id
        ))
        .to_request();

    let closed_list_resp = test::call_service(&app, closed_list_req).await;
    let closed_tickets: serde_json::Value = test::read_body_json(closed_list_resp).await;
    assert!(!closed_tickets.as_array().unwrap().is_empty());
}
