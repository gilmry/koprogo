// E2E tests for payment & payment method HTTP endpoints (Issue #84)
// Tests focus on HTTP layer: endpoints, auth, JSON serialization
// Covers Stripe Payment Integration + SEPA Direct Debit workflows

mod common;

use actix_web::http::header;
use actix_web::{test, App};
use chrono::{Duration, Utc};
use koprogo_api::application::dto::*;
use koprogo_api::infrastructure::web::configure_routes;
use koprogo_api::infrastructure::web::AppState;
use serde_json::json;
use serial_test::serial;
use uuid::Uuid;

/// Helper: Create test fixtures (building, owner, expense) using org_id from common setup
async fn create_test_fixtures(
    app_state: &actix_web::web::Data<AppState>,
    org_id: Uuid,
) -> (String, Uuid, Uuid, Uuid, Uuid) {
    // 1. Register user and get token
    let token = common::register_and_login(app_state, org_id).await;

    // 2. Create building
    let building_dto = CreateBuildingDto {
        organization_id: org_id.to_string(),
        name: format!("Test Building Payment {}", Uuid::new_v4()),
        address: "456 Stripe Ave".to_string(),
        city: "Brussels".to_string(),
        postal_code: "1000".to_string(),
        country: "Belgium".to_string(),
        total_units: 5,
        total_tantiemes: Some(1000),
        construction_year: Some(2020),
    };

    let building = app_state
        .building_use_cases
        .create_building(building_dto)
        .await
        .expect("Failed to create building");

    let building_id = Uuid::parse_str(&building.id).expect("Failed to parse building id");

    // 3. Create owner
    let owner_dto = CreateOwnerDto {
        organization_id: org_id.to_string(),
        first_name: "Payment".to_string(),
        last_name: "Owner".to_string(),
        email: format!("payment-owner-{}@example.com", Uuid::new_v4()),
        phone: Some("+32 2 999 9999".to_string()),
        address: "123 Payment St".to_string(),
        city: "Brussels".to_string(),
        postal_code: "1000".to_string(),
        country: "Belgium".to_string(),
    };

    let owner = app_state
        .owner_use_cases
        .create_owner(owner_dto)
        .await
        .expect("Failed to create owner");

    let owner_id = Uuid::parse_str(&owner.id).expect("Failed to parse owner id");

    // 4. Create expense
    let expense_dto = CreateExpenseDto {
        organization_id: org_id.to_string(),
        building_id: building_id.to_string(),
        category: koprogo_api::domain::entities::ExpenseCategory::Maintenance,
        amount: 500.00,
        description: "Test expense for payment".to_string(),
        expense_date: Utc::now().to_rfc3339(),
        supplier: Some("Test Vendor".to_string()),
        invoice_number: None,
        account_code: None,
    };

    let expense = app_state
        .expense_use_cases
        .create_expense(expense_dto)
        .await
        .expect("Failed to create expense");

    let expense_id = Uuid::parse_str(&expense.id).expect("Failed to parse expense id");

    (token, org_id, building_id, owner_id, expense_id)
}

// ==================== Payment Tests ====================

#[actix_web::test]
#[serial]
async fn test_create_payment_success() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, _org_id, building_id, owner_id, expense_id) =
        create_test_fixtures(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/v1/payments")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "building_id": building_id.to_string(),
            "owner_id": owner_id.to_string(),
            "expense_id": expense_id.to_string(),
            "amount_cents": 50000, // 500.00 EUR
            "payment_method_type": "card",
            "description": "Payment for maintenance expense"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201, "Should create payment successfully");

    let payment: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(payment["amount_cents"], 50000);
    assert_eq!(payment["currency"], "EUR");
    assert_eq!(payment["status"], "pending");
    assert_eq!(payment["payment_method_type"], "card");
    assert_eq!(payment["refunded_amount_cents"], 0);
    assert_eq!(payment["net_amount_cents"], 50000);
}

#[actix_web::test]
#[serial]
async fn test_create_payment_without_auth_fails() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (_token, _org_id, building_id, owner_id, expense_id) =
        create_test_fixtures(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/v1/payments")
        .set_json(json!({
            "building_id": building_id.to_string(),
            "owner_id": owner_id.to_string(),
            "expense_id": expense_id.to_string(),
            "amount_cents": 50000,
            "payment_method_type": "card"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401, "Should require authentication");
}

#[actix_web::test]
#[serial]
async fn test_get_payment_by_id() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, _org_id, building_id, owner_id, expense_id) =
        create_test_fixtures(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create payment first
    let create_req = test::TestRequest::post()
        .uri("/api/v1/payments")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "building_id": building_id.to_string(),
            "owner_id": owner_id.to_string(),
            "expense_id": expense_id.to_string(),
            "amount_cents": 75000,
            "payment_method_type": "sepa_debit",
            "description": "SEPA payment test"
        }))
        .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    let payment: serde_json::Value = test::read_body_json(create_resp).await;
    let payment_id = payment["id"].as_str().unwrap();

    // Get payment by ID
    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/payments/{}", payment_id))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let fetched: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(fetched["id"], payment_id);
    assert_eq!(fetched["amount_cents"], 75000);
    assert_eq!(fetched["payment_method_type"], "sepa_debit");
}

#[actix_web::test]
#[serial]
async fn test_get_payment_not_found() {
    let (app_state, _container, _org_id) = common::setup_test_db().await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let fake_id = Uuid::new_v4();
    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/payments/{}", fake_id))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 404);
}

#[actix_web::test]
#[serial]
async fn test_list_owner_payments() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, _org_id, building_id, owner_id, expense_id) =
        create_test_fixtures(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create 3 payments for the same owner
    for i in 1..=3 {
        let req = test::TestRequest::post()
            .uri("/api/v1/payments")
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
            .set_json(json!({
                "building_id": building_id.to_string(),
                "owner_id": owner_id.to_string(),
                "expense_id": expense_id.to_string(),
                "amount_cents": 10000 * i,
                "payment_method_type": "card"
            }))
            .to_request();

        test::call_service(&app, req).await;
    }

    // List all payments for the owner
    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/owners/{}/payments", owner_id))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let payments: serde_json::Value = test::read_body_json(resp).await;
    let payments_array = payments.as_array().unwrap();
    assert_eq!(payments_array.len(), 3, "Should return all 3 payments");
}

#[actix_web::test]
#[serial]
async fn test_list_building_payments() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, _org_id, building_id, owner_id, expense_id) =
        create_test_fixtures(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create 2 payments for the building
    for i in 1..=2 {
        let req = test::TestRequest::post()
            .uri("/api/v1/payments")
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
            .set_json(json!({
                "building_id": building_id.to_string(),
                "owner_id": owner_id.to_string(),
                "expense_id": expense_id.to_string(),
                "amount_cents": 20000 * i,
                "payment_method_type": "bank_transfer"
            }))
            .to_request();

        test::call_service(&app, req).await;
    }

    // List all payments for the building
    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/buildings/{}/payments", building_id))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let payments: serde_json::Value = test::read_body_json(resp).await;
    assert!(payments.as_array().unwrap().len() >= 2);
}

#[actix_web::test]
#[serial]
async fn test_list_expense_payments() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, _org_id, building_id, owner_id, expense_id) =
        create_test_fixtures(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create payment for specific expense
    let req = test::TestRequest::post()
        .uri("/api/v1/payments")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "building_id": building_id.to_string(),
            "owner_id": owner_id.to_string(),
            "expense_id": expense_id.to_string(),
            "amount_cents": 30000,
            "payment_method_type": "card"
        }))
        .to_request();

    test::call_service(&app, req).await;

    // List payments for the expense
    let list_req = test::TestRequest::get()
        .uri(&format!("/api/v1/expenses/{}/payments", expense_id))
        .to_request();

    let resp = test::call_service(&app, list_req).await;
    assert_eq!(resp.status(), 200);

    let payments: serde_json::Value = test::read_body_json(resp).await;
    assert!(!payments.as_array().unwrap().is_empty());
}

#[actix_web::test]
#[serial]
async fn test_payment_status_transitions() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, _org_id, building_id, owner_id, expense_id) =
        create_test_fixtures(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create payment
    let create_req = test::TestRequest::post()
        .uri("/api/v1/payments")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "building_id": building_id.to_string(),
            "owner_id": owner_id.to_string(),
            "expense_id": expense_id.to_string(),
            "amount_cents": 100000,
            "payment_method_type": "card",
            "description": "Payment lifecycle test"
        }))
        .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    let payment: serde_json::Value = test::read_body_json(create_resp).await;
    let payment_id = payment["id"].as_str().unwrap();
    assert_eq!(payment["status"], "pending");

    // Mark as Processing
    let processing_req = test::TestRequest::put()
        .uri(&format!("/api/v1/payments/{}/processing", payment_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let processing_resp = test::call_service(&app, processing_req).await;
    assert_eq!(processing_resp.status(), 200);
    let updated: serde_json::Value = test::read_body_json(processing_resp).await;
    assert_eq!(updated["status"], "processing");

    // Mark as Succeeded
    let succeeded_req = test::TestRequest::put()
        .uri(&format!("/api/v1/payments/{}/succeeded", payment_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let succeeded_resp = test::call_service(&app, succeeded_req).await;
    assert_eq!(succeeded_resp.status(), 200);
    let succeeded: serde_json::Value = test::read_body_json(succeeded_resp).await;
    assert_eq!(succeeded["status"], "succeeded");
    assert!(succeeded["succeeded_at"].is_string());
}

#[actix_web::test]
#[serial]
async fn test_payment_failed_status() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, _org_id, building_id, owner_id, expense_id) =
        create_test_fixtures(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create payment
    let create_req = test::TestRequest::post()
        .uri("/api/v1/payments")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "building_id": building_id.to_string(),
            "owner_id": owner_id.to_string(),
            "expense_id": expense_id.to_string(),
            "amount_cents": 25000,
            "payment_method_type": "card"
        }))
        .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    let payment: serde_json::Value = test::read_body_json(create_resp).await;
    let payment_id = payment["id"].as_str().unwrap();

    // Mark as Processing
    let processing_req = test::TestRequest::put()
        .uri(&format!("/api/v1/payments/{}/processing", payment_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    test::call_service(&app, processing_req).await;

    // Mark as Failed
    let failed_req = test::TestRequest::put()
        .uri(&format!("/api/v1/payments/{}/failed", payment_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "failure_reason": "Insufficient funds"
        }))
        .to_request();

    let failed_resp = test::call_service(&app, failed_req).await;
    assert_eq!(failed_resp.status(), 200);
    let failed: serde_json::Value = test::read_body_json(failed_resp).await;
    assert_eq!(failed["status"], "failed");
    assert!(failed["failed_at"].is_string());
}

#[actix_web::test]
#[serial]
async fn test_payment_cancelled() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, _org_id, building_id, owner_id, expense_id) =
        create_test_fixtures(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create payment
    let create_req = test::TestRequest::post()
        .uri("/api/v1/payments")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "building_id": building_id.to_string(),
            "owner_id": owner_id.to_string(),
            "expense_id": expense_id.to_string(),
            "amount_cents": 35000,
            "payment_method_type": "card"
        }))
        .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    let payment: serde_json::Value = test::read_body_json(create_resp).await;
    let payment_id = payment["id"].as_str().unwrap();

    // Cancel payment
    let cancel_req = test::TestRequest::put()
        .uri(&format!("/api/v1/payments/{}/cancelled", payment_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let cancel_resp = test::call_service(&app, cancel_req).await;
    assert_eq!(cancel_resp.status(), 200);
    let cancelled: serde_json::Value = test::read_body_json(cancel_resp).await;
    assert_eq!(cancelled["status"], "cancelled");
    assert!(cancelled["cancelled_at"].is_string());
}

#[actix_web::test]
#[serial]
async fn test_refund_payment() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, _org_id, building_id, owner_id, expense_id) =
        create_test_fixtures(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create and succeed payment
    let create_req = test::TestRequest::post()
        .uri("/api/v1/payments")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "building_id": building_id.to_string(),
            "owner_id": owner_id.to_string(),
            "expense_id": expense_id.to_string(),
            "amount_cents": 120000, // 1200.00 EUR
            "payment_method_type": "card"
        }))
        .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    let payment: serde_json::Value = test::read_body_json(create_resp).await;
    let payment_id = payment["id"].as_str().unwrap();

    // Mark as succeeded
    let succeeded_req = test::TestRequest::put()
        .uri(&format!("/api/v1/payments/{}/succeeded", payment_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    test::call_service(&app, succeeded_req).await;

    // Refund partial amount
    let refund_req = test::TestRequest::post()
        .uri(&format!("/api/v1/payments/{}/refund", payment_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "amount_cents": 30000, // Refund 300.00 EUR
            "reason": "Partial refund test"
        }))
        .to_request();

    let refund_resp = test::call_service(&app, refund_req).await;
    assert_eq!(refund_resp.status(), 200);

    let refunded: serde_json::Value = test::read_body_json(refund_resp).await;
    // Partial refund keeps status as "succeeded" (only full refund changes to "refunded")
    assert_eq!(refunded["status"], "succeeded");
    assert_eq!(refunded["refunded_amount_cents"], 30000);
}

#[actix_web::test]
#[serial]
async fn test_list_payments_by_status() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, _org_id, building_id, owner_id, expense_id) =
        create_test_fixtures(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create payment and mark as succeeded
    let create_req = test::TestRequest::post()
        .uri("/api/v1/payments")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "building_id": building_id.to_string(),
            "owner_id": owner_id.to_string(),
            "expense_id": expense_id.to_string(),
            "amount_cents": 50000,
            "payment_method_type": "card"
        }))
        .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    let payment: serde_json::Value = test::read_body_json(create_resp).await;
    let payment_id = payment["id"].as_str().unwrap();

    let succeeded_req = test::TestRequest::put()
        .uri(&format!("/api/v1/payments/{}/succeeded", payment_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    test::call_service(&app, succeeded_req).await;

    // List succeeded payments
    let list_req = test::TestRequest::get()
        .uri("/api/v1/payments/status/succeeded")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, list_req).await;
    assert_eq!(resp.status(), 200);

    let payments: serde_json::Value = test::read_body_json(resp).await;
    assert!(!payments.as_array().unwrap().is_empty());
}

#[actix_web::test]
#[serial]
async fn test_list_pending_payments() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, _org_id, building_id, owner_id, expense_id) =
        create_test_fixtures(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create pending payment
    let req = test::TestRequest::post()
        .uri("/api/v1/payments")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "building_id": building_id.to_string(),
            "owner_id": owner_id.to_string(),
            "expense_id": expense_id.to_string(),
            "amount_cents": 15000,
            "payment_method_type": "sepa_debit"
        }))
        .to_request();

    test::call_service(&app, req).await;

    // List pending payments
    let list_req = test::TestRequest::get()
        .uri("/api/v1/payments/pending")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, list_req).await;
    assert_eq!(resp.status(), 200);

    let payments: serde_json::Value = test::read_body_json(resp).await;
    assert!(!payments.as_array().unwrap().is_empty());
}

#[actix_web::test]
#[serial]
async fn test_delete_payment() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, _org_id, building_id, owner_id, expense_id) =
        create_test_fixtures(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create payment
    let create_req = test::TestRequest::post()
        .uri("/api/v1/payments")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "building_id": building_id.to_string(),
            "owner_id": owner_id.to_string(),
            "expense_id": expense_id.to_string(),
            "amount_cents": 40000,
            "payment_method_type": "card"
        }))
        .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    let payment: serde_json::Value = test::read_body_json(create_resp).await;
    let payment_id = payment["id"].as_str().unwrap();

    // Delete payment
    let delete_req = test::TestRequest::delete()
        .uri(&format!("/api/v1/payments/{}", payment_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let delete_resp = test::call_service(&app, delete_req).await;
    assert_eq!(delete_resp.status(), 204);

    // Verify deletion
    let get_req = test::TestRequest::get()
        .uri(&format!("/api/v1/payments/{}", payment_id))
        .to_request();

    let get_resp = test::call_service(&app, get_req).await;
    assert_eq!(get_resp.status(), 404);
}

#[actix_web::test]
#[serial]
async fn test_get_owner_payment_stats() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, _org_id, building_id, owner_id, expense_id) =
        create_test_fixtures(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create payments with different statuses
    let amounts = vec![10000, 20000, 30000];
    let mut payment_ids = Vec::new();

    for amount in amounts {
        let req = test::TestRequest::post()
            .uri("/api/v1/payments")
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
            .set_json(json!({
                "building_id": building_id.to_string(),
                "owner_id": owner_id.to_string(),
                "expense_id": expense_id.to_string(),
                "amount_cents": amount,
                "payment_method_type": "card"
            }))
            .to_request();

        let resp = test::call_service(&app, req).await;
        let payment: serde_json::Value = test::read_body_json(resp).await;
        payment_ids.push(payment["id"].as_str().unwrap().to_string());
    }

    // Mark first as succeeded, second as failed
    let succeeded_req = test::TestRequest::put()
        .uri(&format!("/api/v1/payments/{}/succeeded", payment_ids[0]))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    test::call_service(&app, succeeded_req).await;

    // Get owner payment stats
    let stats_req = test::TestRequest::get()
        .uri(&format!("/api/v1/owners/{}/payments/stats", owner_id))
        .to_request();

    let stats_resp = test::call_service(&app, stats_req).await;
    assert_eq!(stats_resp.status(), 200);

    let stats: serde_json::Value = test::read_body_json(stats_resp).await;
    assert_eq!(stats["total_count"], 3);
    assert_eq!(stats["succeeded_count"], 1);
    assert_eq!(stats["pending_count"], 2);
}

// ==================== Payment Method Tests ====================

#[actix_web::test]
#[serial]
async fn test_create_payment_method() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, _org_id, _building_id, owner_id, _expense_id) =
        create_test_fixtures(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/v1/payment-methods")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "owner_id": owner_id.to_string(),
            "method_type": "card",
            "stripe_payment_method_id": "pm_test_visa_4242",
            "stripe_customer_id": "cus_test_12345",
            "display_label": "Visa ****4242",
            "is_default": true,
            "expires_at": (Utc::now() + Duration::days(365 * 3)).to_rfc3339()
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201, "Should create payment method");

    let method: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(method["method_type"], "card");
    assert_eq!(method["display_label"], "Visa ****4242");
    assert_eq!(method["is_default"], true);
    assert_eq!(method["is_active"], true);
    assert_eq!(method["is_usable"], true);
}

#[actix_web::test]
#[serial]
async fn test_create_sepa_payment_method() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, _org_id, _building_id, owner_id, _expense_id) =
        create_test_fixtures(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/v1/payment-methods")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "owner_id": owner_id.to_string(),
            "method_type": "sepa_debit",
            "stripe_payment_method_id": "pm_sepa_BE68539007547034",
            "stripe_customer_id": "cus_test_54321",
            "display_label": "SEPA Debit ****7034",
            "is_default": false
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    let method: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(method["method_type"], "sepa_debit");
    assert_eq!(method["is_expired"], false);
}

#[actix_web::test]
#[serial]
async fn test_list_owner_payment_methods() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, _org_id, _building_id, owner_id, _expense_id) =
        create_test_fixtures(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create 2 payment methods
    for i in 1..=2 {
        let req = test::TestRequest::post()
            .uri("/api/v1/payment-methods")
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
            .set_json(json!({
                "owner_id": owner_id.to_string(),
                "method_type": "card",
                "stripe_payment_method_id": format!("pm_test_{}", i),
                "stripe_customer_id": format!("cus_test_{}", i),
                "display_label": format!("Card {}", i),
                "is_default": i == 1
            }))
            .to_request();

        test::call_service(&app, req).await;
    }

    // List all payment methods for owner
    let list_req = test::TestRequest::get()
        .uri(&format!("/api/v1/owners/{}/payment-methods", owner_id))
        .to_request();

    let resp = test::call_service(&app, list_req).await;
    assert_eq!(resp.status(), 200);

    let methods: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(methods.as_array().unwrap().len(), 2);
}

#[actix_web::test]
#[serial]
async fn test_set_payment_method_as_default() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, _org_id, _building_id, owner_id, _expense_id) =
        create_test_fixtures(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create first method as default
    let req1 = test::TestRequest::post()
        .uri("/api/v1/payment-methods")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "owner_id": owner_id.to_string(),
            "method_type": "card",
            "stripe_payment_method_id": "pm_test_1",
            "stripe_customer_id": "cus_test_1",
            "display_label": "Card 1",
            "is_default": true
        }))
        .to_request();

    test::call_service(&app, req1).await;

    // Create second method
    let req2 = test::TestRequest::post()
        .uri("/api/v1/payment-methods")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "owner_id": owner_id.to_string(),
            "method_type": "card",
            "stripe_payment_method_id": "pm_test_2",
            "stripe_customer_id": "cus_test_2",
            "display_label": "Card 2",
            "is_default": false
        }))
        .to_request();

    let resp2 = test::call_service(&app, req2).await;
    let method2: serde_json::Value = test::read_body_json(resp2).await;
    let method2_id = method2["id"].as_str().unwrap();

    // Set second method as default
    let set_default_req = test::TestRequest::put()
        .uri(&format!(
            "/api/v1/payment-methods/{}/set-default",
            method2_id
        ))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "owner_id": owner_id.to_string()
        }))
        .to_request();

    let set_default_resp = test::call_service(&app, set_default_req).await;
    assert_eq!(set_default_resp.status(), 200);

    let updated: serde_json::Value = test::read_body_json(set_default_resp).await;
    assert_eq!(updated["is_default"], true);
}

#[actix_web::test]
#[serial]
async fn test_deactivate_payment_method() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, _org_id, _building_id, owner_id, _expense_id) =
        create_test_fixtures(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create payment method
    let create_req = test::TestRequest::post()
        .uri("/api/v1/payment-methods")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "owner_id": owner_id.to_string(),
            "method_type": "card",
            "stripe_payment_method_id": "pm_test_deactivate",
            "stripe_customer_id": "cus_test_deactivate",
            "display_label": "Card to Deactivate",
            "is_default": false
        }))
        .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    let method: serde_json::Value = test::read_body_json(create_resp).await;
    let method_id = method["id"].as_str().unwrap();
    assert_eq!(method["is_active"], true);

    // Deactivate method
    let deactivate_req = test::TestRequest::put()
        .uri(&format!("/api/v1/payment-methods/{}/deactivate", method_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let deactivate_resp = test::call_service(&app, deactivate_req).await;
    assert_eq!(deactivate_resp.status(), 200);

    let deactivated: serde_json::Value = test::read_body_json(deactivate_resp).await;
    assert_eq!(deactivated["is_active"], false);
    assert_eq!(deactivated["is_usable"], false);
}

#[actix_web::test]
#[serial]
async fn test_reactivate_payment_method() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, _org_id, _building_id, owner_id, _expense_id) =
        create_test_fixtures(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create and deactivate payment method
    let create_req = test::TestRequest::post()
        .uri("/api/v1/payment-methods")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "owner_id": owner_id.to_string(),
            "method_type": "sepa_debit",
            "stripe_payment_method_id": "pm_sepa_test",
            "stripe_customer_id": "cus_sepa_test",
            "display_label": "SEPA Test",
            "is_default": false
        }))
        .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    let method: serde_json::Value = test::read_body_json(create_resp).await;
    let method_id = method["id"].as_str().unwrap();

    // Deactivate
    let deactivate_req = test::TestRequest::put()
        .uri(&format!("/api/v1/payment-methods/{}/deactivate", method_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    test::call_service(&app, deactivate_req).await;

    // Reactivate
    let reactivate_req = test::TestRequest::put()
        .uri(&format!("/api/v1/payment-methods/{}/reactivate", method_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let reactivate_resp = test::call_service(&app, reactivate_req).await;
    assert_eq!(reactivate_resp.status(), 200);

    let reactivated: serde_json::Value = test::read_body_json(reactivate_resp).await;
    assert_eq!(reactivated["is_active"], true);
    assert_eq!(reactivated["is_usable"], true);
}

#[actix_web::test]
#[serial]
async fn test_delete_payment_method() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, _org_id, _building_id, owner_id, _expense_id) =
        create_test_fixtures(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create payment method
    let create_req = test::TestRequest::post()
        .uri("/api/v1/payment-methods")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "owner_id": owner_id.to_string(),
            "method_type": "card",
            "stripe_payment_method_id": "pm_test_delete",
            "stripe_customer_id": "cus_test_delete",
            "display_label": "Card to Delete",
            "is_default": false
        }))
        .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    let method: serde_json::Value = test::read_body_json(create_resp).await;
    let method_id = method["id"].as_str().unwrap();

    // Delete method
    let delete_req = test::TestRequest::delete()
        .uri(&format!("/api/v1/payment-methods/{}", method_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let delete_resp = test::call_service(&app, delete_req).await;
    assert_eq!(delete_resp.status(), 204);

    // Verify deletion
    let get_req = test::TestRequest::get()
        .uri(&format!("/api/v1/payment-methods/{}", method_id))
        .to_request();

    let get_resp = test::call_service(&app, get_req).await;
    assert_eq!(get_resp.status(), 404);
}

#[actix_web::test]
#[serial]
async fn test_complete_payment_lifecycle() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, _org_id, building_id, owner_id, expense_id) =
        create_test_fixtures(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // 1. Create payment method
    let method_req = test::TestRequest::post()
        .uri("/api/v1/payment-methods")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "owner_id": owner_id.to_string(),
            "method_type": "card",
            "stripe_payment_method_id": "pm_lifecycle_test",
            "stripe_customer_id": "cus_lifecycle_test",
            "display_label": "Lifecycle Test Card",
            "is_default": true
        }))
        .to_request();

    let method_resp = test::call_service(&app, method_req).await;
    let method: serde_json::Value = test::read_body_json(method_resp).await;
    let method_id = method["id"].as_str().unwrap();

    // 2. Create payment using saved payment method
    let payment_req = test::TestRequest::post()
        .uri("/api/v1/payments")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "building_id": building_id.to_string(),
            "owner_id": owner_id.to_string(),
            "expense_id": expense_id.to_string(),
            "amount_cents": 200000, // 2000.00 EUR
            "payment_method_type": "card",
            "payment_method_id": method_id,
            "description": "Complete lifecycle payment"
        }))
        .to_request();

    let payment_resp = test::call_service(&app, payment_req).await;
    let payment: serde_json::Value = test::read_body_json(payment_resp).await;
    let payment_id = payment["id"].as_str().unwrap();
    assert_eq!(payment["status"], "pending");
    assert_eq!(payment["amount_cents"], 200000);

    // 3. Process payment (Pending -> Processing)
    let processing_req = test::TestRequest::put()
        .uri(&format!("/api/v1/payments/{}/processing", payment_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    test::call_service(&app, processing_req).await;

    // 4. Mark as succeeded
    let succeeded_req = test::TestRequest::put()
        .uri(&format!("/api/v1/payments/{}/succeeded", payment_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let succeeded_resp = test::call_service(&app, succeeded_req).await;
    let succeeded: serde_json::Value = test::read_body_json(succeeded_resp).await;
    assert_eq!(succeeded["status"], "succeeded");
    assert!(succeeded["succeeded_at"].is_string());

    // 5. Partial refund
    let refund_req = test::TestRequest::post()
        .uri(&format!("/api/v1/payments/{}/refund", payment_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "amount_cents": 50000, // Refund 500.00 EUR
            "reason": "Customer request"
        }))
        .to_request();

    let refund_resp = test::call_service(&app, refund_req).await;
    let refunded: serde_json::Value = test::read_body_json(refund_resp).await;
    // Partial refund keeps status as "succeeded"
    assert_eq!(refunded["status"], "succeeded");
    assert_eq!(refunded["refunded_amount_cents"], 50000);

    // 6. Verify payment in lists
    let owner_payments_req = test::TestRequest::get()
        .uri(&format!("/api/v1/owners/{}/payments", owner_id))
        .to_request();

    let owner_payments_resp = test::call_service(&app, owner_payments_req).await;
    let owner_payments: serde_json::Value = test::read_body_json(owner_payments_resp).await;
    assert!(!owner_payments.as_array().unwrap().is_empty());

    // 7. Get payment stats
    let stats_req = test::TestRequest::get()
        .uri(&format!("/api/v1/owners/{}/payments/stats", owner_id))
        .to_request();

    let stats_resp = test::call_service(&app, stats_req).await;
    let stats: serde_json::Value = test::read_body_json(stats_resp).await;
    assert!(stats["total_count"].as_i64().unwrap() >= 1);
    assert_eq!(stats["total_refunded_cents"], 50000);
}
