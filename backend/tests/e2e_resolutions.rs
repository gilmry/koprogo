// E2E tests for resolution/voting HTTP endpoints (Issue #46)
// Tests focus on HTTP layer: endpoints, auth, JSON serialization
// BDD tests cover business scenarios

mod common;

use actix_web::http::header;
use actix_web::{test, App};
use chrono::{Duration, Utc};
use koprogo_api::application::dto::*;
use koprogo_api::domain::entities::{MeetingType, UnitType};
use koprogo_api::infrastructure::web::configure_routes;
use koprogo_api::infrastructure::web::AppState;
use serde_json::json;
use serial_test::serial;
use uuid::Uuid;

/// Setup function shared across all resolution E2E tests
async fn setup_app() -> (
    actix_web::web::Data<AppState>,
    testcontainers_modules::testcontainers::ContainerAsync<
        testcontainers_modules::postgres::Postgres,
    >,
    Uuid,
) {
    common::setup_test_db().await
}

/// Helper: Create test fixtures (organization, building, meeting, owners, units)
async fn create_test_fixtures(
    app_state: &actix_web::web::Data<AppState>,
    org_id: Uuid,
) -> (String, Uuid, Uuid, Uuid, Uuid, Uuid, Uuid) {
    // 1. Register user and get token
    let email = format!("resolution-test-{}@example.com", Uuid::new_v4());
    let register_req = RegisterRequest {
        email: email.clone(),
        password: "SecurePass123!".to_string(),
        first_name: "Resolution".to_string(),
        last_name: "Tester".to_string(),
        role: "superadmin".to_string(),
        organization_id: Some(org_id),
    };

    let _user = app_state
        .auth_use_cases
        .register(register_req)
        .await
        .expect("Failed to register user");

    let login_req = LoginRequest {
        email,
        password: "SecurePass123!".to_string(),
    };

    let login = app_state
        .auth_use_cases
        .login(login_req)
        .await
        .expect("Failed to login");

    let token = login.token;

    // 2. Create building
    let building_dto = CreateBuildingDto {
        organization_id: org_id.to_string(),
        name: format!("Test Building Resolution {}", Uuid::new_v4()),
        address: "456 Vote Ave".to_string(),
        city: "Brussels".to_string(),
        postal_code: "1000".to_string(),
        country: "Belgium".to_string(),
        total_units: 3,
        total_tantiemes: Some(1000),
        construction_year: Some(2020),
    };

    let building = app_state
        .building_use_cases
        .create_building(building_dto)
        .await
        .expect("Failed to create building");

    let building_id = Uuid::parse_str(&building.id).expect("Failed to parse building id");

    // 3. Create meeting
    let meeting_req = CreateMeetingRequest {
        organization_id: org_id,
        building_id,
        meeting_type: MeetingType::Ordinary,
        title: "Test AG Resolution".to_string(),
        description: Some("Testing resolution voting".to_string()),
        scheduled_date: Utc::now() + Duration::days(7),
        location: "Main Hall".to_string(),
    };

    let meeting = app_state
        .meeting_use_cases
        .create_meeting(meeting_req)
        .await
        .expect("Failed to create meeting");

    // 4. Create owners
    let owner1_dto = CreateOwnerDto {
        organization_id: org_id.to_string(),
        first_name: "Owner".to_string(),
        last_name: "One".to_string(),
        email: format!("owner1-{}@example.com", Uuid::new_v4()),
        phone: Some("+32 2 111 1111".to_string()),
        address: "123 Test St".to_string(),
        city: "Brussels".to_string(),
        postal_code: "1000".to_string(),
        country: "Belgium".to_string(),
    };

    let owner1 = app_state
        .owner_use_cases
        .create_owner(owner1_dto)
        .await
        .expect("Failed to create owner 1");

    let owner1_id = Uuid::parse_str(&owner1.id).expect("Failed to parse owner1 id");

    let owner2_dto = CreateOwnerDto {
        organization_id: org_id.to_string(),
        first_name: "Owner".to_string(),
        last_name: "Two".to_string(),
        email: format!("owner2-{}@example.com", Uuid::new_v4()),
        phone: Some("+32 2 222 2222".to_string()),
        address: "456 Test St".to_string(),
        city: "Brussels".to_string(),
        postal_code: "1000".to_string(),
        country: "Belgium".to_string(),
    };

    let owner2 = app_state
        .owner_use_cases
        .create_owner(owner2_dto)
        .await
        .expect("Failed to create owner 2");

    let owner2_id = Uuid::parse_str(&owner2.id).expect("Failed to parse owner2 id");

    // 5. Create units
    let unit1_dto = CreateUnitDto {
        organization_id: org_id.to_string(),
        building_id: building_id.to_string(),
        unit_number: "A101".to_string(),
        floor: Some(1),
        surface_area: 75.0,
        unit_type: UnitType::Apartment,
        quota: 0.4,
    };

    let unit1 = app_state
        .unit_use_cases
        .create_unit(unit1_dto)
        .await
        .expect("Failed to create unit 1");

    let unit1_id = Uuid::parse_str(&unit1.id).expect("Failed to parse unit1 id");

    let unit2_dto = CreateUnitDto {
        organization_id: org_id.to_string(),
        building_id: building_id.to_string(),
        unit_number: "A102".to_string(),
        floor: Some(1),
        surface_area: 85.0,
        unit_type: UnitType::Apartment,
        quota: 0.6,
    };

    let unit2 = app_state
        .unit_use_cases
        .create_unit(unit2_dto)
        .await
        .expect("Failed to create unit 2");

    let unit2_id = Uuid::parse_str(&unit2.id).expect("Failed to parse unit2 id");

    // 6. Assign owners to units with voting power (milliemes)
    app_state
        .unit_owner_use_cases
        .add_owner_to_unit(unit1_id, owner1_id, 0.4, true) // 400 milliemes (40%)
        .await
        .expect("Failed to add owner 1 to unit 1");

    app_state
        .unit_owner_use_cases
        .add_owner_to_unit(unit2_id, owner2_id, 0.6, true) // 600 milliemes (60%)
        .await
        .expect("Failed to add owner 2 to unit 2");

    (
        token,
        org_id,
        building_id,
        meeting.id,
        owner1_id,
        owner2_id,
        unit1_id,
    )
}

/// Helper: Create an additional unit for a second owner in tests that need it
async fn create_extra_unit(
    app_state: &actix_web::web::Data<AppState>,
    org_id: Uuid,
    building_id: Uuid,
    owner_id: Uuid,
    unit_number: &str,
) -> Uuid {
    let unit_dto = CreateUnitDto {
        organization_id: org_id.to_string(),
        building_id: building_id.to_string(),
        unit_number: unit_number.to_string(),
        floor: Some(1),
        surface_area: 100.0,
        unit_type: UnitType::Apartment,
        quota: 0.6,
    };

    let unit = app_state
        .unit_use_cases
        .create_unit(unit_dto)
        .await
        .expect("Failed to create extra unit");

    let unit_id = Uuid::parse_str(&unit.id).expect("Failed to parse extra unit id");

    app_state
        .unit_owner_use_cases
        .add_owner_to_unit(unit_id, owner_id, 0.6, true)
        .await
        .expect("Failed to add owner to extra unit");

    unit_id
}

// ==================== Resolution Tests ====================

#[actix_web::test]
#[serial]
async fn test_create_resolution_success() {
    let (app_state, _container, org_id) = setup_app().await;
    let (token, _org_id, _building_id, meeting_id, _owner1_id, _owner2_id, _unit1_id) =
        create_test_fixtures(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::post()
        .uri(&format!("/api/v1/meetings/{}/resolutions", meeting_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "meeting_id": meeting_id.to_string(),
            "title": "Approve Annual Budget",
            "description": "Vote to approve the budget for next fiscal year",
            "resolution_type": "ordinary",
            "majority_required": "simple"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201, "Should create resolution successfully");

    let resolution: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(resolution["title"], "Approve Annual Budget");
    assert_eq!(resolution["resolution_type"], "ordinary");
    assert_eq!(resolution["status"], "pending");
    assert_eq!(resolution["vote_count_pour"], 0);
    assert_eq!(resolution["vote_count_contre"], 0);
    assert_eq!(resolution["vote_count_abstention"], 0);
}

#[actix_web::test]
#[serial]
async fn test_create_resolution_without_auth_fails() {
    let (app_state, _container, org_id) = setup_app().await;
    let (_token, _org_id, _building_id, meeting_id, _owner1_id, _owner2_id, _unit1_id) =
        create_test_fixtures(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::post()
        .uri(&format!("/api/v1/meetings/{}/resolutions", meeting_id))
        .set_json(json!({
            "meeting_id": meeting_id.to_string(),
            "title": "Test Resolution",
            "description": "Should fail without auth",
            "resolution_type": "ordinary",
            "majority_required": "simple"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401, "Should require authentication");
}

#[actix_web::test]
#[serial]
async fn test_get_resolution_success() {
    let (app_state, _container, org_id) = setup_app().await;
    let (token, _org_id, _building_id, meeting_id, _owner1_id, _owner2_id, _unit1_id) =
        create_test_fixtures(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create resolution first
    let create_req = test::TestRequest::post()
        .uri(&format!("/api/v1/meetings/{}/resolutions", meeting_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "meeting_id": meeting_id.to_string(),
            "title": "Test Get Resolution",
            "description": "Resolution for testing GET endpoint",
            "resolution_type": "ordinary",
            "majority_required": "absolute"
        }))
        .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    let created: serde_json::Value = test::read_body_json(create_resp).await;
    let resolution_id = created["id"].as_str().unwrap();

    // Get resolution
    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/resolutions/{}", resolution_id))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let resolution: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(resolution["id"], resolution_id);
    assert_eq!(resolution["title"], "Test Get Resolution");
    assert_eq!(resolution["majority_required"], "absolute");
}

#[actix_web::test]
#[serial]
async fn test_get_resolution_not_found() {
    let (app_state, _container, _org_id) = setup_app().await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let fake_id = Uuid::new_v4();
    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/resolutions/{}", fake_id))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.status(),
        404,
        "Should return 404 for non-existent resolution"
    );
}

#[actix_web::test]
#[serial]
async fn test_list_meeting_resolutions() {
    let (app_state, _container, org_id) = setup_app().await;
    let (token, _org_id, _building_id, meeting_id, _owner1_id, _owner2_id, _unit1_id) =
        create_test_fixtures(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create 3 resolutions
    for i in 1..=3 {
        let req = test::TestRequest::post()
            .uri(&format!("/api/v1/meetings/{}/resolutions", meeting_id))
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
            .set_json(json!({
                "meeting_id": meeting_id.to_string(),
                "title": format!("Resolution #{}", i),
                "description": format!("Description for resolution {}", i),
                "resolution_type": "ordinary",
                "majority_required": "simple"
            }))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 201);
    }

    // List all resolutions for the meeting
    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/meetings/{}/resolutions", meeting_id))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let resolutions: serde_json::Value = test::read_body_json(resp).await;
    let resolutions_array = resolutions.as_array().unwrap();
    assert_eq!(
        resolutions_array.len(),
        3,
        "Should return all 3 resolutions"
    );
}

#[actix_web::test]
#[serial]
async fn test_delete_resolution_success() {
    let (app_state, _container, org_id) = setup_app().await;
    let (token, _org_id, _building_id, meeting_id, _owner1_id, _owner2_id, _unit1_id) =
        create_test_fixtures(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create resolution
    let create_req = test::TestRequest::post()
        .uri(&format!("/api/v1/meetings/{}/resolutions", meeting_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "meeting_id": meeting_id.to_string(),
            "title": "Resolution to Delete",
            "description": "This resolution will be deleted",
            "resolution_type": "ordinary",
            "majority_required": "simple"
        }))
        .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    let created: serde_json::Value = test::read_body_json(create_resp).await;
    let resolution_id = created["id"].as_str().unwrap();

    // Delete resolution
    let req = test::TestRequest::delete()
        .uri(&format!("/api/v1/resolutions/{}", resolution_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 204, "Should delete resolution successfully");

    // Verify deletion
    let get_req = test::TestRequest::get()
        .uri(&format!("/api/v1/resolutions/{}", resolution_id))
        .to_request();

    let get_resp = test::call_service(&app, get_req).await;
    assert_eq!(get_resp.status(), 404, "Should return 404 after deletion");
}

// ==================== Vote Tests ====================

#[actix_web::test]
#[serial]
async fn test_cast_vote_pour_success() {
    let (app_state, _container, org_id) = setup_app().await;
    let (token, _org_id, _building_id, meeting_id, owner1_id, _owner2_id, unit1_id) =
        create_test_fixtures(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create resolution
    let create_req = test::TestRequest::post()
        .uri(&format!("/api/v1/meetings/{}/resolutions", meeting_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "meeting_id": meeting_id.to_string(),
            "title": "Resolution for Voting",
            "description": "Test vote casting",
            "resolution_type": "ordinary",
            "majority_required": "simple"
        }))
        .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    let resolution: serde_json::Value = test::read_body_json(create_resp).await;
    let resolution_id = resolution["id"].as_str().unwrap();

    // Cast vote "Pour" (For)
    let req = test::TestRequest::post()
        .uri(&format!("/api/v1/resolutions/{}/vote", resolution_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "owner_id": owner1_id.to_string(),
            "unit_id": unit1_id.to_string(),
            "vote_choice": "pour",
            "voting_power": 0.4 // 400 milliemes (40%)
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201, "Should cast vote successfully");

    let vote: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(vote["vote_choice"], "pour");
    assert_eq!(vote["voting_power"], 0.4);
    assert_eq!(vote["owner_id"], owner1_id.to_string());
}

#[actix_web::test]
#[serial]
async fn test_cast_vote_contre_and_abstention() {
    let (app_state, _container, org_id) = setup_app().await;
    let (token, _org_id, building_id, meeting_id, owner1_id, owner2_id, unit1_id) =
        create_test_fixtures(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create unit 2 for second owner
    let unit2_id = create_extra_unit(&app_state, org_id, building_id, owner2_id, "A103").await;

    // Create resolution
    let create_req = test::TestRequest::post()
        .uri(&format!("/api/v1/meetings/{}/resolutions", meeting_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "meeting_id": meeting_id.to_string(),
            "title": "Resolution with Mixed Votes",
            "description": "Testing Contre and Abstention",
            "resolution_type": "ordinary",
            "majority_required": "simple"
        }))
        .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    let resolution: serde_json::Value = test::read_body_json(create_resp).await;
    let resolution_id = resolution["id"].as_str().unwrap();

    // Vote "Contre" (Against) with owner1
    let req1 = test::TestRequest::post()
        .uri(&format!("/api/v1/resolutions/{}/vote", resolution_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "owner_id": owner1_id.to_string(),
            "unit_id": unit1_id.to_string(),
            "vote_choice": "contre",
            "voting_power": 0.4
        }))
        .to_request();

    let resp1 = test::call_service(&app, req1).await;
    assert_eq!(resp1.status(), 201);
    let vote1: serde_json::Value = test::read_body_json(resp1).await;
    assert_eq!(vote1["vote_choice"], "contre");

    // Vote "Abstention" with owner2
    let req2 = test::TestRequest::post()
        .uri(&format!("/api/v1/resolutions/{}/vote", resolution_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "owner_id": owner2_id.to_string(),
            "unit_id": unit2_id.to_string(),
            "vote_choice": "abstention",
            "voting_power": 0.6
        }))
        .to_request();

    let resp2 = test::call_service(&app, req2).await;
    assert_eq!(resp2.status(), 201);
    let vote2: serde_json::Value = test::read_body_json(resp2).await;
    assert_eq!(vote2["vote_choice"], "abstention");
}

#[actix_web::test]
#[serial]
async fn test_list_resolution_votes() {
    let (app_state, _container, org_id) = setup_app().await;
    let (token, _org_id, building_id, meeting_id, owner1_id, owner2_id, unit1_id) =
        create_test_fixtures(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create unit 2
    let unit2_id = create_extra_unit(&app_state, org_id, building_id, owner2_id, "A104").await;

    // Create resolution
    let create_req = test::TestRequest::post()
        .uri(&format!("/api/v1/meetings/{}/resolutions", meeting_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "meeting_id": meeting_id.to_string(),
            "title": "Resolution with Multiple Votes",
            "description": "Test vote listing",
            "resolution_type": "ordinary",
            "majority_required": "simple"
        }))
        .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    let resolution: serde_json::Value = test::read_body_json(create_resp).await;
    let resolution_id = resolution["id"].as_str().unwrap();

    // Cast 2 votes
    for (owner_id, unit_id, choice, power) in [
        (owner1_id, unit1_id, "pour", 0.4),
        (owner2_id, unit2_id, "contre", 0.6),
    ] {
        let req = test::TestRequest::post()
            .uri(&format!("/api/v1/resolutions/{}/vote", resolution_id))
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
            .set_json(json!({
                "owner_id": owner_id.to_string(),
                "unit_id": unit_id.to_string(),
                "vote_choice": choice,
                "voting_power": power
            }))
            .to_request();

        test::call_service(&app, req).await;
    }

    // List all votes for resolution
    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/resolutions/{}/votes", resolution_id))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let votes: serde_json::Value = test::read_body_json(resp).await;
    let votes_array = votes.as_array().unwrap();
    assert_eq!(votes_array.len(), 2, "Should return all 2 votes");
}

#[actix_web::test]
#[serial]
async fn test_change_vote_success() {
    let (app_state, _container, org_id) = setup_app().await;
    let (token, _org_id, _building_id, meeting_id, owner1_id, _owner2_id, unit1_id) =
        create_test_fixtures(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create resolution
    let create_req = test::TestRequest::post()
        .uri(&format!("/api/v1/meetings/{}/resolutions", meeting_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "meeting_id": meeting_id.to_string(),
            "title": "Resolution for Vote Change",
            "description": "Test changing vote",
            "resolution_type": "ordinary",
            "majority_required": "simple"
        }))
        .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    let resolution: serde_json::Value = test::read_body_json(create_resp).await;
    let resolution_id = resolution["id"].as_str().unwrap();

    // Cast initial vote "Pour"
    let vote_req = test::TestRequest::post()
        .uri(&format!("/api/v1/resolutions/{}/vote", resolution_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "owner_id": owner1_id.to_string(),
            "unit_id": unit1_id.to_string(),
            "vote_choice": "pour",
            "voting_power": 0.4
        }))
        .to_request();

    let vote_resp = test::call_service(&app, vote_req).await;
    let vote: serde_json::Value = test::read_body_json(vote_resp).await;
    let vote_id = vote["id"].as_str().unwrap();

    // Change vote to "Contre"
    let change_req = test::TestRequest::put()
        .uri(&format!("/api/v1/votes/{}", vote_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "vote_choice": "contre"
        }))
        .to_request();

    let change_resp = test::call_service(&app, change_req).await;
    assert_eq!(change_resp.status(), 200, "Should change vote successfully");

    let updated_vote: serde_json::Value = test::read_body_json(change_resp).await;
    assert_eq!(
        updated_vote["vote_choice"], "contre",
        "Vote should be changed to Contre"
    );
}

#[actix_web::test]
#[serial]
async fn test_close_voting_simple_majority() {
    let (app_state, _container, org_id) = setup_app().await;
    let (token, _org_id, building_id, meeting_id, owner1_id, owner2_id, unit1_id) =
        create_test_fixtures(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create unit 2
    let unit2_id = create_extra_unit(&app_state, org_id, building_id, owner2_id, "A105").await;

    // Create resolution with Simple majority
    let create_req = test::TestRequest::post()
        .uri(&format!("/api/v1/meetings/{}/resolutions", meeting_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "meeting_id": meeting_id.to_string(),
            "title": "Resolution with Simple Majority",
            "description": "50% + 1 of votes cast",
            "resolution_type": "ordinary",
            "majority_required": "simple"
        }))
        .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    let resolution: serde_json::Value = test::read_body_json(create_resp).await;
    let resolution_id = resolution["id"].as_str().unwrap();

    // Cast votes: Pour wins (60% voting power)
    let vote1_req = test::TestRequest::post()
        .uri(&format!("/api/v1/resolutions/{}/vote", resolution_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "owner_id": owner1_id.to_string(),
            "unit_id": unit1_id.to_string(),
            "vote_choice": "contre",
            "voting_power": 0.4
        }))
        .to_request();

    test::call_service(&app, vote1_req).await;

    let vote2_req = test::TestRequest::post()
        .uri(&format!("/api/v1/resolutions/{}/vote", resolution_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "owner_id": owner2_id.to_string(),
            "unit_id": unit2_id.to_string(),
            "vote_choice": "pour",
            "voting_power": 0.6
        }))
        .to_request();

    test::call_service(&app, vote2_req).await;

    // Close voting
    let close_req = test::TestRequest::put()
        .uri(&format!("/api/v1/resolutions/{}/close", resolution_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "total_voting_power": 1.0 // 100% total
        }))
        .to_request();

    let close_resp = test::call_service(&app, close_req).await;
    assert_eq!(close_resp.status(), 200, "Should close voting successfully");

    let closed_resolution: serde_json::Value = test::read_body_json(close_resp).await;
    assert_eq!(
        closed_resolution["status"], "adopted",
        "Should be Adopted with Simple majority (60% Pour > 40% Contre)"
    );
    assert_eq!(closed_resolution["vote_count_pour"], 1);
    assert_eq!(closed_resolution["vote_count_contre"], 1);
    assert_eq!(closed_resolution["total_voting_power_pour"], 0.6);
}

#[actix_web::test]
#[serial]
async fn test_close_voting_absolute_majority() {
    let (app_state, _container, org_id) = setup_app().await;
    let (token, _org_id, building_id, meeting_id, _owner1_id, owner2_id, _unit1_id) =
        create_test_fixtures(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create unit 2
    let unit2_id = create_extra_unit(&app_state, org_id, building_id, owner2_id, "A106").await;

    // Create resolution with Absolute majority (50% + 1 of ALL votes)
    let create_req = test::TestRequest::post()
        .uri(&format!("/api/v1/meetings/{}/resolutions", meeting_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "meeting_id": meeting_id.to_string(),
            "title": "Resolution with Absolute Majority",
            "description": "50% + 1 of all possible votes",
            "resolution_type": "extraordinary",
            "majority_required": "absolute"
        }))
        .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    let resolution: serde_json::Value = test::read_body_json(create_resp).await;
    let resolution_id = resolution["id"].as_str().unwrap();

    // Cast only one vote "Pour" with 60% power
    let vote_req = test::TestRequest::post()
        .uri(&format!("/api/v1/resolutions/{}/vote", resolution_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "owner_id": owner2_id.to_string(),
            "unit_id": unit2_id.to_string(),
            "vote_choice": "pour",
            "voting_power": 0.6
        }))
        .to_request();

    test::call_service(&app, vote_req).await;

    // Close voting with total_voting_power = 1.0 (100%)
    let close_req = test::TestRequest::put()
        .uri(&format!("/api/v1/resolutions/{}/close", resolution_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "total_voting_power": 1.0 // 100% total
        }))
        .to_request();

    let close_resp = test::call_service(&app, close_req).await;
    assert_eq!(close_resp.status(), 200);

    let closed_resolution: serde_json::Value = test::read_body_json(close_resp).await;
    assert_eq!(
        closed_resolution["status"], "adopted",
        "Should be Adopted with Absolute majority (60% > 50%)"
    );
}

#[actix_web::test]
#[serial]
async fn test_close_voting_qualified_majority() {
    let (app_state, _container, org_id) = setup_app().await;
    let (token, _org_id, building_id, meeting_id, owner1_id, owner2_id, unit1_id) =
        create_test_fixtures(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create unit 2
    let unit2_id = create_extra_unit(&app_state, org_id, building_id, owner2_id, "A107").await;

    // Create resolution with Qualified majority (2/3 = 66.67%)
    let create_req = test::TestRequest::post()
        .uri(&format!("/api/v1/meetings/{}/resolutions", meeting_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "meeting_id": meeting_id.to_string(),
            "title": "Resolution with Qualified Majority",
            "description": "Requires 2/3 majority (66.67%)",
            "resolution_type": "extraordinary",
            "majority_required": {
                "qualified": 0.6667
            }
        }))
        .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    let resolution: serde_json::Value = test::read_body_json(create_resp).await;
    let resolution_id = resolution["id"].as_str().unwrap();

    // Cast votes: 40% Contre, 60% Pour (60% < 66.67% threshold)
    let vote1_req = test::TestRequest::post()
        .uri(&format!("/api/v1/resolutions/{}/vote", resolution_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "owner_id": owner1_id.to_string(),
            "unit_id": unit1_id.to_string(),
            "vote_choice": "contre",
            "voting_power": 0.4
        }))
        .to_request();

    test::call_service(&app, vote1_req).await;

    let vote2_req = test::TestRequest::post()
        .uri(&format!("/api/v1/resolutions/{}/vote", resolution_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "owner_id": owner2_id.to_string(),
            "unit_id": unit2_id.to_string(),
            "vote_choice": "pour",
            "voting_power": 0.6
        }))
        .to_request();

    test::call_service(&app, vote2_req).await;

    // Close voting
    let close_req = test::TestRequest::put()
        .uri(&format!("/api/v1/resolutions/{}/close", resolution_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "total_voting_power": 1.0
        }))
        .to_request();

    let close_resp = test::call_service(&app, close_req).await;
    assert_eq!(close_resp.status(), 200);

    let closed_resolution: serde_json::Value = test::read_body_json(close_resp).await;
    assert_eq!(
        closed_resolution["status"], "rejected",
        "Should be Rejected (60% < 66.67% threshold)"
    );
}

#[actix_web::test]
#[serial]
async fn test_get_meeting_vote_summary() {
    let (app_state, _container, org_id) = setup_app().await;
    let (token, _org_id, building_id, meeting_id, owner1_id, owner2_id, unit1_id) =
        create_test_fixtures(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create unit 2
    let unit2_id = create_extra_unit(&app_state, org_id, building_id, owner2_id, "A108").await;

    // Create 2 resolutions and vote on them
    for i in 1..=2 {
        let create_req = test::TestRequest::post()
            .uri(&format!("/api/v1/meetings/{}/resolutions", meeting_id))
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
            .set_json(json!({
                "meeting_id": meeting_id.to_string(),
                "title": format!("Resolution #{}", i),
                "description": format!("Description {}", i),
                "resolution_type": "ordinary",
                "majority_required": "simple"
            }))
            .to_request();

        let create_resp = test::call_service(&app, create_req).await;
        let resolution: serde_json::Value = test::read_body_json(create_resp).await;
        let resolution_id = resolution["id"].as_str().unwrap();

        // Cast votes
        for (owner_id, unit_id, choice, power) in [
            (owner1_id, unit1_id, "pour", 0.4),
            (owner2_id, unit2_id, "contre", 0.6),
        ] {
            let vote_req = test::TestRequest::post()
                .uri(&format!("/api/v1/resolutions/{}/vote", resolution_id))
                .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
                .set_json(json!({
                    "owner_id": owner_id.to_string(),
                    "unit_id": unit_id.to_string(),
                    "vote_choice": choice,
                    "voting_power": power
                }))
                .to_request();

            test::call_service(&app, vote_req).await;
        }

        // Close voting
        let close_req = test::TestRequest::put()
            .uri(&format!("/api/v1/resolutions/{}/close", resolution_id))
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
            .set_json(json!({
                "total_voting_power": 1.0
            }))
            .to_request();

        test::call_service(&app, close_req).await;
    }

    // Get vote summary for meeting
    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/meetings/{}/vote-summary", meeting_id))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let summary: serde_json::Value = test::read_body_json(resp).await;
    let summary_array = summary.as_array().unwrap();
    assert_eq!(
        summary_array.len(),
        2,
        "Should return summary for 2 resolutions"
    );

    // Verify all resolutions have status
    for resolution_summary in summary_array {
        assert!(resolution_summary["status"].is_string());
    }
}

#[actix_web::test]
#[serial]
async fn test_complete_voting_lifecycle() {
    let (app_state, _container, org_id) = setup_app().await;
    let (token, _org_id, building_id, meeting_id, owner1_id, owner2_id, unit1_id) =
        create_test_fixtures(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create unit 2
    let unit2_id = create_extra_unit(&app_state, org_id, building_id, owner2_id, "A109").await;

    // 1. Create resolution
    let create_req = test::TestRequest::post()
        .uri(&format!("/api/v1/meetings/{}/resolutions", meeting_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "meeting_id": meeting_id.to_string(),
            "title": "Complete Lifecycle Resolution",
            "description": "Testing full voting workflow",
            "resolution_type": "ordinary",
            "majority_required": "simple"
        }))
        .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    let resolution: serde_json::Value = test::read_body_json(create_resp).await;
    let resolution_id = resolution["id"].as_str().unwrap();
    assert_eq!(resolution["status"], "pending");

    // 2. Cast initial vote
    let vote1_req = test::TestRequest::post()
        .uri(&format!("/api/v1/resolutions/{}/vote", resolution_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "owner_id": owner1_id.to_string(),
            "unit_id": unit1_id.to_string(),
            "vote_choice": "contre",
            "voting_power": 0.4
        }))
        .to_request();

    let vote1_resp = test::call_service(&app, vote1_req).await;
    let vote1: serde_json::Value = test::read_body_json(vote1_resp).await;
    let vote1_id = vote1["id"].as_str().unwrap();

    // 3. Change vote
    let change_req = test::TestRequest::put()
        .uri(&format!("/api/v1/votes/{}", vote1_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "vote_choice": "pour"
        }))
        .to_request();

    let change_resp = test::call_service(&app, change_req).await;
    let changed_vote: serde_json::Value = test::read_body_json(change_resp).await;
    assert_eq!(changed_vote["vote_choice"], "pour");

    // 4. Cast second vote
    let vote2_req = test::TestRequest::post()
        .uri(&format!("/api/v1/resolutions/{}/vote", resolution_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "owner_id": owner2_id.to_string(),
            "unit_id": unit2_id.to_string(),
            "vote_choice": "pour",
            "voting_power": 0.6
        }))
        .to_request();

    test::call_service(&app, vote2_req).await;

    // 5. List all votes
    let list_req = test::TestRequest::get()
        .uri(&format!("/api/v1/resolutions/{}/votes", resolution_id))
        .to_request();

    let list_resp = test::call_service(&app, list_req).await;
    let votes: serde_json::Value = test::read_body_json(list_resp).await;
    assert_eq!(votes.as_array().unwrap().len(), 2);

    // 6. Close voting
    let close_req = test::TestRequest::put()
        .uri(&format!("/api/v1/resolutions/{}/close", resolution_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "total_voting_power": 1.0
        }))
        .to_request();

    let close_resp = test::call_service(&app, close_req).await;
    let closed: serde_json::Value = test::read_body_json(close_resp).await;
    assert_eq!(closed["status"], "adopted");
    assert_eq!(closed["vote_count_pour"], 2);
    assert_eq!(closed["total_voting_power_pour"], 1.0);

    // 7. Get meeting vote summary
    let summary_req = test::TestRequest::get()
        .uri(&format!("/api/v1/meetings/{}/vote-summary", meeting_id))
        .to_request();

    let summary_resp = test::call_service(&app, summary_req).await;
    let summary: serde_json::Value = test::read_body_json(summary_resp).await;
    assert!(!summary.as_array().unwrap().is_empty());
}
