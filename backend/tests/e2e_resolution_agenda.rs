// E2E tests for resolution agenda_item_index field (Issue #310)
// Tests focus on HTTP layer: create resolution with/without agenda_item_index

mod common;

use actix_web::http::header;
use actix_web::{test, App};
use chrono::{Duration, Utc};
use koprogo_api::application::dto::*;
use koprogo_api::domain::entities::MeetingType;
use koprogo_api::infrastructure::web::configure_routes;
use koprogo_api::infrastructure::web::AppState;
use serde_json::json;
use serial_test::serial;
use uuid::Uuid;

/// Setup function shared across resolution agenda E2E tests
async fn setup_app() -> (
    actix_web::web::Data<AppState>,
    testcontainers_modules::testcontainers::ContainerAsync<
        testcontainers_modules::postgres::Postgres,
    >,
    Uuid,
) {
    common::setup_test_db().await
}

/// Helper: Create building, meeting, and return (token, meeting_id)
async fn create_meeting_fixtures(
    app_state: &actix_web::web::Data<AppState>,
    org_id: Uuid,
) -> (String, Uuid, Uuid) {
    // Register and login
    let token = common::register_and_login(app_state, org_id).await;

    // Create building
    let building_dto = CreateBuildingDto {
        organization_id: org_id.to_string(),
        name: format!("Agenda Test Building {}", Uuid::new_v4()),
        address: "789 Agenda Blvd".to_string(),
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

    let building_id = Uuid::parse_str(&building.id).expect("Failed to parse building id");

    // Create meeting
    let meeting_req = CreateMeetingRequest {
        organization_id: org_id,
        building_id,
        meeting_type: MeetingType::Ordinary,
        title: "AG for Agenda Item Testing".to_string(),
        description: Some("Testing agenda_item_index on resolutions".to_string()),
        scheduled_date: Utc::now() + Duration::days(14),
        location: "Conference Room".to_string(),
    };

    let meeting = app_state
        .meeting_use_cases
        .create_meeting(meeting_req)
        .await
        .expect("Failed to create meeting");

    (token, building_id, meeting.id)
}

// ==================== Resolution Agenda Item Tests ====================

#[actix_web::test]
#[serial]
async fn test_create_resolution_with_agenda_item_index() {
    let (app_state, _container, org_id) = setup_app().await;
    let (token, _building_id, meeting_id) = create_meeting_fixtures(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::post()
        .uri(&format!("/api/v1/meetings/{}/resolutions", meeting_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "meeting_id": meeting_id.to_string(),
            "title": "Approve Renovation Budget",
            "description": "Vote on the facade renovation budget for 2026",
            "resolution_type": "ordinary",
            "majority_required": "absolute",
            "agenda_item_index": 3
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.status(),
        201,
        "Should create resolution with agenda_item_index"
    );

    let resolution: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(resolution["title"], "Approve Renovation Budget");
    assert_eq!(resolution["agenda_item_index"], 3);
    assert_eq!(resolution["status"], "pending");
}

#[actix_web::test]
#[serial]
async fn test_create_resolution_without_agenda_item_index() {
    let (app_state, _container, org_id) = setup_app().await;
    let (token, _building_id, meeting_id) = create_meeting_fixtures(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::post()
        .uri(&format!("/api/v1/meetings/{}/resolutions", meeting_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "meeting_id": meeting_id.to_string(),
            "title": "Approve Annual Budget",
            "description": "Vote to approve the general budget",
            "resolution_type": "ordinary",
            "majority_required": "absolute"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.status(),
        201,
        "Should create resolution without agenda_item_index"
    );

    let resolution: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(resolution["title"], "Approve Annual Budget");
    assert!(
        resolution["agenda_item_index"].is_null(),
        "agenda_item_index should be null when not provided"
    );
    assert_eq!(resolution["majority_required"], "absolute");
}

#[actix_web::test]
#[serial]
async fn test_create_resolution_with_zero_agenda_item_index() {
    let (app_state, _container, org_id) = setup_app().await;
    let (token, _building_id, meeting_id) = create_meeting_fixtures(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Agenda item index 0 should be valid (first item)
    let req = test::TestRequest::post()
        .uri(&format!("/api/v1/meetings/{}/resolutions", meeting_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "meeting_id": meeting_id.to_string(),
            "title": "Opening Resolution",
            "description": "First agenda item resolution",
            "resolution_type": "ordinary",
            "majority_required": "absolute",
            "agenda_item_index": 0
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201, "Should accept agenda_item_index of 0");

    let resolution: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(resolution["agenda_item_index"], 0);
}

#[actix_web::test]
#[serial]
async fn test_create_multiple_resolutions_with_different_agenda_indices() {
    let (app_state, _container, org_id) = setup_app().await;
    let (token, _building_id, meeting_id) = create_meeting_fixtures(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create first resolution with agenda_item_index = 1
    let req1 = test::TestRequest::post()
        .uri(&format!("/api/v1/meetings/{}/resolutions", meeting_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "meeting_id": meeting_id.to_string(),
            "title": "Budget Approval",
            "description": "First agenda item",
            "resolution_type": "ordinary",
            "majority_required": "absolute",
            "agenda_item_index": 1
        }))
        .to_request();

    let resp1 = test::call_service(&app, req1).await;
    assert_eq!(resp1.status(), 201);

    // Create second resolution with agenda_item_index = 2
    let req2 = test::TestRequest::post()
        .uri(&format!("/api/v1/meetings/{}/resolutions", meeting_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "meeting_id": meeting_id.to_string(),
            "title": "Syndic Election",
            "description": "Second agenda item",
            "resolution_type": "ordinary",
            "majority_required": "absolute",
            "agenda_item_index": 2
        }))
        .to_request();

    let resp2 = test::call_service(&app, req2).await;
    assert_eq!(resp2.status(), 201);

    // List all resolutions for the meeting
    let list_req = test::TestRequest::get()
        .uri(&format!("/api/v1/meetings/{}/resolutions", meeting_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let list_resp = test::call_service(&app, list_req).await;
    assert_eq!(list_resp.status(), 200);

    let resolutions: Vec<serde_json::Value> = test::read_body_json(list_resp).await;
    assert_eq!(resolutions.len(), 2, "Should have 2 resolutions");

    // Verify agenda_item_index values are present
    let indices: Vec<Option<u64>> = resolutions
        .iter()
        .map(|r| r["agenda_item_index"].as_u64())
        .collect();
    assert!(indices.contains(&Some(1)));
    assert!(indices.contains(&Some(2)));
}

#[actix_web::test]
#[serial]
async fn test_create_resolution_agenda_item_persists_on_get() {
    let (app_state, _container, org_id) = setup_app().await;
    let (token, _building_id, meeting_id) = create_meeting_fixtures(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create resolution with agenda_item_index
    let create_req = test::TestRequest::post()
        .uri(&format!("/api/v1/meetings/{}/resolutions", meeting_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "meeting_id": meeting_id.to_string(),
            "title": "Facade Renovation",
            "description": "Vote on facade renovation project",
            "resolution_type": "extraordinary",
            "majority_required": "two_thirds",
            "agenda_item_index": 5
        }))
        .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    assert_eq!(create_resp.status(), 201);

    let created: serde_json::Value = test::read_body_json(create_resp).await;
    let resolution_id = created["id"].as_str().unwrap();

    // GET the resolution and verify agenda_item_index persists
    let get_req = test::TestRequest::get()
        .uri(&format!("/api/v1/resolutions/{}", resolution_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let get_resp = test::call_service(&app, get_req).await;
    assert_eq!(get_resp.status(), 200);

    let resolution: serde_json::Value = test::read_body_json(get_resp).await;
    assert_eq!(
        resolution["agenda_item_index"], 5,
        "agenda_item_index should persist on GET"
    );
    assert_eq!(resolution["resolution_type"], "extraordinary");
    assert_eq!(resolution["majority_required"], "two_thirds");
}
