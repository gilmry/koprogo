// E2E tests for Energy Campaign HTTP endpoints (Issue #49 - Energy Buying Groups)
// Tests focus on HTTP layer: endpoints, auth, JSON serialization

mod common;

use actix_web::http::header;
use actix_web::{test, App};
use koprogo_api::infrastructure::web::configure_routes;
use serde_json::json;
use serial_test::serial;

/// Helper: create a building for energy campaign tests
async fn create_building(
    app_state: &actix_web::web::Data<koprogo_api::infrastructure::web::AppState>,
    org_id: uuid::Uuid,
) -> String {
    use koprogo_api::application::dto::CreateBuildingDto;
    let dto = CreateBuildingDto {
        organization_id: org_id.to_string(),
        name: format!("Energy Building {}", uuid::Uuid::new_v4()),
        address: "1 Rue Energie".to_string(),
        city: "Brussels".to_string(),
        postal_code: "1000".to_string(),
        country: "BE".to_string(),
        total_units: 10,
        total_tantiemes: Some(1000),
        construction_year: Some(2010),
    };
    app_state
        .building_use_cases
        .create_building(dto)
        .await
        .expect("Failed to create building")
        .id
}

#[actix_web::test]
#[serial]
async fn test_energy_campaigns_create() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = common::register_and_login(&app_state, org_id).await;
    let building_id = create_building(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/v1/energy-campaigns")
        .insert_header(header::ContentType::json())
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "building_id": building_id,
            "campaign_name": "Winter 2027 Energy Group Buy",
            "deadline_participation": "2027-10-31T23:59:59Z",
            "energy_types": ["Electricity"]
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201, "Should create energy campaign");

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["campaign_name"], "Winter 2027 Energy Group Buy");
    assert!(body["id"].as_str().is_some());
    assert_eq!(body["status"], "Draft");
}

#[actix_web::test]
#[serial]
async fn test_energy_campaigns_list() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = common::register_and_login(&app_state, org_id).await;
    let building_id = create_building(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create a campaign
    let create_req = test::TestRequest::post()
        .uri("/api/v1/energy-campaigns")
        .insert_header(header::ContentType::json())
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "building_id": building_id,
            "campaign_name": "Spring 2027 Gas Group Buy",
            "deadline_participation": "2027-03-31T23:59:59Z",
            "energy_types": ["Gas"]
        }))
        .to_request();
    let create_resp = test::call_service(&app, create_req).await;
    assert_eq!(create_resp.status(), 201);

    // List campaigns
    let list_req = test::TestRequest::get()
        .uri("/api/v1/energy-campaigns")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let list_resp = test::call_service(&app, list_req).await;
    assert_eq!(list_resp.status(), 200, "Should list campaigns");

    let body: serde_json::Value = test::read_body_json(list_resp).await;
    assert!(body.is_array(), "Response should be an array");
    assert_eq!(body.as_array().unwrap().len(), 1, "Should have 1 campaign");
}

#[actix_web::test]
#[serial]
async fn test_energy_campaigns_get() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = common::register_and_login(&app_state, org_id).await;
    let building_id = create_building(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create
    let create_req = test::TestRequest::post()
        .uri("/api/v1/energy-campaigns")
        .insert_header(header::ContentType::json())
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "building_id": building_id,
            "campaign_name": "Summer 2027 Both Energy",
            "deadline_participation": "2027-06-30T23:59:59Z",
            "energy_types": ["Both"]
        }))
        .to_request();
    let create_resp = test::call_service(&app, create_req).await;
    assert_eq!(create_resp.status(), 201);
    let create_body: serde_json::Value = test::read_body_json(create_resp).await;
    let campaign_id = create_body["id"].as_str().unwrap();

    // Get by ID
    let get_req = test::TestRequest::get()
        .uri(&format!("/api/v1/energy-campaigns/{}", campaign_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let get_resp = test::call_service(&app, get_req).await;
    assert_eq!(get_resp.status(), 200, "Should retrieve campaign by ID");

    let body: serde_json::Value = test::read_body_json(get_resp).await;
    assert_eq!(body["id"], campaign_id);
    assert_eq!(body["campaign_name"], "Summer 2027 Both Energy");
}

#[actix_web::test]
#[serial]
async fn test_energy_campaigns_unauthorized() {
    let (app_state, _container, _org_id) = common::setup_test_db().await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/v1/energy-campaigns")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401, "Should require authentication");
}

#[actix_web::test]
#[serial]
async fn test_energy_campaigns_update_status() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = common::register_and_login(&app_state, org_id).await;
    let building_id = create_building(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create
    let create_req = test::TestRequest::post()
        .uri("/api/v1/energy-campaigns")
        .insert_header(header::ContentType::json())
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "building_id": building_id,
            "campaign_name": "Autumn 2027 Campaign",
            "deadline_participation": "2027-09-30T23:59:59Z",
            "energy_types": ["Electricity"]
        }))
        .to_request();
    let create_resp = test::call_service(&app, create_req).await;
    assert_eq!(create_resp.status(), 201);
    let create_body: serde_json::Value = test::read_body_json(create_resp).await;
    let campaign_id = create_body["id"].as_str().unwrap();

    // Update status to AwaitingAGVote
    let update_req = test::TestRequest::put()
        .uri(&format!("/api/v1/energy-campaigns/{}/status", campaign_id))
        .insert_header(header::ContentType::json())
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "status": "AwaitingAGVote"
        }))
        .to_request();

    let update_resp = test::call_service(&app, update_req).await;
    assert_eq!(update_resp.status(), 200, "Should update campaign status");

    let body: serde_json::Value = test::read_body_json(update_resp).await;
    assert_eq!(body["status"], "AwaitingAGVote");
}
