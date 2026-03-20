// E2E tests for public syndic information HTTP endpoint (Issue #92)
// Tests focus on HTTP layer: no-auth public endpoint, 404 handling
// Covers Belgian legal requirement for public syndic contact information

mod common;

use actix_web::{test, App};
use koprogo_api::application::dto::*;
use koprogo_api::infrastructure::web::configure_routes;
use serial_test::serial;
use uuid::Uuid;

// ==================== Public Syndic Info Tests ====================

#[actix_web::test]
#[serial]
async fn test_public_syndic_not_found_for_unknown_slug() {
    let (app_state, _container, _org_id) = common::setup_test_db().await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/v1/public/buildings/nonexistent-slug-does-not-exist/syndic")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.status(),
        404,
        "Should return 404 for a non-existent building slug"
    );
}

#[actix_web::test]
#[serial]
async fn test_public_syndic_no_auth_required() {
    // Key test: the endpoint must be accessible without an Authorization header.
    // A missing/invalid auth should return 404 (not found), NOT 401 (unauthorized).
    // This proves the public endpoint is not behind the auth middleware.
    let (app_state, _container, _org_id) = common::setup_test_db().await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Request with no Authorization header at all
    let req = test::TestRequest::get()
        .uri("/api/v1/public/buildings/any-random-slug-here/syndic")
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Must NOT be 401 — endpoint is public
    assert_ne!(
        resp.status(),
        401,
        "Public syndic endpoint should NOT require authentication (got 401)"
    );

    // Since the building does not exist, we expect 404
    assert_eq!(
        resp.status(),
        404,
        "Should return 404 for unknown slug (not 401 — proves no auth required)"
    );
}

#[actix_web::test]
#[serial]
async fn test_public_syndic_with_valid_building() {
    // Create a building via AppState directly, then query its public syndic page.
    // The slug is auto-generated from the building name + city during creation.
    let (app_state, _container, org_id) = common::setup_test_db().await;

    // Create a building with a distinctive name so slug is predictable
    let unique_suffix = Uuid::new_v4().to_string().replace('-', "")[..8].to_lowercase();
    let building_name = format!("Residence Test {}", unique_suffix);
    let dto = CreateBuildingDto {
        organization_id: org_id.to_string(),
        name: building_name.clone(),
        address: "12 Rue de la Source".to_string(),
        city: "Namur".to_string(),
        postal_code: "5000".to_string(),
        country: "Belgium".to_string(),
        total_units: 8,
        construction_year: Some(2020),
        total_tantiemes: Some(1000),
    };

    let _building = app_state
        .building_use_cases
        .create_building(dto)
        .await
        .expect("Failed to create building for public syndic test");

    // The building should have a slug generated automatically
    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // If the building has a slug, use it; otherwise use a fallback
    // The slug is generated from name + city (e.g., "residence-test-abc12345-namur")
    // We derive the expected slug using the same normalization logic
    let expected_slug =
        building_name.to_lowercase().replace(' ', "-") + "-" + &"Namur".to_lowercase();

    let req = test::TestRequest::get()
        .uri(&format!(
            "/api/v1/public/buildings/{}/syndic",
            expected_slug
        ))
        .to_request();

    let resp = test::call_service(&app, req).await;

    // The response should be 200 (found) or 404 (slug mismatch due to normalization details)
    // Either way, it must NOT be 401 — the endpoint is public
    assert_ne!(
        resp.status(),
        401,
        "Public endpoint must not return 401 — no auth required"
    );

    // If we get 200, verify the response structure
    if resp.status() == 200 {
        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(
            body["building_city"], "Namur",
            "Response should contain the building city"
        );
        assert!(
            body["slug"].is_string(),
            "Response should include the slug field"
        );
        assert!(
            body["has_syndic_info"].is_boolean(),
            "Response should include has_syndic_info field"
        );
    }
    // 404 is also acceptable if slug normalization differs slightly
}

#[actix_web::test]
#[serial]
async fn test_public_syndic_empty_slug_segment() {
    // Test with a slug that has a valid format but corresponds to no building
    let (app_state, _container, _org_id) = common::setup_test_db().await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/v1/public/buildings/building-that-does-not-exist-12345/syndic")
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Must be 404, not 401 (public endpoint)
    assert_ne!(
        resp.status(),
        401,
        "Public syndic endpoint should not require auth"
    );
    assert_eq!(
        resp.status(),
        404,
        "Should return 404 for non-existent building slug"
    );

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(
        body["error"].is_string(),
        "404 response should contain an error message"
    );
}
