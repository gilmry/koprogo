use crate::application::dto::{
    AssignTicketRequest, CancelTicketRequest, CreateTicketRequest, ReopenTicketRequest,
    ResolveTicketRequest, TicketResponse,
};
use crate::application::ports::TicketRepository;
use crate::domain::entities::{Ticket, TicketStatus};
use std::sync::Arc;
use uuid::Uuid;

pub struct TicketUseCases {
    ticket_repository: Arc<dyn TicketRepository>,
}

impl TicketUseCases {
    pub fn new(ticket_repository: Arc<dyn TicketRepository>) -> Self {
        Self { ticket_repository }
    }

    /// Create a new maintenance request ticket
    pub async fn create_ticket(
        &self,
        organization_id: Uuid,
        created_by: Uuid,
        request: CreateTicketRequest,
    ) -> Result<TicketResponse, String> {
        let ticket = Ticket::new(
            organization_id,
            request.building_id,
            request.unit_id,
            created_by,
            request.title,
            request.description,
            request.category,
            request.priority,
        )?;

        let created = self.ticket_repository.create(&ticket).await?;
        Ok(TicketResponse::from(created))
    }

    /// Get a ticket by ID
    pub async fn get_ticket(&self, id: Uuid) -> Result<Option<TicketResponse>, String> {
        match self.ticket_repository.find_by_id(id).await? {
            Some(ticket) => Ok(Some(TicketResponse::from(ticket))),
            None => Ok(None),
        }
    }

    /// List all tickets for a building
    pub async fn list_tickets_by_building(
        &self,
        building_id: Uuid,
    ) -> Result<Vec<TicketResponse>, String> {
        let tickets = self.ticket_repository.find_by_building(building_id).await?;
        Ok(tickets.into_iter().map(TicketResponse::from).collect())
    }

    /// List all tickets for an organization
    pub async fn list_tickets_by_organization(
        &self,
        organization_id: Uuid,
    ) -> Result<Vec<TicketResponse>, String> {
        let tickets = self
            .ticket_repository
            .find_by_organization(organization_id)
            .await?;
        Ok(tickets.into_iter().map(TicketResponse::from).collect())
    }

    /// List tickets created by a specific owner
    pub async fn list_my_tickets(&self, created_by: Uuid) -> Result<Vec<TicketResponse>, String> {
        let tickets = self.ticket_repository.find_by_created_by(created_by).await?;
        Ok(tickets.into_iter().map(TicketResponse::from).collect())
    }

    /// List tickets assigned to a specific user (syndic/contractor)
    pub async fn list_assigned_tickets(
        &self,
        assigned_to: Uuid,
    ) -> Result<Vec<TicketResponse>, String> {
        let tickets = self
            .ticket_repository
            .find_by_assigned_to(assigned_to)
            .await?;
        Ok(tickets.into_iter().map(TicketResponse::from).collect())
    }

    /// List tickets by status for a building
    pub async fn list_tickets_by_status(
        &self,
        building_id: Uuid,
        status: TicketStatus,
    ) -> Result<Vec<TicketResponse>, String> {
        let tickets = self
            .ticket_repository
            .find_by_status(building_id, status)
            .await?;
        Ok(tickets.into_iter().map(TicketResponse::from).collect())
    }

    /// Assign a ticket to a user (syndic or contractor)
    pub async fn assign_ticket(
        &self,
        id: Uuid,
        request: AssignTicketRequest,
    ) -> Result<TicketResponse, String> {
        let mut ticket = self
            .ticket_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| "Ticket not found".to_string())?;

        ticket.assign(request.assigned_to)?;

        let updated = self.ticket_repository.update(&ticket).await?;
        Ok(TicketResponse::from(updated))
    }

    /// Start working on a ticket (transition to InProgress)
    pub async fn start_work(&self, id: Uuid) -> Result<TicketResponse, String> {
        let mut ticket = self
            .ticket_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| "Ticket not found".to_string())?;

        ticket.start_work()?;

        let updated = self.ticket_repository.update(&ticket).await?;
        Ok(TicketResponse::from(updated))
    }

    /// Resolve a ticket (mark as work completed)
    pub async fn resolve_ticket(
        &self,
        id: Uuid,
        request: ResolveTicketRequest,
    ) -> Result<TicketResponse, String> {
        let mut ticket = self
            .ticket_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| "Ticket not found".to_string())?;

        ticket.resolve(request.resolution_notes)?;

        let updated = self.ticket_repository.update(&ticket).await?;
        Ok(TicketResponse::from(updated))
    }

    /// Close a ticket (final validation by requester or syndic)
    pub async fn close_ticket(&self, id: Uuid) -> Result<TicketResponse, String> {
        let mut ticket = self
            .ticket_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| "Ticket not found".to_string())?;

        ticket.close()?;

        let updated = self.ticket_repository.update(&ticket).await?;
        Ok(TicketResponse::from(updated))
    }

    /// Cancel a ticket with a reason
    pub async fn cancel_ticket(
        &self,
        id: Uuid,
        request: CancelTicketRequest,
    ) -> Result<TicketResponse, String> {
        let mut ticket = self
            .ticket_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| "Ticket not found".to_string())?;

        ticket.cancel(request.reason)?;

        let updated = self.ticket_repository.update(&ticket).await?;
        Ok(TicketResponse::from(updated))
    }

    /// Reopen a ticket (if incorrectly resolved)
    pub async fn reopen_ticket(
        &self,
        id: Uuid,
        request: ReopenTicketRequest,
    ) -> Result<TicketResponse, String> {
        let mut ticket = self
            .ticket_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| "Ticket not found".to_string())?;

        ticket.reopen(request.reason)?;

        let updated = self.ticket_repository.update(&ticket).await?;
        Ok(TicketResponse::from(updated))
    }

    /// Delete a ticket (only allowed for Open tickets)
    pub async fn delete_ticket(&self, id: Uuid) -> Result<bool, String> {
        let ticket = self
            .ticket_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| "Ticket not found".to_string())?;

        // Only allow deletion of Open tickets
        if ticket.status != TicketStatus::Open {
            return Err("Can only delete tickets with Open status".to_string());
        }

        self.ticket_repository.delete(id).await
    }

    /// Get ticket statistics for a building
    pub async fn get_ticket_statistics(
        &self,
        building_id: Uuid,
    ) -> Result<TicketStatistics, String> {
        let total = self.ticket_repository.count_by_building(building_id).await?;

        let open = self
            .ticket_repository
            .count_by_status(building_id, TicketStatus::Open)
            .await?;

        let in_progress = self
            .ticket_repository
            .count_by_status(building_id, TicketStatus::InProgress)
            .await?;

        let resolved = self
            .ticket_repository
            .count_by_status(building_id, TicketStatus::Resolved)
            .await?;

        let closed = self
            .ticket_repository
            .count_by_status(building_id, TicketStatus::Closed)
            .await?;

        let cancelled = self
            .ticket_repository
            .count_by_status(building_id, TicketStatus::Cancelled)
            .await?;

        Ok(TicketStatistics {
            total,
            open,
            in_progress,
            resolved,
            closed,
            cancelled,
        })
    }

    /// Check for overdue tickets (open for more than specified days)
    pub async fn get_overdue_tickets(
        &self,
        building_id: Uuid,
        max_days: i64,
    ) -> Result<Vec<TicketResponse>, String> {
        let tickets = self.ticket_repository.find_by_building(building_id).await?;

        let overdue: Vec<TicketResponse> = tickets
            .into_iter()
            .filter(|ticket| ticket.is_overdue(max_days))
            .map(TicketResponse::from)
            .collect();

        Ok(overdue)
    }
}

/// Ticket statistics for a building
#[derive(Debug, serde::Serialize)]
pub struct TicketStatistics {
    pub total: i64,
    pub open: i64,
    pub in_progress: i64,
    pub resolved: i64,
    pub closed: i64,
    pub cancelled: i64,
}
