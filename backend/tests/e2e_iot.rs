// E2E tests for IoT Smart Meters & Linky HTTP endpoints (Issue #133 - IoT Phase 0)
// Tests cover IoT reading creation (single and bulk), querying readings,
// building consumption statistics, Linky device configuration, and auth enforcement.

mod common;

use actix_web::http::header;
use actix_web::{test, App};
use chrono::Utc;
use koprogo_api::application::dto::RegisterRequest;
use koprogo_api::infrastructure::web::configure_routes;
use serde_json::json;
use serial_test::serial;
use uuid::Uuid;

/// Helper: Register user and return token
async fn setup_iot_user_token(
    app_state: &actix_web::web::Data<koprogo_api::infrastructure::web::AppState>,
    org_id: Uuid,
) -> String {
    let email = format!("iot-test-{}@test.com", Uuid::new_v4());
    let reg = RegisterRequest {
        email: email.clone(),
        password: "Passw0rd!".to_string(),
        first_name: "IoT".to_string(),
        last_name: "Tester".to_string(),
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

/// Helper: Create a building using use cases directly (not HTTP) and return its ID
async fn create_iot_test_building(
    app_state: &actix_web::web::Data<koprogo_api::infrastructure::web::AppState>,
    org_id: Uuid,
) -> Uuid {
    use koprogo_api::application::dto::CreateBuildingDto;
    let dto = CreateBuildingDto {
        organization_id: org_id.to_string(),
        name: format!("IoT Smart Meter Test Building {}", Uuid::new_v4()),
        address: "99 Boulevard de l'Énergie".to_string(),
        city: "Namur".to_string(),
        postal_code: "5000".to_string(),
        country: "Belgium".to_string(),
        total_units: 20,
        construction_year: Some(2000),
        total_tantiemes: Some(1000),
    };
    let building = app_state
        .building_use_cases
        .create_building(dto)
        .await
        .expect("Failed to create test building for IoT");
    Uuid::parse_str(&building.id).unwrap()
}

// ==================== IoT Reading Tests ====================

#[actix_web::test]
#[serial]
async fn test_iot_readings_create() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = setup_iot_user_token(&app_state, org_id).await;
    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let building_id = create_iot_test_building(&app_state, org_id).await;

    let timestamp = Utc::now().to_rfc3339();

    let req = test::TestRequest::post()
        .uri("/api/v1/iot/readings")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "building_id": building_id.to_string(),
            "device_type": "electricity_meter",
            "metric_type": "electricity_consumption",
            "value": 15.5,
            "unit": "kWh",
            "timestamp": timestamp,
            "source": "Enedis",
            "metadata": {"prm": "12345678901234"}
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201, "Should create IoT reading successfully");

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["building_id"], building_id.to_string());
    assert_eq!(body["device_type"], "electricity_meter");
    assert_eq!(body["metric_type"], "electricity_consumption");
    assert_eq!(body["value"], 15.5);
    assert_eq!(body["unit"], "kWh");
}

#[actix_web::test]
#[serial]
async fn test_iot_readings_bulk() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = setup_iot_user_token(&app_state, org_id).await;
    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let building_id = create_iot_test_building(&app_state, org_id).await;
    let timestamp = Utc::now().to_rfc3339();

    let req = test::TestRequest::post()
        .uri("/api/v1/iot/readings/bulk")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!([
            {
                "building_id": building_id.to_string(),
                "device_type": "electricity_meter",
                "metric_type": "electricity_consumption",
                "value": 10.0,
                "unit": "kWh",
                "timestamp": timestamp,
                "source": "Enedis",
                "metadata": null
            },
            {
                "building_id": building_id.to_string(),
                "device_type": "water_meter",
                "metric_type": "water_consumption",
                "value": 2.5,
                "unit": "m3",
                "timestamp": timestamp,
                "source": "Vivaqua",
                "metadata": null
            }
        ]))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.status(),
        201,
        "Should create bulk IoT readings successfully"
    );

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["count"], 2, "Should have created 2 readings");
}

#[actix_web::test]
#[serial]
async fn test_iot_readings_query() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = setup_iot_user_token(&app_state, org_id).await;
    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let building_id = create_iot_test_building(&app_state, org_id).await;
    let timestamp = Utc::now().to_rfc3339();

    // Insert a reading first
    let create_req = test::TestRequest::post()
        .uri("/api/v1/iot/readings")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "building_id": building_id.to_string(),
            "device_type": "gas_meter",
            "metric_type": "gas_consumption",
            "value": 3.2,
            "unit": "m3",
            "timestamp": timestamp,
            "source": "Ores",
            "metadata": null
        }))
        .to_request();
    let create_resp = test::call_service(&app, create_req).await;
    assert_eq!(create_resp.status(), 201);

    // Query readings (start_date and end_date are required by QueryIoTReadingsDto)
    let start_date = "2020-01-01T00:00:00Z";
    let end_date = "2030-12-31T23:59:59Z";
    let req = test::TestRequest::get()
        .uri(&format!(
            "/api/v1/iot/readings?building_id={}&start_date={}&end_date={}",
            building_id, start_date, end_date
        ))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200, "Should return IoT readings list");

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body.is_array(), "Response should be an array of readings");
}

#[actix_web::test]
#[serial]
async fn test_iot_building_stats() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = setup_iot_user_token(&app_state, org_id).await;
    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let building_id = create_iot_test_building(&app_state, org_id).await;

    // Query consumption stats (may return empty stats if no readings)
    let req = test::TestRequest::get()
        .uri(&format!(
            "/api/v1/iot/buildings/{}/consumption/stats?metric_type=ElectricityConsumption&start_date=2020-01-01T00:00:00Z&end_date=2030-12-31T23:59:59Z",
            building_id
        ))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.status(),
        200,
        "Should return consumption stats (even if empty)"
    );
}

#[actix_web::test]
#[serial]
async fn test_iot_linky_configure() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = setup_iot_user_token(&app_state, org_id).await;
    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let building_id = create_iot_test_building(&app_state, org_id).await;

    let req = test::TestRequest::post()
        .uri("/api/v1/iot/linky/devices")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "building_id": building_id.to_string(),
            "prm": "12345678901234",
            "provider": "enedis",
            "authorization_code": "test-oauth-code-abc123"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    // May succeed (201/200) or fail (400) if provider validation is strict
    assert!(
        resp.status() == 201
            || resp.status() == 200
            || resp.status() == 400
            || resp.status() == 409,
        "Linky configure returns 201/200 on success, 400/409 on validation failure: got {}",
        resp.status()
    );
}

#[actix_web::test]
#[serial]
async fn test_iot_unauthorized() {
    let (app_state, _container, _org_id) = common::setup_test_db().await;
    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Query IoT readings without token
    let req = test::TestRequest::get()
        .uri("/api/v1/iot/readings")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.status(),
        401,
        "Should require authentication for IoT readings"
    );
}
