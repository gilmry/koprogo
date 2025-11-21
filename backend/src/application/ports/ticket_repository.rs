use crate::domain::entities::{Ticket, TicketStatus};
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait TicketRepository: Send + Sync {
    async fn create(&self, ticket: &Ticket) -> Result<Ticket, String>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Ticket>, String>;
    async fn find_by_building(&self, building_id: Uuid) -> Result<Vec<Ticket>, String>;
    async fn find_by_organization(&self, organization_id: Uuid) -> Result<Vec<Ticket>, String>;
    async fn find_by_created_by(&self, created_by: Uuid) -> Result<Vec<Ticket>, String>;
    async fn find_by_assigned_to(&self, assigned_to: Uuid) -> Result<Vec<Ticket>, String>;
    async fn find_by_status(
        &self,
        building_id: Uuid,
        status: TicketStatus,
    ) -> Result<Vec<Ticket>, String>;
    async fn update(&self, ticket: &Ticket) -> Result<Ticket, String>;
    async fn delete(&self, id: Uuid) -> Result<bool, String>;
    async fn count_by_building(&self, building_id: Uuid) -> Result<i64, String>;
    async fn count_by_status(&self, building_id: Uuid, status: TicketStatus)
        -> Result<i64, String>;
}
