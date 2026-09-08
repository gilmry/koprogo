//! Le cloisonnement inter-organisations, vérifié de bout en bout.
//!
//! ── Ce que ce fichier prouve, et que rien ne prouvait ──────────────────────
//!
//! L'issue #772 pose trois critères de fin. Deux étaient remplis :
//!
//! - le cliquet `garde_lecture` existe et ne peut que baisser ;
//! - la dette est passée de 109 routes imbriquées sans identité à 3.
//!
//! Le troisième ne l'était pas : « un test end-to-end couvre au moins une
//! route par module concerné, **en lecture et en écriture** ». L'issue insiste
//! sur ce point dans sa dernière section :
//!
//! > L'**écriture** inter-organisations. Le testeur ne pouvait pas la vérifier
//! > depuis un navigateur sans forger de requêtes. La lecture étant ouverte,
//! > rien ne permet de supposer que l'écriture est fermée.
//!
//! Le cliquet compte des routes ; il ne prouve pas qu'un refus arrive. Un
//! garde peut être appelé et se tromper de comparaison, ou porter sur la
//! mauvaise ressource — c'est le défaut que l'issue redoute le plus, « un
//! garde faux, ce qui est pire qu'absent ».
//!
//! Ces tests montent l'application, créent DEUX organisations réelles, et
//! demandent depuis l'une des ressources de l'autre.
//!
//! ── Pourquoi 403 ou 404, et non 403 seulement ─────────────────────────────
//!
//! Certains gardes remontent la chaîne avant de comparer : ils ne trouvent pas
//! la ressource dans le périmètre de l'appelant et rendent `404`. D'autres la
//! trouvent puis refusent, et rendent `403`.
//!
//! Les deux sont des refus corrects, et distinguer n'apporterait rien ici :
//! ce qui compte est qu'aucune donnée de l'organisation B ne parte vers
//! l'organisation A. Exiger un code précis figerait un détail
//! d'implémentation et rendrait ce test fragile à la première refonte d'un
//! garde — la faute que trois tests de libellés ont commise le 2026-09-07.

mod common;

use actix_web::http::header;
use actix_web::{test, App};
use koprogo_api::infrastructure::web::configure_routes;
use koprogo_api::infrastructure::web::AppState;
use serial_test::serial;
use uuid::Uuid;

/// Deux organisations, chacune avec son syndic et son immeuble.
///
/// Le second `org_id` est créé dans la même base : c'est la situation réelle
/// d'un hébergement mutualisé, et la seule où le cloisonnement se joue
/// vraiment. Deux bases séparées ne prouveraient rien.
struct DeuxOrganisations {
    app_state: actix_web::web::Data<AppState>,
    _container: Option<testcontainers::ContainerAsync<testcontainers_modules::postgres::Postgres>>,
    jeton_a: String,
    immeuble_b: Uuid,
}

async fn preparer() -> DeuxOrganisations {
    let (app_state, container, org_a) = common::setup_test_db().await;
    let org_b = common::create_test_organization(&app_state).await;

    let jeton_a = common::register_and_login_with_role(&app_state, org_a, "syndic").await;
    let immeuble_b = common::create_test_building(&app_state, org_b).await;

    DeuxOrganisations {
        app_state,
        _container: container,
        jeton_a,
        immeuble_b,
    }
}

fn est_un_refus(statut: u16) -> bool {
    statut == 401 || statut == 403 || statut == 404
}

/// LECTURE — le syndic de A ne lit aucune sous-collection de l'immeuble de B.
///
/// Ces cinq routes sont celles que la recette 3 avait trouvées ouvertes, ou
/// leurs voisines immédiates : elles exposent qui détient quoi, qui doit
/// combien, et ce que les copropriétaires ont décidé.
#[actix_web::test]
#[serial]
async fn la_lecture_inter_organisations_est_refusee() {
    let ctx = preparer().await;
    let app = test::init_service(
        App::new()
            .app_data(ctx.app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let routes = [
        format!("/api/v1/buildings/{}/units", ctx.immeuble_b),
        format!("/api/v1/buildings/{}/exchanges", ctx.immeuble_b),
        format!("/api/v1/buildings/{}/polls/active", ctx.immeuble_b),
        format!("/api/v1/buildings/{}/quotes", ctx.immeuble_b),
        format!("/api/v1/buildings/{}/resource-bookings", ctx.immeuble_b),
    ];

    for route in routes {
        let req = test::TestRequest::get()
            .uri(&route)
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", ctx.jeton_a)))
            .to_request();
        let resp = test::call_service(&app, req).await;
        let statut = resp.status().as_u16();

        assert!(
            est_un_refus(statut),
            "FUITE INTER-ORGANISATIONS : {route} a rendu {statut} au syndic d'une \
             AUTRE organisation.\n\n\
             Cette route expose une sous-collection d'un immeuble qui ne relève \
             pas du mandat de l'appelant. Un 2xx ici signifie que des données \
             d'une copropriété partent vers une autre (#772)."
        );
    }
}

/// ÉCRITURE — le syndic de A ne crée rien dans le périmètre de B.
///
/// C'est le volet que l'issue signale comme « non testé, à couvrir avant toute
/// mise en production réelle ». Un refus en lecture ne dit rien de l'écriture :
/// ce sont des gestionnaires différents, et rien ne garantit qu'ils partagent
/// leurs gardes.
#[actix_web::test]
#[serial]
async fn lecriture_inter_organisations_est_refusee() {
    let ctx = preparer().await;
    let app = test::init_service(
        App::new()
            .app_data(ctx.app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let ecritures = [
        (
            format!("/api/v1/buildings/{}/units", ctx.immeuble_b),
            serde_json::json!({
                "building_id": ctx.immeuble_b.to_string(),
                "unit_number": "A1",
                "floor": 1,
                "quota": 100,
                "unit_type": "Apartment"
            }),
        ),
        (
            format!("/api/v1/buildings/{}/polls", ctx.immeuble_b),
            serde_json::json!({
                "building_id": ctx.immeuble_b.to_string(),
                "title": "Sondage intrus",
                "description": "Ne doit jamais être créé",
                "poll_type": "SingleChoice",
                "options": ["oui", "non"]
            }),
        ),
    ];

    for (route, corps) in ecritures {
        let req = test::TestRequest::post()
            .uri(&route)
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", ctx.jeton_a)))
            .set_json(&corps)
            .to_request();
        let resp = test::call_service(&app, req).await;
        let statut = resp.status().as_u16();

        assert!(
            !(200..300).contains(&statut),
            "ÉCRITURE INTER-ORGANISATIONS ACCEPTÉE : {route} a rendu {statut} au \
             syndic d'une AUTRE organisation.\n\n\
             Une ressource vient d'être créée dans le patrimoine d'une \
             copropriété qui n'est pas celle de l'appelant. C'est le volet que \
             #772 signale comme non couvert : « la lecture étant ouverte, rien \
             ne permet de supposer que l'écriture est fermée »."
        );
    }
}

/// Sans ce contrôle, les deux tests ci-dessus passeraient aussi bien avec une
/// application qui refuse TOUT — un jeton invalide, une route inexistante, un
/// serveur en panne.
///
/// Il vérifie que le même syndic, sur SON immeuble, obtient une réponse
/// servie. C'est le cas négatif du cas négatif, et il est indispensable : les
/// six gardes trouvées inopérantes le 2026-09-07 avaient toutes en commun de
/// n'avoir jamais été éprouvées dans les deux sens.
#[actix_web::test]
#[serial]
async fn le_meme_syndic_lit_bien_son_propre_immeuble() {
    let (app_state, _container, org_a) = common::setup_test_db().await;
    let jeton_a = common::register_and_login_with_role(&app_state, org_a, "syndic").await;
    let immeuble_a = common::create_test_building(&app_state, org_a).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/buildings/{}/units", immeuble_a))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", jeton_a)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    let statut = resp.status().as_u16();

    assert!(
        (200..300).contains(&statut),
        "Le syndic ne peut plus lire SON PROPRE immeuble : {statut}.\n\n\
         Les gardes de #772 sont trop stricts, et les deux tests de refus de \
         ce fichier ne prouvent alors plus rien — une application qui refuse \
         tout les satisfait aussi."
    );
}
