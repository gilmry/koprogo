// E2E tests for Technical Inspection HTTP endpoints (Issue #134 - Digital Maintenance Logbook)
// Tests cover creation, retrieval, listing by building, upcoming and overdue queries,
// and deletion of mandatory technical inspections.

mod common;

use actix_web::http::header;
use actix_web::{test, App};
use koprogo_api::application::dto::RegisterRequest;
use koprogo_api::infrastructure::web::configure_routes;
use serde_json::json;
use serial_test::serial;
use uuid::Uuid;

/// Helper: Register user and return token
async fn setup_inspection_user_token(
    app_state: &actix_web::web::Data<koprogo_api::infrastructure::web::AppState>,
    org_id: Uuid,
) -> String {
    let email = format!("tech-inspect-{}@test.com", Uuid::new_v4());
    let reg = RegisterRequest {
        email: email.clone(),
        password: "Passw0rd!".to_string(),
        first_name: "TechInspect".to_string(),
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
async fn create_inspection_test_building(
    app_state: &actix_web::web::Data<koprogo_api::infrastructure::web::AppState>,
    org_id: Uuid,
) -> Uuid {
    use koprogo_api::application::dto::CreateBuildingDto;
    let dto = CreateBuildingDto {
        organization_id: org_id.to_string(),
        name: format!("Technical Inspection Test Building {}", Uuid::new_v4()),
        address: "15 Rue de l'Inspection".to_string(),
        city: "Charleroi".to_string(),
        postal_code: "6000".to_string(),
        country: "Belgium".to_string(),
        total_units: 10,
        construction_year: Some(1970),
        total_tantiemes: Some(1000),
    };
    let building = app_state
        .building_use_cases
        .create_building(dto)
        .await
        .expect("Failed to create test building for technical inspections");
    Uuid::parse_str(&building.id).unwrap()
}

// ==================== Technical Inspection Tests ====================

#[actix_web::test]
#[serial]
async fn test_technical_inspections_create() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = setup_inspection_user_token(&app_state, org_id).await;
    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let building_id = create_inspection_test_building(&app_state, org_id).await;

    let req = test::TestRequest::post()
        .uri("/api/v1/technical-inspections")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "organization_id": org_id.to_string(),
            "building_id": building_id.to_string(),
            "title": "Contrôle annuel ascenseur",
            "description": "Inspection obligatoire annuelle de l'ascenseur (RGIE Art. 271).",
            "inspection_type": "elevator",
            "inspector_name": "ACEG Contrôles Techniques",
            "inspector_company": "ACEG SPRL",
            "inspector_certification": "BE-ELEV-2024",
            "inspection_date": "2026-03-10T00:00:00Z",
            "compliant": true,
            "result_summary": "Conforme — aucun défaut détecté.",
            "cost": 350.0
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.status(),
        201,
        "Should create technical inspection successfully"
    );

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["title"], "Contrôle annuel ascenseur");
    assert_eq!(body["inspection_type"], "elevator");
    assert_eq!(body["inspector_name"], "ACEG Contrôles Techniques");
}

#[actix_web::test]
#[serial]
async fn test_technical_inspections_get() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = setup_inspection_user_token(&app_state, org_id).await;
    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let building_id = create_inspection_test_building(&app_state, org_id).await;

    // Create inspection
    let create_req = test::TestRequest::post()
        .uri("/api/v1/technical-inspections")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "organization_id": org_id.to_string(),
            "building_id": building_id.to_string(),
            "title": "Contrôle chaudière",
            "description": "Inspection annuelle chaudière collective.",
            "inspection_type": "boiler",
            "inspector_name": "Technicien Gaz SA",
            "inspection_date": "2026-02-20T00:00:00Z",
            "compliant": true
        }))
        .to_request();
    let create_resp = test::call_service(&app, create_req).await;
    assert_eq!(create_resp.status(), 201);
    let created: serde_json::Value = test::read_body_json(create_resp).await;
    let inspection_id = created["id"].as_str().unwrap().to_string();

    // Get by ID
    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/technical-inspections/{}", inspection_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200, "Should get technical inspection by ID");

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["id"], inspection_id);
    assert_eq!(body["title"], "Contrôle chaudière");
    assert_eq!(body["inspection_type"], "boiler");
}

#[actix_web::test]
#[serial]
async fn test_technical_inspections_list_by_building() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = setup_inspection_user_token(&app_state, org_id).await;
    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let building_id = create_inspection_test_building(&app_state, org_id).await;

    // Create two inspections
    let types = [("elevator", "Ascenseur"), ("boiler", "Chaudière")];
    for (inspection_type, label) in types {
        let create_req = test::TestRequest::post()
            .uri("/api/v1/technical-inspections")
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
            .set_json(json!({
                "organization_id": org_id.to_string(),
                "building_id": building_id.to_string(),
                "title": format!("Contrôle {}", label),
                "description": null,
                "inspection_type": inspection_type,
                "inspector_name": "Inspecteur Générique",
                "inspection_date": "2026-01-15T00:00:00Z"
            }))
            .to_request();
        let create_resp = test::call_service(&app, create_req).await;
        assert_eq!(create_resp.status(), 201);
    }

    // List by building
    let req = test::TestRequest::get()
        .uri(&format!(
            "/api/v1/buildings/{}/technical-inspections",
            building_id
        ))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.status(),
        200,
        "Should list technical inspections for building"
    );

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body.is_array());
    assert_eq!(
        body.as_array().unwrap().len(),
        2,
        "Should return 2 technical inspections"
    );
}

#[actix_web::test]
#[serial]
async fn test_technical_inspections_upcoming() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = setup_inspection_user_token(&app_state, org_id).await;
    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let building_id = create_inspection_test_building(&app_state, org_id).await;

    // Query upcoming inspections (within 90 days)
    let req = test::TestRequest::get()
        .uri(&format!(
            "/api/v1/buildings/{}/technical-inspections/upcoming?days=90",
            building_id
        ))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.status(),
        200,
        "Should return upcoming inspections list"
    );

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(
        body.is_array(),
        "Upcoming inspections should be an array (may be empty)"
    );
}

#[actix_web::test]
#[serial]
async fn test_technical_inspections_overdue() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = setup_inspection_user_token(&app_state, org_id).await;
    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let building_id = create_inspection_test_building(&app_state, org_id).await;

    // Query overdue inspections
    let req = test::TestRequest::get()
        .uri(&format!(
            "/api/v1/buildings/{}/technical-inspections/overdue",
            building_id
        ))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200, "Should return overdue inspections list");

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(
        body.is_array(),
        "Overdue inspections should be an array (may be empty)"
    );
}

#[actix_web::test]
#[serial]
async fn test_technical_inspections_delete() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = setup_inspection_user_token(&app_state, org_id).await;
    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let building_id = create_inspection_test_building(&app_state, org_id).await;

    // Create inspection to delete
    let create_req = test::TestRequest::post()
        .uri("/api/v1/technical-inspections")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "organization_id": org_id.to_string(),
            "building_id": building_id.to_string(),
            "title": "Inspection à supprimer",
            "description": null,
            "inspection_type": "fire_extinguisher",
            "inspector_name": "Bureau de Contrôle Test",
            "inspection_date": "2026-03-12T00:00:00Z"
        }))
        .to_request();
    let create_resp = test::call_service(&app, create_req).await;
    assert_eq!(create_resp.status(), 201);
    let created: serde_json::Value = test::read_body_json(create_resp).await;
    let inspection_id = created["id"].as_str().unwrap().to_string();

    // Delete
    let req = test::TestRequest::delete()
        .uri(&format!("/api/v1/technical-inspections/{}", inspection_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(
        resp.status() == 204 || resp.status() == 200,
        "Should delete technical inspection (204 or 200)"
    );
}
