//! SyndicResponse — append-only structured reply to a ticket (Story 3.7 —
//! FR32 INV-23).
//!
//! Each row represents a syndic's reply (text + optional declared action).
//! The relation is **append-only**: edits and deletions are blocked both at
//! the SQL trigger level (cf. `20260605050000_create_syndic_responses.sql`)
//! and via the `AppError::ResponseImmutable` typed error returned by any
//! upstream layer that tried to mutate a persisted response.
//!
//! # SLA model
//!
//! The `sla_window_for_severity` function returns the maximum acceptable
//! delay between ticket creation and the first syndic response, depending
//! on the ticket's [`TicketSeverity`]. The SLA escalation cron job
//! ([`crate::infrastructure::jobs::sla_escalation_job`]) compares
//! `Ticket.sla_due_at` to `now()` to flag overdue tickets.
//!
//! | Severity  | Window     | Rationale                              |
//! |-----------|------------|----------------------------------------|
//! | Critical  | 24 hours   | Imminent risk (eau, gaz, sécurité)     |
//! | High      | 72 hours   | Strong impact (ascenseur, chauffage)   |
//! | Normal    | 5 days     | Standard request                       |
//! | Low       | 10 days    | Cosmetic / non-urgent                  |

use crate::application::error::AppError;
use crate::domain::entities::TicketSeverity;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Minimum body length (chars).
pub const MIN_RESPONSE_BODY_LEN: usize = 10;

/// Maximum body length (chars).
pub const MAX_RESPONSE_BODY_LEN: usize = 5000;

/// Whitelisted values for `action_proposed`. Keeping this hard-coded server-
/// side means a hostile client cannot inject free-form action strings into
/// audit-grade rows.
pub const ALLOWED_ACTIONS: &[&str] = &[
    "schedule_inspection",
    "request_quote",
    "closed_no_action",
    "escalated_board",
    "other",
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct SyndicResponse {
    pub id: Uuid,
    pub ticket_id: Uuid,
    pub syndic_user_id: Uuid,
    pub body: String,
    pub action_proposed: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl SyndicResponse {
    /// Constructs a new SyndicResponse. Invariants enforced here:
    ///
    /// - `body.len()` ∈ `[MIN_RESPONSE_BODY_LEN, MAX_RESPONSE_BODY_LEN]`
    ///   (after trim);
    /// - `action_proposed` (if `Some`) MUST be in [`ALLOWED_ACTIONS`].
    ///
    /// The constructor is the ONLY supported way to mint a response — no
    /// public setters; the entity is built once and persisted as-is.
    pub fn new(
        ticket_id: Uuid,
        syndic_user_id: Uuid,
        body: String,
        action_proposed: Option<String>,
    ) -> Result<Self, AppError> {
        let trimmed_body = body.trim().to_string();
        if trimmed_body.len() < MIN_RESPONSE_BODY_LEN {
            return Err(AppError::Validation(format!(
                "SyndicResponse body must be at least {} chars",
                MIN_RESPONSE_BODY_LEN
            )));
        }
        if trimmed_body.len() > MAX_RESPONSE_BODY_LEN {
            return Err(AppError::Validation(format!(
                "SyndicResponse body must be at most {} chars",
                MAX_RESPONSE_BODY_LEN
            )));
        }
        if let Some(ref action) = action_proposed {
            let normalised = action.trim().to_lowercase();
            if !ALLOWED_ACTIONS.contains(&normalised.as_str()) {
                return Err(AppError::Validation(format!(
                    "Invalid action_proposed: {} (allowed: {:?})",
                    action, ALLOWED_ACTIONS
                )));
            }
        }
        if ticket_id.is_nil() || syndic_user_id.is_nil() {
            return Err(AppError::Validation(
                "SyndicResponse references must not be nil UUIDs".to_string(),
            ));
        }

        Ok(Self {
            id: Uuid::new_v4(),
            ticket_id,
            syndic_user_id,
            body: trimmed_body,
            action_proposed: action_proposed.map(|a| a.trim().to_lowercase()),
            created_at: Utc::now(),
        })
    }
}

/// SLA policy : maximum acceptable delay between ticket creation and the
/// first syndic response for a given severity tier.
///
/// Used both by the use-case at create time (to compute
/// `Ticket.sla_due_at`) and by the SLA escalation cron job (to detect
/// overdue tickets).
pub fn sla_window_for_severity(severity: TicketSeverity) -> Duration {
    match severity {
        TicketSeverity::Critical => Duration::hours(24),
        TicketSeverity::High => Duration::hours(72),
        TicketSeverity::Normal => Duration::days(5),
        TicketSeverity::Low => Duration::days(10),
    }
}

// ============================================================================
// Tests — taxonomie 4 catégories obligatoire (CRITICAL.md #3, Story 3.7)
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_ids() -> (Uuid, Uuid) {
        (Uuid::new_v4(), Uuid::new_v4())
    }

    fn fixture_body() -> String {
        "Bonjour, j'ai bien noté votre plainte, devis demandé.".to_string()
    }

    // ---- @happy -------------------------------------------------------------

    #[test]
    fn happy_create_response_with_minimal_body_and_no_action() {
        let (ticket_id, syndic) = fixture_ids();
        // 10 chars exactly (minimum bound — see also @edge).
        let body = "0123456789".to_string();
        let r = SyndicResponse::new(ticket_id, syndic, body.clone(), None)
            .expect("10-char body without action must succeed");
        assert_eq!(r.ticket_id, ticket_id);
        assert_eq!(r.syndic_user_id, syndic);
        assert_eq!(r.body, body);
        assert!(r.action_proposed.is_none());
    }

    #[test]
    fn happy_create_response_with_allowed_action() {
        let (ticket_id, syndic) = fixture_ids();
        let r = SyndicResponse::new(
            ticket_id,
            syndic,
            fixture_body(),
            Some("schedule_inspection".to_string()),
        )
        .expect("allowed action must succeed");
        assert_eq!(r.action_proposed.as_deref(), Some("schedule_inspection"));
    }

    #[test]
    fn happy_sla_window_critical_is_24_hours() {
        assert_eq!(
            sla_window_for_severity(TicketSeverity::Critical),
            Duration::hours(24)
        );
    }

    #[test]
    fn happy_sla_window_low_is_ten_days() {
        assert_eq!(
            sla_window_for_severity(TicketSeverity::Low),
            Duration::days(10)
        );
    }

    #[test]
    fn happy_sla_window_high_is_72_hours_and_normal_is_5_days() {
        assert_eq!(
            sla_window_for_severity(TicketSeverity::High),
            Duration::hours(72)
        );
        assert_eq!(
            sla_window_for_severity(TicketSeverity::Normal),
            Duration::days(5)
        );
    }

    #[test]
    fn happy_sla_windows_strictly_decrease_with_severity() {
        // Triage signal: a more severe ticket MUST get a stricter SLA.
        let critical = sla_window_for_severity(TicketSeverity::Critical);
        let high = sla_window_for_severity(TicketSeverity::High);
        let normal = sla_window_for_severity(TicketSeverity::Normal);
        let low = sla_window_for_severity(TicketSeverity::Low);
        assert!(critical < high);
        assert!(high < normal);
        assert!(normal < low);
    }

    // ---- @edge --------------------------------------------------------------

    #[test]
    fn edge_body_exactly_min_len_is_accepted() {
        let (ticket_id, syndic) = fixture_ids();
        let body = "X".repeat(MIN_RESPONSE_BODY_LEN);
        assert!(SyndicResponse::new(ticket_id, syndic, body, None).is_ok());
    }

    #[test]
    fn edge_body_exactly_max_len_is_accepted() {
        let (ticket_id, syndic) = fixture_ids();
        let body = "X".repeat(MAX_RESPONSE_BODY_LEN);
        assert!(SyndicResponse::new(ticket_id, syndic, body, None).is_ok());
    }

    #[test]
    fn edge_body_one_under_min_is_rejected() {
        let (ticket_id, syndic) = fixture_ids();
        let body = "X".repeat(MIN_RESPONSE_BODY_LEN - 1);
        let err = SyndicResponse::new(ticket_id, syndic, body, None).unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }

    #[test]
    fn edge_body_one_over_max_is_rejected() {
        let (ticket_id, syndic) = fixture_ids();
        let body = "X".repeat(MAX_RESPONSE_BODY_LEN + 1);
        let err = SyndicResponse::new(ticket_id, syndic, body, None).unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }

    #[test]
    fn edge_action_normalised_to_lowercase() {
        let (ticket_id, syndic) = fixture_ids();
        let r = SyndicResponse::new(
            ticket_id,
            syndic,
            fixture_body(),
            Some("  Request_Quote  ".to_string()),
        )
        .expect("whitespace + uppercase should be normalised");
        assert_eq!(r.action_proposed.as_deref(), Some("request_quote"));
    }

    // ---- @security ----------------------------------------------------------

    #[test]
    fn security_unknown_action_is_rejected_no_smuggling() {
        let (ticket_id, syndic) = fixture_ids();
        let err = SyndicResponse::new(
            ticket_id,
            syndic,
            fixture_body(),
            Some("rm_minus_rf".to_string()),
        )
        .unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }

    #[test]
    fn security_nil_ticket_id_is_rejected() {
        let err =
            SyndicResponse::new(Uuid::nil(), Uuid::new_v4(), fixture_body(), None).unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }

    #[test]
    fn security_nil_syndic_id_is_rejected() {
        let err =
            SyndicResponse::new(Uuid::new_v4(), Uuid::nil(), fixture_body(), None).unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }

    // ---- @negative ----------------------------------------------------------

    #[test]
    fn negative_empty_body_is_rejected() {
        let (ticket_id, syndic) = fixture_ids();
        let err = SyndicResponse::new(ticket_id, syndic, String::new(), None).unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }

    #[test]
    fn negative_whitespace_only_body_is_rejected() {
        let (ticket_id, syndic) = fixture_ids();
        let err = SyndicResponse::new(ticket_id, syndic, "        ".to_string(), None).unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }

    #[test]
    fn negative_short_body_five_chars_is_rejected() {
        let (ticket_id, syndic) = fixture_ids();
        let err = SyndicResponse::new(ticket_id, syndic, "short".to_string(), None).unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }
}
