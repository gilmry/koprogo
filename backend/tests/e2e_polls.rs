// E2E tests for Poll System HTTP endpoints (Issue #51)
// Tests focus on HTTP layer: endpoints, auth, JSON serialization
// Covers Belgian consultation polls between general assemblies (Art. 577-8/4 §4 Code Civil)

mod common;

use actix_web::http::header;
use actix_web::{test, App};
use chrono::{Duration, Utc};
use koprogo_api::application::dto::*;
use koprogo_api::domain::entities::UnitType;
use koprogo_api::infrastructure::web::configure_routes;
use koprogo_api::infrastructure::web::AppState;
use serde_json::json;
use serial_test::serial;
use uuid::Uuid;

// ==================== Helper ====================

/// Create a building with at least one unit+owner so Poll::new() passes total_eligible_voters > 0
async fn create_test_building_for_polls(
    app_state: &actix_web::web::Data<AppState>,
    org_id: Uuid,
) -> String {
    let dto = CreateBuildingDto {
        organization_id: org_id.to_string(),
        name: format!("Polls Test Building {}", Uuid::new_v4()),
        address: "789 Poll Street".to_string(),
        city: "Brussels".to_string(),
        postal_code: "1000".to_string(),
        country: "Belgium".to_string(),
        total_units: 4,
        total_tantiemes: Some(1000),
        construction_year: Some(2018),
    };
    let building = app_state
        .building_use_cases
        .create_building(dto)
        .await
        .expect("Failed to create building for polls");
    let building_id = Uuid::parse_str(&building.id).unwrap();

    // Create a unit so find_active_by_building returns > 0 owners
    let unit = app_state
        .unit_use_cases
        .create_unit(CreateUnitDto {
            organization_id: org_id.to_string(),
            building_id: building.id.clone(),
            unit_number: "A1".to_string(),
            unit_type: UnitType::Apartment,
            floor: Some(1),
            surface_area: 75.0,
            quota: 1.0,
        })
        .await
        .expect("Failed to create unit for polls");
    let unit_id = Uuid::parse_str(&unit.id).unwrap();

    // Create an owner
    let owner = app_state
        .owner_use_cases
        .create_owner(CreateOwnerDto {
            organization_id: org_id.to_string(),
            first_name: "Poll".to_string(),
            last_name: "Owner".to_string(),
            email: format!("poll-owner-{}@test.com", Uuid::new_v4()),
            phone: None,
            address: "789 Poll Street A1".to_string(),
            city: "Brussels".to_string(),
            postal_code: "1000".to_string(),
            country: "Belgium".to_string(),
            user_id: None,
        })
        .await
        .expect("Failed to create owner for polls");
    let owner_id = Uuid::parse_str(&owner.id).unwrap();

    // Link owner to unit (100% ownership)
    app_state
        .unit_owner_use_cases
        .add_owner_to_unit(unit_id, owner_id, 1.0, true)
        .await
        .expect("Failed to assign owner to unit for polls");

    building_id.to_string()
}

// ==================== Poll CRUD Tests ====================

#[actix_web::test]
#[serial]
async fn test_polls_create_yesno() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = common::register_and_login(&app_state, org_id).await;
    let building_id = create_test_building_for_polls(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let ends_at = (Utc::now() + Duration::days(7)).to_rfc3339();

    let req = test::TestRequest::post()
        .uri("/api/v1/polls")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "building_id": building_id,
            "title": "Should we repaint the lobby?",
            "description": "Vote on repainting the main lobby in blue",
            "poll_type": "yes_no",
            "options": [
                { "option_text": "Yes", "display_order": 0 },
                { "option_text": "No", "display_order": 1 }
            ],
            "is_anonymous": false,
            "allow_multiple_votes": false,
            "ends_at": ends_at
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201, "Should create YesNo poll successfully");

    let poll: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(poll["title"], "Should we repaint the lobby?");
    assert_eq!(poll["building_id"], building_id);
}

#[actix_web::test]
#[serial]
async fn test_polls_get() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = common::register_and_login(&app_state, org_id).await;
    let building_id = create_test_building_for_polls(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let ends_at = (Utc::now() + Duration::days(7)).to_rfc3339();

    // Create a poll first
    let create_req = test::TestRequest::post()
        .uri("/api/v1/polls")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "building_id": building_id,
            "title": "Get poll test",
            "poll_type": "yes_no",
            "options": [
                { "option_text": "Yes", "display_order": 0 },
                { "option_text": "No", "display_order": 1 }
            ],
            "is_anonymous": false,
            "allow_multiple_votes": false,
            "ends_at": ends_at
        }))
        .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    assert_eq!(create_resp.status(), 201);
    let created: serde_json::Value = test::read_body_json(create_resp).await;
    let poll_id = created["id"].as_str().unwrap();

    // Get the poll by ID
    let get_req = test::TestRequest::get()
        .uri(&format!("/api/v1/polls/{}", poll_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let get_resp = test::call_service(&app, get_req).await;
    assert_eq!(get_resp.status(), 200, "Should get poll by ID");

    let poll: serde_json::Value = test::read_body_json(get_resp).await;
    assert_eq!(poll["id"], poll_id);
    assert_eq!(poll["title"], "Get poll test");
}

#[actix_web::test]
#[serial]
async fn test_polls_list_by_building() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = common::register_and_login(&app_state, org_id).await;
    let building_id = create_test_building_for_polls(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let ends_at = (Utc::now() + Duration::days(7)).to_rfc3339();

    // Create a poll in this building
    let create_req = test::TestRequest::post()
        .uri("/api/v1/polls")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "building_id": building_id,
            "title": "List by building test poll",
            "poll_type": "yes_no",
            "options": [
                { "option_text": "Yes", "display_order": 0 },
                { "option_text": "No", "display_order": 1 }
            ],
            "is_anonymous": false,
            "allow_multiple_votes": false,
            "ends_at": ends_at
        }))
        .to_request();
    let _ = test::call_service(&app, create_req).await;

    // List polls filtered by building_id
    let list_req = test::TestRequest::get()
        .uri(&format!("/api/v1/polls?building_id={}", building_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let list_resp = test::call_service(&app, list_req).await;
    assert_eq!(list_resp.status(), 200, "Should list polls for building");

    let body: serde_json::Value = test::read_body_json(list_resp).await;
    // Response may be paginated with { polls: [...], total: N } or a plain array
    let has_polls = body.get("polls").is_some() || body.is_array();
    assert!(has_polls, "Response should contain polls data");
}

#[actix_web::test]
#[serial]
async fn test_polls_publish() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = common::register_and_login(&app_state, org_id).await;
    let building_id = create_test_building_for_polls(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let ends_at = (Utc::now() + Duration::days(7)).to_rfc3339();

    // Create poll (starts as Draft)
    let create_req = test::TestRequest::post()
        .uri("/api/v1/polls")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "building_id": building_id,
            "title": "Publish test poll",
            "poll_type": "yes_no",
            "options": [
                { "option_text": "Yes", "display_order": 0 },
                { "option_text": "No", "display_order": 1 }
            ],
            "is_anonymous": false,
            "allow_multiple_votes": false,
            "ends_at": ends_at
        }))
        .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    assert_eq!(create_resp.status(), 201);
    let created: serde_json::Value = test::read_body_json(create_resp).await;
    let poll_id = created["id"].as_str().unwrap();

    // Publish the poll (Draft → Active)
    let publish_req = test::TestRequest::post()
        .uri(&format!("/api/v1/polls/{}/publish", poll_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let publish_resp = test::call_service(&app, publish_req).await;
    assert_eq!(
        publish_resp.status(),
        200,
        "Should publish poll successfully"
    );

    let published: serde_json::Value = test::read_body_json(publish_resp).await;
    assert_eq!(
        published["status"], "active",
        "Poll status should be active after publishing"
    );
}

#[actix_web::test]
#[serial]
async fn test_polls_close() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = common::register_and_login(&app_state, org_id).await;
    let building_id = create_test_building_for_polls(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let ends_at = (Utc::now() + Duration::days(7)).to_rfc3339();

    // Create + publish a poll
    let create_req = test::TestRequest::post()
        .uri("/api/v1/polls")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "building_id": building_id,
            "title": "Close test poll",
            "poll_type": "yes_no",
            "options": [
                { "option_text": "Yes", "display_order": 0 },
                { "option_text": "No", "display_order": 1 }
            ],
            "is_anonymous": false,
            "allow_multiple_votes": false,
            "ends_at": ends_at
        }))
        .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    assert_eq!(create_resp.status(), 201);
    let created: serde_json::Value = test::read_body_json(create_resp).await;
    let poll_id = created["id"].as_str().unwrap();

    // Publish first
    let publish_req = test::TestRequest::post()
        .uri(&format!("/api/v1/polls/{}/publish", poll_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();
    let _ = test::call_service(&app, publish_req).await;

    // Close the poll (Active → Closed)
    let close_req = test::TestRequest::post()
        .uri(&format!("/api/v1/polls/{}/close", poll_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let close_resp = test::call_service(&app, close_req).await;
    assert_eq!(close_resp.status(), 200, "Should close poll successfully");

    let closed: serde_json::Value = test::read_body_json(close_resp).await;
    assert_eq!(
        closed["status"], "closed",
        "Poll status should be closed after closing"
    );
}

#[actix_web::test]
#[serial]
async fn test_polls_cast_vote() {
    use koprogo_api::application::dto::RegisterRequest;

    let (app_state, _container, org_id) = common::setup_test_db().await;

    // Register a user and get their user_id
    let email = format!("poll-voter-{}@test.com", Uuid::new_v4());
    let reg = RegisterRequest {
        email: email.clone(),
        password: "Passw0rd!".to_string(),
        first_name: "Poll".to_string(),
        last_name: "Voter".to_string(),
        role: "superadmin".to_string(),
        organization_id: Some(org_id),
    };
    let login_resp = app_state
        .auth_use_cases
        .register(reg)
        .await
        .expect("register voter");
    let token = login_resp.token;
    let user_id = login_resp.user.id;

    // Create building with a unit
    let dto = koprogo_api::application::dto::CreateBuildingDto {
        organization_id: org_id.to_string(),
        name: format!("Polls Vote Building {}", Uuid::new_v4()),
        address: "1 Vote Street".to_string(),
        city: "Brussels".to_string(),
        postal_code: "1000".to_string(),
        country: "Belgium".to_string(),
        total_units: 2,
        total_tantiemes: Some(1000),
        construction_year: Some(2018),
    };
    let building = app_state
        .building_use_cases
        .create_building(dto)
        .await
        .expect("create building for vote test");
    let building_id = building.id.clone();

    let unit = app_state
        .unit_use_cases
        .create_unit(koprogo_api::application::dto::CreateUnitDto {
            organization_id: org_id.to_string(),
            building_id: building_id.clone(),
            unit_number: "V1".to_string(),
            unit_type: UnitType::Apartment,
            floor: Some(1),
            surface_area: 60.0,
            quota: 1.0,
        })
        .await
        .expect("create unit for vote test");
    let unit_id = Uuid::parse_str(&unit.id).unwrap();

    // Insert an owner with id == user_id so cast_vote authorization passes
    sqlx::query(
        r#"INSERT INTO owners (id, organization_id, user_id, first_name, last_name, email,
           address, city, postal_code, country, created_at, updated_at)
           VALUES ($1, $2, $1, 'Poll', 'Voter', $3, '1 Vote Street', 'Brussels', '1000', 'Belgium', NOW(), NOW())"#,
    )
    .bind(user_id)
    .bind(org_id)
    .bind(format!("poll-owner-{}@test.com", Uuid::new_v4()))
    .execute(&app_state.pool)
    .await
    .expect("insert owner for vote test");

    app_state
        .unit_owner_use_cases
        .add_owner_to_unit(unit_id, user_id, 1.0, true)
        .await
        .expect("link owner to unit for vote test");

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let ends_at = (Utc::now() + Duration::days(7)).to_rfc3339();

    // Create and publish poll
    let create_req = test::TestRequest::post()
        .uri("/api/v1/polls")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "building_id": building_id,
            "title": "Vote test poll",
            "poll_type": "yes_no",
            "options": [
                { "option_text": "Yes", "display_order": 0 },
                { "option_text": "No", "display_order": 1 }
            ],
            "is_anonymous": false,
            "allow_multiple_votes": false,
            "ends_at": ends_at
        }))
        .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    assert_eq!(create_resp.status(), 201);
    let created: serde_json::Value = test::read_body_json(create_resp).await;
    let poll_id = created["id"].as_str().unwrap();

    // Get option IDs from the created poll
    let options = created["options"]
        .as_array()
        .expect("Poll should have options");
    let yes_option_id = options[0]["id"].as_str().unwrap();

    // Publish the poll
    let publish_req = test::TestRequest::post()
        .uri(&format!("/api/v1/polls/{}/publish", poll_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();
    let _ = test::call_service(&app, publish_req).await;

    // Cast a vote — endpoint is POST /polls/vote with poll_id in body
    let vote_req = test::TestRequest::post()
        .uri("/api/v1/polls/vote")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "poll_id": poll_id,
            "selected_option_ids": [yes_option_id]
        }))
        .to_request();

    let vote_resp = test::call_service(&app, vote_req).await;
    assert_eq!(vote_resp.status(), 201, "Should cast vote successfully");
}

#[actix_web::test]
#[serial]
async fn test_polls_get_results() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = common::register_and_login(&app_state, org_id).await;
    let building_id = create_test_building_for_polls(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let ends_at = (Utc::now() + Duration::days(7)).to_rfc3339();

    // Create and publish a poll
    let create_req = test::TestRequest::post()
        .uri("/api/v1/polls")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "building_id": building_id,
            "title": "Results test poll",
            "poll_type": "yes_no",
            "options": [
                { "option_text": "Yes", "display_order": 0 },
                { "option_text": "No", "display_order": 1 }
            ],
            "is_anonymous": false,
            "allow_multiple_votes": false,
            "ends_at": ends_at
        }))
        .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    assert_eq!(create_resp.status(), 201);
    let created: serde_json::Value = test::read_body_json(create_resp).await;
    let poll_id = created["id"].as_str().unwrap();

    // Get poll results
    let results_req = test::TestRequest::get()
        .uri(&format!("/api/v1/polls/{}/results", poll_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let results_resp = test::call_service(&app, results_req).await;
    assert_eq!(
        results_resp.status(),
        200,
        "Should get poll results successfully"
    );

    let results: serde_json::Value = test::read_body_json(results_resp).await;
    assert!(
        results.get("poll_id").is_some() || results.get("total_votes_cast").is_some(),
        "Results should contain poll_id or total_votes_cast field"
    );
}

#[actix_web::test]
#[serial]
async fn test_polls_unauthorized() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = common::register_and_login(&app_state, org_id).await;
    let building_id = create_test_building_for_polls(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let ends_at = (Utc::now() + Duration::days(7)).to_rfc3339();

    // Create a poll to get a valid ID
    let create_req = test::TestRequest::post()
        .uri("/api/v1/polls")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "building_id": building_id,
            "title": "Unauthorized test poll",
            "poll_type": "yes_no",
            "options": [
                { "option_text": "Yes", "display_order": 0 },
                { "option_text": "No", "display_order": 1 }
            ],
            "is_anonymous": false,
            "allow_multiple_votes": false,
            "ends_at": ends_at
        }))
        .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    assert_eq!(create_resp.status(), 201);
    let created: serde_json::Value = test::read_body_json(create_resp).await;
    let poll_id = created["id"].as_str().unwrap();

    // Request without token
    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/polls/{}", poll_id))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.status(),
        401,
        "Should return 401 when no auth token provided"
    );
}
