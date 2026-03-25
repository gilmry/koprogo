use crate::application::dto::{
    BudgetResponse, CreateBudgetRequest, PageRequest, UpdateBudgetRequest,
};
use crate::application::ports::{
    BudgetRepository, BudgetStatsResponse, BudgetVarianceResponse, BuildingRepository,
    ExpenseRepository,
};
use crate::domain::entities::{Budget, BudgetStatus};
use std::sync::Arc;
use uuid::Uuid;

pub struct BudgetUseCases {
    repository: Arc<dyn BudgetRepository>,
    building_repository: Arc<dyn BuildingRepository>,
    #[allow(dead_code)]
    expense_repository: Arc<dyn ExpenseRepository>,
}

impl BudgetUseCases {
    pub fn new(
        repository: Arc<dyn BudgetRepository>,
        building_repository: Arc<dyn BuildingRepository>,
        expense_repository: Arc<dyn ExpenseRepository>,
    ) -> Self {
        Self {
            repository,
            building_repository,
            expense_repository,
        }
    }

    /// Create a new budget
    pub async fn create_budget(
        &self,
        request: CreateBudgetRequest,
    ) -> Result<BudgetResponse, String> {
        // Verify building exists
        let _building = self
            .building_repository
            .find_by_id(request.building_id)
            .await?
            .ok_or_else(|| "Building not found".to_string())?;

        // Check if budget already exists for this building/fiscal_year
        if let Some(_existing) = self
            .repository
            .find_by_building_and_fiscal_year(request.building_id, request.fiscal_year)
            .await?
        {
            return Err(format!(
                "Budget already exists for building {} and fiscal year {}",
                request.building_id, request.fiscal_year
            ));
        }

        // Create budget
        let mut budget = Budget::new(
            request.organization_id,
            request.building_id,
            request.fiscal_year,
            request.ordinary_budget,
            request.extraordinary_budget,
        )?;

        // Set notes if provided
        if let Some(notes) = request.notes {
            budget.update_notes(notes);
        }

        let created = self.repository.create(&budget).await?;
        Ok(BudgetResponse::from(created))
    }

    /// Get budget by ID
    pub async fn get_budget(&self, id: Uuid) -> Result<Option<BudgetResponse>, String> {
        let budget = self.repository.find_by_id(id).await?;
        Ok(budget.map(BudgetResponse::from))
    }

    /// Get budget for a building and fiscal year
    pub async fn get_by_building_and_fiscal_year(
        &self,
        building_id: Uuid,
        fiscal_year: i32,
    ) -> Result<Option<BudgetResponse>, String> {
        let budget = self
            .repository
            .find_by_building_and_fiscal_year(building_id, fiscal_year)
            .await?;
        Ok(budget.map(BudgetResponse::from))
    }

    /// Get active budget for a building
    pub async fn get_active_budget(
        &self,
        building_id: Uuid,
    ) -> Result<Option<BudgetResponse>, String> {
        let budget = self.repository.find_active_by_building(building_id).await?;
        Ok(budget.map(BudgetResponse::from))
    }

    /// List budgets for a building
    pub async fn list_by_building(&self, building_id: Uuid) -> Result<Vec<BudgetResponse>, String> {
        let budgets = self.repository.find_by_building(building_id).await?;
        Ok(budgets.into_iter().map(BudgetResponse::from).collect())
    }

    /// List budgets by fiscal year
    pub async fn list_by_fiscal_year(
        &self,
        organization_id: Uuid,
        fiscal_year: i32,
    ) -> Result<Vec<BudgetResponse>, String> {
        let budgets = self
            .repository
            .find_by_fiscal_year(organization_id, fiscal_year)
            .await?;
        Ok(budgets.into_iter().map(BudgetResponse::from).collect())
    }

    /// List budgets by status
    pub async fn list_by_status(
        &self,
        organization_id: Uuid,
        status: BudgetStatus,
    ) -> Result<Vec<BudgetResponse>, String> {
        let budgets = self
            .repository
            .find_by_status(organization_id, status)
            .await?;
        Ok(budgets.into_iter().map(BudgetResponse::from).collect())
    }

    /// List budgets paginated
    pub async fn list_paginated(
        &self,
        page_request: &PageRequest,
        organization_id: Option<Uuid>,
        building_id: Option<Uuid>,
        status: Option<BudgetStatus>,
    ) -> Result<(Vec<BudgetResponse>, i64), String> {
        let (budgets, total) = self
            .repository
            .find_all_paginated(page_request, organization_id, building_id, status)
            .await?;

        let dtos = budgets.into_iter().map(BudgetResponse::from).collect();
        Ok((dtos, total))
    }

    /// Update budget amounts (Draft only)
    pub async fn update_budget(
        &self,
        id: Uuid,
        request: UpdateBudgetRequest,
    ) -> Result<BudgetResponse, String> {
        let mut budget = self
            .repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| "Budget not found".to_string())?;

        // Apply updates (use existing values as defaults for partial updates)
        if request.ordinary_budget.is_some() || request.extraordinary_budget.is_some() {
            let ordinary = request.ordinary_budget.unwrap_or(budget.ordinary_budget);
            let extraordinary = request
                .extraordinary_budget
                .unwrap_or(budget.extraordinary_budget);
            budget.update_amounts(ordinary, extraordinary)?;
        }

        if let Some(notes) = request.notes {
            budget.update_notes(notes);
        }

        let updated = self.repository.update(&budget).await?;
        Ok(BudgetResponse::from(updated))
    }

    /// Submit budget for approval
    pub async fn submit_for_approval(&self, id: Uuid) -> Result<BudgetResponse, String> {
        let mut budget = self
            .repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| "Budget not found".to_string())?;

        budget.submit_for_approval()?;

        let updated = self.repository.update(&budget).await?;
        Ok(BudgetResponse::from(updated))
    }

    /// Approve budget (requires meeting_id for legal traceability)
    pub async fn approve_budget(
        &self,
        id: Uuid,
        meeting_id: Uuid,
    ) -> Result<BudgetResponse, String> {
        let mut budget = self
            .repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| "Budget not found".to_string())?;

        budget.approve(meeting_id)?;

        let updated = self.repository.update(&budget).await?;
        Ok(BudgetResponse::from(updated))
    }

    /// Reject budget (with optional reason)
    pub async fn reject_budget(
        &self,
        id: Uuid,
        reason: Option<String>,
    ) -> Result<BudgetResponse, String> {
        let mut budget = self
            .repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| "Budget not found".to_string())?;

        // Add rejection reason to notes
        if let Some(reason) = reason {
            let current_notes = budget.notes.clone().unwrap_or_default();
            let new_notes = if current_notes.is_empty() {
                format!("REJECTED: {}", reason)
            } else {
                format!("{}\n\nREJECTED: {}", current_notes, reason)
            };
            budget.update_notes(new_notes);
        }

        budget.reject()?;

        let updated = self.repository.update(&budget).await?;
        Ok(BudgetResponse::from(updated))
    }

    /// Archive budget
    pub async fn archive_budget(&self, id: Uuid) -> Result<BudgetResponse, String> {
        let mut budget = self
            .repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| "Budget not found".to_string())?;

        budget.archive()?;

        let updated = self.repository.update(&budget).await?;
        Ok(BudgetResponse::from(updated))
    }

    /// Delete budget
    pub async fn delete_budget(&self, id: Uuid) -> Result<bool, String> {
        self.repository.delete(id).await
    }

    /// Get budget statistics
    pub async fn get_stats(&self, organization_id: Uuid) -> Result<BudgetStatsResponse, String> {
        self.repository.get_stats(organization_id).await
    }

    /// Get budget variance analysis (budget vs actual expenses)
    pub async fn get_variance(
        &self,
        budget_id: Uuid,
    ) -> Result<Option<BudgetVarianceResponse>, String> {
        self.repository.get_variance(budget_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::dto::PageRequest;
    use crate::application::ports::{
        BudgetRepository, BudgetStatsResponse, BudgetVarianceResponse, BuildingRepository,
        ExpenseRepository,
    };
    use crate::domain::entities::{Building, Expense};
    use mockall::mock;
    use mockall::predicate::*;

    // Mock BudgetRepository
    mock! {
        pub BudgetRepo {}

        #[async_trait::async_trait]
        impl BudgetRepository for BudgetRepo {
            async fn create(&self, budget: &Budget) -> Result<Budget, String>;
            async fn find_by_id(&self, id: Uuid) -> Result<Option<Budget>, String>;
            async fn find_by_building_and_fiscal_year(
                &self,
                building_id: Uuid,
                fiscal_year: i32,
            ) -> Result<Option<Budget>, String>;
            async fn find_by_building(&self, building_id: Uuid) -> Result<Vec<Budget>, String>;
            async fn find_active_by_building(&self, building_id: Uuid) -> Result<Option<Budget>, String>;
            async fn find_by_fiscal_year(
                &self,
                organization_id: Uuid,
                fiscal_year: i32,
            ) -> Result<Vec<Budget>, String>;
            async fn find_by_status(
                &self,
                organization_id: Uuid,
                status: BudgetStatus,
            ) -> Result<Vec<Budget>, String>;
            async fn find_all_paginated(
                &self,
                page_request: &PageRequest,
                organization_id: Option<Uuid>,
                building_id: Option<Uuid>,
                status: Option<BudgetStatus>,
            ) -> Result<(Vec<Budget>, i64), String>;
            async fn update(&self, budget: &Budget) -> Result<Budget, String>;
            async fn delete(&self, id: Uuid) -> Result<bool, String>;
            async fn get_stats(&self, organization_id: Uuid) -> Result<BudgetStatsResponse, String>;
            async fn get_variance(&self, budget_id: Uuid) -> Result<Option<BudgetVarianceResponse>, String>;
        }
    }

    // Mock BuildingRepository
    mock! {
        pub BuildingRepo {}

        #[async_trait::async_trait]
        impl BuildingRepository for BuildingRepo {
            async fn create(&self, building: &Building) -> Result<Building, String>;
            async fn find_by_id(&self, id: Uuid) -> Result<Option<Building>, String>;
            async fn find_by_slug(&self, slug: &str) -> Result<Option<Building>, String>;
            async fn find_all(&self) -> Result<Vec<Building>, String>;
            async fn find_all_paginated(
                &self,
                page_request: &crate::application::dto::PageRequest,
                filters: &crate::application::dto::BuildingFilters,
            ) -> Result<(Vec<Building>, i64), String>;
            async fn update(&self, building: &Building) -> Result<Building, String>;
            async fn delete(&self, id: Uuid) -> Result<bool, String>;
        }
    }

    // Mock ExpenseRepository
    mock! {
        pub ExpenseRepo {}

        #[async_trait::async_trait]
        impl ExpenseRepository for ExpenseRepo {
            async fn create(&self, expense: &Expense) -> Result<Expense, String>;
            async fn find_by_id(&self, id: Uuid) -> Result<Option<Expense>, String>;
            async fn find_by_building(&self, building_id: Uuid) -> Result<Vec<Expense>, String>;
            async fn find_all_paginated(
                &self,
                page_request: &crate::application::dto::PageRequest,
                filters: &crate::application::dto::ExpenseFilters,
            ) -> Result<(Vec<Expense>, i64), String>;
            async fn update(&self, expense: &Expense) -> Result<Expense, String>;
            async fn delete(&self, id: Uuid) -> Result<bool, String>;
        }
    }

    /// Helper: create a valid Building for mock returns
    fn make_building(org_id: Uuid) -> Building {
        Building::new(
            org_id,
            "Résidence du Parc".to_string(),
            "12 Rue de la Loi".to_string(),
            "Brussels".to_string(),
            "1000".to_string(),
            "Belgium".to_string(),
            20,
            1000,
            Some(2015),
        )
        .unwrap()
    }

    /// Helper: create a Draft budget ready for use in tests
    fn make_draft_budget(org_id: Uuid, building_id: Uuid) -> Budget {
        Budget::new(org_id, building_id, 2025, 60000.0, 15000.0).unwrap()
    }

    /// Helper: build the BudgetUseCases from three mock repos
    fn make_use_cases(
        budget_repo: MockBudgetRepo,
        building_repo: MockBuildingRepo,
        expense_repo: MockExpenseRepo,
    ) -> BudgetUseCases {
        BudgetUseCases::new(
            Arc::new(budget_repo),
            Arc::new(building_repo),
            Arc::new(expense_repo),
        )
    }

    // ---------------------------------------------------------------
    // 1. Create budget (happy path)
    // ---------------------------------------------------------------
    #[tokio::test]
    async fn test_create_budget_success() {
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();

        let mut budget_repo = MockBudgetRepo::new();
        let mut building_repo = MockBuildingRepo::new();
        let expense_repo = MockExpenseRepo::new();

        // Building exists
        let building = make_building(org_id);
        building_repo
            .expect_find_by_id()
            .with(eq(building_id))
            .times(1)
            .returning(move |_| Ok(Some(building.clone())));

        // No duplicate for this building+fiscal_year
        budget_repo
            .expect_find_by_building_and_fiscal_year()
            .with(eq(building_id), eq(2025))
            .times(1)
            .returning(|_, _| Ok(None));

        // Repo creates successfully
        budget_repo
            .expect_create()
            .times(1)
            .returning(|b| Ok(b.clone()));

        let uc = make_use_cases(budget_repo, building_repo, expense_repo);

        let request = CreateBudgetRequest {
            organization_id: org_id,
            building_id,
            fiscal_year: 2025,
            ordinary_budget: 60000.0,
            extraordinary_budget: 15000.0,
            notes: Some("Budget prévisionnel toiture".to_string()),
        };

        let result = uc.create_budget(request).await;
        assert!(result.is_ok());
        let resp = result.unwrap();
        assert_eq!(resp.fiscal_year, 2025);
        assert_eq!(resp.ordinary_budget, 60000.0);
        assert_eq!(resp.extraordinary_budget, 15000.0);
        assert_eq!(resp.total_budget, 75000.0);
        assert_eq!(resp.status, BudgetStatus::Draft);
        assert!(resp.is_editable);
        assert!(!resp.is_active);
        assert_eq!(resp.notes, Some("Budget prévisionnel toiture".to_string()));
    }

    // ---------------------------------------------------------------
    // 2. Create budget fails when building+fiscal_year already exists
    // ---------------------------------------------------------------
    #[tokio::test]
    async fn test_create_budget_duplicate_fiscal_year() {
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();

        let mut budget_repo = MockBudgetRepo::new();
        let mut building_repo = MockBuildingRepo::new();
        let expense_repo = MockExpenseRepo::new();

        let building = make_building(org_id);
        building_repo
            .expect_find_by_id()
            .with(eq(building_id))
            .times(1)
            .returning(move |_| Ok(Some(building.clone())));

        // Duplicate exists
        let existing = make_draft_budget(org_id, building_id);
        budget_repo
            .expect_find_by_building_and_fiscal_year()
            .with(eq(building_id), eq(2025))
            .times(1)
            .returning(move |_, _| Ok(Some(existing.clone())));

        let uc = make_use_cases(budget_repo, building_repo, expense_repo);

        let request = CreateBudgetRequest {
            organization_id: org_id,
            building_id,
            fiscal_year: 2025,
            ordinary_budget: 60000.0,
            extraordinary_budget: 15000.0,
            notes: None,
        };

        let result = uc.create_budget(request).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Budget already exists"));
    }

    // ---------------------------------------------------------------
    // 3. Submit for approval (Draft -> Submitted)
    // ---------------------------------------------------------------
    #[tokio::test]
    async fn test_submit_for_approval_success() {
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let budget_id = Uuid::new_v4();

        let mut budget_repo = MockBudgetRepo::new();
        let building_repo = MockBuildingRepo::new();
        let expense_repo = MockExpenseRepo::new();

        let mut draft = make_draft_budget(org_id, building_id);
        draft.id = budget_id;

        budget_repo
            .expect_find_by_id()
            .with(eq(budget_id))
            .times(1)
            .returning(move |_| Ok(Some(draft.clone())));

        budget_repo
            .expect_update()
            .times(1)
            .returning(|b| Ok(b.clone()));

        let uc = make_use_cases(budget_repo, building_repo, expense_repo);

        let result = uc.submit_for_approval(budget_id).await;
        assert!(result.is_ok());
        let resp = result.unwrap();
        assert_eq!(resp.status, BudgetStatus::Submitted);
        assert!(resp.submitted_date.is_some());
    }

    // ---------------------------------------------------------------
    // 4. Approve budget (Submitted -> Approved, requires meeting_id)
    // ---------------------------------------------------------------
    #[tokio::test]
    async fn test_approve_budget_success() {
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let budget_id = Uuid::new_v4();
        let meeting_id = Uuid::new_v4();

        let mut budget_repo = MockBudgetRepo::new();
        let building_repo = MockBuildingRepo::new();
        let expense_repo = MockExpenseRepo::new();

        // Budget must be in Submitted state
        let mut submitted = make_draft_budget(org_id, building_id);
        submitted.id = budget_id;
        submitted.submit_for_approval().unwrap();

        budget_repo
            .expect_find_by_id()
            .with(eq(budget_id))
            .times(1)
            .returning(move |_| Ok(Some(submitted.clone())));

        budget_repo
            .expect_update()
            .times(1)
            .returning(|b| Ok(b.clone()));

        let uc = make_use_cases(budget_repo, building_repo, expense_repo);

        let result = uc.approve_budget(budget_id, meeting_id).await;
        assert!(result.is_ok());
        let resp = result.unwrap();
        assert_eq!(resp.status, BudgetStatus::Approved);
        assert!(resp.approved_date.is_some());
        assert_eq!(resp.approved_by_meeting_id, Some(meeting_id));
        assert!(resp.is_active);
    }

    // ---------------------------------------------------------------
    // 5. Reject budget (Submitted -> Rejected, with reason)
    // ---------------------------------------------------------------
    #[tokio::test]
    async fn test_reject_budget_with_reason() {
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let budget_id = Uuid::new_v4();

        let mut budget_repo = MockBudgetRepo::new();
        let building_repo = MockBuildingRepo::new();
        let expense_repo = MockExpenseRepo::new();

        let mut submitted = make_draft_budget(org_id, building_id);
        submitted.id = budget_id;
        submitted.submit_for_approval().unwrap();

        budget_repo
            .expect_find_by_id()
            .with(eq(budget_id))
            .times(1)
            .returning(move |_| Ok(Some(submitted.clone())));

        budget_repo
            .expect_update()
            .times(1)
            .returning(|b| Ok(b.clone()));

        let uc = make_use_cases(budget_repo, building_repo, expense_repo);

        let reason = Some("Montant extraordinaire trop élevé".to_string());
        let result = uc.reject_budget(budget_id, reason).await;
        assert!(result.is_ok());
        let resp = result.unwrap();
        assert_eq!(resp.status, BudgetStatus::Rejected);
        assert!(resp.notes.is_some());
        assert!(resp
            .notes
            .unwrap()
            .contains("REJECTED: Montant extraordinaire trop élevé"));
    }

    // ---------------------------------------------------------------
    // 6. Archive budget (Approved -> Archived)
    // ---------------------------------------------------------------
    #[tokio::test]
    async fn test_archive_budget_success() {
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let budget_id = Uuid::new_v4();
        let meeting_id = Uuid::new_v4();

        let mut budget_repo = MockBudgetRepo::new();
        let building_repo = MockBuildingRepo::new();
        let expense_repo = MockExpenseRepo::new();

        // Budget must be Approved to archive
        let mut approved = make_draft_budget(org_id, building_id);
        approved.id = budget_id;
        approved.submit_for_approval().unwrap();
        approved.approve(meeting_id).unwrap();

        budget_repo
            .expect_find_by_id()
            .with(eq(budget_id))
            .times(1)
            .returning(move |_| Ok(Some(approved.clone())));

        budget_repo
            .expect_update()
            .times(1)
            .returning(|b| Ok(b.clone()));

        let uc = make_use_cases(budget_repo, building_repo, expense_repo);

        let result = uc.archive_budget(budget_id).await;
        assert!(result.is_ok());
        let resp = result.unwrap();
        assert_eq!(resp.status, BudgetStatus::Archived);
        assert!(!resp.is_active);
        assert!(!resp.is_editable);
    }

    // ---------------------------------------------------------------
    // 7. Get variance analysis
    // ---------------------------------------------------------------
    #[tokio::test]
    async fn test_get_variance_returns_analysis() {
        let budget_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();

        let mut budget_repo = MockBudgetRepo::new();
        let building_repo = MockBuildingRepo::new();
        let expense_repo = MockExpenseRepo::new();

        let variance = BudgetVarianceResponse {
            budget_id,
            fiscal_year: 2025,
            building_id,
            budgeted_ordinary: 60000.0,
            budgeted_extraordinary: 15000.0,
            budgeted_total: 75000.0,
            actual_ordinary: 45000.0,
            actual_extraordinary: 20000.0,
            actual_total: 65000.0,
            variance_ordinary: 15000.0,
            variance_extraordinary: -5000.0,
            variance_total: 10000.0,
            variance_ordinary_pct: 25.0,
            variance_extraordinary_pct: -33.33,
            variance_total_pct: 13.33,
            has_overruns: true,
            overrun_categories: vec!["Extraordinary".to_string()],
            months_elapsed: 8,
            projected_year_end_total: 97500.0,
        };

        let expected_variance = variance.clone();

        budget_repo
            .expect_get_variance()
            .with(eq(budget_id))
            .times(1)
            .returning(move |_| Ok(Some(variance.clone())));

        let uc = make_use_cases(budget_repo, building_repo, expense_repo);

        let result = uc.get_variance(budget_id).await;
        assert!(result.is_ok());
        let opt = result.unwrap();
        assert!(opt.is_some());
        let v = opt.unwrap();
        assert_eq!(v.budget_id, expected_variance.budget_id);
        assert_eq!(v.budgeted_total, 75000.0);
        assert_eq!(v.actual_total, 65000.0);
        assert_eq!(v.variance_total, 10000.0);
        assert!(v.has_overruns);
        assert_eq!(v.overrun_categories, vec!["Extraordinary".to_string()]);
    }

    // ---------------------------------------------------------------
    // 8. Get active budget for a building
    // ---------------------------------------------------------------
    #[tokio::test]
    async fn test_get_active_budget_for_building() {
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let meeting_id = Uuid::new_v4();

        let mut budget_repo = MockBudgetRepo::new();
        let building_repo = MockBuildingRepo::new();
        let expense_repo = MockExpenseRepo::new();

        // Active budget = Approved
        let mut approved = make_draft_budget(org_id, building_id);
        approved.submit_for_approval().unwrap();
        approved.approve(meeting_id).unwrap();

        let expected_id = approved.id;

        budget_repo
            .expect_find_active_by_building()
            .with(eq(building_id))
            .times(1)
            .returning(move |_| Ok(Some(approved.clone())));

        let uc = make_use_cases(budget_repo, building_repo, expense_repo);

        let result = uc.get_active_budget(building_id).await;
        assert!(result.is_ok());
        let opt = result.unwrap();
        assert!(opt.is_some());
        let resp = opt.unwrap();
        assert_eq!(resp.id, expected_id);
        assert_eq!(resp.status, BudgetStatus::Approved);
        assert!(resp.is_active);
        assert_eq!(resp.building_id, building_id);
    }
}
