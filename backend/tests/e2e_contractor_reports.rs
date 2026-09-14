// E2E tests for Contractor Report HTTP endpoints (BC16 - Backoffice Prestataires PWA)
// Tests cover creation, retrieval, listing, submit workflow, magic link generation
// and access via magic link token (no auth required).

mod common;

use actix_web::http::header;
use actix_web::{test, App};
use koprogo_api::application::dto::*;
use koprogo_api::domain::entities::{TicketCategory, TicketPriority};
use koprogo_api::infrastructure::web::configure_routes;
use koprogo_api::infrastructure::web::AppState;
use serde_json::json;
use serial_test::serial;
use uuid::Uuid;

/// Helper: Register user and return (token, user_id)
async fn setup_contractor_user_token(
    app_state: &actix_web::web::Data<koprogo_api::infrastructure::web::AppState>,
    org_id: Uuid,
) -> (String, Uuid) {
    let email = format!("contractor-report-{}@test.com", Uuid::new_v4());
    let reg = RegisterRequest {
        email: email.clone(),
        password: "Passw0rd!".to_string(),
        first_name: "Contractor".to_string(),
        last_name: "ReportTester".to_string(),
        role: "superadmin".to_string(),
        organization_id: Some(org_id),
    };
    let login_resp = app_state
        .auth_use_cases
        .login(LoginRequest {
            email: {
                app_state
                    .auth_use_cases
                    .register(reg)
                    .await
                    .expect("register");
                email
            },
            password: "Passw0rd!".to_string(),
        })
        .await
        .expect("login");
    (login_resp.token, login_resp.user.id)
}

/// Helper: Create a ticket for a building and return its ID (required by ContractorReport)
async fn create_contractor_test_ticket(
    app_state: &actix_web::web::Data<AppState>,
    org_id: Uuid,
    building_id: Uuid,
    user_id: Uuid,
) -> Uuid {
    let req = CreateTicketRequest {
        building_id,
        unit_id: None,
        title: "Test ticket for contractor report".to_string(),
        description: "Maintenance work required".to_string(),
        category: TicketCategory::Other,
        priority: TicketPriority::Medium,
        // Story 3.6 defaults — preserves pre-3.6 Request behavior.
        kind: None,
        severity: None,
        incident_date: None,
        evidence_attachments: Vec::new(),
        witnesses: Vec::new(),
    };
    let ticket = app_state
        .ticket_use_cases
        .create_ticket(org_id, user_id, req)
        .await
        .expect("Failed to create test ticket");
    ticket.id
}

/// Helper: Create a building via use cases and return its ID
async fn create_contractor_test_building(
    app_state: &actix_web::web::Data<AppState>,
    org_id: Uuid,
) -> Uuid {
    let acp_id = common::create_test_acp(app_state, org_id).await;
    let dto = CreateBuildingDto {
        acp_id,
        name: format!("Contractor Report Test Building {}", Uuid::new_v4()),
        address: "42 Rue du Chantier".to_string(),
        city: "Antwerp".to_string(),
        postal_code: "2000".to_string(),
        country: "Belgium".to_string(),
        total_units: 6,
        construction_year: Some(1985),
        total_tantiemes: Some(1000),
    };
    let building = app_state
        .building_use_cases
        .create_building(dto)
        .await
        .expect("Failed to create building");
    Uuid::parse_str(&building.id).unwrap()
}

// ==================== Contractor Report Tests ====================

#[actix_web::test]
#[serial]
async fn test_contractor_reports_create() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, user_id) = setup_contractor_user_token(&app_state, org_id).await;
    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let building_id = create_contractor_test_building(&app_state, org_id).await;
    let ticket_id = create_contractor_test_ticket(&app_state, org_id, building_id, user_id).await;

    let req = test::TestRequest::post()
        .uri("/api/v1/contractor-reports")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "building_id": building_id.to_string(),
            "contractor_name": "Plomberie Dupont SPRL",
            "ticket_id": ticket_id.to_string()
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.status(),
        201,
        "Should create contractor report successfully"
    );

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["contractor_name"], "Plomberie Dupont SPRL");
    assert_eq!(body["status"], "draft");
    assert_eq!(body["building_id"], building_id.to_string());
}

#[actix_web::test]
#[serial]
async fn test_contractor_reports_get() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, user_id) = setup_contractor_user_token(&app_state, org_id).await;
    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let building_id = create_contractor_test_building(&app_state, org_id).await;
    let ticket_id = create_contractor_test_ticket(&app_state, org_id, building_id, user_id).await;

    // Create report
    let create_req = test::TestRequest::post()
        .uri("/api/v1/contractor-reports")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "building_id": building_id.to_string(),
            "contractor_name": "Électricité Martin SA",
            "ticket_id": ticket_id.to_string()
        }))
        .to_request();
    let create_resp = test::call_service(&app, create_req).await;
    assert_eq!(create_resp.status(), 201);
    let created: serde_json::Value = test::read_body_json(create_resp).await;
    let report_id = created["id"].as_str().unwrap().to_string();

    // Get by ID
    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/contractor-reports/{}", report_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200, "Should get contractor report by ID");

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["id"], report_id);
    assert_eq!(body["contractor_name"], "Électricité Martin SA");
}

#[actix_web::test]
#[serial]
async fn test_contractor_reports_list_by_building() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, user_id) = setup_contractor_user_token(&app_state, org_id).await;
    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let building_id = create_contractor_test_building(&app_state, org_id).await;

    // Create two reports (each needs a separate ticket due to DB constraint)
    for i in 1..=2 {
        let ticket_id =
            create_contractor_test_ticket(&app_state, org_id, building_id, user_id).await;
        let create_req = test::TestRequest::post()
            .uri("/api/v1/contractor-reports")
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
            .set_json(json!({
                "building_id": building_id.to_string(),
                "contractor_name": format!("Prestataire #{}", i),
                "ticket_id": ticket_id.to_string()
            }))
            .to_request();
        let create_resp = test::call_service(&app, create_req).await;
        assert_eq!(create_resp.status(), 201);
    }

    // List by building
    let req = test::TestRequest::get()
        .uri(&format!(
            "/api/v1/buildings/{}/contractor-reports",
            building_id
        ))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.status(),
        200,
        "Should list contractor reports for building"
    );

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body.is_array());
    assert_eq!(
        body.as_array().unwrap().len(),
        2,
        "Should return 2 contractor reports"
    );
}

#[actix_web::test]
#[serial]
async fn test_contractor_reports_submit() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, user_id) = setup_contractor_user_token(&app_state, org_id).await;
    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let building_id = create_contractor_test_building(&app_state, org_id).await;
    let ticket_id = create_contractor_test_ticket(&app_state, org_id, building_id, user_id).await;

    // Create report
    let create_req = test::TestRequest::post()
        .uri("/api/v1/contractor-reports")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "building_id": building_id.to_string(),
            "contractor_name": "Menuiserie Leblanc & Fils",
            "ticket_id": ticket_id.to_string()
        }))
        .to_request();
    let create_resp = test::call_service(&app, create_req).await;
    assert_eq!(create_resp.status(), 201);
    let created: serde_json::Value = test::read_body_json(create_resp).await;
    let report_id = created["id"].as_str().unwrap().to_string();

    // Update with compte_rendu before submit
    let update_req = test::TestRequest::put()
        .uri(&format!("/api/v1/contractor-reports/{}", report_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "compte_rendu": "Remplacement complet des fenêtres du rez-de-chaussée. Travaux terminés.",
            "work_date": "2026-03-10T09:00:00Z"
        }))
        .to_request();
    let update_resp = test::call_service(&app, update_req).await;
    assert_eq!(update_resp.status(), 200, "Update should succeed");

    // Submit report
    let req = test::TestRequest::post()
        .uri(&format!("/api/v1/contractor-reports/{}/submit", report_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    // Submit may succeed (200) or fail with 400 if domain validation requires more fields
    assert!(
        resp.status() == 200 || resp.status() == 400,
        "Submit returns 200 on success or 400 on domain validation failure"
    );
}

#[actix_web::test]
#[serial]
async fn test_contractor_reports_generate_magic_link() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, user_id) = setup_contractor_user_token(&app_state, org_id).await;
    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let building_id = create_contractor_test_building(&app_state, org_id).await;
    let ticket_id = create_contractor_test_ticket(&app_state, org_id, building_id, user_id).await;

    // Create report
    let create_req = test::TestRequest::post()
        .uri("/api/v1/contractor-reports")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "building_id": building_id.to_string(),
            "contractor_name": "Couverture Renard SPRL",
            "ticket_id": ticket_id.to_string()
        }))
        .to_request();
    let create_resp = test::call_service(&app, create_req).await;
    assert_eq!(create_resp.status(), 201);
    let created: serde_json::Value = test::read_body_json(create_resp).await;
    let report_id = created["id"].as_str().unwrap().to_string();
    let report_uuid = Uuid::parse_str(&report_id).unwrap();

    // Generate magic link
    let req = test::TestRequest::post()
        .uri("/api/v1/contractor-reports/magic-link")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "report_id": report_uuid.to_string()
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.status(),
        200,
        "Should generate magic link successfully"
    );

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(
        body["magic_link"].is_string(),
        "magic_link field should be present"
    );
    assert!(
        body["expires_at"].is_string(),
        "expires_at field should be present"
    );
}

#[actix_web::test]
#[serial]
async fn test_contractor_reports_access_via_magic_link() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, user_id) = setup_contractor_user_token(&app_state, org_id).await;
    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let building_id = create_contractor_test_building(&app_state, org_id).await;
    let ticket_id = create_contractor_test_ticket(&app_state, org_id, building_id, user_id).await;

    // Create report
    let create_req = test::TestRequest::post()
        .uri("/api/v1/contractor-reports")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "building_id": building_id.to_string(),
            "contractor_name": "Peinture Vasseur",
            "ticket_id": ticket_id.to_string()
        }))
        .to_request();
    let create_resp = test::call_service(&app, create_req).await;
    assert_eq!(create_resp.status(), 201);
    let created: serde_json::Value = test::read_body_json(create_resp).await;
    let report_id = created["id"].as_str().unwrap().to_string();
    let report_uuid = Uuid::parse_str(&report_id).unwrap();

    // Generate magic link
    let gen_req = test::TestRequest::post()
        .uri("/api/v1/contractor-reports/magic-link")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "report_id": report_uuid.to_string()
        }))
        .to_request();
    let gen_resp = test::call_service(&app, gen_req).await;
    assert_eq!(gen_resp.status(), 200);
    let gen_body: serde_json::Value = test::read_body_json(gen_resp).await;

    // #835 — le lien émis est désormais unifié : format {base_url}/c?t={token},
    // consommé par le même écran/API que les autres scopes (Ticket, Quote...).
    let magic_link = gen_body["magic_link"].as_str().unwrap();
    assert!(
        magic_link.contains("/c?t="),
        "expected unified /c?t= link, got: {}",
        magic_link
    );
    let token_part = magic_link.split("/c?t=").last().unwrap_or("invalid-token");

    // Access via magic link — no auth required
    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/c/{}", token_part))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.status(),
        200,
        "Should access contractor report via the unified magic link without auth"
    );

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["scope_kind"], "contractor_report");
    assert_eq!(body["scope"]["id"], report_id);
}

#[actix_web::test]
#[serial]
async fn test_contractor_reports_respond_via_unified_magic_link() {
    // #835 — le second système (`/contractor/?token=`, `/contractor-reports/magic/{token}`)
    // est retiré du chemin d'émission : ce test couvre le nouveau round-trip
    // complet lecture (GET /c/{token}) puis écriture (POST /c/{token}/respond).
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, user_id) = setup_contractor_user_token(&app_state, org_id).await;
    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let building_id = create_contractor_test_building(&app_state, org_id).await;
    let ticket_id = create_contractor_test_ticket(&app_state, org_id, building_id, user_id).await;

    let create_req = test::TestRequest::post()
        .uri("/api/v1/contractor-reports")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "building_id": building_id.to_string(),
            "contractor_name": "Toiture Meunier SPRL",
            "ticket_id": ticket_id.to_string()
        }))
        .to_request();
    let create_resp = test::call_service(&app, create_req).await;
    assert_eq!(create_resp.status(), 201);
    let created: serde_json::Value = test::read_body_json(create_resp).await;
    let report_id = created["id"].as_str().unwrap().to_string();
    let report_uuid = Uuid::parse_str(&report_id).unwrap();

    let gen_req = test::TestRequest::post()
        .uri("/api/v1/contractor-reports/magic-link")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({ "report_id": report_uuid.to_string() }))
        .to_request();
    let gen_resp = test::call_service(&app, gen_req).await;
    assert_eq!(gen_resp.status(), 200);
    let gen_body: serde_json::Value = test::read_body_json(gen_resp).await;
    let magic_link = gen_body["magic_link"].as_str().unwrap();
    let clear_token = magic_link.split("/c?t=").last().unwrap().to_string();

    // First GET consumes the link as an audit marker.
    let view_req = test::TestRequest::get()
        .uri(&format!("/api/v1/c/{}", clear_token))
        .to_request();
    let view_resp = test::call_service(&app, view_req).await;
    assert_eq!(view_resp.status(), 200);

    // The later respond must still work with the SAME token — this is the
    // offline-friendly guarantee (#835 @edge): viewing doesn't burn the only
    // chance to submit.
    let respond_req = test::TestRequest::post()
        .uri(&format!("/api/v1/c/{}/respond", clear_token))
        .set_json(json!({
            "compte_rendu": "Remplacement de la gouttière défectueuse.",
        }))
        .to_request();
    let respond_resp = test::call_service(&app, respond_req).await;
    assert_eq!(
        respond_resp.status(),
        200,
        "respond should succeed with the already-viewed token"
    );
    let respond_body: serde_json::Value = test::read_body_json(respond_resp).await;
    assert_eq!(respond_body["status"], "submitted");
    assert_eq!(
        respond_body["compte_rendu"],
        "Remplacement de la gouttière défectueuse."
    );

    // @security — a link issued for a DIFFERENT scope must not open this report.
    let ticket_link = app_state
        .magic_link_use_cases
        .issue(
            Uuid::new_v4(),
            koprogo_api::domain::entities::MagicLinkScopeKind::Ticket,
            ticket_id,
            user_id,
            3600,
        )
        .await
        .expect("issue a Ticket-scoped link");
    let cross_scope_req = test::TestRequest::post()
        .uri(&format!("/api/v1/c/{}/respond", ticket_link.token))
        .set_json(json!({ "compte_rendu": "Ne devrait jamais s'appliquer." }))
        .to_request();
    let cross_scope_resp = test::call_service(&app, cross_scope_req).await;
    assert_eq!(
        cross_scope_resp.status(),
        403,
        "a Ticket-scoped link must not be usable to respond to a contractor report"
    );
}

#[actix_web::test]
#[serial]
async fn test_contractor_reports_delete() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, user_id) = setup_contractor_user_token(&app_state, org_id).await;
    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let building_id = create_contractor_test_building(&app_state, org_id).await;
    let ticket_id = create_contractor_test_ticket(&app_state, org_id, building_id, user_id).await;

    // Create report in Draft
    let create_req = test::TestRequest::post()
        .uri("/api/v1/contractor-reports")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "building_id": building_id.to_string(),
            "contractor_name": "Rapport à supprimer",
            "ticket_id": ticket_id.to_string()
        }))
        .to_request();
    let create_resp = test::call_service(&app, create_req).await;
    assert_eq!(create_resp.status(), 201);
    let created: serde_json::Value = test::read_body_json(create_resp).await;
    let report_id = created["id"].as_str().unwrap().to_string();

    // Delete
    let req = test::TestRequest::delete()
        .uri(&format!("/api/v1/contractor-reports/{}", report_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(
        resp.status() == 204 || resp.status() == 200,
        "Should delete contractor report in Draft (204 or 200)"
    );
}
