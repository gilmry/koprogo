//! Port for persisting [`Mandate`] entities (Story 3.4 — FR7 INV-14).
//!
//! All methods return `Result<_, AppError>` natively — no legacy `String`
//! error debt to migrate later (CRITICAL.md #4 / #555).

use crate::application::error::AppError;
use crate::domain::entities::{Mandate, MandateScope};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[async_trait]
pub trait MandateRepository: Send + Sync {
    /// Persist a freshly issued mandate.
    async fn save(&self, mandate: &Mandate) -> Result<(), AppError>;

    /// Look up a mandate by its primary key.
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Mandate>, AppError>;

    /// List all *currently active* (non-revoked, within validity window)
    /// mandates whose subject is the given user.
    async fn list_active_for_subject(
        &self,
        subject_user_id: Uuid,
    ) -> Result<Vec<Mandate>, AppError>;

    /// List all mandates that target the given scope (regardless of status).
    /// Useful for syndic audit views.
    async fn list_for_scope(&self, scope: &MandateScope) -> Result<Vec<Mandate>, AppError>;

    /// Atomically revoke a mandate. Implementation MUST do
    /// `UPDATE ... SET revoked_at = $2 WHERE id = $1 AND revoked_at IS NULL`
    /// so a double-revoke is a no-op (idempotent).
    async fn revoke(&self, id: Uuid, revoked_at: DateTime<Utc>) -> Result<(), AppError>;
}
