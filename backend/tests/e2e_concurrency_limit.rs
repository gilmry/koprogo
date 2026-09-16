//! Issue #718 — la rafale de `POST /units` / `GET /acps` observée en E2E prod
//! (run du 2026-08-24 : 242 ✓ / 18 ✘, 9 en 502/timeout) doit se traduire par
//! un refus déterministe (429 + Retry-After), jamais par un 502 ou un
//! timeout silencieux.
//!
//! Ces tests exercent `RequestConcurrencyLimit` en isolation (pas de DB, pas
//! de testcontainers) : le comportement à valider est celui du middleware de
//! délestage lui-même, pas celui d'un endpoit métier particulier. Le pool
//! sqlx qu'il protège est couvert ailleurs (`infrastructure/database/pool.rs`).

use actix_web::{http::StatusCode, test, web, App, HttpResponse};
use futures_util::future::join_all;
use koprogo_api::infrastructure::web::{ConcurrencyLimitConfig, RequestConcurrencyLimit};
use std::time::Duration;

/// Route lente : simule le temps qu'une requête réelle passe à attendre une
/// connexion du pool / à faire son travail, le temps que les autres
/// requêtes concurrentes de chaque test se chevauchent.
async fn slow_ok() -> HttpResponse {
    tokio::time::sleep(Duration::from_millis(100)).await;
    HttpResponse::Ok().finish()
}

async fn health_stub() -> HttpResponse {
    HttpResponse::Ok().finish()
}

/// @happy — une création isolée, seule, répond normalement : pas de 429,
/// pas d'en-tête Retry-After, le slot est disponible.
#[actix_web::test]
async fn happy_single_request_succeeds_under_limit() {
    let limiter = RequestConcurrencyLimit::new(ConcurrencyLimitConfig {
        max_concurrent: 2,
        retry_after_secs: 1,
    });
    let app = test::init_service(
        App::new()
            .wrap(limiter)
            .route("/slow", web::get().to(slow_ok)),
    )
    .await;

    let req = test::TestRequest::get().uri("/slow").to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::OK);
    assert!(
        resp.headers().get("Retry-After").is_none(),
        "une requête servie ne doit pas porter Retry-After"
    );
}

/// @negative — sous rafale, une requête refusée pour cause de charge reçoit
/// un 429 explicite avec Retry-After, pas un 502 : c'est la distinction que
/// la story pose comme critère (« un 502 ne dit pas au client s'il doit
/// réessayer ou si l'écriture a eu lieu »).
#[actix_web::test]
async fn negative_overload_returns_429_with_retry_after_not_502() {
    let limiter = RequestConcurrencyLimit::new(ConcurrencyLimitConfig {
        max_concurrent: 1,
        retry_after_secs: 3,
    });
    let app = test::init_service(
        App::new()
            .wrap(limiter)
            .route("/slow", web::get().to(slow_ok)),
    )
    .await;

    let req1 = test::TestRequest::get().uri("/slow").to_request();
    let req2 = test::TestRequest::get().uri("/slow").to_request();

    let (resp1, resp2) = tokio::time::timeout(
        Duration::from_secs(2),
        futures_util::future::join(
            test::call_service(&app, req1),
            test::call_service(&app, req2),
        ),
    )
    .await
    .expect("les deux requêtes doivent répondre sous 2s, jamais rester en attente");

    let statuses = [resp1.status(), resp2.status()];
    assert!(
        statuses.contains(&StatusCode::OK),
        "une des deux requêtes doit passer : {statuses:?}"
    );
    assert!(
        statuses.contains(&StatusCode::TOO_MANY_REQUESTS),
        "l'autre doit être refusée en 429, pas silencieusement : {statuses:?}"
    );
    for status in statuses {
        assert_ne!(status.as_u16(), 502, "jamais de 502 applicatif");
    }

    let rejected = if resp1.status() == StatusCode::TOO_MANY_REQUESTS {
        &resp1
    } else {
        &resp2
    };
    let retry_after = rejected
        .headers()
        .get("Retry-After")
        .expect("un 429 sans Retry-After ne dit pas au client s'il doit réessayer");
    assert_eq!(retry_after.to_str().unwrap(), "3");
}

/// @edge — le scénario exact du run du 2026-08-24 : `seedConformantUnits()`
/// crée plusieurs lots en séquence, deux workers Playwright l'exécutent en
/// parallèle. Ici, une rafale de 8 requêtes concurrentes contre une limite
/// de 3 : chacune doit recevoir une réponse déterministe (200 ou 429),
/// aucune ne doit rester bloquée au-delà d'un délai raisonnable.
#[actix_web::test]
async fn edge_burst_from_two_workers_stays_deterministic() {
    const MAX_CONCURRENT: usize = 3;
    const BURST_SIZE: usize = 8;

    let limiter = RequestConcurrencyLimit::new(ConcurrencyLimitConfig {
        max_concurrent: MAX_CONCURRENT,
        retry_after_secs: 1,
    });
    let app = test::init_service(
        App::new()
            .wrap(limiter)
            .route("/slow", web::get().to(slow_ok)),
    )
    .await;

    let requests = (0..BURST_SIZE).map(|_| test::TestRequest::get().uri("/slow").to_request());
    let futures = requests.map(|req| test::call_service(&app, req));

    let responses = tokio::time::timeout(Duration::from_secs(5), join_all(futures))
        .await
        .expect("aucune requête de la rafale ne doit dépasser un délai borné (pas de timeout 10-30s comme sur le run prod)");

    assert_eq!(responses.len(), BURST_SIZE);

    let ok_count = responses
        .iter()
        .filter(|r| r.status() == StatusCode::OK)
        .count();
    let rejected: Vec<_> = responses
        .iter()
        .filter(|r| r.status() == StatusCode::TOO_MANY_REQUESTS)
        .collect();

    for resp in &responses {
        assert_ne!(resp.status().as_u16(), 502, "jamais de 502 sous rafale");
        assert!(
            resp.status() == StatusCode::OK || resp.status() == StatusCode::TOO_MANY_REQUESTS,
            "réponse inattendue sous rafale : {}",
            resp.status()
        );
    }

    assert!(
        ok_count >= 1 && ok_count <= MAX_CONCURRENT,
        "au plus `max_concurrent` requêtes servies en même temps, au moins une : {ok_count}"
    );
    assert_eq!(ok_count + rejected.len(), BURST_SIZE);

    for resp in rejected {
        let retry_after = resp
            .headers()
            .get("Retry-After")
            .expect("chaque refus porte un Retry-After exploitable par le client");
        retry_after
            .to_str()
            .unwrap()
            .parse::<u64>()
            .expect("Retry-After doit être un entier de secondes");
    }
}

/// @security — le limiteur protège une ressource partagée (le pool), pas une
/// identité : il ne doit ni étouffer la sonde de vivacité (sinon la charge
/// applicative se transforme en redémarrage de conteneur, un incident pire
/// que celui qu'il traite), ni se comporter comme un bannissement. Une fois
/// le slot libéré, une rafale légitime repasse immédiatement — desserrer une
/// limite pour la recette n'a jamais été nécessaire, on en ajoute une
/// nouvelle qui s'efface d'elle-même.
#[actix_web::test]
async fn security_limiter_spares_health_and_is_not_a_ban() {
    let limiter = RequestConcurrencyLimit::new(ConcurrencyLimitConfig {
        max_concurrent: 1,
        retry_after_secs: 1,
    });
    let app = test::init_service(
        App::new()
            .wrap(limiter)
            .route("/slow", web::get().to(slow_ok))
            .route("/api/v1/health", web::get().to(health_stub)),
    )
    .await;

    let req1 = test::TestRequest::get().uri("/slow").to_request();
    let req_health = test::TestRequest::get().uri("/api/v1/health").to_request();
    let req2 = test::TestRequest::get().uri("/slow").to_request();

    let (resp1, resp_health, resp2) = tokio::join!(
        test::call_service(&app, req1),
        test::call_service(&app, req_health),
        test::call_service(&app, req2),
    );

    assert_eq!(resp1.status(), StatusCode::OK);
    assert_eq!(
        resp_health.status(),
        StatusCode::OK,
        "la sonde de vivacité ne doit jamais être délestée, même saturé"
    );
    assert_eq!(resp2.status(), StatusCode::TOO_MANY_REQUESTS);

    // Le slot occupé par req1 est libéré une fois sa réponse envoyée
    // (`resp1` est déjà résolu ici) : une requête suivante doit repasser
    // immédiatement, preuve que ce n'est pas un bannissement persistant.
    let req3 = test::TestRequest::get().uri("/slow").to_request();
    let resp3 = test::call_service(&app, req3).await;
    assert_eq!(
        resp3.status(),
        StatusCode::OK,
        "le refus est transitoire, pas un bannissement : la rafale suivante doit repasser"
    );
}
