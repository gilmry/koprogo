// E2E tests for Consent Recording HTTP endpoints (Issue #315 - GDPR Art. 13-14)
// Tests focus on HTTP layer: endpoints, auth, JSON serialization
// Covers consent lifecycle for privacy policy and terms of service

mod common;

use actix_web::http::header;
use actix_web::{test, App};
use koprogo_api::infrastructure::web::configure_routes;
use koprogo_api::infrastructure::web::AppState;
use serde_json::json;
use serial_test::serial;
use uuid::Uuid;

// ==================== Test Setup ====================

async fn create_test_user(
    app_state: &actix_web::web::Data<AppState>,
    org_id: Uuid,
) -> (Uuid, String) {
    let email = format!("consent_test_{}@example.com", Uuid::new_v4());
    let register_result = app_state
        .auth_use_cases
        .register(koprogo_api::application::dto::RegisterRequest {
            email: email.clone(),
            password: "TestPassword123!".to_string(),
            first_name: "Consent".to_string(),
            last_name: "Tester".to_string(),
            role: "superadmin".to_string(),
            organization_id: Some(org_id),
        })
        .await
        .expect("Failed to register test user");

    let login_result = app_state
        .auth_use_cases
        .login(koprogo_api::application::dto::LoginRequest {
            email,
            password: "TestPassword123!".to_string(),
        })
        .await
        .expect("Failed to login test user");

    (register_result.user.id, login_result.token)
}

// ==================== Record Consent Tests ====================

#[actix_web::test]
#[serial]
async fn test_record_privacy_policy_consent() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (_user_id, token) = create_test_user(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/v1/consent")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "consent_type": "privacy_policy",
            "policy_version": "1.0"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200, "Should record privacy policy consent");

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body["message"].as_str().unwrap().contains("privacy_policy"));
    assert_eq!(body["consent_type"], "privacy_policy");
    assert!(body["accepted_at"].as_str().is_some());
}

#[actix_web::test]
#[serial]
async fn test_record_terms_consent() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (_user_id, token) = create_test_user(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/v1/consent")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "consent_type": "terms"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200, "Should record terms consent");

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["consent_type"], "terms");
}

#[actix_web::test]
#[serial]
async fn test_record_consent_invalid_type() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (_user_id, _token) = create_test_user(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Jeton PORTE PAR UNE ORGANISATION, et non superadmin.
    //
    // `record_consent` verifie `auth.require_organization()` AVANT de valider
    // le `consent_type`. Or le JWT d'un superadmin porte
    // `organization_id: None` : la requete s'arretait sur
    // « Organization context required to record consent » et n'atteignait
    // jamais la validation que ce test vise.
    //
    // Le test passait donc son assertion de statut (400) POUR LA MAUVAISE
    // RAISON, et n'echouait que sur le message. Un 400 obtenu par un autre
    // chemin que celui teste n'atteste rien.
    let org_token = common::register_and_login_with_role(&app_state, org_id, "syndic").await;

    let req = test::TestRequest::post()
        .uri("/api/v1/consent")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", org_token)))
        .set_json(json!({
            "consent_type": "invalid_type"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400, "Should reject invalid consent type");

    let body: serde_json::Value = test::read_body_json(resp).await;
    // Le produit dit « Invalid consent type » (avec une ESPACE), et enumere
    // les valeurs admises. Le test cherchait « Invalid consent_type », avec un
    // souligne : la sous-chaine ne correspondait pas au libelle reel.
    //
    // On asserte sur ce que le produit garantit vraiment — que le message
    // nomme la valeur refusee et rappelle les valeurs admises — plutot que sur
    // une graphie exacte qui se casse au premier reformulage.
    let msg = body["error"].as_str().unwrap_or("<absent>");
    assert!(
        msg.contains("invalid_type") && msg.contains("privacy_policy"),
        "message inattendu : {msg}"
    );
}

#[actix_web::test]
#[serial]
async fn test_record_consent_without_auth() {
    let (app_state, _container, _org_id) = common::setup_test_db().await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/v1/consent")
        .set_json(json!({
            "consent_type": "privacy_policy"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401, "Should require authentication");
}

// ==================== Get Consent Status Tests ====================

#[actix_web::test]
#[serial]
async fn test_get_consent_status() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (user_id, token) = create_test_user(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/v1/consent/status")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200, "Should return consent status");

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["user_id"], user_id.to_string());
    assert!(body["privacy_policy_accepted"].is_boolean());
    assert!(body["terms_accepted"].is_boolean());
}

#[actix_web::test]
#[serial]
async fn test_get_consent_status_without_auth() {
    let (app_state, _container, _org_id) = common::setup_test_db().await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/v1/consent/status")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.status(),
        401,
        "Should require authentication for consent status"
    );
}

// ==================== Consent Lifecycle Test ====================

#[actix_web::test]
#[serial]
async fn test_consent_lifecycle() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (user_id, token) = create_test_user(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // 1. Check initial consent status
    let status_req = test::TestRequest::get()
        .uri("/api/v1/consent/status")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let status_resp = test::call_service(&app, status_req).await;
    assert_eq!(status_resp.status(), 200);

    let status: serde_json::Value = test::read_body_json(status_resp).await;
    assert_eq!(status["user_id"], user_id.to_string());

    // 2. Record privacy policy consent
    let privacy_req = test::TestRequest::post()
        .uri("/api/v1/consent")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "consent_type": "privacy_policy",
            "policy_version": "1.0"
        }))
        .to_request();

    let privacy_resp = test::call_service(&app, privacy_req).await;
    assert_eq!(privacy_resp.status(), 200);

    // 3. Record terms consent
    let terms_req = test::TestRequest::post()
        .uri("/api/v1/consent")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "consent_type": "terms",
            "policy_version": "1.0"
        }))
        .to_request();

    let terms_resp = test::call_service(&app, terms_req).await;
    assert_eq!(terms_resp.status(), 200);

    // 4. Re-check consent status
    let final_status_req = test::TestRequest::get()
        .uri("/api/v1/consent/status")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let final_status_resp = test::call_service(&app, final_status_req).await;
    assert_eq!(final_status_resp.status(), 200);
}

// ==================== Version Tracking Test ====================

#[actix_web::test]
#[serial]
async fn test_record_consent_with_version() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (_user_id, token) = create_test_user(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Record consent with explicit version
    let req = test::TestRequest::post()
        .uri("/api/v1/consent")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "consent_type": "privacy_policy",
            "policy_version": "2.1"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["consent_type"], "privacy_policy");
    assert!(body["accepted_at"].as_str().is_some());
}
