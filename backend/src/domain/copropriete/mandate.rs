//! Mandate — juridical delegation tracker (Story 3.4 — FR7 INV-14).
//!
//! When a syndic must delegate a legal/technical act to an external
//! professional (notaire for a unit sale, avocat for a litigation,
//! architecte for renovation works, BET for technical studies, AMO for
//! project management assistance, gardien for on-site duties), they issue
//! a `Mandate` materialising:
//!
//! - the mandataire (`subject_user_id`),
//! - the `kind` (mapped to a Story 3.1 `UserRole`),
//! - the `scope` (Building or whole ACP),
//! - mandatory temporal validity (`valid_from` / `valid_until`),
//! - immutable audit (`issued_by`, `reason`, timestamps),
//! - optional early revocation (`revoked_at`).
//!
//! # Invariants enforced at `issue()` time
//!
//! - `valid_until > valid_from`
//! - `valid_until - valid_from <= 5 years` (anti-abuse: no unlimited mandates)
//! - `reason.len() in [10, 500]`
//! - `subject_user_id != issued_by` (a syndic may not mandate themselves)
//!
//! Runtime helpers `is_expired()` / `is_revoked()` / `is_currently_active()`
//! are used by the upstream guard `assert_mandate_authorizes` to emit the
//! correctly-typed `AppError::Mandate*` (403/404) instead of generic
//! `Forbidden` strings.

use crate::application::error::AppError;
use crate::domain::entities::UserRole;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Kind of mandate. Mapped to a `UserRole` (Story 3.1) — issuing a mandate
/// presumes the subject already carries the corresponding role.
///
/// `Lawyer`, `Notary`, `Amo`, `Architect`, `Bet`, `Warden` are the six
/// mandataire roles introduced by Story 3.1.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MandateKind {
    Lawyer,
    Notary,
    Amo,
    Architect,
    Bet,
    Warden,
}

impl MandateKind {
    /// `UserRole` required of the subject for the mandate to be coherent.
    /// The handler/use-case should cross-check before issuing.
    pub fn required_user_role(&self) -> UserRole {
        match self {
            MandateKind::Lawyer => UserRole::Lawyer,
            MandateKind::Notary => UserRole::Notary,
            MandateKind::Amo => UserRole::Amo,
            MandateKind::Architect => UserRole::Architect,
            MandateKind::Bet => UserRole::Bet,
            MandateKind::Warden => UserRole::Warden,
        }
    }

    /// Whether issuance currently requires an AG (general assembly) decision.
    ///
    /// Story 3.4 ships the simple syndic-issued variant only; the litigation /
    /// > 5000 EUR mandate workflow is a documented follow-up (see story body).
    pub fn requires_ag_decision(&self) -> bool {
        false
    }
}

impl std::fmt::Display for MandateKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MandateKind::Lawyer => write!(f, "lawyer"),
            MandateKind::Notary => write!(f, "notary"),
            MandateKind::Amo => write!(f, "amo"),
            MandateKind::Architect => write!(f, "architect"),
            MandateKind::Bet => write!(f, "bet"),
            MandateKind::Warden => write!(f, "warden"),
        }
    }
}

impl std::str::FromStr for MandateKind {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "lawyer" => Ok(MandateKind::Lawyer),
            "notary" => Ok(MandateKind::Notary),
            "amo" => Ok(MandateKind::Amo),
            "architect" => Ok(MandateKind::Architect),
            "bet" => Ok(MandateKind::Bet),
            "warden" => Ok(MandateKind::Warden),
            other => Err(AppError::Validation(format!(
                "Invalid mandate kind: {}",
                other
            ))),
        }
    }
}

/// Scope a mandate applies to. Either a single building, or the whole ACP
/// (e.g. a notaire mandated on all transactions of a copropriété).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MandateScope {
    Building(Uuid),
    Acp(Uuid),
}

impl MandateScope {
    pub fn kind_str(&self) -> &'static str {
        match self {
            MandateScope::Building(_) => "building",
            MandateScope::Acp(_) => "acp",
        }
    }

    pub fn id(&self) -> Uuid {
        match self {
            MandateScope::Building(id) | MandateScope::Acp(id) => *id,
        }
    }

    /// Build a scope from the (kind_str, id) tuple used at the persistence /
    /// HTTP boundary. Validates the kind string against the whitelist.
    pub fn from_parts(kind: &str, id: Uuid) -> Result<Self, AppError> {
        match kind.trim().to_lowercase().as_str() {
            "building" => Ok(MandateScope::Building(id)),
            "acp" => Ok(MandateScope::Acp(id)),
            other => Err(AppError::Validation(format!(
                "Invalid mandate scope_kind: {}",
                other
            ))),
        }
    }
}

/// Anti-abuse: a single mandate cannot be active for more than 5 years.
/// Reissue (with a fresh `reason`) keeps the audit trail granular.
pub const MAX_MANDATE_DURATION_DAYS: i64 = 365 * 5;

/// Minimal `reason` length (Belgian deontological practice: a mandate without
/// a stated motive is unenforceable in front of a juge de paix).
pub const MIN_REASON_LEN: usize = 10;

/// Hard upper bound to keep DB rows compact and prevent free-form abuse.
pub const MAX_REASON_LEN: usize = 500;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Mandate {
    pub id: Uuid,
    pub subject_user_id: Uuid,
    pub kind: MandateKind,
    pub scope: MandateScope,
    pub issued_by: Uuid,
    pub reason: String,
    pub valid_from: DateTime<Utc>,
    pub valid_until: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Mandate {
    /// Issue a new mandate. Invariants validated here (cf. module-level docs).
    #[allow(clippy::too_many_arguments)]
    pub fn issue(
        subject_user_id: Uuid,
        kind: MandateKind,
        scope: MandateScope,
        issued_by: Uuid,
        reason: String,
        valid_from: DateTime<Utc>,
        valid_until: DateTime<Utc>,
    ) -> Result<Self, AppError> {
        if subject_user_id == issued_by {
            return Err(AppError::Validation(
                "Mandate subject and issuer must differ".to_string(),
            ));
        }
        if valid_until <= valid_from {
            return Err(AppError::Validation(
                "Mandate valid_until must be strictly after valid_from".to_string(),
            ));
        }
        let duration = valid_until - valid_from;
        if duration > Duration::days(MAX_MANDATE_DURATION_DAYS) {
            return Err(AppError::Validation(format!(
                "Mandate duration exceeds {} days (anti-abuse)",
                MAX_MANDATE_DURATION_DAYS
            )));
        }
        let trimmed_reason = reason.trim().to_string();
        if trimmed_reason.len() < MIN_REASON_LEN {
            return Err(AppError::Validation(format!(
                "Mandate reason must be at least {} chars",
                MIN_REASON_LEN
            )));
        }
        if trimmed_reason.len() > MAX_REASON_LEN {
            return Err(AppError::Validation(format!(
                "Mandate reason must be at most {} chars",
                MAX_REASON_LEN
            )));
        }
        if subject_user_id.is_nil() || issued_by.is_nil() || scope.id().is_nil() {
            return Err(AppError::Validation(
                "Mandate references must not be nil UUIDs".to_string(),
            ));
        }

        let now = Utc::now();
        Ok(Self {
            id: Uuid::new_v4(),
            subject_user_id,
            kind,
            scope,
            issued_by,
            reason: trimmed_reason,
            valid_from,
            valid_until,
            revoked_at: None,
            created_at: now,
            updated_at: now,
        })
    }

    pub fn is_revoked(&self) -> bool {
        self.revoked_at.is_some()
    }

    pub fn is_expired_at(&self, t: DateTime<Utc>) -> bool {
        t >= self.valid_until
    }

    pub fn is_expired(&self) -> bool {
        self.is_expired_at(Utc::now())
    }

    pub fn is_active_at(&self, t: DateTime<Utc>) -> bool {
        !self.is_revoked() && t >= self.valid_from && t < self.valid_until
    }

    pub fn is_currently_active(&self) -> bool {
        self.is_active_at(Utc::now())
    }

    /// Mark this mandate as revoked. Idempotent — second calls keep the
    /// original `revoked_at` (audit-faithful).
    pub fn revoke(&mut self) {
        let now = Utc::now();
        if self.revoked_at.is_none() {
            self.revoked_at = Some(now);
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

    fn fixture_ids() -> (Uuid, Uuid, Uuid) {
        (Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4())
    }

    fn fixture_reason() -> String {
        "Cession unité C12 — acte authentique".to_string()
    }

    fn fixture_valid_window() -> (DateTime<Utc>, DateTime<Utc>) {
        let now = Utc::now();
        (now, now + Duration::days(60))
    }

    // ---- @happy -------------------------------------------------------------

    #[test]
    fn happy_issue_notary_mandate_is_currently_active() {
        let (subject, issuer, scope_id) = fixture_ids();
        let (from, until) = fixture_valid_window();
        let m = Mandate::issue(
            subject,
            MandateKind::Notary,
            MandateScope::Building(scope_id),
            issuer,
            fixture_reason(),
            from,
            until,
        )
        .expect("valid mandate should be issued");

        assert_eq!(m.subject_user_id, subject);
        assert_eq!(m.issued_by, issuer);
        assert_eq!(m.kind, MandateKind::Notary);
        assert!(matches!(m.scope, MandateScope::Building(_)));
        assert!(m.is_currently_active());
        assert!(!m.is_expired());
        assert!(!m.is_revoked());
    }

    #[test]
    fn happy_revoke_sets_revoked_at_and_invalidates() {
        let (subject, issuer, scope_id) = fixture_ids();
        let (from, until) = fixture_valid_window();
        let mut m = Mandate::issue(
            subject,
            MandateKind::Lawyer,
            MandateScope::Acp(scope_id),
            issuer,
            fixture_reason(),
            from,
            until,
        )
        .unwrap();

        m.revoke();
        assert!(m.is_revoked());
        assert!(!m.is_currently_active());
    }

    #[test]
    fn happy_kind_maps_to_corresponding_user_role() {
        assert_eq!(MandateKind::Lawyer.required_user_role(), UserRole::Lawyer);
        assert_eq!(MandateKind::Notary.required_user_role(), UserRole::Notary);
        assert_eq!(MandateKind::Amo.required_user_role(), UserRole::Amo);
        assert_eq!(
            MandateKind::Architect.required_user_role(),
            UserRole::Architect
        );
        assert_eq!(MandateKind::Bet.required_user_role(), UserRole::Bet);
        assert_eq!(MandateKind::Warden.required_user_role(), UserRole::Warden);
    }

    #[test]
    fn happy_kind_roundtrips_via_display_and_from_str() {
        use std::str::FromStr;
        for k in [
            MandateKind::Lawyer,
            MandateKind::Notary,
            MandateKind::Amo,
            MandateKind::Architect,
            MandateKind::Bet,
            MandateKind::Warden,
        ] {
            let s = k.to_string();
            assert_eq!(MandateKind::from_str(&s).unwrap(), k);
        }
    }

    // ---- @edge --------------------------------------------------------------

    #[test]
    fn edge_minimum_window_one_second_is_accepted() {
        let (subject, issuer, scope_id) = fixture_ids();
        let from = Utc::now();
        let until = from + Duration::seconds(1);
        let m = Mandate::issue(
            subject,
            MandateKind::Warden,
            MandateScope::Building(scope_id),
            issuer,
            fixture_reason(),
            from,
            until,
        );
        assert!(m.is_ok());
    }

    #[test]
    fn edge_exactly_five_years_is_accepted_boundary() {
        let (subject, issuer, scope_id) = fixture_ids();
        let from = Utc::now();
        let until = from + Duration::days(MAX_MANDATE_DURATION_DAYS);
        let m = Mandate::issue(
            subject,
            MandateKind::Architect,
            MandateScope::Acp(scope_id),
            issuer,
            fixture_reason(),
            from,
            until,
        );
        assert!(
            m.is_ok(),
            "exactly 5 years must remain inside the allowed bound"
        );
    }

    #[test]
    fn edge_at_valid_until_is_considered_expired() {
        let (subject, issuer, scope_id) = fixture_ids();
        let from = Utc::now() - Duration::days(2);
        let until = Utc::now() - Duration::seconds(1);
        let m = Mandate::issue(
            subject,
            MandateKind::Notary,
            MandateScope::Building(scope_id),
            issuer,
            fixture_reason(),
            from,
            until,
        )
        .unwrap();
        assert!(m.is_expired());
        assert!(!m.is_currently_active());
    }

    #[test]
    fn edge_double_revoke_is_idempotent() {
        let (subject, issuer, scope_id) = fixture_ids();
        let (from, until) = fixture_valid_window();
        let mut m = Mandate::issue(
            subject,
            MandateKind::Bet,
            MandateScope::Building(scope_id),
            issuer,
            fixture_reason(),
            from,
            until,
        )
        .unwrap();
        m.revoke();
        let first = m.revoked_at.expect("first revoke sets timestamp");
        m.revoke();
        assert_eq!(m.revoked_at, Some(first));
    }

    // ---- @security ----------------------------------------------------------

    #[test]
    fn security_subject_equals_issuer_is_rejected() {
        let (subject, _, scope_id) = fixture_ids();
        let (from, until) = fixture_valid_window();
        let err = Mandate::issue(
            subject,
            MandateKind::Lawyer,
            MandateScope::Building(scope_id),
            subject, // self-mandate
            fixture_reason(),
            from,
            until,
        )
        .unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }

    #[test]
    fn security_building_and_acp_scopes_are_distinct() {
        let id = Uuid::new_v4();
        let b = MandateScope::Building(id);
        let a = MandateScope::Acp(id);
        assert_ne!(b, a, "Building and Acp scopes must not compare equal");
        assert_eq!(b.kind_str(), "building");
        assert_eq!(a.kind_str(), "acp");
    }

    #[test]
    fn security_nil_uuids_are_rejected() {
        let (from, until) = fixture_valid_window();
        let err = Mandate::issue(
            Uuid::nil(),
            MandateKind::Notary,
            MandateScope::Building(Uuid::new_v4()),
            Uuid::new_v4(),
            fixture_reason(),
            from,
            until,
        )
        .unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }

    #[test]
    fn security_revoked_mandate_inside_window_is_not_active() {
        let (subject, issuer, scope_id) = fixture_ids();
        let (from, until) = fixture_valid_window();
        let mut m = Mandate::issue(
            subject,
            MandateKind::Amo,
            MandateScope::Acp(scope_id),
            issuer,
            fixture_reason(),
            from,
            until,
        )
        .unwrap();
        m.revoke();
        // Even mid-window, a revoked mandate must NOT authorise actions.
        assert!(!m.is_currently_active());
    }

    // ---- @negative ----------------------------------------------------------

    #[test]
    fn negative_valid_until_before_valid_from_is_rejected() {
        let (subject, issuer, scope_id) = fixture_ids();
        let from = Utc::now();
        let until = from - Duration::seconds(1);
        let err = Mandate::issue(
            subject,
            MandateKind::Notary,
            MandateScope::Building(scope_id),
            issuer,
            fixture_reason(),
            from,
            until,
        )
        .unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }

    #[test]
    fn negative_window_equal_zero_is_rejected() {
        let (subject, issuer, scope_id) = fixture_ids();
        let now = Utc::now();
        let err = Mandate::issue(
            subject,
            MandateKind::Warden,
            MandateScope::Building(scope_id),
            issuer,
            fixture_reason(),
            now,
            now,
        )
        .unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }

    #[test]
    fn negative_duration_above_max_is_rejected() {
        let (subject, issuer, scope_id) = fixture_ids();
        let from = Utc::now();
        let until = from + Duration::days(MAX_MANDATE_DURATION_DAYS + 1);
        let err = Mandate::issue(
            subject,
            MandateKind::Lawyer,
            MandateScope::Acp(scope_id),
            issuer,
            fixture_reason(),
            from,
            until,
        )
        .unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }

    #[test]
    fn negative_reason_too_short_is_rejected() {
        let (subject, issuer, scope_id) = fixture_ids();
        let (from, until) = fixture_valid_window();
        let err = Mandate::issue(
            subject,
            MandateKind::Notary,
            MandateScope::Building(scope_id),
            issuer,
            "court".to_string(), // 5 chars < MIN_REASON_LEN
            from,
            until,
        )
        .unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }

    #[test]
    fn negative_reason_too_long_is_rejected() {
        let (subject, issuer, scope_id) = fixture_ids();
        let (from, until) = fixture_valid_window();
        let too_long = "x".repeat(MAX_REASON_LEN + 1);
        let err = Mandate::issue(
            subject,
            MandateKind::Architect,
            MandateScope::Building(scope_id),
            issuer,
            too_long,
            from,
            until,
        )
        .unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }

    #[test]
    fn negative_invalid_kind_string_is_rejected() {
        use std::str::FromStr;
        let err = MandateKind::from_str("plombier").unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }

    #[test]
    fn negative_invalid_scope_kind_string_is_rejected() {
        let err = MandateScope::from_parts("unit", Uuid::new_v4()).unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }
}
