//! Caractérisation du rattachement d'une ACP à un cabinet (lot 1, story S-1).
//!
//! ── Ce que ce fichier fige ────────────────────────────────────────────────
//!
//! Le comportement ACTUEL, pas le comportement voulu. Les lots 2 à 4 vont
//! remplacer `acps.organization_id` par un mandat daté et les gardes
//! `verify_*_org_access` par un contrôle du mandat actif (ADR 0053, 0055,
//! 0058). Avant d'y toucher, ce filet dit ce que fait le code aujourd'hui :
//!
//! - `PUT /acps/{id}` avec `organization_id` écrase le rattachement, sans
//!   trace de l'ancien cabinet (`Acp::set_organization`) ;
//! - l'accès suit le rattachement à l'instant même : l'ancien cabinet perd
//!   tout, le nouveau gagne tout, y compris le passé ;
//! - un Admin de cabinet ne peut pas déplacer l'ACP vers un autre cabinet,
//!   mais peut la détacher ;
//! - une ACP détachée (auto-gérée) n'est lisible que du SuperAdmin ;
//! - supprimer un cabinet détache ses ACP (`ON DELETE SET NULL`).
//!
//! ── La règle ──────────────────────────────────────────────────────────────
//!
//! Ces tests sont verts sur le code au moment de leur écriture. On ne les
//! modifie jamais pour les faire passer. Quand un lot change délibérément un
//! de ces comportements, la PR de ce lot retire ou réécrit le test concerné
//! et le dit dans sa description. Les comportements douteux sont notés
//! « bug ou voulu ? » dans `docs/maury/transfert-acp-filet/stories.md`
//! (registre S-1) : le test les fige tels quels, la décision revient au PO.
//!
//! Les codes HTTP sont exacts (403, pas « un refus ») : c'est précisément ce
//! qu'une caractérisation doit retenir, pour qu'un changement se voie.

mod common;

use actix_web::{http::header, test, App};
use koprogo_api::infrastructure::web::configure_routes;
use serde_json::{json, Value};
use serial_test::serial;
use uuid::Uuid;

macro_rules! appeler {
    ($app:expr, $jeton:expr, $req:expr) => {{
        let resp = test::call_service(
            &$app,
            $req.insert_header((header::AUTHORIZATION, format!("Bearer {}", $jeton)))
                .to_request(),
        )
        .await;
        let statut = resp.status().as_u16();
        let brut = test::read_body(resp).await;
        let texte = String::from_utf8_lossy(&brut).to_string();
        let json: Value = serde_json::from_str(&texte).unwrap_or(Value::Null);
        (statut, texte, json)
    }};
}

/// Le corps minimal d'un `PUT /acps/{id}` : les champs d'identité sont
/// obligatoires, `organization_id` est ajouté par l'appelant.
fn corps_put(organization_id: Option<Value>) -> Value {
    let mut corps = json!({
        "name": "Résidence Les Tilleuls",
        "address_street": "Rue des Tilleuls 1",
        "address_postal_code": "1000",
        "address_city": "Bruxelles"
    });
    if let Some(org) = organization_id {
        corps["organization_id"] = org;
    }
    corps
}

async fn rattachement_en_base(pool: &sqlx::PgPool, acp_id: Uuid) -> Option<Uuid> {
    let (org,): (Option<Uuid>,) = sqlx::query_as("SELECT organization_id FROM acps WHERE id = $1")
        .bind(acp_id)
        .fetch_one(pool)
        .await
        .expect("lecture de acps.organization_id");
    org
}

async fn acp_de_l_immeuble(
    app_state: &actix_web::web::Data<koprogo_api::infrastructure::web::AppState>,
    immeuble: Uuid,
) -> Uuid {
    let building = app_state
        .building_use_cases
        .get_building(immeuble)
        .await
        .expect("lecture de l'immeuble")
        .expect("immeuble introuvable");
    Uuid::parse_str(&building.acp_id).expect("acp_id illisible")
}

/// C1 et C2 — Le SuperAdmin transfère l'ACP de A vers B : le rattachement
/// est écrasé, l'accès bascule entièrement et à l'instant.
#[actix_web::test]
#[serial]
async fn happy_le_transfert_ecrase_le_rattachement_et_bascule_tout_l_acces() {
    let (app_state, _container, org_a) = common::setup_test_db().await;
    let org_b = common::create_test_organization(&app_state).await;
    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let superadmin = common::register_and_login(&app_state, org_a).await;
    let syndic_a = common::register_and_login_with_role(&app_state, org_a, "syndic").await;
    let syndic_b = common::register_and_login_with_role(&app_state, org_b, "syndic").await;
    let immeuble = common::create_test_building(&app_state, org_a).await;
    let acp = acp_de_l_immeuble(&app_state, immeuble).await;

    // Avant : A voit, B ne voit pas.
    for (jeton, attendu, qui) in [(&syndic_a, 200, "A"), (&syndic_b, 403, "B")] {
        let (statut, texte, _) = appeler!(
            app,
            jeton,
            test::TestRequest::get().uri(&format!("/api/v1/acps/{acp}"))
        );
        assert_eq!(
            statut, attendu,
            "avant transfert, ACP vue par {qui} : {texte}"
        );
        let (statut, texte, _) = appeler!(
            app,
            jeton,
            test::TestRequest::get().uri(&format!("/api/v1/buildings/{immeuble}"))
        );
        assert_eq!(
            statut, attendu,
            "avant transfert, immeuble vu par {qui} : {texte}"
        );
    }

    let (statut, texte, reponse) = appeler!(
        app,
        superadmin,
        test::TestRequest::put()
            .uri(&format!("/api/v1/acps/{acp}"))
            .set_json(corps_put(Some(json!(org_b.to_string()))))
    );
    assert_eq!(statut, 200, "transfert par le SuperAdmin : {texte}");
    assert_eq!(reponse["organization_id"], json!(org_b.to_string()));
    assert_eq!(
        rattachement_en_base(&app_state.pool, acp).await,
        Some(org_b)
    );

    // Aucune trace de A : il n'existe pas de table de mandats.
    let (table,): (Option<String>,) =
        sqlx::query_as("SELECT to_regclass('public.syndic_mandates')::text")
            .fetch_one(&app_state.pool)
            .await
            .expect("lecture du catalogue");
    assert_eq!(table, None, "une table syndic_mandates existe désormais");

    // Après : A ne voit plus rien, B voit tout, y compris ce qui précède.
    for (jeton, attendu, qui) in [(&syndic_a, 403, "A"), (&syndic_b, 200, "B")] {
        let (statut, texte, _) = appeler!(
            app,
            jeton,
            test::TestRequest::get().uri(&format!("/api/v1/acps/{acp}"))
        );
        assert_eq!(
            statut, attendu,
            "après transfert, ACP vue par {qui} : {texte}"
        );
        let (statut, texte, _) = appeler!(
            app,
            jeton,
            test::TestRequest::get().uri(&format!("/api/v1/buildings/{immeuble}"))
        );
        assert_eq!(
            statut, attendu,
            "après transfert, immeuble vu par {qui} : {texte}"
        );
    }
}

/// C3 — Le syndic du cabinet ne touche pas au rattachement : ni déplacement,
/// ni détachement. Seul le SuperAdmin modifie une ACP.
///
/// Le code connaît un appelant `Admin` de cabinet (`AcpCaller::Admin`), qui
/// pourrait détacher l'ACP mais pas la déplacer. Il est inatteignable : aucun
/// compte ne peut porter le rôle `admin`, que `UserRole` refuse. Registre S-1,
/// anomalie R1.
#[actix_web::test]
#[serial]
async fn security_seul_le_superadmin_modifie_le_rattachement() {
    let (app_state, _container, org_a) = common::setup_test_db().await;
    let org_b = common::create_test_organization(&app_state).await;
    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let syndic_a = common::register_and_login_with_role(&app_state, org_a, "syndic").await;
    let immeuble = common::create_test_building(&app_state, org_a).await;
    let acp = acp_de_l_immeuble(&app_state, immeuble).await;

    for (cible, quoi) in [
        (json!(org_b.to_string()), "déplacement vers B"),
        (Value::Null, "détachement"),
    ] {
        let (statut, texte, _) = appeler!(
            app,
            syndic_a,
            test::TestRequest::put()
                .uri(&format!("/api/v1/acps/{acp}"))
                .set_json(corps_put(Some(cible)))
        );
        assert_eq!(statut, 403, "{quoi} par le syndic de A : {texte}");
        assert_eq!(
            rattachement_en_base(&app_state.pool, acp).await,
            Some(org_a)
        );
    }
}

/// C4 — `organization_id: null` ne détache PAS l'ACP : 200 et rattachement
/// inchangé. Le DTO documente l'inverse, mais serde lit `null` comme une clé
/// absente sur un `Option<Option<_>>`. Registre S-1, anomalie R4.
///
/// C5 — Une chaîne vide détache. L'ACP détachée et son immeuble ne sont plus
/// lisibles que du SuperAdmin, son ancien cabinet compris.
#[actix_web::test]
#[serial]
async fn edge_null_ne_detache_pas_et_l_acp_detachee_n_est_lisible_que_du_superadmin() {
    let (app_state, _container, org_a) = common::setup_test_db().await;
    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let superadmin = common::register_and_login(&app_state, org_a).await;
    let syndic_a = common::register_and_login_with_role(&app_state, org_a, "syndic").await;
    let immeuble = common::create_test_building(&app_state, org_a).await;
    let acp = acp_de_l_immeuble(&app_state, immeuble).await;

    let (statut, texte, reponse) = appeler!(
        app,
        superadmin,
        test::TestRequest::put()
            .uri(&format!("/api/v1/acps/{acp}"))
            .set_json(corps_put(Some(Value::Null)))
    );
    assert_eq!(statut, 200, "null envoyé par le SuperAdmin : {texte}");
    assert_eq!(reponse["organization_id"], json!(org_a.to_string()));
    assert_eq!(
        rattachement_en_base(&app_state.pool, acp).await,
        Some(org_a)
    );

    let (statut, texte, reponse) = appeler!(
        app,
        superadmin,
        test::TestRequest::put()
            .uri(&format!("/api/v1/acps/{acp}"))
            .set_json(corps_put(Some(json!(""))))
    );
    assert_eq!(
        statut, 200,
        "chaîne vide envoyée par le SuperAdmin : {texte}"
    );
    assert_eq!(reponse["organization_id"], Value::Null);
    assert_eq!(rattachement_en_base(&app_state.pool, acp).await, None);

    for (jeton, attendu, qui) in [
        (&syndic_a, 403, "le syndic de A"),
        (&superadmin, 200, "le SuperAdmin"),
    ] {
        let (statut, texte, _) = appeler!(
            app,
            jeton,
            test::TestRequest::get().uri(&format!("/api/v1/acps/{acp}"))
        );
        assert_eq!(statut, attendu, "ACP détachée vue par {qui} : {texte}");
        let (statut, texte, _) = appeler!(
            app,
            jeton,
            test::TestRequest::get().uri(&format!("/api/v1/buildings/{immeuble}"))
        );
        assert_eq!(
            statut, attendu,
            "immeuble de l'ACP détachée vu par {qui} : {texte}"
        );
    }
}

/// C6 — Rattacher à un cabinet inexistant est refusé en 400, sans écriture.
#[actix_web::test]
#[serial]
async fn negative_rattacher_a_un_cabinet_inexistant_est_refuse() {
    let (app_state, _container, org_a) = common::setup_test_db().await;
    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let superadmin = common::register_and_login(&app_state, org_a).await;
    let immeuble = common::create_test_building(&app_state, org_a).await;
    let acp = acp_de_l_immeuble(&app_state, immeuble).await;

    let (statut, texte, _) = appeler!(
        app,
        superadmin,
        test::TestRequest::put()
            .uri(&format!("/api/v1/acps/{acp}"))
            .set_json(corps_put(Some(json!(Uuid::new_v4().to_string()))))
    );
    assert_eq!(statut, 400, "cabinet inexistant : {texte}");
    assert_eq!(
        rattachement_en_base(&app_state.pool, acp).await,
        Some(org_a)
    );
}

/// C7 — Supprimer un cabinet détache ses ACP sans les supprimer, et leurs
/// immeubles restent en place (`acps.organization_id ON DELETE SET NULL`).
///
/// Registre S-1, anomalie R2 : ce test ne regarde que `acps` et
/// `buildings`. Ce que la suppression emporte ailleurs (tables portant
/// encore un `organization_id` en cascade) n'est pas caractérisé ici.
#[actix_web::test]
#[serial]
async fn edge_supprimer_un_cabinet_detache_ses_acp_sans_les_supprimer() {
    let (app_state, _container, _org_a) = common::setup_test_db().await;
    let org_c = common::create_test_organization(&app_state).await;
    let immeuble = common::create_test_building(&app_state, org_c).await;
    let acp = acp_de_l_immeuble(&app_state, immeuble).await;

    sqlx::query("DELETE FROM organizations WHERE id = $1")
        .bind(org_c)
        .execute(&app_state.pool)
        .await
        .expect("suppression du cabinet");

    assert_eq!(rattachement_en_base(&app_state.pool, acp).await, None);
    let (immeubles,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM buildings WHERE acp_id = $1")
        .bind(acp)
        .fetch_one(&app_state.pool)
        .await
        .expect("comptage des immeubles");
    assert_eq!(
        immeubles, 1,
        "l'immeuble de l'ACP a disparu avec le cabinet"
    );
}
