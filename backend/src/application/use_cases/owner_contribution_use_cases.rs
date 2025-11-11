use crate::application::ports::OwnerContributionRepository;
use crate::domain::entities::{ContributionType, OwnerContribution, PaymentMethod};
use chrono::{DateTime, Utc};
use std::sync::Arc;
use uuid::Uuid;

pub struct OwnerContributionUseCases {
    repository: Arc<dyn OwnerContributionRepository>,
}

impl OwnerContributionUseCases {
    pub fn new(repository: Arc<dyn OwnerContributionRepository>) -> Self {
        Self { repository }
    }

    /// Create a new owner contribution (appel de fonds)
    #[allow(clippy::too_many_arguments)]
    pub async fn create_contribution(
        &self,
        organization_id: Uuid,
        owner_id: Uuid,
        unit_id: Option<Uuid>,
        description: String,
        amount: f64,
        contribution_type: ContributionType,
        contribution_date: DateTime<Utc>,
        account_code: Option<String>,
    ) -> Result<OwnerContribution, String> {
        // Create domain entity (validates business rules)
        let contribution = OwnerContribution::new(
            organization_id,
            owner_id,
            unit_id,
            description,
            amount,
            contribution_type,
            contribution_date,
            account_code,
        )?;

        // Persist
        self.repository.create(&contribution).await
    }

    /// Record payment for a contribution
    pub async fn record_payment(
        &self,
        contribution_id: Uuid,
        payment_date: DateTime<Utc>,
        payment_method: PaymentMethod,
        payment_reference: Option<String>,
    ) -> Result<OwnerContribution, String> {
        // Find contribution
        let mut contribution = self
            .repository
            .find_by_id(contribution_id)
            .await?
            .ok_or_else(|| format!("Contribution not found: {}", contribution_id))?;

        // Mark as paid (domain logic)
        contribution.mark_as_paid(payment_date, payment_method, payment_reference);

        // Update
        self.repository.update(&contribution).await
    }

    /// Get contribution by ID
    pub async fn get_contribution(
        &self,
        contribution_id: Uuid,
    ) -> Result<Option<OwnerContribution>, String> {
        self.repository.find_by_id(contribution_id).await
    }

    /// Get all contributions for an organization
    pub async fn get_contributions_by_organization(
        &self,
        organization_id: Uuid,
    ) -> Result<Vec<OwnerContribution>, String> {
        self.repository.find_by_organization(organization_id).await
    }

    /// Get all contributions for an owner
    pub async fn get_contributions_by_owner(
        &self,
        owner_id: Uuid,
    ) -> Result<Vec<OwnerContribution>, String> {
        self.repository.find_by_owner(owner_id).await
    }

    /// Get outstanding (unpaid) contributions for an owner
    pub async fn get_outstanding_contributions(
        &self,
        owner_id: Uuid,
    ) -> Result<Vec<OwnerContribution>, String> {
        let contributions = self.repository.find_by_owner(owner_id).await?;

        // Filter unpaid
        Ok(contributions.into_iter().filter(|c| !c.is_paid()).collect())
    }

    /// Get overdue contributions for an owner
    pub async fn get_overdue_contributions(
        &self,
        owner_id: Uuid,
    ) -> Result<Vec<OwnerContribution>, String> {
        let contributions = self.repository.find_by_owner(owner_id).await?;

        // Filter overdue
        Ok(contributions
            .into_iter()
            .filter(|c| c.is_overdue())
            .collect())
    }

    /// Get total outstanding amount for an owner
    pub async fn get_outstanding_amount(&self, owner_id: Uuid) -> Result<f64, String> {
        let outstanding = self.get_outstanding_contributions(owner_id).await?;
        Ok(outstanding.iter().map(|c| c.amount).sum())
    }
}
