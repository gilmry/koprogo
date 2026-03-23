// E2E tests for Security Incident Management HTTP endpoints (Issue #317 - GDPR Art. 33)
// Tests focus on HTTP layer: endpoints, auth, JSON serialization
// Covers security incident lifecycle and 72h APD notification compliance

mod common;

use actix_web::http::header;
use actix_web::{test, App};
use koprogo_api::infrastructure::web::configure_routes;
use koprogo_api::infrastructure::web::AppState;
use serde_json::json;
use serial_test::serial;
use uuid::Uuid;

// ==================== Test Setup ====================

async fn create_superadmin_token(
    app_state: &actix_web::web::Data<AppState>,
    org_id: Uuid,
) -> String {
    common::register_and_login(app_state, org_id).await
}

async fn create_non_superadmin_token(
    app_state: &actix_web::web::Data<AppState>,
    org_id: Uuid,
) -> String {
    let email = format!("syndic+{}@test.com", Uuid::new_v4());
    let reg = koprogo_api::application::dto::RegisterRequest {
        email: email.clone(),
        password: "Passw0rd!".to_string(),
        first_name: "Syndic".to_string(),
        last_name: "User".to_string(),
        role: "syndic".to_string(),
        organization_id: Some(org_id),
    };
    let _ = app_state
        .auth_use_cases
        .register(reg)
        .await
        .expect("register syndic");
    let login_req = koprogo_api::application::dto::LoginRequest {
        email,
        password: "Passw0rd!".to_string(),
    };
    app_state
        .auth_use_cases
        .login(login_req)
        .await
        .expect("login syndic")
        .token
}

// ==================== Create Security Incident Tests ====================

#[actix_web::test]
#[serial]
async fn test_create_security_incident_success() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = create_superadmin_token(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/v1/admin/security-incidents")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "severity": "high",
            "incident_type": "data_breach",
            "title": "Unauthorized access to owner database",
            "description": "Suspicious query patterns detected on the owners table at 03:00 UTC",
            "data_categories_affected": ["personal_data", "contact_info"],
            "affected_subjects_count": 150
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201, "Should create security incident");

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["severity"], "high");
    assert_eq!(body["incident_type"], "data_breach");
    assert_eq!(body["title"], "Unauthorized access to owner database");
    assert_eq!(body["status"], "detected");
    assert!(body["id"].as_str().is_some());
    assert!(body["discovery_at"].as_str().is_some());
    assert!(body["hours_since_discovery"].is_number());
    assert_eq!(body["affected_subjects_count"], 150);
    assert!(body["notification_at"].is_null());
    assert!(body["apd_reference_number"].is_null());
}

#[actix_web::test]
#[serial]
async fn test_create_security_incident_invalid_severity() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = create_superadmin_token(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/v1/admin/security-incidents")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "severity": "extreme",
            "incident_type": "data_breach",
            "title": "Test incident",
            "description": "Test description",
            "data_categories_affected": ["personal_data"]
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400, "Should reject invalid severity");
}

#[actix_web::test]
#[serial]
async fn test_create_security_incident_empty_title() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = create_superadmin_token(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/v1/admin/security-incidents")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "severity": "medium",
            "incident_type": "unauthorized_access",
            "title": "",
            "description": "",
            "data_categories_affected": []
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.status(),
        400,
        "Should reject empty title and description"
    );
}

// ==================== List Security Incidents Tests ====================

#[actix_web::test]
#[serial]
async fn test_list_security_incidents() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = create_superadmin_token(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create 2 incidents
    for i in 1..=2 {
        let req = test::TestRequest::post()
            .uri("/api/v1/admin/security-incidents")
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
            .set_json(json!({
                "severity": "medium",
                "incident_type": "unauthorized_access",
                "title": format!("Incident #{}", i),
                "description": format!("Description for incident {}", i),
                "data_categories_affected": ["personal_data"]
            }))
            .to_request();

        test::call_service(&app, req).await;
    }

    // List all incidents
    let req = test::TestRequest::get()
        .uri("/api/v1/admin/security-incidents")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body["incidents"].as_array().unwrap().len() >= 2);
    assert!(body["total"].as_i64().unwrap() >= 2);
}

#[actix_web::test]
#[serial]
async fn test_list_security_incidents_with_severity_filter() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = create_superadmin_token(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create incidents with different severities
    let severities = ["critical", "high", "low"];
    for severity in severities {
        let req = test::TestRequest::post()
            .uri("/api/v1/admin/security-incidents")
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
            .set_json(json!({
                "severity": severity,
                "incident_type": "data_breach",
                "title": format!("{} severity incident", severity),
                "description": "Test description",
                "data_categories_affected": ["personal_data"]
            }))
            .to_request();

        test::call_service(&app, req).await;
    }

    // Filter by critical severity
    let req = test::TestRequest::get()
        .uri("/api/v1/admin/security-incidents?severity=critical")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = test::read_body_json(resp).await;
    let incidents = body["incidents"].as_array().unwrap();
    for incident in incidents {
        assert_eq!(incident["severity"], "critical");
    }
}

// ==================== Get Security Incident Tests ====================

#[actix_web::test]
#[serial]
async fn test_get_security_incident_by_id() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = create_superadmin_token(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create incident
    let create_req = test::TestRequest::post()
        .uri("/api/v1/admin/security-incidents")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "severity": "critical",
            "incident_type": "malware",
            "title": "Ransomware detected on backup server",
            "description": "CryptoLocker variant detected on backup server at 15:30 UTC",
            "data_categories_affected": ["financial_data", "personal_data"],
            "affected_subjects_count": 500
        }))
        .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    let created: serde_json::Value = test::read_body_json(create_resp).await;
    let incident_id = created["id"].as_str().unwrap();

    // Get incident by ID
    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/admin/security-incidents/{}", incident_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["id"], incident_id);
    assert_eq!(body["severity"], "critical");
    assert_eq!(body["title"], "Ransomware detected on backup server");
    assert_eq!(body["affected_subjects_count"], 500);
}

#[actix_web::test]
#[serial]
async fn test_get_security_incident_not_found() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = create_superadmin_token(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let fake_id = Uuid::new_v4();
    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/admin/security-incidents/{}", fake_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 404);
}

// ==================== Report to APD Tests ====================

#[actix_web::test]
#[serial]
async fn test_report_incident_to_apd() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = create_superadmin_token(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create incident first
    let create_req = test::TestRequest::post()
        .uri("/api/v1/admin/security-incidents")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "severity": "high",
            "incident_type": "data_breach",
            "title": "Data breach requiring APD notification",
            "description": "Personal data of 200+ subjects potentially exposed",
            "data_categories_affected": ["personal_data", "financial_data"],
            "affected_subjects_count": 250
        }))
        .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    let created: serde_json::Value = test::read_body_json(create_resp).await;
    let incident_id = created["id"].as_str().unwrap();

    // Report to APD
    let report_req = test::TestRequest::put()
        .uri(&format!(
            "/api/v1/admin/security-incidents/{}/report-apd",
            incident_id
        ))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "apd_reference_number": "APD-2026-001234",
            "investigation_notes": "Reported within 72 hours. Root cause identified as SQL injection via legacy endpoint."
        }))
        .to_request();

    let report_resp = test::call_service(&app, report_req).await;
    assert_eq!(report_resp.status(), 200, "Should report incident to APD");

    let body: serde_json::Value = test::read_body_json(report_resp).await;
    assert_eq!(body["status"], "reported");
    assert_eq!(body["apd_reference_number"], "APD-2026-001234");
    assert!(body["notification_at"].as_str().is_some());
}

#[actix_web::test]
#[serial]
async fn test_report_incident_to_apd_missing_reference() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = create_superadmin_token(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create incident first
    let create_req = test::TestRequest::post()
        .uri("/api/v1/admin/security-incidents")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "severity": "medium",
            "incident_type": "unauthorized_access",
            "title": "Missing reference test",
            "description": "Test",
            "data_categories_affected": ["personal_data"]
        }))
        .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    let created: serde_json::Value = test::read_body_json(create_resp).await;
    let incident_id = created["id"].as_str().unwrap();

    // Try to report without APD reference number
    let report_req = test::TestRequest::put()
        .uri(&format!(
            "/api/v1/admin/security-incidents/{}/report-apd",
            incident_id
        ))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "investigation_notes": "Some notes"
        }))
        .to_request();

    let report_resp = test::call_service(&app, report_req).await;
    assert_eq!(
        report_resp.status(),
        400,
        "Should require APD reference number"
    );
}

// ==================== Overdue Incidents Tests ====================

#[actix_web::test]
#[serial]
async fn test_list_overdue_incidents() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = create_superadmin_token(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // List overdue incidents (>72h old, not yet reported to APD)
    let req = test::TestRequest::get()
        .uri("/api/v1/admin/security-incidents/overdue")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body["incidents"].is_array());
    assert!(body["total"].is_number());

    // Any incident in the overdue list should have no notification_at
    if let Some(incidents) = body["incidents"].as_array() {
        for incident in incidents {
            assert!(
                incident["notification_at"].is_null(),
                "Overdue incidents should not have been reported"
            );
        }
    }
}

// ==================== Authorization Tests ====================

#[actix_web::test]
#[serial]
async fn test_create_incident_non_superadmin_forbidden() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = create_non_superadmin_token(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/v1/admin/security-incidents")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "severity": "low",
            "incident_type": "unauthorized_access",
            "title": "Unauthorized attempt to create incident",
            "description": "Should be blocked",
            "data_categories_affected": ["personal_data"]
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.status(),
        403,
        "Non-SuperAdmin should not be able to create incidents"
    );
}

#[actix_web::test]
#[serial]
async fn test_list_incidents_without_auth() {
    let (app_state, _container, _org_id) = common::setup_test_db().await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/v1/admin/security-incidents")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401, "Should require authentication");
}

// ==================== Complete Incident Lifecycle Test ====================

#[actix_web::test]
#[serial]
async fn test_complete_incident_lifecycle() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = create_superadmin_token(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // 1. Create incident
    let create_req = test::TestRequest::post()
        .uri("/api/v1/admin/security-incidents")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "severity": "critical",
            "incident_type": "data_breach",
            "title": "Full lifecycle test - critical breach",
            "description": "Complete workflow: detect, investigate, report to APD",
            "data_categories_affected": ["personal_data", "financial_data", "health_data"],
            "affected_subjects_count": 1000
        }))
        .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    assert_eq!(create_resp.status(), 201);

    let created: serde_json::Value = test::read_body_json(create_resp).await;
    let incident_id = created["id"].as_str().unwrap();
    assert_eq!(created["status"], "detected");

    // 2. Retrieve the incident
    let get_req = test::TestRequest::get()
        .uri(&format!("/api/v1/admin/security-incidents/{}", incident_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let get_resp = test::call_service(&app, get_req).await;
    assert_eq!(get_resp.status(), 200);

    let fetched: serde_json::Value = test::read_body_json(get_resp).await;
    assert_eq!(fetched["id"], incident_id);
    assert_eq!(fetched["severity"], "critical");

    // 3. Verify it appears in the list
    let list_req = test::TestRequest::get()
        .uri("/api/v1/admin/security-incidents")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let list_resp = test::call_service(&app, list_req).await;
    assert_eq!(list_resp.status(), 200);

    let list: serde_json::Value = test::read_body_json(list_resp).await;
    assert!(list["total"].as_i64().unwrap() >= 1);

    // 4. Report to APD (Belgian Data Protection Authority)
    let report_req = test::TestRequest::put()
        .uri(&format!(
            "/api/v1/admin/security-incidents/{}/report-apd",
            incident_id
        ))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "apd_reference_number": "APD-2026-LIFECYCLE-001",
            "investigation_notes": "Root cause: SQL injection in legacy API. Remediation: patched endpoint, reset affected credentials."
        }))
        .to_request();

    let report_resp = test::call_service(&app, report_req).await;
    assert_eq!(report_resp.status(), 200);

    let reported: serde_json::Value = test::read_body_json(report_resp).await;
    assert_eq!(reported["status"], "reported");
    assert_eq!(reported["apd_reference_number"], "APD-2026-LIFECYCLE-001");
    assert!(reported["notification_at"].as_str().is_some());

    // 5. Verify double-reporting is blocked (409 Conflict)
    let report_again_req = test::TestRequest::put()
        .uri(&format!(
            "/api/v1/admin/security-incidents/{}/report-apd",
            incident_id
        ))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "apd_reference_number": "APD-2026-DUPLICATE",
            "investigation_notes": "Trying to report again"
        }))
        .to_request();

    let report_again_resp = test::call_service(&app, report_again_req).await;
    assert_eq!(
        report_again_resp.status(),
        409,
        "Should prevent double-reporting to APD"
    );
}
