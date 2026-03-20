// E2E tests for Charge Distribution HTTP endpoints
// Tests cover allocation of invoice charges across unit owners based on ownership percentages
// Belgian copropriété law: charges distributed proportionally to tantièmes

mod common;

use actix_web::http::header;
use actix_web::{test, App};
use chrono::Utc;
use koprogo_api::application::dto::{
    ApproveInvoiceDto, CreateBuildingDto, CreateExpenseDto, CreateOwnerDto, CreateUnitDto,
    SubmitForApprovalDto,
};
use koprogo_api::domain::entities::{ExpenseCategory, UnitType};
use koprogo_api::infrastructure::web::configure_routes;
use koprogo_api::infrastructure::web::AppState;
use serial_test::serial;
use uuid::Uuid;

// ==================== Setup Helpers ====================

/// Setup building, unit, owner, unit-owner relationship, and expense for distribution tests.
/// Returns (token, expense_id, owner_id, unit_id, building_id).
async fn setup_charge_distribution_fixtures(
    app_state: &actix_web::web::Data<AppState>,
    org_id: Uuid,
) -> (String, Uuid, Uuid, Uuid, Uuid) {
    // Register + login, keeping user_id for approve_invoice FK constraint
    let email = format!("charge-dist-user-{}@test.com", Uuid::new_v4());
    let _ = app_state
        .auth_use_cases
        .register(koprogo_api::application::dto::RegisterRequest {
            email: email.clone(),
            password: "Passw0rd!".to_string(),
            first_name: "E2E".to_string(),
            last_name: "Tester".to_string(),
            role: "superadmin".to_string(),
            organization_id: Some(org_id),
        })
        .await
        .expect("register");
    let login_response = app_state
        .auth_use_cases
        .login(koprogo_api::application::dto::LoginRequest {
            email,
            password: "Passw0rd!".to_string(),
        })
        .await
        .expect("login");
    let token = login_response.token;
    let user_id = login_response.user.id;

    // Create building
    let building_dto = CreateBuildingDto {
        organization_id: org_id.to_string(),
        name: format!("Charge Distribution Building {}", Uuid::new_v4()),
        address: "10 Distribution Ave".to_string(),
        city: "Liège".to_string(),
        postal_code: "4000".to_string(),
        country: "Belgium".to_string(),
        total_units: 2,
        total_tantiemes: Some(1000),
        construction_year: Some(2010),
    };
    let building = app_state
        .building_use_cases
        .create_building(building_dto)
        .await
        .expect("Failed to create building");
    let building_id = Uuid::parse_str(&building.id).expect("Invalid building ID");

    // Create unit
    let unit_dto = CreateUnitDto {
        organization_id: org_id.to_string(),
        building_id: building_id.to_string(),
        unit_number: "A1".to_string(),
        unit_type: UnitType::Apartment,
        floor: Some(1),
        surface_area: 80.0,
        quota: 1.0,
    };
    let unit = app_state
        .unit_use_cases
        .create_unit(unit_dto)
        .await
        .expect("Failed to create unit");
    let unit_id = Uuid::parse_str(&unit.id).expect("Invalid unit ID");

    // Create owner
    let owner_email = format!("charge-dist-{}@test.com", Uuid::new_v4());
    let owner_dto = CreateOwnerDto {
        organization_id: org_id.to_string(),
        first_name: "Marie".to_string(),
        last_name: "Dupont".to_string(),
        email: owner_email,
        phone: None,
        address: "10 Distribution Ave A1".to_string(),
        city: "Liège".to_string(),
        postal_code: "4000".to_string(),
        country: "Belgium".to_string(),
    };
    let owner = app_state
        .owner_use_cases
        .create_owner(owner_dto)
        .await
        .expect("Failed to create owner");
    let owner_id = Uuid::parse_str(&owner.id).expect("Invalid owner ID");

    // Assign owner to unit with 100% ownership
    app_state
        .unit_owner_use_cases
        .add_owner_to_unit(unit_id, owner_id, 1.0, true)
        .await
        .expect("Failed to assign owner to unit");

    // Create an expense for the building
    let expense_dto = CreateExpenseDto {
        organization_id: org_id.to_string(),
        building_id: building_id.to_string(),
        category: ExpenseCategory::Maintenance,
        description: "Réparation ascenseur".to_string(),
        amount: 500.0,
        expense_date: Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
        supplier: Some("Elevator SPRL".to_string()),
        invoice_number: Some("INV-2025-001".to_string()),
        account_code: None,
    };
    let expense = app_state
        .expense_use_cases
        .create_expense(expense_dto)
        .await
        .expect("Failed to create expense");
    let expense_id = Uuid::parse_str(&expense.id).expect("Invalid expense ID");

    // Approve the expense so charge distribution can be calculated
    // (calculate_and_save_distribution requires ApprovalStatus::Approved)
    app_state
        .expense_use_cases
        .submit_for_approval(expense_id, SubmitForApprovalDto {})
        .await
        .expect("Failed to submit expense for approval");

    app_state
        .expense_use_cases
        .approve_invoice(
            expense_id,
            ApproveInvoiceDto {
                approved_by_user_id: user_id.to_string(),
            },
        )
        .await
        .expect("Failed to approve expense");

    (token, expense_id, owner_id, unit_id, building_id)
}

// ==================== Charge Distribution Tests ====================

#[actix_web::test]
#[serial]
async fn test_charge_distribution_calculate() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, expense_id, _owner_id, _unit_id, _building_id) =
        setup_charge_distribution_fixtures(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Calculate charge distribution for the expense
    let req = test::TestRequest::post()
        .uri(&format!(
            "/api/v1/invoices/{}/calculate-distribution",
            expense_id
        ))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    let status = resp.status().as_u16();
    assert!(
        status == 200 || status == 201,
        "Should calculate charge distribution (200 or 201), got {}",
        status
    );

    let body: serde_json::Value = test::read_body_json(resp).await;
    // Response should have message + count + distributions
    assert!(
        body.get("count").is_some() || body.get("distributions").is_some() || body.is_array(),
        "Response should contain distribution data, got: {}",
        body
    );
}

#[actix_web::test]
#[serial]
async fn test_charge_distribution_get() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, expense_id, _owner_id, _unit_id, _building_id) =
        setup_charge_distribution_fixtures(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Calculate distribution first
    let calc_req = test::TestRequest::post()
        .uri(&format!(
            "/api/v1/invoices/{}/calculate-distribution",
            expense_id
        ))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();
    let _ = test::call_service(&app, calc_req).await;

    // Get distribution for the invoice
    let get_req = test::TestRequest::get()
        .uri(&format!("/api/v1/invoices/{}/distribution", expense_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let get_resp = test::call_service(&app, get_req).await;
    assert_eq!(
        get_resp.status(),
        200,
        "Should get charge distribution for invoice"
    );

    let distributions: serde_json::Value = test::read_body_json(get_resp).await;
    // Distributions should be an array (possibly empty if not calculated yet)
    assert!(
        distributions.is_array(),
        "Distributions should be an array, got: {}",
        distributions
    );
}

#[actix_web::test]
#[serial]
async fn test_charge_distribution_unauthorized() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (_token, expense_id, _owner_id, _unit_id, _building_id) =
        setup_charge_distribution_fixtures(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Request without token
    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/invoices/{}/distribution", expense_id))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.status(),
        401,
        "Should return 401 when no auth token provided"
    );
}

#[actix_web::test]
#[serial]
async fn test_charge_distribution_calculate_nonexistent_expense() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = common::register_and_login(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let nonexistent_id = Uuid::new_v4();

    let req = test::TestRequest::post()
        .uri(&format!(
            "/api/v1/invoices/{}/calculate-distribution",
            nonexistent_id
        ))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    let status = resp.status().as_u16();
    assert!(
        status == 400 || status == 404 || status == 403,
        "Non-existent expense should return 400/404/403, got {}",
        status
    );
}
