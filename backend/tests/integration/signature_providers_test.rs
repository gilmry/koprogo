//! Tests d'intégration `ElectronicSignatureProvider` — Story 4.4 (ADR-0014).
//!
//! eID belge et itsme sont exercés contre un serveur HTTP simulé
//! (`wiremock`) : les trois prestataires partagent le même contrat REST
//! (`signature_provider_common::RestSignatureProvider`), un seul mock suffit
//! à couvrir les deux. Universign est exercé en mode **simulé**
//! (`UniversignSignatureProvider::simulated`, sans réseau) — c'est
//! explicitement le mode "mock en dev" demandé par la story.
//!
//! Pattern : `backend/tests/integration/acp_test.rs`.

use chrono::Utc;
use koprogo_api::application::ports::{
    ElectronicSignatureProvider, ElectronicSignatureProviderRegistry, SignatureProviderError,
    SignatureRequest,
};
use koprogo_api::domain::plateforme::SignatureProviderKind;
use koprogo_api::infrastructure::external::{
    compute_document_hmac, EidSignatureProvider, ItsmeSignatureProvider, RetryPolicy,
    UniversignSignatureProvider,
};
use serde_json::json;
use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;
use wiremock::matchers::{body_json, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn fast_retry_policy() -> RetryPolicy {
    // 3 tentatives, délai quasi nul : le test vérifie le COMPTE de
    // tentatives, pas la durée réelle du backoff exponentiel.
    RetryPolicy {
        max_attempts: 3,
        base_delay: Duration::from_millis(1),
    }
}

// ============================================================================
// @happy — chemin nominal end-to-end pour les trois prestataires
// ============================================================================

#[tokio::test]
async fn happy_eid_signature_request_and_fetch_returns_qualified_signature() {
    let secret = b"eid-secret".to_vec();
    let document_bytes = b"reglement-copropriete-v3.pdf".to_vec();
    let document_id = Uuid::new_v4();
    let subject_user_id = Uuid::new_v4();
    let expected_hash = compute_document_hmac(&secret, &document_bytes);

    let mock_server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/signatures"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"reference": "eid-ref-1"})))
        .mount(&mock_server)
        .await;
    Mock::given(method("GET"))
        .and(path("/signatures/eid-ref-1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "status": "completed",
            "document_id": document_id,
            "subject_user_id": subject_user_id,
            "document_hash": expected_hash,
            "signed_at": Utc::now().to_rfc3339(),
        })))
        .mount(&mock_server)
        .await;

    let provider = EidSignatureProvider::new(mock_server.uri(), secret);
    let ack = provider
        .request_signature(SignatureRequest {
            document_id,
            document_bytes,
            subject_user_id,
        })
        .await
        .expect("request_signature should succeed");
    assert_eq!(ack.provider_reference, "eid-ref-1");
    assert_eq!(ack.document_hash, expected_hash);

    let signature = provider
        .fetch_signature(&ack.provider_reference, &ack.document_hash)
        .await
        .expect("fetch_signature should succeed");
    assert_eq!(signature.provider, SignatureProviderKind::Eid);
    assert_eq!(signature.document_id, document_id);
    assert_eq!(signature.subject_user_id, subject_user_id);
    assert_eq!(signature.document_hash, expected_hash);
}

#[tokio::test]
async fn happy_itsme_signature_request_and_fetch_returns_qualified_signature() {
    let secret = b"itsme-secret".to_vec();
    let document_bytes = b"proces-verbal-ag-2026.pdf".to_vec();
    let document_id = Uuid::new_v4();
    let subject_user_id = Uuid::new_v4();
    let expected_hash = compute_document_hmac(&secret, &document_bytes);

    let mock_server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/signatures"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"reference": "itsme-ref-1"})))
        .mount(&mock_server)
        .await;
    Mock::given(method("GET"))
        .and(path("/signatures/itsme-ref-1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "status": "completed",
            "document_id": document_id,
            "subject_user_id": subject_user_id,
            "document_hash": expected_hash,
            "signed_at": Utc::now().to_rfc3339(),
        })))
        .mount(&mock_server)
        .await;

    let provider = ItsmeSignatureProvider::new(mock_server.uri(), secret);
    let ack = provider
        .request_signature(SignatureRequest {
            document_id,
            document_bytes,
            subject_user_id,
        })
        .await
        .expect("request_signature should succeed");

    let signature = provider
        .fetch_signature(&ack.provider_reference, &ack.document_hash)
        .await
        .expect("fetch_signature should succeed");
    assert_eq!(signature.provider, SignatureProviderKind::Itsme);
    assert_eq!(signature.document_id, document_id);
}

#[tokio::test]
async fn happy_universign_simulated_signature_request_and_fetch_returns_qualified_signature() {
    // "mock en dev" (story 4.4) : aucun réseau, aucun wiremock.
    let secret = b"universign-secret".to_vec();
    let document_bytes = b"cahier-des-charges-toiture.pdf".to_vec();
    let document_id = Uuid::new_v4();
    let subject_user_id = Uuid::new_v4();

    let provider = UniversignSignatureProvider::simulated(secret);
    let ack = provider
        .request_signature(SignatureRequest {
            document_id,
            document_bytes,
            subject_user_id,
        })
        .await
        .expect("simulated request_signature should succeed");

    let signature = provider
        .fetch_signature(&ack.provider_reference, &ack.document_hash)
        .await
        .expect("simulated fetch_signature should succeed");
    assert_eq!(signature.provider, SignatureProviderKind::Universign);
    assert_eq!(signature.document_id, document_id);
    assert_eq!(signature.subject_user_id, subject_user_id);
    assert_eq!(signature.document_hash, ack.document_hash);
}

// ============================================================================
// @edge — préférence cabinet = itsme, signataire non-BE → repli Universign
// ============================================================================

#[tokio::test]
async fn edge_non_belgian_subject_is_routed_to_universign_despite_itsme_preference() {
    let registry = ElectronicSignatureProviderRegistry::new(vec![
        Arc::new(EidSignatureProvider::new(
            "http://eid.invalid".to_string(),
            b"eid".to_vec(),
        )) as Arc<dyn ElectronicSignatureProvider>,
        Arc::new(ItsmeSignatureProvider::new(
            "http://itsme.invalid".to_string(),
            b"itsme".to_vec(),
        )),
        Arc::new(UniversignSignatureProvider::simulated(
            b"universign".to_vec(),
        )),
    ]);

    // Le cabinet préfère itsme — mais le signataire n'est pas belge.
    let provider = registry
        .resolve(SignatureProviderKind::Itsme, false)
        .expect("registry should resolve a fallback provider");
    assert_eq!(provider.kind(), SignatureProviderKind::Universign);

    // Le repli est vérifié bout en bout : le prestataire résolu signe
    // réellement, ce n'est pas seulement `kind()` qui change.
    let document_id = Uuid::new_v4();
    let subject_user_id = Uuid::new_v4();
    let ack = provider
        .request_signature(SignatureRequest {
            document_id,
            document_bytes: b"reglement-ordre-interieur.pdf".to_vec(),
            subject_user_id,
        })
        .await
        .expect("fallback provider should accept the request");
    let signature = provider
        .fetch_signature(&ack.provider_reference, &ack.document_hash)
        .await
        .expect("fallback provider should complete the signature");
    assert_eq!(signature.provider, SignatureProviderKind::Universign);
}

// ============================================================================
// @security — condensat HMAC-SHA256 calculé avant l'envoi, vérifié à la
// réception
// ============================================================================

#[tokio::test]
async fn security_document_hash_is_computed_before_sending_to_the_provider() {
    let secret = b"eid-secret".to_vec();
    let document_bytes = b"acte-de-base.pdf".to_vec();
    let document_id = Uuid::new_v4();
    let subject_user_id = Uuid::new_v4();
    // Calculé indépendamment, AVANT tout appel à l'adaptateur : c'est ce que
    // l'adaptateur doit reproduire et envoyer dans le corps de la requête.
    let expected_hash = compute_document_hmac(&secret, &document_bytes);

    let mock_server = MockServer::start().await;
    // Le matcher `body_json` exige une correspondance exacte : si
    // l'adaptateur envoyait un condensat différent (calculé après l'envoi,
    // ou pas calculé du tout), aucun mock ne répondrait et le test échouerait
    // avec une 404 wiremock — pas un faux positif silencieux.
    Mock::given(method("POST"))
        .and(path("/signatures"))
        .and(body_json(json!({
            "document_id": document_id,
            "document_hash": expected_hash,
            "subject_user_id": subject_user_id,
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"reference": "sec-ref"})))
        .mount(&mock_server)
        .await;

    let provider = EidSignatureProvider::new(mock_server.uri(), secret);
    let ack = provider
        .request_signature(SignatureRequest {
            document_id,
            document_bytes,
            subject_user_id,
        })
        .await
        .expect("request body must match the pre-computed HMAC or wiremock 404s");
    assert_eq!(ack.document_hash, expected_hash);
}

#[tokio::test]
async fn security_hash_mismatch_at_fetch_is_rejected() {
    let secret = b"eid-secret".to_vec();
    let document_bytes = b"proces-verbal.pdf".to_vec();
    let document_id = Uuid::new_v4();
    let subject_user_id = Uuid::new_v4();
    let expected_hash = compute_document_hmac(&secret, &document_bytes);
    // Un condensat différent, mais toujours 64 caractères hexadécimaux — le
    // prestataire prétend avoir signé un document, mais pas CE document.
    let tampered_hash = "b".repeat(64);
    assert_ne!(expected_hash, tampered_hash);

    let mock_server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/signatures"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"reference": "hm-ref"})))
        .mount(&mock_server)
        .await;
    Mock::given(method("GET"))
        .and(path("/signatures/hm-ref"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "status": "completed",
            "document_id": document_id,
            "subject_user_id": subject_user_id,
            "document_hash": tampered_hash,
            "signed_at": Utc::now().to_rfc3339(),
        })))
        .mount(&mock_server)
        .await;

    let provider = EidSignatureProvider::new(mock_server.uri(), secret);
    let ack = provider
        .request_signature(SignatureRequest {
            document_id,
            document_bytes,
            subject_user_id,
        })
        .await
        .expect("request_signature should still succeed");

    let err = provider
        .fetch_signature(&ack.provider_reference, &ack.document_hash)
        .await
        .expect_err("a hash mismatch must be rejected, not silently accepted");
    match err {
        SignatureProviderError::HashMismatch { expected, actual } => {
            assert_eq!(expected, expected_hash);
            assert_eq!(actual, tampered_hash);
        }
        other => panic!("expected HashMismatch, got {other:?}"),
    }
}

// ============================================================================
// @negative — prestataire en dépassement de délai → 3 tentatives en repli
// exponentiel, puis erreur typée
// ============================================================================

#[tokio::test]
async fn negative_provider_timeout_retries_three_times_then_returns_typed_error() {
    let mock_server = MockServer::start().await;
    // Chaque réponse est délibérément plus lente que le timeout HTTP du
    // client : les 3 tentatives échouent toutes par dépassement de délai.
    Mock::given(method("POST"))
        .and(path("/signatures"))
        .respond_with(ResponseTemplate::new(200).set_delay(Duration::from_millis(150)))
        .mount(&mock_server)
        .await;

    let provider = EidSignatureProvider::new(mock_server.uri(), b"eid-secret".to_vec())
        .with_retry_policy(fast_retry_policy())
        .with_http_timeout(Duration::from_millis(20));

    let err = provider
        .request_signature(SignatureRequest {
            document_id: Uuid::new_v4(),
            document_bytes: b"doc".to_vec(),
            subject_user_id: Uuid::new_v4(),
        })
        .await
        .expect_err("a provider that never answers in time must fail typed, not hang or panic");

    assert!(
        matches!(err, SignatureProviderError::Timeout(_)),
        "expected Timeout, got {err:?}"
    );

    let received = mock_server.received_requests().await.unwrap();
    assert_eq!(
        received.len(),
        3,
        "3 tentatives en repli exponentiel — AC @negative Story 4.4"
    );
}

#[tokio::test]
async fn negative_provider_unavailable_5xx_retries_three_times_then_returns_typed_error() {
    let mock_server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/signatures"))
        .respond_with(ResponseTemplate::new(503))
        .mount(&mock_server)
        .await;

    let provider = ItsmeSignatureProvider::new(mock_server.uri(), b"itsme-secret".to_vec())
        .with_retry_policy(fast_retry_policy());

    let err = provider
        .request_signature(SignatureRequest {
            document_id: Uuid::new_v4(),
            document_bytes: b"doc".to_vec(),
            subject_user_id: Uuid::new_v4(),
        })
        .await
        .expect_err("a provider stuck at 503 must fail typed after retries");

    assert!(
        matches!(err, SignatureProviderError::Unavailable(_)),
        "expected Unavailable, got {err:?}"
    );

    let received = mock_server.received_requests().await.unwrap();
    assert_eq!(received.len(), 3);
}
