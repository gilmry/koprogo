//! Adaptateur itsme — signature via l'application mobile belge (Story 4.4).
//!
//! Prestataire payant (~0,30-0,50 €/signature, ADR-0014 §4), utilisé pour le
//! vote à distance et les signataires non-résidents équipés de l'app. Délègue
//! au contrat REST commun (`signature_provider_common::RestSignatureProvider`).

use super::signature_provider_common::{RestSignatureProvider, RetryPolicy};
use crate::application::ports::electronic_signature_provider::{
    ElectronicSignatureProvider, SignatureProviderError, SignatureRequest, SignatureRequestAck,
};
use crate::domain::plateforme::{QualifiedSignature, SignatureProviderKind};
use async_trait::async_trait;

pub struct ItsmeSignatureProvider(RestSignatureProvider);

impl ItsmeSignatureProvider {
    pub fn new(base_url: String, hmac_secret: Vec<u8>) -> Self {
        Self(RestSignatureProvider::new(
            SignatureProviderKind::Itsme,
            base_url,
            hmac_secret,
        ))
    }

    /// Permet aux tests d'utiliser un délai de réessai quasi nul.
    pub fn with_retry_policy(self, retry_policy: RetryPolicy) -> Self {
        Self(self.0.with_retry_policy(retry_policy))
    }

    /// Permet aux tests de déclencher un vrai `Timeout` réseau.
    pub fn with_http_timeout(self, timeout: std::time::Duration) -> Self {
        Self(self.0.with_http_timeout(timeout))
    }
}

#[async_trait]
impl ElectronicSignatureProvider for ItsmeSignatureProvider {
    fn kind(&self) -> SignatureProviderKind {
        self.0.kind()
    }

    async fn request_signature(
        &self,
        request: SignatureRequest,
    ) -> Result<SignatureRequestAck, SignatureProviderError> {
        self.0.request_signature(request).await
    }

    async fn fetch_signature(
        &self,
        provider_reference: &str,
        expected_document_hash: &str,
    ) -> Result<QualifiedSignature, SignatureProviderError> {
        self.0
            .fetch_signature(provider_reference, expected_document_hash)
            .await
    }
}
