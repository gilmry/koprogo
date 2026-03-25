use crate::application::ports::OwnerContributionRepository;
use crate::domain::entities::{ContributionPaymentMethod, ContributionType, OwnerContribution};
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
        payment_method: ContributionPaymentMethod,
        payment_reference: Option<String>,
    ) -> Result<OwnerContribution, String> {
        // Find contribution
        let mut contribution = self
            .repository
            .find_by_id(contribution_id)
            .await?
            .ok_or_else(|| format!("Contribution not found: {}", contribution_id))?;

        // Prevent double payment
        if contribution.is_paid() {
            return Err("Contribution is already paid".to_string());
        }

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

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use std::collections::HashMap;
    use std::sync::Mutex;

    struct MockOwnerContributionRepository {
        items: Mutex<HashMap<Uuid, OwnerContribution>>,
    }

    impl MockOwnerContributionRepository {
        fn new() -> Self {
            Self {
                items: Mutex::new(HashMap::new()),
            }
        }
    }

    #[async_trait]
    impl OwnerContributionRepository for MockOwnerContributionRepository {
        async fn create(
            &self,
            contribution: &OwnerContribution,
        ) -> Result<OwnerContribution, String> {
            let mut items = self.items.lock().unwrap();
            items.insert(contribution.id, contribution.clone());
            Ok(contribution.clone())
        }

        async fn find_by_id(&self, id: Uuid) -> Result<Option<OwnerContribution>, String> {
            let items = self.items.lock().unwrap();
            Ok(items.get(&id).cloned())
        }

        async fn find_by_organization(
            &self,
            organization_id: Uuid,
        ) -> Result<Vec<OwnerContribution>, String> {
            let items = self.items.lock().unwrap();
            Ok(items
                .values()
                .filter(|c| c.organization_id == organization_id)
                .cloned()
                .collect())
        }

        async fn find_by_owner(
            &self,
            owner_id: Uuid,
        ) -> Result<Vec<OwnerContribution>, String> {
            let items = self.items.lock().unwrap();
            Ok(items
                .values()
                .filter(|c| c.owner_id == owner_id)
                .cloned()
                .collect())
        }

        async fn update(
            &self,
            contribution: &OwnerContribution,
        ) -> Result<OwnerContribution, String> {
            let mut items = self.items.lock().unwrap();
            items.insert(contribution.id, contribution.clone());
            Ok(contribution.clone())
        }
    }

    fn make_use_cases(repo: MockOwnerContributionRepository) -> OwnerContributionUseCases {
        OwnerContributionUseCases::new(Arc::new(repo))
    }

    #[tokio::test]
    async fn test_create_contribution_success() {
        let repo = MockOwnerContributionRepository::new();
        let use_cases = make_use_cases(repo);
        let org_id = Uuid::new_v4();
        let owner_id = Uuid::new_v4();
        let unit_id = Uuid::new_v4();

        let result = use_cases
            .create_contribution(
                org_id,
                owner_id,
                Some(unit_id),
                "Appel de fonds Q1 2026".to_string(),
                750.0,
                ContributionType::Regular,
                Utc::now(),
                Some("7000".to_string()),
            )
            .await;

        assert!(result.is_ok());
        let contrib = result.unwrap();
        assert_eq!(contrib.organization_id, org_id);
        assert_eq!(contrib.owner_id, owner_id);
        assert_eq!(contrib.unit_id, Some(unit_id));
        assert_eq!(contrib.amount, 750.0);
        assert_eq!(contrib.contribution_type, ContributionType::Regular);
        assert!(!contrib.is_paid());
    }

    #[tokio::test]
    async fn test_record_payment_success() {
        let repo = MockOwnerContributionRepository::new();
        let org_id = Uuid::new_v4();
        let owner_id = Uuid::new_v4();

        // Pre-populate with a pending contribution
        let contrib = OwnerContribution::new(
            org_id,
            owner_id,
            None,
            "Charges Q2".to_string(),
            500.0,
            ContributionType::Regular,
            Utc::now(),
            None,
        )
        .unwrap();
        let contrib_id = contrib.id;
        repo.items.lock().unwrap().insert(contrib.id, contrib);

        let use_cases = make_use_cases(repo);
        let result = use_cases
            .record_payment(
                contrib_id,
                Utc::now(),
                ContributionPaymentMethod::BankTransfer,
                Some("VIR-2026-001".to_string()),
            )
            .await;

        assert!(result.is_ok());
        let paid = result.unwrap();
        assert!(paid.is_paid());
        assert!(paid.payment_date.is_some());
        assert_eq!(
            paid.payment_method,
            Some(ContributionPaymentMethod::BankTransfer)
        );
        assert_eq!(
            paid.payment_reference,
            Some("VIR-2026-001".to_string())
        );
    }

    #[tokio::test]
    async fn test_record_payment_double_payment_rejected() {
        let repo = MockOwnerContributionRepository::new();
        let org_id = Uuid::new_v4();
        let owner_id = Uuid::new_v4();

        // Pre-populate with an already-paid contribution
        let mut contrib = OwnerContribution::new(
            org_id,
            owner_id,
            None,
            "Charges Q3".to_string(),
            300.0,
            ContributionType::Regular,
            Utc::now(),
            None,
        )
        .unwrap();
        contrib.mark_as_paid(
            Utc::now(),
            ContributionPaymentMethod::Cash,
            None,
        );
        let contrib_id = contrib.id;
        repo.items.lock().unwrap().insert(contrib.id, contrib);

        let use_cases = make_use_cases(repo);
        let result = use_cases
            .record_payment(
                contrib_id,
                Utc::now(),
                ContributionPaymentMethod::BankTransfer,
                None,
            )
            .await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Contribution is already paid");
    }

    #[tokio::test]
    async fn test_get_outstanding_contributions() {
        let repo = MockOwnerContributionRepository::new();
        let org_id = Uuid::new_v4();
        let owner_id = Uuid::new_v4();

        // Create one paid and two unpaid contributions
        let mut paid_contrib = OwnerContribution::new(
            org_id,
            owner_id,
            None,
            "Charges Q1 - paid".to_string(),
            200.0,
            ContributionType::Regular,
            Utc::now(),
            None,
        )
        .unwrap();
        paid_contrib.mark_as_paid(
            Utc::now(),
            ContributionPaymentMethod::Domiciliation,
            None,
        );

        let unpaid1 = OwnerContribution::new(
            org_id,
            owner_id,
            None,
            "Charges Q2 - unpaid".to_string(),
            300.0,
            ContributionType::Regular,
            Utc::now(),
            None,
        )
        .unwrap();

        let unpaid2 = OwnerContribution::new(
            org_id,
            owner_id,
            None,
            "Travaux extraordinaires".to_string(),
            1500.0,
            ContributionType::Extraordinary,
            Utc::now(),
            None,
        )
        .unwrap();

        {
            let mut items = repo.items.lock().unwrap();
            items.insert(paid_contrib.id, paid_contrib);
            items.insert(unpaid1.id, unpaid1);
            items.insert(unpaid2.id, unpaid2);
        }

        let use_cases = make_use_cases(repo);
        let result = use_cases.get_outstanding_contributions(owner_id).await;

        assert!(result.is_ok());
        let outstanding = result.unwrap();
        assert_eq!(outstanding.len(), 2);
        assert!(outstanding.iter().all(|c| !c.is_paid()));
    }

    #[tokio::test]
    async fn test_get_outstanding_amount() {
        let repo = MockOwnerContributionRepository::new();
        let org_id = Uuid::new_v4();
        let owner_id = Uuid::new_v4();

        // Create one paid (should not count) and two unpaid
        let mut paid = OwnerContribution::new(
            org_id,
            owner_id,
            None,
            "Paid contribution".to_string(),
            100.0,
            ContributionType::Regular,
            Utc::now(),
            None,
        )
        .unwrap();
        paid.mark_as_paid(Utc::now(), ContributionPaymentMethod::Check, None);

        let unpaid1 = OwnerContribution::new(
            org_id,
            owner_id,
            None,
            "Unpaid 1".to_string(),
            250.0,
            ContributionType::Regular,
            Utc::now(),
            None,
        )
        .unwrap();

        let unpaid2 = OwnerContribution::new(
            org_id,
            owner_id,
            None,
            "Unpaid 2".to_string(),
            400.0,
            ContributionType::Extraordinary,
            Utc::now(),
            None,
        )
        .unwrap();

        {
            let mut items = repo.items.lock().unwrap();
            items.insert(paid.id, paid);
            items.insert(unpaid1.id, unpaid1);
            items.insert(unpaid2.id, unpaid2);
        }

        let use_cases = make_use_cases(repo);
        let result = use_cases.get_outstanding_amount(owner_id).await;

        assert!(result.is_ok());
        let amount = result.unwrap();
        assert!((amount - 650.0).abs() < f64::EPSILON);
    }
}
