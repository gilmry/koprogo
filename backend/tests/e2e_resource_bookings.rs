// E2E tests for Resource Booking Calendar HTTP endpoints (Issue #49 - Phase 5)
// Tests focus on HTTP layer: endpoints, auth, JSON serialization
// Covers booking lifecycle: create, get, list, cancel, overlap detection

mod common;

use actix_web::http::header;
use actix_web::{test, App};
use koprogo_api::application::dto::RegisterRequest;
use koprogo_api::infrastructure::web::configure_routes;
use serde_json::json;
use serial_test::serial;
use uuid::Uuid;

/// Helper: Register a user, create a linked owner, return (token, user_id)
async fn setup_booking_user_with_owner(
    app_state: &actix_web::web::Data<koprogo_api::infrastructure::web::AppState>,
    org_id: Uuid,
    label: &str,
) -> (String, Uuid) {
    let email = format!("booking-{}-{}@example.com", label, Uuid::new_v4());
    let reg = RegisterRequest {
        email: email.clone(),
        password: "SecurePass123!".to_string(),
        first_name: "Booking".to_string(),
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

    let owner_id = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO owners (id, organization_id, user_id, first_name, last_name, email,
           address, city, postal_code, country, created_at, updated_at)
           VALUES ($1, $2, $3, 'Booking', $4, $5, '4 Rue Test', 'Brussels', '1000', 'BE', NOW(), NOW())"#,
    )
    .bind(owner_id)
    .bind(org_id)
    .bind(user_id)
    .bind(label)
    .bind(format!("owner-booking-{}-{}@test.com", label, Uuid::new_v4()))
    .execute(&app_state.pool)
    .await
    .expect("Failed to insert owner");

    (token, user_id)
}

/// Helper: Create a building using use cases directly (not HTTP) and return its id
async fn create_building_for_bookings(
    app_state: &actix_web::web::Data<koprogo_api::infrastructure::web::AppState>,
    org_id: Uuid,
    name: &str,
) -> String {
    use koprogo_api::application::dto::CreateBuildingDto;
    let dto = CreateBuildingDto {
        organization_id: org_id.to_string(),
        name: format!("{} {}", name, uuid::Uuid::new_v4()),
        address: "40 Booking Street".to_string(),
        city: "Brussels".to_string(),
        postal_code: "1000".to_string(),
        country: "BE".to_string(),
        total_units: 10,
        total_tantiemes: Some(1000),
        construction_year: Some(2020),
    };
    let building = app_state
        .building_use_cases
        .create_building(dto)
        .await
        .expect("Failed to create test building for resource bookings");
    building.id
}

// ==================== Resource Booking Tests ====================

#[actix_web::test]
#[serial]
async fn test_resource_bookings_create() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, _user_id) = setup_booking_user_with_owner(&app_state, org_id, "create").await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let building_id =
        create_building_for_bookings(&app_state, org_id, "Resource Booking Create Building").await;

    // Use a future time slot to avoid validation errors
    let start_time = "2027-06-01T10:00:00Z";
    let end_time = "2027-06-01T12:00:00Z";

    let req = test::TestRequest::post()
        .uri("/api/v1/resource-bookings")
        .insert_header(header::ContentType::json())
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "building_id": building_id,
            "resource_type": "MeetingRoom",
            "resource_name": "Meeting Room A",
            "start_time": start_time,
            "end_time": end_time,
            "notes": "Residents association meeting",
            "max_advance_days": 1000
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.status(),
        201,
        "Should create resource booking successfully"
    );

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["resource_type"], "MeetingRoom");
    assert_eq!(body["resource_name"], "Meeting Room A");
    assert!(
        body["status"] == "Confirmed" || body["status"] == "Pending",
        "Status should be Confirmed or Pending, got: {}",
        body["status"]
    );
}

#[actix_web::test]
#[serial]
async fn test_resource_bookings_get() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, _user_id) = setup_booking_user_with_owner(&app_state, org_id, "getbooker").await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let building_id =
        create_building_for_bookings(&app_state, org_id, "Resource Booking Get Building").await;

    // Create booking
    let create_req = test::TestRequest::post()
        .uri("/api/v1/resource-bookings")
        .insert_header(header::ContentType::json())
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "building_id": building_id,
            "resource_type": "LaundryRoom",
            "resource_name": "Laundry Room 1",
            "start_time": "2027-07-01T08:00:00Z",
            "end_time": "2027-07-01T10:00:00Z",
            "max_advance_days": 1000
        }))
        .to_request();
    let create_resp = test::call_service(&app, create_req).await;
    assert_eq!(create_resp.status(), 201);
    let create_body: serde_json::Value = test::read_body_json(create_resp).await;
    let booking_id = create_body["id"].as_str().unwrap();

    // Get booking by ID
    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/resource-bookings/{}", booking_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200, "Should retrieve booking by ID");

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["id"], booking_id);
    assert_eq!(body["resource_type"], "LaundryRoom");
    assert_eq!(body["resource_name"], "Laundry Room 1");
}

#[actix_web::test]
#[serial]
async fn test_resource_bookings_list() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, _user_id) = setup_booking_user_with_owner(&app_state, org_id, "listbooker").await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let building_id =
        create_building_for_bookings(&app_state, org_id, "Resource Booking List Building").await;

    // Create two bookings with non-overlapping times
    let bookings = vec![
        ("2027-08-01T09:00:00Z", "2027-08-01T11:00:00Z", "Gym"),
        ("2027-08-02T09:00:00Z", "2027-08-02T11:00:00Z", "Gym"),
    ];
    for (start, end, resource_name) in &bookings {
        let req = test::TestRequest::post()
            .uri("/api/v1/resource-bookings")
            .insert_header(header::ContentType::json())
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
            .set_json(json!({
                "building_id": building_id,
                "resource_type": "Gym",
                "resource_name": resource_name,
                "start_time": start,
                "end_time": end,
                "max_advance_days": 1000
            }))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 201, "Booking creation must succeed");
    }

    // List bookings for the building
    let list_req = test::TestRequest::get()
        .uri(&format!(
            "/api/v1/buildings/{}/resource-bookings",
            building_id
        ))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();
    let list_resp = test::call_service(&app, list_req).await;
    assert_eq!(list_resp.status(), 200, "Should list building bookings");

    let result: serde_json::Value = test::read_body_json(list_resp).await;
    assert!(result.is_array(), "Response should be an array");
    assert_eq!(
        result.as_array().unwrap().len(),
        2,
        "Should have 2 bookings"
    );
}

#[actix_web::test]
#[serial]
async fn test_resource_bookings_cancel() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, _user_id) = setup_booking_user_with_owner(&app_state, org_id, "cancelbooker").await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let building_id =
        create_building_for_bookings(&app_state, org_id, "Resource Booking Cancel Building").await;

    // Create booking
    let create_req = test::TestRequest::post()
        .uri("/api/v1/resource-bookings")
        .insert_header(header::ContentType::json())
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "building_id": building_id,
            "resource_type": "MeetingRoom",
            "resource_name": "Meeting Room B",
            "start_time": "2027-09-01T14:00:00Z",
            "end_time": "2027-09-01T16:00:00Z",
            "notes": "Booking to cancel",
            "max_advance_days": 1000
        }))
        .to_request();
    let create_resp = test::call_service(&app, create_req).await;
    assert_eq!(create_resp.status(), 201);
    let create_body: serde_json::Value = test::read_body_json(create_resp).await;
    let booking_id = create_body["id"].as_str().unwrap();

    // Cancel booking (POST /resource-bookings/:id/cancel per handler)
    let cancel_req = test::TestRequest::post()
        .uri(&format!("/api/v1/resource-bookings/{}/cancel", booking_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();
    let cancel_resp = test::call_service(&app, cancel_req).await;
    assert_eq!(
        cancel_resp.status(),
        200,
        "Should cancel booking successfully"
    );

    let body: serde_json::Value = test::read_body_json(cancel_resp).await;
    assert_eq!(
        body["status"], "Cancelled",
        "Booking status should be Cancelled"
    );
}

#[actix_web::test]
#[serial]
async fn test_resource_bookings_overlap_fails() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, _user_id) =
        setup_booking_user_with_owner(&app_state, org_id, "overlapbooker").await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let building_id =
        create_building_for_bookings(&app_state, org_id, "Resource Booking Overlap Building").await;

    // Create first booking
    let first_req = test::TestRequest::post()
        .uri("/api/v1/resource-bookings")
        .insert_header(header::ContentType::json())
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "building_id": building_id,
            "resource_type": "MeetingRoom",
            "resource_name": "Meeting Room C",
            "start_time": "2027-10-01T10:00:00Z",
            "end_time": "2027-10-01T12:00:00Z",
            "notes": "First booking",
            "max_advance_days": 1000
        }))
        .to_request();
    let first_resp = test::call_service(&app, first_req).await;
    assert_eq!(first_resp.status(), 201, "First booking must succeed");

    // Create second booking that overlaps with the first
    let overlap_req = test::TestRequest::post()
        .uri("/api/v1/resource-bookings")
        .insert_header(header::ContentType::json())
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "building_id": building_id,
            "resource_type": "MeetingRoom",
            "resource_name": "Meeting Room C",
            "start_time": "2027-10-01T11:00:00Z",
            "end_time": "2027-10-01T13:00:00Z",
            "notes": "Overlapping booking - should fail",
            "max_advance_days": 1000
        }))
        .to_request();
    let overlap_resp = test::call_service(&app, overlap_req).await;
    let status = overlap_resp.status().as_u16();
    assert!(
        status == 400 || status == 409,
        "Should reject overlapping booking (got {})",
        status
    );
}
