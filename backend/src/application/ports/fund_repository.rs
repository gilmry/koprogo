//! Port for persisting [`Fund`] entities and their [`FundReassignment`]
//! audit trail (issue #635 — fonds affectés & thésaurisation, ADR-0054).
//!
//! All methods return `Result<_, AppError>` natively — no legacy `String`
//! error debt to migrate later (CRITICAL.md #4 / #555).

use crate::application::error::AppError;
use crate::domain::entities::{Fund, FundReassignment};
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait FundRepository: Send + Sync {
    /// Persist a freshly created fund.
    async fn create(&self, fund: &Fund) -> Result<Fund, AppError>;

    /// Look up a fund by its primary key.
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Fund>, AppError>;

    /// List all funds belonging to an ACP (all three natures combined).
    async fn find_by_acp_id(&self, acp_id: Uuid) -> Result<Vec<Fund>, AppError>;

    /// Persist balance/purpose changes (contribution, reassignment).
    async fn update(&self, fund: &Fund) -> Result<Fund, AppError>;

    /// Append a reassignment to the audit trail. Never mutated afterward
    /// (append-only, like `SyndicResponse` — cf. `AppError::ResponseImmutable`
    /// precedent).
    async fn record_reassignment(&self, reassignment: &FundReassignment) -> Result<(), AppError>;

    /// List the reassignment history of a fund, most recent first.
    async fn list_reassignments(&self, fund_id: Uuid) -> Result<Vec<FundReassignment>, AppError>;
}
