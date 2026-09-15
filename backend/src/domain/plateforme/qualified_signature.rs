//! `QualifiedSignature` — signature électronique qualifiée (Story 4.4, ADR-0014).
//!
//! Trois prestataires (eID belge/FAS, itsme, Universign) parlent chacun leur
//! protocole ; ce que le domaine connaît, c'est le résultat : un document,
//! un signataire, un prestataire, un instant, un condensat vérifié.
//!
//! Le condensat (`document_hash`) est un HMAC-SHA256 hex (64 caractères) —
//! calculé avant l'envoi au prestataire et vérifié à la réception
//! (`application::ports::electronic_signature_provider`, AC @security). Le
//! domaine ne calcule pas le HMAC lui-même : la clé est un secret
//! d'adaptateur (cf. `infrastructure::external::signature_provider_common`),
//! pas une donnée métier. Il valide seulement la FORME du condensat qu'on
//! lui présente.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

/// Le prestataire de signature électronique sollicité.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SignatureProviderKind {
    /// eID belge — carte d'identité électronique (FAS, Federal Authentication
    /// Service). Prestataire par défaut v0.1.0, gratuit (ADR-0014 §4).
    Eid,
    /// itsme — signature via l'application mobile belge.
    Itsme,
    /// Universign — prestataire tiers, repli pour les signataires non-BE.
    Universign,
}

impl SignatureProviderKind {
    /// Encodage stable (config, logs, DB future).
    pub fn as_db_str(&self) -> &'static str {
        match self {
            Self::Eid => "eid",
            Self::Itsme => "itsme",
            Self::Universign => "universign",
        }
    }

    pub fn from_db_str(s: &str) -> Option<Self> {
        match s {
            "eid" => Some(Self::Eid),
            "itsme" => Some(Self::Itsme),
            "universign" => Some(Self::Universign),
            _ => None,
        }
    }
}

impl std::fmt::Display for SignatureProviderKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_db_str())
    }
}

/// Sélectionne le prestataire à solliciter : la préférence du cabinet, sauf
/// pour un signataire non-belge — l'eID et itsme supposent une identité
/// belge (FAS/RRN), Universign ne le suppose pas (ADR-0014 §4).
///
/// La préférence est celle du cabinet, la contrainte est celle du
/// signataire : c'est leur rencontre qu'il faut évaluer, pas l'une ou
/// l'autre isolément.
pub fn select_signature_provider(
    cabinet_preference: SignatureProviderKind,
    subject_is_belgian: bool,
) -> SignatureProviderKind {
    if subject_is_belgian {
        cabinet_preference
    } else {
        SignatureProviderKind::Universign
    }
}

/// Erreurs de construction d'une `QualifiedSignature`.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum QualifiedSignatureError {
    #[error(
        "Le condensat du document doit être un HMAC-SHA256 hexadécimal (64 caractères), \
         reçu {0} caractère(s)"
    )]
    DocumentHashInvalidLength(usize),

    #[error("Le condensat du document contient des caractères non hexadécimaux")]
    DocumentHashNotHex,

    #[error("La référence prestataire ne peut pas être vide")]
    ProviderReferenceEmpty,
}

/// Une signature électronique qualifiée, reçue d'un prestataire et prête à
/// être auditée par le use-case appelant.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QualifiedSignature {
    pub id: Uuid,
    pub provider: SignatureProviderKind,
    pub subject_user_id: Uuid,
    pub document_id: Uuid,
    /// HMAC-SHA256 hex (64 caractères) — vérifié contre le condensat calculé
    /// avant l'envoi (AC @security, Story 4.4).
    pub document_hash: String,
    /// Identifiant attribué par le prestataire (utilisé pour `fetch_signature`).
    pub provider_reference: String,
    pub signed_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

impl QualifiedSignature {
    pub fn new(
        provider: SignatureProviderKind,
        subject_user_id: Uuid,
        document_id: Uuid,
        document_hash: String,
        provider_reference: String,
        signed_at: DateTime<Utc>,
    ) -> Result<Self, QualifiedSignatureError> {
        if document_hash.len() != 64 {
            return Err(QualifiedSignatureError::DocumentHashInvalidLength(
                document_hash.len(),
            ));
        }
        if !document_hash.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(QualifiedSignatureError::DocumentHashNotHex);
        }
        let provider_reference = provider_reference.trim().to_string();
        if provider_reference.is_empty() {
            return Err(QualifiedSignatureError::ProviderReferenceEmpty);
        }

        Ok(Self {
            id: Uuid::new_v4(),
            provider,
            subject_user_id,
            document_id,
            document_hash,
            provider_reference,
            signed_at,
            created_at: Utc::now(),
        })
    }
}

// ============================================================================
// Tests — taxonomie 4 catégories obligatoire (CRITICAL.md #3)
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_hash() -> String {
        "a".repeat(64)
    }

    // ------------------------------------------------------------------
    // @happy
    // ------------------------------------------------------------------

    #[test]
    fn happy_new_builds_a_valid_qualified_signature() {
        let sig = QualifiedSignature::new(
            SignatureProviderKind::Eid,
            Uuid::new_v4(),
            Uuid::new_v4(),
            valid_hash(),
            "ref-123".to_string(),
            Utc::now(),
        )
        .unwrap();
        assert_eq!(sig.provider, SignatureProviderKind::Eid);
        assert_eq!(sig.provider_reference, "ref-123");
    }

    #[test]
    fn happy_belgian_subject_gets_cabinet_preference() {
        let selected = select_signature_provider(SignatureProviderKind::Itsme, true);
        assert_eq!(selected, SignatureProviderKind::Itsme);
    }

    #[test]
    fn happy_universign_preference_is_never_overridden() {
        let selected = select_signature_provider(SignatureProviderKind::Universign, true);
        assert_eq!(selected, SignatureProviderKind::Universign);
    }

    // ------------------------------------------------------------------
    // @edge — la préférence du cabinet rencontre la contrainte du signataire
    // ------------------------------------------------------------------

    #[test]
    fn edge_non_belgian_subject_falls_back_to_universign_even_if_itsme_preferred() {
        let selected = select_signature_provider(SignatureProviderKind::Itsme, false);
        assert_eq!(selected, SignatureProviderKind::Universign);
    }

    #[test]
    fn edge_non_belgian_subject_falls_back_to_universign_even_if_eid_preferred() {
        // eID suppose aussi une identité belge (FAS/RRN) : le repli s'applique
        // pareillement, pas seulement pour itsme.
        let selected = select_signature_provider(SignatureProviderKind::Eid, false);
        assert_eq!(selected, SignatureProviderKind::Universign);
    }

    #[test]
    fn edge_document_hash_must_be_exactly_64_chars() {
        let err = QualifiedSignature::new(
            SignatureProviderKind::Eid,
            Uuid::new_v4(),
            Uuid::new_v4(),
            "a".repeat(63),
            "ref".to_string(),
            Utc::now(),
        )
        .unwrap_err();
        assert_eq!(err, QualifiedSignatureError::DocumentHashInvalidLength(63));
    }

    // ------------------------------------------------------------------
    // @security
    // ------------------------------------------------------------------

    #[test]
    fn security_document_hash_must_be_hex_only() {
        // Un condensat qui contient des caractères non hexadécimaux ne peut
        // pas être un HMAC-SHA256 valide — le rejeter ici évite de persister
        // une "preuve" d'intégrité qui n'en est pas une.
        let err = QualifiedSignature::new(
            SignatureProviderKind::Itsme,
            Uuid::new_v4(),
            Uuid::new_v4(),
            "z".repeat(64),
            "ref".to_string(),
            Utc::now(),
        )
        .unwrap_err();
        assert_eq!(err, QualifiedSignatureError::DocumentHashNotHex);
    }

    // ------------------------------------------------------------------
    // @negative — défaillance correcte (erreur typée, pas de panic)
    // ------------------------------------------------------------------

    #[test]
    fn negative_empty_provider_reference_is_rejected() {
        let err = QualifiedSignature::new(
            SignatureProviderKind::Universign,
            Uuid::new_v4(),
            Uuid::new_v4(),
            valid_hash(),
            "   ".to_string(),
            Utc::now(),
        )
        .unwrap_err();
        assert_eq!(err, QualifiedSignatureError::ProviderReferenceEmpty);
    }

    #[test]
    fn negative_from_db_str_unknown_value_returns_none_not_panic() {
        assert_eq!(SignatureProviderKind::from_db_str("docusign"), None);
    }
}
