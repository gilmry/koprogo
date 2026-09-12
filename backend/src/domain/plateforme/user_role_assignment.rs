use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::user::UserRole;

/// Represents an assignment of a role to a user within an optional organization scope.
///
/// # Story 3.5 — Temporary role delegation (FR8 INV-8)
///
/// Two optional fields turn a permanent native assignment into a time-bounded
/// delegated assignment:
///
/// - `valid_until = None` ⇒ permanent / native role (legacy behaviour).
/// - `valid_until = Some(t)` ⇒ delegated assignment that auto-expires at `t`.
/// - `delegated_from_user_id = Some(delegator)` ⇒ trail of who delegated the
///   role; used by [`crate::application::use_cases::RoleDelegationUseCases`]
///   to enforce the **non-transitive** invariant (a user cannot re-delegate a
///   role that was itself delegated to them).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UserRoleAssignment {
    pub id: Uuid,
    pub user_id: Uuid,
    pub role: UserRole,
    pub organization_id: Option<Uuid>,
    pub is_primary: bool,
    /// Story 3.5: end of validity. `None` = permanent native role.
    #[serde(default)]
    pub valid_until: Option<DateTime<Utc>>,
    /// Story 3.5: source of the delegation. `None` = native role.
    /// `Some(uid)` = `uid` granted this role for a bounded duration.
    #[serde(default)]
    pub delegated_from_user_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl UserRoleAssignment {
    /// Creates a new permanent (native) role assignment.
    ///
    /// `valid_until` and `delegated_from_user_id` default to `None`. Use
    /// [`UserRoleAssignment::new_delegated`] for a time-bounded delegation.
    pub fn new(
        user_id: Uuid,
        role: UserRole,
        organization_id: Option<Uuid>,
        is_primary: bool,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            user_id,
            role,
            organization_id,
            is_primary,
            valid_until: None,
            delegated_from_user_id: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Creates a new delegated role assignment (Story 3.5).
    ///
    /// A delegated assignment is never the primary role of the target user —
    /// keeping the user's native primary intact for `users.role` consistency.
    pub fn new_delegated(
        user_id: Uuid,
        role: UserRole,
        organization_id: Option<Uuid>,
        valid_until: DateTime<Utc>,
        delegated_from_user_id: Uuid,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            user_id,
            role,
            organization_id,
            is_primary: false,
            valid_until: Some(valid_until),
            delegated_from_user_id: Some(delegated_from_user_id),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn set_primary(&mut self, primary: bool) {
        self.is_primary = primary;
        self.updated_at = Utc::now();
    }

    // === Story 3.5 helpers =================================================

    /// True if this assignment is a delegated (non-native) role.
    pub fn is_delegated(&self) -> bool {
        self.delegated_from_user_id.is_some()
    }

    /// True if the assignment carries a `valid_until` and it is in the past.
    ///
    /// A permanent native role (`valid_until = None`) is never expired.
    pub fn is_expired(&self) -> bool {
        self.is_expired_at(Utc::now())
    }

    /// Testable variant of [`is_expired`] taking an explicit reference time.
    pub fn is_expired_at(&self, t: DateTime<Utc>) -> bool {
        match self.valid_until {
            None => false,
            Some(until) => t >= until,
        }
    }

    /// True if the assignment authorises actions right now.
    ///
    /// Native (permanent) assignments are always active. Delegated
    /// assignments are active iff their window has not elapsed.
    pub fn is_currently_active(&self) -> bool {
        !self.is_expired()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    // ------------------------------------------------------------------
    // Legacy tests (kept for retrocompat)
    // ------------------------------------------------------------------

    #[test]
    fn test_new_assignment_defaults() {
        let user_id = Uuid::new_v4();
        let assignment = UserRoleAssignment::new(user_id, UserRole::Syndic, None, true);

        assert_eq!(assignment.user_id, user_id);
        assert_eq!(assignment.role, UserRole::Syndic);
        assert!(assignment.is_primary);
        assert!(assignment.organization_id.is_none());
        // Story 3.5: native role => not delegated, not expirable.
        assert!(!assignment.is_delegated());
        assert!(assignment.valid_until.is_none());
        assert!(assignment.is_currently_active());
    }

    #[test]
    fn test_set_primary_updates_timestamp() {
        let mut assignment =
            UserRoleAssignment::new(Uuid::new_v4(), UserRole::Accountant, None, false);
        let original_updated_at = assignment.updated_at;

        assignment.set_primary(true);

        assert!(assignment.is_primary);
        assert!(
            assignment.updated_at > original_updated_at,
            "Updated_at should change when toggling primary flag"
        );
    }

    // ------------------------------------------------------------------
    // Story 3.5 — delegation helpers — 4 categories
    // ------------------------------------------------------------------

    // --- @happy --------------------------------------------------------

    #[test]
    fn happy_new_delegated_assignment_is_currently_active() {
        let user_id = Uuid::new_v4();
        let delegator = Uuid::new_v4();
        let valid_until = Utc::now() + Duration::days(7);
        let a = UserRoleAssignment::new_delegated(
            user_id,
            UserRole::Syndic,
            None,
            valid_until,
            delegator,
        );

        assert!(a.is_currently_active());
        assert!(!a.is_expired());
        assert!(a.is_delegated());
        assert_eq!(a.delegated_from_user_id, Some(delegator));
        assert_eq!(a.valid_until, Some(valid_until));
        assert!(
            !a.is_primary,
            "delegated assignment must never be the user's primary role"
        );
    }

    #[test]
    fn happy_native_assignment_is_not_delegated() {
        let a = UserRoleAssignment::new(Uuid::new_v4(), UserRole::Owner, None, true);
        assert!(!a.is_delegated());
        assert!(a.is_currently_active());
        assert!(!a.is_expired());
    }

    // --- @edge ---------------------------------------------------------

    #[test]
    fn edge_just_before_valid_until_is_active() {
        let now = Utc::now();
        let valid_until = now + Duration::milliseconds(500);
        let a = UserRoleAssignment::new_delegated(
            Uuid::new_v4(),
            UserRole::Syndic,
            None,
            valid_until,
            Uuid::new_v4(),
        );
        // 100ms before the boundary: still active.
        assert!(!a.is_expired_at(now + Duration::milliseconds(100)));
        assert!(a.is_currently_active());
    }

    #[test]
    fn edge_at_or_after_valid_until_is_expired() {
        let valid_until = Utc::now() + Duration::milliseconds(10);
        let a = UserRoleAssignment::new_delegated(
            Uuid::new_v4(),
            UserRole::Syndic,
            None,
            valid_until,
            Uuid::new_v4(),
        );
        // Exactly at valid_until is considered expired (half-open window).
        assert!(a.is_expired_at(valid_until));
        // 1ms after: expired.
        assert!(a.is_expired_at(valid_until + Duration::milliseconds(1)));
    }

    // --- @security -----------------------------------------------------

    #[test]
    fn security_delegated_flag_is_preserved_through_serde_roundtrip() {
        let delegator = Uuid::new_v4();
        let a = UserRoleAssignment::new_delegated(
            Uuid::new_v4(),
            UserRole::Syndic,
            None,
            Utc::now() + Duration::days(3),
            delegator,
        );
        let json = serde_json::to_string(&a).expect("serialize ok");
        let back: UserRoleAssignment = serde_json::from_str(&json).expect("deserialize ok");
        // Critical for the @security non-transitive invariant: the delegation
        // trail MUST survive persistence + transport unchanged.
        assert_eq!(back.delegated_from_user_id, Some(delegator));
        assert_eq!(back.valid_until, a.valid_until);
        assert!(back.is_delegated());
    }

    #[test]
    fn security_native_role_serde_keeps_none_delegation() {
        let a = UserRoleAssignment::new(Uuid::new_v4(), UserRole::Owner, None, true);
        let json = serde_json::to_string(&a).expect("serialize ok");
        let back: UserRoleAssignment = serde_json::from_str(&json).expect("deserialize ok");
        // A native role must never be silently flagged as delegated by serde
        // defaults — protects the @security invariant in the upstream check.
        assert!(back.delegated_from_user_id.is_none());
        assert!(back.valid_until.is_none());
        assert!(!back.is_delegated());
    }

    // --- @negative -----------------------------------------------------

    #[test]
    fn negative_expired_delegation_is_not_currently_active() {
        // Construct an already-expired delegation by bypassing the helper —
        // simulating a row read from DB whose `valid_until` is in the past.
        let now = Utc::now();
        let a = UserRoleAssignment {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            role: UserRole::Syndic,
            organization_id: None,
            is_primary: false,
            valid_until: Some(now - Duration::seconds(1)),
            delegated_from_user_id: Some(Uuid::new_v4()),
            created_at: now - Duration::days(1),
            updated_at: now - Duration::days(1),
        };
        assert!(a.is_expired());
        assert!(!a.is_currently_active());
    }
}
