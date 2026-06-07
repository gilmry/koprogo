//! Story 3.5 — Temporary role delegation use cases (FR8 INV-8).
//!
//! A syndic (or any holder of a *native* role) may delegate their role to
//! another user for a bounded duration. The platform enforces:
//!
//! - **Self-delegation forbidden** (`delegator != target`).
//! - **Duration bounded** in (`now`, `now + MAX_DELEGATION_DAYS`].
//! - **Anti-double-grant** (409): the target must not already hold this role
//!   actively (native or delegated).
//! - **Non-transitive** (403): a user that received the role through a
//!   delegation cannot re-delegate it. Caller MUST hold the role as a *native*
//!   assignment (`delegated_from_user_id IS NULL`).
//!
//! Handlers enforce the upstream "caller actually holds the role" check via
//! the JWT role + an active assignment lookup before calling `delegate_role`.

use crate::application::error::AppError;
use crate::application::ports::RoleDelegationRepository;
use crate::domain::entities::{UserRole, UserRoleAssignment};
use chrono::{DateTime, Duration, Utc};
use std::sync::Arc;
use uuid::Uuid;

/// Anti-abuse cap: no single delegation can outlive 90 days. A renewal MUST
/// go through a fresh `delegate_role` call (audit-faithful).
pub const MAX_DELEGATION_DAYS: i64 = 90;

pub struct RoleDelegationUseCases {
    repo: Arc<dyn RoleDelegationRepository>,
}

impl RoleDelegationUseCases {
    pub fn new(repo: Arc<dyn RoleDelegationRepository>) -> Self {
        Self { repo }
    }

    /// Delegate a role to another user.
    ///
    /// The caller (handler) MUST have already verified that `delegator_user_id`
    /// owns the role to be delegated (via JWT + native assignment lookup);
    /// this use-case re-checks the **non-transitive** invariant by inspecting
    /// the persisted assignment(s).
    pub async fn delegate_role(
        &self,
        delegator_user_id: Uuid,
        target_user_id: Uuid,
        role: UserRole,
        organization_id: Option<Uuid>,
        valid_until: DateTime<Utc>,
    ) -> Result<UserRoleAssignment, AppError> {
        // --- @security : self-delegation forbidden ----------------------
        if delegator_user_id == target_user_id {
            return Err(AppError::Validation(
                "Cannot delegate a role to oneself".to_string(),
            ));
        }
        if delegator_user_id.is_nil() || target_user_id.is_nil() {
            return Err(AppError::Validation(
                "Delegation user ids must not be nil UUIDs".to_string(),
            ));
        }

        // --- @edge : validity window in the future and bounded ----------
        let now = Utc::now();
        if valid_until <= now {
            return Err(AppError::Validation(
                "Delegation valid_until must be strictly in the future".to_string(),
            ));
        }
        let duration = valid_until - now;
        if duration > Duration::days(MAX_DELEGATION_DAYS) {
            return Err(AppError::Validation(format!(
                "Delegation duration exceeds {} days (anti-abuse)",
                MAX_DELEGATION_DAYS
            )));
        }

        // --- @security : non-transitive — delegator must hold the role
        //                NATIVELY (not via a prior delegation). ----------
        let delegator_assignments = self
            .repo
            .find_active_by_user_and_role(delegator_user_id, &role, organization_id)
            .await?;
        let has_native = delegator_assignments
            .iter()
            .any(|a| !a.is_delegated() && a.is_currently_active());
        if !has_native {
            // Either the delegator has no assignment for this role at all,
            // or the only ones they have are themselves delegations.
            return Err(AppError::DelegationChainNotAllowed);
        }

        // --- @negative : target already holds the role actively → 409 ---
        let target_assignments = self
            .repo
            .find_active_by_user_and_role(target_user_id, &role, organization_id)
            .await?;
        if target_assignments.iter().any(|a| a.is_currently_active()) {
            return Err(AppError::RoleAlreadyAssigned {
                user_id: target_user_id,
                role: role.to_string(),
            });
        }

        // --- Persist the delegation -------------------------------------
        let assignment = UserRoleAssignment::new_delegated(
            target_user_id,
            role,
            organization_id,
            valid_until,
            delegator_user_id,
        );
        self.repo.save(&assignment).await?;
        Ok(assignment)
    }

    /// Manually revoke a delegation before its term. Idempotent.
    pub async fn revoke_delegation(&self, assignment_id: Uuid) -> Result<(), AppError> {
        let existing =
            self.repo.find_by_id(assignment_id).await?.ok_or_else(|| {
                AppError::NotFound(format!("Delegation {} not found", assignment_id))
            })?;
        if !existing.is_delegated() {
            return Err(AppError::Validation(
                "Assignment is not a delegation".to_string(),
            ));
        }
        self.repo.revoke(assignment_id).await?;
        Ok(())
    }

    /// List active delegations involving `user_id` (received OR granted).
    pub async fn list_delegations_of(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<UserRoleAssignment>, AppError> {
        self.repo.list_delegations_of(user_id).await
    }
}

// ============================================================================
// Tests — taxonomie 4 catégories obligatoire (CRITICAL.md #3)
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use std::sync::Mutex;

    #[derive(Default)]
    struct InMemoryRepo {
        rows: Mutex<Vec<UserRoleAssignment>>,
    }

    impl InMemoryRepo {
        fn push(&self, a: UserRoleAssignment) {
            self.rows.lock().unwrap().push(a);
        }
    }

    #[async_trait]
    impl RoleDelegationRepository for InMemoryRepo {
        async fn save(&self, a: &UserRoleAssignment) -> Result<(), AppError> {
            self.rows.lock().unwrap().push(a.clone());
            Ok(())
        }

        async fn find_by_id(&self, id: Uuid) -> Result<Option<UserRoleAssignment>, AppError> {
            Ok(self
                .rows
                .lock()
                .unwrap()
                .iter()
                .find(|a| a.id == id)
                .cloned())
        }

        async fn find_active_by_user_and_role(
            &self,
            user_id: Uuid,
            role: &UserRole,
            organization_id: Option<Uuid>,
        ) -> Result<Vec<UserRoleAssignment>, AppError> {
            Ok(self
                .rows
                .lock()
                .unwrap()
                .iter()
                .filter(|a| {
                    a.user_id == user_id
                        && &a.role == role
                        && a.organization_id == organization_id
                        && a.is_currently_active()
                })
                .cloned()
                .collect())
        }

        async fn list_delegations_of(
            &self,
            user_id: Uuid,
        ) -> Result<Vec<UserRoleAssignment>, AppError> {
            Ok(self
                .rows
                .lock()
                .unwrap()
                .iter()
                .filter(|a| {
                    a.is_delegated()
                        && a.is_currently_active()
                        && (a.user_id == user_id || a.delegated_from_user_id == Some(user_id))
                })
                .cloned()
                .collect())
        }

        async fn revoke(&self, id: Uuid) -> Result<(), AppError> {
            self.rows.lock().unwrap().retain(|a| a.id != id);
            Ok(())
        }
    }

    fn factory() -> (Arc<InMemoryRepo>, RoleDelegationUseCases) {
        let repo: Arc<InMemoryRepo> = Arc::new(InMemoryRepo::default());
        let uc = RoleDelegationUseCases::new(repo.clone() as Arc<dyn RoleDelegationRepository>);
        (repo, uc)
    }

    /// Seed a native (non-delegated) role for `user`.
    fn seed_native(repo: &InMemoryRepo, user: Uuid, role: UserRole) {
        repo.push(UserRoleAssignment::new(user, role, None, true));
    }

    // ---- @happy ------------------------------------------------------------

    #[tokio::test]
    async fn happy_delegate_then_list_finds_assignment() {
        let (repo, uc) = factory();
        let syndic = Uuid::new_v4();
        let owner = Uuid::new_v4();
        seed_native(&repo, syndic, UserRole::Syndic);

        let valid_until = Utc::now() + Duration::days(7);
        let delegation = uc
            .delegate_role(syndic, owner, UserRole::Syndic, None, valid_until)
            .await
            .expect("delegate ok");

        assert_eq!(delegation.user_id, owner);
        assert_eq!(delegation.delegated_from_user_id, Some(syndic));
        assert!(delegation.is_delegated());
        assert!(delegation.is_currently_active());

        // Target's view of received delegations
        let received = uc.list_delegations_of(owner).await.unwrap();
        assert_eq!(received.len(), 1);
        assert_eq!(received[0].id, delegation.id);

        // Delegator's view of granted delegations
        let granted = uc.list_delegations_of(syndic).await.unwrap();
        assert_eq!(granted.len(), 1);
        assert_eq!(granted[0].id, delegation.id);
    }

    #[tokio::test]
    async fn happy_revoke_delegation_removes_it() {
        let (repo, uc) = factory();
        let syndic = Uuid::new_v4();
        let owner = Uuid::new_v4();
        seed_native(&repo, syndic, UserRole::Syndic);

        let valid_until = Utc::now() + Duration::days(7);
        let d = uc
            .delegate_role(syndic, owner, UserRole::Syndic, None, valid_until)
            .await
            .unwrap();
        uc.revoke_delegation(d.id).await.unwrap();
        assert!(uc.list_delegations_of(owner).await.unwrap().is_empty());
    }

    // ---- @edge -------------------------------------------------------------

    #[tokio::test]
    async fn edge_valid_until_exactly_now_is_rejected() {
        let (repo, uc) = factory();
        let syndic = Uuid::new_v4();
        let owner = Uuid::new_v4();
        seed_native(&repo, syndic, UserRole::Syndic);
        // Pin valid_until far enough in the past that the use-case `now`
        // computed during execution will be strictly later — i.e. the
        // window is already closed at validation time.
        let valid_until = Utc::now() - Duration::seconds(1);
        let err = uc
            .delegate_role(syndic, owner, UserRole::Syndic, None, valid_until)
            .await
            .unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }

    #[tokio::test]
    async fn edge_max_window_minus_epsilon_is_accepted() {
        let (repo, uc) = factory();
        let syndic = Uuid::new_v4();
        let owner = Uuid::new_v4();
        seed_native(&repo, syndic, UserRole::Syndic);
        // Slightly inside the MAX_DELEGATION_DAYS cap to avoid the boundary
        // race between caller `now` and use-case `now`.
        let valid_until = Utc::now() + Duration::days(MAX_DELEGATION_DAYS) - Duration::seconds(5);
        let res = uc
            .delegate_role(syndic, owner, UserRole::Syndic, None, valid_until)
            .await;
        assert!(res.is_ok(), "{:?}", res.err());
    }

    // ---- @security ---------------------------------------------------------

    #[tokio::test]
    async fn security_self_delegation_is_rejected() {
        let (repo, uc) = factory();
        let same = Uuid::new_v4();
        seed_native(&repo, same, UserRole::Syndic);
        let err = uc
            .delegate_role(
                same,
                same,
                UserRole::Syndic,
                None,
                Utc::now() + Duration::days(7),
            )
            .await
            .unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }

    #[tokio::test]
    async fn security_non_transitive_delegated_role_cannot_be_redelegated() {
        // Setup: original syndic delegates to ownerA. OwnerA tries to
        // re-delegate to ownerB — must fail with DelegationChainNotAllowed.
        let (repo, uc) = factory();
        let original_syndic = Uuid::new_v4();
        let owner_a = Uuid::new_v4();
        let owner_b = Uuid::new_v4();
        seed_native(&repo, original_syndic, UserRole::Syndic);

        let valid_until = Utc::now() + Duration::days(7);
        let _ = uc
            .delegate_role(
                original_syndic,
                owner_a,
                UserRole::Syndic,
                None,
                valid_until,
            )
            .await
            .expect("first delegation ok");

        // ownerA has Syndic via delegation only — try to re-delegate.
        let err = uc
            .delegate_role(owner_a, owner_b, UserRole::Syndic, None, valid_until)
            .await
            .unwrap_err();
        assert!(
            matches!(err, AppError::DelegationChainNotAllowed),
            "expected DelegationChainNotAllowed, got {:?}",
            err
        );
    }

    #[tokio::test]
    async fn security_delegator_without_any_role_is_rejected() {
        // The caller does not even hold the role. The use-case must refuse
        // (same chain-not-allowed error — strictest possible).
        let (_repo, uc) = factory();
        let stranger = Uuid::new_v4();
        let target = Uuid::new_v4();
        let err = uc
            .delegate_role(
                stranger,
                target,
                UserRole::Syndic,
                None,
                Utc::now() + Duration::days(7),
            )
            .await
            .unwrap_err();
        assert!(matches!(err, AppError::DelegationChainNotAllowed));
    }

    #[tokio::test]
    async fn security_nil_ids_are_rejected() {
        let (_repo, uc) = factory();
        let err = uc
            .delegate_role(
                Uuid::nil(),
                Uuid::new_v4(),
                UserRole::Syndic,
                None,
                Utc::now() + Duration::days(7),
            )
            .await
            .unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }

    // ---- @negative ---------------------------------------------------------

    #[tokio::test]
    async fn negative_duration_above_max_is_rejected() {
        let (repo, uc) = factory();
        let syndic = Uuid::new_v4();
        let target = Uuid::new_v4();
        seed_native(&repo, syndic, UserRole::Syndic);

        let valid_until = Utc::now() + Duration::days(MAX_DELEGATION_DAYS + 1);
        let err = uc
            .delegate_role(syndic, target, UserRole::Syndic, None, valid_until)
            .await
            .unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }

    #[tokio::test]
    async fn negative_target_already_has_role_returns_conflict() {
        let (repo, uc) = factory();
        let syndic_a = Uuid::new_v4();
        let target = Uuid::new_v4();
        seed_native(&repo, syndic_a, UserRole::Syndic);
        // Target already has Syndic natively → cannot re-grant via delegation.
        seed_native(&repo, target, UserRole::Syndic);

        let err = uc
            .delegate_role(
                syndic_a,
                target,
                UserRole::Syndic,
                None,
                Utc::now() + Duration::days(7),
            )
            .await
            .unwrap_err();
        assert!(
            matches!(err, AppError::RoleAlreadyAssigned { .. }),
            "expected RoleAlreadyAssigned, got {:?}",
            err
        );
    }

    #[tokio::test]
    async fn negative_revoke_unknown_id_returns_not_found() {
        let (_repo, uc) = factory();
        let err = uc.revoke_delegation(Uuid::new_v4()).await.unwrap_err();
        assert!(matches!(err, AppError::NotFound(_)));
    }

    #[tokio::test]
    async fn negative_revoke_native_assignment_is_rejected() {
        let (repo, uc) = factory();
        let user = Uuid::new_v4();
        let native = UserRoleAssignment::new(user, UserRole::Syndic, None, true);
        let native_id = native.id;
        repo.push(native);
        // Revoking a native (non-delegated) row through this use-case is a
        // categorical error — it should never reach this surface.
        let err = uc.revoke_delegation(native_id).await.unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }
}
