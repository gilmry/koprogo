use crate::domain::entities::{Ticket, TicketCategory, TicketPriority, TicketStatus};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
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
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub closed_at: Option<DateTime<Utc>>,
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
            created_at: ticket.created_at,
            updated_at: ticket.updated_at,
            resolved_at: ticket.resolved_at,
            closed_at: ticket.closed_at,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateTicketRequest {
    pub building_id: Uuid,
    pub unit_id: Option<Uuid>,
    pub title: String,
    pub description: String,
    pub category: TicketCategory,
    pub priority: TicketPriority,
}

#[derive(Debug, Deserialize)]
pub struct AssignTicketRequest {
    pub assigned_to: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct ResolveTicketRequest {
    pub resolution_notes: String,
}

#[derive(Debug, Deserialize)]
pub struct CancelTicketRequest {
    pub reason: String,
}

#[derive(Debug, Deserialize)]
pub struct ReopenTicketRequest {
    pub reason: String,
}
