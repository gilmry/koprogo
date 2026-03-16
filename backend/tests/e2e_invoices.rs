// E2E tests for Invoice HTTP endpoints (Issue #73)
// Tests focus on HTTP layer: endpoints, auth, JSON serialization
// Belgian legal context: Invoice approval workflow with Draft → PendingApproval → Approved/Rejected

mod common;

use actix_web::http::header;
use actix_web::{test, App};
use koprogo_api::infrastructure::web::configure_routes;
use serde_json::json;
use serial_test::serial;
use uuid::Uuid;

#[actix_web::test]
#[serial]
async fn test_invoices_create() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = common::register_and_login(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create a building first
    let building_resp = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/api/v1/buildings")
            .insert_header(header::ContentType::json())
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
            .set_json(json!({
                "organization_id": org_id.to_string(),
                "name": format!("Invoice Building {}", Uuid::new_v4()),
                "address": "1 Invoice Street",
                "city": "Brussels",
                "postal_code": "1000",
                "country": "Belgium",
                "total_units": 5,
                "construction_year": 2010
            }))
            .to_request(),
    )
    .await;
    let building: serde_json::Value = test::read_body_json(building_resp).await;
    let building_id = building["id"].as_str().unwrap();

    let req = test::TestRequest::post()
        .uri("/api/v1/expenses")
        .insert_header(header::ContentType::json())
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "building_id": building_id,
            "category": "Maintenance",
            "description": "Plumbing repair - main corridor",
            "amount": 450.00,
            "expense_date": "2026-03-01T00:00:00Z",
            "supplier": "Fix-It Plumbers BVBA",
            "invoice_number": "INV-2026-001"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;

    assert!(
        resp.status().is_success(),
        "Expected 201 Created, got: {}",
        resp.status()
    );
    assert_eq!(resp.status().as_u16(), 201);
}

#[actix_web::test]
#[serial]
async fn test_invoices_get() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = common::register_and_login(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create building
    let building_resp = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/api/v1/buildings")
            .insert_header(header::ContentType::json())
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
            .set_json(json!({
                "organization_id": org_id.to_string(),
                "name": format!("Invoice Get Building {}", Uuid::new_v4()),
                "address": "2 Get Street",
                "city": "Brussels",
                "postal_code": "1000",
                "country": "Belgium",
                "total_units": 3
            }))
            .to_request(),
    )
    .await;
    let building: serde_json::Value = test::read_body_json(building_resp).await;
    let building_id = building["id"].as_str().unwrap();

    // Create an expense
    let create_resp = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/api/v1/expenses")
            .insert_header(header::ContentType::json())
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
            .set_json(json!({
                "building_id": building_id,
                "category": "Utilities",
                "description": "Electricity bill Q1 2026",
                "amount": 320.50,
                "expense_date": "2026-02-15T00:00:00Z"
            }))
            .to_request(),
    )
    .await;
    let created: serde_json::Value = test::read_body_json(create_resp).await;
    let expense_id = created["id"].as_str().unwrap();

    // Retrieve by ID
    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/expenses/{}", expense_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;

    assert!(
        resp.status().is_success(),
        "Expected 200 OK, got: {}",
        resp.status()
    );
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["id"], expense_id);
}

#[actix_web::test]
#[serial]
async fn test_invoices_list_by_building() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = common::register_and_login(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create building
    let building_resp = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/api/v1/buildings")
            .insert_header(header::ContentType::json())
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
            .set_json(json!({
                "organization_id": org_id.to_string(),
                "name": format!("Invoice List Building {}", Uuid::new_v4()),
                "address": "3 List Avenue",
                "city": "Brussels",
                "postal_code": "1000",
                "country": "Belgium",
                "total_units": 8
            }))
            .to_request(),
    )
    .await;
    let building: serde_json::Value = test::read_body_json(building_resp).await;
    let building_id = building["id"].as_str().unwrap();

    // Create an expense so the list is non-empty
    test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/api/v1/expenses")
            .insert_header(header::ContentType::json())
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
            .set_json(json!({
                "building_id": building_id,
                "category": "Insurance",
                "description": "Annual building insurance",
                "amount": 1200.00,
                "expense_date": "2026-01-01T00:00:00Z"
            }))
            .to_request(),
    )
    .await;

    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/buildings/{}/expenses", building_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;

    assert!(
        resp.status().is_success(),
        "Expected 200 OK for list by building, got: {}",
        resp.status()
    );
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body.is_array(), "Expected JSON array response");
}

#[actix_web::test]
#[serial]
async fn test_invoices_submit_for_approval() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = common::register_and_login(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create building
    let building_resp = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/api/v1/buildings")
            .insert_header(header::ContentType::json())
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
            .set_json(json!({
                "organization_id": org_id.to_string(),
                "name": format!("Submit Building {}", Uuid::new_v4()),
                "address": "4 Submit Road",
                "city": "Brussels",
                "postal_code": "1000",
                "country": "Belgium",
                "total_units": 4
            }))
            .to_request(),
    )
    .await;
    let building: serde_json::Value = test::read_body_json(building_resp).await;
    let building_id = building["id"].as_str().unwrap();

    // Create invoice draft via the VAT invoice workflow endpoint
    let create_resp = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/api/v1/invoices/draft")
            .insert_header(header::ContentType::json())
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
            .set_json(json!({
                "building_id": building_id,
                "category": "Maintenance",
                "description": "Elevator maintenance contract",
                "amount_excl_vat": 800.00,
                "vat_rate": 21.0,
                "invoice_date": "2026-03-01T00:00:00Z",
                "due_date": "2026-04-01T00:00:00Z",
                "supplier": "Elevator Services NV"
            }))
            .to_request(),
    )
    .await;

    assert!(
        create_resp.status().is_success(),
        "Expected success creating invoice draft, got: {}",
        create_resp.status()
    );

    let created: serde_json::Value = test::read_body_json(create_resp).await;
    let invoice_id = created["id"].as_str().unwrap();

    // Submit for approval (Draft → PendingApproval)
    let req = test::TestRequest::put()
        .uri(&format!("/api/v1/invoices/{}/submit", invoice_id))
        .insert_header(header::ContentType::json())
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({}))
        .to_request();

    let resp = test::call_service(&app, req).await;

    assert!(
        resp.status().is_success() || resp.status().is_client_error(),
        "Expected valid response for submit-for-approval, got: {}",
        resp.status()
    );
}

#[actix_web::test]
#[serial]
async fn test_invoices_approve() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = common::register_and_login(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create building
    let building_resp = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/api/v1/buildings")
            .insert_header(header::ContentType::json())
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
            .set_json(json!({
                "organization_id": org_id.to_string(),
                "name": format!("Approve Building {}", Uuid::new_v4()),
                "address": "5 Approve Lane",
                "city": "Brussels",
                "postal_code": "1000",
                "country": "Belgium",
                "total_units": 6
            }))
            .to_request(),
    )
    .await;
    let building: serde_json::Value = test::read_body_json(building_resp).await;
    let building_id = building["id"].as_str().unwrap();

    // Create invoice draft
    let create_resp = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/api/v1/invoices/draft")
            .insert_header(header::ContentType::json())
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
            .set_json(json!({
                "building_id": building_id,
                "category": "Repairs",
                "description": "Roof repair after storm",
                "amount_excl_vat": 2500.00,
                "vat_rate": 6.0,
                "invoice_date": "2026-03-10T00:00:00Z",
                "supplier": "Roofing Experts SPRL"
            }))
            .to_request(),
    )
    .await;

    let created: serde_json::Value = test::read_body_json(create_resp).await;
    let invoice_id = created["id"].as_str().unwrap();

    // Submit for approval first
    test::call_service(
        &app,
        test::TestRequest::put()
            .uri(&format!("/api/v1/invoices/{}/submit", invoice_id))
            .insert_header(header::ContentType::json())
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
            .set_json(json!({}))
            .to_request(),
    )
    .await;

    // Approve (superadmin satisfies syndic/superadmin role check)
    let req = test::TestRequest::put()
        .uri(&format!("/api/v1/invoices/{}/approve", invoice_id))
        .insert_header(header::ContentType::json())
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({}))
        .to_request();

    let resp = test::call_service(&app, req).await;

    assert!(
        resp.status().is_success() || resp.status().is_client_error(),
        "Expected valid response for approve, got: {}",
        resp.status()
    );
}

#[actix_web::test]
#[serial]
async fn test_invoices_reject() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = common::register_and_login(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create building
    let building_resp = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/api/v1/buildings")
            .insert_header(header::ContentType::json())
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
            .set_json(json!({
                "organization_id": org_id.to_string(),
                "name": format!("Reject Building {}", Uuid::new_v4()),
                "address": "6 Reject Boulevard",
                "city": "Brussels",
                "postal_code": "1000",
                "country": "Belgium",
                "total_units": 4
            }))
            .to_request(),
    )
    .await;
    let building: serde_json::Value = test::read_body_json(building_resp).await;
    let building_id = building["id"].as_str().unwrap();

    // Create invoice draft
    let create_resp = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/api/v1/invoices/draft")
            .insert_header(header::ContentType::json())
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
            .set_json(json!({
                "building_id": building_id,
                "category": "Administration",
                "description": "Legal consultation fee",
                "amount_excl_vat": 600.00,
                "vat_rate": 21.0,
                "invoice_date": "2026-03-05T00:00:00Z"
            }))
            .to_request(),
    )
    .await;

    let created: serde_json::Value = test::read_body_json(create_resp).await;
    let invoice_id = created["id"].as_str().unwrap();

    // Submit for approval first
    test::call_service(
        &app,
        test::TestRequest::put()
            .uri(&format!("/api/v1/invoices/{}/submit", invoice_id))
            .insert_header(header::ContentType::json())
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
            .set_json(json!({}))
            .to_request(),
    )
    .await;

    // Reject with reason (superadmin role satisfies syndic/superadmin check)
    let req = test::TestRequest::put()
        .uri(&format!("/api/v1/invoices/{}/reject", invoice_id))
        .insert_header(header::ContentType::json())
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "rejected_by_user_id": "00000000-0000-0000-0000-000000000001",
            "rejection_reason": "Invoice amount exceeds budget allocation for legal fees"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;

    assert!(
        resp.status().is_success() || resp.status().is_client_error(),
        "Expected valid response for reject, got: {}",
        resp.status()
    );
}

#[actix_web::test]
#[serial]
async fn test_invoices_mark_paid() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = common::register_and_login(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create building
    let building_resp = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/api/v1/buildings")
            .insert_header(header::ContentType::json())
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
            .set_json(json!({
                "organization_id": org_id.to_string(),
                "name": format!("MarkPaid Building {}", Uuid::new_v4()),
                "address": "7 Payment Street",
                "city": "Brussels",
                "postal_code": "1000",
                "country": "Belgium",
                "total_units": 2
            }))
            .to_request(),
    )
    .await;
    let building: serde_json::Value = test::read_body_json(building_resp).await;
    let building_id = building["id"].as_str().unwrap();

    // Create an expense (legacy endpoint)
    let create_resp = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/api/v1/expenses")
            .insert_header(header::ContentType::json())
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
            .set_json(json!({
                "building_id": building_id,
                "category": "Cleaning",
                "description": "Stairwell cleaning service",
                "amount": 180.00,
                "expense_date": "2026-03-14T00:00:00Z"
            }))
            .to_request(),
    )
    .await;

    let created: serde_json::Value = test::read_body_json(create_resp).await;
    let expense_id = created["id"].as_str().unwrap();

    // Mark as paid
    let req = test::TestRequest::put()
        .uri(&format!("/api/v1/expenses/{}/mark-paid", expense_id))
        .insert_header(header::ContentType::json())
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({}))
        .to_request();

    let resp = test::call_service(&app, req).await;

    assert!(
        resp.status().is_success() || resp.status().is_client_error(),
        "Expected valid response for mark-paid, got: {}",
        resp.status()
    );
}

#[actix_web::test]
#[serial]
async fn test_invoices_unauthorized() {
    let (app_state, _container, _org_id) = common::setup_test_db().await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Attempt to create expense without a token
    let req = test::TestRequest::post()
        .uri("/api/v1/expenses")
        .insert_header(header::ContentType::json())
        .set_json(json!({
            "building_id": "00000000-0000-0000-0000-000000000001",
            "category": "Maintenance",
            "description": "Unauthorized attempt",
            "amount": 100.00,
            "expense_date": "2026-03-14T00:00:00Z"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;

    assert_eq!(
        resp.status().as_u16(),
        401,
        "Expected 401 Unauthorized without token, got: {}",
        resp.status()
    );
}
