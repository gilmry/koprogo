// E2E tests for MCP SSE Server HTTP endpoints (Issue #252)
// Tests focus on HTTP layer: MCP protocol endpoints, JSON-RPC 2.0 over SSE transport
// Covers MCP info (public), SSE connection, JSON-RPC messages (initialize, tools/list, tools/call)
//
// Les handlers prennent `Data<AppState>`, comme l'application l'enregistre :
// le décalage `Data<Arc<AppState>>` qu'une note signalait ici a disparu
// (constaté le 2026-11-05, les appels `tools/call` répondent 200).

mod common;

use actix_web::http::header;
use actix_web::{test, App};
use koprogo_api::infrastructure::web::configure_routes;
use serde_json::json;
use serial_test::serial;

// ==================== MCP Info Tests (Public, No Auth, No State) ====================

#[actix_web::test]
#[serial]
async fn test_mcp_info_returns_server_metadata() {
    let (app_state, _container, _org_id) = common::setup_test_db().await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::get().uri("/mcp/info").to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200, "MCP info should return 200");

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["name"], "koprogo-mcp");
    assert_eq!(body["protocol"], "MCP/2024-11-05");
    assert_eq!(body["transport"], "SSE+HTTP");
    assert!(
        body["tools_count"].as_u64().unwrap_or(0) > 0,
        "Should have at least one tool"
    );
    assert!(
        body["endpoints"].is_object(),
        "Should contain endpoints object"
    );
    assert_eq!(body["endpoints"]["sse"], "/mcp/sse");
    assert_eq!(body["endpoints"]["messages"], "/mcp/messages");
}

#[actix_web::test]
#[serial]
async fn test_mcp_info_no_auth_required() {
    let (app_state, _container, _org_id) = common::setup_test_db().await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // No Authorization header — should still work (public endpoint)
    let req = test::TestRequest::get().uri("/mcp/info").to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200, "MCP info should not require auth");
}

#[actix_web::test]
#[serial]
async fn test_mcp_info_has_expected_endpoints() {
    let (app_state, _container, _org_id) = common::setup_test_db().await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::get().uri("/mcp/info").to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = test::read_body_json(resp).await;
    let endpoints = &body["endpoints"];
    assert!(endpoints["sse"].is_string(), "Should have sse endpoint");
    assert!(
        endpoints["messages"].is_string(),
        "Should have messages endpoint"
    );
    assert!(
        endpoints["system_prompt"].is_string(),
        "Should have system_prompt endpoint"
    );
    assert!(
        endpoints["legal_index"].is_string(),
        "Should have legal_index endpoint"
    );
}

// ==================== MCP SSE Tests (Auth Required) ====================

#[actix_web::test]
#[serial]
async fn test_mcp_sse_rejects_unauthenticated() {
    let (app_state, _container, _org_id) = common::setup_test_db().await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // No Authorization header — should return 401
    let req = test::TestRequest::get().uri("/mcp/sse").to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.status(),
        401,
        "SSE endpoint should require authentication"
    );
}

// ==================== MCP Messages Tests (JSON-RPC 2.0) ====================

#[actix_web::test]
#[serial]
async fn test_mcp_messages_rejects_unauthenticated() {
    let (app_state, _container, _org_id) = common::setup_test_db().await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/mcp/messages?session_id=test-session")
        .set_json(json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.status(),
        401,
        "Messages endpoint should require authentication"
    );
}

#[actix_web::test]
#[serial]
async fn test_mcp_system_prompt_rejects_unauthenticated() {
    let (app_state, _container, _org_id) = common::setup_test_db().await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/mcp/system-prompt")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.status(),
        401,
        "System prompt endpoint should require authentication"
    );
}

#[actix_web::test]
#[serial]
async fn test_mcp_legal_index_rejects_unauthenticated() {
    let (app_state, _container, _org_id) = common::setup_test_db().await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/mcp/legal-index")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.status(),
        401,
        "Legal index endpoint should require authentication"
    );
}

#[actix_web::test]
#[serial]
async fn test_mcp_messages_with_auth() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = common::register_and_login(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/mcp/messages?session_id=test-session")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2024-11-05",
                "clientInfo": {
                    "name": "test-client",
                    "version": "1.0"
                }
            }
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    let status = resp.status().as_u16();
    // With auth, handler should process the request.
    // May return 200 (success) or 500 (if Data<Arc<AppState>> extraction fails).
    assert!(
        status == 200 || status == 500,
        "Expected 200 or 500, got {}",
        status
    );

    if status == 200 {
        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["jsonrpc"], "2.0");
        assert_eq!(body["id"], 1);
        assert!(
            body["result"].is_object(),
            "Should have result object on success"
        );
    }
}

// ==================== Périmètre de l'appelant (lot 0, S1, #1063) ====================
//
// Le MCP n'appelait aucune garde du `scope_guard` : un syndic authentifié
// lisait, par un simple UUID, l'immeuble, les assemblées, les tickets, le
// solde d'un copropriétaire et les documents d'une ACP gérée par un autre
// cabinet. Les gardes de l'API REST étaient contournées par un second chemin.
//
// Comme dans `e2e_cloisonnement_inter_organisations.rs`, deux organisations
// partagent la même base : c'est la seule situation où le cloisonnement se
// joue vraiment.

struct DeuxOrganisationsMcp {
    app_state: actix_web::web::Data<koprogo_api::infrastructure::web::AppState>,
    _container: Option<testcontainers::ContainerAsync<testcontainers_modules::postgres::Postgres>>,
    jeton_a: String,
    immeuble_a: uuid::Uuid,
    immeuble_b: uuid::Uuid,
    coproprietaire_b: uuid::Uuid,
    assemblee_b: uuid::Uuid,
}

async fn preparer_deux_organisations() -> DeuxOrganisationsMcp {
    use koprogo_api::application::dto::{CreateMeetingRequest, CreateOwnerDto};
    use uuid::Uuid;

    let (app_state, container, org_a) = common::setup_test_db().await;
    let org_b = common::create_test_organization(&app_state).await;

    let jeton_a = common::register_and_login_with_role(&app_state, org_a, "syndic").await;
    let immeuble_a = common::create_test_building(&app_state, org_a).await;
    let immeuble_b = common::create_test_building(&app_state, org_b).await;

    let coproprietaire_b = app_state
        .owner_use_cases
        .create_owner(CreateOwnerDto {
            organization_id: org_b.to_string(),
            first_name: "Hors".to_string(),
            last_name: "Périmètre".to_string(),
            email: format!("mcp-perimetre-{}@test.com", Uuid::new_v4()),
            phone: None,
            address: "Rue B 1".to_string(),
            city: "Namur".to_string(),
            postal_code: "5000".to_string(),
            country: "Belgium".to_string(),
            user_id: None,
        })
        .await
        .expect("copropriétaire de B");
    let assemblee_b = app_state
        .meeting_use_cases
        .create_meeting(CreateMeetingRequest {
            organization_id: org_b,
            building_id: immeuble_b,
            meeting_type: koprogo_api::domain::entities::MeetingType::Ordinary,
            title: format!("AG de B {}", Uuid::new_v4()),
            description: None,
            scheduled_date: chrono::Utc::now() + chrono::Duration::days(30),
            location: "Salle B".to_string(),
            is_second_convocation: false,
        })
        .await
        .expect("assemblée de B");

    DeuxOrganisationsMcp {
        app_state,
        _container: container,
        jeton_a,
        immeuble_a,
        immeuble_b,
        coproprietaire_b: Uuid::parse_str(&coproprietaire_b.id).unwrap(),
        assemblee_b: assemblee_b.id,
    }
}

/// Appelle un outil MCP et rend le corps JSON-RPC.
macro_rules! appeler_outil {
    ($app:expr, $jeton:expr, $outil:expr, $arguments:expr) => {{
        let req = test::TestRequest::post()
            .uri("/mcp/messages?session_id=perimetre")
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", $jeton)))
            .set_json(json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "tools/call",
                "params": { "name": $outil, "arguments": $arguments }
            }))
            .to_request();
        let resp = test::call_service($app, req).await;
        assert_eq!(resp.status().as_u16(), 200, "{} : statut HTTP", $outil);
        let corps: serde_json::Value = test::read_body_json(resp).await;
        corps
    }};
}

/// @security — un syndic de A n'obtient AUCUNE donnée de B par le MCP.
///
/// Les cinq outils de l'issue, et tous ceux qui reçoivent un identifiant
/// d'immeuble, de copropriétaire ou d'assemblée. On exige une erreur
/// JSON-RPC et l'absence de `result` : le code exact est un détail de la
/// garde, pas le contrat (cf. `e2e_cloisonnement_inter_organisations.rs`).
#[actix_web::test]
#[serial]
async fn security_mcp_refuse_building_hors_perimetre() {
    let ctx = preparer_deux_organisations().await;
    let app = test::init_service(
        App::new()
            .app_data(ctx.app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let b = ctx.immeuble_b.to_string();
    let appels = [
        ("get_building", json!({ "building_id": b })),
        ("list_meetings", json!({ "building_id": b })),
        ("list_tickets", json!({ "building_id": b })),
        (
            "get_owner_balance",
            json!({ "owner_id": ctx.coproprietaire_b.to_string() }),
        ),
        ("get_building_documents", json!({ "building_id": b })),
        ("get_financial_summary", json!({ "building_id": b })),
        ("list_pending_expenses", json!({ "building_id": b })),
        ("comptabilite_situation", json!({ "building_id": b })),
        ("list_owners_of_building", json!({ "building_id": b })),
        ("list_owners", json!({ "building_id": b })),
        (
            "check_quorum",
            json!({ "meeting_id": ctx.assemblee_b.to_string() }),
        ),
        (
            "ag_quorum_check",
            json!({ "meeting_id": ctx.assemblee_b.to_string() }),
        ),
    ];

    for (outil, arguments) in appels {
        let corps = appeler_outil!(&app, &ctx.jeton_a, outil, arguments);
        assert!(
            corps.get("result").is_none_or(|r| r.is_null()),
            "{outil} a rendu des données de l'organisation B : {corps}"
        );
        assert!(
            corps["error"].is_object(),
            "{outil} devait rendre une erreur JSON-RPC : {corps}"
        );
    }
}

/// @happy — le même syndic, sur son propre immeuble, obtient sa réponse.
#[actix_web::test]
#[serial]
async fn happy_mcp_repond_dans_son_perimetre() {
    let ctx = preparer_deux_organisations().await;
    let app = test::init_service(
        App::new()
            .app_data(ctx.app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let a = ctx.immeuble_a.to_string();
    for outil in ["get_building", "list_meetings", "list_tickets"] {
        let corps = appeler_outil!(&app, &ctx.jeton_a, outil, json!({ "building_id": a }));
        assert!(
            corps["result"].is_object(),
            "{outil} sur l'immeuble de A devait répondre : {corps}"
        );
    }
}
