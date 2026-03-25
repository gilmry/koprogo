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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::ports::{
        CallForFundsRepository, OwnerContributionRepository, UnitOwnerRepository,
    };
    use crate::domain::entities::{
        CallForFunds, CallForFundsStatus, ContributionType, OwnerContribution, UnitOwner,
    };
    use async_trait::async_trait;
    use chrono::{Duration, Utc};
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};
    use uuid::Uuid;

    // ── Mock: CallForFundsRepository ──────────────────────────────────

    struct MockCallForFundsRepo {
        store: Mutex<HashMap<Uuid, CallForFunds>>,
        overdue: Mutex<Vec<CallForFunds>>,
    }

    impl MockCallForFundsRepo {
        fn new() -> Self {
            Self {
                store: Mutex::new(HashMap::new()),
                overdue: Mutex::new(Vec::new()),
            }
        }

        fn with_overdue(overdue: Vec<CallForFunds>) -> Self {
            Self {
                store: Mutex::new(HashMap::new()),
                overdue: Mutex::new(overdue),
            }
        }
    }

    #[async_trait]
    impl CallForFundsRepository for MockCallForFundsRepo {
        async fn create(&self, cff: &CallForFunds) -> Result<CallForFunds, String> {
            let mut store = self.store.lock().unwrap();
            store.insert(cff.id, cff.clone());
            Ok(cff.clone())
        }

        async fn find_by_id(&self, id: Uuid) -> Result<Option<CallForFunds>, String> {
            Ok(self.store.lock().unwrap().get(&id).cloned())
        }

        async fn find_by_building(&self, building_id: Uuid) -> Result<Vec<CallForFunds>, String> {
            Ok(self
                .store
                .lock()
                .unwrap()
                .values()
                .filter(|c| c.building_id == building_id)
                .cloned()
                .collect())
        }

        async fn find_by_organization(
            &self,
            organization_id: Uuid,
        ) -> Result<Vec<CallForFunds>, String> {
            Ok(self
                .store
                .lock()
                .unwrap()
                .values()
                .filter(|c| c.organization_id == organization_id)
                .cloned()
                .collect())
        }

        async fn update(&self, cff: &CallForFunds) -> Result<CallForFunds, String> {
            let mut store = self.store.lock().unwrap();
            store.insert(cff.id, cff.clone());
            Ok(cff.clone())
        }

        async fn delete(&self, id: Uuid) -> Result<bool, String> {
            Ok(self.store.lock().unwrap().remove(&id).is_some())
        }

        async fn find_overdue(&self) -> Result<Vec<CallForFunds>, String> {
            Ok(self.overdue.lock().unwrap().clone())
        }
    }

    // ── Mock: OwnerContributionRepository ─────────────────────────────

    struct MockOwnerContributionRepo {
        store: Mutex<Vec<OwnerContribution>>,
    }

    impl MockOwnerContributionRepo {
        fn new() -> Self {
            Self {
                store: Mutex::new(Vec::new()),
            }
        }
    }

    #[async_trait]
    impl OwnerContributionRepository for MockOwnerContributionRepo {
        async fn create(
            &self,
            contribution: &OwnerContribution,
        ) -> Result<OwnerContribution, String> {
            self.store.lock().unwrap().push(contribution.clone());
            Ok(contribution.clone())
        }

        async fn find_by_id(&self, id: Uuid) -> Result<Option<OwnerContribution>, String> {
            Ok(self
                .store
                .lock()
                .unwrap()
                .iter()
                .find(|c| c.id == id)
                .cloned())
        }

        async fn find_by_organization(
            &self,
            organization_id: Uuid,
        ) -> Result<Vec<OwnerContribution>, String> {
            Ok(self
                .store
                .lock()
                .unwrap()
                .iter()
                .filter(|c| c.organization_id == organization_id)
                .cloned()
                .collect())
        }

        async fn find_by_owner(&self, owner_id: Uuid) -> Result<Vec<OwnerContribution>, String> {
            Ok(self
                .store
                .lock()
                .unwrap()
                .iter()
                .filter(|c| c.owner_id == owner_id)
                .cloned()
                .collect())
        }

        async fn update(
            &self,
            contribution: &OwnerContribution,
        ) -> Result<OwnerContribution, String> {
            Ok(contribution.clone())
        }
    }

    // ── Mock: UnitOwnerRepository ─────────────────────────────────────

    struct MockUnitOwnerRepo {
        active_by_building: Mutex<Vec<(Uuid, Uuid, f64)>>,
    }

    impl MockUnitOwnerRepo {
        fn new() -> Self {
            Self {
                active_by_building: Mutex::new(Vec::new()),
            }
        }

        fn with_owners(owners: Vec<(Uuid, Uuid, f64)>) -> Self {
            Self {
                active_by_building: Mutex::new(owners),
            }
        }
    }

    #[async_trait]
    impl UnitOwnerRepository for MockUnitOwnerRepo {
        async fn create(&self, _uo: &UnitOwner) -> Result<UnitOwner, String> {
            unimplemented!()
        }
        async fn find_by_id(&self, _id: Uuid) -> Result<Option<UnitOwner>, String> {
            unimplemented!()
        }
        async fn find_current_owners_by_unit(
            &self,
            _unit_id: Uuid,
        ) -> Result<Vec<UnitOwner>, String> {
            unimplemented!()
        }
        async fn find_current_units_by_owner(
            &self,
            _owner_id: Uuid,
        ) -> Result<Vec<UnitOwner>, String> {
            unimplemented!()
        }
        async fn find_all_owners_by_unit(
            &self,
            _unit_id: Uuid,
        ) -> Result<Vec<UnitOwner>, String> {
            unimplemented!()
        }
        async fn find_all_units_by_owner(
            &self,
            _owner_id: Uuid,
        ) -> Result<Vec<UnitOwner>, String> {
            unimplemented!()
        }
        async fn update(&self, _uo: &UnitOwner) -> Result<UnitOwner, String> {
            unimplemented!()
        }
        async fn delete(&self, _id: Uuid) -> Result<(), String> {
            unimplemented!()
        }
        async fn has_active_owners(&self, _unit_id: Uuid) -> Result<bool, String> {
            unimplemented!()
        }
        async fn get_total_ownership_percentage(&self, _unit_id: Uuid) -> Result<f64, String> {
            unimplemented!()
        }
        async fn find_active_by_unit_and_owner(
            &self,
            _unit_id: Uuid,
            _owner_id: Uuid,
        ) -> Result<Option<UnitOwner>, String> {
            unimplemented!()
        }
        async fn find_active_by_building(
            &self,
            _building_id: Uuid,
        ) -> Result<Vec<(Uuid, Uuid, f64)>, String> {
            Ok(self.active_by_building.lock().unwrap().clone())
        }
    }

    // ── Helpers ───────────────────────────────────────────────────────

    fn make_use_cases(
        cff_repo: Arc<dyn CallForFundsRepository>,
        contrib_repo: Arc<dyn OwnerContributionRepository>,
        uo_repo: Arc<dyn UnitOwnerRepository>,
    ) -> CallForFundsUseCases {
        CallForFundsUseCases::new(cff_repo, contrib_repo, uo_repo)
    }

    fn sample_dates() -> (chrono::DateTime<Utc>, chrono::DateTime<Utc>) {
        let call_date = Utc::now();
        let due_date = call_date + Duration::days(30);
        (call_date, due_date)
    }

    // ── 1. Create ─────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_create_call_for_funds_success() {
        let cff_repo = Arc::new(MockCallForFundsRepo::new());
        let contrib_repo = Arc::new(MockOwnerContributionRepo::new());
        let uo_repo = Arc::new(MockUnitOwnerRepo::new());
        let uc = make_use_cases(cff_repo.clone(), contrib_repo, uo_repo);

        let (call_date, due_date) = sample_dates();
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();

        let result = uc
            .create_call_for_funds(
                org_id,
                building_id,
                "Appel Q1".to_string(),
                "Charges courantes".to_string(),
                10_000.0,
                ContributionType::Regular,
                call_date,
                due_date,
                Some("7000".to_string()),
                Some(Uuid::new_v4()),
            )
            .await;

        assert!(result.is_ok());
        let cff = result.unwrap();
        assert_eq!(cff.total_amount, 10_000.0);
        assert_eq!(cff.status, CallForFundsStatus::Draft);
        assert_eq!(cff.organization_id, org_id);
        assert_eq!(cff.building_id, building_id);
        // Verify it was persisted in the mock store
        assert!(cff_repo.store.lock().unwrap().contains_key(&cff.id));
    }

    // ── 2. Send (generates contributions) ─────────────────────────────

    #[tokio::test]
    async fn test_send_call_for_funds_generates_contributions() {
        let cff_repo = Arc::new(MockCallForFundsRepo::new());
        let contrib_repo = Arc::new(MockOwnerContributionRepo::new());

        let unit1 = Uuid::new_v4();
        let unit2 = Uuid::new_v4();
        let owner1 = Uuid::new_v4();
        let owner2 = Uuid::new_v4();
        let uo_repo = Arc::new(MockUnitOwnerRepo::with_owners(vec![
            (unit1, owner1, 0.60),
            (unit2, owner2, 0.40),
        ]));

        let uc = make_use_cases(cff_repo.clone(), contrib_repo.clone(), uo_repo);

        let (call_date, due_date) = sample_dates();

        let cff = uc
            .create_call_for_funds(
                Uuid::new_v4(),
                Uuid::new_v4(),
                "Appel Q2".to_string(),
                "Charges extraordinaires".to_string(),
                5_000.0,
                ContributionType::Extraordinary,
                call_date,
                due_date,
                None,
                None,
            )
            .await
            .unwrap();

        // Send — should generate individual contributions
        let result = uc.send_call_for_funds(cff.id).await;
        assert!(result.is_ok());

        let sent = result.unwrap();
        assert_eq!(sent.status, CallForFundsStatus::Sent);
        assert!(sent.sent_date.is_some());

        // Verify two contributions were created with correct amounts
        let contributions = contrib_repo.store.lock().unwrap();
        assert_eq!(contributions.len(), 2);

        let mut amounts: Vec<f64> = contributions.iter().map(|c| c.amount).collect();
        amounts.sort_by(|a, b| a.partial_cmp(b).unwrap());
        // 40% of 5000 = 2000, 60% of 5000 = 3000
        assert!((amounts[0] - 2_000.0).abs() < 0.01);
        assert!((amounts[1] - 3_000.0).abs() < 0.01);
    }

    // ── 3. Cancel ─────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_cancel_call_for_funds() {
        let cff_repo = Arc::new(MockCallForFundsRepo::new());
        let contrib_repo = Arc::new(MockOwnerContributionRepo::new());
        let uo_repo = Arc::new(MockUnitOwnerRepo::new());
        let uc = make_use_cases(cff_repo.clone(), contrib_repo, uo_repo);

        let (call_date, due_date) = sample_dates();

        let cff = uc
            .create_call_for_funds(
                Uuid::new_v4(),
                Uuid::new_v4(),
                "Appel annulable".to_string(),
                "Description".to_string(),
                1_000.0,
                ContributionType::Regular,
                call_date,
                due_date,
                None,
                None,
            )
            .await
            .unwrap();

        let result = uc.cancel_call_for_funds(cff.id).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().status, CallForFundsStatus::Cancelled);
    }

    // ── 4. Delete (draft only, rejects sent) ──────────────────────────

    #[tokio::test]
    async fn test_delete_call_for_funds_draft_succeeds() {
        let cff_repo = Arc::new(MockCallForFundsRepo::new());
        let contrib_repo = Arc::new(MockOwnerContributionRepo::new());
        let uo_repo = Arc::new(MockUnitOwnerRepo::new());
        let uc = make_use_cases(cff_repo.clone(), contrib_repo, uo_repo);

        let (call_date, due_date) = sample_dates();

        let cff = uc
            .create_call_for_funds(
                Uuid::new_v4(),
                Uuid::new_v4(),
                "Supprimable".to_string(),
                "Description".to_string(),
                500.0,
                ContributionType::Advance,
                call_date,
                due_date,
                None,
                None,
            )
            .await
            .unwrap();

        let result = uc.delete_call_for_funds(cff.id).await;
        assert!(result.is_ok());
        assert!(result.unwrap());
        assert!(!cff_repo.store.lock().unwrap().contains_key(&cff.id));
    }

    #[tokio::test]
    async fn test_delete_call_for_funds_rejects_non_draft() {
        let cff_repo = Arc::new(MockCallForFundsRepo::new());
        let contrib_repo = Arc::new(MockOwnerContributionRepo::new());
        let uo_repo = Arc::new(MockUnitOwnerRepo::with_owners(vec![(
            Uuid::new_v4(),
            Uuid::new_v4(),
            1.0,
        )]));
        let uc = make_use_cases(cff_repo.clone(), contrib_repo, uo_repo);

        let (call_date, due_date) = sample_dates();

        let cff = uc
            .create_call_for_funds(
                Uuid::new_v4(),
                Uuid::new_v4(),
                "Sent call".to_string(),
                "Description".to_string(),
                500.0,
                ContributionType::Regular,
                call_date,
                due_date,
                None,
                None,
            )
            .await
            .unwrap();

        // Send so it is no longer Draft
        uc.send_call_for_funds(cff.id).await.unwrap();

        let result = uc.delete_call_for_funds(cff.id).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("Cannot delete a call for funds that has been sent"));
    }

    // ── 5. Find overdue ───────────────────────────────────────────────

    #[tokio::test]
    async fn test_get_overdue_calls() {
        let call_date = Utc::now() - Duration::days(60);
        let due_date = Utc::now() - Duration::days(30);
        let overdue_cff = CallForFunds::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "Overdue call".to_string(),
            "Past due".to_string(),
            2_000.0,
            ContributionType::Regular,
            call_date,
            due_date,
            None,
        )
        .unwrap();

        let cff_repo = Arc::new(MockCallForFundsRepo::with_overdue(vec![overdue_cff.clone()]));
        let contrib_repo = Arc::new(MockOwnerContributionRepo::new());
        let uo_repo = Arc::new(MockUnitOwnerRepo::new());
        let uc = make_use_cases(cff_repo, contrib_repo, uo_repo);

        let result = uc.get_overdue_calls().await;
        assert!(result.is_ok());
        let overdue = result.unwrap();
        assert_eq!(overdue.len(), 1);
        assert_eq!(overdue[0].title, "Overdue call");
    }

    // ── 6. List by building ───────────────────────────────────────────

    #[tokio::test]
    async fn test_list_by_building() {
        let cff_repo = Arc::new(MockCallForFundsRepo::new());
        let contrib_repo = Arc::new(MockOwnerContributionRepo::new());
        let uo_repo = Arc::new(MockUnitOwnerRepo::new());
        let uc = make_use_cases(cff_repo.clone(), contrib_repo, uo_repo);

        let building_id = Uuid::new_v4();
        let other_building = Uuid::new_v4();
        let org_id = Uuid::new_v4();
        let (call_date, due_date) = sample_dates();

        // Two calls for our building
        uc.create_call_for_funds(
            org_id,
            building_id,
            "Appel 1".to_string(),
            "Desc 1".to_string(),
            1_000.0,
            ContributionType::Regular,
            call_date,
            due_date,
            None,
            None,
        )
        .await
        .unwrap();

        uc.create_call_for_funds(
            org_id,
            building_id,
            "Appel 2".to_string(),
            "Desc 2".to_string(),
            2_000.0,
            ContributionType::Extraordinary,
            call_date,
            due_date,
            None,
            None,
        )
        .await
        .unwrap();

        // One call for another building (noise)
        uc.create_call_for_funds(
            org_id,
            other_building,
            "Autre appel".to_string(),
            "Autre desc".to_string(),
            500.0,
            ContributionType::Regular,
            call_date,
            due_date,
            None,
            None,
        )
        .await
        .unwrap();

        let result = uc.list_by_building(building_id).await;
        assert!(result.is_ok());
        let list = result.unwrap();
        assert_eq!(list.len(), 2);
        assert!(list.iter().all(|c| c.building_id == building_id));
    }
}
