//! Port for persisting Story 3.5 role delegations.
//!
//! A delegated assignment is a `UserRoleAssignment` whose `valid_until` is
//! `Some(_)` and whose `delegated_from_user_id` is `Some(_)`. The dedicated
//! port surface is `AppError`-typed (CRITICAL.md #4) so the use-case never
//! handles `Result<_, String>`.

use crate::application::error::AppError;
use crate::domain::entities::{UserRole, UserRoleAssignment};
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait RoleDelegationRepository: Send + Sync {
    /// Persist a freshly created delegation assignment.
    async fn save(&self, assignment: &UserRoleAssignment) -> Result<(), AppError>;

    /// Look up a delegation row by id. Returns `None` if not found OR not a
    /// delegation row (i.e. `valid_until IS NULL`).
    async fn find_by_id(&self, id: Uuid) -> Result<Option<UserRoleAssignment>, AppError>;

    /// Look up the (active or expired) assignments currently held by `user_id`
    /// with a given `role`. Used to enforce the @security non-transitive
    /// invariant (the caller must have a *native* assignment for the role
    /// they want to delegate) and the anti-double-grant 409.
    async fn find_active_by_user_and_role(
        &self,
        user_id: Uuid,
        role: &UserRole,
        organization_id: Option<Uuid>,
    ) -> Result<Vec<UserRoleAssignment>, AppError>;

    /// List all active delegations involving `user_id`, either as target
    /// (received) or as delegator (granted). Used by the audit list view.
    async fn list_delegations_of(&self, user_id: Uuid)
        -> Result<Vec<UserRoleAssignment>, AppError>;

    /// Revoke a delegation by its assignment id (best-effort delete).
    /// Idempotent: revoking an already-removed row returns `Ok(())`.
    async fn revoke(&self, id: Uuid) -> Result<(), AppError>;
}
