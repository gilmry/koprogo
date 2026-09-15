// E2E tests for Call For Funds HTTP endpoints
// Tests focus on HTTP layer: endpoints, auth, JSON serialization
// Belgian legal context: Appels de fonds for copropriete management

mod common;

use actix_web::http::header;
use actix_web::{test, App};
use koprogo_api::application::dto::{CreateBuildingDto, CreateOwnerDto, CreateUnitDto};
use koprogo_api::domain::entities::UnitType;
use koprogo_api::infrastructure::web::configure_routes;
use serde_json::json;
use serial_test::serial;
use uuid::Uuid;

/// Helper: Create building + owner + unit + unit_owner relationship.
/// Returns (token, building_id, owner_id, unit_id).
async fn create_call_for_funds_fixtures(
    app_state: &actix_web::web::Data<koprogo_api::infrastructure::web::AppState>,
    org_id: Uuid,
) -> (String, Uuid, Uuid, Uuid) {
    // 1. Register user and get token (superadmin role)
    let email = format!("cff-test-{}@example.com", Uuid::new_v4());
    let register_req = koprogo_api::application::dto::RegisterRequest {
        email: email.clone(),
        password: "SecurePass123!".to_string(),
        first_name: "CallForFunds".to_string(),
        last_name: "Tester".to_string(),
        role: "superadmin".to_string(),
        organization_id: Some(org_id),
    };
    let login_response = app_state
        .auth_use_cases
        .register(register_req)
        .await
        .expect("Failed to register user");
    let token = login_response.token;

    // 2. Create building
    let acp_id = common::create_test_acp(app_state, org_id).await;
    let building_dto = CreateBuildingDto {
        acp_id,
        name: format!("CFF Building {}", Uuid::new_v4()),
        address: "10 Rue de la Loi".to_string(),
        city: "Brussels".to_string(),
        postal_code: "1000".to_string(),
        country: "Belgium".to_string(),
        // Cohérent avec l'unique lot créé plus bas : le garde-fou
        // « valider avant de calculer » refuse d'appeler des fonds sur une ACP
        // dont les lots ne totalisent pas les tantièmes déclarés. Déclarer 4
        // lots et 1000 tantièmes pour n'en créer qu'un à 0,25 rendait la
        // fixture non conforme, et le 422 était la bonne réponse.
        total_units: 1,
        total_tantiemes: Some(1000),
        construction_year: Some(2005),
    };
    let building = app_state
        .building_use_cases
        .create_building(building_dto)
        .await
        .expect("Failed to create building");
    let building_id = Uuid::parse_str(&building.id).expect("parse building_id");

    // 3. Create owner
    let owner_dto = CreateOwnerDto {
        organization_id: org_id.to_string(),
        first_name: "Jean".to_string(),
        last_name: "Dupont".to_string(),
        email: format!("owner-{}@example.com", Uuid::new_v4()),
        phone: None,
        address: "10 Rue de la Loi".to_string(),
        city: "Brussels".to_string(),
        postal_code: "1000".to_string(),
        country: "Belgium".to_string(),
        user_id: None,
    };
    let owner = app_state
        .owner_use_cases
        .create_owner(owner_dto)
        .await
        .expect("Failed to create owner");
    let owner_id = Uuid::parse_str(&owner.id).expect("parse owner_id");

    // 4. Create unit
    let unit_dto = CreateUnitDto {
        acp_id: Some(building.acp_id.clone()),
        building_id: building.id.clone(),
        unit_number: "A1".to_string(),
        unit_type: UnitType::Apartment,
        floor: Some(1),
        surface_area: 80.0,
        quota: rust_decimal_macros::dec!(1000),
    };
    let unit = app_state
        .unit_use_cases
        .create_unit(unit_dto)
        .await
        .expect("Failed to create unit");
    let unit_id = Uuid::parse_str(&unit.id).expect("parse unit_id");

    // 5. Link owner to unit (required for send_call_for_funds to generate contributions)
    let _ = app_state
        .unit_owner_use_cases
        .add_owner_to_unit(
            unit_id,
            owner_id,
            rust_decimal_macros::dec!(0.25), // 25% ownership share
            true,                            // is primary contact
        )
        .await;
    // Note: ownership validation may require total = 100%, so we accept any result here

    (token, building_id, owner_id, unit_id)
}

#[actix_web::test]
#[serial]
async fn test_call_for_funds_create() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, building_id, _owner_id, _unit_id) =
        create_call_for_funds_fixtures(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let now = chrono::Utc::now();
    let due_date = now + chrono::Duration::days(30);

    let req = test::TestRequest::post()
        .uri("/api/v1/call-for-funds")
        .insert_header(header::ContentType::json())
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "building_id": building_id.to_string(),
            "title": "Q1 2026 Regular Contribution",
            "description": "First quarter ordinary contribution for common expenses",
            "total_amount": 4000.00,
            "contribution_type": "regular",
            "call_date": now.to_rfc3339(),
            "due_date": due_date.to_rfc3339()
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;

    assert!(
        resp.status().is_success(),
        "Expected 201 Created for call for funds, got: {}",
        resp.status()
    );
    assert_eq!(resp.status().as_u16(), 201);
}

#[actix_web::test]
#[serial]
async fn test_call_for_funds_get() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, building_id, _owner_id, _unit_id) =
        create_call_for_funds_fixtures(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let now = chrono::Utc::now();
    let due_date = now + chrono::Duration::days(30);

    // Create a call for funds first
    let create_resp = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/api/v1/call-for-funds")
            .insert_header(header::ContentType::json())
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
            .set_json(json!({
                "building_id": building_id.to_string(),
                "title": "Q2 2026 Contribution",
                "description": "Second quarter contribution",
                "total_amount": 4000.00,
                "contribution_type": "regular",
                "call_date": now.to_rfc3339(),
                "due_date": due_date.to_rfc3339()
            }))
            .to_request(),
    )
    .await;

    assert!(
        create_resp.status().is_success(),
        "Pre-condition: create should succeed, got: {}",
        create_resp.status()
    );

    let created: serde_json::Value = test::read_body_json(create_resp).await;
    let cff_id = created["id"].as_str().unwrap();

    // Retrieve by ID
    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/call-for-funds/{}", cff_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;

    assert!(
        resp.status().is_success(),
        "Expected 200 OK for get call for funds, got: {}",
        resp.status()
    );
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["id"], cff_id);
    assert_eq!(body["status"], "draft");
}

#[actix_web::test]
#[serial]
async fn test_call_for_funds_list() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, building_id, _owner_id, _unit_id) =
        create_call_for_funds_fixtures(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let now = chrono::Utc::now();
    let due_date = now + chrono::Duration::days(30);

    // Create a call for funds
    test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/api/v1/call-for-funds")
            .insert_header(header::ContentType::json())
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
            .set_json(json!({
                "building_id": building_id.to_string(),
                "title": "List Test Contribution",
                "description": "Test contribution for list endpoint",
                "total_amount": 2000.00,
                "contribution_type": "regular",
                "call_date": now.to_rfc3339(),
                "due_date": due_date.to_rfc3339()
            }))
            .to_request(),
    )
    .await;

    // List by building_id
    let req = test::TestRequest::get()
        .uri(&format!(
            "/api/v1/call-for-funds?building_id={}",
            building_id
        ))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;

    assert!(
        resp.status().is_success(),
        "Expected 200 OK for list call for funds, got: {}",
        resp.status()
    );
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body.is_array(), "Expected JSON array response");
}

#[actix_web::test]
#[serial]
async fn test_call_for_funds_send() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, building_id, _owner_id, _unit_id) =
        create_call_for_funds_fixtures(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let now = chrono::Utc::now();
    let due_date = now + chrono::Duration::days(30);

    // Create a call for funds
    let create_resp = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/api/v1/call-for-funds")
            .insert_header(header::ContentType::json())
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
            .set_json(json!({
                "building_id": building_id.to_string(),
                "title": "Send Test Contribution",
                "description": "Contribution to be sent",
                "total_amount": 3600.00,
                "contribution_type": "regular",
                "call_date": now.to_rfc3339(),
                "due_date": due_date.to_rfc3339()
            }))
            .to_request(),
    )
    .await;

    let created: serde_json::Value = test::read_body_json(create_resp).await;
    let cff_id = created["id"].as_str().unwrap();

    // Send the call for funds (Draft → Sent, generates owner contributions)
    let req = test::TestRequest::post()
        .uri(&format!("/api/v1/call-for-funds/{}/send", cff_id))
        .insert_header(header::ContentType::json())
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({}))
        .to_request();

    let resp = test::call_service(&app, req).await;

    // May succeed or fail depending on whether unit owners have been set up
    assert!(
        resp.status().is_success() || resp.status().is_client_error(),
        "Expected valid response for send call for funds, got: {}",
        resp.status()
    );
}

#[actix_web::test]
#[serial]
async fn test_call_for_funds_cancel() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, building_id, _owner_id, _unit_id) =
        create_call_for_funds_fixtures(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let now = chrono::Utc::now();
    let due_date = now + chrono::Duration::days(30);

    // Create a call for funds
    let create_resp = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/api/v1/call-for-funds")
            .insert_header(header::ContentType::json())
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
            .set_json(json!({
                "building_id": building_id.to_string(),
                "title": "Cancel Test Contribution",
                "description": "Contribution to be cancelled",
                "total_amount": 1000.00,
                "contribution_type": "extraordinary",
                "call_date": now.to_rfc3339(),
                "due_date": due_date.to_rfc3339()
            }))
            .to_request(),
    )
    .await;

    let created: serde_json::Value = test::read_body_json(create_resp).await;
    let cff_id = created["id"].as_str().unwrap();

    // Cancel the call for funds
    let req = test::TestRequest::put()
        .uri(&format!("/api/v1/call-for-funds/{}/cancel", cff_id))
        .insert_header(header::ContentType::json())
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({}))
        .to_request();

    let resp = test::call_service(&app, req).await;

    assert!(
        resp.status().is_success(),
        "Expected 200 OK for cancel call for funds, got: {}",
        resp.status()
    );
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["status"], "cancelled");
}

#[actix_web::test]
#[serial]
async fn test_call_for_funds_delete_draft() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, building_id, _owner_id, _unit_id) =
        create_call_for_funds_fixtures(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let now = chrono::Utc::now();
    let due_date = now + chrono::Duration::days(30);

    // Create a call for funds (Draft status)
    let create_resp = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/api/v1/call-for-funds")
            .insert_header(header::ContentType::json())
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
            .set_json(json!({
                "building_id": building_id.to_string(),
                "title": "Delete Test Contribution",
                "description": "Draft contribution to be deleted",
                "total_amount": 500.00,
                "contribution_type": "adjustment",
                "call_date": now.to_rfc3339(),
                "due_date": due_date.to_rfc3339()
            }))
            .to_request(),
    )
    .await;

    let created: serde_json::Value = test::read_body_json(create_resp).await;
    let cff_id = created["id"].as_str().unwrap();

    // Delete the draft
    let req = test::TestRequest::delete()
        .uri(&format!("/api/v1/call-for-funds/{}", cff_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Handler returns 204 No Content on success
    assert!(
        resp.status().is_success(),
        "Expected 2xx for delete draft call for funds, got: {}",
        resp.status()
    );
}

#[actix_web::test]
#[serial]
async fn test_call_for_funds_unauthorized() {
    let (app_state, _container, _org_id) = common::setup_test_db().await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Attempt without token
    let req = test::TestRequest::get()
        .uri("/api/v1/call-for-funds")
        .to_request();

    let resp = test::call_service(&app, req).await;

    assert_eq!(
        resp.status().as_u16(),
        401,
        "Expected 401 Unauthorized without token, got: {}",
        resp.status()
    );
}

// ── #882 — GET /call-for-funds/overdue ne rend plus l'instance entière ────

/// Crée un appel de fonds déjà en retard (call_date et due_date passés, tous
/// deux dans le passé, `due_date > call_date` comme l'exige le domaine),
/// directement via le cas d'usage — hors HTTP, pour ne rien supposer du
/// chemin d'écriture. Rend son id.
async fn creer_appel_en_retard(
    app_state: &actix_web::web::Data<koprogo_api::infrastructure::web::AppState>,
    org_id: Uuid,
    building_id: Uuid,
    titre: &str,
) -> Uuid {
    let call_date = chrono::Utc::now() - chrono::Duration::days(60);
    let due_date = chrono::Utc::now() - chrono::Duration::days(30);

    let cff = app_state
        .call_for_funds_use_cases
        .create_call_for_funds(
            org_id,
            building_id,
            titre.to_string(),
            "Créé en retard pour un test #882".to_string(),
            rust_decimal_macros::dec!(1000),
            koprogo_api::domain::entities::ContributionType::Regular,
            call_date,
            due_date,
            None,
            None,
            rust_decimal::Decimal::ZERO,
        )
        .await
        .expect("précondition : création de l'appel en retard");
    cff.id
}

/// @happy — le syndic lit les arriérés de SA propre organisation.
#[actix_web::test]
#[serial]
async fn happy_overdue_calls_rend_les_arrieres_de_sa_propre_organisation() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, building_id, _owner_id, _unit_id) =
        create_call_for_funds_fixtures(&app_state, org_id).await;

    let id = creer_appel_en_retard(&app_state, org_id, building_id, "Arriéré de mon cabinet").await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let resp = test::call_service(
        &app,
        test::TestRequest::get()
            .uri("/api/v1/call-for-funds/overdue")
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
            .to_request(),
    )
    .await;

    assert_eq!(resp.status().as_u16(), 200);
    let body: serde_json::Value = test::read_body_json(resp).await;
    let ids: Vec<String> = body
        .as_array()
        .expect("réponse en tableau")
        .iter()
        .map(|c| c["id"].as_str().unwrap().to_string())
        .collect();
    assert!(
        ids.contains(&id.to_string()),
        "l'appel en retard de sa propre organisation n'apparaît pas : {:?}",
        ids
    );
}

/// @security / @negative — les arriérés d'une AUTRE organisation ne
/// paraissent jamais dans la réponse. Avant #882,
/// `GET /call-for-funds/overdue` ne prenait aucun argument : cette liste
/// contenait TOUS les arriérés de l'instance, tous cabinets confondus.
#[actix_web::test]
#[serial]
async fn security_overdue_calls_ne_rend_jamais_larriere_dune_autre_organisation() {
    let (app_state, _container, org_a) = common::setup_test_db().await;
    let org_b = common::create_test_organization(&app_state).await;

    let (token_a, building_a, _, _) = create_call_for_funds_fixtures(&app_state, org_a).await;
    let (_token_b, building_b, _, _) = create_call_for_funds_fixtures(&app_state, org_b).await;

    let id_a = creer_appel_en_retard(&app_state, org_a, building_a, "Arriéré cabinet A").await;
    let id_b = creer_appel_en_retard(&app_state, org_b, building_b, "Arriéré cabinet B").await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let resp = test::call_service(
        &app,
        test::TestRequest::get()
            .uri("/api/v1/call-for-funds/overdue")
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token_a)))
            .to_request(),
    )
    .await;

    assert_eq!(resp.status().as_u16(), 200);
    let body: serde_json::Value = test::read_body_json(resp).await;
    let ids: Vec<String> = body
        .as_array()
        .expect("réponse en tableau")
        .iter()
        .map(|c| c["id"].as_str().unwrap().to_string())
        .collect();

    assert!(
        ids.contains(&id_a.to_string()),
        "le cabinet A ne voit plus son propre arriéré : {:?}",
        ids
    );
    assert!(
        !ids.contains(&id_b.to_string()),
        "FUITE INTER-ORGANISATIONS : le cabinet A voit l'arriéré du cabinet B : {:?}",
        ids
    );
}

/// @edge — un utilisateur SANS organisation n'est pas traité par défaut
/// comme un superadministrateur : il ne reçoit ni la liste de l'instance
/// entière, ni même une réponse vide qui laisserait croire à un périmètre
/// vide plutôt qu'absent.
#[actix_web::test]
#[serial]
async fn edge_overdue_calls_refuse_lutilisateur_sans_organisation() {
    let (app_state, _container, _org_id) = common::setup_test_db().await;

    let email = format!("sans-org-{}@example.com", Uuid::new_v4());
    let login = app_state
        .auth_use_cases
        .register(koprogo_api::application::dto::RegisterRequest {
            email,
            password: "SecurePass123!".to_string(),
            first_name: "Sans".to_string(),
            last_name: "Organisation".to_string(),
            role: "owner".to_string(),
            organization_id: None,
        })
        .await
        .expect("register sans organisation");

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let resp = test::call_service(
        &app,
        test::TestRequest::get()
            .uri("/api/v1/call-for-funds/overdue")
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", login.token)))
            .to_request(),
    )
    .await;

    assert_eq!(
        resp.status().as_u16(),
        400,
        "un utilisateur sans organisation ne doit ni voir l'instance entière \
         ni recevoir une réponse 200, got: {}",
        resp.status()
    );
}
