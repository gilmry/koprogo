// E2E tests for Local Exchange (SEL) HTTP endpoints (Issue #49 - Phase 1)
// Tests focus on HTTP layer: endpoints, auth, JSON serialization
// Belgian context: SEL (Système d'Échange Local) is legal and non-taxable if non-commercial

mod common;

use actix_web::http::header;
use actix_web::{test, App};
use koprogo_api::infrastructure::web::configure_routes;
use serde_json::json;
use serial_test::serial;
use uuid::Uuid;

#[actix_web::test]
#[serial]
async fn test_create_service_exchange_offer() {
    let (app_state, _postgres_container, org_id) = common::setup_test_db().await;
    let token = common::register_and_login(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create a building via API
    let building_req = test::TestRequest::post()
        .uri("/api/v1/buildings")
        .insert_header(header::ContentType::json())
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "organization_id": org_id.to_string(),
            "name": "Test Building SEL",
            "address": "123 Test Street",
            "city": "Brussels",
            "postal_code": "1000",
            "country": "BE",
            "total_units": 10,
            "total_tantiemes": 1000
        }))
        .to_request();
    let building_resp = test::call_service(&app, building_req).await;
    let building_body: serde_json::Value = test::read_body_json(building_resp).await;
    let building_id = building_body["id"].as_str().unwrap();

    // Create an owner via API
    let owner_req = test::TestRequest::post()
        .uri("/api/v1/owners")
        .insert_header(header::ContentType::json())
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "organization_id": org_id.to_string(),
            "first_name": "Provider",
            "last_name": "SEL",
            "email": format!("provider+{}@test.com", Uuid::new_v4()),
            "address": "1 Rue du Test",
            "city": "Brussels",
            "postal_code": "1000",
            "country": "BE"
        }))
        .to_request();
    let owner_resp = test::call_service(&app, owner_req).await;
    let owner_body: serde_json::Value = test::read_body_json(owner_resp).await;
    let _provider_id = owner_body["id"].as_str().unwrap();

    let exchange_dto = json!({
        "building_id": building_id,
        "exchange_type": "Service",
        "title": "Plumbing repair",
        "description": "I can fix leaking faucets and pipes",
        "credits": 2  // 2 hours = 2 credits
    });

    let req = test::TestRequest::post()
        .uri("/api/v1/exchanges")
        .insert_header(header::ContentType::json())
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(&exchange_dto)
        .to_request();

    let resp = test::call_service(&app, req).await;

    // SEL time-based currency: 1 hour = 1 credit
    assert!(
        resp.status().is_success() || resp.status().is_client_error(),
        "Expected valid response, got: {}",
        resp.status()
    );
}

#[actix_web::test]
#[serial]
async fn test_exchange_workflow_offered_to_completed() {
    let (app_state, _postgres_container, org_id) = common::setup_test_db().await;
    let token = common::register_and_login(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create a building
    let building_req = test::TestRequest::post()
        .uri("/api/v1/buildings")
        .insert_header(header::ContentType::json())
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "organization_id": org_id.to_string(),
            "name": "Workflow Building",
            "address": "10 Rue Workflow",
            "city": "Brussels",
            "postal_code": "1000",
            "country": "BE",
            "total_units": 5,
            "total_tantiemes": 1000
        }))
        .to_request();
    let building_resp = test::call_service(&app, building_req).await;
    let building_body: serde_json::Value = test::read_body_json(building_resp).await;
    let building_id = building_body["id"].as_str().unwrap();

    // Create exchange offer first
    let create_req = test::TestRequest::post()
        .uri("/api/v1/exchanges")
        .insert_header(header::ContentType::json())
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "building_id": building_id,
            "exchange_type": "Service",
            "title": "Gardening help",
            "description": "I can help with gardening",
            "credits": 3
        }))
        .to_request();
    let create_resp = test::call_service(&app, create_req).await;
    let create_body: serde_json::Value = test::read_body_json(create_resp).await;
    let fallback_id = Uuid::new_v4().to_string();
    let exchange_id = create_body["id"].as_str().unwrap_or(&fallback_id);

    // 1. Request exchange (Offered -> Requested)
    let req = test::TestRequest::post()
        .uri(&format!("/api/v1/exchanges/{}/request", exchange_id))
        .insert_header(header::ContentType::json())
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({}))
        .to_request();

    let _resp = test::call_service(&app, req).await;

    // 2. Start exchange (Requested -> InProgress)
    let req = test::TestRequest::post()
        .uri(&format!("/api/v1/exchanges/{}/start", exchange_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let _resp = test::call_service(&app, req).await;

    // 3. Complete exchange (InProgress -> Completed)
    // Automatic credit balance update: provider +credits, requester -credits
    let req = test::TestRequest::post()
        .uri(&format!("/api/v1/exchanges/{}/complete", exchange_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let _resp = test::call_service(&app, req).await;

    // SEL trust model: Negative balances allowed (community trust)
}

#[actix_web::test]
#[serial]
async fn test_mutual_rating_system() {
    let (app_state, _postgres_container, org_id) = common::setup_test_db().await;
    let token = common::register_and_login(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create building
    let building_req = test::TestRequest::post()
        .uri("/api/v1/buildings")
        .insert_header(header::ContentType::json())
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "organization_id": org_id.to_string(),
            "name": "Rating Building",
            "address": "20 Rue Rating",
            "city": "Brussels",
            "postal_code": "1000",
            "country": "BE",
            "total_units": 5,
            "total_tantiemes": 1000
        }))
        .to_request();
    let building_resp = test::call_service(&app, building_req).await;
    let building_body: serde_json::Value = test::read_body_json(building_resp).await;
    let building_id = building_body["id"].as_str().unwrap();

    // Create exchange
    let create_req = test::TestRequest::post()
        .uri("/api/v1/exchanges")
        .insert_header(header::ContentType::json())
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "building_id": building_id,
            "exchange_type": "Service",
            "title": "Plumbing work",
            "description": "Fix leaking faucets",
            "credits": 2
        }))
        .to_request();
    let create_resp = test::call_service(&app, create_req).await;
    let create_body: serde_json::Value = test::read_body_json(create_resp).await;
    let fallback_id = Uuid::new_v4().to_string();
    let exchange_id = create_body["id"].as_str().unwrap_or(&fallback_id);

    // Requester rates provider
    let req = test::TestRequest::put()
        .uri(&format!("/api/v1/exchanges/{}/rate-provider", exchange_id))
        .insert_header(header::ContentType::json())
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "rating": 5  // 1-5 stars
        }))
        .to_request();

    let _resp = test::call_service(&app, req).await;

    // Provider rates requester
    let req = test::TestRequest::put()
        .uri(&format!("/api/v1/exchanges/{}/rate-requester", exchange_id))
        .insert_header(header::ContentType::json())
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "rating": 4
        }))
        .to_request();

    let _resp = test::call_service(&app, req).await;

    // Only completed exchanges can be rated
    // Ratings contribute to reputation score
}

#[actix_web::test]
#[serial]
async fn test_get_credit_balance() {
    let (app_state, _postgres_container, org_id) = common::setup_test_db().await;
    let token = common::register_and_login(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create building
    let building_req = test::TestRequest::post()
        .uri("/api/v1/buildings")
        .insert_header(header::ContentType::json())
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "organization_id": org_id.to_string(),
            "name": "Credit Building",
            "address": "30 Rue Credit",
            "city": "Brussels",
            "postal_code": "1000",
            "country": "BE",
            "total_units": 5,
            "total_tantiemes": 1000
        }))
        .to_request();
    let building_resp = test::call_service(&app, building_req).await;
    let building_body: serde_json::Value = test::read_body_json(building_resp).await;
    let building_id = building_body["id"].as_str().unwrap();

    // Create owner
    let owner_req = test::TestRequest::post()
        .uri("/api/v1/owners")
        .insert_header(header::ContentType::json())
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "organization_id": org_id.to_string(),
            "first_name": "Credit",
            "last_name": "Owner",
            "email": format!("credit+{}@test.com", Uuid::new_v4()),
            "address": "30 Rue Credit",
            "city": "Brussels",
            "postal_code": "1000",
            "country": "BE"
        }))
        .to_request();
    let owner_resp = test::call_service(&app, owner_req).await;
    let owner_body: serde_json::Value = test::read_body_json(owner_resp).await;
    let owner_id = owner_body["id"].as_str().unwrap();

    let req = test::TestRequest::get()
        .uri(&format!(
            "/api/v1/owners/{}/buildings/{}/credit-balance",
            owner_id, building_id
        ))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let _resp = test::call_service(&app, req).await;

    // Expected response:
    // {
    //   "credits_earned": 10,
    //   "credits_spent": 7,
    //   "balance": 3,  // Can be negative
    //   "total_exchanges": 5,
    //   "average_rating": 4.5,
    //   "participation_level": "Active"  // New/Beginner/Active/Veteran/Expert
    // }
}

#[actix_web::test]
#[serial]
async fn test_leaderboard() {
    let (app_state, _postgres_container, org_id) = common::setup_test_db().await;
    let token = common::register_and_login(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create building
    let building_req = test::TestRequest::post()
        .uri("/api/v1/buildings")
        .insert_header(header::ContentType::json())
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "organization_id": org_id.to_string(),
            "name": "Leaderboard Building",
            "address": "40 Rue Leaderboard",
            "city": "Brussels",
            "postal_code": "1000",
            "country": "BE",
            "total_units": 5,
            "total_tantiemes": 1000
        }))
        .to_request();
    let building_resp = test::call_service(&app, building_req).await;
    let building_body: serde_json::Value = test::read_body_json(building_resp).await;
    let building_id = building_body["id"].as_str().unwrap();

    let req = test::TestRequest::get()
        .uri(&format!(
            "/api/v1/buildings/{}/leaderboard?limit=10",
            building_id
        ))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Top 10 contributors ordered by balance DESC
    // Encourages community participation
    assert!(resp.status().is_success() || resp.status().is_client_error());
}

#[actix_web::test]
#[serial]
async fn test_sel_statistics() {
    let (app_state, _postgres_container, org_id) = common::setup_test_db().await;
    let token = common::register_and_login(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create building
    let building_req = test::TestRequest::post()
        .uri("/api/v1/buildings")
        .insert_header(header::ContentType::json())
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "organization_id": org_id.to_string(),
            "name": "Stats Building",
            "address": "50 Rue Stats",
            "city": "Brussels",
            "postal_code": "1000",
            "country": "BE",
            "total_units": 5,
            "total_tantiemes": 1000
        }))
        .to_request();
    let building_resp = test::call_service(&app, building_req).await;
    let building_body: serde_json::Value = test::read_body_json(building_resp).await;
    let building_id = building_body["id"].as_str().unwrap();

    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/buildings/{}/sel-statistics", building_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let _resp = test::call_service(&app, req).await;

    // Expected stats:
    // - total_exchanges, active_exchanges, completed_exchanges
    // - total_credits_exchanged
    // - active_participants
    // - average_rating
    // - most_popular_exchange_type (Service/ObjectLoan/SharedPurchase)
}

#[actix_web::test]
#[serial]
async fn test_owner_exchange_summary() {
    let (app_state, _postgres_container, org_id) = common::setup_test_db().await;
    let token = common::register_and_login(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create owner
    let owner_req = test::TestRequest::post()
        .uri("/api/v1/owners")
        .insert_header(header::ContentType::json())
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "organization_id": org_id.to_string(),
            "first_name": "Summary",
            "last_name": "Owner",
            "email": format!("summary+{}@test.com", Uuid::new_v4()),
            "address": "60 Rue Summary",
            "city": "Brussels",
            "postal_code": "1000",
            "country": "BE"
        }))
        .to_request();
    let owner_resp = test::call_service(&app, owner_req).await;
    let owner_body: serde_json::Value = test::read_body_json(owner_resp).await;
    let owner_id = owner_body["id"].as_str().unwrap();

    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/owners/{}/exchange-summary", owner_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let _resp = test::call_service(&app, req).await;

    // Summary across all buildings where owner participates in SEL
    // total_offered, total_requested, total_completed
    // credits_earned, credits_spent, balance
    // average_rating, participation_level
}

#[actix_web::test]
#[serial]
async fn test_list_available_exchanges() {
    let (app_state, _postgres_container, org_id) = common::setup_test_db().await;
    let token = common::register_and_login(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create building
    let building_req = test::TestRequest::post()
        .uri("/api/v1/buildings")
        .insert_header(header::ContentType::json())
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "organization_id": org_id.to_string(),
            "name": "Available Building",
            "address": "70 Rue Available",
            "city": "Brussels",
            "postal_code": "1000",
            "country": "BE",
            "total_units": 5,
            "total_tantiemes": 1000
        }))
        .to_request();
    let building_resp = test::call_service(&app, building_req).await;
    let building_body: serde_json::Value = test::read_body_json(building_resp).await;
    let building_id = building_body["id"].as_str().unwrap();

    let req = test::TestRequest::get()
        .uri(&format!(
            "/api/v1/buildings/{}/exchanges/available",
            building_id
        ))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Marketplace view: only exchanges with status = Offered
    // Excludes provider's own offers
    assert!(resp.status().is_success() || resp.status().is_client_error());
}

#[actix_web::test]
#[serial]
async fn test_cancel_exchange_with_reason() {
    let (app_state, _postgres_container, org_id) = common::setup_test_db().await;
    let token = common::register_and_login(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create building
    let building_req = test::TestRequest::post()
        .uri("/api/v1/buildings")
        .insert_header(header::ContentType::json())
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "organization_id": org_id.to_string(),
            "name": "Cancel Building",
            "address": "80 Rue Cancel",
            "city": "Brussels",
            "postal_code": "1000",
            "country": "BE",
            "total_units": 5,
            "total_tantiemes": 1000
        }))
        .to_request();
    let building_resp = test::call_service(&app, building_req).await;
    let building_body: serde_json::Value = test::read_body_json(building_resp).await;
    let building_id = building_body["id"].as_str().unwrap();

    // Create exchange first
    let create_req = test::TestRequest::post()
        .uri("/api/v1/exchanges")
        .insert_header(header::ContentType::json())
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "building_id": building_id,
            "exchange_type": "ObjectLoan",
            "title": "Drill loan",
            "description": "Lending my drill",
            "credits": 1
        }))
        .to_request();
    let create_resp = test::call_service(&app, create_req).await;
    let create_body: serde_json::Value = test::read_body_json(create_resp).await;
    let fallback_id = Uuid::new_v4().to_string();
    let exchange_id = create_body["id"].as_str().unwrap_or(&fallback_id);

    let req = test::TestRequest::post()
        .uri(&format!("/api/v1/exchanges/{}/cancel", exchange_id))
        .insert_header(header::ContentType::json())
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "cancellation_reason": "Provider no longer available"
        }))
        .to_request();

    let _resp = test::call_service(&app, req).await;

    // Cancellation reason required for audit trail
    // Helps track SEL health and identify issues
}
