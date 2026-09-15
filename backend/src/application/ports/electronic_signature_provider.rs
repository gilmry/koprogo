//! Port `ElectronicSignatureProvider` — Story 4.4 (ADR-0014).
//!
//! Le domaine connaît la signature électronique qualifiée
//! (`domain::plateforme::QualifiedSignature`) ; il ne connaît ni FAS
//! (eID belge), ni itsme, ni Universign. Ce port est le seul point de
//! contact entre l'application et ces trois prestataires — trois
//! aujourd'hui, un quatrième probable demain (ADR-0014 §4).
//!
//! # Contrat en deux temps
//!
//! `request_signature` envoie le document et rend une référence prestataire
//! plus le condensat HMAC-SHA256 **calculé avant l'envoi** (AC @security).
//! `fetch_signature` interroge le prestataire et **vérifie** que le
//! condensat qu'il rend correspond à celui calculé à l'aller — sans cette
//! vérification, on signerait la parole du prestataire sur ce qu'il a reçu,
//! et non le document envoyé.
//!
//! # Persistance de l'audit
//!
//! Ce port ne persiste rien lui-même : chaque `QualifiedSignature` rendue
//! par `fetch_signature`, ainsi que chaque échec typé, sont destinés à
//! `AuditLogRepository` (cf. `application::ports::audit_log_repository`) —
//! câblage laissé au use-case appelant. Hors périmètre de cette story (Files
//! listés dans l'issue #579 : port + 3 adapters + tests d'intégration).

use crate::domain::plateforme::{QualifiedSignature, SignatureProviderKind};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

/// Ce qui part vers le prestataire.
#[derive(Debug, Clone)]
pub struct SignatureRequest {
    pub document_id: Uuid,
    pub document_bytes: Vec<u8>,
    pub subject_user_id: Uuid,
}

/// Accusé de réception d'une demande de signature.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignatureRequestAck {
    /// Identifiant attribué par le prestataire — à repasser à `fetch_signature`.
    pub provider_reference: String,
    /// HMAC-SHA256 hex du document, calculé **avant** l'envoi (AC @security).
    pub document_hash: String,
}

/// Port hexagonal vers un prestataire de signature électronique qualifiée.
#[async_trait]
pub trait ElectronicSignatureProvider: Send + Sync {
    fn kind(&self) -> SignatureProviderKind;

    /// Envoie le document au prestataire. Le condensat est calculé et
    /// renvoyé AVANT l'appel réseau (AC @security).
    async fn request_signature(
        &self,
        request: SignatureRequest,
    ) -> Result<SignatureRequestAck, SignatureProviderError>;

    /// Interroge le prestataire pour une référence donnée.
    /// `expected_document_hash` est celui rendu par `request_signature` : le
    /// résultat est rejeté (`HashMismatch`) si le prestataire ne confirme pas
    /// le même condensat.
    async fn fetch_signature(
        &self,
        provider_reference: &str,
        expected_document_hash: &str,
    ) -> Result<QualifiedSignature, SignatureProviderError>;
}

/// Erreurs typées du port — jamais de `Result<_, String>` (CRITICAL.md #4).
/// Cluster coord Story 4.4 : `NEW → AppError natif` — le bridge vers
/// `AppError` se fera au use-case appelant, hors périmètre de cette story.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum SignatureProviderError {
    #[error("Le prestataire de signature ({0}) a dépassé le délai imparti")]
    Timeout(String),

    #[error("Le prestataire de signature ({0}) est indisponible")]
    Unavailable(String),

    #[error("Erreur HTTP du prestataire de signature ({0}) : {1}")]
    Http(String, String),

    #[error(
        "Condensat non vérifié : attendu {expected}, reçu {actual} — le prestataire n'a pas \
         signé le document envoyé"
    )]
    HashMismatch { expected: String, actual: String },

    #[error("Réponse du prestataire de signature ({0}) invalide : {1}")]
    InvalidResponse(String, String),

    #[error("Référence de signature {0} inconnue chez le prestataire")]
    NotFound(String),
}

impl SignatureProviderError {
    /// Une nouvelle tentative a-t-elle un sens ? Seuls les échecs
    /// transitoires (délai, indisponibilité) sont rejoués — une réponse 4xx
    /// ou un condensat qui ne correspond pas ne se résolvent pas en
    /// réessayant (AC @negative).
    pub fn is_retryable(&self) -> bool {
        matches!(self, Self::Timeout(_) | Self::Unavailable(_))
    }
}

/// Détient les adaptateurs enregistrés, un par `SignatureProviderKind`, et
/// résout lequel solliciter.
///
/// Compose `domain::plateforme::select_signature_provider` (préférence
/// cabinet + repli non-BE) avec les instances d'adaptateurs concrètes.
pub struct ElectronicSignatureProviderRegistry {
    providers: HashMap<SignatureProviderKind, Arc<dyn ElectronicSignatureProvider>>,
}

impl ElectronicSignatureProviderRegistry {
    pub fn new(providers: Vec<Arc<dyn ElectronicSignatureProvider>>) -> Self {
        let providers = providers.into_iter().map(|p| (p.kind(), p)).collect();
        Self { providers }
    }

    /// Résout le prestataire à solliciter (préférence cabinet + repli
    /// non-BE), puis renvoie l'adaptateur correspondant.
    ///
    /// # Errors
    /// `SignatureProviderError::NotFound` si l'adaptateur résolu n'a pas été
    /// enregistré (mauvaise configuration au démarrage) — erreur typée,
    /// jamais un panic sur `HashMap::get().unwrap()`.
    pub fn resolve(
        &self,
        cabinet_preference: SignatureProviderKind,
        subject_is_belgian: bool,
    ) -> Result<Arc<dyn ElectronicSignatureProvider>, SignatureProviderError> {
        let kind = crate::domain::plateforme::select_signature_provider(
            cabinet_preference,
            subject_is_belgian,
        );
        self.providers
            .get(&kind)
            .cloned()
            .ok_or_else(|| SignatureProviderError::NotFound(kind.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct StubProvider(SignatureProviderKind);

    #[async_trait]
    impl ElectronicSignatureProvider for StubProvider {
        fn kind(&self) -> SignatureProviderKind {
            self.0
        }

        async fn request_signature(
            &self,
            _request: SignatureRequest,
        ) -> Result<SignatureRequestAck, SignatureProviderError> {
            unimplemented!("not exercised by registry resolution tests")
        }

        async fn fetch_signature(
            &self,
            _provider_reference: &str,
            _expected_document_hash: &str,
        ) -> Result<QualifiedSignature, SignatureProviderError> {
            unimplemented!("not exercised by registry resolution tests")
        }
    }

    fn full_registry() -> ElectronicSignatureProviderRegistry {
        ElectronicSignatureProviderRegistry::new(vec![
            Arc::new(StubProvider(SignatureProviderKind::Eid)),
            Arc::new(StubProvider(SignatureProviderKind::Itsme)),
            Arc::new(StubProvider(SignatureProviderKind::Universign)),
        ])
    }

    // @happy
    #[test]
    fn happy_resolve_returns_cabinet_preference_for_belgian_subject() {
        let registry = full_registry();
        let provider = registry
            .resolve(SignatureProviderKind::Itsme, true)
            .unwrap();
        assert_eq!(provider.kind(), SignatureProviderKind::Itsme);
    }

    // @edge — préférence itsme, signataire non-BE → repli Universign
    #[test]
    fn edge_resolve_falls_back_to_universign_for_non_belgian_subject() {
        let registry = full_registry();
        let provider = registry
            .resolve(SignatureProviderKind::Itsme, false)
            .unwrap();
        assert_eq!(provider.kind(), SignatureProviderKind::Universign);
    }

    // @negative — adaptateur résolu mais jamais enregistré
    #[test]
    fn negative_resolve_returns_typed_not_found_when_adapter_missing() {
        let registry = ElectronicSignatureProviderRegistry::new(vec![Arc::new(StubProvider(
            SignatureProviderKind::Eid,
        ))]);
        let err = registry
            .resolve(SignatureProviderKind::Itsme, false)
            .unwrap_err();
        assert_eq!(
            err,
            SignatureProviderError::NotFound(SignatureProviderKind::Universign.to_string())
        );
    }

    // @security — un échec de condensat n'est jamais retenté
    #[test]
    fn security_hash_mismatch_is_not_retryable() {
        let err = SignatureProviderError::HashMismatch {
            expected: "a".repeat(64),
            actual: "b".repeat(64),
        };
        assert!(!err.is_retryable());
    }

    #[test]
    fn security_timeout_and_unavailable_are_retryable() {
        assert!(SignatureProviderError::Timeout("eid".into()).is_retryable());
        assert!(SignatureProviderError::Unavailable("itsme".into()).is_retryable());
    }
}
