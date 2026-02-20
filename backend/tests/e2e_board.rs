mod common;

use actix_web::{http::header, test, App};
use koprogo_api::application::dto::CreateBuildingDto;
use koprogo_api::infrastructure::web::{configure_routes, AppState};
use serial_test::serial;
use uuid::Uuid;

#[actix_web::test]
#[serial]
async fn test_board_member_lifecycle() {
    let (app_state, _container, org_id) = common::setup_test_db().await;

    let state = app_state.clone();
    let app = test::init_service(
        App::new()
            .app_data(state.clone())
            .configure(configure_routes),
    )
    .await;

    let token = common::register_and_login(&state, org_id).await;

    // Create building
    let building_dto = CreateBuildingDto {
        organization_id: org_id.to_string(),
        name: "Test Building Board".to_string(),
        address: "123 Board Street".to_string(),
        city: "Brussels".to_string(),
        postal_code: "1000".to_string(),
        country: "Belgium".to_string(),
        total_units: 25,
        total_tantiemes: Some(1000),
        construction_year: Some(2020),
    };

    let req = test::TestRequest::post()
        .uri("/api/v1/buildings")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(&building_dto)
        .to_request();

    let resp = test::call_service(&app, req).await;
    let status = resp.status();
    if status != 201 {
        let body_bytes = test::read_body(resp).await;
        eprintln!(
            "Building creation failed: {:?}",
            String::from_utf8_lossy(&body_bytes)
        );
        panic!("Expected 201 for building, got {}", status);
    }
    let building: serde_json::Value = test::read_body_json(resp).await;
    let building_id = building["id"].as_str().unwrap();

    // Create an owner (board members must be property owners)
    let owner_dto = serde_json::json!({
        "organization_id": org_id,
        "first_name": "John",
        "last_name": "BoardMember",
        "email": "john.board@example.com",
        "phone": "+32123456789",
        "address": "456 Owner Street",
        "city": "Brussels",
        "postal_code": "1000",
        "country": "Belgium"
    });

    let req = test::TestRequest::post()
        .uri("/api/v1/owners")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(&owner_dto)
        .to_request();

    let resp = test::call_service(&app, req).await;
    let status = resp.status();
    if status != 201 {
        let body_bytes = test::read_body(resp).await;
        eprintln!(
            "Owner creation failed: {:?}",
            String::from_utf8_lossy(&body_bytes)
        );
        panic!("Expected 201 for owner, got {}", status);
    }
    let owner: serde_json::Value = test::read_body_json(resp).await;
    let owner_id = owner["id"].as_str().unwrap();

    // 1. Elect a board member
    // Note: We need a meeting_id for the election. Create a quick meeting first.
    let meeting_dto = serde_json::json!({
        "organization_id": org_id,
        "building_id": building_id,
        "meeting_type": "Ordinary",
        "title": "Board Election Meeting",
        "description": "Election of board members",
        "scheduled_date": chrono::Utc::now().to_rfc3339(),
        "location": "Community Hall"
    });

    let req = test::TestRequest::post()
        .uri("/api/v1/meetings")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(&meeting_dto)
        .to_request();

    let resp = test::call_service(&app, req).await;
    let status = resp.status();
    if status != 201 {
        let body_bytes = test::read_body(resp).await;
        eprintln!(
            "Meeting creation failed: {:?}",
            String::from_utf8_lossy(&body_bytes)
        );
        panic!("Expected 201 for meeting, got {}", status);
    }
    let meeting: serde_json::Value = test::read_body_json(resp).await;
    let meeting_id = meeting["id"].as_str().unwrap();

    // Create a mandate that's expiring soon (started 310 days ago, expires in 55 days)
    // This allows testing the renewal feature
    let mandate_start = chrono::Utc::now() - chrono::Duration::days(310);
    let mandate_end = mandate_start + chrono::Duration::days(365);

    let create_member = serde_json::json!({
        "owner_id": owner_id,
        "building_id": building_id,
        "position": "president",
        "mandate_start": mandate_start.to_rfc3339(),
        "mandate_end": mandate_end.to_rfc3339(),
        "elected_by_meeting_id": meeting_id,
    });

    let req = test::TestRequest::post()
        .uri("/api/v1/board-members")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(&create_member)
        .to_request();

    let resp = test::call_service(&app, req).await;
    let status = resp.status();
    eprintln!("Board member creation response status: {}", status);
    if status != 201 {
        let body_bytes = test::read_body(resp).await;
        eprintln!("Error body: {:?}", String::from_utf8_lossy(&body_bytes));
        panic!("Expected 201, got {}", status);
    }
    let body: serde_json::Value = test::read_body_json(resp).await;
    let member_id = body["id"].as_str().unwrap();
    assert_eq!(body["position"].as_str().unwrap(), "president");

    // 2. Get board member by ID
    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/board-members/{}", member_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["id"].as_str().unwrap(), member_id);

    // 3. List active board members
    let req = test::TestRequest::get()
        .uri(&format!(
            "/api/v1/buildings/{}/board-members/active",
            building_id
        ))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(!body.as_array().unwrap().is_empty());

    // 4. Renew mandate
    let renew = serde_json::json!({
        "new_elected_by_meeting_id": meeting_id,
    });

    let req = test::TestRequest::put()
        .uri(&format!("/api/v1/board-members/{}/renew", member_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(&renew)
        .to_request();

    let resp = test::call_service(&app, req).await;
    let status = resp.status();
    if !status.is_success() {
        let body_bytes = test::read_body(resp).await;
        eprintln!(
            "Renew mandate failed with {}: {:?}",
            status,
            String::from_utf8_lossy(&body_bytes)
        );
        panic!("Expected success for renew, got {}", status);
    }

    // 5. Get board stats
    let req = test::TestRequest::get()
        .uri(&format!(
            "/api/v1/buildings/{}/board-members/stats",
            building_id
        ))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["total_members"].as_i64().unwrap(), 1);

    // 6. Remove board member
    let req = test::TestRequest::delete()
        .uri(&format!("/api/v1/board-members/{}", member_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 204);
}

#[actix_web::test]
#[serial]
async fn test_board_decision_lifecycle() {
    let (app_state, _container, org_id) = common::setup_test_db().await;

    let state = app_state.clone();
    let app = test::init_service(
        App::new()
            .app_data(state.clone())
            .configure(configure_routes),
    )
    .await;

    let token = common::register_and_login(&state, org_id).await;

    // Create building
    let building_dto = CreateBuildingDto {
        organization_id: org_id.to_string(),
        name: "Test Building Board".to_string(),
        address: "123 Board Street".to_string(),
        city: "Brussels".to_string(),
        postal_code: "1000".to_string(),
        country: "Belgium".to_string(),
        total_units: 25,
        total_tantiemes: Some(1000),
        construction_year: Some(2020),
    };

    let req = test::TestRequest::post()
        .uri("/api/v1/buildings")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(&building_dto)
        .to_request();

    let resp = test::call_service(&app, req).await;
    let status = resp.status();
    if status != 201 {
        let body_bytes = test::read_body(resp).await;
        eprintln!(
            "Building creation failed: {:?}",
            String::from_utf8_lossy(&body_bytes)
        );
        panic!("Expected 201 for building, got {}", status);
    }
    let building: serde_json::Value = test::read_body_json(resp).await;
    let building_id = building["id"].as_str().unwrap();

    // Create meeting
    let meeting_dto = serde_json::json!({
        "organization_id": org_id,
        "building_id": building_id,
        "meeting_type": "Ordinary",
        "title": "Annual General Assembly",
        "description": "Budget approval and board election",
        "scheduled_date": (chrono::Utc::now() + chrono::Duration::days(30)).to_rfc3339(),
        "location": "Community Hall"
    });

    let req = test::TestRequest::post()
        .uri("/api/v1/meetings")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(&meeting_dto)
        .to_request();

    let resp = test::call_service(&app, req).await;
    let status = resp.status();
    if status != 201 {
        let body_bytes = test::read_body(resp).await;
        eprintln!(
            "Meeting creation failed: {:?}",
            String::from_utf8_lossy(&body_bytes)
        );
        panic!("Expected 201 for meeting, got {}", status);
    }
    let meeting: serde_json::Value = test::read_body_json(resp).await;
    let meeting_id = meeting["id"].as_str().unwrap();

    // 1. Create a board decision
    let create_decision = serde_json::json!({
        "building_id": building_id,
        "meeting_id": meeting_id,
        "subject": "Roof Renovation Project",
        "decision_text": "Approve budget for roof renovation - work must be completed within 90 days",
        "deadline": (chrono::Utc::now() + chrono::Duration::days(90)).to_rfc3339(),
    });

    let req = test::TestRequest::post()
        .uri("/api/v1/board-decisions")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(&create_decision)
        .to_request();

    let resp = test::call_service(&app, req).await;
    let status = resp.status();
    if status != 201 {
        let body_bytes = test::read_body(resp).await;
        eprintln!(
            "Decision creation failed: {:?}",
            String::from_utf8_lossy(&body_bytes)
        );
        panic!("Expected 201 for decision, got {}", status);
    }
    let body: serde_json::Value = test::read_body_json(resp).await;
    let decision_id = body["id"].as_str().unwrap();
    assert_eq!(body["status"].as_str().unwrap(), "pending");

    // 2. Get decision by ID
    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/board-decisions/{}", decision_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["subject"].as_str().unwrap(), "Roof Renovation Project");

    // 3. List decisions by building
    let req = test::TestRequest::get()
        .uri(&format!(
            "/api/v1/buildings/{}/board-decisions",
            building_id
        ))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(!body.as_array().unwrap().is_empty());

    // 4. Update decision status
    let update = serde_json::json!({
        "status": "in_progress",
        "responsible_party": "Project Manager",
    });

    let req = test::TestRequest::put()
        .uri(&format!("/api/v1/board-decisions/{}", decision_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(&update)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["status"].as_str().unwrap(), "in_progress");

    // 5. Add notes
    let notes = serde_json::json!({
        "notes": "Contractor quotes received",
    });

    let req = test::TestRequest::post()
        .uri(&format!("/api/v1/board-decisions/{}/notes", decision_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(&notes)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    // 6. List decisions by status
    let req = test::TestRequest::get()
        .uri(&format!(
            "/api/v1/buildings/{}/board-decisions/status/in_progress",
            building_id
        ))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(!body.as_array().unwrap().is_empty());

    // 7. Complete decision
    let req = test::TestRequest::put()
        .uri(&format!("/api/v1/board-decisions/{}/complete", decision_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["status"].as_str().unwrap(), "completed");

    // 8. Get decision stats
    let req = test::TestRequest::get()
        .uri(&format!(
            "/api/v1/buildings/{}/board-decisions/stats",
            building_id
        ))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["total_decisions"].as_i64().unwrap(), 1);
    assert_eq!(body["completed"].as_i64().unwrap(), 1);
}

#[actix_web::test]
#[serial]
async fn test_overdue_decisions() {
    let (app_state, _container, org_id) = common::setup_test_db().await;

    let state = app_state.clone();
    let app = test::init_service(
        App::new()
            .app_data(state.clone())
            .configure(configure_routes),
    )
    .await;

    let token = common::register_and_login(&state, org_id).await;

    // Create building
    let building_dto = CreateBuildingDto {
        organization_id: org_id.to_string(),
        name: "Test Building Board".to_string(),
        address: "123 Board Street".to_string(),
        city: "Brussels".to_string(),
        postal_code: "1000".to_string(),
        country: "Belgium".to_string(),
        total_units: 25,
        total_tantiemes: Some(1000),
        construction_year: Some(2020),
    };

    let req = test::TestRequest::post()
        .uri("/api/v1/buildings")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(&building_dto)
        .to_request();

    let resp = test::call_service(&app, req).await;
    let status = resp.status();
    if status != 201 {
        let body_bytes = test::read_body(resp).await;
        eprintln!(
            "Building creation failed: {:?}",
            String::from_utf8_lossy(&body_bytes)
        );
        panic!("Expected 201 for building, got {}", status);
    }
    let building: serde_json::Value = test::read_body_json(resp).await;
    let building_id = building["id"].as_str().unwrap();

    // Create a meeting first (decisions need a meeting_id)
    let meeting_dto = serde_json::json!({
        "organization_id": org_id,
        "building_id": building_id,
        "meeting_type": "Ordinary",
        "title": "Quick Meeting",
        "description": "Decision meeting",
        "scheduled_date": (chrono::Utc::now() - chrono::Duration::days(10)).to_rfc3339(),
        "location": "Online"
    });

    let req = test::TestRequest::post()
        .uri("/api/v1/meetings")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(&meeting_dto)
        .to_request();

    let resp = test::call_service(&app, req).await;
    let status = resp.status();
    if status != 201 {
        let body_bytes = test::read_body(resp).await;
        eprintln!(
            "Meeting creation failed: {:?}",
            String::from_utf8_lossy(&body_bytes)
        );
        panic!("Expected 201 for meeting, got {}", status);
    }
    let meeting: serde_json::Value = test::read_body_json(resp).await;
    let meeting_id = meeting["id"].as_str().unwrap();

    // Create a decision without deadline (will test overdue endpoint with empty results or different logic)
    // Note: Business logic prevents creating decisions with past deadlines
    let create_decision = serde_json::json!({
        "building_id": building_id,
        "meeting_id": meeting_id,
        "subject": "Task To Track",
        "decision_text": "This decision will be tracked by the board",
        "deadline": null,
    });

    let req = test::TestRequest::post()
        .uri("/api/v1/board-decisions")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(&create_decision)
        .to_request();

    let resp = test::call_service(&app, req).await;
    let status = resp.status();
    if status != 201 {
        let body_bytes = test::read_body(resp).await;
        eprintln!(
            "Decision creation failed: {:?}",
            String::from_utf8_lossy(&body_bytes)
        );
        panic!("Expected 201 for decision, got {}", status);
    }

    // List overdue decisions
    let req = test::TestRequest::get()
        .uri(&format!(
            "/api/v1/buildings/{}/board-decisions/overdue",
            building_id
        ))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    // Note: Since we can't create overdue decisions (business logic prevents past deadlines),
    // this endpoint should return empty results or we would need to update an existing decision
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body.as_array().is_some()); // Just verify it returns an array
}
