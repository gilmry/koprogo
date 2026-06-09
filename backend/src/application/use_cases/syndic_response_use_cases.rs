//! Use cases for SyndicResponse + SLA escalation (Story 3.7 — FR32 INV-23).
//!
//! Three operations exposed to handlers:
//!
//! `SyndicResponseUseCases::respond` — a syndic posts a response to a ticket.
//! The caller (handler) MUST already have enforced the syndic / superadmin
//! role check. The use case:
//!
//! - checks the target ticket exists (else `AppError::NotFound`);
//! - mints a `SyndicResponse` (entity-level validation);
//! - persists it (append-only — repo MUST NOT `UPDATE`);
//! - if the response is posted BEFORE the ticket's `sla_due_at`, marks the
//!   ticket as escalated (idempotent) to pre-empt the future cron
//!   escalation. This is the SLA-satisfied path: the SLA window is
//!   "consumed" by a timely response.
//!
//! `SyndicResponseUseCases::list_for_ticket` — read-only listing for the
//! ticket detail view.
//!
//! `SyndicResponseUseCases::escalate_overdue` — cron entry point. Returns
//! the list of ticket ids that were just escalated by this pass. Idempotent
//! across calls.

use crate::application::error::AppError;
use crate::application::ports::{SyndicResponseRepository, TicketRepository};
use crate::domain::entities::SyndicResponse;
use chrono::{DateTime, Utc};
use std::sync::Arc;
use uuid::Uuid;

pub struct SyndicResponseUseCases<R, T>
where
    R: SyndicResponseRepository,
    T: TicketRepository,
{
    repo: Arc<R>,
    ticket_repo: Arc<T>,
}

impl<R, T> SyndicResponseUseCases<R, T>
where
    R: SyndicResponseRepository,
    T: TicketRepository,
{
    pub fn new(repo: Arc<R>, ticket_repo: Arc<T>) -> Self {
        Self { repo, ticket_repo }
    }

    /// Syndic responds to a ticket.
    ///
    /// Invariants:
    /// - Ticket MUST exist → `AppError::NotFound`.
    /// - `SyndicResponse::new` validates body + action_proposed.
    ///
    /// Side effect — SLA pre-emption: if the response is posted strictly
    /// before the ticket's `sla_due_at`, we set `sla_escalated_at = now()`
    /// idempotently so the SLA cron job will not double-escalate later.
    /// The `sla_escalated_at` column is here a "SLA consumed" marker, not
    /// strictly an "escalation happened" marker — its semantics is
    /// "the open SLA window is closed for this ticket".
    pub async fn respond(
        &self,
        ticket_id: Uuid,
        syndic_user_id: Uuid,
        body: String,
        action_proposed: Option<String>,
    ) -> Result<SyndicResponse, AppError> {
        // Confirm ticket exists. We bridge the legacy `Result<_, String>`
        // surface of TicketRepository here (cluster #555 follow-up will
        // migrate the trait).
        let ticket = self
            .ticket_repo
            .find_by_id(ticket_id)
            .await
            .map_err(AppError::from)?
            .ok_or_else(|| AppError::NotFound(format!("ticket {}", ticket_id)))?;

        let response = SyndicResponse::new(ticket_id, syndic_user_id, body, action_proposed)?;
        self.repo.save(&response).await?;

        // SLA pre-emption — only when a deadline exists AND it's still
        // open AND it has not yet been consumed (idempotency).
        let now = Utc::now();
        if let Some(due) = ticket.sla_due_at {
            if ticket.sla_escalated_at.is_none() && now < due {
                // Best-effort: an error here MUST NOT undo the response,
                // which is already persisted. Surface the error so the
                // caller can decide; in practice the repo only errors on
                // infra failures which the handler maps to 500.
                self.repo.mark_ticket_escalated(ticket_id, now).await?;
            }
        }

        Ok(response)
    }

    pub async fn list_for_ticket(&self, ticket_id: Uuid) -> Result<Vec<SyndicResponse>, AppError> {
        self.repo.list_for_ticket(ticket_id).await
    }

    /// Cron entry point — one pass of SLA escalation.
    ///
    /// Returns the list of ticket ids that were just escalated (i.e. went
    /// from `sla_escalated_at IS NULL` to `Some(now)`). Subsequent calls
    /// at the same `now` are no-ops (idempotency).
    ///
    /// NOTE: no role guard — the caller is the scheduler, not a user.
    pub async fn escalate_overdue(&self, now: DateTime<Utc>) -> Result<Vec<Uuid>, AppError> {
        let overdue = self.repo.find_overdue_tickets(now).await?;
        let mut escalated = Vec::with_capacity(overdue.len());
        for id in overdue {
            self.repo.mark_ticket_escalated(id, now).await?;
            escalated.push(id);
        }
        Ok(escalated)
    }
}

// ============================================================================
// Tests — taxonomie 4 catégories (CRITICAL.md #3, Story 3.7)
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::ports::TicketRepository;
    use crate::domain::entities::{
        Ticket, TicketCategory, TicketKind, TicketPriority, TicketSeverity, TicketStatus,
    };
    use async_trait::async_trait;
    use chrono::Duration;
    use std::collections::HashMap;
    use std::sync::Mutex;

    // ── Mock SyndicResponseRepository ────────────────────────────────────

    #[derive(Default)]
    struct InMemorySyndicResponseRepo {
        rows: Mutex<Vec<SyndicResponse>>,
        /// `ticket_id → sla_escalated_at` (proxy for the column on `tickets`,
        /// kept here because the mock ticket repo is independent).
        escalated: Mutex<HashMap<Uuid, DateTime<Utc>>>,
        /// Tickets whose SLA is "due" (computed externally by the test).
        /// Pretend a separate index keyed on `(sla_due_at <= now)` exists.
        due_tickets: Mutex<HashMap<Uuid, DateTime<Utc>>>,
    }

    #[async_trait]
    impl SyndicResponseRepository for InMemorySyndicResponseRepo {
        async fn save(&self, response: &SyndicResponse) -> Result<(), AppError> {
            self.rows.lock().unwrap().push(response.clone());
            Ok(())
        }

        async fn list_for_ticket(&self, ticket_id: Uuid) -> Result<Vec<SyndicResponse>, AppError> {
            let mut out: Vec<SyndicResponse> = self
                .rows
                .lock()
                .unwrap()
                .iter()
                .filter(|r| r.ticket_id == ticket_id)
                .cloned()
                .collect();
            out.sort_by_key(|r| r.created_at);
            Ok(out)
        }

        async fn find_overdue_tickets(&self, now: DateTime<Utc>) -> Result<Vec<Uuid>, AppError> {
            let due = self.due_tickets.lock().unwrap();
            let escalated = self.escalated.lock().unwrap();
            Ok(due
                .iter()
                .filter(|(id, deadline)| **deadline <= now && !escalated.contains_key(id))
                .map(|(id, _)| *id)
                .collect())
        }

        async fn mark_ticket_escalated(
            &self,
            ticket_id: Uuid,
            escalated_at: DateTime<Utc>,
        ) -> Result<(), AppError> {
            // Idempotent: first one wins.
            self.escalated
                .lock()
                .unwrap()
                .entry(ticket_id)
                .or_insert(escalated_at);
            Ok(())
        }
    }

    // ── Mock TicketRepository (minimal — only what we need) ──────────────

    struct InMemoryTicketRepo {
        tickets: Mutex<HashMap<Uuid, Ticket>>,
    }

    impl InMemoryTicketRepo {
        fn new() -> Self {
            Self {
                tickets: Mutex::new(HashMap::new()),
            }
        }

        fn insert(&self, t: Ticket) {
            self.tickets.lock().unwrap().insert(t.id, t);
        }
    }

    #[async_trait]
    impl TicketRepository for InMemoryTicketRepo {
        async fn create(&self, ticket: &Ticket) -> Result<Ticket, String> {
            self.tickets
                .lock()
                .unwrap()
                .insert(ticket.id, ticket.clone());
            Ok(ticket.clone())
        }
        async fn find_by_id(&self, id: Uuid) -> Result<Option<Ticket>, String> {
            Ok(self.tickets.lock().unwrap().get(&id).cloned())
        }
        async fn find_by_building(&self, _: Uuid) -> Result<Vec<Ticket>, String> {
            Ok(vec![])
        }
        async fn find_by_organization(&self, _: Uuid) -> Result<Vec<Ticket>, String> {
            Ok(vec![])
        }
        async fn find_by_created_by(&self, _: Uuid) -> Result<Vec<Ticket>, String> {
            Ok(vec![])
        }
        async fn find_by_assigned_to(&self, _: Uuid) -> Result<Vec<Ticket>, String> {
            Ok(vec![])
        }
        async fn find_by_status(&self, _: Uuid, _: TicketStatus) -> Result<Vec<Ticket>, String> {
            Ok(vec![])
        }
        async fn update(&self, ticket: &Ticket) -> Result<Ticket, String> {
            self.tickets
                .lock()
                .unwrap()
                .insert(ticket.id, ticket.clone());
            Ok(ticket.clone())
        }
        async fn delete(&self, id: Uuid) -> Result<bool, String> {
            Ok(self.tickets.lock().unwrap().remove(&id).is_some())
        }
        async fn count_by_building(&self, _: Uuid) -> Result<i64, String> {
            Ok(0)
        }
        async fn count_by_status(&self, _: Uuid, _: TicketStatus) -> Result<i64, String> {
            Ok(0)
        }
        async fn count_by_organization(&self, _: Uuid) -> Result<i64, String> {
            Ok(0)
        }
        async fn count_by_organization_and_status(
            &self,
            _: Uuid,
            _: TicketStatus,
        ) -> Result<i64, String> {
            Ok(0)
        }
    }

    // ── Helpers ──────────────────────────────────────────────────────────

    fn make_use_cases() -> (
        Arc<InMemorySyndicResponseRepo>,
        Arc<InMemoryTicketRepo>,
        SyndicResponseUseCases<InMemorySyndicResponseRepo, InMemoryTicketRepo>,
    ) {
        let resp_repo = Arc::new(InMemorySyndicResponseRepo::default());
        let ticket_repo = Arc::new(InMemoryTicketRepo::new());
        let uc = SyndicResponseUseCases::new(resp_repo.clone(), ticket_repo.clone());
        (resp_repo, ticket_repo, uc)
    }

    fn make_complaint_ticket(severity: TicketSeverity) -> Ticket {
        Ticket::new_with_kind(
            Uuid::new_v4(),
            Uuid::new_v4(),
            None,
            Uuid::new_v4(),
            "Plainte tapage".to_string(),
            "Description suffisamment longue".to_string(),
            TicketCategory::Other,
            TicketPriority::High,
            TicketKind::Complaint,
            Some(severity),
            None,
            Vec::new(),
            Vec::new(),
        )
        .expect("complaint must be valid in fixture")
    }

    fn fixture_body() -> String {
        "Bonjour, devis demandé chez le prestataire.".to_string()
    }

    // ---- @happy ---------------------------------------------------------

    #[tokio::test]
    async fn happy_respond_before_sla_due_marks_ticket_as_escalated() {
        let (resp_repo, ticket_repo, uc) = make_use_cases();
        let ticket = make_complaint_ticket(TicketSeverity::Critical);
        let ticket_id = ticket.id;
        ticket_repo.insert(ticket);

        let response = uc
            .respond(ticket_id, Uuid::new_v4(), fixture_body(), None)
            .await
            .expect("respond should succeed");

        assert_eq!(response.ticket_id, ticket_id);
        // Response persisted.
        assert_eq!(resp_repo.rows.lock().unwrap().len(), 1);
        // SLA pre-emption: ticket marked as "SLA consumed".
        assert!(
            resp_repo.escalated.lock().unwrap().contains_key(&ticket_id),
            "respond inside SLA window must pre-empt future cron escalation"
        );
    }

    #[tokio::test]
    async fn happy_list_for_ticket_returns_responses_oldest_first() {
        let (_resp_repo, ticket_repo, uc) = make_use_cases();
        let ticket = make_complaint_ticket(TicketSeverity::Normal);
        let ticket_id = ticket.id;
        ticket_repo.insert(ticket);

        uc.respond(
            ticket_id,
            Uuid::new_v4(),
            "Premier message du syndic.".to_string(),
            None,
        )
        .await
        .unwrap();
        // Second response — needs a different syndic to be realistic but
        // not strictly required by the entity (multiple replies allowed).
        uc.respond(
            ticket_id,
            Uuid::new_v4(),
            "Deuxième mise à jour du syndic.".to_string(),
            Some("schedule_inspection".to_string()),
        )
        .await
        .unwrap();

        let listed = uc.list_for_ticket(ticket_id).await.unwrap();
        assert_eq!(listed.len(), 2);
        assert!(listed[0].created_at <= listed[1].created_at);
    }

    // ---- @edge ----------------------------------------------------------

    #[tokio::test]
    async fn edge_respond_after_sla_due_does_not_pre_empt_escalation() {
        let (resp_repo, ticket_repo, uc) = make_use_cases();
        let mut ticket = make_complaint_ticket(TicketSeverity::Critical);
        // Force the deadline to the past.
        ticket.sla_due_at = Some(Utc::now() - Duration::seconds(10));
        let ticket_id = ticket.id;
        ticket_repo.insert(ticket);

        let _ = uc
            .respond(ticket_id, Uuid::new_v4(), fixture_body(), None)
            .await
            .expect("respond past deadline still records the reply");

        // No SLA pre-emption (window already closed).
        assert!(
            !resp_repo.escalated.lock().unwrap().contains_key(&ticket_id),
            "late response must NOT touch sla_escalated_at"
        );
    }

    #[tokio::test]
    async fn edge_respond_on_ticket_without_sla_due_succeeds_and_no_escalation() {
        let (resp_repo, ticket_repo, uc) = make_use_cases();
        // Build a Request ticket without severity → sla_due_at = None.
        let t = Ticket::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            None,
            Uuid::new_v4(),
            "Title".to_string(),
            "Description".to_string(),
            TicketCategory::Other,
            TicketPriority::Low,
        )
        .unwrap();
        let ticket_id = t.id;
        ticket_repo.insert(t);

        uc.respond(ticket_id, Uuid::new_v4(), fixture_body(), None)
            .await
            .unwrap();

        assert!(!resp_repo.escalated.lock().unwrap().contains_key(&ticket_id));
    }

    // ---- @security ------------------------------------------------------

    #[tokio::test]
    async fn security_escalate_overdue_is_idempotent_on_already_escalated_tickets() {
        let (resp_repo, _ticket_repo, uc) = make_use_cases();

        let t1 = Uuid::new_v4();
        let t2 = Uuid::new_v4();
        let now = Utc::now();

        // Two due tickets, neither escalated yet.
        resp_repo
            .due_tickets
            .lock()
            .unwrap()
            .insert(t1, now - Duration::minutes(5));
        resp_repo
            .due_tickets
            .lock()
            .unwrap()
            .insert(t2, now - Duration::hours(1));

        let first_pass = uc.escalate_overdue(now).await.unwrap();
        assert_eq!(first_pass.len(), 2, "first pass should escalate both");

        // Second pass at the same `now` — both already escalated.
        let second_pass = uc.escalate_overdue(now).await.unwrap();
        assert!(
            second_pass.is_empty(),
            "second pass must be a no-op (idempotency)"
        );
    }

    // ---- @negative ------------------------------------------------------

    #[tokio::test]
    async fn negative_respond_on_unknown_ticket_returns_not_found() {
        let (_resp_repo, _ticket_repo, uc) = make_use_cases();
        let err = uc
            .respond(Uuid::new_v4(), Uuid::new_v4(), fixture_body(), None)
            .await
            .unwrap_err();
        assert!(matches!(err, AppError::NotFound(_)));
    }

    #[tokio::test]
    async fn negative_respond_with_short_body_is_rejected_at_entity_level() {
        let (_resp_repo, ticket_repo, uc) = make_use_cases();
        let ticket = make_complaint_ticket(TicketSeverity::Normal);
        let ticket_id = ticket.id;
        ticket_repo.insert(ticket);

        let err = uc
            .respond(ticket_id, Uuid::new_v4(), "ko".to_string(), None)
            .await
            .unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }
}
