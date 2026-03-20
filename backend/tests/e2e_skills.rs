// E2E tests for Skills Directory HTTP endpoints (Issue #49 - Phase 3)
// Tests focus on HTTP layer: endpoints, auth, JSON serialization
// Covers skill lifecycle: create, get, list, update, delete

mod common;

use actix_web::http::header;
use actix_web::{test, App};
use koprogo_api::application::dto::RegisterRequest;
use koprogo_api::infrastructure::web::configure_routes;
use serde_json::json;
use serial_test::serial;
use uuid::Uuid;

/// Helper: Register a user, create a linked owner, return (token, user_id)
async fn setup_skills_user_with_owner(
    app_state: &actix_web::web::Data<koprogo_api::infrastructure::web::AppState>,
    org_id: Uuid,
) -> (String, Uuid) {
    let email = format!("skills-test-{}@example.com", Uuid::new_v4());
    let reg = RegisterRequest {
        email: email.clone(),
        password: "SecurePass123!".to_string(),
        first_name: "Skills".to_string(),
        last_name: "Tester".to_string(),
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

    // Create an owner in the DB and link to this user
    let owner_id = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO owners (id, organization_id, user_id, first_name, last_name, email,
           address, city, postal_code, country, created_at, updated_at)
           VALUES ($1, $2, $3, 'Skills', 'Tester', $4, '2 Rue Test', 'Brussels', '1000', 'BE', NOW(), NOW())"#,
    )
    .bind(owner_id)
    .bind(org_id)
    .bind(user_id)
    .bind(format!("owner-skills-{}@test.com", Uuid::new_v4()))
    .execute(&app_state.pool)
    .await
    .expect("Failed to insert owner");

    (token, user_id)
}

// ==================== Skills CRUD Tests ====================

#[actix_web::test]
#[serial]
async fn test_skills_create() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, _user_id) = setup_skills_user_with_owner(&app_state, org_id).await;

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
            "name": "Skills Test Building",
            "address": "20 Skills Street",
            "city": "Brussels",
            "postal_code": "1000",
            "country": "BE",
            "total_units": 8,
            "total_tantiemes": 1000
        }))
        .to_request();
    let building_resp = test::call_service(&app, building_req).await;
    let building_body: serde_json::Value = test::read_body_json(building_resp).await;
    let building_id = building_body["id"].as_str().unwrap();

    let req = test::TestRequest::post()
        .uri("/api/v1/skills")
        .insert_header(header::ContentType::json())
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "building_id": building_id,
            "skill_category": "HomeRepair",
            "skill_name": "Plumbing",
            "expertise_level": "Intermediate",
            "description": "I can fix leaking faucets and pipes.",
            "is_available_for_help": true,
            "hourly_rate_credits": 2
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201, "Should create skill successfully");

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["skill_name"], "Plumbing");
    assert_eq!(body["skill_category"], "HomeRepair");
    assert_eq!(body["expertise_level"], "Intermediate");
    assert_eq!(body["is_available_for_help"], true);
}

#[actix_web::test]
#[serial]
async fn test_skills_get() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, _user_id) = setup_skills_user_with_owner(&app_state, org_id).await;

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
            "name": "Skills Get Building",
            "address": "21 Skills Street",
            "city": "Brussels",
            "postal_code": "1000",
            "country": "BE",
            "total_units": 8,
            "total_tantiemes": 1000
        }))
        .to_request();
    let building_resp = test::call_service(&app, building_req).await;
    let building_body: serde_json::Value = test::read_body_json(building_resp).await;
    let building_id = building_body["id"].as_str().unwrap();

    // Create skill
    let create_req = test::TestRequest::post()
        .uri("/api/v1/skills")
        .insert_header(header::ContentType::json())
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "building_id": building_id,
            "skill_category": "Technology",
            "skill_name": "Web Development",
            "expertise_level": "Expert",
            "description": "Full-stack web developer available to help.",
            "is_available_for_help": true
        }))
        .to_request();
    let create_resp = test::call_service(&app, create_req).await;
    assert_eq!(create_resp.status(), 201);
    let create_body: serde_json::Value = test::read_body_json(create_resp).await;
    let skill_id = create_body["id"].as_str().unwrap();

    // Get skill by ID
    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/skills/{}", skill_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200, "Should retrieve skill by ID");

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["id"], skill_id);
    assert_eq!(body["skill_name"], "Web Development");
    assert_eq!(body["expertise_level"], "Expert");
}

#[actix_web::test]
#[serial]
async fn test_skills_list_by_building() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, _user_id) = setup_skills_user_with_owner(&app_state, org_id).await;

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
            "name": "Skills List Building",
            "address": "22 Skills Street",
            "city": "Brussels",
            "postal_code": "1000",
            "country": "BE",
            "total_units": 8,
            "total_tantiemes": 1000
        }))
        .to_request();
    let building_resp = test::call_service(&app, building_req).await;
    let building_body: serde_json::Value = test::read_body_json(building_resp).await;
    let building_id = building_body["id"].as_str().unwrap();

    // Create two skills
    for (skill_name, category) in &[("Gardening", "Gardening"), ("Cooking", "Cooking")] {
        let req = test::TestRequest::post()
            .uri("/api/v1/skills")
            .insert_header(header::ContentType::json())
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
            .set_json(json!({
                "building_id": building_id,
                "skill_category": category,
                "skill_name": skill_name,
                "expertise_level": "Beginner",
                "description": "Available to help with this skill.",
                "is_available_for_help": true
            }))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 201);
    }

    // List skills for building
    let list_req = test::TestRequest::get()
        .uri(&format!("/api/v1/buildings/{}/skills", building_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();
    let list_resp = test::call_service(&app, list_req).await;
    assert_eq!(list_resp.status(), 200, "Should list building skills");

    let skills: serde_json::Value = test::read_body_json(list_resp).await;
    assert!(skills.is_array(), "Response should be an array");
    assert_eq!(skills.as_array().unwrap().len(), 2, "Should have 2 skills");
}

#[actix_web::test]
#[serial]
async fn test_skills_update() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, _user_id) = setup_skills_user_with_owner(&app_state, org_id).await;

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
            "name": "Skills Update Building",
            "address": "23 Skills Street",
            "city": "Brussels",
            "postal_code": "1000",
            "country": "BE",
            "total_units": 8,
            "total_tantiemes": 1000
        }))
        .to_request();
    let building_resp = test::call_service(&app, building_req).await;
    let building_body: serde_json::Value = test::read_body_json(building_resp).await;
    let building_id = building_body["id"].as_str().unwrap();

    // Create skill
    let create_req = test::TestRequest::post()
        .uri("/api/v1/skills")
        .insert_header(header::ContentType::json())
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "building_id": building_id,
            "skill_category": "HomeRepair",
            "skill_name": "Painting",
            "expertise_level": "Beginner",
            "description": "Basic painting skills.",
            "is_available_for_help": false
        }))
        .to_request();
    let create_resp = test::call_service(&app, create_req).await;
    assert_eq!(create_resp.status(), 201);
    let create_body: serde_json::Value = test::read_body_json(create_resp).await;
    let skill_id = create_body["id"].as_str().unwrap();

    // Update skill
    let update_req = test::TestRequest::put()
        .uri(&format!("/api/v1/skills/{}", skill_id))
        .insert_header(header::ContentType::json())
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "expertise_level": "Intermediate",
            "is_available_for_help": true,
            "description": "Intermediate painting and wallpaper."
        }))
        .to_request();
    let update_resp = test::call_service(&app, update_req).await;
    assert_eq!(
        update_resp.status(),
        200,
        "Should update skill successfully"
    );

    let body: serde_json::Value = test::read_body_json(update_resp).await;
    assert_eq!(body["expertise_level"], "Intermediate");
    assert_eq!(body["is_available_for_help"], true);
}

#[actix_web::test]
#[serial]
async fn test_skills_delete() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, _user_id) = setup_skills_user_with_owner(&app_state, org_id).await;

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
            "name": "Skills Delete Building",
            "address": "24 Skills Street",
            "city": "Brussels",
            "postal_code": "1000",
            "country": "BE",
            "total_units": 8,
            "total_tantiemes": 1000
        }))
        .to_request();
    let building_resp = test::call_service(&app, building_req).await;
    let building_body: serde_json::Value = test::read_body_json(building_resp).await;
    let building_id = building_body["id"].as_str().unwrap();

    // Create skill
    let create_req = test::TestRequest::post()
        .uri("/api/v1/skills")
        .insert_header(header::ContentType::json())
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "building_id": building_id,
            "skill_category": "Technology",
            "skill_name": "Skill to Delete",
            "expertise_level": "Beginner",
            "description": "This skill will be deleted.",
            "is_available_for_help": true
        }))
        .to_request();
    let create_resp = test::call_service(&app, create_req).await;
    assert_eq!(create_resp.status(), 201);
    let create_body: serde_json::Value = test::read_body_json(create_resp).await;
    let skill_id = create_body["id"].as_str().unwrap();

    // Delete skill
    let delete_req = test::TestRequest::delete()
        .uri(&format!("/api/v1/skills/{}", skill_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();
    let delete_resp = test::call_service(&app, delete_req).await;
    let status = delete_resp.status().as_u16();
    assert!(
        status == 200 || status == 204,
        "Should delete skill (got {})",
        status
    );

    // Verify skill is gone
    let get_req = test::TestRequest::get()
        .uri(&format!("/api/v1/skills/{}", skill_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();
    let get_resp = test::call_service(&app, get_req).await;
    assert_eq!(
        get_resp.status(),
        404,
        "Skill should not exist after delete"
    );
}
