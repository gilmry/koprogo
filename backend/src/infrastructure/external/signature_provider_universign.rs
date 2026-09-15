//! Adaptateur Universign — prestataire tiers, repli pour les signataires
//! non-BE (Story 4.4, ADR-0014 §4).
//!
//! Deux modes :
//! - `new(...)` — appelle un vrai endpoint Universign via le contrat REST
//!   commun (`signature_provider_common::RestSignatureProvider`).
//! - `simulated(...)` — **adaptateur simulé utilisable en développement**
//!   (DoD Story 4.4) : aucun appel réseau, la signature est immédiate et
//!   déterministe. À ne jamais construire en production — sert à faire
//!   tourner un flux de signature complet en local ou en tests sans dépendre
//!   d'un vrai compte Universign.

use super::signature_provider_common::{compute_document_hmac, RestSignatureProvider, RetryPolicy};
use crate::application::ports::electronic_signature_provider::{
    ElectronicSignatureProvider, SignatureProviderError, SignatureRequest, SignatureRequestAck,
};
use crate::domain::plateforme::{QualifiedSignature, SignatureProviderKind};
use async_trait::async_trait;
use chrono::Utc;
use uuid::Uuid;

pub enum UniversignSignatureProvider {
    Http(RestSignatureProvider),
    /// Dev-only : signe immédiatement en mémoire, sans réseau.
    Simulated {
        hmac_secret: Vec<u8>,
    },
}

impl UniversignSignatureProvider {
    pub fn new(base_url: String, hmac_secret: Vec<u8>) -> Self {
        Self::Http(RestSignatureProvider::new(
            SignatureProviderKind::Universign,
            base_url,
            hmac_secret,
        ))
    }

    /// Prestataire simulé utilisable en développement (DoD Story 4.4).
    pub fn simulated(hmac_secret: Vec<u8>) -> Self {
        Self::Simulated { hmac_secret }
    }

    /// Permet aux tests d'utiliser un délai de réessai quasi nul (mode Http).
    pub fn with_retry_policy(self, retry_policy: RetryPolicy) -> Self {
        match self {
            Self::Http(inner) => Self::Http(inner.with_retry_policy(retry_policy)),
            simulated => simulated,
        }
    }
}

/// Référence simulée : encode `document_id` et `subject_user_id` — le mode
/// simulé ne persiste rien entre `request_signature` et `fetch_signature`,
/// il reconstitue l'état à partir de la référence elle-même.
fn simulated_reference(document_id: Uuid, subject_user_id: Uuid) -> String {
    format!("sim:{document_id}:{subject_user_id}")
}

fn parse_simulated_reference(reference: &str) -> Result<(Uuid, Uuid), SignatureProviderError> {
    let invalid = || {
        SignatureProviderError::InvalidResponse(
            SignatureProviderKind::Universign.to_string(),
            format!("référence simulée invalide : {reference}"),
        )
    };
    let rest = reference.strip_prefix("sim:").ok_or_else(invalid)?;
    let mut parts = rest.split(':');
    let document_id = parts
        .next()
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or_else(invalid)?;
    let subject_user_id = parts
        .next()
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or_else(invalid)?;
    Ok((document_id, subject_user_id))
}

#[async_trait]
impl ElectronicSignatureProvider for UniversignSignatureProvider {
    fn kind(&self) -> SignatureProviderKind {
        SignatureProviderKind::Universign
    }

    async fn request_signature(
        &self,
        request: SignatureRequest,
    ) -> Result<SignatureRequestAck, SignatureProviderError> {
        match self {
            Self::Http(inner) => inner.request_signature(request).await,
            Self::Simulated { hmac_secret } => {
                let document_hash = compute_document_hmac(hmac_secret, &request.document_bytes);
                Ok(SignatureRequestAck {
                    provider_reference: simulated_reference(
                        request.document_id,
                        request.subject_user_id,
                    ),
                    document_hash,
                })
            }
        }
    }

    async fn fetch_signature(
        &self,
        provider_reference: &str,
        expected_document_hash: &str,
    ) -> Result<QualifiedSignature, SignatureProviderError> {
        match self {
            Self::Http(inner) => {
                inner
                    .fetch_signature(provider_reference, expected_document_hash)
                    .await
            }
            Self::Simulated { .. } => {
                let (document_id, subject_user_id) = parse_simulated_reference(provider_reference)?;
                QualifiedSignature::new(
                    SignatureProviderKind::Universign,
                    subject_user_id,
                    document_id,
                    expected_document_hash.to_string(),
                    provider_reference.to_string(),
                    Utc::now(),
                )
                .map_err(|e| {
                    SignatureProviderError::InvalidResponse(
                        SignatureProviderKind::Universign.to_string(),
                        e.to_string(),
                    )
                })
            }
        }
    }
}
