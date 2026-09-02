use crate::application::error::AppError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use uuid::Uuid;

/// Story 3.6 (INV-24) — Ticket is locked to direct edits beyond this window
/// after creation. Subsequent changes go through workflow endpoints (assign,
/// resolve, …) which carry their own typed errors.
pub const TICKET_EDIT_WINDOW_MINUTES: i64 = 5;

/// Story 3.6 (FR31) — Hard cap on the number of evidence URLs accepted on a
/// single ticket (anti-abuse + UI sanity).
pub const MAX_EVIDENCE_ATTACHMENTS: usize = 10;

/// Story 3.6 (FR31) — Hard cap on the number of witnesses on a single ticket.
pub const MAX_WITNESSES: usize = 10;

/// Story 3.6 (FR31) — Distinguishes a maintenance request (default) from a
/// formal complaint (incident report → triage / mediation workflow).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum TicketKind {
    /// Standard maintenance request (backward-compat default).
    #[default]
    Request,
    /// Formal complaint / incident report (community moderation workflow).
    Complaint,
}

impl std::fmt::Display for TicketKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TicketKind::Request => write!(f, "request"),
            TicketKind::Complaint => write!(f, "complaint"),
        }
    }
}

impl std::str::FromStr for TicketKind {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "request" => Ok(TicketKind::Request),
            "complaint" => Ok(TicketKind::Complaint),
            other => Err(AppError::Validation(format!(
                "Invalid ticket kind: {}",
                other
            ))),
        }
    }
}

/// Story 3.6 (FR31) — Severity tier for ticket triage. Ordered so callers
/// can compare (e.g. `>= High`) when deciding alerting / SLA targets.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, utoipa::ToSchema,
)]
#[serde(rename_all = "lowercase")]
pub enum TicketSeverity {
    Low,
    Normal,
    High,
    Critical,
}

impl std::fmt::Display for TicketSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TicketSeverity::Low => write!(f, "low"),
            TicketSeverity::Normal => write!(f, "normal"),
            TicketSeverity::High => write!(f, "high"),
            TicketSeverity::Critical => write!(f, "critical"),
        }
    }
}

impl std::str::FromStr for TicketSeverity {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "low" => Ok(TicketSeverity::Low),
            "normal" => Ok(TicketSeverity::Normal),
            "high" => Ok(TicketSeverity::High),
            "critical" => Ok(TicketSeverity::Critical),
            other => Err(AppError::Validation(format!(
                "Invalid ticket severity: {}",
                other
            ))),
        }
    }
}

/// Ticket Category - Types of maintenance requests
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, utoipa::ToSchema)]
pub enum TicketCategory {
    Plumbing,    // Plomberie
    Electrical,  // Électricité
    Heating,     // Chauffage
    CommonAreas, // Parties communes
    Elevator,    // Ascenseur
    Security,    // Sécurité
    Cleaning,    // Nettoyage
    Landscaping, // Espaces verts
    Other,       // Autre
}

/// Ticket Priority
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd, utoipa::ToSchema)]
pub enum TicketPriority {
    Low,      // Basse
    Medium,   // Moyenne
    High,     // Haute
    Critical, // Critique/Urgente
}

/// Ticket Status - Workflow states
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, utoipa::ToSchema)]
pub enum TicketStatus {
    Open,       // Ouvert (nouveau ticket)
    InProgress, // En cours de traitement
    Resolved,   // Résolu (intervention terminée)
    Closed,     // Fermé (validé par le demandeur)
    Cancelled,  // Annulé
}

/// Ticket Entity - Maintenance request from owners
///
/// Represents a maintenance request (ticket) submitted by a co-owner
/// for issues in the building (plumbing, electrical, etc.).
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct Ticket {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub building_id: Uuid,
    pub unit_id: Option<Uuid>, // If specific to a unit, None for common areas
    pub created_by: Uuid,      // Owner who created the ticket
    pub assigned_to: Option<Uuid>, // User (syndic, contractor) assigned
    pub title: String,
    pub description: String,
    pub category: TicketCategory,
    pub priority: TicketPriority,
    pub status: TicketStatus,
    pub resolution_notes: Option<String>, // Notes from resolver
    pub work_order_sent_at: Option<DateTime<Utc>>, // When magic link PWA was sent to contractor (Issue #309)
    /// Story 3.6 (FR31) — Request (default) vs Complaint. Drives triage UI and
    /// (for Complaint) the severity invariant.
    #[serde(default)]
    pub kind: TicketKind,
    /// Story 3.6 (FR31) — Triage tier. MUST be `Some` when `kind == Complaint`
    /// (enforced by `validate_invariants`).
    #[serde(default)]
    pub severity: Option<TicketSeverity>,
    /// Story 3.6 (FR31) — Optional incident timestamp (legal evidence). Free
    /// of an upper bound here; the use-case layer may compare against
    /// `created_at` if needed.
    #[serde(default)]
    pub incident_date: Option<DateTime<Utc>>,
    /// Story 3.6 (FR31) — Up to [`MAX_EVIDENCE_ATTACHMENTS`] URLs / object-
    /// store references attached as proof.
    #[serde(default)]
    pub evidence_attachments: Vec<String>,
    /// Story 3.6 (FR31) — Up to [`MAX_WITNESSES`] user_ids witnesses. No
    /// duplicates (enforced by `validate_invariants`).
    #[serde(default)]
    pub witnesses: Vec<Uuid>,
    /// Story 3.7 (FR32) — Deadline before which the syndic must respond to
    /// the ticket. Computed by [`Ticket::new_with_kind`] from `severity` via
    /// [`crate::domain::entities::sla_window_for_severity`]. `None` for
    /// pre-3.7 tickets and for tickets without a severity tier.
    #[serde(default)]
    pub sla_due_at: Option<DateTime<Utc>>,
    /// Story 3.7 (FR32) — Timestamp at which the SLA was "consumed": either
    /// because a syndic responded in time (pre-empts escalation) or because
    /// the cron job marked the ticket as overdue. `None` while the SLA is
    /// still open.
    #[serde(default)]
    pub sla_escalated_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub closed_at: Option<DateTime<Utc>>,
}

impl Ticket {
    /// Create a new ticket
    pub fn new(
        organization_id: Uuid,
        building_id: Uuid,
        unit_id: Option<Uuid>,
        created_by: Uuid,
        title: String,
        description: String,
        category: TicketCategory,
        priority: TicketPriority,
    ) -> Result<Self, String> {
        // Validation
        if title.trim().is_empty() {
            return Err("Title cannot be empty".to_string());
        }

        if title.len() > 200 {
            return Err("Title cannot exceed 200 characters".to_string());
        }

        if description.trim().is_empty() {
            return Err("Description cannot be empty".to_string());
        }

        if description.len() > 5000 {
            return Err("Description cannot exceed 5000 characters".to_string());
        }

        let now = Utc::now();

        Ok(Self {
            id: Uuid::new_v4(),
            organization_id,
            building_id,
            unit_id,
            created_by,
            assigned_to: None,
            title,
            description,
            category,
            priority,
            status: TicketStatus::Open,
            resolution_notes: None,
            work_order_sent_at: None,
            kind: TicketKind::Request,
            severity: None,
            incident_date: None,
            evidence_attachments: Vec::new(),
            witnesses: Vec::new(),
            // Story 3.7 — legacy `new()` has no severity → no SLA.
            sla_due_at: None,
            sla_escalated_at: None,
            created_at: now,
            updated_at: now,
            resolved_at: None,
            closed_at: None,
        })
    }

    /// Story 3.6 (FR31) — Constructor supporting the full Story 3.6 surface
    /// (kind / severity / incident_date / evidence / witnesses). Returns
    /// `AppError` natively so the use-case layer doesn't have to bridge a
    /// legacy `String` error.
    ///
    /// Invariants enforced here in addition to the legacy `new()` checks:
    /// - `kind == Complaint` → `severity.is_some()`
    /// - `evidence_attachments.len() <= MAX_EVIDENCE_ATTACHMENTS`
    /// - `witnesses.len() <= MAX_WITNESSES`
    /// - no duplicate witnesses
    /// - `created_by` not in `witnesses` (self-witnessing is incoherent)
    #[allow(clippy::too_many_arguments)]
    pub fn new_with_kind(
        organization_id: Uuid,
        building_id: Uuid,
        unit_id: Option<Uuid>,
        created_by: Uuid,
        title: String,
        description: String,
        category: TicketCategory,
        priority: TicketPriority,
        kind: TicketKind,
        severity: Option<TicketSeverity>,
        incident_date: Option<DateTime<Utc>>,
        evidence_attachments: Vec<String>,
        witnesses: Vec<Uuid>,
    ) -> Result<Self, AppError> {
        // Reuse the legacy validation surface (title/description) to stay DRY.
        // Bridge its `String` errors into `AppError::Validation` so the caller
        // sees a 400, not a 500.
        let mut t = Ticket::new(
            organization_id,
            building_id,
            unit_id,
            created_by,
            title,
            description,
            category,
            priority,
        )
        .map_err(AppError::Validation)?;

        t.kind = kind;
        t.severity = severity;
        t.incident_date = incident_date;
        t.evidence_attachments = evidence_attachments;
        t.witnesses = witnesses;

        // Story 3.7 — derive `sla_due_at` from severity at create-time. We
        // compute it on every ticket carrying a severity (Request or
        // Complaint), since the brief allows severity on Request too.
        t.sla_due_at = severity.map(|s| {
            t.created_at + crate::domain::entities::syndic_response::sla_window_for_severity(s)
        });

        t.validate_invariants()?;
        Ok(t)
    }

    // ------------------------------------------------------------------
    // Story 3.6 (INV-24) — Immutability window.
    // ------------------------------------------------------------------

    /// Returns `true` when `t` falls strictly within the
    /// [`TICKET_EDIT_WINDOW_MINUTES`] window after `created_at`.
    pub fn is_editable_at(&self, t: DateTime<Utc>) -> bool {
        let window = chrono::Duration::minutes(TICKET_EDIT_WINDOW_MINUTES);
        t - self.created_at < window
    }

    /// Convenience wrapper using `Utc::now()` — see [`Self::is_editable_at`].
    pub fn is_currently_editable(&self) -> bool {
        self.is_editable_at(Utc::now())
    }

    /// Story 3.6 (FR31) — Validate every Complaint/evidence/witness invariant
    /// at once. Used by both constructors and the PATCH use-case (after the
    /// caller mutates fields) so the entity never escapes in an inconsistent
    /// state.
    ///
    /// Returns `AppError::Validation` with a self-describing message on the
    /// first violation found.
    pub fn validate_invariants(&self) -> Result<(), AppError> {
        if self.kind == TicketKind::Complaint && self.severity.is_none() {
            return Err(AppError::Validation(
                "Complaint tickets require an explicit severity".to_string(),
            ));
        }
        if self.evidence_attachments.len() > MAX_EVIDENCE_ATTACHMENTS {
            return Err(AppError::Validation(format!(
                "Too many evidence attachments (max {})",
                MAX_EVIDENCE_ATTACHMENTS
            )));
        }
        if self.witnesses.len() > MAX_WITNESSES {
            return Err(AppError::Validation(format!(
                "Too many witnesses (max {})",
                MAX_WITNESSES
            )));
        }
        if self.witnesses.contains(&self.created_by) {
            return Err(AppError::Validation(
                "Ticket creator cannot also be listed as a witness".to_string(),
            ));
        }
        let mut seen: HashSet<Uuid> = HashSet::with_capacity(self.witnesses.len());
        for w in &self.witnesses {
            if !seen.insert(*w) {
                return Err(AppError::Validation(
                    "Duplicate witness in ticket".to_string(),
                ));
            }
        }
        // Evidence URLs: trim sanity (empty strings forbidden).
        if self
            .evidence_attachments
            .iter()
            .any(|a| a.trim().is_empty())
        {
            return Err(AppError::Validation(
                "Evidence attachment URL cannot be empty".to_string(),
            ));
        }
        Ok(())
    }

    /// Assign ticket to a user (syndic or contractor)
    pub fn assign(&mut self, user_id: Uuid) -> Result<(), String> {
        if self.status == TicketStatus::Closed || self.status == TicketStatus::Cancelled {
            return Err("Cannot assign a closed or cancelled ticket".to_string());
        }

        self.assigned_to = Some(user_id);
        self.updated_at = Utc::now();

        // Auto-transition to InProgress if still Open
        if self.status == TicketStatus::Open {
            self.status = TicketStatus::InProgress;
        }

        Ok(())
    }

    /// Mark ticket as in progress
    pub fn start_work(&mut self) -> Result<(), String> {
        match self.status {
            TicketStatus::Open => {
                self.status = TicketStatus::InProgress;
                self.updated_at = Utc::now();
                Ok(())
            }
            TicketStatus::InProgress => Ok(()), // Already in progress
            _ => Err(format!(
                "Cannot start work on ticket in status {:?}",
                self.status
            )),
        }
    }

    /// Resolve ticket (work completed)
    pub fn resolve(&mut self, resolution_notes: String) -> Result<(), String> {
        if resolution_notes.trim().is_empty() {
            return Err("Resolution notes are required".to_string());
        }

        if resolution_notes.len() > 2000 {
            return Err("Resolution notes cannot exceed 2000 characters".to_string());
        }

        match self.status {
            TicketStatus::Open | TicketStatus::InProgress => {
                self.status = TicketStatus::Resolved;
                self.resolution_notes = Some(resolution_notes);
                self.resolved_at = Some(Utc::now());
                self.updated_at = Utc::now();
                Ok(())
            }
            TicketStatus::Resolved => {
                // Allow updating resolution notes
                self.resolution_notes = Some(resolution_notes);
                self.updated_at = Utc::now();
                Ok(())
            }
            _ => Err(format!("Cannot resolve ticket in status {:?}", self.status)),
        }
    }

    /// Close ticket (validation by requester or syndic)
    pub fn close(&mut self) -> Result<(), String> {
        match self.status {
            TicketStatus::Resolved => {
                self.status = TicketStatus::Closed;
                self.closed_at = Some(Utc::now());
                self.updated_at = Utc::now();
                Ok(())
            }
            TicketStatus::Closed => Ok(()), // Already closed
            _ => Err(format!(
                "Cannot close ticket in status {:?}. Must be Resolved first.",
                self.status
            )),
        }
    }

    /// Cancel ticket
    pub fn cancel(&mut self, reason: String) -> Result<(), String> {
        if self.status == TicketStatus::Closed {
            return Err("Cannot cancel an already closed ticket".to_string());
        }

        if reason.trim().is_empty() {
            return Err("Cancellation reason is required".to_string());
        }

        self.status = TicketStatus::Cancelled;
        self.resolution_notes = Some(format!("CANCELLED: {}", reason));
        self.updated_at = Utc::now();

        Ok(())
    }

    /// Reopen ticket (if incorrectly resolved)
    pub fn reopen(&mut self, reason: String) -> Result<(), String> {
        if self.status != TicketStatus::Resolved && self.status != TicketStatus::Closed {
            return Err("Can only reopen resolved or closed tickets".to_string());
        }

        if reason.trim().is_empty() {
            return Err("Reopen reason is required".to_string());
        }

        self.status = TicketStatus::InProgress;
        self.resolution_notes = Some(format!(
            "{}\n\nREOPENED: {}",
            self.resolution_notes.as_deref().unwrap_or(""),
            reason
        ));
        self.resolved_at = None;
        self.closed_at = None;
        self.updated_at = Utc::now();

        Ok(())
    }

    /// Send work order to contractor (magic link PWA) - Issue #309
    /// Validates ticket is in InProgress status (must be assigned)
    pub fn send_work_order_to_contractor(&mut self) -> Result<(), String> {
        if self.status != TicketStatus::InProgress {
            return Err(
                "Can only send work order to contractor for tickets in InProgress status"
                    .to_string(),
            );
        }

        if self.assigned_to.is_none() {
            return Err(
                "Ticket must be assigned to a contractor before sending work order".to_string(),
            );
        }

        self.work_order_sent_at = Some(Utc::now());
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Check if ticket is overdue (open for more than X days)
    pub fn is_overdue(&self, max_days: i64) -> bool {
        if self.status == TicketStatus::Closed || self.status == TicketStatus::Cancelled {
            return false;
        }

        let now = Utc::now();
        let age = now - self.created_at;

        age.num_days() > max_days
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_ticket_success() {
        let ticket = Ticket::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Some(Uuid::new_v4()),
            Uuid::new_v4(),
            "Fuite d'eau salle de bain".to_string(),
            "L'eau coule du plafond de la salle de bain".to_string(),
            TicketCategory::Plumbing,
            TicketPriority::High,
        );

        assert!(ticket.is_ok());
        let ticket = ticket.unwrap();
        assert_eq!(ticket.status, TicketStatus::Open);
        assert!(ticket.assigned_to.is_none());
    }

    #[test]
    fn test_create_ticket_empty_title() {
        let result = Ticket::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            None,
            Uuid::new_v4(),
            "   ".to_string(),
            "Description".to_string(),
            TicketCategory::Plumbing,
            TicketPriority::Low,
        );

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Title cannot be empty");
    }

    #[test]
    fn test_assign_ticket() {
        let mut ticket = Ticket::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            None,
            Uuid::new_v4(),
            "Test".to_string(),
            "Test description".to_string(),
            TicketCategory::Electrical,
            TicketPriority::Medium,
        )
        .unwrap();

        let contractor_id = Uuid::new_v4();
        let result = ticket.assign(contractor_id);

        assert!(result.is_ok());
        assert_eq!(ticket.assigned_to, Some(contractor_id));
        assert_eq!(ticket.status, TicketStatus::InProgress); // Auto-transitioned
    }

    #[test]
    fn test_resolve_ticket() {
        let mut ticket = Ticket::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            None,
            Uuid::new_v4(),
            "Test".to_string(),
            "Test description".to_string(),
            TicketCategory::Heating,
            TicketPriority::Low,
        )
        .unwrap();

        ticket.start_work().unwrap();

        let result = ticket.resolve("Chaudière réparée, pièce remplacée".to_string());

        assert!(result.is_ok());
        assert_eq!(ticket.status, TicketStatus::Resolved);
        assert!(ticket.resolved_at.is_some());
        assert!(ticket.resolution_notes.is_some());
    }

    #[test]
    fn test_close_ticket() {
        let mut ticket = Ticket::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            None,
            Uuid::new_v4(),
            "Test".to_string(),
            "Test description".to_string(),
            TicketCategory::CommonAreas,
            TicketPriority::Medium,
        )
        .unwrap();

        ticket.start_work().unwrap();
        ticket.resolve("Fixed".to_string()).unwrap();

        let result = ticket.close();

        assert!(result.is_ok());
        assert_eq!(ticket.status, TicketStatus::Closed);
        assert!(ticket.closed_at.is_some());
    }

    #[test]
    fn test_cannot_close_open_ticket() {
        let mut ticket = Ticket::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            None,
            Uuid::new_v4(),
            "Test".to_string(),
            "Test description".to_string(),
            TicketCategory::Elevator,
            TicketPriority::Critical,
        )
        .unwrap();

        let result = ticket.close();

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Must be Resolved first"));
    }

    #[test]
    fn test_cancel_ticket() {
        let mut ticket = Ticket::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            None,
            Uuid::new_v4(),
            "Test".to_string(),
            "Test description".to_string(),
            TicketCategory::Other,
            TicketPriority::Low,
        )
        .unwrap();

        let result = ticket.cancel("Erreur de déclaration".to_string());

        assert!(result.is_ok());
        assert_eq!(ticket.status, TicketStatus::Cancelled);
    }

    #[test]
    fn test_reopen_ticket() {
        let mut ticket = Ticket::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            None,
            Uuid::new_v4(),
            "Test".to_string(),
            "Test description".to_string(),
            TicketCategory::Plumbing,
            TicketPriority::High,
        )
        .unwrap();

        ticket.start_work().unwrap();
        ticket.resolve("Fixed".to_string()).unwrap();
        ticket.close().unwrap();

        let result = ticket.reopen("Problème persiste".to_string());

        assert!(result.is_ok());
        assert_eq!(ticket.status, TicketStatus::InProgress);
        assert!(ticket.closed_at.is_none());
        assert!(ticket.resolution_notes.unwrap().contains("REOPENED"));
    }

    #[test]
    fn test_is_overdue() {
        let mut ticket = Ticket::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            None,
            Uuid::new_v4(),
            "Test".to_string(),
            "Test description".to_string(),
            TicketCategory::Plumbing,
            TicketPriority::High,
        )
        .unwrap();

        // Simulate old ticket (10 days ago)
        ticket.created_at = Utc::now() - chrono::Duration::days(10);

        assert!(ticket.is_overdue(5));
        assert!(!ticket.is_overdue(15));

        // Closed tickets are never overdue
        ticket.status = TicketStatus::Closed;
        assert!(!ticket.is_overdue(5));
    }

    #[test]
    fn test_send_work_order_success() {
        let mut ticket = Ticket::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            None,
            Uuid::new_v4(),
            "Test".to_string(),
            "Test description".to_string(),
            TicketCategory::Plumbing,
            TicketPriority::High,
        )
        .unwrap();

        // Assign contractor
        ticket.assign(Uuid::new_v4()).unwrap();
        assert_eq!(ticket.status, TicketStatus::InProgress);

        // Send work order
        let result = ticket.send_work_order_to_contractor();
        assert!(result.is_ok());
        assert!(ticket.work_order_sent_at.is_some());
    }

    #[test]
    fn test_send_work_order_requires_assignment() {
        let mut ticket = Ticket::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            None,
            Uuid::new_v4(),
            "Test".to_string(),
            "Test description".to_string(),
            TicketCategory::Electrical,
            TicketPriority::Medium,
        )
        .unwrap();

        // Try to send without assigning
        let result = ticket.send_work_order_to_contractor();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("InProgress"));
    }

    #[test]
    fn test_send_work_order_requires_in_progress() {
        let mut ticket = Ticket::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            None,
            Uuid::new_v4(),
            "Test".to_string(),
            "Test description".to_string(),
            TicketCategory::Heating,
            TicketPriority::Low,
        )
        .unwrap();

        let contractor_id = Uuid::new_v4();
        ticket.assign(contractor_id).unwrap();
        ticket.start_work().unwrap();
        ticket.resolve("Fixed".to_string()).unwrap();

        // Try to send work order on resolved ticket
        let result = ticket.send_work_order_to_contractor();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("InProgress status"));
    }

    // ========================================================================
    // Story 3.6 — Complaint / evidence / witnesses / INV-24 (FR31)
    // 4-cat taxonomy: @happy / @edge / @security / @negative
    // ========================================================================

    fn make_request_ticket() -> Ticket {
        Ticket::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            None,
            Uuid::new_v4(),
            "Title".to_string(),
            "Description".to_string(),
            TicketCategory::Plumbing,
            TicketPriority::Medium,
        )
        .unwrap()
    }

    // ---- @happy -------------------------------------------------------------

    #[test]
    fn happy_legacy_new_defaults_kind_to_request_and_empty_complaint_fields() {
        let t = make_request_ticket();
        assert_eq!(t.kind, TicketKind::Request);
        assert!(t.severity.is_none());
        assert!(t.incident_date.is_none());
        assert!(t.evidence_attachments.is_empty());
        assert!(t.witnesses.is_empty());
        assert!(t.validate_invariants().is_ok());
        // Story 3.7 — no severity → no SLA.
        assert!(t.sla_due_at.is_none());
        assert!(t.sla_escalated_at.is_none());
    }

    // ---- Story 3.7 — SLA computation at create-time -------------------------

    #[test]
    fn happy_new_with_kind_critical_sets_sla_due_at_24h_after_created_at() {
        let t = Ticket::new_with_kind(
            Uuid::new_v4(),
            Uuid::new_v4(),
            None,
            Uuid::new_v4(),
            "T".to_string(),
            "D".to_string(),
            TicketCategory::Other,
            TicketPriority::High,
            TicketKind::Complaint,
            Some(TicketSeverity::Critical),
            None,
            Vec::new(),
            Vec::new(),
        )
        .expect("complaint critical must succeed");

        let sla = t.sla_due_at.expect("Critical severity must produce a SLA");
        // 24h window for Critical (cf. sla_window_for_severity).
        assert_eq!(sla - t.created_at, chrono::Duration::hours(24));
        assert!(t.sla_escalated_at.is_none());
    }

    #[test]
    fn happy_new_with_kind_no_severity_leaves_sla_due_at_none() {
        // Request without severity — pre-3.7 retro-compat path.
        let t = Ticket::new_with_kind(
            Uuid::new_v4(),
            Uuid::new_v4(),
            None,
            Uuid::new_v4(),
            "T".to_string(),
            "D".to_string(),
            TicketCategory::Other,
            TicketPriority::Low,
            TicketKind::Request,
            None,
            None,
            Vec::new(),
            Vec::new(),
        )
        .expect("Request without severity must succeed");
        assert!(t.sla_due_at.is_none());
    }

    #[test]
    fn happy_new_with_kind_complaint_critical_with_evidence_and_witnesses() {
        let org = Uuid::new_v4();
        let bld = Uuid::new_v4();
        let creator = Uuid::new_v4();
        let evidence = vec![
            "https://obj.store/a.jpg".to_string(),
            "https://obj.store/b.jpg".to_string(),
            "https://obj.store/c.jpg".to_string(),
        ];
        let witnesses = vec![Uuid::new_v4(), Uuid::new_v4()];

        let t = Ticket::new_with_kind(
            org,
            bld,
            None,
            creator,
            "Bruit nocturne récurrent".to_string(),
            "Tapage du 3e étage, plaintes répétées".to_string(),
            TicketCategory::Other,
            TicketPriority::High,
            TicketKind::Complaint,
            Some(TicketSeverity::Critical),
            Some(Utc::now() - chrono::Duration::hours(2)),
            evidence.clone(),
            witnesses.clone(),
        )
        .expect("Complaint with severity + ≤10 evidence + ≤10 witnesses must succeed");

        assert_eq!(t.kind, TicketKind::Complaint);
        assert_eq!(t.severity, Some(TicketSeverity::Critical));
        assert_eq!(t.evidence_attachments, evidence);
        assert_eq!(t.witnesses, witnesses);
        assert!(t.incident_date.is_some());
    }

    #[test]
    fn happy_kind_severity_display_and_from_str_roundtrip() {
        use std::str::FromStr;
        for k in [TicketKind::Request, TicketKind::Complaint] {
            assert_eq!(TicketKind::from_str(&k.to_string()).unwrap(), k);
        }
        for s in [
            TicketSeverity::Low,
            TicketSeverity::Normal,
            TicketSeverity::High,
            TicketSeverity::Critical,
        ] {
            assert_eq!(TicketSeverity::from_str(&s.to_string()).unwrap(), s);
        }
    }

    #[test]
    fn happy_severity_ordering_supports_high_alerting_threshold() {
        // Ordering used by triage code: alert if `severity >= High`.
        assert!(TicketSeverity::Critical > TicketSeverity::High);
        assert!(TicketSeverity::High > TicketSeverity::Normal);
        assert!(TicketSeverity::Normal > TicketSeverity::Low);
    }

    // ---- @edge --------------------------------------------------------------

    #[test]
    fn edge_complaint_with_no_evidence_is_allowed_text_only_path() {
        // The UI flags "no evidence attached" but the entity accepts it.
        let creator = Uuid::new_v4();
        let t = Ticket::new_with_kind(
            Uuid::new_v4(),
            Uuid::new_v4(),
            None,
            creator,
            "Plainte textuelle".to_string(),
            "Aucune photo dispo".to_string(),
            TicketCategory::Security,
            TicketPriority::Medium,
            TicketKind::Complaint,
            Some(TicketSeverity::Normal),
            None,
            Vec::new(),
            Vec::new(),
        )
        .expect("Complaint without evidence/witnesses must be allowed (text-only)");
        assert!(t.evidence_attachments.is_empty());
    }

    #[test]
    fn edge_exactly_max_evidence_attachments_is_accepted() {
        let evidence: Vec<String> = (0..MAX_EVIDENCE_ATTACHMENTS)
            .map(|i| format!("https://e.test/{i}"))
            .collect();
        let t = Ticket::new_with_kind(
            Uuid::new_v4(),
            Uuid::new_v4(),
            None,
            Uuid::new_v4(),
            "T".to_string(),
            "D".to_string(),
            TicketCategory::Other,
            TicketPriority::Low,
            TicketKind::Complaint,
            Some(TicketSeverity::Low),
            None,
            evidence,
            Vec::new(),
        );
        assert!(
            t.is_ok(),
            "exactly {} evidence URLs is on-bound",
            MAX_EVIDENCE_ATTACHMENTS
        );
    }

    #[test]
    fn edge_one_over_max_evidence_attachments_is_rejected() {
        let evidence: Vec<String> = (0..(MAX_EVIDENCE_ATTACHMENTS + 1))
            .map(|i| format!("https://e.test/{i}"))
            .collect();
        let err = Ticket::new_with_kind(
            Uuid::new_v4(),
            Uuid::new_v4(),
            None,
            Uuid::new_v4(),
            "T".to_string(),
            "D".to_string(),
            TicketCategory::Other,
            TicketPriority::Low,
            TicketKind::Complaint,
            Some(TicketSeverity::Low),
            None,
            evidence,
            Vec::new(),
        )
        .unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }

    #[test]
    fn edge_is_editable_at_just_under_window_returns_true() {
        let mut t = make_request_ticket();
        let now = Utc::now();
        t.created_at = now;
        // 4m59s
        let probe = now + chrono::Duration::seconds(4 * 60 + 59);
        assert!(t.is_editable_at(probe));
    }

    #[test]
    fn edge_is_editable_at_just_past_window_returns_false() {
        let mut t = make_request_ticket();
        let now = Utc::now();
        t.created_at = now;
        // 5m1s — INV-24 locks the ticket
        let probe = now + chrono::Duration::seconds(5 * 60 + 1);
        assert!(!t.is_editable_at(probe));
    }

    // ---- @security ----------------------------------------------------------

    #[test]
    fn security_duplicate_witnesses_are_rejected() {
        let dup = Uuid::new_v4();
        let err = Ticket::new_with_kind(
            Uuid::new_v4(),
            Uuid::new_v4(),
            None,
            Uuid::new_v4(),
            "T".to_string(),
            "D".to_string(),
            TicketCategory::Other,
            TicketPriority::Low,
            TicketKind::Complaint,
            Some(TicketSeverity::Normal),
            None,
            Vec::new(),
            vec![dup, dup],
        )
        .unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }

    #[test]
    fn security_request_with_severity_is_allowed_severity_not_exclusive_to_complaint() {
        // FR31 — severity may decorate a Request too (e.g. internal triage).
        let t = Ticket::new_with_kind(
            Uuid::new_v4(),
            Uuid::new_v4(),
            None,
            Uuid::new_v4(),
            "T".to_string(),
            "D".to_string(),
            TicketCategory::Heating,
            TicketPriority::Medium,
            TicketKind::Request,
            Some(TicketSeverity::High),
            None,
            Vec::new(),
            Vec::new(),
        )
        .expect("Request + severity must be accepted (severity not exclusive)");
        assert_eq!(t.kind, TicketKind::Request);
        assert_eq!(t.severity, Some(TicketSeverity::High));
    }

    #[test]
    fn security_creator_listed_as_witness_is_rejected() {
        let creator = Uuid::new_v4();
        let err = Ticket::new_with_kind(
            Uuid::new_v4(),
            Uuid::new_v4(),
            None,
            creator,
            "T".to_string(),
            "D".to_string(),
            TicketCategory::Other,
            TicketPriority::Low,
            TicketKind::Complaint,
            Some(TicketSeverity::Normal),
            None,
            Vec::new(),
            vec![creator],
        )
        .unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }

    // ---- @negative ----------------------------------------------------------

    #[test]
    fn negative_complaint_without_severity_is_rejected() {
        let err = Ticket::new_with_kind(
            Uuid::new_v4(),
            Uuid::new_v4(),
            None,
            Uuid::new_v4(),
            "T".to_string(),
            "D".to_string(),
            TicketCategory::Other,
            TicketPriority::Low,
            TicketKind::Complaint,
            None, // ← missing
            None,
            Vec::new(),
            Vec::new(),
        )
        .unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }

    #[test]
    fn negative_eleven_witnesses_is_rejected() {
        let witnesses: Vec<Uuid> = (0..(MAX_WITNESSES + 1)).map(|_| Uuid::new_v4()).collect();
        let err = Ticket::new_with_kind(
            Uuid::new_v4(),
            Uuid::new_v4(),
            None,
            Uuid::new_v4(),
            "T".to_string(),
            "D".to_string(),
            TicketCategory::Other,
            TicketPriority::Low,
            TicketKind::Complaint,
            Some(TicketSeverity::Low),
            None,
            Vec::new(),
            witnesses,
        )
        .unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }

    #[test]
    fn negative_empty_evidence_url_is_rejected() {
        let err = Ticket::new_with_kind(
            Uuid::new_v4(),
            Uuid::new_v4(),
            None,
            Uuid::new_v4(),
            "T".to_string(),
            "D".to_string(),
            TicketCategory::Other,
            TicketPriority::Low,
            TicketKind::Complaint,
            Some(TicketSeverity::Low),
            None,
            vec!["   ".to_string()],
            Vec::new(),
        )
        .unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }

    #[test]
    fn negative_invalid_kind_string_is_rejected() {
        use std::str::FromStr;
        let err = TicketKind::from_str("incident").unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }

    #[test]
    fn negative_invalid_severity_string_is_rejected() {
        use std::str::FromStr;
        let err = TicketSeverity::from_str("urgent").unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }
}
