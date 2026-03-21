// E2E tests for Journal Entry HTTP endpoints (Manual Double-Entry Accounting)
// Inspired by Noalyss (GPL-2.0+) - Belgian accounting software
// Tests cover: ACH/VEN/FIN/ODS journal types, balanced entries, access control

mod common;

use actix_web::http::header;
use actix_web::{test, App};
use chrono::Utc;
use koprogo_api::infrastructure::web::configure_routes;
use serde_json::json;
use serial_test::serial;
use uuid::Uuid;

/// Seed Belgian PCMN accounts so journal entry FK constraints pass
async fn seed_accounts_for_journal_entries(
    app_state: &actix_web::web::Data<koprogo_api::infrastructure::web::AppState>,
    org_id: Uuid,
) {
    app_state
        .account_use_cases
        .seed_belgian_pcmn(org_id)
        .await
        .expect("Failed to seed Belgian PCMN accounts");
}

// ==================== Journal Entry Tests ====================

#[actix_web::test]
#[serial]
async fn test_journal_entries_create() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = common::register_and_login(&app_state, org_id).await;
    seed_accounts_for_journal_entries(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let entry_date = Utc::now().to_rfc3339();

    let req = test::TestRequest::post()
        .uri("/api/v1/journal-entries")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "journal_type": "ACH",
            "entry_date": entry_date,
            "description": "Achat fournitures de bureau",
            "document_ref": "FA-2025-001",
            "lines": [
                {
                    "account_code": "604",
                    "debit": 100.0,
                    "credit": 0.0,
                    "description": "Fournitures de bureau"
                },
                {
                    "account_code": "440",
                    "debit": 0.0,
                    "credit": 100.0,
                    "description": "Fournisseur X"
                }
            ]
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.status(),
        201,
        "Should create balanced journal entry successfully"
    );

    let entry: serde_json::Value = test::read_body_json(resp).await;
    assert!(entry.get("id").is_some(), "Created entry should have an ID");
    assert_eq!(entry["journal_type"], "ACH", "Journal type should be ACH");
}

#[actix_web::test]
#[serial]
async fn test_journal_entries_get() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = common::register_and_login(&app_state, org_id).await;
    seed_accounts_for_journal_entries(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let entry_date = Utc::now().to_rfc3339();

    // Create a journal entry first
    let create_req = test::TestRequest::post()
        .uri("/api/v1/journal-entries")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "journal_type": "FIN",
            "entry_date": entry_date,
            "description": "Paiement fournisseur",
            "lines": [
                {
                    "account_code": "440",
                    "debit": 250.0,
                    "credit": 0.0,
                    "description": "Apurement compte fournisseur"
                },
                {
                    "account_code": "550",
                    "debit": 0.0,
                    "credit": 250.0,
                    "description": "Banque"
                }
            ]
        }))
        .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    assert_eq!(create_resp.status(), 201);
    let created: serde_json::Value = test::read_body_json(create_resp).await;
    let entry_id = created["id"].as_str().unwrap();

    // Get the journal entry by ID
    let get_req = test::TestRequest::get()
        .uri(&format!("/api/v1/journal-entries/{}", entry_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let get_resp = test::call_service(&app, get_req).await;
    assert_eq!(get_resp.status(), 200, "Should get journal entry by ID");

    let entry: serde_json::Value = test::read_body_json(get_resp).await;
    // Entry may be returned as { entry: {...}, lines: [...] } or flat
    let has_id =
        entry.get("id").is_some() || entry.get("entry").and_then(|e| e.get("id")).is_some();
    assert!(has_id, "Response should contain the entry ID");
}

#[actix_web::test]
#[serial]
async fn test_journal_entries_list() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = common::register_and_login(&app_state, org_id).await;
    seed_accounts_for_journal_entries(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let entry_date = Utc::now().to_rfc3339();

    // Create a journal entry
    let create_req = test::TestRequest::post()
        .uri("/api/v1/journal-entries")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "journal_type": "ODS",
            "entry_date": entry_date,
            "description": "Opération diverse",
            "lines": [
                {
                    "account_code": "499",
                    "debit": 50.0,
                    "credit": 0.0,
                    "description": "Débit divers"
                },
                {
                    "account_code": "700",
                    "debit": 0.0,
                    "credit": 50.0,
                    "description": "Produit divers"
                }
            ]
        }))
        .to_request();
    let _ = test::call_service(&app, create_req).await;

    // List journal entries
    let list_req = test::TestRequest::get()
        .uri("/api/v1/journal-entries")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let list_resp = test::call_service(&app, list_req).await;
    assert_eq!(
        list_resp.status(),
        200,
        "Should list journal entries successfully"
    );

    let body: serde_json::Value = test::read_body_json(list_resp).await;
    // Accept either paginated { entries: [...] } or plain array
    let has_entries =
        body.is_array() || body.get("entries").is_some() || body.get("data").is_some();
    assert!(has_entries, "Response should contain journal entries");
}

#[actix_web::test]
#[serial]
async fn test_journal_entries_delete() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = common::register_and_login(&app_state, org_id).await;
    seed_accounts_for_journal_entries(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let entry_date = Utc::now().to_rfc3339();

    // Create a journal entry to delete
    let create_req = test::TestRequest::post()
        .uri("/api/v1/journal-entries")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "journal_type": "VEN",
            "entry_date": entry_date,
            "description": "Vente à supprimer",
            "lines": [
                {
                    "account_code": "411",
                    "debit": 121.0,
                    "credit": 0.0,
                    "description": "Client"
                },
                {
                    "account_code": "700",
                    "debit": 0.0,
                    "credit": 100.0,
                    "description": "Vente de services"
                },
                {
                    "account_code": "451",
                    "debit": 0.0,
                    "credit": 21.0,
                    "description": "TVA collectée 21%"
                }
            ]
        }))
        .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    assert_eq!(create_resp.status(), 201);
    let created: serde_json::Value = test::read_body_json(create_resp).await;
    let entry_id = created["id"].as_str().unwrap();

    // Delete the journal entry
    let delete_req = test::TestRequest::delete()
        .uri(&format!("/api/v1/journal-entries/{}", entry_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let delete_resp = test::call_service(&app, delete_req).await;
    let status = delete_resp.status().as_u16();
    assert!(
        status == 204 || status == 200,
        "Should delete journal entry with 204 or 200, got {}",
        status
    );
}

#[actix_web::test]
#[serial]
async fn test_journal_entries_unbalanced_fails() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let token = common::register_and_login(&app_state, org_id).await;
    seed_accounts_for_journal_entries(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let entry_date = Utc::now().to_rfc3339();

    // Attempt to create an unbalanced entry: debit 100, credit 90 (mismatch!)
    let req = test::TestRequest::post()
        .uri("/api/v1/journal-entries")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "journal_type": "ACH",
            "entry_date": entry_date,
            "description": "Unbalanced entry attempt",
            "lines": [
                {
                    "account_code": "604",
                    "debit": 100.0,
                    "credit": 0.0,
                    "description": "Débit"
                },
                {
                    "account_code": "440",
                    "debit": 0.0,
                    "credit": 90.0,
                    "description": "Crédit incomplet"
                }
            ]
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    let status = resp.status().as_u16();
    assert!(
        status == 400 || status == 422 || status == 500,
        "Unbalanced journal entry should be rejected (400/422/500), got {}",
        status
    );
}

#[actix_web::test]
#[serial]
async fn test_journal_entries_unauthorized() {
    let (app_state, _container, _org_id) = common::setup_test_db().await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Request without token
    let req = test::TestRequest::get()
        .uri("/api/v1/journal-entries")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.status(),
        401,
        "Should return 401 when no auth token provided"
    );
}
