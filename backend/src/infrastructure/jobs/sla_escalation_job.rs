//! SLA escalation cron job stub (Story 3.7 — FR32 INV-23).
//!
//! Phase A scope: expose a single `run_once()` entrypoint that performs one
//! pass of SLA escalation by delegating to
//! [`SyndicResponseUseCases::escalate_overdue`]. The real scheduler (tokio
//! interval task) lands in Phase B / a follow-up; tests call `run_once`
//! manually for now.

use crate::application::error::AppError;
use crate::application::ports::{SyndicResponseRepository, TicketRepository};
use crate::application::use_cases::SyndicResponseUseCases;
use chrono::Utc;
use std::sync::Arc;

pub struct SlaEscalationJob<R, T>
where
    R: SyndicResponseRepository,
    T: TicketRepository,
{
    use_cases: Arc<SyndicResponseUseCases<R, T>>,
}

impl<R, T> SlaEscalationJob<R, T>
where
    R: SyndicResponseRepository,
    T: TicketRepository,
{
    pub fn new(use_cases: Arc<SyndicResponseUseCases<R, T>>) -> Self {
        Self { use_cases }
    }

    /// One escalation pass. Returns the number of tickets whose SLA was
    /// just consumed by this pass. Subsequent calls at roughly the same
    /// time are no-ops (idempotency at the use-case layer).
    pub async fn run_once(&self) -> Result<usize, AppError> {
        let now = Utc::now();
        let escalated = self.use_cases.escalate_overdue(now).await?;
        Ok(escalated.len())
    }
}
