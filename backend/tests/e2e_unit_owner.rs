// E2E tests for unit_owner HTTP endpoints (Issue #32)
// Tests focus on HTTP layer: endpoints, auth, JSON serialization
// Integration tests (integration_unit_owner.rs) cover business logic

mod common;

use actix_web::http::header;
use actix_web::{test, App};
use koprogo_api::application::dto::*;
use koprogo_api::domain::entities::UnitType;
use koprogo_api::infrastructure::web::configure_routes;
use serde_json::json;
use serial_test::serial;
use uuid::Uuid;

/// Helper: Register user, get JWT, create building, unit, and owner for tests.
/// Uses the shared common::setup_test_db() for AppState + container + org_id.
async fn create_test_fixtures(
    app_state: &actix_web::web::Data<koprogo_api::infrastructure::web::AppState>,
    org_id: Uuid,
) -> (String, Uuid, Uuid, Uuid) {
    // Register + login to get JWT token
    let token = common::register_and_login(app_state, org_id).await;

    // Create building using DTO
    let building_dto = CreateBuildingDto {
        organization_id: org_id.to_string(),
        name: "Test Building".to_string(),
        address: "123 Main St".to_string(),
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

    let building_id = Uuid::parse_str(&building.id).expect("Invalid building ID");

    // Create unit using DTO
    let unit_dto = CreateUnitDto {
        organization_id: org_id.to_string(),
        building_id: building_id.to_string(),
        unit_number: "101".to_string(),
        unit_type: UnitType::Apartment,
        floor: Some(1),
        surface_area: 75.5,
        quota: 0.5,
    };

    let unit = app_state
        .unit_use_cases
        .create_unit(unit_dto)
        .await
        .expect("Failed to create unit");

    let unit_id = Uuid::parse_str(&unit.id).expect("Invalid unit ID");

    // Create owner using DTO
    let owner_dto = CreateOwnerDto {
        organization_id: org_id.to_string(),
        first_name: "John".to_string(),
        last_name: "Doe".to_string(),
        email: "john.doe@example.com".to_string(),
        phone: Some("+32123456789".to_string()),
        address: "456 Oak St".to_string(),
        city: "Brussels".to_string(),
        postal_code: "1000".to_string(),
        country: "Belgium".to_string(),
    };

    let owner = app_state
        .owner_use_cases
        .create_owner(owner_dto)
        .await
        .expect("Failed to create owner");

    let owner_id = Uuid::parse_str(&owner.id).expect("Invalid owner ID");

    (token, building_id, unit_id, owner_id)
}

//
// ═══════════════════════════════════════════════════════════════════════════
// TEST: POST /units/:unit_id/owners (Add owner to unit)
// ═══════════════════════════════════════════════════════════════════════════
//

#[actix_web::test]
#[serial]
async fn test_add_owner_to_unit_success() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, _building_id, unit_id, owner_id) = create_test_fixtures(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Add owner with 100% ownership
    let req = test::TestRequest::post()
        .uri(&format!("/api/v1/units/{}/owners", unit_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "owner_id": owner_id.to_string(),
            "ownership_percentage": 1.0,
            "is_primary_contact": true
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201, "Should create unit-owner relationship");

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["unit_id"], unit_id.to_string());
    assert_eq!(body["owner_id"], owner_id.to_string());
    assert_eq!(body["ownership_percentage"], 1.0);
    assert_eq!(body["is_primary_contact"], true);
    assert_eq!(body["is_active"], true);
}

#[actix_web::test]
#[serial]
async fn test_add_owner_to_unit_exceeds_100_percent() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, _building_id, unit_id, owner_id) = create_test_fixtures(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Add first owner with 60%
    let req = test::TestRequest::post()
        .uri(&format!("/api/v1/units/{}/owners", unit_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "owner_id": owner_id.to_string(),
            "ownership_percentage": 0.6,
            "is_primary_contact": true
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    // Create second owner
    let owner_dto2 = CreateOwnerDto {
        organization_id: org_id.to_string(),
        first_name: "Jane".to_string(),
        last_name: "Smith".to_string(),
        email: "jane.smith@example.com".to_string(),
        phone: Some("+32987654321".to_string()),
        address: "789 Elm St".to_string(),
        city: "Brussels".to_string(),
        postal_code: "1000".to_string(),
        country: "Belgium".to_string(),
    };

    let owner2 = app_state
        .owner_use_cases
        .create_owner(owner_dto2)
        .await
        .expect("Failed to create second owner");

    let owner2_id = Uuid::parse_str(&owner2.id).expect("Invalid owner2 ID");

    // Try to add second owner with 50% (total would be 110%)
    let req = test::TestRequest::post()
        .uri(&format!("/api/v1/units/{}/owners", unit_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "owner_id": owner2_id.to_string(),
            "ownership_percentage": 0.5,
            "is_primary_contact": false
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.status(),
        400,
        "Should fail when total ownership exceeds 100%"
    );

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(
        body["error"].as_str().unwrap().contains("exceed 100%"),
        "Error message should mention exceeding 100%"
    );
}

//
// ═══════════════════════════════════════════════════════════════════════════
// TEST: GET /units/:unit_id/owners (List owners for unit)
// ═══════════════════════════════════════════════════════════════════════════
//

#[actix_web::test]
#[serial]
async fn test_list_owners_for_unit_success() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, _building_id, unit_id, owner_id) = create_test_fixtures(&app_state, org_id).await;

    // Add owner first
    let req = test::TestRequest::post()
        .uri(&format!("/api/v1/units/{}/owners", unit_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "owner_id": owner_id.to_string(),
            "ownership_percentage": 1.0,
            "is_primary_contact": true
        }))
        .to_request();

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    // List owners
    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/units/{}/owners", unit_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = test::read_body_json(resp).await;
    // Handler returns flat Vec<UnitOwnerResponseDto>
    assert!(body.is_array(), "Should return array of unit owners");
    let owners = body.as_array().unwrap();
    assert_eq!(owners.len(), 1);
    assert_eq!(owners[0]["ownership_percentage"], 1.0);
}

//
// ═══════════════════════════════════════════════════════════════════════════
// TEST: DELETE /units/:unit_id/owners/:owner_id (Remove owner from unit)
// ═══════════════════════════════════════════════════════════════════════════
//

#[actix_web::test]
#[serial]
async fn test_remove_owner_from_unit_success() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, _building_id, unit_id, owner_id) = create_test_fixtures(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Add owner
    let req = test::TestRequest::post()
        .uri(&format!("/api/v1/units/{}/owners", unit_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "owner_id": owner_id.to_string(),
            "ownership_percentage": 1.0,
            "is_primary_contact": true
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    // Remove owner
    let req = test::TestRequest::delete()
        .uri(&format!("/api/v1/units/{}/owners/{}", unit_id, owner_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200, "Should remove owner successfully");

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(
        body["is_active"], false,
        "Ownership should be marked inactive"
    );
    assert!(body["end_date"].is_string(), "end_date should be set");
}

//
// ═══════════════════════════════════════════════════════════════════════════
// TEST: PUT /unit-owners/:id (Update ownership details)
// ═══════════════════════════════════════════════════════════════════════════
//

#[actix_web::test]
#[serial]
async fn test_update_unit_owner_percentage_success() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, _building_id, unit_id, owner_id) = create_test_fixtures(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Add owner with 100%
    let req = test::TestRequest::post()
        .uri(&format!("/api/v1/units/{}/owners", unit_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "owner_id": owner_id.to_string(),
            "ownership_percentage": 1.0,
            "is_primary_contact": true
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    let created: serde_json::Value = test::read_body_json(resp).await;
    let relationship_id = created["id"].as_str().unwrap();

    // Update to 80%
    let req = test::TestRequest::put()
        .uri(&format!("/api/v1/unit-owners/{}", relationship_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "ownership_percentage": 0.8,
            "is_primary_contact": true
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200, "Should update ownership percentage");

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["ownership_percentage"], 0.8);
}

//
// ═══════════════════════════════════════════════════════════════════════════
// TEST: GET /owners/:owner_id/units (Get owner's units)
// ═══════════════════════════════════════════════════════════════════════════
//

#[actix_web::test]
#[serial]
async fn test_get_owner_units_success() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, _building_id, unit_id, owner_id) = create_test_fixtures(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Add owner to unit
    let req = test::TestRequest::post()
        .uri(&format!("/api/v1/units/{}/owners", unit_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "owner_id": owner_id.to_string(),
            "ownership_percentage": 1.0,
            "is_primary_contact": true
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    // Get owner's units
    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/owners/{}/units", owner_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = test::read_body_json(resp).await;
    // Handler returns flat Vec<UnitOwnerResponseDto>
    assert!(
        body.is_array(),
        "Should return array of unit-owner relationships"
    );
    let units = body.as_array().unwrap();
    assert_eq!(units.len(), 1);
    assert_eq!(units[0]["owner_id"], owner_id.to_string());
}

//
// ═══════════════════════════════════════════════════════════════════════════
// TEST: GET /units/:unit_id/ownership-history (Get ownership history)
// ═══════════════════════════════════════════════════════════════════════════
//

#[actix_web::test]
#[serial]
async fn test_get_unit_ownership_history_success() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, _building_id, unit_id, owner_id) = create_test_fixtures(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Add owner
    let req = test::TestRequest::post()
        .uri(&format!("/api/v1/units/{}/owners", unit_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "owner_id": owner_id.to_string(),
            "ownership_percentage": 1.0,
            "is_primary_contact": true
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    // Remove owner (creates history)
    let req = test::TestRequest::delete()
        .uri(&format!("/api/v1/units/{}/owners/{}", unit_id, owner_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    // Get history
    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/units/{}/owners/history", unit_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body.is_array());
    assert!(!body.as_array().unwrap().is_empty(), "Should have history");
}

//
// ═══════════════════════════════════════════════════════════════════════════
// TEST: POST /units/:unit_id/owners/transfer (Transfer ownership)
// ═══════════════════════════════════════════════════════════════════════════
//

#[actix_web::test]
#[serial]
async fn test_transfer_ownership_success() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, _building_id, unit_id, owner1_id) = create_test_fixtures(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Add first owner with 100%
    let req = test::TestRequest::post()
        .uri(&format!("/api/v1/units/{}/owners", unit_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "owner_id": owner1_id.to_string(),
            "ownership_percentage": 1.0,
            "is_primary_contact": true
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    // Create second owner
    let owner_dto2 = CreateOwnerDto {
        organization_id: org_id.to_string(),
        first_name: "Jane".to_string(),
        last_name: "Smith".to_string(),
        email: "jane.transfer@example.com".to_string(),
        phone: Some("+32111111111".to_string()),
        address: "789 Elm St".to_string(),
        city: "Brussels".to_string(),
        postal_code: "1000".to_string(),
        country: "Belgium".to_string(),
    };

    let owner2 = app_state
        .owner_use_cases
        .create_owner(owner_dto2)
        .await
        .expect("Failed to create second owner");

    let owner2_id = Uuid::parse_str(&owner2.id).expect("Invalid owner2 ID");

    // Transfer ownership
    let req = test::TestRequest::post()
        .uri(&format!("/api/v1/units/{}/owners/transfer", unit_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "from_owner_id": owner1_id.to_string(),
            "to_owner_id": owner2_id.to_string()
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200, "Should transfer ownership successfully");

    let body: serde_json::Value = test::read_body_json(resp).await;
    // Handler returns { ended_relationship, new_relationship }
    assert!(
        body["new_relationship"].is_object(),
        "Should have new_relationship"
    );
    assert_eq!(body["new_relationship"]["owner_id"], owner2_id.to_string());
    assert_eq!(body["new_relationship"]["ownership_percentage"], 1.0);
    assert_eq!(body["new_relationship"]["is_active"], true);
}

//
// ═══════════════════════════════════════════════════════════════════════════
// TEST: GET /units/:unit_id/owners/total-percentage (Get total ownership %)
// ═══════════════════════════════════════════════════════════════════════════
//

#[actix_web::test]
#[serial]
async fn test_get_total_ownership_percentage_success() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, _building_id, unit_id, owner_id) = create_test_fixtures(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Add owner with 60%
    let req = test::TestRequest::post()
        .uri(&format!("/api/v1/units/{}/owners", unit_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "owner_id": owner_id.to_string(),
            "ownership_percentage": 0.6,
            "is_primary_contact": true
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    // Get total percentage
    let req = test::TestRequest::get()
        .uri(&format!(
            "/api/v1/units/{}/owners/total-percentage",
            unit_id
        ))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["total_ownership_percentage"], 0.6);
}

//
// ═══════════════════════════════════════════════════════════════════════════
// TEST: Authentication & Authorization
// ═══════════════════════════════════════════════════════════════════════════
//

#[actix_web::test]
#[serial]
async fn test_unit_owner_endpoints_require_auth() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (_token, _building_id, unit_id, owner_id) = create_test_fixtures(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Try POST without auth
    let req = test::TestRequest::post()
        .uri(&format!("/api/v1/units/{}/owners", unit_id))
        .set_json(json!({
            "owner_id": owner_id.to_string(),
            "ownership_percentage": 1.0,
            "is_primary_contact": true
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401, "Should reject unauthorized request");

    // Try GET without auth
    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/units/{}/owners", unit_id))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401, "Should reject unauthorized request");
}
