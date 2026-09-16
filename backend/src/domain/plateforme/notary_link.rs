//! NotaryLink — signed, time-boxed, single-resource access link for notaries.
//!
//! Issue #855, [ADR 0048](../../../../../docs/adr/0048-identite-notaire-et-routes-publiques.md),
//! [ADR 0051](../../../../../docs/adr/0051-lien-notaire-sept-jours-renouvelable.md).
//!
//! `GET /etats-dates/reference/{reference_number}` served the full financial
//! position of a named co-owner (arrears, quote-parts, Art. 3.94/3.95) to
//! anyone who knew a reference number — 32 bits of the string are random, the
//! rest derivable. ADR 0048 ruled the route must be gated; ADR 0051 ruled the
//! mechanism: a syndic-issued signed link, valid seven calendar days,
//! multi-read within that window, revocable, renewable.
//!
//! Deliberately NOT a reuse of [`super::magic_link::MagicLink`]: `MagicLink`
//! requires `subject_user_id: Uuid NOT NULL REFERENCES users`, and a notary
//! consulted under this mechanism has no KoproGo account — ADR 0051 leaves
//! that question open, and this module resolves it by not requiring one.
//! Reusing `MagicLink`'s table/trait would have forced a fake user record, or
//! a nullable FK on a table three other features already depend on. #835
//! already flags the cost of parallel magic-link systems; this one earns its
//! own table by not being a magic link at all — no `consumed_at`, no subject.
//!
//! # Security model
//! - Clear token: 32 random bytes, base64url, generated once, never stored —
//!   only its SHA-256 hex digest (`token_hash`) is persisted.
//! - Multi-read: no consumption flag. A notary reopening the link,
//!   refreshing a page, or forwarding it to a clerk must not need a fresh
//!   link every time (ADR 0051 — usage unique "se paie en support").
//! - Scoped to ONE `etat_date_id` — a link issued for état daté A does not
//!   authorize état daté B (`authorizes()`).
//! - `expires_at` = issue time + [`VALIDITE_JOURS`] (7 days), fixed, not
//!   caller-supplied. ADR 0051 explicitly rejects aligning this TTL with
//!   `releve_notaire::DELAI_JOURS` (30 days): one is a security parameter an
//!   engineer sets, the other a legal deadline (Art. 3.89 § 5, 5°) the
//!   legislator sets — coupling them would make it impossible to tighten one
//!   without touching the other.
//! - Revocable before term (`revoke()`), renewable at any time except once
//!   revoked (`renew()`) — an explicit syndic act, never automatic.

use crate::application::error::AppError;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

/// ADR 0051 — la fenêtre d'exposition est le seul paramètre qui protège
/// vraiment. Sept jours couvrent une instruction courante sans laisser un
/// lien vivant un mois dans une boîte aux lettres. Distinct de
/// `releve_notaire::DELAI_JOURS` (30, légal) : celui-ci est un TTL de
/// sécurité, et les faire coïncider aurait amarré l'un à l'autre pour de
/// mauvaises raisons (ADR 0051, section « Une correction de ma
/// recommandation »).
pub const VALIDITE_JOURS: i64 = 7;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NotaryLink {
    pub id: Uuid,
    /// SHA-256 hex of the clear token. The clear token is NEVER stored.
    pub token_hash: String,
    /// The état daté this link authorizes access to — and ONLY this one.
    pub etat_date_id: Uuid,
    /// Syndic who issued the link (audit trail).
    pub issued_by: Uuid,
    pub expires_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl NotaryLink {
    /// Issue a new NotaryLink for one état daté, valid [`VALIDITE_JOURS`]
    /// calendar days from now. Returns the persisted entity AND the clear
    /// token (shown once in the HTTP response, never stored elsewhere).
    ///
    /// # Errors
    /// - `AppError::Validation` if `etat_date_id` or `issued_by` is nil.
    pub fn issue(etat_date_id: Uuid, issued_by: Uuid) -> Result<(Self, String), AppError> {
        if etat_date_id.is_nil() {
            return Err(AppError::Validation(
                "NotaryLink etat_date_id must not be nil".to_string(),
            ));
        }
        if issued_by.is_nil() {
            return Err(AppError::Validation(
                "NotaryLink issued_by must not be nil".to_string(),
            ));
        }

        let clear_token = Self::generate_clear_token();
        let token_hash = Self::hash_token(&clear_token);
        let now = Utc::now();
        let expires_at = now + Duration::days(VALIDITE_JOURS);

        let entity = Self {
            id: Uuid::new_v4(),
            token_hash,
            etat_date_id,
            issued_by,
            expires_at,
            revoked_at: None,
            created_at: now,
            updated_at: now,
        };

        Ok((entity, clear_token))
    }

    /// SHA-256 hex digest of a clear token. Public so the repository / use
    /// case can hash an incoming token and look it up by hash.
    pub fn hash_token(clear_token: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(clear_token.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Generate a random 32-byte token, base64url-encoded (no padding).
    /// Produces a ~43-character URL-safe string with ~256 bits of entropy —
    /// unlike the reference number it accompanies, this is not brute-forceable.
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

    pub fn is_revoked(&self) -> bool {
        self.revoked_at.is_some()
    }

    pub fn is_valid(&self) -> bool {
        !self.is_expired() && !self.is_revoked()
    }

    /// True if this link is currently valid AND scoped to `etat_date_id`.
    /// The scope check is what stops a link issued for état daté A from
    /// opening état daté B (#855 @edge) — the property that distinguishes
    /// this mechanism from a bare bearer token.
    pub fn authorizes(&self, etat_date_id: Uuid) -> bool {
        self.is_valid() && self.etat_date_id == etat_date_id
    }

    /// Revoke before term. Idempotent — a second call does not overwrite the
    /// original revocation timestamp.
    pub fn revoke(&mut self) {
        let now = Utc::now();
        if self.revoked_at.is_none() {
            self.revoked_at = Some(now);
        }
        self.updated_at = now;
    }

    /// Explicit syndic act: extend the link [`VALIDITE_JOURS`] days from now.
    ///
    /// Allowed even if already expired — ADR 0051: « une vente qui dépasse
    /// sept jours est normale, c'est au syndic de le constater. » NOT allowed
    /// on a revoked link: revocation is a deliberate, terminal act, and
    /// silently reviving it would defeat the syndic's decision to end access.
    ///
    /// # Errors
    /// - `AppError::NotaryLinkRevoked` if the link was revoked. This variant
    ///   is distinct from `NotaryLinkInvalid` on purpose: the caller here is
    ///   an authenticated syndic acting on a link id they already hold, not
    ///   an anonymous notary probing a reference number — there is no
    ///   enumeration risk in naming the cause precisely.
    pub fn renew(&mut self) -> Result<(), AppError> {
        if self.is_revoked() {
            return Err(AppError::NotaryLinkRevoked);
        }
        let now = Utc::now();
        self.expires_at = now + Duration::days(VALIDITE_JOURS);
        self.updated_at = now;
        Ok(())
    }
}

// ============================================================================
// Tests — taxonomie 4 catégories obligatoire (CRITICAL.md #3)
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_pair() -> (Uuid, Uuid) {
        (Uuid::new_v4(), Uuid::new_v4())
    }

    // ------------------------------------------------------------------------
    // @happy
    // ------------------------------------------------------------------------

    #[test]
    fn happy_issue_returns_valid_pair() {
        let (etat_date_id, issuer) = fixture_pair();
        let (link, clear) =
            NotaryLink::issue(etat_date_id, issuer).expect("issue should succeed for valid inputs");

        assert_eq!(link.etat_date_id, etat_date_id);
        assert_eq!(link.issued_by, issuer);
        assert!(link.is_valid());
        assert!(!link.is_revoked());
        assert!(!link.is_expired());
        assert!(clear.len() >= 40, "clear token too short: {}", clear.len());
    }

    #[test]
    fn happy_revoke_sets_revoked_at_and_invalidates() {
        let (etat_date_id, issuer) = fixture_pair();
        let (mut link, _) = NotaryLink::issue(etat_date_id, issuer).unwrap();

        assert!(link.is_valid());
        link.revoke();
        assert!(link.is_revoked());
        assert!(!link.is_valid());
        assert!(link.revoked_at.is_some());
    }

    #[test]
    fn happy_hash_token_is_deterministic_hex64() {
        let h1 = NotaryLink::hash_token("hello");
        let h2 = NotaryLink::hash_token("hello");
        assert_eq!(h1, h2);
        assert_eq!(h1.len(), 64); // sha256 hex
        assert!(h1.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn happy_renew_extends_expiry() {
        let (etat_date_id, issuer) = fixture_pair();
        let (mut link, _) = NotaryLink::issue(etat_date_id, issuer).unwrap();
        let original_expiry = link.expires_at;

        link.renew().expect("renew should succeed on a live link");
        assert!(link.expires_at > original_expiry);
        assert!(link.is_valid());
    }

    #[test]
    fn happy_authorizes_true_for_matching_scope_within_window() {
        let (etat_date_id, issuer) = fixture_pair();
        let (link, _) = NotaryLink::issue(etat_date_id, issuer).unwrap();
        assert!(link.authorizes(etat_date_id));
    }

    // ------------------------------------------------------------------------
    // @edge
    // ------------------------------------------------------------------------

    #[test]
    fn edge_token_at_exact_expiry_is_invalid_after_now() {
        let (etat_date_id, issuer) = fixture_pair();
        let (mut link, _) = NotaryLink::issue(etat_date_id, issuer).unwrap();
        link.expires_at = Utc::now() - Duration::seconds(1);
        assert!(link.is_expired());
        assert!(!link.is_valid());
        assert!(!link.authorizes(etat_date_id));
    }

    #[test]
    fn edge_double_revoke_is_idempotent() {
        let (etat_date_id, issuer) = fixture_pair();
        let (mut link, _) = NotaryLink::issue(etat_date_id, issuer).unwrap();
        link.revoke();
        let first_revoked_at = link.revoked_at.expect("first revoke sets timestamp");
        link.revoke();
        assert_eq!(link.revoked_at, Some(first_revoked_at));
    }

    #[test]
    fn edge_renew_after_expiry_restores_validity() {
        // ADR 0051 : une vente qui dépasse sept jours est normale, le
        // renouvellement d'un lien déjà expiré (mais pas révoqué) doit le
        // faire revivre — c'est un acte explicite du syndic, pas un abus.
        let (etat_date_id, issuer) = fixture_pair();
        let (mut link, _) = NotaryLink::issue(etat_date_id, issuer).unwrap();
        link.expires_at = Utc::now() - Duration::days(1);
        assert!(!link.is_valid());

        link.renew()
            .expect("renew should revive an expired-but-not-revoked link");
        assert!(link.is_valid());
    }

    #[test]
    fn edge_authorizes_false_for_different_etat_date_id() {
        // Le lien émis pour l'état daté A ne doit jamais ouvrir B — c'est la
        // propriété qui distingue ce mécanisme d'un jeton global (#855 @edge).
        let (etat_date_id_a, issuer) = fixture_pair();
        let etat_date_id_b = Uuid::new_v4();
        let (link, _) = NotaryLink::issue(etat_date_id_a, issuer).unwrap();
        assert!(link.authorizes(etat_date_id_a));
        assert!(!link.authorizes(etat_date_id_b));
    }

    #[test]
    fn edge_expiry_is_exactly_validite_jours_from_issue() {
        let (etat_date_id, issuer) = fixture_pair();
        let (link, _) = NotaryLink::issue(etat_date_id, issuer).unwrap();
        let delta = link.expires_at - link.created_at;
        assert_eq!(delta, Duration::days(VALIDITE_JOURS));
    }

    // ------------------------------------------------------------------------
    // @security
    // ------------------------------------------------------------------------

    #[test]
    fn security_each_issue_returns_distinct_token_and_hash() {
        let (etat_date_id, issuer) = fixture_pair();
        let (link_a, clear_a) = NotaryLink::issue(etat_date_id, issuer).unwrap();
        let (link_b, clear_b) = NotaryLink::issue(etat_date_id, issuer).unwrap();
        assert_ne!(clear_a, clear_b, "tokens must be unique per issue");
        assert_ne!(
            link_a.token_hash, link_b.token_hash,
            "hashes must differ since tokens differ"
        );
        assert_ne!(link_a.id, link_b.id);
    }

    #[test]
    fn security_clear_token_is_never_equal_to_stored_hash() {
        let (etat_date_id, issuer) = fixture_pair();
        let (link, clear) = NotaryLink::issue(etat_date_id, issuer).unwrap();
        assert_ne!(clear, link.token_hash);
        assert_eq!(NotaryLink::hash_token(&clear), link.token_hash);
    }

    #[test]
    fn security_different_inputs_produce_different_hashes() {
        let h1 = NotaryLink::hash_token("token-A");
        let h2 = NotaryLink::hash_token("token-B");
        assert_ne!(h1, h2);
    }

    #[test]
    fn security_expired_link_reports_invalid_even_if_not_revoked() {
        let (etat_date_id, issuer) = fixture_pair();
        let (mut link, _) = NotaryLink::issue(etat_date_id, issuer).unwrap();
        link.expires_at = Utc::now() - Duration::seconds(10);
        assert!(!link.is_revoked());
        assert!(!link.is_valid());
    }

    #[test]
    fn security_revoked_link_cannot_be_renewed() {
        let (etat_date_id, issuer) = fixture_pair();
        let (mut link, _) = NotaryLink::issue(etat_date_id, issuer).unwrap();
        link.revoke();
        let err = link.renew().unwrap_err();
        assert!(matches!(err, AppError::NotaryLinkRevoked));
        // Et le lien reste hors validité : le renouvellement raté ne l'a pas
        // fait revivre par effet de bord.
        assert!(!link.is_valid());
    }

    #[test]
    fn security_authorizes_false_once_revoked() {
        let (etat_date_id, issuer) = fixture_pair();
        let (mut link, _) = NotaryLink::issue(etat_date_id, issuer).unwrap();
        link.revoke();
        assert!(!link.authorizes(etat_date_id));
    }

    // ------------------------------------------------------------------------
    // @negative
    // ------------------------------------------------------------------------

    #[test]
    fn negative_nil_etat_date_id_is_rejected() {
        let (_, issuer) = fixture_pair();
        let err = NotaryLink::issue(Uuid::nil(), issuer).unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }

    #[test]
    fn negative_nil_issued_by_is_rejected() {
        let (etat_date_id, _) = fixture_pair();
        let err = NotaryLink::issue(etat_date_id, Uuid::nil()).unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }

    #[test]
    fn negative_renew_revoked_link_returns_typed_error_not_panic() {
        let (etat_date_id, issuer) = fixture_pair();
        let (mut link, _) = NotaryLink::issue(etat_date_id, issuer).unwrap();
        link.revoke();
        // Ne doit jamais paniquer : l'appel renvoie un Result typé.
        let result = link.renew();
        assert!(result.is_err());
    }
}
