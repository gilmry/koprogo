use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Ticket Category - Types of maintenance requests
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub enum TicketPriority {
    Low,      // Basse
    Medium,   // Moyenne
    High,     // Haute
    Critical, // Critique/Urgente
}

/// Ticket Status - Workflow states
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
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
            created_at: now,
            updated_at: now,
            resolved_at: None,
            closed_at: None,
        })
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
}
