use crate::application::ports::{
    CallForFundsRepository, OwnerContributionRepository, UnitOwnerRepository,
};
use crate::domain::entities::{CallForFunds, ContributionType, OwnerContribution};
use chrono::{DateTime, Utc};
use std::sync::Arc;
use uuid::Uuid;

pub struct CallForFundsUseCases {
    call_for_funds_repository: Arc<dyn CallForFundsRepository>,
    owner_contribution_repository: Arc<dyn OwnerContributionRepository>,
    unit_owner_repository: Arc<dyn UnitOwnerRepository>,
}

impl CallForFundsUseCases {
    pub fn new(
        call_for_funds_repository: Arc<dyn CallForFundsRepository>,
        owner_contribution_repository: Arc<dyn OwnerContributionRepository>,
        unit_owner_repository: Arc<dyn UnitOwnerRepository>,
    ) -> Self {
        Self {
            call_for_funds_repository,
            owner_contribution_repository,
            unit_owner_repository,
        }
    }

    /// Create a new call for funds
    #[allow(clippy::too_many_arguments)]
    pub async fn create_call_for_funds(
        &self,
        organization_id: Uuid,
        building_id: Uuid,
        title: String,
        description: String,
        total_amount: f64,
        contribution_type: ContributionType,
        call_date: DateTime<Utc>,
        due_date: DateTime<Utc>,
        account_code: Option<String>,
        created_by: Option<Uuid>,
    ) -> Result<CallForFunds, String> {
        // Create the call for funds entity
        let mut call_for_funds = CallForFunds::new(
            organization_id,
            building_id,
            title,
            description,
            total_amount,
            contribution_type.clone(),
            call_date,
            due_date,
            account_code,
        )?;

        call_for_funds.created_by = created_by;

        // Save to database
        self.call_for_funds_repository.create(&call_for_funds).await
    }

    /// Get a call for funds by ID
    pub async fn get_call_for_funds(&self, id: Uuid) -> Result<Option<CallForFunds>, String> {
        self.call_for_funds_repository.find_by_id(id).await
    }

    /// List all calls for funds for a building
    pub async fn list_by_building(&self, building_id: Uuid) -> Result<Vec<CallForFunds>, String> {
        self.call_for_funds_repository
            .find_by_building(building_id)
            .await
    }

    /// List all calls for funds for an organization
    pub async fn list_by_organization(
        &self,
        organization_id: Uuid,
    ) -> Result<Vec<CallForFunds>, String> {
        self.call_for_funds_repository
            .find_by_organization(organization_id)
            .await
    }

    /// Mark call for funds as sent and generate individual owner contributions
    /// This is the key operation that automatically creates contributions for all owners
    pub async fn send_call_for_funds(&self, id: Uuid) -> Result<CallForFunds, String> {
        // Get the call for funds
        let mut call_for_funds = self
            .call_for_funds_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| "Call for funds not found".to_string())?;

        // Mark as sent
        call_for_funds.mark_as_sent();

        // Update in database
        let updated_call = self
            .call_for_funds_repository
            .update(&call_for_funds)
            .await?;

        // Generate individual contributions for all owners in the building
        self.generate_owner_contributions(&updated_call).await?;

        Ok(updated_call)
    }

    /// Generate individual owner contributions based on ownership percentages
    async fn generate_owner_contributions(
        &self,
        call_for_funds: &CallForFunds,
    ) -> Result<Vec<OwnerContribution>, String> {
        // Get all active unit owners for the building
        // Returns (unit_id, owner_id, percentage)
        let unit_owners = self
            .unit_owner_repository
            .find_active_by_building(call_for_funds.building_id)
            .await?;

        if unit_owners.is_empty() {
            return Err("No active owners found for this building".to_string());
        }

        let mut contributions = Vec::new();

        for (unit_id, owner_id, percentage) in unit_owners {
            // Calculate individual amount based on ownership percentage
            let individual_amount = call_for_funds.total_amount * percentage;

            // Create contribution description
            let description = format!(
                "{} - Quote-part: {:.2}%",
                call_for_funds.title,
                percentage * 100.0
            );

            // Create owner contribution
            let mut contribution = OwnerContribution::new(
                call_for_funds.organization_id,
                owner_id,
                Some(unit_id),
                description,
                individual_amount,
                call_for_funds.contribution_type.clone(),
                call_for_funds.call_date,
                call_for_funds.account_code.clone(),
            )?;

            // Link to the call for funds
            contribution.call_for_funds_id = Some(call_for_funds.id);

            // Save contribution
            let saved = self
                .owner_contribution_repository
                .create(&contribution)
                .await?;

            contributions.push(saved);
        }

        Ok(contributions)
    }

    /// Cancel a call for funds
    pub async fn cancel_call_for_funds(&self, id: Uuid) -> Result<CallForFunds, String> {
        let mut call_for_funds = self
            .call_for_funds_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| "Call for funds not found".to_string())?;

        call_for_funds.cancel();

        self.call_for_funds_repository.update(&call_for_funds).await
    }

    /// Get all overdue calls for funds
    pub async fn get_overdue_calls(&self) -> Result<Vec<CallForFunds>, String> {
        self.call_for_funds_repository.find_overdue().await
    }

    /// Delete a call for funds (only if not sent)
    pub async fn delete_call_for_funds(&self, id: Uuid) -> Result<bool, String> {
        let call_for_funds = self
            .call_for_funds_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| "Call for funds not found".to_string())?;

        // Don't allow deletion if already sent
        if call_for_funds.status != crate::domain::entities::CallForFundsStatus::Draft {
            return Err("Cannot delete a call for funds that has been sent".to_string());
        }

        self.call_for_funds_repository.delete(id).await
    }
}
