// E2E tests for GET /organizations/{organization_id}/users
//
// Story S1 — docs/maury/syndic-org-users-endpoint/stories.md (SIGNED v1.0
// @gilmry 2026-08-07). Org-scoped listing (syndic/accountant own org,
// superadmin any org), mirror exact du pattern list_organization_tickets.

mod common;

use actix_web::http::header;
use actix_web::{test, App};
use koprogo_api::infrastructure::web::configure_routes;
use serial_test::serial;

/// @happy — syndic listing their own organization gets 200 + the users.
#[actix_web::test]
#[serial]
async fn test_organization_users_happy_syndic_own_org() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let syndic_token = common::register_and_login_with_role(&app_state, org_id, "syndic").await;
    let _contractor_token =
        common::register_and_login_with_role(&app_state, org_id, "contractor").await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/organizations/{}/users", org_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", syndic_token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200, "syndic should list own org's users");

    let body: serde_json::Value = test::read_body_json(resp).await;
    let data = body["data"].as_array().expect("data should be an array");
    // syndic (self) + contractor registered above.
    assert!(
        data.len() >= 2,
        "should contain at least the syndic and the contractor"
    );
    assert!(data
        .iter()
        .any(|u| u["role"] == "contractor" || u["active_role"]["role"] == "contractor"));
}

/// @happy — accountant listing their own organization gets 200 too
/// (verify_org_access does not distinguish syndic/accountant).
#[actix_web::test]
#[serial]
async fn test_organization_users_happy_accountant_own_org() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let accountant_token =
        common::register_and_login_with_role(&app_state, org_id, "accountant").await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/organizations/{}/users", org_id))
        .insert_header((
            header::AUTHORIZATION,
            format!("Bearer {}", accountant_token),
        ))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200, "accountant should list own org's users");
}

/// @happy — superadmin can list any organization's users.
#[actix_web::test]
#[serial]
async fn test_organization_users_happy_superadmin_any_org() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let superadmin_token = common::register_and_login(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/organizations/{}/users", org_id))
        .insert_header((
            header::AUTHORIZATION,
            format!("Bearer {}", superadmin_token),
        ))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200, "superadmin should list any org's users");
}

/// @edge — an organization with no users beyond the caller returns 200 +
/// an empty-ish list handled gracefully (no error).
#[actix_web::test]
#[serial]
async fn test_organization_users_edge_org_with_single_user() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let syndic_token = common::register_and_login_with_role(&app_state, org_id, "syndic").await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/organizations/{}/users", org_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", syndic_token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = test::read_body_json(resp).await;
    let data = body["data"].as_array().expect("data should be an array");
    assert_eq!(
        data.len(),
        1,
        "org should contain exactly the syndic who just registered"
    );
}

/// @security — a syndic from org A cannot list org B's users.
#[actix_web::test]
#[serial]
async fn test_organization_users_security_cross_org_forbidden() {
    let (app_state, _container, org_a_id) = common::setup_test_db().await;
    let syndic_a_token = common::register_and_login_with_role(&app_state, org_a_id, "syndic").await;

    // Second organization (org B), distinct from org A.
    let org_b = app_state
        .organization_use_cases
        .create(
            "Org B Test".to_string(),
            format!("org-b-test-{}", uuid::Uuid::new_v4()),
            format!("org-b-{}@test.com", uuid::Uuid::new_v4()),
            None,
            "starter".to_string(),
        )
        .await
        .expect("create org B");

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/organizations/{}/users", org_b.id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", syndic_a_token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.status(),
        403,
        "syndic of org A must not list org B's users"
    );
}

/// @negative — no Bearer token → 401 (existing `AuthenticatedUser` extractor
/// behaviour, no new code path, kept as a regression guard for this route).
#[actix_web::test]
#[serial]
async fn test_organization_users_negative_unauthenticated() {
    let (app_state, _container, org_id) = common::setup_test_db().await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/organizations/{}/users", org_id))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401, "should require authentication");
}

/// Regression — `GET /users` (superadmin-only, unscoped) is untouched by
/// this new endpoint.
#[actix_web::test]
#[serial]
async fn test_users_unscoped_still_superadmin_only_after_change() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let syndic_token = common::register_and_login_with_role(&app_state, org_id, "syndic").await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/v1/users")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", syndic_token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.status(),
        403,
        "GET /users must remain superadmin-only — no regression from the new org-scoped endpoint"
    );
}
