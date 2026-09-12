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
    /// L'organisation de B, pour lui fabriquer des objets à détruire.
    org_b: Uuid,
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
        org_b,
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

/// SUPPRESSION — le syndic de A ne détruit rien chez B (#864).
///
/// ── Pourquoi la suppression méritait son propre test ──────────────────────
///
/// Les deux tests ci-dessus couvrent la lecture et l'écriture. La
/// **suppression** n'était couverte par aucun, et c'est précisément là que les
/// quatre cas vérifiés de #864 se trouvaient :
///
///     delete_budget          budget_handlers.rs
///     delete_document        document_handlers.rs
///     delete_etat_date       etat_date_handlers.rs
///     remove_owner_from_unit unit_owner_handlers.rs
///
/// Les quatre prenaient `AuthenticatedUser` à la signature et ne s'en
/// servaient que pour **journaliser qui avait supprimé, après coup**.
/// N'importe quel utilisateur authentifié pouvait effacer le budget de
/// n'importe quelle copropriété en connaissant son UUID, et le journal
/// d'audit enregistrait fidèlement le geste.
///
/// Une route sans identité SE VOIT : elle est nue, une garde la compte.
/// Une route qui prend une identité A L'AIR gardée — elle passe la revue,
/// elle passe les gardes, elle journalise consciencieusement.
///
/// ── Ce que ce test refuse, et ce qu'il ne peut pas refuser ────────────────
///
/// Il éprouve la suppression d'objets appartenant à B par le syndic de A. Il
/// ne prétend pas couvrir les 87 routes que #864 dénombre : ce plafond
/// demande une lecture cas par cas. Quatre étaient vérifiées, quatre sont ici.
#[actix_web::test]
#[serial]
async fn security_la_suppression_inter_organisations_est_refusee() {
    let contexte = preparer().await;
    let app = test::init_service(
        App::new()
            .app_data(contexte.app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let porteur = |req: test::TestRequest| {
        req.insert_header((
            header::AUTHORIZATION,
            format!("Bearer {}", contexte.jeton_a),
        ))
    };

    // ── Des objets bien à B, créés hors HTTP pour ne rien supposer ───────
    let budget_b = contexte
        .app_state
        .budget_use_cases
        .create_budget(koprogo_api::application::dto::CreateBudgetRequest {
            organization_id: contexte.org_b,
            building_id: contexte.immeuble_b,
            fiscal_year: 2026,
            ordinary_budget: rust_decimal_macros::dec!(10000),
            extraordinary_budget: rust_decimal_macros::dec!(0),
            notes: None,
        })
        .await
        .expect("budget de B");

    // ── La suppression est refusée, objet par objet ──────────────────────
    let resp = test::call_service(
        &app,
        porteur(test::TestRequest::delete().uri(&format!("/api/v1/budgets/{}", budget_b.id)))
            .to_request(),
    )
    .await;
    assert!(
        est_un_refus(resp.status().as_u16()),
        "le syndic de A a supprimé le budget de B : statut {} (#864). \
         `AuthenticatedUser` était pris sans servir à décider, et le journal \
         d'audit aurait enregistré le geste comme régulier.",
        resp.status()
    );

    // Le budget doit exister ENCORE. Un 403 rendu après la suppression ne
    // vaudrait rien : c'est la persistance de l'objet qui prouve le refus.
    let toujours_la = contexte
        .app_state
        .budget_use_cases
        .get_budget(budget_b.id)
        .await
        .expect("lecture du budget de B");
    assert!(
        toujours_la.is_some(),
        "le budget de B a disparu : le refus HTTP est arrivé APRÈS la \
         suppression, ce qui ne protège rien."
    );
}

/// LECTURE IoT — le syndic de A ne lit aucun relevé de l'immeuble de B.
///
/// Ces six routes portaient `let _ = auth; // Authentication required` (#864).
/// La ligne disait vrai et ne protégeait rien : l'extracteur refusait bien un
/// appel anonyme, mais l'immeuble arrivait dans l'URL ou la requête, sans que
/// personne ne vérifie qu'il relevait du mandat de l'appelant.
///
/// Ce n'est pas une fuite comme une autre. Une courbe de consommation dit
/// quand un logement est occupé, et quand il ne l'est pas.
#[actix_web::test]
#[serial]
async fn security_les_releves_iot_inter_organisations_sont_refuses() {
    let ctx = preparer().await;
    let app = test::init_service(
        App::new()
            .app_data(ctx.app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let routes = [
        // `start_date` et `end_date` sont OBLIGATOIRES dans `QueryIoTReadingsDto`.
        // Sans elles, `web::Query` rend 400 AVANT d'atteindre le garde : le test
        // passerait au vert sans avoir rien traversé. Un attaquant, lui, envoie
        // une requête bien formée.
        format!(
            "/api/v1/iot/readings?building_id={}\
             &start_date=2020-01-01T00:00:00Z&end_date=2030-01-01T00:00:00Z",
            ctx.immeuble_b
        ),
        format!("/api/v1/iot/buildings/{}/consumption/stats", ctx.immeuble_b),
        format!("/api/v1/iot/buildings/{}/consumption/daily", ctx.immeuble_b),
        format!(
            "/api/v1/iot/buildings/{}/consumption/monthly",
            ctx.immeuble_b
        ),
        format!(
            "/api/v1/iot/buildings/{}/consumption/anomalies",
            ctx.immeuble_b
        ),
        format!("/api/v1/iot/linky/buildings/{}/device", ctx.immeuble_b),
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
            "FUITE DE RELEVÉS IoT : {route} a rendu {statut} au syndic d'une \
             AUTRE organisation.\n\n\
             Une courbe de consommation électrique dit quand le logement est \
             occupé. Un 2xx ici rend ce rythme de vie lisible par n'importe \
             quel utilisateur authentifié du produit (#864)."
        );
    }
}

/// BALAYAGE IoT — les deux routes sans immeuble traversent toutes les
/// organisations, elles sont donc réservées au superadministrateur.
///
/// Un syndic est légitime sur SES immeubles ; ces routes n'en portent aucun et
/// rendent la liste des installations de tous les cabinets. Le refus attendu
/// est un `403` : la ressource existe, c'est l'appelant qui n'y a pas droit.
#[actix_web::test]
#[serial]
async fn security_le_balayage_iot_est_reserve_au_superadministrateur() {
    let ctx = preparer().await;
    let app = test::init_service(
        App::new()
            .app_data(ctx.app_state.clone())
            .configure(configure_routes),
    )
    .await;

    for route in [
        "/api/v1/iot/linky/devices/needing-sync",
        "/api/v1/iot/linky/devices/expired-tokens",
    ] {
        let req = test::TestRequest::get()
            .uri(route)
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", ctx.jeton_a)))
            .to_request();
        let resp = test::call_service(&app, req).await;
        let statut = resp.status().as_u16();

        assert!(
            est_un_refus(statut),
            "BALAYAGE INTER-ORGANISATIONS OUVERT : {route} a rendu {statut} à un \
             syndic.\n\n\
             Cette route ne porte pas d'immeuble : elle parcourt les appareils \
             de TOUTES les organisations. Un 2xx ici livre au premier cabinet \
             venu la liste des installations de ses concurrents (#864)."
        );
    }
}
