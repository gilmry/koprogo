// E2E tests for work order endpoint on tickets (Issue #309)
// Tests focus on HTTP layer: send-work-order endpoint, auth, status validation

mod common;

use actix_web::http::header;
use actix_web::{test, App};
use koprogo_api::application::dto::*;
use koprogo_api::infrastructure::web::configure_routes;
use koprogo_api::infrastructure::web::AppState;
use serde_json::json;
use serial_test::serial;
use uuid::Uuid;

/// Setup function shared across work order E2E tests
async fn setup_app() -> (
    actix_web::web::Data<AppState>,
    testcontainers_modules::testcontainers::ContainerAsync<
        testcontainers_modules::postgres::Postgres,
    >,
    Uuid,
) {
    common::setup_test_db().await
}

/// Helper: Create a building for ticket tests
async fn create_test_building(app_state: &actix_web::web::Data<AppState>, org_id: Uuid) -> Uuid {
    let building_dto = CreateBuildingDto {
        organization_id: org_id.to_string(),
        name: format!("Test Building WorkOrder {}", Uuid::new_v4()),
        address: "123 Work Order St".to_string(),
        city: "Brussels".to_string(),
        postal_code: "1000".to_string(),
        country: "Belgium".to_string(),
        total_units: 5,
        total_tantiemes: Some(1000),
        construction_year: Some(2020),
    };

    let building = app_state
        .building_use_cases
        .create_building(building_dto)
        .await
        .expect("Failed to create building");

    Uuid::parse_str(&building.id).expect("Failed to parse building id")
}

// ==================== Work Order Tests ====================

#[actix_web::test]
#[serial]
async fn test_send_work_order_for_in_progress_ticket() {
    let (app_state, _container, org_id) = setup_app().await;
    let token = common::register_and_login(&app_state, org_id).await;
    let building_id = create_test_building(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // 1. Create a ticket via HTTP
    let create_req = test::TestRequest::post()
        .uri("/api/v1/tickets")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "building_id": building_id.to_string(),
            "title": "Broken pipe in basement",
            "description": "Water leak detected",
            "category": "Plumbing",
            "priority": "High"
        }))
        .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    assert_eq!(create_resp.status(), 201, "Should create ticket");
    let created: serde_json::Value = test::read_body_json(create_resp).await;
    let ticket_id = created["id"].as_str().unwrap();

    // 2. Assign the ticket to a contractor
    let contractor_id = Uuid::new_v4();
    let assign_req = test::TestRequest::put()
        .uri(&format!("/api/v1/tickets/{}/assign", ticket_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({ "assigned_to": contractor_id.to_string() }))
        .to_request();

    let assign_resp = test::call_service(&app, assign_req).await;
    assert_eq!(assign_resp.status(), 200, "Should assign ticket");

    // 3. Send work order (ticket is now InProgress after assignment)
    let work_order_req = test::TestRequest::put()
        .uri(&format!("/api/v1/tickets/{}/send-work-order", ticket_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let work_order_resp = test::call_service(&app, work_order_req).await;
    assert_eq!(
        work_order_resp.status(),
        200,
        "Should send work order for InProgress ticket"
    );

    let ticket: serde_json::Value = test::read_body_json(work_order_resp).await;
    assert!(
        ticket["work_order_sent_at"].as_str().is_some(),
        "work_order_sent_at should be set"
    );
    assert_eq!(ticket["status"], "InProgress");
}

#[actix_web::test]
#[serial]
async fn test_send_work_order_rejected_for_open_ticket() {
    let (app_state, _container, org_id) = setup_app().await;
    let token = common::register_and_login(&app_state, org_id).await;
    let building_id = create_test_building(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // 1. Create a ticket (status = Open) via HTTP
    let create_req = test::TestRequest::post()
        .uri("/api/v1/tickets")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "building_id": building_id.to_string(),
            "title": "Elevator noise",
            "description": "Elevator making strange noises",
            "category": "General",
            "priority": "Medium"
        }))
        .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    assert_eq!(create_resp.status(), 201);
    let created: serde_json::Value = test::read_body_json(create_resp).await;
    let ticket_id = created["id"].as_str().unwrap();

    // 2. Try to send work order on Open ticket (should fail)
    let work_order_req = test::TestRequest::put()
        .uri(&format!("/api/v1/tickets/{}/send-work-order", ticket_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let work_order_resp = test::call_service(&app, work_order_req).await;
    assert_eq!(
        work_order_resp.status(),
        400,
        "Should reject work order for Open ticket"
    );

    let body: serde_json::Value = test::read_body_json(work_order_resp).await;
    assert!(
        body["error"].as_str().unwrap().contains("InProgress"),
        "Error should mention InProgress status requirement"
    );
}

#[actix_web::test]
#[serial]
async fn test_send_work_order_without_auth_fails() {
    let (app_state, _container, org_id) = setup_app().await;
    let token = common::register_and_login(&app_state, org_id).await;
    let building_id = create_test_building(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create ticket via HTTP
    let create_req = test::TestRequest::post()
        .uri("/api/v1/tickets")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "building_id": building_id.to_string(),
            "title": "Light broken",
            "description": "Hallway light broken",
            "category": "Electrical",
            "priority": "Low"
        }))
        .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    let created: serde_json::Value = test::read_body_json(create_resp).await;
    let ticket_id = created["id"].as_str().unwrap();

    // Try without auth
    let req = test::TestRequest::put()
        .uri(&format!("/api/v1/tickets/{}/send-work-order", ticket_id))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401, "Should require authentication");
}

#[actix_web::test]
#[serial]
async fn test_send_work_order_ticket_not_found() {
    let (app_state, _container, org_id) = setup_app().await;
    let token = common::register_and_login(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let fake_id = Uuid::new_v4();
    let req = test::TestRequest::put()
        .uri(&format!("/api/v1/tickets/{}/send-work-order", fake_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.status(),
        400,
        "Should return 400 for non-existent ticket"
    );
}

#[actix_web::test]
#[serial]
async fn test_send_work_order_rejected_for_resolved_ticket() {
    let (app_state, _container, org_id) = setup_app().await;
    let token = common::register_and_login(&app_state, org_id).await;
    let building_id = create_test_building(&app_state, org_id).await;

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
            "title": "Heating issue",
            "description": "Heating not working in unit A3",
            "category": "Heating",
            "priority": "High"
        }))
        .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    let created: serde_json::Value = test::read_body_json(create_resp).await;
    let ticket_id = created["id"].as_str().unwrap();

    // 2. Assign to contractor
    let contractor_id = Uuid::new_v4();
    let assign_req = test::TestRequest::put()
        .uri(&format!("/api/v1/tickets/{}/assign", ticket_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({ "assigned_to": contractor_id.to_string() }))
        .to_request();
    let _ = test::call_service(&app, assign_req).await;

    // 3. Start work
    let start_req = test::TestRequest::put()
        .uri(&format!("/api/v1/tickets/{}/start-work", ticket_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();
    let _ = test::call_service(&app, start_req).await;

    // 4. Resolve ticket
    let resolve_req = test::TestRequest::put()
        .uri(&format!("/api/v1/tickets/{}/resolve", ticket_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({ "resolution_notes": "Replaced heating unit" }))
        .to_request();
    let _ = test::call_service(&app, resolve_req).await;

    // 5. Try to send work order on resolved ticket (should fail)
    let work_order_req = test::TestRequest::put()
        .uri(&format!("/api/v1/tickets/{}/send-work-order", ticket_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let work_order_resp = test::call_service(&app, work_order_req).await;
    assert_eq!(
        work_order_resp.status(),
        400,
        "Should reject work order for resolved ticket"
    );
}
