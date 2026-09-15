//! Abstraction commune aux trois adaptateurs `ElectronicSignatureProvider`
//! (eID belge, itsme, Universign) — Story 4.4, ADR-0014.
//!
//! Les trois prestataires exposent — en v0.1.0, en l'absence de contrat API
//! signé avec chacun — le même contrat REST minimal : `POST
//! {base_url}/signatures` pour déposer un document, `GET
//! {base_url}/signatures/{reference}` pour en connaître le statut. Ce qui les
//! distingue est `SignatureProviderKind` (traçabilité / config), pas le
//! protocole. Le jour où un prestataire réel s'écarte de ce contrat, son
//! fichier `signature_provider_<x>.rs` cesse de déléguer à
//! `RestSignatureProvider` et implémente le trait directement — cette
//! abstraction n'est pas un mur.

use crate::application::ports::electronic_signature_provider::{
    ElectronicSignatureProvider, SignatureProviderError, SignatureRequest, SignatureRequestAck,
};
use crate::domain::plateforme::{QualifiedSignature, SignatureProviderKind};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::time::Duration;
use uuid::Uuid;

type HmacSha256 = Hmac<Sha256>;

/// HMAC-SHA256 hex (64 caractères) du document, calculé avec la clé
/// d'adaptateur — jamais envoyée au prestataire (AC @security).
pub fn compute_document_hmac(secret: &[u8], document_bytes: &[u8]) -> String {
    let mut mac =
        HmacSha256::new_from_slice(secret).expect("HMAC-SHA256 accepts a key of any length");
    mac.update(document_bytes);
    hex::encode(mac.finalize().into_bytes())
}

/// Politique de réessai : 3 tentatives par défaut, délai exponentiel.
/// Configurable pour que les tests n'attendent pas des délais réels.
#[derive(Debug, Clone, Copy)]
pub struct RetryPolicy {
    pub max_attempts: u32,
    pub base_delay: Duration,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            base_delay: Duration::from_millis(200),
        }
    }
}

impl RetryPolicy {
    /// Exécute `f` jusqu'à `max_attempts` fois, avec un délai exponentiel
    /// (`base_delay * 2^tentative`) entre chaque essai. Seules les erreurs
    /// `is_retryable()` sont rejouées — une erreur HTTP 4xx ou un
    /// `HashMismatch` échouent tout de suite : réessayer ne changerait rien
    /// (AC @negative).
    pub async fn run<T, F, Fut>(&self, mut f: F) -> Result<T, SignatureProviderError>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = Result<T, SignatureProviderError>>,
    {
        let mut attempt: u32 = 0;
        loop {
            match f().await {
                Ok(v) => return Ok(v),
                Err(e) if e.is_retryable() && attempt + 1 < self.max_attempts => {
                    tokio::time::sleep(self.base_delay * 2u32.pow(attempt)).await;
                    attempt += 1;
                }
                Err(e) => return Err(e),
            }
        }
    }
}

#[derive(Debug, Serialize)]
struct SignatureRequestBody<'a> {
    document_id: Uuid,
    document_hash: &'a str,
    subject_user_id: Uuid,
}

#[derive(Debug, Deserialize)]
struct SignatureRequestResponseBody {
    reference: String,
}

#[derive(Debug, Deserialize)]
struct SignatureStatusResponseBody {
    status: String,
    document_id: Uuid,
    subject_user_id: Uuid,
    document_hash: Option<String>,
    signed_at: Option<DateTime<Utc>>,
}

fn classify_transport_error(
    kind: SignatureProviderKind,
    e: &reqwest::Error,
) -> SignatureProviderError {
    if e.is_timeout() {
        SignatureProviderError::Timeout(kind.to_string())
    } else if e.is_connect() {
        SignatureProviderError::Unavailable(kind.to_string())
    } else {
        SignatureProviderError::Http(kind.to_string(), e.to_string())
    }
}

fn map_status_errors(
    kind: SignatureProviderKind,
    status: reqwest::StatusCode,
) -> Result<(), SignatureProviderError> {
    if status.is_server_error() {
        // 5xx est un signal d'indisponibilité transitoire, pas une réponse
        // définitive : il vaut la peine de rejouer (AC @negative).
        return Err(SignatureProviderError::Unavailable(format!(
            "{kind} (HTTP {status})"
        )));
    }
    if !status.is_success() {
        return Err(SignatureProviderError::Http(
            kind.to_string(),
            format!("HTTP {status}"),
        ));
    }
    Ok(())
}

/// Adaptateur REST générique — les trois prestataires y délèguent (cf. docs
/// de module). `kind` distingue la traçabilité / config, pas le protocole.
pub struct RestSignatureProvider {
    kind: SignatureProviderKind,
    client: reqwest::Client,
    base_url: String,
    hmac_secret: Vec<u8>,
    retry_policy: RetryPolicy,
}

impl RestSignatureProvider {
    pub fn new(kind: SignatureProviderKind, base_url: String, hmac_secret: Vec<u8>) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to build HTTP client");
        Self {
            kind,
            client,
            base_url,
            hmac_secret,
            retry_policy: RetryPolicy::default(),
        }
    }

    /// Permet aux tests d'utiliser un délai de réessai quasi nul plutôt que
    /// d'attendre des centaines de millisecondes réelles.
    pub fn with_retry_policy(mut self, retry_policy: RetryPolicy) -> Self {
        self.retry_policy = retry_policy;
        self
    }

    /// Permet aux tests de déclencher un vrai `Timeout` (via un stub HTTP
    /// qui répond après ce délai) sans attendre les 30s par défaut.
    pub fn with_http_timeout(mut self, timeout: Duration) -> Self {
        self.client = reqwest::Client::builder()
            .timeout(timeout)
            .build()
            .expect("Failed to build HTTP client");
        self
    }
}

#[async_trait]
impl ElectronicSignatureProvider for RestSignatureProvider {
    fn kind(&self) -> SignatureProviderKind {
        self.kind
    }

    async fn request_signature(
        &self,
        request: SignatureRequest,
    ) -> Result<SignatureRequestAck, SignatureProviderError> {
        let document_hash = compute_document_hmac(&self.hmac_secret, &request.document_bytes);
        let url = format!("{}/signatures", self.base_url);
        let kind = self.kind;

        let response_body = self
            .retry_policy
            .run(|| async {
                let response = self
                    .client
                    .post(&url)
                    .json(&SignatureRequestBody {
                        document_id: request.document_id,
                        document_hash: &document_hash,
                        subject_user_id: request.subject_user_id,
                    })
                    .send()
                    .await
                    .map_err(|e| classify_transport_error(kind, &e))?;

                map_status_errors(kind, response.status())?;

                response
                    .json::<SignatureRequestResponseBody>()
                    .await
                    .map_err(|e| {
                        SignatureProviderError::InvalidResponse(kind.to_string(), e.to_string())
                    })
            })
            .await?;

        Ok(SignatureRequestAck {
            provider_reference: response_body.reference,
            document_hash,
        })
    }

    async fn fetch_signature(
        &self,
        provider_reference: &str,
        expected_document_hash: &str,
    ) -> Result<QualifiedSignature, SignatureProviderError> {
        let url = format!("{}/signatures/{}", self.base_url, provider_reference);
        let kind = self.kind;

        let status = self
            .retry_policy
            .run(|| async {
                let response = self
                    .client
                    .get(&url)
                    .send()
                    .await
                    .map_err(|e| classify_transport_error(kind, &e))?;

                if response.status() == reqwest::StatusCode::NOT_FOUND {
                    return Err(SignatureProviderError::NotFound(
                        provider_reference.to_string(),
                    ));
                }
                map_status_errors(kind, response.status())?;

                response
                    .json::<SignatureStatusResponseBody>()
                    .await
                    .map_err(|e| {
                        SignatureProviderError::InvalidResponse(kind.to_string(), e.to_string())
                    })
            })
            .await?;

        if status.status != "completed" {
            return Err(SignatureProviderError::InvalidResponse(
                kind.to_string(),
                format!("statut '{}' — signature pas encore complète", status.status),
            ));
        }

        let document_hash = status.document_hash.ok_or_else(|| {
            SignatureProviderError::InvalidResponse(
                kind.to_string(),
                "signature complète sans document_hash".to_string(),
            )
        })?;
        // Vérification à la réception (AC @security) : le prestataire doit
        // confirmer le MÊME condensat que celui calculé avant l'envoi.
        if document_hash != expected_document_hash {
            return Err(SignatureProviderError::HashMismatch {
                expected: expected_document_hash.to_string(),
                actual: document_hash,
            });
        }
        let signed_at = status.signed_at.ok_or_else(|| {
            SignatureProviderError::InvalidResponse(
                kind.to_string(),
                "signature complète sans signed_at".to_string(),
            )
        })?;

        QualifiedSignature::new(
            kind,
            status.subject_user_id,
            status.document_id,
            document_hash,
            provider_reference.to_string(),
            signed_at,
        )
        .map_err(|e| SignatureProviderError::InvalidResponse(kind.to_string(), e.to_string()))
    }
}
