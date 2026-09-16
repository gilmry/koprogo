//! Port for persisting and looking up [`NotaryLink`] entities (Issue #855).
//!
//! All methods return `Result<_, AppError>` natively (no `Result<_, String>`
//! debt — CRITICAL.md #4).

use crate::application::error::AppError;
use crate::domain::entities::NotaryLink;
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait NotaryLinkRepository: Send + Sync {
    /// Persist a freshly issued NotaryLink. Caller guarantees `token_hash`
    /// uniqueness (sha256 of a 256-bit random token is collision-free in
    /// practice).
    async fn save(&self, link: &NotaryLink) -> Result<(), AppError>;

    /// Look up a link by its `token_hash` (already hashed by the caller).
    /// Returns `Ok(None)` if no record matches — callers must translate this
    /// to `AppError::NotaryLinkInvalid` so a forged token and an unknown
    /// token are indistinguishable from the client side (no enumeration).
    async fn find_by_token_hash(&self, token_hash: &str) -> Result<Option<NotaryLink>, AppError>;

    /// Look up a link by id (syndic-facing administrative actions: revoke,
    /// renew, display in the dashboard — the caller already knows the id).
    async fn find_by_id(&self, id: Uuid) -> Result<Option<NotaryLink>, AppError>;

    /// All links ever issued for one état daté, most recent first — feeds
    /// the syndic's emission screen and the "en défaut" follow-up view.
    async fn list_by_etat_date(&self, etat_date_id: Uuid) -> Result<Vec<NotaryLink>, AppError>;

    /// Persist mutations to an existing link (`revoke()`, `renew()`).
    async fn update(&self, link: &NotaryLink) -> Result<(), AppError>;
}
