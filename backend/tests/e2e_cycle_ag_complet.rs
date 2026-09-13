//! Le cycle de vie d'une assemblée générale, de bout en bout (#780).
//!
//! ── Pourquoi ce fichier existe ────────────────────────────────────────────
//!
//! La recette 4 a trouvé **trois verrous indépendants** entre la création
//! d'une AG et sa clôture, dont aucun n'était contournable depuis l'interface.
//! C'est le parcours central du produit, et le seul endroit où le travail du
//! syndic s'arrête net.
//!
//! Les trois ont été corrigés séparément. Rien ne prouvait qu'ils
//! s'enchaînent.
//!
//! `e2e_meetings.rs` porte déjà un `test_meeting_complete_lifecycle`. Il
//! marche : créer, ordre du jour, modifier, clôturer. **Il passe à côté des
//! trois verrous** — ni convocation, ni envoi, ni résolution, ni vote, ni
//! clôture du vote. Un test dont le nom promet le cycle complet et qui évite
//! précisément la partie qui ne marchait pas.
//!
//! ── Ce que celui-ci exige ─────────────────────────────────────────────────
//!
//! Chaque étape doit rendre le code attendu, dans l'ordre, sur une même
//! assemblée. Aucune n'est simulée : c'est l'API réelle, sur une base réelle.

mod common;

use actix_web::{http::header, test, App};
use common::{create_test_building, register_and_login_with_role, setup_test_db};
use koprogo_api::infrastructure::web::routes::configure_routes;
use serde_json::json;
use uuid::Uuid;

/// Lit une réponse sans jamais faire disparaître ce qu'elle contenait.
///
/// `test::read_body_json` panique AVANT l'assertion quand le corps n'est pas du
/// JSON — un 404 au corps vide, par exemple. On perd alors le statut et
/// l'adresse appelée, c'est-à-dire tout ce qui aurait permis de comprendre.
async fn lire(
    resp: actix_web::dev::ServiceResponse<impl actix_web::body::MessageBody + Unpin>,
) -> (actix_web::http::StatusCode, String, serde_json::Value) {
    let statut = resp.status();
    let brut = test::read_body(resp).await;
    let texte = String::from_utf8_lossy(&brut).to_string();
    let json = serde_json::from_str(&texte).unwrap_or(serde_json::Value::Null);
    (statut, texte, json)
}

/// Le parcours complet, du calendrier au procès-verbal.
///
/// Les trois verrous de #780 sont franchis dans l'ordre où ils se présentent
/// au syndic :
///
///   1. convoquer une AG dont la date laisse le délai légal (Art. 3.87 § 3) ;
///   2. l'ENVOYER, ce qui suppose des destinataires ;
///   3. mettre une résolution aux voix, puis clore le vote.
#[actix_web::test]
async fn happy_le_cycle_dune_ag_va_de_la_convocation_a_la_cloture() {
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

    // ── Un immeuble habité ───────────────────────────────────────────────
    //
    // Sans lot ni copropriétaire, la convocation n'a personne à joindre et le
    // vote n'a aucune quotité à lire. Ce n'est pas du décor : c'est ce que le
    // produit exige désormais, et ce que la recette 4 n'avait pas.
    let mut lots = Vec::new();
    for (numero, quota) in [("A1", 600.0_f64), ("A2", 400.0_f64)] {
        let resp = test::call_service(
            &app,
            porteur(
                test::TestRequest::post()
                    .uri("/api/v1/owners")
                    .set_json(json!({
                        "organization_id": org_id.to_string(),
                        "first_name": "Copro",
                        "last_name": numero,
                        "email": format!("copro-{}-{}@example.com", numero, Uuid::new_v4()),
                        "phone": "+32123456789",
                        "address": "1 rue du Lot",
                        "city": "Bruxelles",
                        "postal_code": "1000",
                        "country": "Belgium"
                    })),
            )
            .to_request(),
        )
        .await;
        let (statut, texte, owner) = lire(resp).await;
        assert_eq!(statut, 201, "création du copropriétaire {numero} : {texte}");
        let owner_id = owner["id"].as_str().unwrap().to_string();

        let resp = test::call_service(
            &app,
            porteur(
                test::TestRequest::post()
                    .uri("/api/v1/units")
                    .set_json(json!({
                        "acp_id": acp_id.to_string(),
                        "building_id": building_id.to_string(),
                        "unit_number": numero,
                        "unit_type": "Apartment",
                        "floor": 1,
                        "surface_area": 85.5,
                        "quota": quota
                    })),
            )
            .to_request(),
        )
        .await;
        let (statut, texte, unit) = lire(resp).await;
        assert_eq!(statut, 201, "création du lot {numero} : {texte}");
        let unit_id = unit["id"].as_str().unwrap().to_string();

        let resp = test::call_service(
            &app,
            porteur(
                test::TestRequest::post()
                    .uri(&format!("/api/v1/units/{unit_id}/owners"))
                    .set_json(json!({
                        "owner_id": owner_id,
                        "ownership_percentage": 1.0,
                        "is_primary_contact": true
                    })),
            )
            .to_request(),
        )
        .await;
        let (statut, texte, _) = lire(resp).await;
        assert_eq!(statut, 201, "rattachement du lot {numero} : {texte}");

        lots.push((owner_id, unit_id, quota));
    }

    // ── 1. Une AG dont la date laisse le délai légal ─────────────────────
    //
    // Trente jours : le délai de quinze jours de l'Art. 3.87 § 3 tient
    // largement. La recette 4 butait ici parce que l'AG avait été créée à
    // moins de quinze jours, sans que rien ne l'ait signalé à la saisie.
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
    let (statut, texte, meeting_corps) = lire(resp).await;
    assert_eq!(statut, 201, "création de l'assemblée : {texte}");
    let meeting_id = meeting_corps["id"].as_str().unwrap().to_string();

    // Un point d'ordre du jour : sans lui, toute décision serait nulle
    // (Art. 3.87 § 2 CC) et le vote est refusé.
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
    assert_eq!(
        statut, 200,
        "inscription du point à l'ordre du jour : {texte}"
    );

    // ── 2. La convocation, créée PUIS envoyée ────────────────────────────
    let resp = test::call_service(
        &app,
        porteur(
            test::TestRequest::post()
                .uri("/api/v1/convocations")
                .set_json(json!({
                    "building_id": building_id.to_string(),
                    "meeting_id": meeting_id,
                    "meeting_type": "Ordinary",
                    "meeting_date": date_ag,
                    "language": "Fr"
                })),
        )
        .to_request(),
    )
    .await;
    let (statut, texte, convocation) = lire(resp).await;
    assert_eq!(statut, 201, "création de la convocation : {texte}");
    let convocation_id = convocation["id"].as_str().unwrap().to_string();

    // Le corps est VIDE, exactement comme l'interface l'envoie : elle n'a
    // aucun contrôle pour désigner les destinataires. Le serveur déduit —
    // convoquer une assemblée, c'est convoquer tout le monde.
    let resp = test::call_service(
        &app,
        porteur(
            test::TestRequest::post()
                .uri(&format!("/api/v1/convocations/{convocation_id}/send"))
                .set_json(json!({})),
        )
        .to_request(),
    )
    .await;
    let (statut, _texte, corps) = lire(resp).await;
    assert_eq!(
        statut, 200,
        "l'envoi de la convocation doit aboutir sans destinataires explicites, got: {corps}"
    );
    assert_eq!(
        corps["total_recipients"].as_i64(),
        Some(2),
        "les deux copropriétaires de l'immeuble doivent être convoqués, got: {corps}"
    );

    // ── 3. Le quorum, la résolution, les votes, la clôture ───────────────
    //
    // Art. 3.87 § 5 : sans quorum, aucun vote n'est valable. Les deux lots
    // sont présents, soit 1000 millièmes sur 1000.
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
    assert_eq!(statut, 200, "validation du quorum : {texte}");

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

    for (owner_id, unit_id, quota) in &lots {
        let resp = test::call_service(
            &app,
            porteur(
                test::TestRequest::post()
                    .uri(&format!("/api/v1/resolutions/{resolution_id}/vote"))
                    .set_json(json!({
                        "owner_id": owner_id,
                        "unit_id": unit_id,
                        "vote_choice": "pour"
                    })),
            )
            .to_request(),
        )
        .await;
        let statut = resp.status();
        let brut = test::read_body(resp).await;
        let texte = String::from_utf8_lossy(&brut).to_string();
        assert_eq!(statut, 201, "vote du lot, got: {statut} {texte}");
        let vote: serde_json::Value = serde_json::from_str(&texte).expect("corps JSON du vote");

        // #850 — la voix enregistrée est la quotité du lot, lue sur l'acte de
        // base. Aucune puissance n'a été transmise dans l'appel ci-dessus.
        assert_eq!(
            vote["voting_power"]
                .as_str()
                .and_then(|s| s.parse::<f64>().ok()),
            Some(*quota),
            "la voix doit valoir la quotité du lot (Art. 3.87 § 8 CC), got: {vote}"
        );
    }

    let resp = test::call_service(
        &app,
        porteur(
            test::TestRequest::put()
                .uri(&format!("/api/v1/resolutions/{resolution_id}/close"))
                .set_json(json!({})),
        )
        .to_request(),
    )
    .await;
    let (statut, _texte, close) = lire(resp).await;
    assert_eq!(
        statut, 200,
        "la clôture du vote doit aboutir sans dénominateur transmis, got: {close}"
    );
    assert_eq!(
        close["status"], "adopted",
        "deux voix pour, aucune contre : la résolution est adoptée, got: {close}"
    );

    // ── Le plafond de l'Art. 3.87 § 7 al. 4, enfin observable ────────────
    //
    // Les deux lots pèsent 600 et 400 millièmes. Le total retenu au décompte
    // est **800**, pas 1000 : nul ne peut voter pour un nombre de voix
    // supérieur à la somme des voix des autres copropriétaires présents, donc
    // les 600 sont ramenés à 400.
    //
    // #780 disait que ce plafonnement resterait invérifiable tant que le
    // cycle ne pourrait pas aller à son terme. Il va à son terme, et le
    // plafond s'observe — sans qu'aucune assertion n'ait eu à le provoquer.
    assert_eq!(
        close["total_voting_power_pour"]
            .as_str()
            .and_then(|s| s.parse::<f64>().ok()),
        Some(800.0),
        "600 millièmes plafonnés à 400, la somme des autres présents \
         (Art. 3.87 § 7 al. 4 CC), got: {close}"
    );

    // ── Le terme : l'assemblée se clôt ───────────────────────────────────
    //
    // C'est la condition que la recette 4 n'a jamais pu atteindre : les trois
    // verrous étaient cumulatifs, et aucun ne se franchissait seul.
    let resp = test::call_service(
        &app,
        porteur(
            test::TestRequest::post()
                .uri(&format!("/api/v1/meetings/{meeting_id}/complete"))
                // Les deux copropriétaires étaient présents : c'est le même
                // fait que le quorum validé plus haut.
                .set_json(json!({ "attendees_count": 2 })),
        )
        .to_request(),
    )
    .await;
    let (statut, texte, clos) = lire(resp).await;
    assert_eq!(
        statut, 200,
        "l'assemblée doit pouvoir être clôturée : {texte}"
    );
    assert_eq!(clos["status"], "Completed", "got: {clos}");
}
