// E2E tests for AG Session (Visioconférence) HTTP endpoints (BC15 - Art. 3.87 §1 CC)
// Tests cover the full lifecycle of AG video sessions: create, start, end, cancel,
// record remote join, combined quorum, and delete.

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

/// Helper: Create a building via use cases and return its ID
async fn create_ag_test_building(app_state: &actix_web::web::Data<AppState>, org_id: Uuid) -> Uuid {
    let dto = CreateBuildingDto {
        organization_id: org_id.to_string(),
        name: format!("AG Session Test Building {}", Uuid::new_v4()),
        address: "10 Rue des Sessions".to_string(),
        city: "Brussels".to_string(),
        postal_code: "1000".to_string(),
        country: "Belgium".to_string(),
        total_units: 8,
        construction_year: Some(2010),
        total_tantiemes: Some(1000),
    };
    let b = app_state
        .building_use_cases
        .create_building(dto)
        .await
        .expect("building");
    Uuid::parse_str(&b.id).unwrap()
}

/// Helper: Create a meeting via use cases and return its ID
async fn create_ag_test_meeting(
    app_state: &actix_web::web::Data<AppState>,
    org_id: Uuid,
    building_id: Uuid,
) -> Uuid {
    let req = CreateMeetingRequest {
        organization_id: org_id,
        building_id,
        meeting_type: MeetingType::Ordinary,
        title: format!("AG Visio Test {}", Uuid::new_v4()),
        description: Some("Test AG session".to_string()),
        scheduled_date: Utc::now() + Duration::days(30),
        location: "Visioconférence".to_string(),
        is_second_convocation: false,
    };
    let m = app_state
        .meeting_use_cases
        .create_meeting(req)
        .await
        .expect("meeting");
    m.id
}

/// Helper: Register user and return token
async fn setup_user_token(
    app_state: &actix_web::web::Data<koprogo_api::infrastructure::web::AppState>,
    org_id: Uuid,
) -> String {
    let email = format!("ag-session-{}@test.com", Uuid::new_v4());
    let reg = RegisterRequest {
        email: email.clone(),
        password: "Passw0rd!".to_string(),
        first_name: "AG".to_string(),
        last_name: "SessionTester".to_string(),
        role: "superadmin".to_string(),
        organization_id: Some(org_id),
    };
    app_state
        .auth_use_cases
        .register(reg)
        .await
        .expect("register")
        .token
}

// ==================== AG Session Tests ====================

#[actix_web::test]
#[serial]
async fn test_ag_sessions_create() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = setup_user_token(&app_state, org_id).await;
    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let building_id = create_ag_test_building(&app_state, org_id).await;
    let meeting_id = create_ag_test_meeting(&app_state, org_id, building_id).await;

    let scheduled_start = (Utc::now() + Duration::hours(2)).to_rfc3339();

    let req = test::TestRequest::post()
        .uri(&format!("/api/v1/meetings/{}/ag-session", meeting_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "meeting_id": meeting_id.to_string(),
            "platform": "zoom",
            "video_url": "https://zoom.us/j/123456789",
            "host_url": "https://zoom.us/s/123456789",
            "scheduled_start": scheduled_start,
            "access_password": "test-pw",
            "waiting_room_enabled": true,
            "recording_enabled": false
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201, "Should create AG session successfully");

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["platform"], "zoom");
    assert_eq!(body["status"], "scheduled");
    assert_eq!(body["meeting_id"], meeting_id.to_string());
    assert!(body["waiting_room_enabled"].as_bool().unwrap_or(false));
}

#[actix_web::test]
#[serial]
async fn test_ag_sessions_get_for_meeting() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = setup_user_token(&app_state, org_id).await;
    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let building_id = create_ag_test_building(&app_state, org_id).await;
    let meeting_id = create_ag_test_meeting(&app_state, org_id, building_id).await;
    let scheduled_start = (Utc::now() + Duration::hours(2)).to_rfc3339();

    // Create session first
    let create_req = test::TestRequest::post()
        .uri(&format!("/api/v1/meetings/{}/ag-session", meeting_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "meeting_id": meeting_id.to_string(),
            "platform": "microsoft_teams",
            "video_url": "https://teams.microsoft.com/l/meetup-join/abc",
            "scheduled_start": scheduled_start
        }))
        .to_request();
    let create_resp = test::call_service(&app, create_req).await;
    assert_eq!(create_resp.status(), 201);

    // Now get session for meeting
    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/meetings/{}/ag-session", meeting_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200, "Should get AG session for meeting");

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["meeting_id"], meeting_id.to_string());
    assert_eq!(body["platform"], "microsoft_teams");
}

#[actix_web::test]
#[serial]
async fn test_ag_sessions_start() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = setup_user_token(&app_state, org_id).await;
    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let building_id = create_ag_test_building(&app_state, org_id).await;
    let meeting_id = create_ag_test_meeting(&app_state, org_id, building_id).await;
    let scheduled_start = (Utc::now() + Duration::hours(2)).to_rfc3339();

    // Create session
    let create_req = test::TestRequest::post()
        .uri(&format!("/api/v1/meetings/{}/ag-session", meeting_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "meeting_id": meeting_id.to_string(),
            "platform": "google_meet",
            "video_url": "https://meet.google.com/abc-defg-hij",
            "scheduled_start": scheduled_start
        }))
        .to_request();
    let create_resp = test::call_service(&app, create_req).await;
    assert_eq!(create_resp.status(), 201);
    let session: serde_json::Value = test::read_body_json(create_resp).await;
    let session_id = session["id"].as_str().unwrap().to_string();

    // Start session
    let req = test::TestRequest::put()
        .uri(&format!("/api/v1/ag-sessions/{}/start", session_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200, "Should start AG session");

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["status"], "live");
    assert!(body["actual_start"].is_string());
}

#[actix_web::test]
#[serial]
async fn test_ag_sessions_end() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = setup_user_token(&app_state, org_id).await;
    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let building_id = create_ag_test_building(&app_state, org_id).await;
    let meeting_id = create_ag_test_meeting(&app_state, org_id, building_id).await;
    let scheduled_start = (Utc::now() + Duration::hours(2)).to_rfc3339();

    // Create session
    let create_req = test::TestRequest::post()
        .uri(&format!("/api/v1/meetings/{}/ag-session", meeting_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "meeting_id": meeting_id.to_string(),
            "platform": "jitsi",
            "video_url": "https://meet.jit.si/KoproGoAG",
            "scheduled_start": scheduled_start,
            "recording_enabled": true
        }))
        .to_request();
    let create_resp = test::call_service(&app, create_req).await;
    assert_eq!(create_resp.status(), 201);
    let session: serde_json::Value = test::read_body_json(create_resp).await;
    let session_id = session["id"].as_str().unwrap().to_string();

    // Start session first
    let start_req = test::TestRequest::put()
        .uri(&format!("/api/v1/ag-sessions/{}/start", session_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();
    let start_resp = test::call_service(&app, start_req).await;
    assert_eq!(start_resp.status(), 200);

    // End session
    let req = test::TestRequest::put()
        .uri(&format!("/api/v1/ag-sessions/{}/end", session_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "recording_url": "https://recordings.jit.si/KoproGoAG/recording.mp4"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200, "Should end AG session");

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["status"], "ended");
    assert!(body["actual_end"].is_string());
}

#[actix_web::test]
#[serial]
async fn test_ag_sessions_cancel() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = setup_user_token(&app_state, org_id).await;
    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let building_id = create_ag_test_building(&app_state, org_id).await;
    let meeting_id = create_ag_test_meeting(&app_state, org_id, building_id).await;
    let scheduled_start = (Utc::now() + Duration::hours(2)).to_rfc3339();

    // Create session
    let create_req = test::TestRequest::post()
        .uri(&format!("/api/v1/meetings/{}/ag-session", meeting_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "meeting_id": meeting_id.to_string(),
            "platform": "whereby",
            "video_url": "https://whereby.com/koprogo-ag",
            "scheduled_start": scheduled_start
        }))
        .to_request();
    let create_resp = test::call_service(&app, create_req).await;
    assert_eq!(create_resp.status(), 201);
    let session: serde_json::Value = test::read_body_json(create_resp).await;
    let session_id = session["id"].as_str().unwrap().to_string();

    // Cancel session
    let req = test::TestRequest::put()
        .uri(&format!("/api/v1/ag-sessions/{}/cancel", session_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200, "Should cancel AG session");

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["status"], "cancelled");
}

#[actix_web::test]
#[serial]
async fn test_ag_sessions_record_remote_join() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = setup_user_token(&app_state, org_id).await;
    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let building_id = create_ag_test_building(&app_state, org_id).await;
    let meeting_id = create_ag_test_meeting(&app_state, org_id, building_id).await;
    let scheduled_start = (Utc::now() + Duration::hours(2)).to_rfc3339();

    // Create session
    let create_req = test::TestRequest::post()
        .uri(&format!("/api/v1/meetings/{}/ag-session", meeting_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "meeting_id": meeting_id.to_string(),
            "platform": "zoom",
            "video_url": "https://zoom.us/j/987654321",
            "scheduled_start": scheduled_start
        }))
        .to_request();
    let create_resp = test::call_service(&app, create_req).await;
    assert_eq!(create_resp.status(), 201);
    let session: serde_json::Value = test::read_body_json(create_resp).await;
    let session_id = session["id"].as_str().unwrap().to_string();

    // Start session
    let start_req = test::TestRequest::put()
        .uri(&format!("/api/v1/ag-sessions/{}/start", session_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();
    let start_resp = test::call_service(&app, start_req).await;
    assert_eq!(start_resp.status(), 200);

    // Record remote join
    let req = test::TestRequest::post()
        .uri(&format!("/api/v1/ag-sessions/{}/join", session_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "voting_power": 150.0,
            "total_building_quotas": 1000.0
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.status(),
        200,
        "Should record remote participant joining"
    );

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body["remote_attendees_count"].as_i64().unwrap_or(0) >= 1);
}

#[actix_web::test]
#[serial]
async fn test_ag_sessions_combined_quorum() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = setup_user_token(&app_state, org_id).await;
    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let building_id = create_ag_test_building(&app_state, org_id).await;
    let meeting_id = create_ag_test_meeting(&app_state, org_id, building_id).await;
    let scheduled_start = (Utc::now() + Duration::hours(2)).to_rfc3339();

    // Create session
    let create_req = test::TestRequest::post()
        .uri(&format!("/api/v1/meetings/{}/ag-session", meeting_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "meeting_id": meeting_id.to_string(),
            "platform": "zoom",
            "video_url": "https://zoom.us/j/111222333",
            "scheduled_start": scheduled_start
        }))
        .to_request();
    let create_resp = test::call_service(&app, create_req).await;
    assert_eq!(create_resp.status(), 201);
    let session: serde_json::Value = test::read_body_json(create_resp).await;
    let session_id = session["id"].as_str().unwrap().to_string();

    // Query combined quorum
    let req = test::TestRequest::get()
        .uri(&format!(
            "/api/v1/ag-sessions/{}/quorum?physical_quotas=600.0&total_building_quotas=1000.0",
            session_id
        ))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200, "Should return combined quorum data");

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body["combined_percentage"].is_number());
    assert!(body["quorum_reached"].is_boolean());
    assert_eq!(body["session_id"], session_id);
}

#[actix_web::test]
#[serial]
async fn test_ag_sessions_delete() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = setup_user_token(&app_state, org_id).await;
    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let building_id = create_ag_test_building(&app_state, org_id).await;
    let meeting_id = create_ag_test_meeting(&app_state, org_id, building_id).await;
    let scheduled_start = (Utc::now() + Duration::hours(2)).to_rfc3339();

    // Create session
    let create_req = test::TestRequest::post()
        .uri(&format!("/api/v1/meetings/{}/ag-session", meeting_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "meeting_id": meeting_id.to_string(),
            "platform": "zoom",
            "video_url": "https://zoom.us/j/delete-test",
            "scheduled_start": scheduled_start
        }))
        .to_request();
    let create_resp = test::call_service(&app, create_req).await;
    assert_eq!(create_resp.status(), 201);
    let session: serde_json::Value = test::read_body_json(create_resp).await;
    let session_id = session["id"].as_str().unwrap().to_string();

    // Delete session
    let req = test::TestRequest::delete()
        .uri(&format!("/api/v1/ag-sessions/{}", session_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(
        resp.status() == 204 || resp.status() == 200,
        "Should delete AG session (204 or 200)"
    );
}
