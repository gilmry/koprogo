//! Un lot détenu à deux ne peut pas voter, et rien ne lève la suspension (#848).
//!
//! ── Ce que ce fichier démontre ────────────────────────────────────────────
//!
//! #848 établit sa chaîne maillon par maillon **dans les sources**, et dit
//! elle-même ce qui lui manque :
//!
//! > Je n'ai pas exercé le scénario contre une instance vivante. […] La
//! > démonstration de bout en bout — créer un lot, y rattacher deux
//! > copropriétaires, tenter un vote, lire le 422 — reste à faire. C'est une
//! > vérification de quelques minutes qui vaut plus que ce raisonnement.
//!
//! La voici, exécutée.
//!
//! ── Pourquoi ce test dit l'état ACTUEL, et pas celui qu'on voudrait ───────
//!
//! L'Art. 3.87 § 1er CC ne suspend pas le vote d'une indivision : il prévoit
//! que les indivisaires **désignent** celui d'entre eux, ou un tiers, qui
//! exercera le droit de vote. La suspension n'est que l'état par défaut avant
//! désignation.
//!
//! KoproGo a implémenté la moitié qui interdit et pas celle qui permet. La
//! corriger demande de trancher **qui désigne** — les indivisaires entre eux,
//! ou le syndic à leur place, et sur quelle preuve. C'est une décision
//! produit, la même que #781 et #588.
//!
//! Ce test fige donc le comportement observé, avec son article et son défaut
//! nommés. Le jour où la désignation existera, il devra être MODIFIÉ — et
//! c'est exactement ce qu'on veut : que la levée d'une règle légale soit un
//! geste délibéré, pas un effet de bord.

mod common;

use actix_web::{http::header, test, App};
use common::{
    create_test_building, register_and_login_returning_user, register_and_login_with_role,
    setup_test_db,
};
use koprogo_api::infrastructure::web::routes::configure_routes;
use serde_json::json;
use uuid::Uuid;

/// Lit une réponse sans jamais faire disparaître ce qu'elle contenait.
async fn lire(
    resp: actix_web::dev::ServiceResponse<impl actix_web::body::MessageBody + Unpin>,
) -> (actix_web::http::StatusCode, String, serde_json::Value) {
    let statut = resp.status();
    let brut = test::read_body(resp).await;
    let texte = String::from_utf8_lossy(&brut).to_string();
    let json = serde_json::from_str(&texte).unwrap_or(serde_json::Value::Null);
    (statut, texte, json)
}

/// Le cas le plus ordinaire d'une copropriété belge : un couple, un
/// appartement, deux titulaires actifs sur le même lot.
#[actix_web::test]
async fn negative_un_lot_detenu_a_deux_ne_peut_pas_voter() {
    let (app_state, _container, org_id) = setup_test_db().await;
    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let token = register_and_login_with_role(&app_state, org_id, "syndic").await;
    let building_id = create_test_building(&app_state, org_id).await;
    let acp_id = common::ensure_default_acp_for_org(&app_state.pool, org_id).await;

    let porteur = |req: test::TestRequest| {
        req.insert_header((header::AUTHORIZATION, format!("Bearer {token}")))
    };

    // ── Un lot, deux titulaires ──────────────────────────────────────────
    let resp = test::call_service(
        &app,
        porteur(
            test::TestRequest::post()
                .uri("/api/v1/units")
                .set_json(json!({
                    "acp_id": acp_id.to_string(),
                    "building_id": building_id.to_string(),
                    "unit_number": "A1",
                    "unit_type": "Apartment",
                    "floor": 1,
                    "surface_area": 85.5,
                    "quota": 1000.0
                })),
        )
        .to_request(),
    )
    .await;
    let (statut, texte, unit) = lire(resp).await;
    assert_eq!(statut, 201, "création du lot : {texte}");
    let unit_id = unit["id"].as_str().unwrap().to_string();

    let mut proprietaires = Vec::new();
    let mut jetons: Vec<String> = Vec::new();
    for prenom in ["Alice", "Bertrand"] {
        let resp = test::call_service(
            &app,
            porteur(
                test::TestRequest::post()
                    .uri("/api/v1/owners")
                    .set_json(json!({
                        "organization_id": org_id.to_string(),
                        "first_name": prenom,
                        "last_name": "Indivis",
                        "email": format!("{}-{}@example.com", prenom.to_lowercase(), Uuid::new_v4()),
                        "phone": "+32123456789",
                        "address": "1 rue de l'Indivision",
                        "city": "Bruxelles",
                        "postal_code": "1000",
                        "country": "Belgium"
                    })),
            )
            .to_request(),
        )
        .await;
        let (statut, texte, owner) = lire(resp).await;
        assert_eq!(statut, 201, "création de {prenom} : {texte}");
        let owner_id = owner["id"].as_str().unwrap().to_string();

        // Aucune contrainte d'unicité sur `unit_owners` : le SECOND
        // rattachement passe, et c'est ce qui crée l'indivision.
        let resp = test::call_service(
            &app,
            porteur(
                test::TestRequest::post()
                    .uri(&format!("/api/v1/units/{unit_id}/owners"))
                    .set_json(json!({
                        "owner_id": owner_id,
                        "ownership_percentage": 0.5,
                        "is_primary_contact": prenom == "Alice"
                    })),
            )
            .to_request(),
        )
        .await;
        let (statut, texte, lien) = lire(resp).await;
        assert_eq!(statut, 201, "rattachement de {prenom} : {texte}");

        // La branche qui LÈVE la suspension est morte : la colonne existe,
        // elle vaut `false` par défaut, et rien ne l'écrit jamais.
        assert_ne!(
            lien["is_voting_representative"],
            serde_json::Value::Bool(true),
            "si un rattachement désigne désormais un représentant de vote, la \
             suspension se lève et ce test doit être réécrit — pas rendu vert. \
             Voir #848 : l'arbitrage est « qui désigne ? ». got: {lien}"
        );

        // ── Le copropriétaire reçoit un COMPTE (#850) ───────────────────────
        //
        // Voter exige désormais une fiche de copropriétaire rattachée au
        // compte de l'appelant : `resolution_handlers` refuse en 403 sinon.
        // Ce test votait avec le jeton du syndic — sa SPÉCIFICATION est
        // périmée, pas le produit.
        //
        // Aucune assertion n'est retirée : le test continue de vérifier que
        // l'indivision suspend le vote (422) tant qu'aucun représentant n'est
        // désigné, Art. 3.87 § 1er CC. Il le vérifie simplement depuis le
        // compte qui a le droit de voter.
        let (jeton_copro, user_id) =
            register_and_login_returning_user(&app_state, org_id, "owner").await;
        app_state
            .owner_use_cases
            .link_user_to_owner(
                Uuid::parse_str(&owner_id).expect("identifiant de copropriétaire lisible"),
                Some(user_id),
            )
            .await
            .expect("rattachement du compte au copropriétaire");
        jetons.push(jeton_copro);

        proprietaires.push(owner_id);
    }

    // ── Une assemblée en quorum, avec sa résolution ──────────────────────
    let date_ag = (chrono::Utc::now() + chrono::Duration::days(30)).to_rfc3339();
    let resp = test::call_service(
        &app,
        porteur(
            test::TestRequest::post()
                .uri("/api/v1/meetings")
                .set_json(json!({
                    "organization_id": org_id.to_string(),
                    "building_id": building_id.to_string(),
                    "meeting_type": "Ordinary",
                    "title": "AGO 2026",
                    "description": "Assemblée générale ordinaire",
                    "scheduled_date": date_ag,
                    "location": "Salle des fêtes"
                })),
        )
        .to_request(),
    )
    .await;
    let (statut, texte, meeting) = lire(resp).await;
    assert_eq!(statut, 201, "création de l'assemblée : {texte}");
    let meeting_id = meeting["id"].as_str().unwrap().to_string();

    let resp = test::call_service(
        &app,
        porteur(
            test::TestRequest::post()
                .uri(&format!("/api/v1/meetings/{meeting_id}/agenda"))
                .set_json(json!({ "item": "Approbation des comptes 2025" })),
        )
        .to_request(),
    )
    .await;
    let (statut, texte, _) = lire(resp).await;
    assert_eq!(statut, 200, "ordre du jour : {texte}");

    let resp = test::call_service(
        &app,
        porteur(
            test::TestRequest::post()
                .uri(&format!("/api/v1/meetings/{meeting_id}/validate-quorum"))
                .set_json(json!({ "present_quotas": 1000.0, "total_quotas": 1000.0 })),
        )
        .to_request(),
    )
    .await;
    let (statut, texte, _) = lire(resp).await;
    assert_eq!(statut, 200, "quorum : {texte}");

    let resp = test::call_service(
        &app,
        porteur(
            test::TestRequest::post()
                .uri(&format!("/api/v1/meetings/{meeting_id}/resolutions"))
                .set_json(json!({
                    "meeting_id": meeting_id,
                    "title": "Approbation des comptes 2025",
                    "description": "Comptes de l'exercice écoulé",
                    "resolution_type": "ordinary",
                    "majority_required": "absolute",
                    "agenda_item_index": 0
                })),
        )
        .to_request(),
    )
    .await;
    let (statut, texte, resolution) = lire(resp).await;
    assert_eq!(statut, 201, "création de la résolution : {texte}");
    let resolution_id = resolution["id"].as_str().unwrap().to_string();

    // ── Le vote est refusé, et c'est ce que #848 décrit ──────────────────
    let resp = test::call_service(
        &app,
        {
            let jeton = jetons.first().expect("un compte de copropriétaire").clone();
            let porteur_copro = move |req: test::TestRequest| {
                req.insert_header((header::AUTHORIZATION, format!("Bearer {jeton}")))
            };
            porteur_copro(
                test::TestRequest::post()
                    .uri(&format!("/api/v1/resolutions/{resolution_id}/vote"))
                    .set_json(json!({
                        "owner_id": proprietaires[0],
                        "unit_id": unit_id,
                        "vote_choice": "pour"
                    })),
            )
        }
        .to_request(),
    )
    .await;
    let (statut, texte, erreur) = lire(resp).await;

    assert_eq!(
        statut, 422,
        "un lot en indivision voit son vote refusé tant qu'aucun représentant \
         n'est désigné (Art. 3.87 § 1er CC) : {texte}"
    );
    assert_eq!(
        erreur["details"]["code"], "VOTING_RIGHT_SUSPENDED",
        "le refus doit porter son code, que l'interface consomme pour expliquer \
         au syndic ce qu'il faut faire : {texte}"
    );
    assert_eq!(
        erreur["details"]["unit_id"], unit_id,
        "le refus doit nommer LE lot concerné : un syndic qui gère quatre cents \
         lots ne peut rien faire d'un refus anonyme : {texte}"
    );

    // La suite — désigner un représentant pour lever cette suspension — vit
    // désormais dans les tests ci-dessous : `happy_designer_un_representant_
    // leve_la_suspension`, `edge_un_second_representant_est_refuse`,
    // `security_un_tiers_ne_peut_pas_designer`. Ce test-ci fige ce qui reste
    // vrai APRÈS #848 : sans désignation, la suspension s'applique toujours,
    // et c'est correct (Art. 3.87 §1 CC).
}

/// @happy — désigner l'un des deux titulaires comme représentant lève la
/// suspension : le vote, refusé jusque-là, est accepté (#848).
#[actix_web::test]
async fn happy_designer_un_representant_leve_la_suspension() {
    let (app_state, _container, org_id) = setup_test_db().await;
    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let token = register_and_login_with_role(&app_state, org_id, "syndic").await;
    let building_id = create_test_building(&app_state, org_id).await;
    let acp_id = common::ensure_default_acp_for_org(&app_state.pool, org_id).await;

    let porteur = |req: test::TestRequest| {
        req.insert_header((header::AUTHORIZATION, format!("Bearer {token}")))
    };

    let resp = test::call_service(
        &app,
        porteur(
            test::TestRequest::post()
                .uri("/api/v1/units")
                .set_json(json!({
                    "acp_id": acp_id.to_string(),
                    "building_id": building_id.to_string(),
                    "unit_number": "A1",
                    "unit_type": "Apartment",
                    "floor": 1,
                    "surface_area": 85.5,
                    "quota": 1000.0
                })),
        )
        .to_request(),
    )
    .await;
    let (statut, texte, unit) = lire(resp).await;
    assert_eq!(statut, 201, "création du lot : {texte}");
    let unit_id = unit["id"].as_str().unwrap().to_string();

    let mut proprietaires = Vec::new();
    let mut jetons: Vec<String> = Vec::new();
    for prenom in ["Alice", "Bertrand"] {
        let resp = test::call_service(
            &app,
            porteur(
                test::TestRequest::post()
                    .uri("/api/v1/owners")
                    .set_json(json!({
                        "organization_id": org_id.to_string(),
                        "first_name": prenom,
                        "last_name": "Indivis",
                        "email": format!("{}-{}@example.com", prenom.to_lowercase(), Uuid::new_v4()),
                        "phone": "+32123456789",
                        "address": "1 rue de l'Indivision",
                        "city": "Bruxelles",
                        "postal_code": "1000",
                        "country": "Belgium"
                    })),
            )
            .to_request(),
        )
        .await;
        let (statut, texte, owner) = lire(resp).await;
        assert_eq!(statut, 201, "création de {prenom} : {texte}");
        let owner_id = owner["id"].as_str().unwrap().to_string();

        let resp = test::call_service(
            &app,
            porteur(
                test::TestRequest::post()
                    .uri(&format!("/api/v1/units/{unit_id}/owners"))
                    .set_json(json!({
                        "owner_id": owner_id,
                        "ownership_percentage": 0.5,
                        "is_primary_contact": prenom == "Alice"
                    })),
            )
            .to_request(),
        )
        .await;
        let (statut, texte, _) = lire(resp).await;
        assert_eq!(statut, 201, "rattachement de {prenom} : {texte}");

        // ── Le copropriétaire reçoit un COMPTE (#850) ───────────────────────
        //
        // Voter exige désormais une fiche de copropriétaire rattachée au
        // compte de l'appelant : `resolution_handlers` refuse en 403 sinon.
        // Ce test votait avec le jeton du syndic — sa SPÉCIFICATION est
        // périmée, pas le produit.
        //
        // Aucune assertion n'est retirée : le test continue de vérifier que
        // l'indivision suspend le vote (422) tant qu'aucun représentant n'est
        // désigné, Art. 3.87 § 1er CC. Il le vérifie simplement depuis le
        // compte qui a le droit de voter.
        let (jeton_copro, user_id) =
            register_and_login_returning_user(&app_state, org_id, "owner").await;
        app_state
            .owner_use_cases
            .link_user_to_owner(
                Uuid::parse_str(&owner_id).expect("identifiant de copropriétaire lisible"),
                Some(user_id),
            )
            .await
            .expect("rattachement du compte au copropriétaire");
        jetons.push(jeton_copro);

        proprietaires.push(owner_id);
    }

    // La désignation d'Alice comme représentante lève la suspension.
    let resp = test::call_service(
        &app,
        porteur(
            test::TestRequest::post()
                .uri(&format!("/api/v1/units/{unit_id}/voting-representative"))
                .set_json(json!({ "owner_id": proprietaires[0] })),
        )
        .to_request(),
    )
    .await;
    let (statut, texte, designation) = lire(resp).await;
    assert_eq!(statut, 200, "désignation du représentant : {texte}");
    assert_eq!(designation["is_voting_representative"], true);
    assert_eq!(designation["unit_id"], unit_id);
    assert_eq!(designation["owner_id"], proprietaires[0]);

    // Assemblée + résolution, comme dans le scénario de suspension.
    let date_ag = (chrono::Utc::now() + chrono::Duration::days(30)).to_rfc3339();
    let resp = test::call_service(
        &app,
        porteur(
            test::TestRequest::post()
                .uri("/api/v1/meetings")
                .set_json(json!({
                    "organization_id": org_id.to_string(),
                    "building_id": building_id.to_string(),
                    "meeting_type": "Ordinary",
                    "title": "AGO 2026",
                    "description": "Assemblée générale ordinaire",
                    "scheduled_date": date_ag,
                    "location": "Salle des fêtes"
                })),
        )
        .to_request(),
    )
    .await;
    let (statut, texte, meeting) = lire(resp).await;
    assert_eq!(statut, 201, "création de l'assemblée : {texte}");
    let meeting_id = meeting["id"].as_str().unwrap().to_string();

    let resp = test::call_service(
        &app,
        porteur(
            test::TestRequest::post()
                .uri(&format!("/api/v1/meetings/{meeting_id}/agenda"))
                .set_json(json!({ "item": "Approbation des comptes 2025" })),
        )
        .to_request(),
    )
    .await;
    let (statut, texte, _) = lire(resp).await;
    assert_eq!(statut, 200, "ordre du jour : {texte}");

    let resp = test::call_service(
        &app,
        porteur(
            test::TestRequest::post()
                .uri(&format!("/api/v1/meetings/{meeting_id}/validate-quorum"))
                .set_json(json!({ "present_quotas": 1000.0, "total_quotas": 1000.0 })),
        )
        .to_request(),
    )
    .await;
    let (statut, texte, _) = lire(resp).await;
    assert_eq!(statut, 200, "quorum : {texte}");

    let resp = test::call_service(
        &app,
        porteur(
            test::TestRequest::post()
                .uri(&format!("/api/v1/meetings/{meeting_id}/resolutions"))
                .set_json(json!({
                    "meeting_id": meeting_id,
                    "title": "Approbation des comptes 2025",
                    "description": "Comptes de l'exercice écoulé",
                    "resolution_type": "ordinary",
                    "majority_required": "absolute",
                    "agenda_item_index": 0
                })),
        )
        .to_request(),
    )
    .await;
    let (statut, texte, resolution) = lire(resp).await;
    assert_eq!(statut, 201, "création de la résolution : {texte}");
    let resolution_id = resolution["id"].as_str().unwrap().to_string();

    // Le vote est désormais accepté : la suspension a été levée.
    let resp = test::call_service(
        &app,
        {
            let jeton = jetons.first().expect("un compte de copropriétaire").clone();
            let porteur_copro = move |req: test::TestRequest| {
                req.insert_header((header::AUTHORIZATION, format!("Bearer {jeton}")))
            };
            porteur_copro(
                test::TestRequest::post()
                    .uri(&format!("/api/v1/resolutions/{resolution_id}/vote"))
                    .set_json(json!({
                        "owner_id": proprietaires[0],
                        "unit_id": unit_id,
                        "vote_choice": "pour"
                    })),
            )
        }
        porteur(
            test::TestRequest::post()
                .uri(&format!("/api/v1/resolutions/{resolution_id}/vote"))
                .set_json(json!({
                    "owner_id": proprietaires[0],
                    "unit_id": unit_id,
                    "vote_choice": "pour",
                    "auth_method": "presence"
                })),
        )
        .to_request(),
    )
    .await;
    let (statut, texte, _) = lire(resp).await;
    assert_eq!(
        statut, 201,
        "un lot dont le représentant de vote est désigné doit pouvoir voter \
         (Art. 3.87 §1 CC, #848) : {texte}"
    );
}

/// @edge — un second représentant sur le même lot est refusé
/// (`assert_single_voting_representative`, Art. 3.87 §1 : représentant
/// UNIQUE). Redésigner celui déjà en place reste accepté : ce n'est pas une
/// seconde désignation, c'est un no-op.
#[actix_web::test]
async fn edge_un_second_representant_est_refuse() {
    let (app_state, _container, org_id) = setup_test_db().await;
    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let token = register_and_login_with_role(&app_state, org_id, "syndic").await;
    let building_id = create_test_building(&app_state, org_id).await;
    let acp_id = common::ensure_default_acp_for_org(&app_state.pool, org_id).await;

    let porteur = |req: test::TestRequest| {
        req.insert_header((header::AUTHORIZATION, format!("Bearer {token}")))
    };

    let resp = test::call_service(
        &app,
        porteur(
            test::TestRequest::post()
                .uri("/api/v1/units")
                .set_json(json!({
                    "acp_id": acp_id.to_string(),
                    "building_id": building_id.to_string(),
                    "unit_number": "A1",
                    "unit_type": "Apartment",
                    "floor": 1,
                    "surface_area": 85.5,
                    "quota": 1000.0
                })),
        )
        .to_request(),
    )
    .await;
    let (statut, texte, unit) = lire(resp).await;
    assert_eq!(statut, 201, "création du lot : {texte}");
    let unit_id = unit["id"].as_str().unwrap().to_string();

    let mut proprietaires = Vec::new();
    let mut jetons: Vec<String> = Vec::new();
    for prenom in ["Alice", "Bertrand"] {
        let resp = test::call_service(
            &app,
            porteur(
                test::TestRequest::post()
                    .uri("/api/v1/owners")
                    .set_json(json!({
                        "organization_id": org_id.to_string(),
                        "first_name": prenom,
                        "last_name": "Indivis",
                        "email": format!("{}-{}@example.com", prenom.to_lowercase(), Uuid::new_v4()),
                        "phone": "+32123456789",
                        "address": "1 rue de l'Indivision",
                        "city": "Bruxelles",
                        "postal_code": "1000",
                        "country": "Belgium"
                    })),
            )
            .to_request(),
        )
        .await;
        let (statut, texte, owner) = lire(resp).await;
        assert_eq!(statut, 201, "création de {prenom} : {texte}");
        let owner_id = owner["id"].as_str().unwrap().to_string();

        let resp = test::call_service(
            &app,
            porteur(
                test::TestRequest::post()
                    .uri(&format!("/api/v1/units/{unit_id}/owners"))
                    .set_json(json!({
                        "owner_id": owner_id,
                        "ownership_percentage": 0.5,
                        "is_primary_contact": prenom == "Alice"
                    })),
            )
            .to_request(),
        )
        .await;
        let (statut, texte, _) = lire(resp).await;
        assert_eq!(statut, 201, "rattachement de {prenom} : {texte}");

        // ── Le copropriétaire reçoit un COMPTE (#850) ───────────────────────
        //
        // Voter exige désormais une fiche de copropriétaire rattachée au
        // compte de l'appelant : `resolution_handlers` refuse en 403 sinon.
        // Ce test votait avec le jeton du syndic — sa SPÉCIFICATION est
        // périmée, pas le produit.
        //
        // Aucune assertion n'est retirée : le test continue de vérifier que
        // l'indivision suspend le vote (422) tant qu'aucun représentant n'est
        // désigné, Art. 3.87 § 1er CC. Il le vérifie simplement depuis le
        // compte qui a le droit de voter.
        let (jeton_copro, user_id) =
            register_and_login_returning_user(&app_state, org_id, "owner").await;
        app_state
            .owner_use_cases
            .link_user_to_owner(
                Uuid::parse_str(&owner_id).expect("identifiant de copropriétaire lisible"),
                Some(user_id),
            )
            .await
            .expect("rattachement du compte au copropriétaire");
        jetons.push(jeton_copro);

        proprietaires.push(owner_id);
    }

    let resp = test::call_service(
        &app,
        porteur(
            test::TestRequest::post()
                .uri(&format!("/api/v1/units/{unit_id}/voting-representative"))
                .set_json(json!({ "owner_id": proprietaires[0] })),
        )
        .to_request(),
    )
    .await;
    let (statut, texte, _) = lire(resp).await;
    assert_eq!(statut, 200, "désignation d'Alice : {texte}");

    // Bertrand ne peut pas être désigné à son tour : un seul représentant par
    // lot (Art. 3.87 §1 CC).
    let resp = test::call_service(
        &app,
        porteur(
            test::TestRequest::post()
                .uri(&format!("/api/v1/units/{unit_id}/voting-representative"))
                .set_json(json!({ "owner_id": proprietaires[1] })),
        )
        .to_request(),
    )
    .await;
    let (statut, texte, _) = lire(resp).await;
    assert_eq!(
        statut, 409,
        "un second représentant pour le même lot doit être refusé : {texte}"
    );

    // Redésigner Alice, déjà en place, reste accepté (idempotent).
    let resp = test::call_service(
        &app,
        porteur(
            test::TestRequest::post()
                .uri(&format!("/api/v1/units/{unit_id}/voting-representative"))
                .set_json(json!({ "owner_id": proprietaires[0] })),
        )
        .to_request(),
    )
    .await;
    let (statut, texte, _) = lire(resp).await;
    assert_eq!(
        statut, 200,
        "redésigner le représentant déjà en place n'est pas une seconde \
         désignation : {texte}"
    );
}

/// @security — un syndic d'une AUTRE organisation ne peut pas désigner le
/// représentant de vote d'un lot qui n'est pas le sien : la désignation
/// engage le droit de vote d'autrui, elle n'est pas ouverte à l'organisation
/// entière (#848).
#[actix_web::test]
async fn security_un_tiers_ne_peut_pas_designer() {
    let (app_state, _container, org_id) = setup_test_db().await;
    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let token = register_and_login_with_role(&app_state, org_id, "syndic").await;
    let building_id = create_test_building(&app_state, org_id).await;
    let acp_id = common::ensure_default_acp_for_org(&app_state.pool, org_id).await;

    let porteur = |req: test::TestRequest| {
        req.insert_header((header::AUTHORIZATION, format!("Bearer {token}")))
    };

    let resp = test::call_service(
        &app,
        porteur(
            test::TestRequest::post()
                .uri("/api/v1/units")
                .set_json(json!({
                    "acp_id": acp_id.to_string(),
                    "building_id": building_id.to_string(),
                    "unit_number": "A1",
                    "unit_type": "Apartment",
                    "floor": 1,
                    "surface_area": 85.5,
                    "quota": 1000.0
                })),
        )
        .to_request(),
    )
    .await;
    let (statut, texte, unit) = lire(resp).await;
    assert_eq!(statut, 201, "création du lot : {texte}");
    let unit_id = unit["id"].as_str().unwrap().to_string();

    let mut proprietaires = Vec::new();
    let mut jetons: Vec<String> = Vec::new();
    for prenom in ["Alice", "Bertrand"] {
        let resp = test::call_service(
            &app,
            porteur(
                test::TestRequest::post()
                    .uri("/api/v1/owners")
                    .set_json(json!({
                        "organization_id": org_id.to_string(),
                        "first_name": prenom,
                        "last_name": "Indivis",
                        "email": format!("{}-{}@example.com", prenom.to_lowercase(), Uuid::new_v4()),
                        "phone": "+32123456789",
                        "address": "1 rue de l'Indivision",
                        "city": "Bruxelles",
                        "postal_code": "1000",
                        "country": "Belgium"
                    })),
            )
            .to_request(),
        )
        .await;
        let (statut, texte, owner) = lire(resp).await;
        assert_eq!(statut, 201, "création de {prenom} : {texte}");
        let owner_id = owner["id"].as_str().unwrap().to_string();

        let resp = test::call_service(
            &app,
            porteur(
                test::TestRequest::post()
                    .uri(&format!("/api/v1/units/{unit_id}/owners"))
                    .set_json(json!({
                        "owner_id": owner_id,
                        "ownership_percentage": 0.5,
                        "is_primary_contact": prenom == "Alice"
                    })),
            )
            .to_request(),
        )
        .await;
        let (statut, texte, _) = lire(resp).await;
        assert_eq!(statut, 201, "rattachement de {prenom} : {texte}");

        // ── Le copropriétaire reçoit un COMPTE (#850) ───────────────────────
        //
        // Voter exige désormais une fiche de copropriétaire rattachée au
        // compte de l'appelant : `resolution_handlers` refuse en 403 sinon.
        // Ce test votait avec le jeton du syndic — sa SPÉCIFICATION est
        // périmée, pas le produit.
        //
        // Aucune assertion n'est retirée : le test continue de vérifier que
        // l'indivision suspend le vote (422) tant qu'aucun représentant n'est
        // désigné, Art. 3.87 § 1er CC. Il le vérifie simplement depuis le
        // compte qui a le droit de voter.
        let (jeton_copro, user_id) =
            register_and_login_returning_user(&app_state, org_id, "owner").await;
        app_state
            .owner_use_cases
            .link_user_to_owner(
                Uuid::parse_str(&owner_id).expect("identifiant de copropriétaire lisible"),
                Some(user_id),
            )
            .await
            .expect("rattachement du compte au copropriétaire");
        jetons.push(jeton_copro);

        proprietaires.push(owner_id);
    }

    // Un syndic d'une organisation SANS mandat sur ce lot.
    let autre_org_id = common::create_test_organization(&app_state).await;
    let autre_token = register_and_login_with_role(&app_state, autre_org_id, "syndic").await;

    let resp = test::call_service(
        &app,
        test::TestRequest::post()
            .uri(&format!("/api/v1/units/{unit_id}/voting-representative"))
            .insert_header((header::AUTHORIZATION, format!("Bearer {autre_token}")))
            .set_json(json!({ "owner_id": proprietaires[0] }))
            .to_request(),
    )
    .await;
    let (statut, texte, _) = lire(resp).await;
    assert_eq!(
        statut, 403,
        "un syndic hors mandat sur ce lot ne peut pas désigner son \
         représentant de vote : {texte}"
    );
}
