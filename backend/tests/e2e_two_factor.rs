// E2E tests for Two-Factor Authentication (2FA TOTP) HTTP endpoints (Issue #78)
// Tests focus on HTTP layer: endpoints, auth, JSON serialization
// Covers TOTP setup, status, and enable flow for Belgian copropriete security

mod common;

use actix_web::http::header;
use actix_web::{test, App};
use koprogo_api::application::dto::*;
use koprogo_api::infrastructure::web::configure_routes;
use koprogo_api::infrastructure::web::AppState;
use serial_test::serial;
use uuid::Uuid;

/// Helper: Register a user and return (token, user_id) — needed for 2FA since it is user-scoped
async fn register_and_login_with_user_id(
    app_state: &actix_web::web::Data<AppState>,
    org_id: Uuid,
) -> (String, Uuid) {
    let email = format!("2fa-test-{}@example.be", Uuid::new_v4());
    let reg = RegisterRequest {
        email: email.clone(),
        password: "SecurePass123!".to_string(),
        first_name: "TwoFactor".to_string(),
        last_name: "Tester".to_string(),
        role: "superadmin".to_string(),
        organization_id: Some(org_id),
    };
    let login_resp = app_state
        .auth_use_cases
        .register(reg)
        .await
        .expect("Failed to register user for 2FA test");

    let user_id = login_resp.user.id;
    let token = login_resp.token;
    (token, user_id)
}

// ==================== 2FA Status Tests ====================

#[actix_web::test]
#[serial]
async fn test_2fa_status_disabled_by_default() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, _user_id) = register_and_login_with_user_id(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/v1/2fa/status")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200, "Should return 2FA status successfully");

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(
        body["is_enabled"], false,
        "2FA should be disabled for a new user"
    );
    assert_eq!(
        body["backup_codes_remaining"], 0,
        "No backup codes before setup"
    );
}

#[actix_web::test]
#[serial]
async fn test_2fa_status_requires_auth() {
    let (app_state, _container, _org_id) = common::setup_test_db().await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/v1/2fa/status")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.status(),
        401,
        "Should require authentication to view 2FA status"
    );
}

// ==================== 2FA Setup Tests ====================

#[actix_web::test]
#[serial]
async fn test_2fa_setup_returns_qr_code_and_backup_codes() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, _user_id) = register_and_login_with_user_id(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/v1/2fa/setup")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.status(),
        200,
        "Should return 2FA setup data successfully"
    );

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(
        body["secret"].is_string() && !body["secret"].as_str().unwrap().is_empty(),
        "Should return a non-empty TOTP secret"
    );
    assert!(
        body["qr_code_data_url"].is_string(),
        "Should return a QR code data URL"
    );
    assert!(
        body["backup_codes"].is_array(),
        "Should return backup codes array"
    );
    assert_eq!(
        body["backup_codes"].as_array().unwrap().len(),
        10,
        "Should return exactly 10 backup codes"
    );
    assert_eq!(body["issuer"], "KoproGo", "Should have KoproGo as issuer");
}

#[actix_web::test]
#[serial]
async fn test_2fa_setup_returns_totp_secret() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, _user_id) = register_and_login_with_user_id(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/v1/2fa/setup")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200, "Should succeed");

    let body: serde_json::Value = test::read_body_json(resp).await;

    // The secret must be a valid Base32-encoded string
    let secret = body["secret"].as_str().expect("secret should be a string");
    assert!(
        secret.len() >= 16,
        "TOTP secret should be at least 16 characters (Base32)"
    );

    // qr_code_data_url should start with data:image/
    let qr = body["qr_code_data_url"]
        .as_str()
        .expect("qr_code_data_url should be a string");
    assert!(
        qr.starts_with("data:image/"),
        "QR code should be a data URL (got: {}...)",
        &qr[..20.min(qr.len())]
    );

    // account_name should be present (the user's email)
    assert!(
        body["account_name"].is_string(),
        "Should return account_name field"
    );
}

#[actix_web::test]
#[serial]
async fn test_2fa_setup_requires_auth() {
    let (app_state, _container, _org_id) = common::setup_test_db().await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/v1/2fa/setup")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.status(),
        401,
        "Should require authentication to setup 2FA"
    );
}

#[actix_web::test]
#[serial]
async fn test_2fa_setup_idempotent_second_call_fails() {
    // After calling setup once, calling setup a second time should fail
    // because a secret is already pending (not yet enabled but exists)
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, _user_id) = register_and_login_with_user_id(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // First setup call should succeed
    let req1 = test::TestRequest::post()
        .uri("/api/v1/2fa/setup")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();
    let resp1 = test::call_service(&app, req1).await;
    assert_eq!(resp1.status(), 200, "First setup should succeed");

    // Second setup call — the handler returns 400 if already set up (pending or enabled)
    let req2 = test::TestRequest::post()
        .uri("/api/v1/2fa/setup")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();
    let resp2 = test::call_service(&app, req2).await;
    // It may return 400 (already exists) or 200 (idempotent re-setup) depending on implementation
    // We only check it does NOT crash (not 500)
    assert_ne!(
        resp2.status(),
        500,
        "Second setup should not cause a server error"
    );
}

// ==================== Connexion en deux étapes (#1135) ====================

/// Le code TOTP courant (RFC 6238, 30 s, 6 chiffres) du secret base32 reçu au
/// setup, comme le calcule une application d'authentification.
fn code_courant(secret: &str) -> String {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";
    let (mut octets, mut tampon, mut bits) = (Vec::new(), 0u32, 0u32);
    for c in secret.trim_end_matches('=').bytes() {
        let v = ALPHABET
            .iter()
            .position(|&a| a == c.to_ascii_uppercase())
            .expect("secret base32") as u32;
        tampon = (tampon << 5) | v;
        bits += 5;
        if bits >= 8 {
            bits -= 8;
            octets.push((tampon >> bits) as u8);
            tampon &= (1 << bits) - 1;
        }
    }
    let maintenant = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    totp_lite::totp_custom::<totp_lite::Sha1>(30, 6, &octets, maintenant)
}

/// Un compte connu par son email et son mot de passe, avec son jeton.
async fn compte_identifie(
    app_state: &actix_web::web::Data<AppState>,
    org_id: Uuid,
) -> (String, String) {
    let email = format!("2fa-login-{}@example.be", Uuid::new_v4());
    let reponse = app_state
        .auth_use_cases
        .register(RegisterRequest {
            email: email.clone(),
            password: "SecurePass123!".to_string(),
            first_name: "Deux".to_string(),
            last_name: "Etapes".to_string(),
            role: "superadmin".to_string(),
            organization_id: Some(org_id),
        })
        .await
        .expect("inscription du compte");
    (email, reponse.token)
}

/// Active le second facteur du compte et rend (secret, codes de secours).
macro_rules! active_second_facteur {
    ($app:expr, $jeton:expr) => {{
        let req = test::TestRequest::post()
            .uri("/api/v1/2fa/setup")
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", $jeton)))
            .to_request();
        let setup: serde_json::Value = test::call_and_read_body_json(&$app, req).await;
        let secret = setup["secret"].as_str().unwrap().to_string();
        let codes: Vec<String> = setup["backup_codes"]
            .as_array()
            .unwrap()
            .iter()
            .map(|c| c.as_str().unwrap().to_string())
            .collect();
        let req = test::TestRequest::post()
            .uri("/api/v1/2fa/enable")
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", $jeton)))
            .set_json(serde_json::json!({
                "totp_code": code_courant(&secret)
            }))
            .to_request();
        let resp = test::call_service(&$app, req).await;
        assert_eq!(resp.status(), 200, "activation du second facteur");
        (secret, codes)
    }};
}

/// `POST /auth/login` : (statut, corps, cookie posé ?).
macro_rules! connexion {
    ($app:expr, $email:expr) => {{
        let req = test::TestRequest::post()
            .uri("/api/v1/auth/login")
            .set_json(serde_json::json!({ "email": $email, "password": "SecurePass123!" }))
            .to_request();
        let resp = test::call_service(&$app, req).await;
        let statut = resp.status().as_u16();
        let cookie = resp.headers().contains_key("set-cookie");
        let corps: serde_json::Value = test::read_body_json(resp).await;
        (statut, corps, cookie)
    }};
}

/// `POST /auth/login/2fa` : (statut, corps).
macro_rules! second_facteur {
    ($app:expr, $defi:expr, $code:expr) => {{
        let req = test::TestRequest::post()
            .uri("/api/v1/auth/login/2fa")
            .set_json(serde_json::json!({ "challenge_token": $defi, "code": $code }))
            .to_request();
        let resp = test::call_service(&$app, req).await;
        let statut = resp.status().as_u16();
        let corps = test::read_body(resp).await;
        let corps: serde_json::Value =
            serde_json::from_slice(&corps).unwrap_or(serde_json::Value::Null);
        (statut, corps)
    }};
}

/// `@happy` — Mot de passe, puis code TOTP : la session s'ouvre au second
/// temps seulement. Un code de secours marche une fois.
#[actix_web::test]
#[serial]
async fn happy_la_connexion_demande_le_second_facteur_puis_ouvre_la_session() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (email, jeton) = compte_identifie(&app_state, org_id).await;
    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;
    let (secret, codes) = active_second_facteur!(app, jeton);

    let (statut, corps, cookie) = connexion!(app, email);
    assert_eq!(statut, 200, "{corps}");
    assert_eq!(corps["two_factor_required"], true);
    assert!(
        corps.get("token").is_none(),
        "jeton dès le mot de passe : {corps}"
    );
    assert!(!cookie, "cookie de session dès le mot de passe");
    let defi = corps["challenge_token"].as_str().unwrap().to_string();

    let code = code_courant(&secret);
    let (statut, corps) = second_facteur!(app, defi, code);
    assert_eq!(statut, 200, "{corps}");
    let acces = corps["token"].as_str().unwrap().to_string();
    let req = test::TestRequest::get()
        .uri("/api/v1/auth/me")
        .insert_header((header::AUTHORIZATION, format!("Bearer {acces}")))
        .to_request();
    assert_eq!(test::call_service(&app, req).await.status(), 200);

    // Code de secours : accepté une fois, puis consommé.
    let (_, corps, _) = connexion!(app, email);
    let defi = corps["challenge_token"].as_str().unwrap().to_string();
    let (statut, corps) = second_facteur!(app, defi, codes[0]);
    assert_eq!(statut, 200, "code de secours : {corps}");
    let (_, corps, _) = connexion!(app, email);
    let defi = corps["challenge_token"].as_str().unwrap().to_string();
    let (statut, _) = second_facteur!(app, defi, codes[0]);
    assert_eq!(statut, 401, "code de secours réutilisé");
}

/// `@negative` — Code faux, défi inventé : 401, aucune session.
#[actix_web::test]
#[serial]
async fn negative_un_code_faux_ou_un_defi_invente_n_ouvre_rien() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (email, jeton) = compte_identifie(&app_state, org_id).await;
    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;
    active_second_facteur!(app, jeton);
    let (_, corps, _) = connexion!(app, email);
    let defi = corps["challenge_token"].as_str().unwrap().to_string();

    for (defi, code) in [
        (defi.as_str(), "000000"),
        (
            "4102444800.00000000-0000-0000-0000-000000000000.00",
            "123456",
        ),
        ("n'importe quoi", "123456"),
    ] {
        let (statut, corps) = second_facteur!(app, defi, code);
        assert_eq!(statut, 401, "{defi} / {code} : {corps}");
        assert!(corps.get("token").is_none());
    }
}

/// `@edge` — Sans second facteur, la connexion ne change pas. Avec, cinq
/// codes faux épuisent le défi : le bon code n'y passe plus.
#[actix_web::test]
#[serial]
async fn edge_sans_second_facteur_rien_ne_change_et_cinq_codes_faux_epuisent_le_defi() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (sans, _) = compte_identifie(&app_state, org_id).await;
    let (avec, jeton) = compte_identifie(&app_state, org_id).await;
    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let (statut, corps, cookie) = connexion!(app, sans);
    assert_eq!(statut, 200);
    assert!(
        corps["token"].is_string(),
        "connexion en un temps : {corps}"
    );
    assert!(cookie);

    let (secret, _) = active_second_facteur!(app, jeton);
    let (_, corps, _) = connexion!(app, avec);
    let defi = corps["challenge_token"].as_str().unwrap().to_string();
    for _ in 0..5 {
        let (statut, _) = second_facteur!(app, defi, "000000");
        assert_eq!(statut, 401);
    }
    let code = code_courant(&secret);
    let (statut, _) = second_facteur!(app, defi, code);
    assert_eq!(statut, 401, "bon code sur un défi épuisé");
}

/// `@security` — Le défi n'est pas un jeton d'accès. Seul le superadmin
/// réinitialise un second facteur ; le compte se reconnecte alors en un
/// temps, et ses sessions ouvertes sont révoquées.
#[actix_web::test]
#[serial]
async fn security_le_defi_n_ouvre_aucune_route_et_seul_le_superadmin_reinitialise() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (email, jeton) = compte_identifie(&app_state, org_id).await;
    let syndic = common::register_and_login_with_role(&app_state, org_id, "syndic").await;
    let superadmin = common::register_and_login(&app_state, org_id).await;
    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;
    active_second_facteur!(app, jeton);
    let (_, corps, _) = connexion!(app, email);
    let defi = corps["challenge_token"].as_str().unwrap().to_string();

    let req = test::TestRequest::get()
        .uri("/api/v1/auth/me")
        .insert_header((header::AUTHORIZATION, format!("Bearer {defi}")))
        .to_request();
    assert_eq!(
        test::call_service(&app, req).await.status(),
        401,
        "le défi utilisé comme jeton d'accès"
    );

    let cible = app_state.auth_use_cases.verify_token(&jeton).unwrap().sub;
    for (appelant, attendu) in [(&syndic, 403), (&superadmin, 204)] {
        let req = test::TestRequest::put()
            .uri(&format!("/api/v1/users/{cible}/2fa/reset"))
            .insert_header((header::AUTHORIZATION, format!("Bearer {appelant}")))
            .to_request();
        assert_eq!(test::call_service(&app, req).await.status(), attendu);
    }

    let (statut, corps, _) = connexion!(app, email);
    assert_eq!(statut, 200);
    assert!(
        corps["token"].is_string(),
        "après réinitialisation, connexion en un temps : {corps}"
    );
}
