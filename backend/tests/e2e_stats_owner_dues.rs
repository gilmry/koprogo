// E2E tests for GET /stats/owner/dues-by-acp (Issue #867)
//
// ── Ce que cette route corrige ───────────────────────────────────────────
//
// Un copropriétaire peut détenir des lots dans plusieurs ACP — l'ordinaire
// d'un investisseur. Chaque ACP est une personne morale distincte, avec son
// propre compte bancaire (Art. 3.86 § 1er et § 3). Un montant unique invite
// à un virement groupé qui paierait la mauvaise personne morale pour une
// partie de la somme.
//
// `stats_use_cases.rs` couvre déjà la logique de passage (owner introuvable
// → liste vide) avec un dépôt simulé. Ce fichier couvre ce qu'un simulacre
// ne peut pas garder honnête : la requête SQL elle-même — le groupement par
// ACP, le filtrage par copropriétaire, et le comportement HTTP réel.

mod common;

use actix_web::http::header;
use actix_web::{test, App};
use chrono::Utc;
use koprogo_api::application::dto::*;
use koprogo_api::application::use_cases::acp_use_cases::AcpCaller;
use koprogo_api::domain::entities::{ExpenseCategory, UnitType};
use koprogo_api::infrastructure::web::{configure_routes, AppState};
use serial_test::serial;
use uuid::Uuid;

/// Crée un copropriétaire — utilisateur + fiche `owners` liée — et rend
/// `(owner_id, token)`.
async fn creer_coproprietaire(
    app_state: &actix_web::web::Data<AppState>,
    org_id: Uuid,
    prenom: &str,
) -> (Uuid, String) {
    let email = format!("{}-{}@test.com", prenom.to_lowercase(), Uuid::new_v4());
    let reg = RegisterRequest {
        email: email.clone(),
        password: "Passw0rd!".to_string(),
        first_name: prenom.to_string(),
        last_name: "Investisseur".to_string(),
        role: "owner".to_string(),
        organization_id: Some(org_id),
    };
    let login = app_state
        .auth_use_cases
        .register(reg)
        .await
        .expect("register owner");

    let owner_dto = CreateOwnerDto {
        organization_id: org_id.to_string(),
        first_name: prenom.to_string(),
        last_name: "Investisseur".to_string(),
        email,
        phone: None,
        address: "Rue du Test 1".to_string(),
        city: "Bruxelles".to_string(),
        postal_code: "1000".to_string(),
        country: "Belgium".to_string(),
        user_id: Some(login.user.id.to_string()),
    };
    let owner = app_state
        .owner_use_cases
        .create_owner(owner_dto)
        .await
        .expect("create owner");

    (
        Uuid::parse_str(&owner.id).expect("owner id illisible"),
        login.token,
    )
}

/// Crée une ACP, un immeuble qui lui est rattaché, un lot, rattache
/// `owner_id` à ce lot à 100 %, puis une charge en attente du `montant`
/// donné sur cet immeuble. Rend l'identifiant de l'ACP tel que la réponse
/// HTTP le sert (String — c'est ainsi que `DuAupresDuneAcp.acp_id` le
/// sérialise).
async fn creer_dette_pending(
    app_state: &actix_web::web::Data<AppState>,
    org_id: Uuid,
    owner_id: Uuid,
    nom_acp: &str,
    montant: rust_decimal::Decimal,
) -> String {
    let acp_dto = CreateAcpDto {
        organization_id: Some(org_id.to_string()),
        name: nom_acp.to_string(),
        address_street: "Rue du Test 1".to_string(),
        address_postal_code: "1000".to_string(),
        address_city: "Bruxelles".to_string(),
        bce_number: Some(format!("0{}", &Uuid::new_v4().simple().to_string()[..9])),
        total_tantiemes: None,
    };
    let acp = app_state
        .acp_use_cases
        .create_acp(&AcpCaller::SuperAdmin, acp_dto)
        .await
        .expect("create acp");

    let building_dto = CreateBuildingDto {
        acp_id: acp.id.clone(),
        name: format!("{nom_acp} - immeuble"),
        address: "Rue du Test 1".to_string(),
        city: "Bruxelles".to_string(),
        postal_code: "1000".to_string(),
        country: "Belgium".to_string(),
        total_units: 1,
        total_tantiemes: Some(1000),
        construction_year: Some(2010),
    };
    let building = app_state
        .building_use_cases
        .create_building(building_dto)
        .await
        .expect("create building");
    let building_id = Uuid::parse_str(&building.id).expect("building id illisible");

    let unit_dto = CreateUnitDto {
        acp_id: Some(acp.id.clone()),
        building_id: building_id.to_string(),
        unit_number: "1".to_string(),
        unit_type: UnitType::Apartment,
        floor: Some(0),
        surface_area: 80.0,
        quota: rust_decimal_macros::dec!(1.0),
    };
    let unit = app_state
        .unit_use_cases
        .create_unit(unit_dto)
        .await
        .expect("create unit");
    let unit_id = Uuid::parse_str(&unit.id).expect("unit id illisible");

    app_state
        .unit_owner_use_cases
        .add_owner_to_unit(unit_id, owner_id, rust_decimal_macros::dec!(1.0), true)
        .await
        .expect("add owner to unit");

    let expense_dto = CreateExpenseDto {
        organization_id: org_id.to_string(),
        building_id: building_id.to_string(),
        category: ExpenseCategory::Maintenance,
        amount: montant,
        description: format!("Charge {nom_acp}"),
        expense_date: Utc::now().to_rfc3339(),
        supplier: Some("Fournisseur Test".to_string()),
        invoice_number: None,
        account_code: None,
        amount_excl_vat: None,
        vat_rate: None,
        due_date: None,
        line_items: None,
    };
    app_state
        .expense_use_cases
        .create_expense(expense_dto)
        .await
        .expect("create expense");

    acp.id
}

/// @happy — deux ACP, deux montants, jamais un total agrégé.
///
/// C'est le cœur de #867 : la réponse ne doit pas rendre un montant unique
/// qu'un copropriétaire pourrait régler d'un seul virement — juridiquement
/// faux puisque chaque ACP est une personne morale distincte avec son propre
/// compte bancaire.
#[actix_web::test]
#[serial]
async fn happy_deux_acp_rendent_deux_lignes_avec_leur_propre_montant() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (owner_id, token) = creer_coproprietaire(&app_state, org_id, "Colette").await;

    let acp1_id = creer_dette_pending(
        &app_state,
        org_id,
        owner_id,
        "Les Erables",
        rust_decimal_macros::dec!(842.50),
    )
    .await;
    let acp2_id = creer_dette_pending(
        &app_state,
        org_id,
        owner_id,
        "Les Glycines",
        rust_decimal_macros::dec!(420.00),
    )
    .await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/v1/stats/owner/dues-by-acp")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = test::read_body_json(resp).await;
    let lignes = body.as_array().expect("réponse doit être un tableau");
    assert_eq!(
        lignes.len(),
        2,
        "un copropriétaire de deux ACP doit voir deux lignes, jamais un total fusionné : {body}"
    );

    let ids: Vec<&str> = lignes
        .iter()
        .map(|l| l["acp_id"].as_str().expect("acp_id"))
        .collect();
    assert!(ids.contains(&acp1_id.as_str()));
    assert!(ids.contains(&acp2_id.as_str()));

    for ligne in lignes {
        let montant = ligne["montant"].as_f64().expect("montant numérique");
        let attendu = if ligne["acp_id"] == acp1_id {
            842.50
        } else {
            420.00
        };
        assert!(
            (montant - attendu).abs() < 0.001,
            "montant de {} attendu {attendu}, reçu {montant}",
            ligne["acp_name"]
        );
    }
}

/// @edge — une seule ACP : une seule ligne, pas de décomposition à imposer.
///
/// La décomposition par ACP est une réponse à un cas ordinaire d'investisseur
/// multi-copropriétés. La majorité des copropriétaires n'en ont qu'une : le
/// contrat serveur ne doit ni fusionner à tort, ni inventer une deuxième
/// ligne pour ce cas — un tableau à un élément suffit, c'est au frontend de
/// décider de ne pas afficher de sélecteur pour un choix unique.
#[actix_web::test]
#[serial]
async fn edge_une_seule_acp_rend_une_seule_ligne() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (owner_id, token) = creer_coproprietaire(&app_state, org_id, "Marc").await;

    let acp_id = creer_dette_pending(
        &app_state,
        org_id,
        owner_id,
        "Résidence Unique",
        rust_decimal_macros::dec!(150.00),
    )
    .await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/v1/stats/owner/dues-by-acp")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = test::read_body_json(resp).await;
    let lignes = body.as_array().expect("réponse doit être un tableau");
    assert_eq!(
        lignes.len(),
        1,
        "une seule ACP ne doit rendre qu'une ligne : {body}"
    );
    assert_eq!(lignes[0]["acp_id"], acp_id);
}

/// @security — les dettes d'un copropriétaire ne fuient pas vers un autre.
///
/// `get_owner_dues_by_acp` filtre en SQL sur `uo.owner_id = $1`. Un test au
/// dépôt simulé ne peut pas garder cette clause honnête : seule une requête
/// contre une vraie base, avec deux copropriétaires réels dans des ACP
/// différentes, vérifie qu'aucune ligne de B n'apparaît dans la réponse
/// servie à A.
#[actix_web::test]
#[serial]
async fn security_dettes_ne_fuient_pas_vers_un_autre_coproprietaire() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (owner_a, token_a) = creer_coproprietaire(&app_state, org_id, "Alice").await;
    let (owner_b, _token_b) = creer_coproprietaire(&app_state, org_id, "Bob").await;

    let acp_a = creer_dette_pending(
        &app_state,
        org_id,
        owner_a,
        "ACP Alice",
        rust_decimal_macros::dec!(100.00),
    )
    .await;
    let acp_b = creer_dette_pending(
        &app_state,
        org_id,
        owner_b,
        "ACP Bob",
        rust_decimal_macros::dec!(999.00),
    )
    .await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/v1/stats/owner/dues-by-acp")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token_a)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = test::read_body_json(resp).await;
    let lignes = body.as_array().expect("réponse doit être un tableau");
    assert_eq!(
        lignes.len(),
        1,
        "Alice ne doit voir que ses propres ACP, jamais celles de Bob : {body}"
    );
    assert_eq!(lignes[0]["acp_id"], acp_a);
    assert_ne!(lignes[0]["acp_id"], acp_b);
}

/// @security — un syndic n'a pas de fiche copropriétaire : la route lui est
/// fermée, comme `/stats/owner`.
#[actix_web::test]
#[serial]
async fn security_refuse_un_syndic() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = common::register_and_login_with_role(&app_state, org_id, "syndic").await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/v1/stats/owner/dues-by-acp")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.status(),
        403,
        "un syndic n'a pas de dettes personnelles à consulter ici"
    );
}

/// @negative — sans authentification, la route refuse plutôt que de paniquer.
#[actix_web::test]
#[serial]
async fn negative_refuse_sans_authentification() {
    let (app_state, _container, _org_id) = common::setup_test_db().await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/v1/stats/owner/dues-by-acp")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401);
}

/// @negative — un compte "owner" sans fiche `owners` liée rend une liste
/// vide, pas une erreur. Ce n'est pas un dysfonctionnement : c'est un compte
/// qui n'a encore aucun lot, donc rien à devoir à personne.
#[actix_web::test]
#[serial]
async fn negative_owner_sans_fiche_rend_liste_vide() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = common::register_and_login_with_role(&app_state, org_id, "owner").await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/v1/stats/owner/dues-by-acp")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body.as_array().expect("tableau attendu").len(), 0);
}
