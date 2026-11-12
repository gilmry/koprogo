//! Un copropriétaire ne voit que sa fiche et ses lots (#1027).
//!
//! La garde d'organisation suffit au syndic et au comptable, qui gèrent tous
//! les copropriétaires de leurs immeubles. Elle ne suffit pas au
//! copropriétaire : il est membre de la même organisation que ses voisins, et
//! elle le laissait lire leurs fiches (emails compris), leurs lots, leurs
//! paiements et leurs dettes.
//!
//! Le décor : un immeuble, deux lots, Alice détient A1, Bertrand détient B1.
//! Alice se connecte ; tout ce qui porte sur Bertrand doit lui être fermé.

mod common;

use actix_web::{http::header, test, App};
use common::{
    create_test_building, register_and_login_returning_user, register_and_login_with_role,
    setup_test_db,
};
use koprogo_api::infrastructure::web::routes::configure_routes;
use serde_json::{json, Value};
use uuid::Uuid;

/// Ce que les tests consultent du décor.
struct Decor {
    syndic: String,
    alice: String,
    alice_id: String,
    bertrand_id: String,
    lot_alice: String,
    lot_bertrand: String,
    immeuble: Uuid,
    /// Compte copropriétaire rattaché à aucune fiche.
    orphelin: String,
}

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

macro_rules! decor {
    ($app:expr, $app_state:expr, $org_id:expr) => {{
        let syndic = register_and_login_with_role(&$app_state, $org_id, "syndic").await;
        let immeuble = create_test_building(&$app_state, $org_id).await;
        let acp_id = common::ensure_default_acp_for_org(&$app_state.pool, $org_id).await;

        let mut lots = Vec::new();
        for numero in ["A1", "B1"] {
            let (statut, texte, lot) = appeler!(
                $app,
                syndic,
                test::TestRequest::post().uri("/api/v1/units").set_json(json!({
                    "acp_id": acp_id.to_string(),
                    "building_id": immeuble.to_string(),
                    "unit_number": numero,
                    "unit_type": "Apartment",
                    "floor": 1,
                    "surface_area": 80.0,
                    "quota": 500.0
                }))
            );
            assert_eq!(statut, 201, "création du lot {numero} : {texte}");
            lots.push(lot["id"].as_str().unwrap().to_string());
        }

        let mut fiches = Vec::new();
        let mut jetons = Vec::new();
        for (prenom, lot) in [("Alice", &lots[0]), ("Bertrand", &lots[1])] {
            let (statut, texte, fiche) = appeler!(
                $app,
                syndic,
                test::TestRequest::post().uri("/api/v1/owners").set_json(json!({
                    "first_name": prenom,
                    "last_name": "Voisin",
                    "email": format!("{}-{}@example.com", prenom.to_lowercase(), Uuid::new_v4()),
                    "phone": "+32123456789",
                    "address": "1 rue du Palier",
                    "city": "Bruxelles",
                    "postal_code": "1000",
                    "country": "Belgium"
                }))
            );
            assert_eq!(statut, 201, "création de {prenom} : {texte}");
            let fiche_id = fiche["id"].as_str().unwrap().to_string();

            let (statut, texte, _) = appeler!(
                $app,
                syndic,
                test::TestRequest::post()
                    .uri(&format!("/api/v1/units/{lot}/owners"))
                    .set_json(json!({
                        "owner_id": fiche_id,
                        "ownership_percentage": 1.0,
                        "is_primary_contact": true
                    }))
            );
            assert_eq!(statut, 201, "rattachement de {prenom} à son lot : {texte}");

            let (jeton, user_id) =
                register_and_login_returning_user(&$app_state, $org_id, "owner").await;
            $app_state
                .owner_use_cases
                .link_user_to_owner(Uuid::parse_str(&fiche_id).unwrap(), Some(user_id))
                .await
                .expect("rattachement du compte à la fiche");
            fiches.push(fiche_id);
            jetons.push(jeton);
        }

        let orphelin = register_and_login_with_role(&$app_state, $org_id, "owner").await;

        Decor {
            syndic,
            alice: jetons[0].clone(),
            alice_id: fiches[0].clone(),
            bertrand_id: fiches[1].clone(),
            lot_alice: lots[0].clone(),
            lot_bertrand: lots[1].clone(),
            immeuble,
            orphelin,
        }
    }};
}

fn ids(page: &Value) -> Vec<String> {
    let liste = page.get("data").unwrap_or(page);
    liste
        .as_array()
        .map(|l| {
            l.iter()
                .filter_map(|e| e["id"].as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default()
}

// ── @happy ───────────────────────────────────────────────────────────────

#[actix_web::test]
async fn happy_un_coproprietaire_voit_sa_fiche_et_ses_lots() {
    let (app_state, _container, org_id) = setup_test_db().await;
    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;
    let d = decor!(app, app_state, org_id);

    let (statut, texte, page) =
        appeler!(app, d.alice, test::TestRequest::get().uri("/api/v1/owners"));
    assert_eq!(statut, 200, "{texte}");
    assert_eq!(ids(&page), vec![d.alice_id.clone()], "{texte}");

    let (statut, texte, page) =
        appeler!(app, d.alice, test::TestRequest::get().uri("/api/v1/units"));
    assert_eq!(statut, 200, "{texte}");
    assert_eq!(ids(&page), vec![d.lot_alice.clone()], "{texte}");

    for chemin in [
        format!("/api/v1/owners/{}", d.alice_id),
        format!("/api/v1/owners/{}/units", d.alice_id),
        format!("/api/v1/units/{}", d.lot_alice),
        format!("/api/v1/units/{}/owners", d.lot_alice),
        format!("/api/v1/owners/{}/payments", d.alice_id),
    ] {
        let (statut, texte, _) = appeler!(app, d.alice, test::TestRequest::get().uri(&chemin));
        assert_eq!(statut, 200, "{chemin} : {texte}");
    }
}

// ── @negative ────────────────────────────────────────────────────────────

#[actix_web::test]
async fn negative_tout_ce_qui_porte_sur_le_voisin_est_ferme() {
    let (app_state, _container, org_id) = setup_test_db().await;
    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;
    let d = decor!(app, app_state, org_id);

    for chemin in [
        format!("/api/v1/owners/{}", d.bertrand_id),
        format!("/api/v1/owners/{}/units", d.bertrand_id),
        format!("/api/v1/owners/{}/units/history", d.bertrand_id),
        format!("/api/v1/owners/{}/payments", d.bertrand_id),
        format!("/api/v1/owners/{}/payment-methods", d.bertrand_id),
        format!("/api/v1/owners/{}/distributions", d.bertrand_id),
        format!("/api/v1/owners/{}/total-due", d.bertrand_id),
        format!("/api/v1/owners/{}/payment-reminders", d.bertrand_id),
        format!("/api/v1/owner-contributions?owner_id={}", d.bertrand_id),
        format!("/api/v1/units/{}", d.lot_bertrand),
        format!("/api/v1/units/{}/owners", d.lot_bertrand),
        format!("/api/v1/units/{}/owners/history", d.lot_bertrand),
    ] {
        let (statut, texte, corps) = appeler!(app, d.alice, test::TestRequest::get().uri(&chemin));
        let ferme = matches!(statut, 403 | 404)
            || (statut == 200 && corps.as_array().is_some_and(|l| l.is_empty()));
        assert!(ferme, "{chemin} ouvert à la voisine : {statut} {texte}");
        assert!(
            !texte.contains("Bertrand"),
            "{chemin} laisse passer le nom du voisin : {texte}"
        );
    }
}

// ── @edge ────────────────────────────────────────────────────────────────

#[actix_web::test]
async fn edge_un_compte_sans_fiche_ne_voit_personne() {
    let (app_state, _container, org_id) = setup_test_db().await;
    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;
    let d = decor!(app, app_state, org_id);

    let (statut, texte, page) = appeler!(
        app,
        d.orphelin,
        test::TestRequest::get().uri("/api/v1/owners")
    );
    assert_eq!(statut, 200, "{texte}");
    assert!(ids(&page).is_empty(), "{texte}");

    let (statut, texte, page) = appeler!(
        app,
        d.orphelin,
        test::TestRequest::get().uri("/api/v1/units")
    );
    assert_eq!(statut, 200, "{texte}");
    assert!(ids(&page).is_empty(), "{texte}");
}

#[actix_web::test]
async fn edge_dans_l_immeuble_seul_son_lot_apparait() {
    let (app_state, _container, org_id) = setup_test_db().await;
    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;
    let d = decor!(app, app_state, org_id);

    let chemin = format!("/api/v1/buildings/{}/units", d.immeuble);
    let (statut, texte, lots) = appeler!(app, d.alice, test::TestRequest::get().uri(&chemin));
    assert_eq!(statut, 200, "{texte}");
    assert_eq!(ids(&lots), vec![d.lot_alice.clone()], "{texte}");
}

// ── @security ────────────────────────────────────────────────────────────

/// Le syndic garde sa vue complète : la restriction ne vaut que pour le rôle
/// copropriétaire.
#[actix_web::test]
async fn security_le_syndic_voit_toujours_tous_les_coproprietaires() {
    let (app_state, _container, org_id) = setup_test_db().await;
    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;
    let d = decor!(app, app_state, org_id);

    let (statut, texte, page) = appeler!(
        app,
        d.syndic,
        test::TestRequest::get().uri("/api/v1/owners")
    );
    assert_eq!(statut, 200, "{texte}");
    let vus = ids(&page);
    assert!(
        vus.contains(&d.alice_id) && vus.contains(&d.bertrand_id),
        "{texte}"
    );

    let chemin = format!("/api/v1/buildings/{}/units", d.immeuble);
    let (statut, texte, lots) = appeler!(app, d.syndic, test::TestRequest::get().uri(&chemin));
    assert_eq!(statut, 200, "{texte}");
    assert_eq!(ids(&lots).len(), 2, "{texte}");
}
