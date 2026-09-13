use crate::application::dto::{
    BudgetResponse, CreateBudgetRequest, PageRequest, UpdateBudgetRequest,
};
use crate::application::error::AppError;
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
    ) -> Result<BudgetResponse, AppError> {
        // L'immeuble donne l'ACP : c'est elle qui vote et supporte le budget
        // (Art. 3.89 § 5, 16°), le syndic ne fait que le préparer. Cf. ADR-0045.
        let building = self
            .building_repository
            .find_by_id(request.building_id)
            .await
            .map_err(AppError::from)?
            .ok_or_else(|| AppError::NotFound("Building not found".to_string()))?;

        // Check if budget already exists for this building/fiscal_year
        if let Some(_existing) = self
            .repository
            .find_by_building_and_fiscal_year(request.building_id, request.fiscal_year)
            .await?
        {
            return Err(AppError::Conflict(format!(
                "Budget already exists for building {} and fiscal year {}",
                request.building_id, request.fiscal_year
            )));
        }

        // Create budget
        let mut budget = Budget::new(
            building.acp_id,
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
    pub async fn get_budget(&self, id: Uuid) -> Result<Option<BudgetResponse>, AppError> {
        let budget = self.repository.find_by_id(id).await?;
        Ok(budget.map(BudgetResponse::from))
    }

    /// Get budget for a building and fiscal year
    pub async fn get_by_building_and_fiscal_year(
        &self,
        building_id: Uuid,
        fiscal_year: i32,
    ) -> Result<Option<BudgetResponse>, AppError> {
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
    ) -> Result<Option<BudgetResponse>, AppError> {
        let budget = self.repository.find_active_by_building(building_id).await?;
        Ok(budget.map(BudgetResponse::from))
    }

    /// List budgets for a building
    pub async fn list_by_building(
        &self,
        building_id: Uuid,
    ) -> Result<Vec<BudgetResponse>, AppError> {
        let budgets = self.repository.find_by_building(building_id).await?;
        Ok(budgets.into_iter().map(BudgetResponse::from).collect())
    }

    /// List budgets by fiscal year
    pub async fn list_by_fiscal_year(
        &self,
        organization_id: Uuid,
        fiscal_year: i32,
    ) -> Result<Vec<BudgetResponse>, AppError> {
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
    ) -> Result<Vec<BudgetResponse>, AppError> {
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
    ) -> Result<(Vec<BudgetResponse>, i64), AppError> {
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
    ) -> Result<BudgetResponse, AppError> {
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
    pub async fn submit_for_approval(&self, id: Uuid) -> Result<BudgetResponse, AppError> {
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
    ) -> Result<BudgetResponse, AppError> {
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
    ) -> Result<BudgetResponse, AppError> {
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
    pub async fn archive_budget(&self, id: Uuid) -> Result<BudgetResponse, AppError> {
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
    pub async fn delete_budget(&self, id: Uuid) -> Result<bool, AppError> {
        self.repository.delete(id).await
    }

    /// Get budget statistics
    pub async fn get_stats(&self, organization_id: Uuid) -> Result<BudgetStatsResponse, AppError> {
        self.repository.get_stats(organization_id).await
    }

    /// Get budget variance analysis (budget vs actual expenses)
    pub async fn get_variance(
        &self,
        budget_id: Uuid,
    ) -> Result<Option<BudgetVarianceResponse>, AppError> {
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
    use rust_decimal_macros::dec;

    // Mock BudgetRepository
    mock! {
        pub BudgetRepo {}

        #[async_trait::async_trait]
        impl BudgetRepository for BudgetRepo {
            async fn create(&self, budget: &Budget) -> Result<Budget, AppError>;
            async fn find_by_id(&self, id: Uuid) -> Result<Option<Budget>, AppError>;
            async fn find_by_building_and_fiscal_year(
                &self,
                building_id: Uuid,
                fiscal_year: i32,
            ) -> Result<Option<Budget>, AppError>;
            async fn find_by_building(&self, building_id: Uuid) -> Result<Vec<Budget>, AppError>;
            async fn find_active_by_building(&self, building_id: Uuid) -> Result<Option<Budget>, AppError>;
            async fn find_by_fiscal_year(
                &self,
                organization_id: Uuid,
                fiscal_year: i32,
            ) -> Result<Vec<Budget>, AppError>;
            async fn find_by_status(
                &self,
                organization_id: Uuid,
                status: BudgetStatus,
            ) -> Result<Vec<Budget>, AppError>;
            async fn find_all_paginated(
                &self,
                page_request: &PageRequest,
                organization_id: Option<Uuid>,
                building_id: Option<Uuid>,
                status: Option<BudgetStatus>,
            ) -> Result<(Vec<Budget>, i64), AppError>;
            async fn update(&self, budget: &Budget) -> Result<Budget, AppError>;
            async fn delete(&self, id: Uuid) -> Result<bool, AppError>;
            async fn get_stats(&self, organization_id: Uuid) -> Result<BudgetStatsResponse, AppError>;
            async fn get_variance(&self, budget_id: Uuid) -> Result<Option<BudgetVarianceResponse>, AppError>;
        }
    }

    // Mock BuildingRepository (port still uses String; do not migrate here)
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
            async fn find_by_id_with_metrics(
                &self,
                id: Uuid,
            ) -> Result<Option<(Building, crate::domain::entities::BuildingMetrics)>, String>;
        }
    }

    // Mock ExpenseRepository (port still uses String; do not migrate here)
    mock! {
        pub ExpenseRepo {}

        #[async_trait::async_trait]
        impl ExpenseRepository for ExpenseRepo {
        async fn enregistrer_lignes_de_facture(
            &self,
            _expense_id: Uuid,
            _lignes: &[crate::application::ports::expense_repository::LigneDeFacture],
        ) -> Result<(), String> {
            // Mock : rien à enregistrer. Le port n'offre pas d'implémentation
            // par défaut, précisément pour que ce choix soit écrit ici plutôt
            // que subi partout.
            Ok(())
        }

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
    /// Un immeuble rattaché à une ACP.
    ///
    /// Le premier argument de `Building::new` est l'**ACP**, pas
    /// l'organisation : un immeuble appartient à une copropriété, pas à un
    /// cabinet de syndic (Art. 3.85 § 1er, ADR-0010).
    fn make_building(acp_id: Uuid) -> Building {
        Building::new(
            acp_id,
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
    ///
    /// L'ACP vient en premier : c'est elle qui vote le budget et le supporte
    /// (Art. 3.89 § 5, 16°), le syndic ne fait que le préparer.
    fn make_draft_budget(acp_id: Uuid, org_id: Uuid, building_id: Uuid) -> Budget {
        Budget::new(acp_id, org_id, building_id, 2025, dec!(60000), dec!(15000)).unwrap()
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

    /// Art. 3.89 § 5, 16° et ADR-0045 : le budget appartient à l'ACP.
    ///
    /// L'ACP se **déduit de l'immeuble**, elle n'est jamais lue dans la
    /// requête. C'est ce qui empêche un cabinet d'écrire dans le dossier d'un
    /// autre en forgeant un identifiant : le lien immeuble → ACP est fixé par
    /// l'acte de base, la requête ne peut pas le contredire.
    #[tokio::test]
    async fn test_le_budget_appartient_a_lacp_de_limmeuble_pas_au_syndic() {
        let acp_de_limmeuble = Uuid::new_v4();
        let cabinet_qui_encode = Uuid::new_v4();
        let building_id = Uuid::new_v4();

        let mut budget_repo = MockBudgetRepo::new();
        let mut building_repo = MockBuildingRepo::new();
        let expense_repo = MockExpenseRepo::new();

        let building = make_building(acp_de_limmeuble);
        building_repo
            .expect_find_by_id()
            .with(eq(building_id))
            .times(1)
            .returning(move |_| Ok(Some(building.clone())));
        budget_repo
            .expect_find_by_building_and_fiscal_year()
            .times(1)
            .returning(|_, _| Ok(None));
        budget_repo
            .expect_create()
            .times(1)
            .returning(|b| Ok(b.clone()));

        let uc = make_use_cases(budget_repo, building_repo, expense_repo);

        let budget = uc
            .create_budget(CreateBudgetRequest {
                organization_id: cabinet_qui_encode,
                building_id,
                fiscal_year: 2026,
                ordinary_budget: dec!(48000),
                extraordinary_budget: dec!(12000),
                notes: None,
            })
            .await
            .expect("création valide");

        assert_eq!(
            budget.acp_id, acp_de_limmeuble,
            "le budget doit être rattaché à l'ACP de l'immeuble"
        );
        assert_eq!(
            budget.organization_id, cabinet_qui_encode,
            "le syndic reste tracé comme auteur, sans que ça lui donne un droit"
        );
        assert_ne!(
            budget.acp_id, budget.organization_id,
            "l'ACP et le syndic sont deux entités distinctes, pas deux noms de la même"
        );
    }

    // ---------------------------------------------------------------
    // 1. Create budget (happy path)
    // ---------------------------------------------------------------
    #[tokio::test]
    async fn test_create_budget_success() {
        let acp_id = Uuid::new_v4();
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();

        let mut budget_repo = MockBudgetRepo::new();
        let mut building_repo = MockBuildingRepo::new();
        let expense_repo = MockExpenseRepo::new();

        // Building exists
        let building = make_building(acp_id);
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
            ordinary_budget: dec!(60000),
            extraordinary_budget: dec!(15000),
            notes: Some("Budget prévisionnel toiture".to_string()),
        };

        let result = uc.create_budget(request).await;
        assert!(result.is_ok());
        let resp = result.unwrap();
        assert_eq!(resp.fiscal_year, 2025);
        assert_eq!(resp.ordinary_budget, dec!(60000));
        assert_eq!(resp.extraordinary_budget, dec!(15000));
        assert_eq!(resp.total_budget, dec!(75000));
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
        let acp_id = Uuid::new_v4();
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();

        let mut budget_repo = MockBudgetRepo::new();
        let mut building_repo = MockBuildingRepo::new();
        let expense_repo = MockExpenseRepo::new();

        let building = make_building(acp_id);
        building_repo
            .expect_find_by_id()
            .with(eq(building_id))
            .times(1)
            .returning(move |_| Ok(Some(building.clone())));

        // Duplicate exists
        let existing = make_draft_budget(acp_id, org_id, building_id);
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
            ordinary_budget: dec!(60000),
            extraordinary_budget: dec!(15000),
            notes: None,
        };

        let result = uc.create_budget(request).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Budget already exists"));
    }

    // ---------------------------------------------------------------
    // 3. Submit for approval (Draft -> Submitted)
    // ---------------------------------------------------------------
    #[tokio::test]
    async fn test_submit_for_approval_success() {
        let acp_id = Uuid::new_v4();
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let budget_id = Uuid::new_v4();

        let mut budget_repo = MockBudgetRepo::new();
        let building_repo = MockBuildingRepo::new();
        let expense_repo = MockExpenseRepo::new();

        let mut draft = make_draft_budget(acp_id, org_id, building_id);
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
        let acp_id = Uuid::new_v4();
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let budget_id = Uuid::new_v4();
        let meeting_id = Uuid::new_v4();

        let mut budget_repo = MockBudgetRepo::new();
        let building_repo = MockBuildingRepo::new();
        let expense_repo = MockExpenseRepo::new();

        // Budget must be in Submitted state
        let mut submitted = make_draft_budget(acp_id, org_id, building_id);
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
        let acp_id = Uuid::new_v4();
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let budget_id = Uuid::new_v4();

        let mut budget_repo = MockBudgetRepo::new();
        let building_repo = MockBuildingRepo::new();
        let expense_repo = MockExpenseRepo::new();

        let mut submitted = make_draft_budget(acp_id, org_id, building_id);
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
        let acp_id = Uuid::new_v4();
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let budget_id = Uuid::new_v4();
        let meeting_id = Uuid::new_v4();

        let mut budget_repo = MockBudgetRepo::new();
        let building_repo = MockBuildingRepo::new();
        let expense_repo = MockExpenseRepo::new();

        // Budget must be Approved to archive
        let mut approved = make_draft_budget(acp_id, org_id, building_id);
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
            budgeted_ordinary: dec!(60000),
            budgeted_extraordinary: dec!(15000),
            budgeted_total: dec!(75000),
            actual_ordinary: dec!(45000),
            actual_extraordinary: dec!(20000),
            actual_total: dec!(65000),
            variance_ordinary: dec!(15000),
            variance_extraordinary: dec!(-5000),
            variance_total: dec!(10000),
            variance_ordinary_pct: dec!(25),
            variance_extraordinary_pct: dec!(-33.33),
            variance_total_pct: dec!(13.33),
            has_overruns: true,
            overrun_categories: vec!["Extraordinary".to_string()],
            months_elapsed: 8,
            projected_year_end_total: dec!(97500),
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
        assert_eq!(v.budgeted_total, dec!(75000));
        assert_eq!(v.actual_total, dec!(65000));
        assert_eq!(v.variance_total, dec!(10000));
        assert!(v.has_overruns);
        assert_eq!(v.overrun_categories, vec!["Extraordinary".to_string()]);
    }

    // ---------------------------------------------------------------
    // 8. Get active budget for a building
    // ---------------------------------------------------------------
    #[tokio::test]
    async fn test_get_active_budget_for_building() {
        let acp_id = Uuid::new_v4();
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let meeting_id = Uuid::new_v4();

        let mut budget_repo = MockBudgetRepo::new();
        let building_repo = MockBuildingRepo::new();
        let expense_repo = MockExpenseRepo::new();

        // Active budget = Approved
        let mut approved = make_draft_budget(acp_id, org_id, building_id);
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
