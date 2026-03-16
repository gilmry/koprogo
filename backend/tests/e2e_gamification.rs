// E2E tests for Gamification & Achievements HTTP endpoints (Issue #49 - Phase 6)
// Tests focus on HTTP layer: endpoints, auth, JSON serialization
// Covers achievements, challenges, and gamification stats

mod common;

use actix_web::http::header;
use actix_web::{test, App};
use koprogo_api::application::dto::RegisterRequest;
use koprogo_api::infrastructure::web::configure_routes;
use serde_json::json;
use serial_test::serial;
use uuid::Uuid;

/// Helper: Register a user and return (token, user_id)
async fn setup_gamification_user(
    app_state: &actix_web::web::Data<koprogo_api::infrastructure::web::AppState>,
    org_id: Uuid,
    label: &str,
) -> (String, Uuid) {
    let email = format!("gamification-{}-{}@example.com", label, Uuid::new_v4());
    let reg = RegisterRequest {
        email: email.clone(),
        password: "SecurePass123!".to_string(),
        first_name: "Gamification".to_string(),
        last_name: label.to_string(),
        role: "superadmin".to_string(),
        organization_id: Some(org_id),
    };
    let login_resp = app_state
        .auth_use_cases
        .register(reg)
        .await
        .expect("Failed to register user");
    let user_id = login_resp.user.id;
    let token = login_resp.token;
    (token, user_id)
}

// ==================== Achievement Tests ====================

#[actix_web::test]
#[serial]
async fn test_achievements_create() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, _user_id) = setup_gamification_user(&app_state, org_id, "achcreate").await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/v1/achievements")
        .insert_header(header::ContentType::json())
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "organization_id": org_id.to_string(),
            "category": "Community",
            "tier": "Bronze",
            "name": "First Step",
            "description": "Created your first community post or action.",
            "icon": "🌟",
            "points_value": 10,
            "requirements": "{\"action\": \"first_community_post\"}",
            "is_secret": false,
            "is_repeatable": false,
            "display_order": 1
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201, "Should create achievement successfully");

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["name"], "First Step");
    assert_eq!(body["category"], "Community");
    assert_eq!(body["tier"], "Bronze");
    assert_eq!(body["points_value"], 10);
    assert_eq!(body["is_secret"], false);
    assert_eq!(body["organization_id"], org_id.to_string());
}

#[actix_web::test]
#[serial]
async fn test_achievements_get() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, _user_id) = setup_gamification_user(&app_state, org_id, "achget").await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create achievement
    let create_req = test::TestRequest::post()
        .uri("/api/v1/achievements")
        .insert_header(header::ContentType::json())
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "organization_id": org_id.to_string(),
            "category": "Sel",
            "tier": "Silver",
            "name": "Exchange Pioneer",
            "description": "Completed 5 local exchanges in the SEL system.",
            "icon": "🔄",
            "points_value": 50,
            "requirements": "{\"action\": \"exchange_completed\", \"count\": 5}",
            "is_secret": false,
            "is_repeatable": false,
            "display_order": 2
        }))
        .to_request();
    let create_resp = test::call_service(&app, create_req).await;
    assert_eq!(create_resp.status(), 201);
    let create_body: serde_json::Value = test::read_body_json(create_resp).await;
    let achievement_id = create_body["id"].as_str().unwrap();

    // Get achievement by ID
    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/achievements/{}", achievement_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200, "Should retrieve achievement by ID");

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["id"], achievement_id);
    assert_eq!(body["name"], "Exchange Pioneer");
    assert_eq!(body["tier"], "Silver");
    assert_eq!(body["points_value"], 50);
}

#[actix_web::test]
#[serial]
async fn test_achievements_list_by_org() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, _user_id) = setup_gamification_user(&app_state, org_id, "achlist").await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create two achievements
    for (name, category, tier, points) in &[
        ("Booking Star", "Booking", "Bronze", 15),
        ("Skill Sharer", "Skills", "Gold", 100),
    ] {
        let req = test::TestRequest::post()
            .uri("/api/v1/achievements")
            .insert_header(header::ContentType::json())
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
            .set_json(json!({
                "organization_id": org_id.to_string(),
                "category": category,
                "tier": tier,
                "name": name,
                "description": "An achievement for the community.",
                "icon": "⭐",
                "points_value": points,
                "requirements": "{\"action\": \"any\"}",
                "is_secret": false,
                "is_repeatable": false,
                "display_order": 1
            }))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 201, "Achievement creation must succeed");
    }

    // List achievements for organization
    let list_req = test::TestRequest::get()
        .uri(&format!("/api/v1/organizations/{}/achievements", org_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();
    let list_resp = test::call_service(&app, list_req).await;
    assert_eq!(list_resp.status(), 200, "Should list org achievements");

    let achievements: serde_json::Value = test::read_body_json(list_resp).await;
    assert!(achievements.is_array(), "Response should be an array");
    assert_eq!(
        achievements.as_array().unwrap().len(),
        2,
        "Should have 2 achievements"
    );
}

// ==================== Challenge Tests ====================

#[actix_web::test]
#[serial]
async fn test_challenges_create() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, _user_id) = setup_gamification_user(&app_state, org_id, "chalcreate").await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/v1/challenges")
        .insert_header(header::ContentType::json())
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "organization_id": org_id.to_string(),
            "challenge_type": "Individual",
            "title": "Spring Cleaning Challenge",
            "description": "Complete 3 shared object loans during spring.",
            "icon": "🌸",
            "start_date": "2027-03-01T00:00:00Z",
            "end_date": "2027-05-31T23:59:59Z",
            "target_metric": "shared_object_loans",
            "target_value": 3,
            "reward_points": 150
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201, "Should create challenge successfully");

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["title"], "Spring Cleaning Challenge");
    assert_eq!(body["challenge_type"], "Individual");
    assert_eq!(body["status"], "Draft");
    assert_eq!(body["target_value"], 3);
    assert_eq!(body["reward_points"], 150);
    assert_eq!(body["organization_id"], org_id.to_string());
}

#[actix_web::test]
#[serial]
async fn test_challenges_activate() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, _user_id) = setup_gamification_user(&app_state, org_id, "chalactivate").await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create a challenge (starts in Draft)
    let create_req = test::TestRequest::post()
        .uri("/api/v1/challenges")
        .insert_header(header::ContentType::json())
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "organization_id": org_id.to_string(),
            "challenge_type": "Individual",
            "title": "Neighbour Outreach Challenge",
            "description": "Meet 5 neighbours through community events.",
            "icon": "🤝",
            "start_date": "2027-04-01T00:00:00Z",
            "end_date": "2027-06-30T23:59:59Z",
            "target_metric": "community_meetings",
            "target_value": 5,
            "reward_points": 200
        }))
        .to_request();
    let create_resp = test::call_service(&app, create_req).await;
    assert_eq!(create_resp.status(), 201, "Challenge creation must succeed");
    let create_body: serde_json::Value = test::read_body_json(create_resp).await;
    let challenge_id = create_body["id"].as_str().unwrap();
    assert_eq!(create_body["status"], "Draft");

    // Activate the challenge
    let activate_req = test::TestRequest::put()
        .uri(&format!("/api/v1/challenges/{}/activate", challenge_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();
    let activate_resp = test::call_service(&app, activate_req).await;
    assert_eq!(
        activate_resp.status(),
        200,
        "Should activate challenge successfully"
    );

    let body: serde_json::Value = test::read_body_json(activate_resp).await;
    assert_eq!(
        body["status"], "Active",
        "Challenge status should be Active"
    );
    assert_eq!(body["id"], challenge_id);
}

// ==================== Gamification Stats Tests ====================

#[actix_web::test]
#[serial]
async fn test_gamification_user_stats() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, _user_id) = setup_gamification_user(&app_state, org_id, "statsuser").await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Get gamification stats for the authenticated user (via org endpoint)
    let req = test::TestRequest::get()
        .uri(&format!(
            "/api/v1/organizations/{}/gamification/stats",
            org_id
        ))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200, "Should retrieve gamification stats");

    let body: serde_json::Value = test::read_body_json(resp).await;
    // A new user has 0 points, 0 achievements, 0 challenges completed
    assert!(
        body["total_points"].is_number(),
        "total_points should be a number"
    );
    assert!(
        body["achievements_earned"].is_number(),
        "achievements_earned should be a number"
    );
    assert!(
        body["challenges_completed"].is_number(),
        "challenges_completed should be a number"
    );
}

#[actix_web::test]
#[serial]
async fn test_gamification_leaderboard() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, _user_id) = setup_gamification_user(&app_state, org_id, "leaderuser").await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Get gamification leaderboard for the organization
    let req = test::TestRequest::get()
        .uri(&format!(
            "/api/v1/organizations/{}/gamification/leaderboard",
            org_id
        ))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.status(),
        200,
        "Should retrieve gamification leaderboard"
    );

    let body: serde_json::Value = test::read_body_json(resp).await;
    // Leaderboard should be an array (possibly empty for a fresh org)
    assert!(
        body["entries"].is_array() || body.is_array(),
        "Response should contain leaderboard entries"
    );
}
