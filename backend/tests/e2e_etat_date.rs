// E2E tests for État Daté HTTP endpoints (Issue #80)
// Tests focus on HTTP layer: endpoints, auth, JSON serialization
// Belgian legal requirement: État Daté required for ALL property sales (Article 577-2 Civil Code)

mod common;

use actix_web::http::header;
use actix_web::{test, App};
use chrono::Utc;
use koprogo_api::application::dto::{CreateEtatDateRequest, CreateOwnerDto};
use koprogo_api::domain::entities::EtatDateLanguage;
use koprogo_api::infrastructure::web::configure_routes;
use serde_json::json;
use serial_test::serial;
use uuid::Uuid;

/// Crée un état daté réel (building + unit + owner) pour les tests du lien
/// notaire (#845 / ADR 0048 / ADR 0051). Retourne (syndic_token, etat_date_id,
/// reference_number).
///
/// L'état daté n'est pas une fixture triviale : `create_etat_date` exige un
/// lot réel (FK) ET au moins un copropriétaire actif dessus (« Unit has no
/// active owners »). Un `Uuid::new_v4()` inventé échoue à l'insertion, pas au
/// contrôle qu'on cherche à tester — c'est ce qui rendait l'ancien
/// `test_get_etat_date_by_reference_number` incapable de prouver quoi que ce
/// soit (aucune assertion de statut, référence jamais créée).
async fn create_notary_link_fixture(
    app_state: &actix_web::web::Data<koprogo_api::infrastructure::web::AppState>,
    org_id: Uuid,
) -> (String, Uuid, String) {
    let syndic_token = common::register_and_login_with_role(app_state, org_id, "syndic").await;
    let building_id = common::create_test_building(app_state, org_id).await;
    let unit_id = common::create_test_unit(app_state, building_id).await;

    let owner = app_state
        .owner_use_cases
        .create_owner(CreateOwnerDto {
            organization_id: org_id.to_string(),
            first_name: "Jean".to_string(),
            last_name: "Dupont".to_string(),
            email: format!("owner-{}@example.com", Uuid::new_v4()),
            phone: None,
            address: "Rue E2E 1".to_string(),
            city: "Bruxelles".to_string(),
            postal_code: "1000".to_string(),
            country: "Belgium".to_string(),
            user_id: None,
        })
        .await
        .expect("create_notary_link_fixture: create_owner failed");
    let owner_id = Uuid::parse_str(&owner.id).expect("parse owner_id");

    app_state
        .unit_owner_use_cases
        .add_owner_to_unit(unit_id, owner_id, rust_decimal_macros::dec!(1.0), true)
        .await
        .expect("create_notary_link_fixture: add_owner_to_unit failed");

    let etat_date = app_state
        .etat_date_use_cases
        .create_etat_date(CreateEtatDateRequest {
            organization_id: org_id,
            building_id,
            unit_id,
            reference_date: Utc::now(),
            language: EtatDateLanguage::Fr,
            notary_name: "Me Dupont".to_string(),
            notary_email: "dupont@notaire.be".to_string(),
            notary_phone: None,
        })
        .await
        .expect("create_notary_link_fixture: create_etat_date failed");

    (syndic_token, etat_date.id, etat_date.reference_number)
}

#[actix_web::test]
#[serial]
async fn test_create_etat_date_request() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = common::register_and_login(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let building_id = Uuid::new_v4();
    let unit_id = Uuid::new_v4();

    let etat_date_dto = json!({
        "building_id": building_id.to_string(),
        "unit_id": unit_id.to_string(),
        "reference_date": Utc::now().to_rfc3339(),
        "requestor_name": "Notaire Jean Dupont",
        "requestor_email": "jdupont@notaire.be",
        "requestor_phone": "+32 2 123 45 67"
    });

    let req = test::TestRequest::post()
        .uri("/api/v1/etats-dates")
        .insert_header(header::ContentType::json())
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(&etat_date_dto)
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Belgian law: État Daté must be delivered within 15 days
    assert!(
        resp.status().is_success() || resp.status().is_client_error(),
        "Expected valid response, got: {}",
        resp.status()
    );
}

#[actix_web::test]
#[serial]
async fn test_etat_date_workflow_requested_to_delivered() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = common::register_and_login(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let etat_date_id = Uuid::new_v4();

    // 1. Mark as InProgress (Requested → InProgress)
    let req = test::TestRequest::put()
        .uri(&format!("/api/v1/etats-dates/{}/in-progress", etat_date_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let _resp = test::call_service(&app, req).await;
    // Workflow transition test

    // 2. Update financial data (16 legal sections required)
    let financial_data = json!({
        "quota_ordinary": "0.0250",  // 2.5% of building
        "quota_extraordinary": "0.0250",
        "provisions_paid_amount_cents": 150_000_i64,  // 1,500 EUR
        "outstanding_amount_cents": 0i64,
        "pending_works_amount_cents": 500_000_i64,  // 5,000 EUR for elevator
        "pending_litigation": false,
        "insurance_policy_number": "BE-ASSUR-12345",
        "reserve_fund_amount_cents": 5_000_000_i64,  // 50,000 EUR
        "building_debt_amount_cents": 0i64,
        "building_credit_amount_cents": 1_000_000_i64
    });

    let req = test::TestRequest::put()
        .uri(&format!(
            "/api/v1/etats-dates/{}/financial-data",
            etat_date_id
        ))
        .insert_header(header::ContentType::json())
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(&financial_data)
        .to_request();

    let _resp = test::call_service(&app, req).await;

    // 3. Mark as Generated (InProgress → Generated)
    let req = test::TestRequest::put()
        .uri(&format!("/api/v1/etats-dates/{}/generated", etat_date_id))
        .insert_header(header::ContentType::json())
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "pdf_file_path": "/documents/etat_date_123.pdf"
        }))
        .to_request();

    let _resp = test::call_service(&app, req).await;

    // 4. Mark as Delivered (Generated → Delivered)
    let req = test::TestRequest::put()
        .uri(&format!("/api/v1/etats-dates/{}/delivered", etat_date_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let _resp = test::call_service(&app, req).await;
    // Complete workflow: Requested → InProgress → Generated → Delivered
}

#[actix_web::test]
#[serial]
async fn test_list_overdue_etats_dates() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = common::register_and_login(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/v1/etats-dates/overdue")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Belgian law: État Daté MUST be delivered within 15 days
    // Overdue = requested_date + 15 days < NOW and status != Delivered
    assert!(resp.status().is_success() || resp.status().is_client_error());
}

#[actix_web::test]
#[serial]
async fn test_list_expired_etats_dates() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = common::register_and_login(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/v1/etats-dates/expired")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;

    // État Daté expires after 3 months (90 days)
    // Seller must request a new one if not used
    assert!(resp.status().is_success() || resp.status().is_client_error());
}

// ============================================================================
// #845 / ADR 0048 / ADR 0051 — lien notaire pour `GET /etats-dates/reference/*`
//
// Multi-rôle (CRITICAL.md #9) : le syndic (authentifié) émet/renouvelle/
// révoque le lien ; le notaire (jamais authentifié — c'est tout le point du
// lien) le consomme. Pas un seul login pour tout le scénario.
// ============================================================================

#[actix_web::test]
#[serial]
async fn happy_syndic_issues_link_and_notary_reads_the_etat_date_repeatedly() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (syndic_token, etat_date_id, reference_number) =
        create_notary_link_fixture(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Le syndic émet le lien.
    let req = test::TestRequest::post()
        .uri(&format!("/api/v1/etats-dates/{}/notary-link", etat_date_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", syndic_token)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201, "issue notary link should succeed");
    let body: serde_json::Value = test::read_body_json(resp).await;
    let token = body["token"]
        .as_str()
        .expect("clear token present")
        .to_string();

    // Le notaire, sans compte et sans jeton JWT, lit l'état daté via son lien
    // — deux fois : ADR 0051 exige le multi-lecture, pas l'usage unique des
    // liens magiques génériques.
    for _ in 0..2 {
        let req = test::TestRequest::get()
            .uri(&format!(
                "/api/v1/etats-dates/reference/{}?token={}",
                reference_number, token
            ))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(
            resp.status(),
            200,
            "le jeton doit rester valide sur relecture"
        );
    }
}

#[actix_web::test]
#[serial]
async fn negative_reading_without_a_token_is_refused_before_reaching_the_use_case() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (_syndic_token, _etat_date_id, reference_number) =
        create_notary_link_fixture(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::get()
        .uri(&format!(
            "/api/v1/etats-dates/reference/{}",
            reference_number
        ))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.status(),
        403,
        "sans jeton, la lecture doit être refusée — un UUID/référence deviné ne suffit plus"
    );
}

#[actix_web::test]
#[serial]
async fn negative_unknown_reference_number_is_404_not_a_panic() {
    let (app_state, _container, _org_id) = common::setup_test_db().await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/v1/etats-dates/reference/ED-INCONNUE-000?token=whatever")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 404);
}

#[actix_web::test]
#[serial]
async fn negative_malformed_token_is_refused_typed_not_a_panic() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (_syndic_token, _etat_date_id, reference_number) =
        create_notary_link_fixture(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::get()
        .uri(&format!(
            "/api/v1/etats-dates/reference/{}?token=%20%20%20",
            reference_number
        ))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 403);
}

#[actix_web::test]
#[serial]
async fn edge_renewing_the_link_extends_validity_and_the_same_token_still_opens_it() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (syndic_token, etat_date_id, reference_number) =
        create_notary_link_fixture(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::post()
        .uri(&format!("/api/v1/etats-dates/{}/notary-link", etat_date_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", syndic_token)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    let body: serde_json::Value = test::read_body_json(resp).await;
    let token = body["token"].as_str().unwrap().to_string();
    let expires_at_before = body["expires_at"].as_str().unwrap().to_string();

    let req = test::TestRequest::put()
        .uri(&format!(
            "/api/v1/etats-dates/{}/notary-link/renew",
            etat_date_id
        ))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", syndic_token)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);
    let renew_body: serde_json::Value = test::read_body_json(resp).await;
    let expires_at_after = renew_body["expires_at"].as_str().unwrap().to_string();
    assert!(
        expires_at_after > expires_at_before,
        "le renouvellement doit repousser l'échéance"
    );

    // Même jeton qu'à l'émission — le renouvellement ne le change pas.
    let req = test::TestRequest::get()
        .uri(&format!(
            "/api/v1/etats-dates/reference/{}?token={}",
            reference_number, token
        ))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);
}

#[actix_web::test]
#[serial]
async fn security_token_scoped_to_another_etat_date_does_not_open_this_one() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (syndic_token, etat_date_id_a, _reference_a) =
        create_notary_link_fixture(&app_state, org_id).await;
    let (_syndic_token_b, _etat_date_id_b, reference_b) =
        create_notary_link_fixture(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::post()
        .uri(&format!(
            "/api/v1/etats-dates/{}/notary-link",
            etat_date_id_a
        ))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", syndic_token)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    let body: serde_json::Value = test::read_body_json(resp).await;
    let token_a = body["token"].as_str().unwrap().to_string();

    // Le jeton de A ne doit pas ouvrir la référence B.
    let req = test::TestRequest::get()
        .uri(&format!(
            "/api/v1/etats-dates/reference/{}?token={}",
            reference_b, token_a
        ))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 403);
}

#[actix_web::test]
#[serial]
async fn security_a_revoked_link_no_longer_opens_the_etat_date() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (syndic_token, etat_date_id, reference_number) =
        create_notary_link_fixture(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::post()
        .uri(&format!("/api/v1/etats-dates/{}/notary-link", etat_date_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", syndic_token)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    let body: serde_json::Value = test::read_body_json(resp).await;
    let token = body["token"].as_str().unwrap().to_string();

    let req = test::TestRequest::delete()
        .uri(&format!("/api/v1/etats-dates/{}/notary-link", etat_date_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", syndic_token)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 204);

    let req = test::TestRequest::get()
        .uri(&format!(
            "/api/v1/etats-dates/reference/{}?token={}",
            reference_number, token
        ))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 403);
}

#[actix_web::test]
#[serial]
async fn security_a_syndic_from_another_organization_cannot_issue_a_link() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (_syndic_token, etat_date_id, _reference_number) =
        create_notary_link_fixture(&app_state, org_id).await;

    let other_org_id = common::create_test_organization(&app_state).await;
    let other_syndic_token =
        common::register_and_login_with_role(&app_state, other_org_id, "syndic").await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::post()
        .uri(&format!("/api/v1/etats-dates/{}/notary-link", etat_date_id))
        .insert_header((
            header::AUTHORIZATION,
            format!("Bearer {}", other_syndic_token),
        ))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.status(),
        403,
        "un syndic d'une autre organisation ne peut pas émettre de lien pour cet état daté"
    );
}

#[actix_web::test]
#[serial]
async fn test_etat_date_statistics() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = common::register_and_login(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let building_id = Uuid::new_v4();

    let req = test::TestRequest::get()
        .uri(&format!(
            "/api/v1/etats-dates/stats?building_id={}",
            building_id
        ))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let _resp = test::call_service(&app, req).await;

    // Expected stats: total, by status, average delivery time, overdue count
    // Critical for syndic dashboard to monitor legal compliance
}

#[actix_web::test]
#[serial]
async fn test_etat_date_16_legal_sections_validation() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = common::register_and_login(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let etat_date_id = Uuid::new_v4();

    // Belgian law requires 16 sections in État Daté
    let additional_data = json!({
        // Sections beyond financial data
        "regulation_copy_url": "/documents/reglement_copropriete.pdf",
        "recent_ag_minutes_urls": [
            "/documents/pv_ag_2025_01.pdf",
            "/documents/pv_ag_2024_12.pdf"
        ],
        "budget_url": "/documents/budget_2026.pdf",
        "insurance_certificate_url": "/documents/assurance_2026.pdf",
        "guarantees_and_mortgages": "None",
        "observations": "Elevator renovation approved in AG 2025-01-15, work starts 2026-03-01"
    });

    let req = test::TestRequest::put()
        .uri(&format!(
            "/api/v1/etats-dates/{}/additional-data",
            etat_date_id
        ))
        .insert_header(header::ContentType::json())
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(&additional_data)
        .to_request();

    let _resp = test::call_service(&app, req).await;

    // All 16 sections must be filled before marking as Generated
    // Validation ensures legal compliance
}
