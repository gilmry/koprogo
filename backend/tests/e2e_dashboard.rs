// E2E tests for Dashboard HTTP endpoints
// Tests focus on HTTP layer: endpoints, auth, JSON serialization

mod common;

use actix_web::http::header;
use actix_web::{test, App};
use koprogo_api::infrastructure::web::configure_routes;
use serial_test::serial;

#[actix_web::test]
#[serial]
async fn test_dashboard_accountant_stats() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = common::register_and_login(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/v1/dashboard/accountant/stats")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200, "Should return dashboard stats");

    let body: serde_json::Value = test::read_body_json(resp).await;

    // ── L'assertion d'avant ne pouvait pas échouer ────────────────────────
    //
    // Elle disait :
    //     body.get("total_expenses").is_some()
    //         || body.get("pending_payments").is_some()
    //         || body.is_object()
    //
    // Le dernier terme est VRAI dès que la réponse est un objet, ce que
    // `read_body_json` garantit déjà. Elle attestait donc l'existence de
    // champs sans jamais la vérifier — et aucun des deux noms cités
    // n'existe : le DTO sert `total_paid` et `total_pending`.
    //
    // C'est la troisième assertion de ce genre trouvée aujourd'hui, après les
    // deux de `e2e_legal_api.rs` qui acceptaient un 500.
    for champ in [
        "total_expenses_current_month",
        "total_paid",
        "paid_percentage",
        "total_pending",
        "pending_percentage",
        "owners_with_overdue",
    ] {
        assert!(
            body.get(champ).is_some(),
            "le champ `{champ}` manque à la réponse : {body}"
        );
    }
}

/// @security — les chiffres financiers ne sont pas servis à un copropriétaire.
///
/// ── Ce que la route faisait ───────────────────────────────────────────────
///
/// Elle prenait `AuthenticatedUser` et ne le consultait QUE pour lire
/// `organization_id`. Tout membre de l'organisation obtenait donc le total
/// encaissé, le total en attente, et surtout `owners_with_overdue` — le
/// nombre de copropriétaires en retard de paiement.
///
/// Ce dernier chiffre dit à un copropriétaire combien de ses voisins ne
/// paient pas. Il n'a aucun titre à le savoir.
///
/// C'est un des 87 cas de #864 : une route qui PREND une identité sans s'en
/// servir pour décider. Plus trompeuse qu'une route nue — celle-ci avait
/// l'air gardée, et passait les trois gardes du dépôt.
#[actix_web::test]
#[serial]
async fn security_dashboard_accountant_stats_refuse_un_coproprietaire() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = common::register_and_login_with_role(&app_state, org_id, "owner").await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/v1/dashboard/accountant/stats")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.status(),
        403,
        "un copropriétaire ne doit pas lire les comptes de la copropriété"
    );
}

/// @happy — le syndic, lui, les lit : c'est son mandat.
#[actix_web::test]
#[serial]
async fn happy_dashboard_accountant_stats_ouvert_au_syndic() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = common::register_and_login_with_role(&app_state, org_id, "syndic").await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/v1/dashboard/accountant/stats")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.status(),
        200,
        "le syndic est mandaté par l'ACP : les comptes lui sont ouverts"
    );
}

#[actix_web::test]
#[serial]
async fn test_dashboard_accountant_stats_unauthorized() {
    let (app_state, _container, _org_id) = common::setup_test_db().await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/v1/dashboard/accountant/stats")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401, "Should require authentication");
}

#[actix_web::test]
#[serial]
async fn test_dashboard_recent_transactions() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = common::register_and_login(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/v1/dashboard/accountant/transactions?limit=5")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200, "Should return recent transactions");

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body.is_array(), "Response should be an array");
}

#[actix_web::test]
#[serial]
async fn test_dashboard_transactions_unauthorized() {
    let (app_state, _container, _org_id) = common::setup_test_db().await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/v1/dashboard/accountant/transactions")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401, "Should require authentication");
}
