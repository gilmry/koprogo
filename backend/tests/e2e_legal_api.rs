// E2E tests for Legal API HTTP endpoints (Issue #277)
// Tests focus on HTTP layer: public legal reference endpoints, no authentication required
// Covers Belgian copropriété legal index (Code Civil, GDPR, PCMN)

mod common;

use actix_web::{test, App};
use koprogo_api::infrastructure::web::configure_routes;
use serial_test::serial;

// ==================== Legal Rules Tests ====================

#[actix_web::test]
#[serial]
async fn test_legal_rules_list_all() {
    let (app_state, _container, _org_id) = common::setup_test_db().await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/v1/legal/rules")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.status().as_u16(),
        200,
        "Expected 200 from /legal/rules"
    );
}

#[actix_web::test]
#[serial]
async fn test_legal_rules_filter_by_role() {
    let (app_state, _container, _org_id) = common::setup_test_db().await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/v1/legal/rules?role=syndic")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.status().as_u16(),
        200,
        "Expected 200 from /legal/rules?role=syndic"
    );
}

#[actix_web::test]
#[serial]
async fn test_legal_rules_filter_by_category() {
    let (app_state, _container, _org_id) = common::setup_test_db().await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/v1/legal/rules?category=assemblee-generale")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.status().as_u16(),
        200,
        "Expected 200 from /legal/rules?category=assemblee-generale"
    );
}

#[actix_web::test]
#[serial]
async fn test_legal_rule_get_by_code() {
    let (app_state, _container, _org_id) = common::setup_test_db().await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Try to get a specific rule by code
    let req = test::TestRequest::get()
        .uri("/api/v1/legal/rules/AG01")
        .to_request();

    let resp = test::call_service(&app, req).await;
    // 200 if rule found, 404 if not found (key AG01 may or may not exist in index)
    let status = resp.status().as_u16();
    assert!(
        status == 200 || status == 404,
        "Expected 200 or 404, got {}",
        status
    );
}

#[actix_web::test]
#[serial]
async fn test_legal_rule_get_nonexistent_code() {
    let (app_state, _container, _org_id) = common::setup_test_db().await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/v1/legal/rules/NONEXISTENT_CODE_XYZ")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.status().as_u16(),
        404,
        "Expected 404 for nonexistent code"
    );
}

#[actix_web::test]
#[serial]
async fn test_legal_ag_sequence() {
    let (app_state, _container, _org_id) = common::setup_test_db().await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/v1/legal/ag-sequence")
        .to_request();

    let resp = test::call_service(&app, req).await;
    let status = resp.status().as_u16();

    // ── Ce test DÉCRIVAIT la panne et l'entérinait ─────────────────────────
    //
    // « 200 if ag_sequence key exists, 500 if not », les deux acceptés. La clé
    // n'existait pas, la route rendait donc 500 depuis toujours, et le test
    // était vert.
    //
    // Trois choses masquaient cette route morte : ce test qui acceptait tout,
    // son unique appelant `LegalHelper.svelte` qui n'était monté nulle part,
    // et sa recette Playwright qui la déclarait « public and functional » dans
    // un en-tête tout en la sautant.
    assert_eq!(
        status, 200,
        "GET /legal/ag-sequence doit répondre 200. Un 500 signifie que la clé \
         `ag_sequence` manque à `legal_index.json`."
    );

    let sequence: serde_json::Value = test::read_body_json(resp).await;
    let etapes = sequence
        .as_array()
        .expect("la séquence d'AG est une liste d'étapes");
    assert!(
        !etapes.is_empty(),
        "une séquence d'assemblée vide ne guide personne"
    );

    // Chaque étape doit porter son numéro et son intitulé : une étape sans
    // libellé n'est pas une étape, c'est une ligne.
    for etape in etapes {
        assert!(
            etape.get("step").and_then(|v| v.as_i64()).is_some(),
            "étape sans numéro : {etape}"
        );
        assert!(
            etape
                .get("point_odj")
                .and_then(|v| v.as_str())
                .is_some_and(|s| !s.trim().is_empty()),
            "étape sans intitulé : {etape}"
        );
    }
}

#[actix_web::test]
#[serial]
async fn test_legal_majority_for_ordinary() {
    let (app_state, _container, _org_id) = common::setup_test_db().await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/v1/legal/majority-for/ordinary")
        .to_request();

    let resp = test::call_service(&app, req).await;
    let status = resp.status().as_u16();

    // ── Ce test acceptait les TROIS issues possibles ────────────────────────
    //
    // Il affirmait « 200 if found, 404 if decision_type not found, 500 if
    // majority_types key missing » et acceptait les trois. Un 500 — le serveur
    // qui échoue sur son propre index — comptait comme un succès.
    //
    // Une assertion qui ne peut pas échouer n'est pas une assertion. Celle-ci
    // était verte depuis toujours, comptait dans les totaux, et attestait
    // d'une route qui n'a JAMAIS fonctionné : `majority_types` était absent de
    // `legal_index.json`, tout comme `ag_sequence`.
    //
    // `ordinary` est la majorité par défaut de l'Art. 3.88 § 1er : si elle
    // n'est pas servie, ce n'est pas un cas limite, c'est une panne.
    assert_eq!(
        status, 200,
        "GET /legal/majority-for/ordinary doit répondre 200. Un 500 signifie \
         que `majority_types` manque à `legal_index.json` ; un 404, que la \
         majorité par défaut de l'Art. 3.88 § 1er n'y figure pas."
    );

    let corps: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(
        corps.get("decision_type").and_then(|v| v.as_str()),
        Some("ordinary"),
        "la réponse doit porter la majorité demandée"
    );
    assert!(
        corps
            .get("article")
            .and_then(|v| v.as_str())
            .is_some_and(|a| a.contains("3.88")),
        "une majorité sans son article ne dit pas sur quoi elle se fonde : {corps}"
    );
}

#[actix_web::test]
#[serial]
async fn test_legal_majority_for_nonexistent_type() {
    let (app_state, _container, _org_id) = common::setup_test_db().await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/v1/legal/majority-for/invalid_type")
        .to_request();

    let resp = test::call_service(&app, req).await;
    let status = resp.status().as_u16();
    // 404 if majority_types exists but type not found, 500 if key missing
    assert!(
        status == 404 || status == 500,
        "Expected 404 or 500 for nonexistent type, got {}",
        status
    );
}

#[actix_web::test]
#[serial]
async fn test_legal_rules_no_auth_required() {
    let (app_state, _container, _org_id) = common::setup_test_db().await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // All legal endpoints should work without authentication
    let endpoints = vec![
        "/api/v1/legal/rules",
        "/api/v1/legal/rules/AG01",
        "/api/v1/legal/ag-sequence",
        "/api/v1/legal/majority-for/ordinary",
    ];

    for uri in endpoints {
        let req = test::TestRequest::get().uri(uri).to_request();
        let resp = test::call_service(&app, req).await;
        let status = resp.status().as_u16();
        // Should never return 401 since these are public endpoints
        assert_ne!(
            status, 401,
            "Legal endpoint {} should not require authentication",
            uri
        );
    }
}
