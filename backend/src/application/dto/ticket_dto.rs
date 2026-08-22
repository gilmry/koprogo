use crate::domain::entities::{
    Ticket, TicketCategory, TicketKind, TicketPriority, TicketSeverity, TicketStatus,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct TicketResponse {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub building_id: Uuid,
    pub unit_id: Option<Uuid>,
    pub created_by: Uuid,
    pub assigned_to: Option<Uuid>,
    pub title: String,
    pub description: String,
    pub category: TicketCategory,
    pub priority: TicketPriority,
    pub status: TicketStatus,
    pub resolution_notes: Option<String>,
    pub work_order_sent_at: Option<DateTime<Utc>>,
    /// Story 3.6 (FR31) — Request (default) vs Complaint.
    #[serde(default)]
    pub kind: TicketKind,
    /// Story 3.6 (FR31) — Triage severity (mandatory for Complaint).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub severity: Option<TicketSeverity>,
    /// Story 3.6 (FR31) — Optional incident timestamp.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub incident_date: Option<DateTime<Utc>>,
    /// Story 3.6 (FR31) — URLs / S3-MinIO references attached as evidence.
    #[serde(default)]
    pub evidence_attachments: Vec<String>,
    /// Story 3.6 (FR31) — user_ids témoins.
    #[serde(default)]
    pub witnesses: Vec<Uuid>,
    /// Story 3.7 (FR32) — Syndic SLA deadline (computed from `severity`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sla_due_at: Option<DateTime<Utc>>,
    /// Story 3.7 (FR32) — Timestamp at which SLA was consumed (response in
    /// time or cron escalation).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sla_escalated_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub closed_at: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub requester_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub assigned_to_name: Option<String>,
}

impl From<Ticket> for TicketResponse {
    fn from(ticket: Ticket) -> Self {
        Self {
            id: ticket.id,
            organization_id: ticket.organization_id,
            building_id: ticket.building_id,
            unit_id: ticket.unit_id,
            created_by: ticket.created_by,
            assigned_to: ticket.assigned_to,
            title: ticket.title,
            description: ticket.description,
            category: ticket.category,
            priority: ticket.priority,
            status: ticket.status,
            resolution_notes: ticket.resolution_notes,
            work_order_sent_at: ticket.work_order_sent_at,
            kind: ticket.kind,
            severity: ticket.severity,
            incident_date: ticket.incident_date,
            evidence_attachments: ticket.evidence_attachments,
            witnesses: ticket.witnesses,
            sla_due_at: ticket.sla_due_at,
            sla_escalated_at: ticket.sla_escalated_at,
            created_at: ticket.created_at,
            updated_at: ticket.updated_at,
            resolved_at: ticket.resolved_at,
            closed_at: ticket.closed_at,
            requester_name: None,
            assigned_to_name: None,
        }
    }
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateTicketRequest {
    pub building_id: Uuid,
    pub unit_id: Option<Uuid>,
    pub title: String,
    pub description: String,
    pub category: TicketCategory,
    pub priority: TicketPriority,
    /// Story 3.6 (FR31) — Request (default) vs Complaint. Optional for
    /// backward compatibility with pre-3.6 clients (defaults to Request).
    #[serde(default)]
    pub kind: Option<TicketKind>,
    /// Story 3.6 (FR31) — REQUIRED when `kind = Complaint`.
    #[serde(default)]
    pub severity: Option<TicketSeverity>,
    /// Story 3.6 (FR31) — Optional incident timestamp.
    #[serde(default)]
    pub incident_date: Option<DateTime<Utc>>,
    /// Story 3.6 (FR31) — URL / S3-MinIO references. Up to 10.
    #[serde(default)]
    pub evidence_attachments: Vec<String>,
    /// Story 3.6 (FR31) — Up to 10 witness user_ids (no duplicates).
    #[serde(default)]
    pub witnesses: Vec<Uuid>,
}

/// Story 3.6 (FR31 / INV-24) — Partial update applicable only inside the
/// 5-minute editability window. After that, the use-case returns
/// `AppError::TicketImmutable` (403).
#[derive(Debug, Default, Deserialize, utoipa::ToSchema)]
pub struct UpdateTicketRequest {
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub category: Option<TicketCategory>,
    #[serde(default)]
    pub priority: Option<TicketPriority>,
    #[serde(default)]
    pub severity: Option<TicketSeverity>,
    #[serde(default)]
    pub incident_date: Option<DateTime<Utc>>,
    #[serde(default)]
    pub evidence_attachments: Option<Vec<String>>,
    #[serde(default)]
    pub witnesses: Option<Vec<Uuid>>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct AssignTicketRequest {
    pub assigned_to: Uuid,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct ResolveTicketRequest {
    pub resolution_notes: String,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CancelTicketRequest {
    pub reason: String,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct ReopenTicketRequest {
    pub reason: String,
}
