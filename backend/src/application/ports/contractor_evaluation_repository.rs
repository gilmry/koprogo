//! Port for persisting [`ContractorEvaluation`] entities (Story 3.9 — FR34
//! FR35 INV-21 INV-24).
//!
//! Distinct from [`ContractEvaluationRepository`](super::contract_evaluation_repository::ContractEvaluationRepository),
//! the legacy marketplace-rating port (Issue #276). Story 3.9 introduces the
//! audit-grade flow gated by an approved TechnicalSpec.
//!
//! All methods return `Result<_, AppError>` natively — no legacy `String`
//! error debt to migrate later (CRITICAL.md #4 / #555).
//!
//! Evaluations are append-only — the repository exposes only [`save`] and
//! read methods; mutation guards are enforced at the DB trigger level (cf.
//! migration `20260605070000_create_contractor_evaluations.sql`).

use crate::application::error::AppError;
use crate::domain::entities::ContractorEvaluation;
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait ContractorEvaluationRepository: Send + Sync {
    /// Persist a freshly minted evaluation (append-only).
    async fn save(&self, evaluation: &ContractorEvaluation) -> Result<(), AppError>;

    async fn find_by_id(&self, id: Uuid) -> Result<Option<ContractorEvaluation>, AppError>;

    /// List every evaluation collected against a given contractor user,
    /// newest first. Used by the contractor's "reputation" view.
    async fn list_for_contractor(
        &self,
        contractor_user_id: Uuid,
    ) -> Result<Vec<ContractorEvaluation>, AppError>;
}
