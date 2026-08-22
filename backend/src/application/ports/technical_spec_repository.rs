//! Port for persisting [`TechnicalSpec`] and [`TechnicalSpecSignature`]
//! entities (Story 3.8 — FR33).
//!
//! All methods return `Result<_, AppError>` natively — no legacy `String`
//! error debt to migrate later (CRITICAL.md #4 / #555).
//!
//! Signatures are append-only — the repository exposes only [`save_signature`]
//! and a few read methods; mutation guards are enforced at the DB trigger
//! level (cf. migration `20260605060000_create_technical_specs.sql`).

use crate::application::error::AppError;
use crate::domain::entities::{TechnicalSpec, TechnicalSpecSignature};
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait TechnicalSpecRepository: Send + Sync {
    /// Persist a freshly minted spec (Draft).
    async fn save(&self, spec: &TechnicalSpec) -> Result<(), AppError>;

    /// Update a spec's mutable workflow attributes (status / updated_at).
    /// Used by the workflow transitions (`submit`, `mark_approved`).
    /// The repository implementation MUST NOT allow title / description /
    /// version edits — those happen exclusively via `bump_version` which
    /// goes through [`save`] on a brand-new row.
    async fn update_status(
        &self,
        spec_id: Uuid,
        status: &str,
        updated_at: chrono::DateTime<chrono::Utc>,
    ) -> Result<(), AppError>;

    async fn find_by_id(&self, id: Uuid) -> Result<Option<TechnicalSpec>, AppError>;

    /// List all specs for an ACP, newest first.
    async fn list_for_acp(&self, acp_id: Uuid) -> Result<Vec<TechnicalSpec>, AppError>;

    // ---- signatures ----

    /// Persist an append-only signature. The DB UNIQUE constraint on
    /// (spec_id, signatory_user_id, role) means a duplicate insert will
    /// surface as `AppError::SignatureAlreadyExists` (the impl translates
    /// the SQL conflict).
    async fn save_signature(&self, sig: &TechnicalSpecSignature) -> Result<(), AppError>;

    async fn list_signatures_for_spec(
        &self,
        spec_id: Uuid,
    ) -> Result<Vec<TechnicalSpecSignature>, AppError>;
}
