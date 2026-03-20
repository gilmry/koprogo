// E2E tests for AGE Request HTTP endpoints (BC17 - Art. 3.87 §2 CC)
// Covers creation, listing, state transitions (open, submit) and deletion
// of demandes d'AGE par copropriétaires.

mod common;

use actix_web::http::header;
use actix_web::{test, App};
use koprogo_api::application::dto::RegisterRequest;
use koprogo_api::infrastructure::web::configure_routes;
use serde_json::json;
use serial_test::serial;
use uuid::Uuid;

/// Helper: Register user and return token
async fn setup_age_user_token(
    app_state: &actix_web::web::Data<koprogo_api::infrastructure::web::AppState>,
    org_id: Uuid,
) -> String {
    let email = format!("age-req-{}@test.com", Uuid::new_v4());
    let reg = RegisterRequest {
        email: email.clone(),
        password: "Passw0rd!".to_string(),
        first_name: "AGE".to_string(),
        last_name: "RequestTester".to_string(),
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
async fn create_age_test_building(
    app_state: &actix_web::web::Data<koprogo_api::infrastructure::web::AppState>,
    org_id: Uuid,
) -> Uuid {
    use koprogo_api::application::dto::CreateBuildingDto;
    let dto = CreateBuildingDto {
        organization_id: org_id.to_string(),
        name: format!("AGE Request Test Building {}", Uuid::new_v4()),
        address: "5 Avenue du Parlement".to_string(),
        city: "Liège".to_string(),
        postal_code: "4000".to_string(),
        country: "Belgium".to_string(),
        total_units: 12,
        construction_year: Some(1995),
        total_tantiemes: Some(1000),
    };
    let building = app_state
        .building_use_cases
        .create_building(dto)
        .await
        .expect("Failed to create test building for AGE requests");
    Uuid::parse_str(&building.id).unwrap()
}

// ==================== AGE Request Tests ====================

#[actix_web::test]
#[serial]
async fn test_age_requests_create() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = setup_age_user_token(&app_state, org_id).await;
    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let building_id = create_age_test_building(&app_state, org_id).await;

    let req = test::TestRequest::post()
        .uri(&format!(
            "/api/v1/buildings/{}/age-requests",
            building_id
        ))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "title": "Demande d'AGE — Rénovation toiture urgente",
            "description": "Suite aux dégâts causés par les intempéries, une AGE est nécessaire pour voter les travaux d'urgence."
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201, "Should create AGE request successfully");

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["title"], "Demande d'AGE — Rénovation toiture urgente");
    assert_eq!(body["status"], "draft");
    assert_eq!(body["building_id"], building_id.to_string());
    assert!(!body["threshold_reached"].as_bool().unwrap_or(true));
}

#[actix_web::test]
#[serial]
async fn test_age_requests_get() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = setup_age_user_token(&app_state, org_id).await;
    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let building_id = create_age_test_building(&app_state, org_id).await;

    // Create AGE request
    let create_req = test::TestRequest::post()
        .uri(&format!("/api/v1/buildings/{}/age-requests", building_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "title": "AGE pour travaux d'ascenseur",
            "description": "Remplacement de la cabine d'ascenseur hors normes."
        }))
        .to_request();
    let create_resp = test::call_service(&app, create_req).await;
    assert_eq!(create_resp.status(), 201);
    let created: serde_json::Value = test::read_body_json(create_resp).await;
    let age_id = created["id"].as_str().unwrap().to_string();

    // Get by ID
    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/age-requests/{}", age_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200, "Should get AGE request by ID");

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["id"], age_id);
    assert_eq!(body["title"], "AGE pour travaux d'ascenseur");
    assert_eq!(body["building_id"], building_id.to_string());
}

#[actix_web::test]
#[serial]
async fn test_age_requests_list_by_building() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = setup_age_user_token(&app_state, org_id).await;
    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let building_id = create_age_test_building(&app_state, org_id).await;

    // Create two AGE requests
    for i in 1..=2 {
        let create_req = test::TestRequest::post()
            .uri(&format!("/api/v1/buildings/{}/age-requests", building_id))
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
            .set_json(json!({
                "title": format!("AGE Demande #{}", i),
                "description": null
            }))
            .to_request();
        let create_resp = test::call_service(&app, create_req).await;
        assert_eq!(create_resp.status(), 201);
    }

    // List all for building
    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/buildings/{}/age-requests", building_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200, "Should list AGE requests for building");

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body.is_array());
    assert_eq!(
        body.as_array().unwrap().len(),
        2,
        "Should return 2 AGE requests"
    );
}

#[actix_web::test]
#[serial]
async fn test_age_requests_open() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = setup_age_user_token(&app_state, org_id).await;
    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let building_id = create_age_test_building(&app_state, org_id).await;

    // Create AGE request in Draft
    let create_req = test::TestRequest::post()
        .uri(&format!("/api/v1/buildings/{}/age-requests", building_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "title": "Demande d'AGE — Installation panneaux solaires",
            "description": "Étude de faisabilité et vote."
        }))
        .to_request();
    let create_resp = test::call_service(&app, create_req).await;
    assert_eq!(create_resp.status(), 201);
    let created: serde_json::Value = test::read_body_json(create_resp).await;
    let age_id = created["id"].as_str().unwrap().to_string();

    // Open for signatures
    let req = test::TestRequest::put()
        .uri(&format!("/api/v1/age-requests/{}/open", age_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200, "Should open AGE request for signatures");

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["status"], "open");
}

#[actix_web::test]
#[serial]
async fn test_age_requests_submit_requires_threshold() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = setup_age_user_token(&app_state, org_id).await;
    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let building_id = create_age_test_building(&app_state, org_id).await;

    // Create and open AGE request
    let create_req = test::TestRequest::post()
        .uri(&format!("/api/v1/buildings/{}/age-requests", building_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "title": "AGE sans cosignataires suffisants",
            "description": "Tentative de soumission sans atteindre le seuil de 1/5."
        }))
        .to_request();
    let create_resp = test::call_service(&app, create_req).await;
    assert_eq!(create_resp.status(), 201);
    let created: serde_json::Value = test::read_body_json(create_resp).await;
    let age_id = created["id"].as_str().unwrap().to_string();

    // Open first
    let open_req = test::TestRequest::put()
        .uri(&format!("/api/v1/age-requests/{}/open", age_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();
    let open_resp = test::call_service(&app, open_req).await;
    assert_eq!(open_resp.status(), 200);

    // Try to submit without enough cosignatories — should fail (400)
    let req = test::TestRequest::post()
        .uri(&format!("/api/v1/age-requests/{}/submit", age_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.status(),
        400,
        "Should reject submit when threshold not reached"
    );
}

#[actix_web::test]
#[serial]
async fn test_age_requests_delete() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = setup_age_user_token(&app_state, org_id).await;
    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let building_id = create_age_test_building(&app_state, org_id).await;

    // Create AGE request (Draft state — deletable)
    let create_req = test::TestRequest::post()
        .uri(&format!("/api/v1/buildings/{}/age-requests", building_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "title": "AGE à supprimer",
            "description": null
        }))
        .to_request();
    let create_resp = test::call_service(&app, create_req).await;
    assert_eq!(create_resp.status(), 201);
    let created: serde_json::Value = test::read_body_json(create_resp).await;
    let age_id = created["id"].as_str().unwrap().to_string();

    // Delete
    let req = test::TestRequest::delete()
        .uri(&format!("/api/v1/age-requests/{}", age_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(
        resp.status() == 204 || resp.status() == 200,
        "Should delete AGE request in Draft state (204 or 200)"
    );
}
