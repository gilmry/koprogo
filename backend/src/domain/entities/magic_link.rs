//! MagicLink — public-access tokens for contractors / external parties.
//!
//! Story 3.2 (FR6 INV-13 INV-17). A syndic can issue a magic link to give
//! temporary, scoped, single-use read access to a ticket / quote / invoice /
//! contractor-evaluation **without** requiring the recipient to create an
//! account. Typical use case: a plumber receives an SMS/email link with a
//! tokenised URL, opens it, sees the relevant ticket, and may submit a
//! response — all without an authenticated session.
//!
//! # Security model
//!
//! - The clear token is generated once (32 random bytes, base64url) and **never
//!   stored**. Only its SHA-256 digest (hex) is persisted.
//! - Single-use: `consumed_at` is set the first time the token is validated.
//!   A second validation fails (replay protection).
//! - Time-bounded: `expires_at` enforces a TTL chosen at issue time.
//! - Bound to a single scope (`scope_kind` + `scope_id`) — the link cannot be
//!   reused to access another resource.
//!
//! Mirrors the simpler `RefreshToken` pattern but replaces the `revoked: bool`
//! flag with `consumed_at: Option<DateTime<Utc>>` to enforce single-use.

use crate::application::error::AppError;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

/// What kind of resource a MagicLink grants access to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MagicLinkScopeKind {
    Ticket,
    Quote,
    Invoice,
    ContractorEvaluation,
}

impl std::fmt::Display for MagicLinkScopeKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MagicLinkScopeKind::Ticket => write!(f, "ticket"),
            MagicLinkScopeKind::Quote => write!(f, "quote"),
            MagicLinkScopeKind::Invoice => write!(f, "invoice"),
            MagicLinkScopeKind::ContractorEvaluation => write!(f, "contractor_evaluation"),
        }
    }
}

impl std::str::FromStr for MagicLinkScopeKind {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "ticket" => Ok(MagicLinkScopeKind::Ticket),
            "quote" => Ok(MagicLinkScopeKind::Quote),
            "invoice" => Ok(MagicLinkScopeKind::Invoice),
            "contractor_evaluation" => Ok(MagicLinkScopeKind::ContractorEvaluation),
            other => Err(AppError::Validation(format!(
                "Invalid magic link scope_kind: {}",
                other
            ))),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MagicLink {
    pub id: Uuid,
    /// SHA-256 hex of the clear token. The clear token is NEVER stored.
    pub token_hash: String,
    /// User to whom the link grants access (e.g. external contractor user).
    pub subject_user_id: Uuid,
    pub scope_kind: MagicLinkScopeKind,
    pub scope_id: Uuid,
    /// Syndic / superadmin user who issued the link (audit trail).
    pub issued_by: Uuid,
    pub expires_at: DateTime<Utc>,
    pub consumed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl MagicLink {
    /// Issue a new MagicLink. Returns the persisted entity AND the clear token
    /// that must be returned in the HTTP response (and never stored elsewhere).
    ///
    /// # Errors
    /// - `AppError::Validation` if `ttl <= 0`, `scope_id` is nil, or
    ///   `subject_user_id == issued_by`.
    pub fn issue(
        subject_user_id: Uuid,
        scope_kind: MagicLinkScopeKind,
        scope_id: Uuid,
        issued_by: Uuid,
        ttl: Duration,
    ) -> Result<(Self, String), AppError> {
        if ttl <= Duration::zero() {
            return Err(AppError::Validation(
                "MagicLink ttl must be strictly positive".to_string(),
            ));
        }
        if scope_id.is_nil() {
            return Err(AppError::Validation(
                "MagicLink scope_id must not be nil".to_string(),
            ));
        }
        if subject_user_id == issued_by {
            return Err(AppError::Validation(
                "MagicLink subject and issuer must differ".to_string(),
            ));
        }

        let clear_token = Self::generate_clear_token();
        let token_hash = Self::hash_token(&clear_token);
        let now = Utc::now();
        let expires_at = now + ttl;

        let entity = Self {
            id: Uuid::new_v4(),
            token_hash,
            subject_user_id,
            scope_kind,
            scope_id,
            issued_by,
            expires_at,
            consumed_at: None,
            created_at: now,
            updated_at: now,
        };

        Ok((entity, clear_token))
    }

    /// SHA-256 hex digest of a clear token. Public so the repository / handler
    /// can hash an incoming token and look it up.
    pub fn hash_token(clear_token: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(clear_token.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Generate a random 32-byte token, base64url-encoded (no padding).
    /// Produces a ~43-character URL-safe string with ~256 bits of entropy.
    fn generate_clear_token() -> String {
        let mut bytes = [0u8; 32];
        for b in bytes.iter_mut() {
            *b = rand::random::<u8>();
        }
        URL_SAFE_NO_PAD.encode(bytes)
    }

    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    pub fn is_consumed(&self) -> bool {
        self.consumed_at.is_some()
    }

    pub fn is_valid(&self) -> bool {
        !self.is_expired() && !self.is_consumed()
    }

    /// Mark this link as consumed. Idempotent — subsequent calls are no-ops
    /// in-memory but should be guarded by the repository's atomic UPDATE.
    pub fn consume(&mut self) {
        let now = Utc::now();
        if self.consumed_at.is_none() {
            self.consumed_at = Some(now);
        }
        self.updated_at = now;
    }
}

// ============================================================================
// Tests — taxonomie 4 catégories obligatoire (CRITICAL.md #3)
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_pair() -> (Uuid, Uuid, Uuid) {
        (Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4())
    }

    // ------------------------------------------------------------------------
    // @happy
    // ------------------------------------------------------------------------

    #[test]
    fn happy_issue_returns_valid_pair() {
        let (subject, issuer, scope) = fixture_pair();
        let (link, clear) = MagicLink::issue(
            subject,
            MagicLinkScopeKind::Ticket,
            scope,
            issuer,
            Duration::days(7),
        )
        .expect("issue should succeed for valid inputs");

        assert_eq!(link.subject_user_id, subject);
        assert_eq!(link.issued_by, issuer);
        assert_eq!(link.scope_id, scope);
        assert_eq!(link.scope_kind, MagicLinkScopeKind::Ticket);
        assert!(link.is_valid());
        assert!(!link.is_consumed());
        assert!(!link.is_expired());
        // Clear token is non-empty, URL-safe length around 43 chars (32 bytes base64url no-pad).
        assert!(clear.len() >= 40, "clear token too short: {}", clear.len());
    }

    #[test]
    fn happy_consume_sets_consumed_at_and_invalidates() {
        let (subject, issuer, scope) = fixture_pair();
        let (mut link, _) = MagicLink::issue(
            subject,
            MagicLinkScopeKind::Quote,
            scope,
            issuer,
            Duration::hours(1),
        )
        .unwrap();

        assert!(link.is_valid());
        link.consume();
        assert!(link.is_consumed());
        assert!(!link.is_valid());
        assert!(link.consumed_at.is_some());
    }

    #[test]
    fn happy_hash_token_is_deterministic_hex64() {
        let h1 = MagicLink::hash_token("hello");
        let h2 = MagicLink::hash_token("hello");
        assert_eq!(h1, h2);
        assert_eq!(h1.len(), 64); // sha256 hex
        assert!(h1.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn happy_scope_kind_roundtrips_via_display_and_from_str() {
        use std::str::FromStr;
        for kind in [
            MagicLinkScopeKind::Ticket,
            MagicLinkScopeKind::Quote,
            MagicLinkScopeKind::Invoice,
            MagicLinkScopeKind::ContractorEvaluation,
        ] {
            let s = kind.to_string();
            let parsed = MagicLinkScopeKind::from_str(&s).expect("roundtrip");
            assert_eq!(parsed, kind);
        }
    }

    // ------------------------------------------------------------------------
    // @edge
    // ------------------------------------------------------------------------

    #[test]
    fn edge_token_at_exact_expiry_is_invalid_after_now() {
        let (subject, issuer, scope) = fixture_pair();
        let (mut link, _) = MagicLink::issue(
            subject,
            MagicLinkScopeKind::Invoice,
            scope,
            issuer,
            Duration::seconds(1),
        )
        .unwrap();
        // Force expiry exactly to 1 second in the past.
        link.expires_at = Utc::now() - Duration::seconds(1);
        assert!(link.is_expired());
        assert!(!link.is_valid());
    }

    #[test]
    fn edge_double_consume_is_idempotent() {
        let (subject, issuer, scope) = fixture_pair();
        let (mut link, _) = MagicLink::issue(
            subject,
            MagicLinkScopeKind::Ticket,
            scope,
            issuer,
            Duration::minutes(5),
        )
        .unwrap();
        link.consume();
        let first_consumed_at = link.consumed_at.expect("first consume sets timestamp");
        link.consume();
        // second consume must not overwrite the original timestamp
        assert_eq!(link.consumed_at, Some(first_consumed_at));
    }

    #[test]
    fn edge_min_ttl_one_second_is_valid_immediately() {
        let (subject, issuer, scope) = fixture_pair();
        let (link, _) = MagicLink::issue(
            subject,
            MagicLinkScopeKind::ContractorEvaluation,
            scope,
            issuer,
            Duration::seconds(1),
        )
        .unwrap();
        assert!(link.is_valid());
    }

    // ------------------------------------------------------------------------
    // @security
    // ------------------------------------------------------------------------

    #[test]
    fn security_each_issue_returns_distinct_token_and_hash() {
        let (subject, issuer, scope) = fixture_pair();
        let (link_a, clear_a) = MagicLink::issue(
            subject,
            MagicLinkScopeKind::Ticket,
            scope,
            issuer,
            Duration::hours(1),
        )
        .unwrap();
        let (link_b, clear_b) = MagicLink::issue(
            subject,
            MagicLinkScopeKind::Ticket,
            scope,
            issuer,
            Duration::hours(1),
        )
        .unwrap();
        assert_ne!(clear_a, clear_b, "tokens must be unique per issue");
        assert_ne!(
            link_a.token_hash, link_b.token_hash,
            "hashes must differ since tokens differ"
        );
        assert_ne!(link_a.id, link_b.id);
    }

    #[test]
    fn security_clear_token_is_never_equal_to_stored_hash() {
        let (subject, issuer, scope) = fixture_pair();
        let (link, clear) = MagicLink::issue(
            subject,
            MagicLinkScopeKind::Quote,
            scope,
            issuer,
            Duration::hours(1),
        )
        .unwrap();
        assert_ne!(clear, link.token_hash);
        // And re-hashing the clear token reproduces the stored hash.
        assert_eq!(MagicLink::hash_token(&clear), link.token_hash);
    }

    #[test]
    fn security_different_inputs_produce_different_hashes() {
        let h1 = MagicLink::hash_token("token-A");
        let h2 = MagicLink::hash_token("token-B");
        assert_ne!(h1, h2);
    }

    #[test]
    fn security_expired_link_reports_invalid_even_if_not_consumed() {
        let (subject, issuer, scope) = fixture_pair();
        let (mut link, _) = MagicLink::issue(
            subject,
            MagicLinkScopeKind::Ticket,
            scope,
            issuer,
            Duration::hours(1),
        )
        .unwrap();
        link.expires_at = Utc::now() - Duration::seconds(10);
        assert!(!link.is_consumed());
        assert!(!link.is_valid());
    }

    // ------------------------------------------------------------------------
    // @negative
    // ------------------------------------------------------------------------

    #[test]
    fn negative_zero_ttl_is_rejected() {
        let (subject, issuer, scope) = fixture_pair();
        let err = MagicLink::issue(
            subject,
            MagicLinkScopeKind::Ticket,
            scope,
            issuer,
            Duration::zero(),
        )
        .unwrap_err();
        match err {
            AppError::Validation(_) => {}
            other => panic!("expected Validation, got {:?}", other),
        }
    }

    #[test]
    fn negative_negative_ttl_is_rejected() {
        let (subject, issuer, scope) = fixture_pair();
        let err = MagicLink::issue(
            subject,
            MagicLinkScopeKind::Ticket,
            scope,
            issuer,
            Duration::seconds(-1),
        )
        .unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }

    #[test]
    fn negative_nil_scope_id_is_rejected() {
        let (subject, issuer, _) = fixture_pair();
        let err = MagicLink::issue(
            subject,
            MagicLinkScopeKind::Quote,
            Uuid::nil(),
            issuer,
            Duration::hours(1),
        )
        .unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }

    #[test]
    fn negative_subject_equals_issuer_is_rejected() {
        let (subject, _, scope) = fixture_pair();
        let err = MagicLink::issue(
            subject,
            MagicLinkScopeKind::Ticket,
            scope,
            subject, // same as subject — syndic can't self-issue
            Duration::hours(1),
        )
        .unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }

    #[test]
    fn negative_invalid_scope_kind_string_is_rejected() {
        use std::str::FromStr;
        let err = MagicLinkScopeKind::from_str("payment").unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }
}
