use crate::domain::entities::CallForFunds;
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait CallForFundsRepository: Send + Sync {
    /// Create a new call for funds
    async fn create(&self, call_for_funds: &CallForFunds) -> Result<CallForFunds, String>;

    /// Find a call for funds by ID
    async fn find_by_id(&self, id: Uuid) -> Result<Option<CallForFunds>, String>;

    /// Find all calls for funds for a building
    async fn find_by_building(&self, building_id: Uuid) -> Result<Vec<CallForFunds>, String>;

    /// Find all calls for funds for an organization
    async fn find_by_organization(
        &self,
        organization_id: Uuid,
    ) -> Result<Vec<CallForFunds>, String>;

    /// Update a call for funds
    async fn update(&self, call_for_funds: &CallForFunds) -> Result<CallForFunds, String>;

    /// Delete a call for funds
    async fn delete(&self, id: Uuid) -> Result<bool, String>;

    /// Find overdue calls for funds (past due date, not completed/cancelled),
    /// scoped to one organization.
    ///
    /// `organization_id` est obligatoire depuis #882 : sans lui, cette
    /// méthode rendait les arriérés de TOUTE l'instance à qui la lisait.
    async fn find_overdue(&self, organization_id: Uuid) -> Result<Vec<CallForFunds>, String>;
}
