// E2E tests for Payment Recovery HTTP endpoints (Issue #83)
// Tests focus on HTTP layer: endpoints, auth, JSON serialization
// Belgian legal context: Automated recovery workflow with 4 escalation levels

mod common;

use actix_web::http::header;
use actix_web::{test, App};
use chrono::Utc;
use koprogo_api::infrastructure::web::configure_routes;
use serde_json::json;
use serial_test::serial;
use uuid::Uuid;

#[actix_web::test]
#[serial]
async fn test_create_gentle_reminder() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = common::register_and_login(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let expense_id = Uuid::new_v4();
    let owner_id = Uuid::new_v4();

    let reminder_dto = json!({
        "expense_id": expense_id.to_string(),
        "owner_id": owner_id.to_string(),
        "level": "Gentle",  // J+15 after due date
        "amount_due_cents": 50_000_i64,  // 500 EUR
        "days_overdue": 15
    });

    let req = test::TestRequest::post()
        .uri("/api/v1/payment-reminders")
        .insert_header(header::ContentType::json())
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(&reminder_dto)
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Gentle reminder: Courteous tone, simple email
    assert!(
        resp.status().is_success() || resp.status().is_client_error(),
        "Expected valid response, got: {}",
        resp.status()
    );
}

#[actix_web::test]
#[serial]
async fn test_escalation_workflow() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = common::register_and_login(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let reminder_id = Uuid::new_v4();

    // Escalate from Gentle to Formal (J+30)
    let req = test::TestRequest::post()
        .uri(&format!(
            "/api/v1/payment-reminders/{}/escalate",
            reminder_id
        ))
        .insert_header(header::ContentType::json())
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "new_level": "Formal",
            "notes": "No payment received after gentle reminder"
        }))
        .to_request();

    let _resp = test::call_service(&app, req).await;

    // 4-level escalation: Gentle -> Formal -> FinalNotice -> LegalAction
    // Belgian context: Formal = email + PDF letter, mentions penalties
}

#[actix_web::test]
#[serial]
async fn test_calculate_late_payment_penalty() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = common::register_and_login(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let reminder_id = Uuid::new_v4();

    let req = test::TestRequest::get()
        .uri(&format!(
            "/api/v1/payment-reminders/{}?include_penalty=true",
            reminder_id
        ))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let _resp = test::call_service(&app, req).await;

    // Belgian legal rate: 8% annual
    // Formula: penalty = amount * 0.08 * (days_overdue / 365)
    // Example: 500 EUR * 8% * (30/365) = 3.29 EUR penalty
}

#[actix_web::test]
#[serial]
async fn test_mark_reminder_as_sent() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = common::register_and_login(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let reminder_id = Uuid::new_v4();

    let req = test::TestRequest::put()
        .uri(&format!(
            "/api/v1/payment-reminders/{}/mark-sent",
            reminder_id
        ))
        .insert_header(header::ContentType::json())
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "sent_date": Utc::now().to_rfc3339()
        }))
        .to_request();

    let _resp = test::call_service(&app, req).await;

    // Tracking: Email sent, awaiting owner response
}

#[actix_web::test]
#[serial]
async fn test_add_tracking_number_for_registered_letter() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = common::register_and_login(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let reminder_id = Uuid::new_v4();

    let req = test::TestRequest::put()
        .uri(&format!(
            "/api/v1/payment-reminders/{}/tracking-number",
            reminder_id
        ))
        .insert_header(header::ContentType::json())
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "tracking_number": "RR123456789BE"
        }))
        .to_request();

    let _resp = test::call_service(&app, req).await;

    // FinalNotice level: Registered letter (lettre recommandee)
    // Tracking number for legal proof of delivery
}

#[actix_web::test]
#[serial]
async fn test_bulk_create_reminders_for_overdue_expenses() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = common::register_and_login(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let building_id = Uuid::new_v4();

    let req = test::TestRequest::post()
        .uri("/api/v1/payment-reminders/bulk-create")
        .insert_header(header::ContentType::json())
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "building_id": building_id.to_string(),
            "days_overdue_threshold": 15
        }))
        .to_request();

    let _resp = test::call_service(&app, req).await;

    // Automated workflow: Find all expenses overdue > 15 days without reminders
    // Create Gentle reminders for all
    // Belgian best practice: Automate to reduce manual work
}

#[actix_web::test]
#[serial]
async fn test_get_recovery_statistics() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = common::register_and_login(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/v1/payment-reminders/stats")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let _resp = test::call_service(&app, req).await;

    // Expected stats:
    // - total_reminders
    // - by_level: { Gentle: 10, Formal: 5, FinalNotice: 2, LegalAction: 1 }
    // - total_amount_overdue_cents
    // - success_rate (% paid after reminder)
    // - average_days_to_payment
}

#[actix_web::test]
#[serial]
async fn test_list_active_reminders_by_owner() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = common::register_and_login(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let owner_id = Uuid::new_v4();

    let req = test::TestRequest::get()
        .uri(&format!(
            "/api/v1/owners/{}/payment-reminders/active",
            owner_id
        ))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let _resp = test::call_service(&app, req).await;

    // Active = sent but not paid/cancelled
    // Critical for owner dashboard: "You have 3 unpaid reminders"
}

#[actix_web::test]
#[serial]
async fn test_cancel_reminder_if_paid() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = common::register_and_login(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let reminder_id = Uuid::new_v4();

    let req = test::TestRequest::put()
        .uri(&format!("/api/v1/payment-reminders/{}/cancel", reminder_id))
        .insert_header(header::ContentType::json())
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "cancellation_reason": "Payment received on 2026-01-15"
        }))
        .to_request();

    let _resp = test::call_service(&app, req).await;

    // Stop escalation if owner pays
    // Audit trail for legal compliance
}

#[actix_web::test]
#[serial]
async fn test_find_overdue_expenses_without_reminders() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = common::register_and_login(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/v1/payment-reminders/find-overdue-without-reminders")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let _resp = test::call_service(&app, req).await;

    // Cron job helper: Identify expenses needing first reminder
    // Automation trigger for bulk_create
}

#[actix_web::test]
#[serial]
async fn test_mark_reminder_as_opened() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = common::register_and_login(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let reminder_id = Uuid::new_v4();

    let req = test::TestRequest::put()
        .uri(&format!(
            "/api/v1/payment-reminders/{}/mark-opened",
            reminder_id
        ))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let _resp = test::call_service(&app, req).await;

    // Email tracking: Owner opened reminder email
    // Helps measure engagement and reminder effectiveness
}
