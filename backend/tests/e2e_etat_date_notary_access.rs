// E2E HTTP tests for the notary access token on `GET
// /etats-dates/reference/{reference_number}` (issue #855, ADR 0048).
//
// The full happy-path / edge (resource-binding) round-trip — issue a token
// then read the état daté through it — is already covered at the use-case
// layer (`magic_link_use_cases.rs::tests`), which is where the actual
// business logic lives; the handler is a thin pass-through. Building a real
// état daté here would require a full building + unit + owner fixture chain
// that doesn't yet have a test helper (see `tests/common/mod.rs`).
//
// What these tests validate is the HTTP wiring itself: the route no longer
// serves data to an anonymous caller, refuses uniformly regardless of cause,
// and rate-limits repeated attempts — none of which require a real état daté
// to exist (a nonexistent reference is refused exactly like an invalid
// token, which is the point).

mod common;

use actix_web::http::header;
use actix_web::{test, App};
use koprogo_api::infrastructure::web::configure_routes;
use serde_json::{json, Value};
use serial_test::serial;
use uuid::Uuid;

#[actix_web::test]
#[serial]
async fn negative_get_by_reference_without_token_is_refused() {
    let (app_state, _container, _org_id) = common::setup_test_db().await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/v1/etats-dates/reference/ED-2026-000-INEXISTANT")
        .to_request();

    let resp = test::call_service(&app, req).await;

    assert_eq!(
        resp.status(),
        403,
        "sans jeton, la route doit refuser plutôt que servir la donnée"
    );
}

#[actix_web::test]
#[serial]
async fn negative_unknown_reference_and_forged_token_are_refused_identically() {
    let (app_state, _container, _org_id) = common::setup_test_db().await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Référence inexistante, aucun jeton.
    let req_no_token = test::TestRequest::get()
        .uri("/api/v1/etats-dates/reference/ED-2026-000-INEXISTANT")
        .to_request();
    let resp_no_token = test::call_service(&app, req_no_token).await;
    let status_no_token = resp_no_token.status();
    let body_no_token: Value = test::read_body_json(resp_no_token).await;

    // Référence inexistante, jeton forgé.
    let req_forged = test::TestRequest::get()
        .uri("/api/v1/etats-dates/reference/ED-2026-000-INEXISTANT?token=forged-token")
        .to_request();
    let resp_forged = test::call_service(&app, req_forged).await;
    let status_forged = resp_forged.status();
    let body_forged: Value = test::read_body_json(resp_forged).await;

    // Un attaquant qui varie le jeton ne doit rien apprendre : même statut,
    // même message, que la référence existe ou non côté serveur.
    assert_eq!(status_no_token, status_forged);
    assert_eq!(status_no_token, 403);
    assert_eq!(
        body_no_token, body_forged,
        "le refus doit être uniforme, cf. issue #855 critère @negative"
    );
}

#[actix_web::test]
#[serial]
async fn security_repeated_attempts_are_rate_limited() {
    let (app_state, _container, _org_id) = common::setup_test_db().await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // 32 bits d'aléa dans une référence ne résistent pas à un balayage non
    // borné (issue #855 critère @security) — la garantie testée ici est que
    // la route se ferme avant que ça n'ait d'importance.
    let mut saw_rate_limited = false;
    for _ in 0..25 {
        let req = test::TestRequest::get()
            .uri("/api/v1/etats-dates/reference/ED-2026-000-BALAYAGE")
            .to_request();
        let resp = test::call_service(&app, req).await;
        if resp.status() == 429 {
            saw_rate_limited = true;
            break;
        }
        assert_eq!(resp.status(), 403);
    }

    assert!(
        saw_rate_limited,
        "25 tentatives rapides sans jeton devraient finir par être limitées (429)"
    );
}

/// @security — trouvé en revue (rust-expert) après le premier jet de cette
/// story : le endpoint générique `POST /magic-links` ne vérifie que le rôle,
/// jamais que l'appelant a la main sur l'organisation propriétaire de
/// `scope_id`. Accepter `scope_kind: "etat_date"` ici contournerait
/// entièrement `verify_etat_date_org_access`, posé sur la route dédiée
/// `POST /etats-dates/{id}/notary-access` — un syndic du cabinet A pourrait
/// émettre un lien pour l'état daté du cabinet B et le lire via
/// `GET /etats-dates/reference/{ref}?token=`, qui ne vérifie que la
/// correspondance jeton↔ressource, pas l'organisation.
#[actix_web::test]
#[serial]
async fn security_generic_magic_link_endpoint_refuses_etat_date_scope() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = common::register_and_login_with_role(&app_state, org_id, "syndic").await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let body = json!({
        "subject_user_id": Uuid::new_v4(),
        "scope_kind": "etat_date",
        "scope_id": Uuid::new_v4(),
        "expires_in_seconds": 3600,
    });

    let req = test::TestRequest::post()
        .uri("/api/v1/magic-links")
        .insert_header(header::ContentType::json())
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(&body)
        .to_request();

    let resp = test::call_service(&app, req).await;

    assert_eq!(
        resp.status(),
        400,
        "la portée etat_date doit être refusée sur le endpoint générique, \
         faute de quoi le cloisonnement organisationnel est contournable"
    );
}
