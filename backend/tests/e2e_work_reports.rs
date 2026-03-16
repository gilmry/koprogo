// E2E tests for Work Report HTTP endpoints (Issue #134 - Digital Maintenance Logbook)
// Tests cover creation, retrieval, listing by building, update, delete,
// and active warranties query.

mod common;

use actix_web::http::header;
use actix_web::{test, App};
use koprogo_api::application::dto::RegisterRequest;
use koprogo_api::infrastructure::web::configure_routes;
use serde_json::json;
use serial_test::serial;
use uuid::Uuid;

/// Helper: Register user and return token
async fn setup_work_report_token(
    app_state: &actix_web::web::Data<koprogo_api::infrastructure::web::AppState>,
    org_id: Uuid,
) -> String {
    let email = format!("work-report-{}@test.com", Uuid::new_v4());
    let reg = RegisterRequest {
        email: email.clone(),
        password: "Passw0rd!".to_string(),
        first_name: "Work".to_string(),
        last_name: "ReportTester".to_string(),
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
async fn create_work_report_test_building(
    app_state: &actix_web::web::Data<koprogo_api::infrastructure::web::AppState>,
    org_id: Uuid,
) -> Uuid {
    use koprogo_api::application::dto::CreateBuildingDto;
    let dto = CreateBuildingDto {
        organization_id: org_id.to_string(),
        name: format!("Work Report Test Building {}", Uuid::new_v4()),
        address: "77 Rue des Travaux".to_string(),
        city: "Ghent".to_string(),
        postal_code: "9000".to_string(),
        country: "Belgium".to_string(),
        total_units: 15,
        construction_year: Some(1975),
        total_tantiemes: Some(1000),
    };
    let building = app_state
        .building_use_cases
        .create_building(dto)
        .await
        .expect("Failed to create test building for work reports");
    Uuid::parse_str(&building.id).unwrap()
}

// ==================== Work Report Tests ====================

#[actix_web::test]
#[serial]
async fn test_work_reports_create() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = setup_work_report_token(&app_state, org_id).await;
    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let building_id = create_work_report_test_building(&app_state, org_id).await;

    let req = test::TestRequest::post()
        .uri("/api/v1/work-reports")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "organization_id": org_id.to_string(),
            "building_id": building_id.to_string(),
            "title": "Remplacement chaudière",
            "description": "Remplacement de la chaudière collective — condensation haute efficacité.",
            "work_type": "repair",
            "contractor_name": "Chauffage Dupuis SPRL",
            "contractor_contact": "info@dupuis-chauffage.be",
            "work_date": "2026-03-01T00:00:00Z",
            "completion_date": "2026-03-03T00:00:00Z",
            "cost": 8500.0,
            "invoice_number": "INV-2026-0301",
            "notes": "Garantie 2 ans pièces et main d'œuvre",
            "warranty_type": "standard"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201, "Should create work report successfully");

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["title"], "Remplacement chaudière");
    assert_eq!(body["contractor_name"], "Chauffage Dupuis SPRL");
    assert_eq!(body["work_type"], "repair");
    assert_eq!(body["warranty_type"], "standard");
}

#[actix_web::test]
#[serial]
async fn test_work_reports_get() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = setup_work_report_token(&app_state, org_id).await;
    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let building_id = create_work_report_test_building(&app_state, org_id).await;

    // Create work report
    let create_req = test::TestRequest::post()
        .uri("/api/v1/work-reports")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "organization_id": org_id.to_string(),
            "building_id": building_id.to_string(),
            "title": "Rénovation façade",
            "description": "Ravalement de façade et isolation thermique.",
            "work_type": "renovation",
            "contractor_name": "Façaderie Moreau",
            "work_date": "2026-02-15T00:00:00Z",
            "cost": 45000.0,
            "warranty_type": "decennial"
        }))
        .to_request();
    let create_resp = test::call_service(&app, create_req).await;
    assert_eq!(create_resp.status(), 201);
    let created: serde_json::Value = test::read_body_json(create_resp).await;
    let report_id = created["id"].as_str().unwrap().to_string();

    // Get by ID
    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/work-reports/{}", report_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200, "Should get work report by ID");

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["id"], report_id);
    assert_eq!(body["title"], "Rénovation façade");
    assert_eq!(body["work_type"], "renovation");
}

#[actix_web::test]
#[serial]
async fn test_work_reports_list_by_building() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = setup_work_report_token(&app_state, org_id).await;
    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let building_id = create_work_report_test_building(&app_state, org_id).await;

    // Create two work reports
    for (i, work_type) in [(1, "maintenance"), (2, "repair")] {
        let create_req = test::TestRequest::post()
            .uri("/api/v1/work-reports")
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
            .set_json(json!({
                "organization_id": org_id.to_string(),
                "building_id": building_id.to_string(),
                "title": format!("Travaux #{}", i),
                "description": "Description des travaux",
                "work_type": work_type,
                "contractor_name": format!("Entrepreneur #{}", i),
                "work_date": "2026-01-10T00:00:00Z",
                "cost": 1000.0,
                "warranty_type": "none"
            }))
            .to_request();
        let create_resp = test::call_service(&app, create_req).await;
        assert_eq!(create_resp.status(), 201);
    }

    // List by building
    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/buildings/{}/work-reports", building_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200, "Should list work reports for building");

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body.is_array());
    assert_eq!(
        body.as_array().unwrap().len(),
        2,
        "Should return 2 work reports"
    );
}

#[actix_web::test]
#[serial]
async fn test_work_reports_update() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = setup_work_report_token(&app_state, org_id).await;
    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let building_id = create_work_report_test_building(&app_state, org_id).await;

    // Create work report
    let create_req = test::TestRequest::post()
        .uri("/api/v1/work-reports")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "organization_id": org_id.to_string(),
            "building_id": building_id.to_string(),
            "title": "Titre initial",
            "description": "Description initiale.",
            "work_type": "maintenance",
            "contractor_name": "Contrat Initial",
            "work_date": "2026-03-05T00:00:00Z",
            "cost": 500.0,
            "warranty_type": "none"
        }))
        .to_request();
    let create_resp = test::call_service(&app, create_req).await;
    assert_eq!(create_resp.status(), 201);
    let created: serde_json::Value = test::read_body_json(create_resp).await;
    let report_id = created["id"].as_str().unwrap().to_string();

    // Update
    let req = test::TestRequest::put()
        .uri(&format!("/api/v1/work-reports/{}", report_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "title": "Titre mis à jour",
            "notes": "Ajout de notes post-travaux",
            "cost": 650.0
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200, "Should update work report successfully");

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["title"], "Titre mis à jour");
}

#[actix_web::test]
#[serial]
async fn test_work_reports_delete() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = setup_work_report_token(&app_state, org_id).await;
    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let building_id = create_work_report_test_building(&app_state, org_id).await;

    // Create work report
    let create_req = test::TestRequest::post()
        .uri("/api/v1/work-reports")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "organization_id": org_id.to_string(),
            "building_id": building_id.to_string(),
            "title": "Rapport à supprimer",
            "description": "Ce rapport sera supprimé.",
            "work_type": "inspection",
            "contractor_name": "Inspecteur Test",
            "work_date": "2026-03-08T00:00:00Z",
            "cost": 200.0,
            "warranty_type": "none"
        }))
        .to_request();
    let create_resp = test::call_service(&app, create_req).await;
    assert_eq!(create_resp.status(), 201);
    let created: serde_json::Value = test::read_body_json(create_resp).await;
    let report_id = created["id"].as_str().unwrap().to_string();

    // Delete
    let req = test::TestRequest::delete()
        .uri(&format!("/api/v1/work-reports/{}", report_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(
        resp.status() == 204 || resp.status() == 200,
        "Should delete work report (204 or 200)"
    );
}

#[actix_web::test]
#[serial]
async fn test_work_reports_warranties_active() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = setup_work_report_token(&app_state, org_id).await;
    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let building_id = create_work_report_test_building(&app_state, org_id).await;

    // Create a work report with a decennial warranty (10 years — will be active)
    let create_req = test::TestRequest::post()
        .uri("/api/v1/work-reports")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "organization_id": org_id.to_string(),
            "building_id": building_id.to_string(),
            "title": "Rénovation structure — garantie décennale",
            "description": "Travaux de structure avec garantie décennale.",
            "work_type": "renovation",
            "contractor_name": "Construction Legrand SA",
            "work_date": "2025-06-01T00:00:00Z",
            "cost": 120000.0,
            "warranty_type": "decennial"
        }))
        .to_request();
    let create_resp = test::call_service(&app, create_req).await;
    assert_eq!(create_resp.status(), 201);

    // Query active warranties
    let req = test::TestRequest::get()
        .uri(&format!(
            "/api/v1/buildings/{}/work-reports/warranties/active",
            building_id
        ))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200, "Should return active warranties list");

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body.is_array(), "Response should be an array");
    assert!(
        !body.as_array().unwrap().is_empty(),
        "Should contain the decennial warranty work report"
    );
}
