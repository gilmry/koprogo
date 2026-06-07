//! Port for persisting [`SyndicResponse`] entities (Story 3.7 — FR32 INV-23).
//!
//! All methods return `Result<_, AppError>` natively — no legacy `String`
//! error debt to migrate later (CRITICAL.md #4 / #555).
//!
//! The repository also exposes the SLA-escalation tracking primitives
//! ([`SyndicResponseRepository::find_overdue_tickets`],
//! [`SyndicResponseRepository::mark_ticket_escalated`]) used by the cron job
//! [`crate::infrastructure::jobs::sla_escalation_job::SlaEscalationJob`].
//! These primitives are intentionally on this repository (and not on
//! `TicketRepository`) because they are part of the SyndicResponse / SLA
//! bounded responsibility and the existing `TicketRepository` still carries
//! the legacy `Result<_, String>` surface (cluster #555 migration WIP).

use crate::application::error::AppError;
use crate::domain::entities::SyndicResponse;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[async_trait]
pub trait SyndicResponseRepository: Send + Sync {
    /// Persist a freshly minted response. Implementation MUST NOT attempt
    /// `UPDATE ... ON CONFLICT` — the table is append-only and the DB
    /// trigger guards against any subsequent mutation (INV-23).
    async fn save(&self, response: &SyndicResponse) -> Result<(), AppError>;

    /// List every response attached to a ticket, oldest first (audit order).
    async fn list_for_ticket(&self, ticket_id: Uuid) -> Result<Vec<SyndicResponse>, AppError>;

    /// Return the ids of tickets whose SLA deadline (`sla_due_at`) is past
    /// `now` AND that have not been escalated yet
    /// (`sla_escalated_at IS NULL`). Cron entry point.
    async fn find_overdue_tickets(&self, now: DateTime<Utc>) -> Result<Vec<Uuid>, AppError>;

    /// Mark a ticket as SLA-escalated. Idempotent: if the column already
    /// carries a timestamp, the implementation SHOULD leave it untouched
    /// (audit fidelity — first event wins).
    async fn mark_ticket_escalated(
        &self,
        ticket_id: Uuid,
        escalated_at: DateTime<Utc>,
    ) -> Result<(), AppError>;
}
