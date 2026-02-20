mod common;

use actix_web::{http::header, test, web, App};
use chrono::{Duration, Utc};
use koprogo_api::application::dto::{
    CreateBuildingDto, LoginRequest, QuoteComparisonResponseDto, QuoteResponseDto, RegisterRequest,
};
use koprogo_api::infrastructure::web::{configure_routes, AppState};
use serde_json::json;
use serial_test::serial;
use uuid::Uuid;

// ==================== Test Setup ====================

async fn create_test_user(app_state: &web::Data<AppState>, org_id: Uuid) -> (Uuid, String) {
    let email = format!("quote_test_{}@example.com", Uuid::new_v4());
    let register_result = app_state
        .auth_use_cases
        .register(RegisterRequest {
            email: email.clone(),
            password: "TestPassword123!".to_string(),
            first_name: "Test".to_string(),
            last_name: "User".to_string(),
            role: "superadmin".to_string(),
            organization_id: Some(org_id),
        })
        .await
        .expect("Failed to register test user");

    let login_result = app_state
        .auth_use_cases
        .login(LoginRequest {
            email,
            password: "TestPassword123!".to_string(),
        })
        .await
        .expect("Failed to login test user");

    (register_result.user.id, login_result.token)
}

async fn create_test_building(app_state: &web::Data<AppState>, organization_id: Uuid) -> Uuid {
    let building_name = format!("Test Building {}", Uuid::new_v4());
    let dto = CreateBuildingDto {
        organization_id: organization_id.to_string(),
        name: building_name,
        address: "123 Test Street".to_string(),
        city: "Test City".to_string(),
        postal_code: "12345".to_string(),
        country: "BE".to_string(),
        total_units: 10,
        total_tantiemes: Some(1000),
        construction_year: Some(2020),
    };
    let building = app_state
        .building_use_cases
        .create_building(dto)
        .await
        .expect("Failed to create test building");

    Uuid::parse_str(&building.id).expect("Failed to parse building id")
}

async fn create_test_contractor(app_state: &web::Data<AppState>, org_id: Uuid) -> Uuid {
    let email = format!("contractor_{}@example.com", Uuid::new_v4());
    let contractor = app_state
        .auth_use_cases
        .register(RegisterRequest {
            email: email.clone(),
            password: "ContractorPass123!".to_string(),
            first_name: "Contractor".to_string(),
            last_name: "Test".to_string(),
            role: "superadmin".to_string(),
            organization_id: Some(org_id),
        })
        .await
        .expect("Failed to create test contractor");

    contractor.user.id
}

// ==================== Quote CRUD Tests ====================

#[actix_web::test]
#[serial]
async fn test_create_quote_success() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (_user_id, token) = create_test_user(&app_state, org_id).await;
    let building_id = create_test_building(&app_state, org_id).await;
    let contractor_id = create_test_contractor(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let validity_date = Utc::now() + Duration::days(30);

    let create_req = test::TestRequest::post()
        .uri("/api/v1/quotes")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "building_id": building_id.to_string(),
            "contractor_id": contractor_id.to_string(),
            "project_title": "Roof Repair Project",
            "project_description": "Repair leaking roof tiles and replace gutters",
            "amount_excl_vat": "5000.00",
            "vat_rate": "0.21",
            "validity_date": validity_date.to_rfc3339(),
            "estimated_start_date": (Utc::now() + Duration::days(45)).to_rfc3339(),
            "estimated_duration_days": 14,
            "warranty_years": 10
        }))
        .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    assert_eq!(
        create_resp.status(),
        201,
        "Expected 201 Created for quote creation"
    );

    let quote: QuoteResponseDto = test::read_body_json(create_resp).await;
    assert_eq!(quote.building_id, building_id.to_string());
    assert_eq!(quote.contractor_id, contractor_id.to_string());
    assert_eq!(quote.project_title, "Roof Repair Project");
    assert_eq!(quote.amount_excl_vat, "5000.00");
    assert_eq!(quote.amount_incl_vat, "6050.00"); // 5000 * 1.21
    assert_eq!(quote.status, "Requested");
    assert!(!quote.is_expired);
    assert_eq!(quote.estimated_duration_days, 14);
    assert_eq!(quote.warranty_years, 10);
}

#[actix_web::test]
#[serial]
async fn test_create_quote_without_auth() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (_user_id, _token) = create_test_user(&app_state, org_id).await;
    let building_id = create_test_building(&app_state, org_id).await;
    let contractor_id = create_test_contractor(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let validity_date = Utc::now() + Duration::days(30);

    let create_req = test::TestRequest::post()
        .uri("/api/v1/quotes")
        .set_json(json!({
            "building_id": building_id.to_string(),
            "contractor_id": contractor_id.to_string(),
            "project_title": "Roof Repair",
            "project_description": "Fix leaks",
            "amount_excl_vat": "5000.00",
            "vat_rate": "0.21",
            "validity_date": validity_date.to_rfc3339(),
            "estimated_duration_days": 14,
            "warranty_years": 10
        }))
        .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    assert_eq!(
        create_resp.status(),
        401,
        "Expected 401 Unauthorized without authentication"
    );
}

#[actix_web::test]
#[serial]
async fn test_create_quote_belgian_vat_rates() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (_user_id, token) = create_test_user(&app_state, org_id).await;
    let building_id = create_test_building(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let vat_rates = vec![
        ("0.06", "5300.00"), // 6% reduced rate (renovations)
        ("0.21", "6050.00"), // 21% standard rate (new construction)
    ];

    for (vat_rate, expected_incl_vat) in vat_rates {
        let contractor_id = create_test_contractor(&app_state, org_id).await;
        let validity_date = Utc::now() + Duration::days(30);

        let create_req = test::TestRequest::post()
            .uri("/api/v1/quotes")
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
            .set_json(json!({
                "building_id": building_id.to_string(),
                "contractor_id": contractor_id.to_string(),
                "project_title": format!("Project with VAT {}", vat_rate),
                "project_description": "Test Belgian VAT rates",
                "amount_excl_vat": "5000.00",
                "vat_rate": vat_rate,
                "validity_date": validity_date.to_rfc3339(),
                "estimated_duration_days": 14,
                "warranty_years": 10
            }))
            .to_request();

        let create_resp = test::call_service(&app, create_req).await;
        assert_eq!(
            create_resp.status(),
            201,
            "Expected 201 Created for VAT rate {}",
            vat_rate
        );

        let quote: QuoteResponseDto = test::read_body_json(create_resp).await;
        assert_eq!(quote.vat_rate, vat_rate);
        assert_eq!(quote.amount_incl_vat, expected_incl_vat);
    }
}

#[actix_web::test]
#[serial]
async fn test_create_quote_belgian_warranty_standards() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (_user_id, token) = create_test_user(&app_state, org_id).await;
    let building_id = create_test_building(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let warranty_standards = vec![
        (2, "2 years - Apparent defects"),
        (10, "10 years - Structural (décennale)"),
    ];

    for (warranty_years, description) in warranty_standards {
        let contractor_id = create_test_contractor(&app_state, org_id).await;
        let validity_date = Utc::now() + Duration::days(30);

        let create_req = test::TestRequest::post()
            .uri("/api/v1/quotes")
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
            .set_json(json!({
                "building_id": building_id.to_string(),
                "contractor_id": contractor_id.to_string(),
                "project_title": format!("Project - {}", description),
                "project_description": "Test Belgian warranty standards",
                "amount_excl_vat": "5000.00",
                "vat_rate": "0.21",
                "validity_date": validity_date.to_rfc3339(),
                "estimated_duration_days": 14,
                "warranty_years": warranty_years
            }))
            .to_request();

        let create_resp = test::call_service(&app, create_req).await;
        assert_eq!(
            create_resp.status(),
            201,
            "Expected 201 Created for {} warranty",
            description
        );

        let quote: QuoteResponseDto = test::read_body_json(create_resp).await;
        assert_eq!(quote.warranty_years, warranty_years);
    }
}

#[actix_web::test]
#[serial]
async fn test_get_quote_by_id() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (_user_id, token) = create_test_user(&app_state, org_id).await;
    let building_id = create_test_building(&app_state, org_id).await;
    let contractor_id = create_test_contractor(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let validity_date = Utc::now() + Duration::days(30);

    // Create quote
    let create_req = test::TestRequest::post()
        .uri("/api/v1/quotes")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "building_id": building_id.to_string(),
            "contractor_id": contractor_id.to_string(),
            "project_title": "Test Quote",
            "project_description": "Test description",
            "amount_excl_vat": "5000.00",
            "vat_rate": "0.21",
            "validity_date": validity_date.to_rfc3339(),
            "estimated_duration_days": 14,
            "warranty_years": 10
        }))
        .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    let created_quote: QuoteResponseDto = test::read_body_json(create_resp).await;

    // Get by ID
    let get_req = test::TestRequest::get()
        .uri(&format!("/api/v1/quotes/{}", created_quote.id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let get_resp = test::call_service(&app, get_req).await;
    assert_eq!(get_resp.status(), 200);

    let fetched_quote: QuoteResponseDto = test::read_body_json(get_resp).await;
    assert_eq!(fetched_quote.id, created_quote.id);
    assert_eq!(fetched_quote.project_title, "Test Quote");
}

#[actix_web::test]
#[serial]
async fn test_get_quote_not_found() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (_user_id, token) = create_test_user(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let non_existent_id = Uuid::new_v4();
    let get_req = test::TestRequest::get()
        .uri(&format!("/api/v1/quotes/{}", non_existent_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let get_resp = test::call_service(&app, get_req).await;
    assert_eq!(get_resp.status(), 404);
}

#[actix_web::test]
#[serial]
async fn test_list_building_quotes() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (_user_id, token) = create_test_user(&app_state, org_id).await;
    let building_id = create_test_building(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create 3 quotes for the same building
    for i in 0..3 {
        let contractor_id = create_test_contractor(&app_state, org_id).await;
        let validity_date = Utc::now() + Duration::days(30);

        let create_req = test::TestRequest::post()
            .uri("/api/v1/quotes")
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
            .set_json(json!({
                "building_id": building_id.to_string(),
                "contractor_id": contractor_id.to_string(),
                "project_title": format!("Quote #{}", i + 1),
                "project_description": "Test quote",
                "amount_excl_vat": "5000.00",
                "vat_rate": "0.21",
                "validity_date": validity_date.to_rfc3339(),
                "estimated_duration_days": 14,
                "warranty_years": 10
            }))
            .to_request();

        test::call_service(&app, create_req).await;
    }

    // List building quotes
    let list_req = test::TestRequest::get()
        .uri(&format!("/api/v1/buildings/{}/quotes", building_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let list_resp = test::call_service(&app, list_req).await;
    assert_eq!(list_resp.status(), 200);

    let quotes: Vec<QuoteResponseDto> = test::read_body_json(list_resp).await;
    assert_eq!(quotes.len(), 3, "Expected 3 quotes for building");
}

#[actix_web::test]
#[serial]
async fn test_list_contractor_quotes() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (_user_id, token) = create_test_user(&app_state, org_id).await;
    let building1_id = create_test_building(&app_state, org_id).await;
    let building2_id = create_test_building(&app_state, org_id).await;
    let contractor_id = create_test_contractor(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create 2 quotes for the same contractor on different buildings
    for building_id in [building1_id, building2_id] {
        let validity_date = Utc::now() + Duration::days(30);

        let create_req = test::TestRequest::post()
            .uri("/api/v1/quotes")
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
            .set_json(json!({
                "building_id": building_id.to_string(),
                "contractor_id": contractor_id.to_string(),
                "project_title": "Test Quote",
                "project_description": "Test description",
                "amount_excl_vat": "5000.00",
                "vat_rate": "0.21",
                "validity_date": validity_date.to_rfc3339(),
                "estimated_duration_days": 14,
                "warranty_years": 10
            }))
            .to_request();

        test::call_service(&app, create_req).await;
    }

    // List contractor quotes
    let list_req = test::TestRequest::get()
        .uri(&format!("/api/v1/contractors/{}/quotes", contractor_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let list_resp = test::call_service(&app, list_req).await;
    assert_eq!(list_resp.status(), 200);

    let quotes: Vec<QuoteResponseDto> = test::read_body_json(list_resp).await;
    assert_eq!(quotes.len(), 2, "Expected 2 quotes for contractor");
}

#[actix_web::test]
#[serial]
async fn test_list_quotes_by_status() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (_user_id, token) = create_test_user(&app_state, org_id).await;
    let building_id = create_test_building(&app_state, org_id).await;
    let contractor_id = create_test_contractor(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let validity_date = Utc::now() + Duration::days(30);

    // Create quote
    let create_req = test::TestRequest::post()
        .uri("/api/v1/quotes")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "building_id": building_id.to_string(),
            "contractor_id": contractor_id.to_string(),
            "project_title": "Test Quote",
            "project_description": "Test description",
            "amount_excl_vat": "5000.00",
            "vat_rate": "0.21",
            "validity_date": validity_date.to_rfc3339(),
            "estimated_duration_days": 14,
            "warranty_years": 10
        }))
        .to_request();

    test::call_service(&app, create_req).await;

    // List by status (Requested)
    let list_req = test::TestRequest::get()
        .uri(&format!(
            "/api/v1/buildings/{}/quotes/status/Requested",
            building_id
        ))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let list_resp = test::call_service(&app, list_req).await;
    assert_eq!(list_resp.status(), 200);

    let quotes: Vec<QuoteResponseDto> = test::read_body_json(list_resp).await;
    assert!(
        !quotes.is_empty(),
        "Expected at least 1 quote with Requested status"
    );
}

#[actix_web::test]
#[serial]
async fn test_delete_quote() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (_user_id, token) = create_test_user(&app_state, org_id).await;
    let building_id = create_test_building(&app_state, org_id).await;
    let contractor_id = create_test_contractor(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let validity_date = Utc::now() + Duration::days(30);

    // Create quote
    let create_req = test::TestRequest::post()
        .uri("/api/v1/quotes")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "building_id": building_id.to_string(),
            "contractor_id": contractor_id.to_string(),
            "project_title": "Test Quote",
            "project_description": "Test description",
            "amount_excl_vat": "5000.00",
            "vat_rate": "0.21",
            "validity_date": validity_date.to_rfc3339(),
            "estimated_duration_days": 14,
            "warranty_years": 10
        }))
        .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    let created_quote: QuoteResponseDto = test::read_body_json(create_resp).await;

    // Delete quote
    let delete_req = test::TestRequest::delete()
        .uri(&format!("/api/v1/quotes/{}", created_quote.id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let delete_resp = test::call_service(&app, delete_req).await;
    assert_eq!(delete_resp.status(), 204);

    // Verify deletion
    let get_req = test::TestRequest::get()
        .uri(&format!("/api/v1/quotes/{}", created_quote.id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let get_resp = test::call_service(&app, get_req).await;
    assert_eq!(get_resp.status(), 404, "Expected 404 after deletion");
}

// ==================== Quote Workflow Tests ====================

#[actix_web::test]
#[serial]
async fn test_submit_quote() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (_user_id, token) = create_test_user(&app_state, org_id).await;
    let building_id = create_test_building(&app_state, org_id).await;
    let contractor_id = create_test_contractor(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let validity_date = Utc::now() + Duration::days(30);

    // Create quote (Requested)
    let create_req = test::TestRequest::post()
        .uri("/api/v1/quotes")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "building_id": building_id.to_string(),
            "contractor_id": contractor_id.to_string(),
            "project_title": "Roof Repair",
            "project_description": "Repair leaking roof",
            "amount_excl_vat": "5000.00",
            "vat_rate": "0.21",
            "validity_date": validity_date.to_rfc3339(),
            "estimated_duration_days": 14,
            "warranty_years": 10
        }))
        .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    let quote: QuoteResponseDto = test::read_body_json(create_resp).await;
    assert_eq!(quote.status, "Requested");

    // Submit quote (Requested -> Received)
    let submit_req = test::TestRequest::post()
        .uri(&format!("/api/v1/quotes/{}/submit", quote.id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "amount_excl_vat": "4800.00",
            "vat_rate": "0.21",
            "estimated_start_date": (Utc::now() + Duration::days(45)).to_rfc3339(),
            "estimated_duration_days": 12,
            "warranty_years": 10
        }))
        .to_request();

    let submit_resp = test::call_service(&app, submit_req).await;
    assert_eq!(submit_resp.status(), 200);

    let submitted_quote: QuoteResponseDto = test::read_body_json(submit_resp).await;
    assert_eq!(submitted_quote.status, "Received");
    // submit_quote only changes status, does not update amounts
    assert_eq!(submitted_quote.amount_excl_vat, "5000.00");
    assert!(submitted_quote.submitted_at.is_some());
}

#[actix_web::test]
#[serial]
async fn test_start_review() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (_user_id, token) = create_test_user(&app_state, org_id).await;
    let building_id = create_test_building(&app_state, org_id).await;
    let contractor_id = create_test_contractor(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let validity_date = Utc::now() + Duration::days(30);

    // Create and submit quote
    let create_req = test::TestRequest::post()
        .uri("/api/v1/quotes")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "building_id": building_id.to_string(),
            "contractor_id": contractor_id.to_string(),
            "project_title": "Test Quote",
            "project_description": "Test description",
            "amount_excl_vat": "5000.00",
            "vat_rate": "0.21",
            "validity_date": validity_date.to_rfc3339(),
            "estimated_duration_days": 14,
            "warranty_years": 10
        }))
        .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    let quote: QuoteResponseDto = test::read_body_json(create_resp).await;

    let submit_req = test::TestRequest::post()
        .uri(&format!("/api/v1/quotes/{}/submit", quote.id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "amount_excl_vat": "4800.00"
        }))
        .to_request();

    test::call_service(&app, submit_req).await;

    // Start review (Received -> UnderReview)
    let review_req = test::TestRequest::post()
        .uri(&format!("/api/v1/quotes/{}/review", quote.id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let review_resp = test::call_service(&app, review_req).await;
    assert_eq!(review_resp.status(), 200);

    let reviewed_quote: QuoteResponseDto = test::read_body_json(review_resp).await;
    assert_eq!(reviewed_quote.status, "UnderReview");
    assert!(reviewed_quote.reviewed_at.is_some());
}

#[actix_web::test]
#[serial]
async fn test_accept_quote() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (_user_id, token) = create_test_user(&app_state, org_id).await;
    let building_id = create_test_building(&app_state, org_id).await;
    let contractor_id = create_test_contractor(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let validity_date = Utc::now() + Duration::days(30);

    // Create, submit, and start review
    let create_req = test::TestRequest::post()
        .uri("/api/v1/quotes")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "building_id": building_id.to_string(),
            "contractor_id": contractor_id.to_string(),
            "project_title": "Test Quote",
            "project_description": "Test description",
            "amount_excl_vat": "5000.00",
            "vat_rate": "0.21",
            "validity_date": validity_date.to_rfc3339(),
            "estimated_duration_days": 14,
            "warranty_years": 10
        }))
        .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    let quote: QuoteResponseDto = test::read_body_json(create_resp).await;

    let submit_req = test::TestRequest::post()
        .uri(&format!("/api/v1/quotes/{}/submit", quote.id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({}))
        .to_request();

    test::call_service(&app, submit_req).await;

    let review_req = test::TestRequest::post()
        .uri(&format!("/api/v1/quotes/{}/review", quote.id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    test::call_service(&app, review_req).await;

    // Accept quote (UnderReview -> Accepted)
    let accept_req = test::TestRequest::post()
        .uri(&format!("/api/v1/quotes/{}/accept", quote.id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "decision_notes": "Best price and warranty combination"
        }))
        .to_request();

    let accept_resp = test::call_service(&app, accept_req).await;
    assert_eq!(accept_resp.status(), 200);

    let accepted_quote: QuoteResponseDto = test::read_body_json(accept_resp).await;
    assert_eq!(accepted_quote.status, "Accepted");
    assert!(accepted_quote.decision_at.is_some());
    assert_eq!(
        accepted_quote.decision_notes,
        Some("Best price and warranty combination".to_string())
    );
}

#[actix_web::test]
#[serial]
async fn test_reject_quote() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (_user_id, token) = create_test_user(&app_state, org_id).await;
    let building_id = create_test_building(&app_state, org_id).await;
    let contractor_id = create_test_contractor(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let validity_date = Utc::now() + Duration::days(30);

    // Create, submit, and start review
    let create_req = test::TestRequest::post()
        .uri("/api/v1/quotes")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "building_id": building_id.to_string(),
            "contractor_id": contractor_id.to_string(),
            "project_title": "Test Quote",
            "project_description": "Test description",
            "amount_excl_vat": "5000.00",
            "vat_rate": "0.21",
            "validity_date": validity_date.to_rfc3339(),
            "estimated_duration_days": 14,
            "warranty_years": 10
        }))
        .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    let quote: QuoteResponseDto = test::read_body_json(create_resp).await;

    let submit_req = test::TestRequest::post()
        .uri(&format!("/api/v1/quotes/{}/submit", quote.id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({}))
        .to_request();

    test::call_service(&app, submit_req).await;

    let review_req = test::TestRequest::post()
        .uri(&format!("/api/v1/quotes/{}/review", quote.id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    test::call_service(&app, review_req).await;

    // Reject quote (UnderReview -> Rejected)
    let reject_req = test::TestRequest::post()
        .uri(&format!("/api/v1/quotes/{}/reject", quote.id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "decision_notes": "Price too high compared to other quotes"
        }))
        .to_request();

    let reject_resp = test::call_service(&app, reject_req).await;
    assert_eq!(reject_resp.status(), 200);

    let rejected_quote: QuoteResponseDto = test::read_body_json(reject_resp).await;
    assert_eq!(rejected_quote.status, "Rejected");
    assert!(rejected_quote.decision_at.is_some());
    assert_eq!(
        rejected_quote.decision_notes,
        Some("Price too high compared to other quotes".to_string())
    );
}

#[actix_web::test]
#[serial]
async fn test_withdraw_quote() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (_user_id, token) = create_test_user(&app_state, org_id).await;
    let building_id = create_test_building(&app_state, org_id).await;
    let contractor_id = create_test_contractor(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let validity_date = Utc::now() + Duration::days(30);

    // Create quote
    let create_req = test::TestRequest::post()
        .uri("/api/v1/quotes")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "building_id": building_id.to_string(),
            "contractor_id": contractor_id.to_string(),
            "project_title": "Test Quote",
            "project_description": "Test description",
            "amount_excl_vat": "5000.00",
            "vat_rate": "0.21",
            "validity_date": validity_date.to_rfc3339(),
            "estimated_duration_days": 14,
            "warranty_years": 10
        }))
        .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    let quote: QuoteResponseDto = test::read_body_json(create_resp).await;

    // Withdraw quote (Requested -> Withdrawn)
    let withdraw_req = test::TestRequest::post()
        .uri(&format!("/api/v1/quotes/{}/withdraw", quote.id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let withdraw_resp = test::call_service(&app, withdraw_req).await;
    assert_eq!(withdraw_resp.status(), 200);

    let withdrawn_quote: QuoteResponseDto = test::read_body_json(withdraw_resp).await;
    assert_eq!(withdrawn_quote.status, "Withdrawn");
}

#[actix_web::test]
#[serial]
async fn test_update_contractor_rating() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (_user_id, token) = create_test_user(&app_state, org_id).await;
    let building_id = create_test_building(&app_state, org_id).await;
    let contractor_id = create_test_contractor(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let validity_date = Utc::now() + Duration::days(30);

    // Create quote
    let create_req = test::TestRequest::post()
        .uri("/api/v1/quotes")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "building_id": building_id.to_string(),
            "contractor_id": contractor_id.to_string(),
            "project_title": "Test Quote",
            "project_description": "Test description",
            "amount_excl_vat": "5000.00",
            "vat_rate": "0.21",
            "validity_date": validity_date.to_rfc3339(),
            "estimated_duration_days": 14,
            "warranty_years": 10
        }))
        .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    let quote: QuoteResponseDto = test::read_body_json(create_resp).await;

    // Update contractor rating (0-100 scale)
    let rating_req = test::TestRequest::put()
        .uri(&format!("/api/v1/quotes/{}/contractor-rating", quote.id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "rating": 85
        }))
        .to_request();

    let rating_resp = test::call_service(&app, rating_req).await;
    assert_eq!(rating_resp.status(), 200);

    let updated_quote: QuoteResponseDto = test::read_body_json(rating_resp).await;
    assert_eq!(updated_quote.contractor_rating, Some(85));
}

// ==================== Belgian Legal Requirement: 3 Quotes Comparison ====================

#[actix_web::test]
#[serial]
async fn test_compare_quotes_minimum_three() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (_user_id, token) = create_test_user(&app_state, org_id).await;
    let building_id = create_test_building(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create 3 quotes with different prices, durations, warranties, and ratings
    let quote_configs = vec![
        ("4500.00", 10, 10, 90), // Low price, short duration, long warranty, high rating
        ("5000.00", 14, 5, 70),  // Medium price, medium duration, short warranty, medium rating
        ("5500.00", 7, 10, 60),  // High price, very short duration, long warranty, low rating
    ];

    let mut quote_ids = Vec::new();

    for (price, duration, warranty, rating) in quote_configs {
        let contractor_id = create_test_contractor(&app_state, org_id).await;
        let validity_date = Utc::now() + Duration::days(30);

        let create_req = test::TestRequest::post()
            .uri("/api/v1/quotes")
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
            .set_json(json!({
                "building_id": building_id.to_string(),
                "contractor_id": contractor_id.to_string(),
                "project_title": "Roof Repair Project",
                "project_description": "Repair leaking roof tiles",
                "amount_excl_vat": price,
                "vat_rate": "0.21",
                "validity_date": validity_date.to_rfc3339(),
                "estimated_duration_days": duration,
                "warranty_years": warranty
            }))
            .to_request();

        let create_resp = test::call_service(&app, create_req).await;
        let quote: QuoteResponseDto = test::read_body_json(create_resp).await;

        // Submit quote
        let submit_req = test::TestRequest::post()
            .uri(&format!("/api/v1/quotes/{}/submit", quote.id))
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
            .set_json(json!({}))
            .to_request();

        test::call_service(&app, submit_req).await;

        // Update contractor rating
        let rating_req = test::TestRequest::put()
            .uri(&format!("/api/v1/quotes/{}/contractor-rating", quote.id))
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
            .set_json(json!({
                "rating": rating
            }))
            .to_request();

        test::call_service(&app, rating_req).await;

        quote_ids.push(quote.id);
    }

    // Compare quotes (Belgian legal requirement: minimum 3 quotes)
    let compare_req = test::TestRequest::post()
        .uri("/api/v1/quotes/compare")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "quote_ids": quote_ids.iter().map(|id| id.to_string()).collect::<Vec<_>>()
        }))
        .to_request();

    let compare_resp = test::call_service(&app, compare_req).await;
    assert_eq!(compare_resp.status(), 200);

    let comparison: QuoteComparisonResponseDto = test::read_body_json(compare_resp).await;
    assert_eq!(comparison.total_quotes, 3);
    assert_eq!(comparison.comparison_items.len(), 3);
    // min/max_price are based on amount_incl_vat (4500*1.21=5445, 5500*1.21=6655)
    assert_eq!(comparison.min_price, "5445.00");
    assert_eq!(comparison.max_price, "6655.00");
    assert!(
        comparison.recommended_quote_id.is_some(),
        "Expected recommended quote"
    );

    // Verify scoring order (rank 1 = best score)
    assert_eq!(comparison.comparison_items[0].rank, 1);
    assert_eq!(comparison.comparison_items[1].rank, 2);
    assert_eq!(comparison.comparison_items[2].rank, 3);

    // Verify scores are present (individual scores can be 0 for worst-in-category)
    for item in &comparison.comparison_items {
        assert!(item.score.is_some(), "Expected score for all quotes");
        let score = item.score.as_ref().unwrap();
        assert!(
            score.total_score >= 0.0,
            "Expected non-negative total score"
        );
        assert!(
            score.reputation_score >= 0.0,
            "Expected non-negative reputation score"
        );
    }
}

#[actix_web::test]
#[serial]
async fn test_compare_quotes_automatic_scoring() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (_user_id, token) = create_test_user(&app_state, org_id).await;
    let building_id = create_test_building(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create 3 quotes with clear winner: lowest price, shortest duration, longest warranty, highest rating
    let quote_configs = vec![
        ("4000.00", 5, 10, 100), // WINNER: Best on all dimensions
        ("5000.00", 14, 5, 70),  // Average
        ("6000.00", 21, 2, 50),  // Worst
    ];

    let mut quote_ids = Vec::new();

    for (price, duration, warranty, rating) in quote_configs {
        let contractor_id = create_test_contractor(&app_state, org_id).await;
        let validity_date = Utc::now() + Duration::days(30);

        let create_req = test::TestRequest::post()
            .uri("/api/v1/quotes")
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
            .set_json(json!({
                "building_id": building_id.to_string(),
                "contractor_id": contractor_id.to_string(),
                "project_title": "Roof Repair Project",
                "project_description": "Repair leaking roof tiles",
                "amount_excl_vat": price,
                "vat_rate": "0.21",
                "validity_date": validity_date.to_rfc3339(),
                "estimated_duration_days": duration,
                "warranty_years": warranty
            }))
            .to_request();

        let create_resp = test::call_service(&app, create_req).await;
        let quote: QuoteResponseDto = test::read_body_json(create_resp).await;

        let submit_req = test::TestRequest::post()
            .uri(&format!("/api/v1/quotes/{}/submit", quote.id))
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
            .set_json(json!({}))
            .to_request();

        test::call_service(&app, submit_req).await;

        let rating_req = test::TestRequest::put()
            .uri(&format!("/api/v1/quotes/{}/contractor-rating", quote.id))
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
            .set_json(json!({
                "contractor_rating": rating
            }))
            .to_request();

        test::call_service(&app, rating_req).await;

        quote_ids.push(quote.id);
    }

    // Compare quotes
    let compare_req = test::TestRequest::post()
        .uri("/api/v1/quotes/compare")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "quote_ids": quote_ids.iter().map(|id| id.to_string()).collect::<Vec<_>>()
        }))
        .to_request();

    let compare_resp = test::call_service(&app, compare_req).await;
    let comparison: QuoteComparisonResponseDto = test::read_body_json(compare_resp).await;

    // The first quote (4000.00, 5 days, 10 years warranty, 100 rating) should be rank 1 (best)
    assert_eq!(comparison.comparison_items[0].rank, 1);
    assert_eq!(
        comparison.comparison_items[0].quote.amount_excl_vat,
        "4000.00"
    );
    assert_eq!(
        comparison.recommended_quote_id,
        Some(quote_ids[0].to_string())
    );

    // Verify scoring algorithm weights: Price 40%, Delay 30%, Warranty 20%, Reputation 10%
    // The best quote should have the highest total score
    let best_score = comparison.comparison_items[0].score.as_ref().unwrap();
    let worst_score = comparison.comparison_items[2].score.as_ref().unwrap();
    assert!(
        best_score.total_score > worst_score.total_score,
        "Best quote should have higher total score"
    );
}

// ==================== Complete Lifecycle Test ====================

#[actix_web::test]
#[serial]
async fn test_complete_quote_lifecycle() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (_user_id, token) = create_test_user(&app_state, org_id).await;
    let building_id = create_test_building(&app_state, org_id).await;
    let contractor_id = create_test_contractor(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let validity_date = Utc::now() + Duration::days(30);

    // 1. Create quote (Requested)
    let create_req = test::TestRequest::post()
        .uri("/api/v1/quotes")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "building_id": building_id.to_string(),
            "contractor_id": contractor_id.to_string(),
            "project_title": "Roof Repair Project",
            "project_description": "Repair leaking roof tiles and replace gutters",
            "amount_excl_vat": "5000.00",
            "vat_rate": "0.21",
            "validity_date": validity_date.to_rfc3339(),
            "estimated_start_date": (Utc::now() + Duration::days(45)).to_rfc3339(),
            "estimated_duration_days": 14,
            "warranty_years": 10
        }))
        .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    let quote: QuoteResponseDto = test::read_body_json(create_resp).await;
    assert_eq!(quote.status, "Requested");

    // 2. Contractor submits quote (Requested -> Received)
    let submit_req = test::TestRequest::post()
        .uri(&format!("/api/v1/quotes/{}/submit", quote.id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "amount_excl_vat": "4800.00",
            "vat_rate": "0.21",
            "estimated_start_date": (Utc::now() + Duration::days(40)).to_rfc3339(),
            "estimated_duration_days": 12,
            "warranty_years": 10
        }))
        .to_request();

    let submit_resp = test::call_service(&app, submit_req).await;
    let submitted_quote: QuoteResponseDto = test::read_body_json(submit_resp).await;
    assert_eq!(submitted_quote.status, "Received");

    // 3. Update contractor rating
    let rating_req = test::TestRequest::put()
        .uri(&format!("/api/v1/quotes/{}/contractor-rating", quote.id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "rating": 85
        }))
        .to_request();

    test::call_service(&app, rating_req).await;

    // 4. Syndic starts review (Received -> UnderReview)
    let review_req = test::TestRequest::post()
        .uri(&format!("/api/v1/quotes/{}/review", quote.id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let review_resp = test::call_service(&app, review_req).await;
    let reviewed_quote: QuoteResponseDto = test::read_body_json(review_resp).await;
    assert_eq!(reviewed_quote.status, "UnderReview");

    // 5. Accept quote (UnderReview -> Accepted)
    let accept_req = test::TestRequest::post()
        .uri(&format!("/api/v1/quotes/{}/accept", quote.id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "decision_notes": "Best price and warranty combination. Contractor has excellent reputation."
        }))
        .to_request();

    let accept_resp = test::call_service(&app, accept_req).await;
    let accepted_quote: QuoteResponseDto = test::read_body_json(accept_resp).await;
    assert_eq!(accepted_quote.status, "Accepted");
    assert!(accepted_quote.decision_at.is_some());
    assert!(accepted_quote.decision_by.is_some());
    assert_eq!(
        accepted_quote.decision_notes,
        Some(
            "Best price and warranty combination. Contractor has excellent reputation.".to_string()
        )
    );

    // 6. Verify final quote state
    let get_req = test::TestRequest::get()
        .uri(&format!("/api/v1/quotes/{}", quote.id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let get_resp = test::call_service(&app, get_req).await;
    let final_quote: QuoteResponseDto = test::read_body_json(get_resp).await;

    assert_eq!(final_quote.status, "Accepted");
    // submit_quote only changes status, does not update amounts or duration
    assert_eq!(final_quote.amount_excl_vat, "5000.00");
    assert_eq!(final_quote.amount_incl_vat, "6050.00"); // 5000 * 1.21
    assert_eq!(final_quote.estimated_duration_days, 14);
    assert_eq!(final_quote.warranty_years, 10);
    assert_eq!(final_quote.contractor_rating, Some(85));
    assert!(!final_quote.is_expired);
}

// ==================== Count Endpoints ====================

#[actix_web::test]
#[serial]
async fn test_count_building_quotes() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (_user_id, token) = create_test_user(&app_state, org_id).await;
    let building_id = create_test_building(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create 5 quotes
    for _ in 0..5 {
        let contractor_id = create_test_contractor(&app_state, org_id).await;
        let validity_date = Utc::now() + Duration::days(30);

        let create_req = test::TestRequest::post()
            .uri("/api/v1/quotes")
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
            .set_json(json!({
                "building_id": building_id.to_string(),
                "contractor_id": contractor_id.to_string(),
                "project_title": "Test Quote",
                "project_description": "Test description",
                "amount_excl_vat": "5000.00",
                "vat_rate": "0.21",
                "validity_date": validity_date.to_rfc3339(),
                "estimated_duration_days": 14,
                "warranty_years": 10
            }))
            .to_request();

        test::call_service(&app, create_req).await;
    }

    // Count building quotes
    let count_req = test::TestRequest::get()
        .uri(&format!("/api/v1/buildings/{}/quotes/count", building_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let count_resp = test::call_service(&app, count_req).await;
    assert_eq!(count_resp.status(), 200);

    let count: serde_json::Value = test::read_body_json(count_resp).await;
    assert_eq!(count["count"], 5);
}

#[actix_web::test]
#[serial]
async fn test_count_quotes_by_status() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (_user_id, token) = create_test_user(&app_state, org_id).await;
    let building_id = create_test_building(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create 3 quotes with Requested status
    for _ in 0..3 {
        let contractor_id = create_test_contractor(&app_state, org_id).await;
        let validity_date = Utc::now() + Duration::days(30);

        let create_req = test::TestRequest::post()
            .uri("/api/v1/quotes")
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
            .set_json(json!({
                "building_id": building_id.to_string(),
                "contractor_id": contractor_id.to_string(),
                "project_title": "Test Quote",
                "project_description": "Test description",
                "amount_excl_vat": "5000.00",
                "vat_rate": "0.21",
                "validity_date": validity_date.to_rfc3339(),
                "estimated_duration_days": 14,
                "warranty_years": 10
            }))
            .to_request();

        test::call_service(&app, create_req).await;
    }

    // Count quotes by status (Requested)
    let count_req = test::TestRequest::get()
        .uri(&format!(
            "/api/v1/buildings/{}/quotes/status/Requested/count",
            building_id
        ))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let count_resp = test::call_service(&app, count_req).await;
    assert_eq!(count_resp.status(), 200);

    let count: serde_json::Value = test::read_body_json(count_resp).await;
    assert_eq!(count["count"], 3);
}
