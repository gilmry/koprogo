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
        let tickets = self
            .ticket_repository
            .find_by_created_by(created_by)
            .await?;
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
        let total = self
            .ticket_repository
            .count_by_building(building_id)
            .await?;

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

    /// Get ticket statistics for an organization (all buildings)
    pub async fn get_ticket_statistics_by_organization(
        &self,
        organization_id: Uuid,
    ) -> Result<TicketStatistics, String> {
        let total = self
            .ticket_repository
            .count_by_organization(organization_id)
            .await?;

        let open = self
            .ticket_repository
            .count_by_organization_and_status(organization_id, TicketStatus::Open)
            .await?;

        let in_progress = self
            .ticket_repository
            .count_by_organization_and_status(organization_id, TicketStatus::InProgress)
            .await?;

        let resolved = self
            .ticket_repository
            .count_by_organization_and_status(organization_id, TicketStatus::Resolved)
            .await?;

        let closed = self
            .ticket_repository
            .count_by_organization_and_status(organization_id, TicketStatus::Closed)
            .await?;

        let cancelled = self
            .ticket_repository
            .count_by_organization_and_status(organization_id, TicketStatus::Cancelled)
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

    /// Check for overdue tickets in organization (all buildings)
    pub async fn get_overdue_tickets_by_organization(
        &self,
        organization_id: Uuid,
        max_days: i64,
    ) -> Result<Vec<TicketResponse>, String> {
        let tickets = self
            .ticket_repository
            .find_by_organization(organization_id)
            .await?;

        let overdue: Vec<TicketResponse> = tickets
            .into_iter()
            .filter(|ticket| ticket.is_overdue(max_days))
            .map(TicketResponse::from)
            .collect();

        Ok(overdue)
    }

    /// Send work order to contractor via PWA magic link (Issue #309)
    /// Validates ticket is in InProgress status and sends work order notification
    pub async fn send_work_order(&self, ticket_id: Uuid) -> Result<TicketResponse, String> {
        let mut ticket = self
            .ticket_repository
            .find_by_id(ticket_id)
            .await?
            .ok_or_else(|| "Ticket not found".to_string())?;

        ticket.send_work_order_to_contractor()?;
        let updated = self.ticket_repository.update(&ticket).await?;

        Ok(TicketResponse::from(updated))
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::ports::TicketRepository;
    use crate::domain::entities::{Ticket, TicketCategory, TicketPriority, TicketStatus};
    use async_trait::async_trait;
    use std::collections::HashMap;
    use std::sync::Mutex;

    // ── Manual mock for TicketRepository ──────────────────────────────

    struct MockTicketRepository {
        tickets: Mutex<HashMap<Uuid, Ticket>>,
    }

    impl MockTicketRepository {
        fn new() -> Self {
            Self {
                tickets: Mutex::new(HashMap::new()),
            }
        }
    }

    #[async_trait]
    impl TicketRepository for MockTicketRepository {
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

        async fn find_by_building(&self, building_id: Uuid) -> Result<Vec<Ticket>, String> {
            Ok(self
                .tickets
                .lock()
                .unwrap()
                .values()
                .filter(|t| t.building_id == building_id)
                .cloned()
                .collect())
        }

        async fn find_by_organization(
            &self,
            organization_id: Uuid,
        ) -> Result<Vec<Ticket>, String> {
            Ok(self
                .tickets
                .lock()
                .unwrap()
                .values()
                .filter(|t| t.organization_id == organization_id)
                .cloned()
                .collect())
        }

        async fn find_by_created_by(&self, created_by: Uuid) -> Result<Vec<Ticket>, String> {
            Ok(self
                .tickets
                .lock()
                .unwrap()
                .values()
                .filter(|t| t.created_by == created_by)
                .cloned()
                .collect())
        }

        async fn find_by_assigned_to(&self, assigned_to: Uuid) -> Result<Vec<Ticket>, String> {
            Ok(self
                .tickets
                .lock()
                .unwrap()
                .values()
                .filter(|t| t.assigned_to == Some(assigned_to))
                .cloned()
                .collect())
        }

        async fn find_by_status(
            &self,
            building_id: Uuid,
            status: TicketStatus,
        ) -> Result<Vec<Ticket>, String> {
            Ok(self
                .tickets
                .lock()
                .unwrap()
                .values()
                .filter(|t| t.building_id == building_id && t.status == status)
                .cloned()
                .collect())
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

        async fn count_by_building(&self, building_id: Uuid) -> Result<i64, String> {
            Ok(self
                .tickets
                .lock()
                .unwrap()
                .values()
                .filter(|t| t.building_id == building_id)
                .count() as i64)
        }

        async fn count_by_status(
            &self,
            building_id: Uuid,
            status: TicketStatus,
        ) -> Result<i64, String> {
            Ok(self
                .tickets
                .lock()
                .unwrap()
                .values()
                .filter(|t| t.building_id == building_id && t.status == status)
                .count() as i64)
        }

        async fn count_by_organization(&self, organization_id: Uuid) -> Result<i64, String> {
            Ok(self
                .tickets
                .lock()
                .unwrap()
                .values()
                .filter(|t| t.organization_id == organization_id)
                .count() as i64)
        }

        async fn count_by_organization_and_status(
            &self,
            organization_id: Uuid,
            status: TicketStatus,
        ) -> Result<i64, String> {
            Ok(self
                .tickets
                .lock()
                .unwrap()
                .values()
                .filter(|t| t.organization_id == organization_id && t.status == status)
                .count() as i64)
        }
    }

    // ── Helpers ──────────────────────────────────────────────────────

    fn make_use_cases(repo: Arc<dyn TicketRepository>) -> TicketUseCases {
        TicketUseCases::new(repo)
    }

    fn make_create_request(building_id: Uuid) -> CreateTicketRequest {
        CreateTicketRequest {
            building_id,
            unit_id: None,
            title: "Fuite d'eau cuisine".to_string(),
            description: "L'eau coule sous l'évier de la cuisine commune".to_string(),
            category: TicketCategory::Plumbing,
            priority: TicketPriority::High,
        }
    }

    /// Helper: create a ticket through the use case and return its id
    async fn create_ticket_helper(
        use_cases: &TicketUseCases,
        org_id: Uuid,
        building_id: Uuid,
    ) -> TicketResponse {
        let user_id = Uuid::new_v4();
        let request = make_create_request(building_id);
        use_cases
            .create_ticket(org_id, user_id, request)
            .await
            .expect("create_ticket should succeed")
    }

    // ── Tests ────────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_create_ticket_success() {
        let repo = Arc::new(MockTicketRepository::new());
        let use_cases = make_use_cases(repo);

        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let request = make_create_request(building_id);

        let result = use_cases.create_ticket(org_id, user_id, request).await;

        assert!(result.is_ok());
        let ticket = result.unwrap();
        assert_eq!(ticket.status, TicketStatus::Open);
        assert_eq!(ticket.building_id, building_id);
        assert_eq!(ticket.organization_id, org_id);
        assert_eq!(ticket.created_by, user_id);
        assert_eq!(ticket.title, "Fuite d'eau cuisine");
        assert!(ticket.assigned_to.is_none());
        assert!(ticket.resolution_notes.is_none());
    }

    #[tokio::test]
    async fn test_create_ticket_empty_title_fails() {
        let repo = Arc::new(MockTicketRepository::new());
        let use_cases = make_use_cases(repo);

        let request = CreateTicketRequest {
            building_id: Uuid::new_v4(),
            unit_id: None,
            title: "   ".to_string(),
            description: "Some description".to_string(),
            category: TicketCategory::Electrical,
            priority: TicketPriority::Low,
        };

        let result = use_cases
            .create_ticket(Uuid::new_v4(), Uuid::new_v4(), request)
            .await;

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Title cannot be empty"));
    }

    #[tokio::test]
    async fn test_assign_contractor_transitions_to_in_progress() {
        let repo = Arc::new(MockTicketRepository::new());
        let use_cases = make_use_cases(repo);

        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let ticket = create_ticket_helper(&use_cases, org_id, building_id).await;
        assert_eq!(ticket.status, TicketStatus::Open);

        let contractor_id = Uuid::new_v4();
        let assign_request = AssignTicketRequest {
            assigned_to: contractor_id,
        };

        let result = use_cases.assign_ticket(ticket.id, assign_request).await;

        assert!(result.is_ok());
        let updated = result.unwrap();
        assert_eq!(updated.status, TicketStatus::InProgress);
        assert_eq!(updated.assigned_to, Some(contractor_id));
    }

    #[tokio::test]
    async fn test_start_work_open_to_in_progress() {
        let repo = Arc::new(MockTicketRepository::new());
        let use_cases = make_use_cases(repo);

        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let ticket = create_ticket_helper(&use_cases, org_id, building_id).await;

        let result = use_cases.start_work(ticket.id).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap().status, TicketStatus::InProgress);
    }

    #[tokio::test]
    async fn test_resolve_ticket_with_notes() {
        let repo = Arc::new(MockTicketRepository::new());
        let use_cases = make_use_cases(repo);

        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let ticket = create_ticket_helper(&use_cases, org_id, building_id).await;

        // Transition to InProgress first
        use_cases.start_work(ticket.id).await.unwrap();

        let resolve_request = ResolveTicketRequest {
            resolution_notes: "Fuite réparée, joint remplacé".to_string(),
        };
        let result = use_cases.resolve_ticket(ticket.id, resolve_request).await;

        assert!(result.is_ok());
        let resolved = result.unwrap();
        assert_eq!(resolved.status, TicketStatus::Resolved);
        assert_eq!(
            resolved.resolution_notes.as_deref(),
            Some("Fuite réparée, joint remplacé")
        );
        assert!(resolved.resolved_at.is_some());
    }

    #[tokio::test]
    async fn test_close_ticket_after_resolution() {
        let repo = Arc::new(MockTicketRepository::new());
        let use_cases = make_use_cases(repo);

        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let ticket = create_ticket_helper(&use_cases, org_id, building_id).await;

        // Full lifecycle: Open -> InProgress -> Resolved -> Closed
        use_cases.start_work(ticket.id).await.unwrap();
        use_cases
            .resolve_ticket(
                ticket.id,
                ResolveTicketRequest {
                    resolution_notes: "Done".to_string(),
                },
            )
            .await
            .unwrap();

        let result = use_cases.close_ticket(ticket.id).await;

        assert!(result.is_ok());
        let closed = result.unwrap();
        assert_eq!(closed.status, TicketStatus::Closed);
        assert!(closed.closed_at.is_some());
    }

    #[tokio::test]
    async fn test_close_open_ticket_fails() {
        let repo = Arc::new(MockTicketRepository::new());
        let use_cases = make_use_cases(repo);

        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let ticket = create_ticket_helper(&use_cases, org_id, building_id).await;

        // Attempt to close directly from Open (skipping Resolved)
        let result = use_cases.close_ticket(ticket.id).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Must be Resolved first"));
    }

    #[tokio::test]
    async fn test_cancel_ticket_with_reason() {
        let repo = Arc::new(MockTicketRepository::new());
        let use_cases = make_use_cases(repo);

        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let ticket = create_ticket_helper(&use_cases, org_id, building_id).await;

        let cancel_request = CancelTicketRequest {
            reason: "Erreur de déclaration, problème déjà résolu".to_string(),
        };
        let result = use_cases.cancel_ticket(ticket.id, cancel_request).await;

        assert!(result.is_ok());
        let cancelled = result.unwrap();
        assert_eq!(cancelled.status, TicketStatus::Cancelled);
        assert!(cancelled
            .resolution_notes
            .as_deref()
            .unwrap()
            .contains("CANCELLED"));
    }

    #[tokio::test]
    async fn test_reopen_resolved_ticket() {
        let repo = Arc::new(MockTicketRepository::new());
        let use_cases = make_use_cases(repo);

        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let ticket = create_ticket_helper(&use_cases, org_id, building_id).await;

        // Open -> InProgress -> Resolved
        use_cases.start_work(ticket.id).await.unwrap();
        use_cases
            .resolve_ticket(
                ticket.id,
                ResolveTicketRequest {
                    resolution_notes: "Fixed".to_string(),
                },
            )
            .await
            .unwrap();

        // Reopen
        let reopen_request = ReopenTicketRequest {
            reason: "Problème persiste après intervention".to_string(),
        };
        let result = use_cases.reopen_ticket(ticket.id, reopen_request).await;

        assert!(result.is_ok());
        let reopened = result.unwrap();
        assert_eq!(reopened.status, TicketStatus::InProgress);
        assert!(reopened.resolved_at.is_none());
        assert!(reopened.closed_at.is_none());
        assert!(reopened
            .resolution_notes
            .as_deref()
            .unwrap()
            .contains("REOPENED"));
    }

    #[tokio::test]
    async fn test_reopen_open_ticket_fails() {
        let repo = Arc::new(MockTicketRepository::new());
        let use_cases = make_use_cases(repo);

        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let ticket = create_ticket_helper(&use_cases, org_id, building_id).await;

        // Ticket is Open, cannot reopen
        let reopen_request = ReopenTicketRequest {
            reason: "Some reason".to_string(),
        };
        let result = use_cases.reopen_ticket(ticket.id, reopen_request).await;

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("Can only reopen resolved or closed tickets"));
    }

    #[tokio::test]
    async fn test_send_work_order_in_progress_ticket() {
        let repo = Arc::new(MockTicketRepository::new());
        let use_cases = make_use_cases(repo);

        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let ticket = create_ticket_helper(&use_cases, org_id, building_id).await;

        // Assign contractor (auto-transitions to InProgress)
        let contractor_id = Uuid::new_v4();
        use_cases
            .assign_ticket(
                ticket.id,
                AssignTicketRequest {
                    assigned_to: contractor_id,
                },
            )
            .await
            .unwrap();

        // Send work order
        let result = use_cases.send_work_order(ticket.id).await;

        assert!(result.is_ok());
        let updated = result.unwrap();
        assert!(updated.work_order_sent_at.is_some());
        assert_eq!(updated.status, TicketStatus::InProgress);
    }

    #[tokio::test]
    async fn test_send_work_order_open_ticket_fails() {
        let repo = Arc::new(MockTicketRepository::new());
        let use_cases = make_use_cases(repo);

        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let ticket = create_ticket_helper(&use_cases, org_id, building_id).await;

        // Ticket is Open (not InProgress), should fail
        let result = use_cases.send_work_order(ticket.id).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("InProgress"));
    }

    #[tokio::test]
    async fn test_get_overdue_tickets() {
        let repo = Arc::new(MockTicketRepository::new());
        let use_cases = make_use_cases(repo.clone());

        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        // Create a ticket and manually age it
        let request = make_create_request(building_id);
        let created = use_cases
            .create_ticket(org_id, user_id, request)
            .await
            .unwrap();

        // Manually set created_at to 10 days ago in the mock store
        {
            let mut store = repo.tickets.lock().unwrap();
            if let Some(ticket) = store.get_mut(&created.id) {
                ticket.created_at = chrono::Utc::now() - chrono::Duration::days(10);
            }
        }

        let overdue = use_cases
            .get_overdue_tickets(building_id, 5)
            .await
            .unwrap();
        assert_eq!(overdue.len(), 1);
        assert_eq!(overdue[0].id, created.id);

        // With a longer threshold, ticket should not be overdue
        let not_overdue = use_cases
            .get_overdue_tickets(building_id, 15)
            .await
            .unwrap();
        assert_eq!(not_overdue.len(), 0);
    }

    #[tokio::test]
    async fn test_get_ticket_statistics() {
        let repo = Arc::new(MockTicketRepository::new());
        let use_cases = make_use_cases(repo);

        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();

        // Create 3 tickets, transition them to different states
        let t1 = create_ticket_helper(&use_cases, org_id, building_id).await; // stays Open
        let t2 = create_ticket_helper(&use_cases, org_id, building_id).await;
        let t3 = create_ticket_helper(&use_cases, org_id, building_id).await;

        // t2 -> InProgress -> Resolved
        use_cases.start_work(t2.id).await.unwrap();
        use_cases
            .resolve_ticket(
                t2.id,
                ResolveTicketRequest {
                    resolution_notes: "Done".to_string(),
                },
            )
            .await
            .unwrap();

        // t3 -> Cancelled
        use_cases
            .cancel_ticket(
                t3.id,
                CancelTicketRequest {
                    reason: "Duplicate".to_string(),
                },
            )
            .await
            .unwrap();

        let stats = use_cases
            .get_ticket_statistics(building_id)
            .await
            .unwrap();

        assert_eq!(stats.total, 3);
        assert_eq!(stats.open, 1); // t1
        assert_eq!(stats.in_progress, 0);
        assert_eq!(stats.resolved, 1); // t2
        assert_eq!(stats.closed, 0);
        assert_eq!(stats.cancelled, 1); // t3
    }

    #[tokio::test]
    async fn test_delete_open_ticket_succeeds() {
        let repo = Arc::new(MockTicketRepository::new());
        let use_cases = make_use_cases(repo);

        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let ticket = create_ticket_helper(&use_cases, org_id, building_id).await;

        let result = use_cases.delete_ticket(ticket.id).await;

        assert!(result.is_ok());
        assert!(result.unwrap());

        // Verify ticket no longer exists
        let fetched = use_cases.get_ticket(ticket.id).await.unwrap();
        assert!(fetched.is_none());
    }

    #[tokio::test]
    async fn test_delete_in_progress_ticket_fails() {
        let repo = Arc::new(MockTicketRepository::new());
        let use_cases = make_use_cases(repo);

        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let ticket = create_ticket_helper(&use_cases, org_id, building_id).await;

        // Transition to InProgress
        use_cases.start_work(ticket.id).await.unwrap();

        let result = use_cases.delete_ticket(ticket.id).await;

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("Can only delete tickets with Open status"));
    }
}
