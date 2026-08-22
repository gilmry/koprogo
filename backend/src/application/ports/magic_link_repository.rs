//! Port for persisting and looking up [`MagicLink`] entities.
//!
//! All methods return `Result<_, AppError>` natively (no `Result<_, String>`
//! debt to migrate — Story 3.2 ships AppError-clean per CRITICAL.md #4 / #555).

use crate::application::error::AppError;
use crate::domain::entities::MagicLink;
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait MagicLinkRepository: Send + Sync {
    /// Persist a freshly issued MagicLink. Caller guarantees `token_hash`
    /// uniqueness (sha256 of 256-bit random token is collision-free in practice).
    async fn save(&self, link: &MagicLink) -> Result<(), AppError>;

    /// Look up a link by its `token_hash` (already hashed by the caller).
    /// Returns `Ok(None)` if no record matches — callers must translate this
    /// to `AppError::MagicLinkInvalid` so a forged token and an unknown token
    /// are indistinguishable from the client side (no enumeration).
    async fn find_by_token_hash(&self, token_hash: &str) -> Result<Option<MagicLink>, AppError>;

    /// Atomically mark a link as consumed (sets `consumed_at = NOW()`).
    /// The implementation MUST be a single `UPDATE ... WHERE id = $1
    /// AND consumed_at IS NULL` so two concurrent consumers cannot both
    /// succeed (race-free single-use guarantee).
    async fn mark_consumed(&self, id: Uuid) -> Result<(), AppError>;
}
