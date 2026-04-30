// E2E tests for second convocation and meeting minutes (Issues #311, #313)
// Tests focus on HTTP layer: second convocation scheduling, quorum skip, minutes attachment
// Belgian law: Art. 3.87 §5 CC — second convocation has no quorum requirement

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

/// Setup function shared across second convocation E2E tests
async fn setup_app() -> (
    actix_web::web::Data<AppState>,
    Option<
        testcontainers_modules::testcontainers::ContainerAsync<
            testcontainers_modules::postgres::Postgres,
        >,
    >,
    Uuid,
) {
    common::setup_test_db().await
}

/// Helper: Create a building
async fn create_test_building(app_state: &actix_web::web::Data<AppState>, org_id: Uuid) -> Uuid {
    let building_dto = CreateBuildingDto {
        organization_id: org_id.to_string(),
        name: format!("Test Building SecondConv {}", Uuid::new_v4()),
        address: "42 Rue du Quorum".to_string(),
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

    Uuid::parse_str(&building.id).expect("Failed to parse building id")
}

/// Helper: Create a meeting
async fn create_test_meeting(
    app_state: &actix_web::web::Data<AppState>,
    org_id: Uuid,
    building_id: Uuid,
    days_from_now: i64,
) -> Uuid {
    let meeting_req = CreateMeetingRequest {
        organization_id: org_id,
        building_id,
        meeting_type: MeetingType::Ordinary,
        title: format!("AG Test Meeting {}", Uuid::new_v4()),
        description: Some("Annual general assembly".to_string()),
        scheduled_date: Utc::now() + Duration::days(days_from_now),
        location: "Building Main Hall".to_string(),
        is_second_convocation: true,
    };

    let meeting = app_state
        .meeting_use_cases
        .create_meeting(meeting_req)
        .await
        .expect("Failed to create meeting");

    meeting.id
}

// ==================== Second Convocation Tests ====================

#[actix_web::test]
#[serial]
async fn test_schedule_second_convocation_success() {
    let (app_state, _container, org_id) = setup_app().await;
    let token = common::register_and_login(&app_state, org_id).await;
    let building_id = create_test_building(&app_state, org_id).await;

    // Create first meeting (30 days from now, well in the past for second conv scheduling)
    let first_meeting_id = create_test_meeting(&app_state, org_id, building_id, 30).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Schedule second convocation (must be >= 15 days after first meeting)
    let new_meeting_date = Utc::now() + Duration::days(60);
    let req = test::TestRequest::post()
        .uri("/api/v1/convocations/second")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "building_id": building_id.to_string(),
            "first_meeting_id": first_meeting_id.to_string(),
            "new_meeting_date": new_meeting_date.to_rfc3339(),
            "language": "FR"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.status(),
        201,
        "Should create second convocation successfully"
    );

    let convocation: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(convocation["meeting_type"], "SecondConvocation");
    assert_eq!(convocation["no_quorum_required"], true);
    assert_eq!(
        convocation["first_meeting_id"],
        first_meeting_id.to_string()
    );
    assert_eq!(convocation["language"], "FR");
    assert_eq!(convocation["status"], "Draft");
}

#[actix_web::test]
#[serial]
async fn test_second_convocation_has_no_quorum_required() {
    let (app_state, _container, org_id) = setup_app().await;
    let token = common::register_and_login(&app_state, org_id).await;
    let building_id = create_test_building(&app_state, org_id).await;

    let first_meeting_id = create_test_meeting(&app_state, org_id, building_id, 30).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let new_meeting_date = Utc::now() + Duration::days(60);
    let req = test::TestRequest::post()
        .uri("/api/v1/convocations/second")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "building_id": building_id.to_string(),
            "first_meeting_id": first_meeting_id.to_string(),
            "new_meeting_date": new_meeting_date.to_rfc3339(),
            "language": "NL"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    let convocation: serde_json::Value = test::read_body_json(resp).await;

    // Art. 3.87 §5 CC: no quorum required for second convocation
    assert_eq!(
        convocation["no_quorum_required"], true,
        "Second convocation must have no_quorum_required = true (Art. 3.87 §5 CC)"
    );

    // Verify it references the first meeting
    assert_eq!(
        convocation["first_meeting_id"],
        first_meeting_id.to_string(),
        "Should reference the first meeting"
    );

    // Language should be NL
    assert_eq!(convocation["language"], "NL");
}

#[actix_web::test]
#[serial]
async fn test_second_convocation_without_auth_fails() {
    let (app_state, _container, org_id) = setup_app().await;
    let _token = common::register_and_login(&app_state, org_id).await;
    let building_id = create_test_building(&app_state, org_id).await;
    let first_meeting_id = create_test_meeting(&app_state, org_id, building_id, 30).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let new_meeting_date = Utc::now() + Duration::days(60);
    let req = test::TestRequest::post()
        .uri("/api/v1/convocations/second")
        .set_json(json!({
            "building_id": building_id.to_string(),
            "first_meeting_id": first_meeting_id.to_string(),
            "new_meeting_date": new_meeting_date.to_rfc3339(),
            "language": "FR"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401, "Should require authentication");
}

// ==================== Meeting Minutes Tests ====================

#[actix_web::test]
#[serial]
async fn test_attach_minutes_to_completed_meeting() {
    let (app_state, _container, org_id) = setup_app().await;
    let token = common::register_and_login(&app_state, org_id).await;
    let building_id = create_test_building(&app_state, org_id).await;
    let meeting_id = create_test_meeting(&app_state, org_id, building_id, 7).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // 1. Complete the meeting first
    let complete_req = test::TestRequest::post()
        .uri(&format!("/api/v1/meetings/{}/complete", meeting_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({ "attendees_count": 15 }))
        .to_request();

    let complete_resp = test::call_service(&app, complete_req).await;
    assert_eq!(complete_resp.status(), 200, "Should complete meeting");

    // 2. Attach minutes document
    let document_id = Uuid::new_v4();
    let attach_req = test::TestRequest::post()
        .uri(&format!("/api/v1/meetings/{}/attach-minutes", meeting_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({ "document_id": document_id.to_string() }))
        .to_request();

    let attach_resp = test::call_service(&app, attach_req).await;
    assert_eq!(
        attach_resp.status(),
        200,
        "Should attach minutes to completed meeting"
    );

    let meeting: serde_json::Value = test::read_body_json(attach_resp).await;
    assert_eq!(
        meeting["minutes_document_id"],
        document_id.to_string(),
        "minutes_document_id should be set"
    );
    assert!(
        meeting["minutes_sent_at"].as_str().is_some(),
        "minutes_sent_at should be set"
    );
}

#[actix_web::test]
#[serial]
async fn test_attach_minutes_to_non_completed_meeting_fails() {
    let (app_state, _container, org_id) = setup_app().await;
    let token = common::register_and_login(&app_state, org_id).await;
    let building_id = create_test_building(&app_state, org_id).await;

    // Meeting is in Scheduled status (not completed)
    let meeting_id = create_test_meeting(&app_state, org_id, building_id, 7).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Try to attach minutes to a non-completed meeting
    let document_id = Uuid::new_v4();
    let req = test::TestRequest::post()
        .uri(&format!("/api/v1/meetings/{}/attach-minutes", meeting_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({ "document_id": document_id.to_string() }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.status(),
        400,
        "Should reject minutes attachment for non-completed meeting"
    );

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(
        body["error"].as_str().unwrap().contains("completed"),
        "Error should mention that meeting must be completed"
    );
}

#[actix_web::test]
#[serial]
async fn test_attach_minutes_without_auth_fails() {
    let (app_state, _container, org_id) = setup_app().await;
    let _token = common::register_and_login(&app_state, org_id).await;
    let building_id = create_test_building(&app_state, org_id).await;
    let meeting_id = create_test_meeting(&app_state, org_id, building_id, 7).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let document_id = Uuid::new_v4();
    let req = test::TestRequest::post()
        .uri(&format!("/api/v1/meetings/{}/attach-minutes", meeting_id))
        .set_json(json!({ "document_id": document_id.to_string() }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401, "Should require authentication");
}
