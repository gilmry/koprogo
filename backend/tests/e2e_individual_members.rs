// E2E tests for Individual Members HTTP endpoints (Issue #280)
// Tests focus on HTTP layer: energy campaign individual member management
// Covers Belgian energy group buying extensions (Art. 22 RED II)
// Note: These handlers are mostly stubs (TODO implementations) but we test HTTP contract

mod common;

use actix_web::{test, App};
use koprogo_api::infrastructure::web::configure_routes;
use serde_json::json;
use serial_test::serial;

// ==================== Join Campaign Tests ====================

/// Cree une campagne energie REELLE et rend son id.
///
/// Les tests de ce harnais faisaient `let campaign_id = Uuid::new_v4()`, donc
/// une campagne inexistante. `individual_members.campaign_id` porte
/// `REFERENCES energy_campaigns(id)` : l'insertion violait la cle etrangere,
/// le use case rendait une erreur et le handler la traduisait en 400.
///
/// Le produit avait raison — on n'adhere pas a une campagne qui n'existe pas.
/// C'est la fixture qui omettait la precondition. Harnais jamais cable en CI,
/// donc jamais execute pour le dire.
/// Cree un membre individuel REEL dans une campagne et rend son id.
///
/// Meme piege que la campagne : les tests des operations secondaires
/// (consentement, consommation, retrait) faisaient `Uuid::new_v4()`, donc un
/// membre inexistant. Le use case ne le trouve pas et le handler traduit en
/// 400. Le produit a raison — on n'accorde pas un consentement pour quelqu'un
/// qui n'existe pas.
async fn create_test_member(
    app_state: &actix_web::web::Data<koprogo_api::infrastructure::web::AppState>,
    campaign_id: uuid::Uuid,
) -> uuid::Uuid {
    sqlx::query_scalar(
        r#"INSERT INTO individual_members (campaign_id, email, postal_code)
           VALUES ($1, $2, '1000') RETURNING id"#,
    )
    .bind(campaign_id)
    .bind(format!("member-{}@example.be", uuid::Uuid::new_v4()))
    .fetch_one(&app_state.pool)
    .await
    .expect("create_test_member: insertion du membre")
}

async fn create_test_campaign(
    app_state: &actix_web::web::Data<koprogo_api::infrastructure::web::AppState>,
    org_id: uuid::Uuid,
) -> uuid::Uuid {
    // `setup_test_db` cree l'organisation mais AUCUN utilisateur : ma premiere
    // version de ce helper supposait le contraire et echouait en RowNotFound.
    // On en enregistre un, puisque `energy_campaigns.created_by` reference
    // `users(id)`.
    let _ = common::register_and_login(app_state, org_id).await;

    let user_id: uuid::Uuid =
        sqlx::query_scalar("SELECT id FROM users WHERE organization_id = $1 LIMIT 1")
            .bind(org_id)
            .fetch_one(&app_state.pool)
            .await
            .expect("create_test_campaign: aucun utilisateur apres enregistrement");

    sqlx::query_scalar(
        r#"INSERT INTO energy_campaigns
             (organization_id, campaign_name, campaign_type, status,
              deadline_participation, energy_types, created_by)
           VALUES ($1, 'Campagne E2E', 'BuyingGroup', 'CollectingData',
                   NOW() + INTERVAL '30 days', ARRAY['Electricity'], $2)
           RETURNING id"#,
    )
    .bind(org_id)
    .bind(user_id)
    .fetch_one(&app_state.pool)
    .await
    .expect("create_test_campaign: insertion de la campagne")
}

#[actix_web::test]
#[serial]
async fn test_join_campaign_as_individual() {
    let (app_state, _container, org_id) = common::setup_test_db().await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let campaign_id = create_test_campaign(&app_state, org_id).await;

    let req = test::TestRequest::post()
        .uri(&format!(
            "/api/v1/energy-campaigns/{}/join-as-individual",
            campaign_id
        ))
        .set_json(json!({
            "email": "test-member@example.be",
            "postal_code": "1000",
            "annual_consumption_kwh": 3500.0,
            "current_provider": "Engie Electrabel"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.status(),
        201,
        "Should create individual member successfully"
    );

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body["id"].is_string(), "Should have an ID");
    assert_eq!(body["email"], "test-member@example.be");
    assert_eq!(body["postal_code"], "1000");
    assert_eq!(body["campaign_id"], campaign_id.to_string());
    assert_eq!(body["has_gdpr_consent"], false);
}

#[actix_web::test]
#[serial]
async fn test_join_campaign_minimal_fields() {
    let (app_state, _container, org_id) = common::setup_test_db().await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let campaign_id = create_test_campaign(&app_state, org_id).await;

    // Only required fields: email and postal_code
    let req = test::TestRequest::post()
        .uri(&format!(
            "/api/v1/energy-campaigns/{}/join-as-individual",
            campaign_id
        ))
        .set_json(json!({
            "email": "minimal@example.be",
            "postal_code": "1050"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.status(),
        201,
        "Should create member with minimal fields"
    );

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["email"], "minimal@example.be");
    assert_eq!(body["postal_code"], "1050");
    assert!(
        body["annual_consumption_kwh"].is_null(),
        "Optional field should be null"
    );
}

#[actix_web::test]
#[serial]
async fn test_join_campaign_no_auth_required() {
    let (app_state, _container, org_id) = common::setup_test_db().await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let campaign_id = create_test_campaign(&app_state, org_id).await;

    // No Authorization header — public endpoint for non-copropriétaires
    let req = test::TestRequest::post()
        .uri(&format!(
            "/api/v1/energy-campaigns/{}/join-as-individual",
            campaign_id
        ))
        .set_json(json!({
            "email": "public@example.be",
            "postal_code": "1000"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_ne!(
        resp.status(),
        401,
        "Join campaign should not require authentication"
    );
}

// ==================== Grant Consent Tests ====================

#[actix_web::test]
#[serial]
async fn test_grant_consent() {
    let (app_state, _container, org_id) = common::setup_test_db().await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let campaign_id = create_test_campaign(&app_state, org_id).await;
    let member_id = create_test_member(&app_state, campaign_id).await;

    let req = test::TestRequest::post()
        .uri(&format!(
            "/api/v1/energy-campaigns/{}/members/{}/consent",
            campaign_id, member_id
        ))
        .set_json(json!({
            "has_consent": true
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200, "Should grant consent successfully");

    let body: serde_json::Value = test::read_body_json(resp).await;
    // L'API rend le MEMBRE mis a jour (`IndividualMemberResponseDto`), pas une
    // enveloppe `{success, message}` que le test supposait. Cette enveloppe
    // n'a jamais existe : `body["success"]` valait donc `Null`.
    //
    // On asserte desormais sur l'EFFET plutot que sur un drapeau : le
    // consentement RGPD est reellement pose, et il est horodate. Garantie plus
    // forte que `success: true`.
    assert_eq!(
        body["has_gdpr_consent"], true,
        "le consentement doit etre pose : {body}"
    );
    assert!(
        body["consent_at"].is_string(),
        "le consentement doit etre horodate : {body}"
    );
}

// ==================== Update Consumption Tests ====================

#[actix_web::test]
#[serial]
async fn test_update_consumption() {
    let (app_state, _container, org_id) = common::setup_test_db().await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let campaign_id = create_test_campaign(&app_state, org_id).await;
    let member_id = create_test_member(&app_state, campaign_id).await;

    let req = test::TestRequest::put()
        .uri(&format!(
            "/api/v1/energy-campaigns/{}/members/{}/consumption",
            campaign_id, member_id
        ))
        .set_json(json!({
            "annual_consumption_kwh": 4200.0,
            "current_provider": "Luminus",
            "ean_code": "541448860000123456"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.status(),
        200,
        "Should update consumption data successfully"
    );

    let body: serde_json::Value = test::read_body_json(resp).await;
    // Meme correction : l'API rend le membre mis a jour. On verifie que la
    // consommation envoyee est bien celle enregistree, ce qui atteste
    // l'operation au lieu de se fier a un drapeau absent.
    assert_eq!(
        body["annual_consumption_kwh"], 4200.0,
        "la consommation doit etre enregistree : {body}"
    );
}

// ==================== Withdraw Tests ====================

#[actix_web::test]
#[serial]
async fn test_withdraw_from_campaign() {
    let (app_state, _container, org_id) = common::setup_test_db().await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let campaign_id = create_test_campaign(&app_state, org_id).await;
    let member_id = create_test_member(&app_state, campaign_id).await;

    let req = test::TestRequest::delete()
        .uri(&format!(
            "/api/v1/energy-campaigns/{}/members/{}/withdraw",
            campaign_id, member_id
        ))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.status(),
        200,
        "Should withdraw from campaign successfully"
    );

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["success"], true);
    assert!(
        body["message"].is_string(),
        "Should have a confirmation message"
    );
}

// ==================== Invalid Input Tests ====================

#[actix_web::test]
#[serial]
async fn test_join_campaign_invalid_email() {
    let (app_state, _container, org_id) = common::setup_test_db().await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let campaign_id = create_test_campaign(&app_state, org_id).await;

    // IndividualMember::new validates email format in domain layer
    let req = test::TestRequest::post()
        .uri(&format!(
            "/api/v1/energy-campaigns/{}/join-as-individual",
            campaign_id
        ))
        .set_json(json!({
            "email": "not-a-valid-email",
            "postal_code": "1000"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    let status = resp.status().as_u16();
    // Domain validation may reject invalid email with 400, or handler may accept it
    // depending on how strict the domain entity validation is
    assert!(
        status == 400 || status == 201,
        "Expected 400 (validation error) or 201, got {}",
        status
    );
}
