// E2E tests for meeting HTTP endpoints (Issue #75)
// Tests focus on HTTP layer: endpoints, auth, JSON serialization
// BDD tests (meetings.feature, meetings_manage.feature) cover business scenarios

mod common;

use actix_web::http::header;
use actix_web::test;
use chrono::{Duration, Utc};
use koprogo_api::application::dto::{LoginRequest, RegisterRequest};
use koprogo_api::infrastructure::web::configure_routes;
use serde_json::json;
use serial_test::serial;
use uuid::Uuid;

/// Helper: Create organization, user, and building for tests
async fn create_test_fixtures(
    app_state: &actix_web::web::Data<koprogo_api::infrastructure::web::AppState>,
    org_id: Uuid,
) -> (String, Uuid, Uuid) {
    // Register user
    let email = format!("e2e+{}@test.com", Uuid::new_v4());
    let reg = RegisterRequest {
        email: email.clone(),
        password: "Passw0rd!".to_string(),
        first_name: "E2E".to_string(),
        last_name: "User".to_string(),
        role: "superadmin".to_string(),
        organization_id: Some(org_id),
    };

    let _register_result = app_state
        .auth_use_cases
        .register(reg)
        .await
        .expect("Failed to register user");

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

    let token = login_result.token;

    // Create building via HTTP API
    let app = test::init_service(
        actix_web::App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let building_req = test::TestRequest::post()
        .uri("/api/v1/buildings")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "organization_id": org_id.to_string(),
            "name": "Test Building",
            "address": "123 Main St",
            "city": "Brussels",
            "postal_code": "1000",
            "country": "Belgium",
            "total_units": 10,
            "construction_year": 2020
        }))
        .to_request();

    let building_resp = test::call_service(&app, building_req).await;
    assert_eq!(building_resp.status(), 201, "Failed to create building");

    let building: serde_json::Value = test::read_body_json(building_resp).await;
    let building_id =
        Uuid::parse_str(building["id"].as_str().unwrap()).expect("Invalid building ID");

    (token, org_id, building_id)
}

//
// TEST: POST /meetings (Create meeting)
//

#[actix_web::test]
#[serial]
async fn test_create_meeting_success() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, org_id, building_id) = create_test_fixtures(&app_state, org_id).await;

    let app = test::init_service(
        actix_web::App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let scheduled_date = Utc::now() + Duration::days(30);

    let req = test::TestRequest::post()
        .uri("/api/v1/meetings")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "organization_id": org_id.to_string(),
            "building_id": building_id.to_string(),
            "meeting_type": "Ordinary",
            "title": "AG Ordinaire 2025",
            "description": "Assemblée générale ordinaire annuelle",
            "scheduled_date": scheduled_date.to_rfc3339(),
            "location": "Salle communautaire"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201, "Should create meeting successfully");

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["building_id"], building_id.to_string());
    assert_eq!(body["title"], "AG Ordinaire 2025");
    assert_eq!(body["meeting_type"], "Ordinary");
    assert_eq!(body["status"], "Scheduled");
    assert!(body["agenda"].is_array());
}

#[actix_web::test]
#[serial]
async fn test_create_meeting_without_auth_fails() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (_token, org_id, building_id) = create_test_fixtures(&app_state, org_id).await;

    let app = test::init_service(
        actix_web::App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let scheduled_date = Utc::now() + Duration::days(30);

    let req = test::TestRequest::post()
        .uri("/api/v1/meetings")
        .set_json(json!({
            "organization_id": org_id.to_string(),
            "building_id": building_id.to_string(),
            "meeting_type": "Ordinary",
            "title": "Unauthorized Meeting",
            "scheduled_date": scheduled_date.to_rfc3339(),
            "location": "Somewhere"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401, "Should reject unauthorized request");
}

//
// TEST: GET /meetings/:id (Get meeting details)
//

#[actix_web::test]
#[serial]
async fn test_get_meeting_by_id_success() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, org_id, building_id) = create_test_fixtures(&app_state, org_id).await;

    let app = test::init_service(
        actix_web::App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create a meeting first
    let scheduled_date = Utc::now() + Duration::days(30);

    let req = test::TestRequest::post()
        .uri("/api/v1/meetings")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "organization_id": org_id.to_string(),
            "building_id": building_id.to_string(),
            "meeting_type": "Ordinary",
            "title": "Test Meeting",
            "scheduled_date": scheduled_date.to_rfc3339(),
            "location": "Test Location"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    let created: serde_json::Value = test::read_body_json(resp).await;
    let meeting_id = created["id"].as_str().unwrap();

    // Get the meeting
    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/meetings/{}", meeting_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["id"], meeting_id);
    assert_eq!(body["title"], "Test Meeting");
}

#[actix_web::test]
#[serial]
async fn test_get_meeting_not_found() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, _org_id, _building_id) = create_test_fixtures(&app_state, org_id).await;

    let app = test::init_service(
        actix_web::App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let fake_id = Uuid::new_v4();

    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/meetings/{}", fake_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.status(),
        404,
        "Should return 404 for non-existent meeting"
    );
}

//
// TEST: GET /meetings (List all meetings - paginated)
//

#[actix_web::test]
#[serial]
async fn test_list_meetings_paginated() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, org_id, building_id) = create_test_fixtures(&app_state, org_id).await;

    let app = test::init_service(
        actix_web::App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create 3 meetings
    for i in 1..=3 {
        let scheduled_date = Utc::now() + Duration::days(30 + i);

        let req = test::TestRequest::post()
            .uri("/api/v1/meetings")
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
            .set_json(json!({
                "organization_id": org_id.to_string(),
                "building_id": building_id.to_string(),
                "meeting_type": "Ordinary",
                "title": format!("Meeting {}", i),
                "scheduled_date": scheduled_date.to_rfc3339(),
                "location": "Location"
            }))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 201);
    }

    // List meetings with pagination
    let req = test::TestRequest::get()
        .uri("/api/v1/meetings?page=1&per_page=10")
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
// TEST: GET /buildings/:id/meetings (List meetings by building)
//

#[actix_web::test]
#[serial]
async fn test_list_meetings_by_building() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, org_id, building_id) = create_test_fixtures(&app_state, org_id).await;

    let app = test::init_service(
        actix_web::App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create 2 meetings for this building
    for i in 1..=2 {
        let scheduled_date = Utc::now() + Duration::days(30 + i);

        let req = test::TestRequest::post()
            .uri("/api/v1/meetings")
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
            .set_json(json!({
                "organization_id": org_id.to_string(),
                "building_id": building_id.to_string(),
                "meeting_type": "Ordinary",
                "title": format!("Building Meeting {}", i),
                "scheduled_date": scheduled_date.to_rfc3339(),
                "location": "Building Hall"
            }))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 201);
    }

    // List meetings for building
    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/buildings/{}/meetings", building_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body.is_array());
    assert!(body.as_array().unwrap().len() >= 2);

    // Verify all meetings belong to this building
    for meeting in body.as_array().unwrap() {
        assert_eq!(meeting["building_id"], building_id.to_string());
    }
}

//
// TEST: PUT /meetings/:id (Update meeting)
//

#[actix_web::test]
#[serial]
async fn test_update_meeting_success() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, org_id, building_id) = create_test_fixtures(&app_state, org_id).await;

    let app = test::init_service(
        actix_web::App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create meeting
    let scheduled_date = Utc::now() + Duration::days(30);

    let req = test::TestRequest::post()
        .uri("/api/v1/meetings")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "organization_id": org_id.to_string(),
            "building_id": building_id.to_string(),
            "meeting_type": "Ordinary",
            "title": "Original Title",
            "scheduled_date": scheduled_date.to_rfc3339(),
            "location": "Original Location"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    let created: serde_json::Value = test::read_body_json(resp).await;
    let meeting_id = created["id"].as_str().unwrap();

    // Update meeting
    let req = test::TestRequest::put()
        .uri(&format!("/api/v1/meetings/{}", meeting_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "title": "Updated Title",
            "location": "Updated Location"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200, "Should update meeting successfully");

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["title"], "Updated Title");
    assert_eq!(body["location"], "Updated Location");
}

//
// TEST: POST /meetings/:id/agenda (Add agenda item)
//

#[actix_web::test]
#[serial]
async fn test_add_agenda_item_success() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, org_id, building_id) = create_test_fixtures(&app_state, org_id).await;

    let app = test::init_service(
        actix_web::App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create meeting
    let scheduled_date = Utc::now() + Duration::days(30);

    let req = test::TestRequest::post()
        .uri("/api/v1/meetings")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "organization_id": org_id.to_string(),
            "building_id": building_id.to_string(),
            "meeting_type": "Ordinary",
            "title": "Meeting with Agenda",
            "scheduled_date": scheduled_date.to_rfc3339(),
            "location": "Hall"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    let created: serde_json::Value = test::read_body_json(resp).await;
    let meeting_id = created["id"].as_str().unwrap();

    // Add agenda item
    let req = test::TestRequest::post()
        .uri(&format!("/api/v1/meetings/{}/agenda", meeting_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "item": "1. Approbation du budget"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200, "Should add agenda item successfully");

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body["agenda"].is_array());
    assert!(!body["agenda"].as_array().unwrap().is_empty());
    assert!(body["agenda"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item.as_str().unwrap().contains("Approbation du budget")));
}

//
// TEST: POST /meetings/:id/complete (Complete meeting)
//

#[actix_web::test]
#[serial]
async fn test_complete_meeting_success() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, org_id, building_id) = create_test_fixtures(&app_state, org_id).await;

    let app = test::init_service(
        actix_web::App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create meeting
    let scheduled_date = Utc::now() + Duration::days(30);

    let req = test::TestRequest::post()
        .uri("/api/v1/meetings")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "organization_id": org_id.to_string(),
            "building_id": building_id.to_string(),
            "meeting_type": "Ordinary",
            "title": "Meeting to Complete",
            "scheduled_date": scheduled_date.to_rfc3339(),
            "location": "Hall"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    let created: serde_json::Value = test::read_body_json(resp).await;
    let meeting_id = created["id"].as_str().unwrap();

    // Complete meeting
    let req = test::TestRequest::post()
        .uri(&format!("/api/v1/meetings/{}/complete", meeting_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "attendees_count": 42
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200, "Should complete meeting successfully");

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["status"], "Completed");
    assert_eq!(body["attendees_count"], 42);
}

//
// TEST: POST /meetings/:id/cancel (Cancel meeting)
//

#[actix_web::test]
#[serial]
async fn test_cancel_meeting_success() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, org_id, building_id) = create_test_fixtures(&app_state, org_id).await;

    let app = test::init_service(
        actix_web::App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create meeting
    let scheduled_date = Utc::now() + Duration::days(30);

    let req = test::TestRequest::post()
        .uri("/api/v1/meetings")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "organization_id": org_id.to_string(),
            "building_id": building_id.to_string(),
            "meeting_type": "Ordinary",
            "title": "Meeting to Cancel",
            "scheduled_date": scheduled_date.to_rfc3339(),
            "location": "Hall"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    let created: serde_json::Value = test::read_body_json(resp).await;
    let meeting_id = created["id"].as_str().unwrap();

    // Cancel meeting
    let req = test::TestRequest::post()
        .uri(&format!("/api/v1/meetings/{}/cancel", meeting_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200, "Should cancel meeting successfully");

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["status"], "Cancelled");
}

//
// TEST: POST /meetings/:id/reschedule (Reschedule meeting)
//

#[actix_web::test]
#[serial]
async fn test_reschedule_meeting_success() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, org_id, building_id) = create_test_fixtures(&app_state, org_id).await;

    let app = test::init_service(
        actix_web::App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create meeting
    let original_date = Utc::now() + Duration::days(30);

    let req = test::TestRequest::post()
        .uri("/api/v1/meetings")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "organization_id": org_id.to_string(),
            "building_id": building_id.to_string(),
            "meeting_type": "Ordinary",
            "title": "Meeting to Reschedule",
            "scheduled_date": original_date.to_rfc3339(),
            "location": "Hall"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    let created: serde_json::Value = test::read_body_json(resp).await;
    let meeting_id = created["id"].as_str().unwrap();

    // Reschedule meeting
    let new_date = Utc::now() + Duration::days(45);

    let req = test::TestRequest::post()
        .uri(&format!("/api/v1/meetings/{}/reschedule", meeting_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "scheduled_date": new_date.to_rfc3339()
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200, "Should reschedule meeting successfully");

    let body: serde_json::Value = test::read_body_json(resp).await;
    // Verify new date is different from original
    assert_ne!(
        body["scheduled_date"],
        original_date.to_rfc3339(),
        "Scheduled date should be updated"
    );
}

//
// TEST: DELETE /meetings/:id (Delete meeting)
//

#[actix_web::test]
#[serial]
async fn test_delete_meeting_success() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, org_id, building_id) = create_test_fixtures(&app_state, org_id).await;

    let app = test::init_service(
        actix_web::App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create meeting
    let scheduled_date = Utc::now() + Duration::days(30);

    let req = test::TestRequest::post()
        .uri("/api/v1/meetings")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "organization_id": org_id.to_string(),
            "building_id": building_id.to_string(),
            "meeting_type": "Ordinary",
            "title": "Meeting to Delete",
            "scheduled_date": scheduled_date.to_rfc3339(),
            "location": "Hall"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    let created: serde_json::Value = test::read_body_json(resp).await;
    let meeting_id = created["id"].as_str().unwrap();

    // Delete meeting
    let req = test::TestRequest::delete()
        .uri(&format!("/api/v1/meetings/{}", meeting_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 204, "Should delete meeting successfully");

    // Verify meeting is deleted
    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/meetings/{}", meeting_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.status(),
        404,
        "Deleted meeting should no longer be found"
    );
}

//
// TEST: Complete meeting lifecycle workflow
//

#[actix_web::test]
#[serial]
async fn test_meeting_complete_lifecycle() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, org_id, building_id) = create_test_fixtures(&app_state, org_id).await;

    let app = test::init_service(
        actix_web::App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let scheduled_date = Utc::now() + Duration::days(30);

    // 1. Create meeting
    let req = test::TestRequest::post()
        .uri("/api/v1/meetings")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "organization_id": org_id.to_string(),
            "building_id": building_id.to_string(),
            "meeting_type": "Ordinary",
            "title": "Lifecycle Test Meeting",
            "description": "Testing complete lifecycle",
            "scheduled_date": scheduled_date.to_rfc3339(),
            "location": "Community Hall"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    let created: serde_json::Value = test::read_body_json(resp).await;
    let meeting_id = created["id"].as_str().unwrap();
    assert_eq!(created["status"], "Scheduled");

    // 2. Add agenda items
    let req = test::TestRequest::post()
        .uri(&format!("/api/v1/meetings/{}/agenda", meeting_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "item": "1. Approbation des comptes"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let req = test::TestRequest::post()
        .uri(&format!("/api/v1/meetings/{}/agenda", meeting_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "item": "2. Vote du budget"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["agenda"].as_array().unwrap().len(), 2);

    // 3. Update meeting details
    let req = test::TestRequest::put()
        .uri(&format!("/api/v1/meetings/{}", meeting_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "title": "Lifecycle Test Meeting - Updated",
            "location": "Updated Hall"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    // 4. Complete meeting
    let req = test::TestRequest::post()
        .uri(&format!("/api/v1/meetings/{}/complete", meeting_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "attendees_count": 35
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let completed: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(completed["status"], "Completed");
    assert_eq!(completed["attendees_count"], 35);
    assert_eq!(
        completed["title"], "Lifecycle Test Meeting - Updated",
        "Title should remain updated"
    );
    assert_eq!(
        completed["location"], "Updated Hall",
        "Location should remain updated"
    );
}
