//! Port for persisting and looking up [`LienNotaire`] entities.
//!
//! #845 / ADR 0051. All methods return `Result<_, AppError>` natively — no
//! `Result<_, String>` debt to migrate (CRITICAL.md #4).

use crate::application::error::AppError;
use crate::domain::entities::LienNotaire;
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait LienNotaireRepository: Send + Sync {
    /// Persist a freshly issued link.
    async fn save(&self, lien: &LienNotaire) -> Result<(), AppError>;

    /// Look up a link by its `token_hash` (already hashed by the caller).
    /// Returns `Ok(None)` if no record matches.
    async fn find_by_token_hash(&self, token_hash: &str) -> Result<Option<LienNotaire>, AppError>;

    /// The most recently issued, non-revoked link for a given état daté —
    /// what `renew`/`revoke` act on. May be expired: expiration is not
    /// terminal, only revocation is (ADR 0051 — le renouvellement reste
    /// possible après l'échéance).
    async fn find_active_by_etat_date_id(
        &self,
        etat_date_id: Uuid,
    ) -> Result<Option<LienNotaire>, AppError>;

    /// Persist a mutated link (renewal or revocation).
    async fn update(&self, lien: &LienNotaire) -> Result<(), AppError>;
}
