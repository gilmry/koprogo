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
    pub async fn list_by_building(
        &self,
        building_id: Uuid,
    ) -> Result<Vec<BudgetResponse>, String> {
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
        let budgets = self.repository.find_by_status(organization_id, status).await?;
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

        // Apply updates
        if let Some(ordinary) = request.ordinary_budget {
            if let Some(extraordinary) = request.extraordinary_budget {
                budget.update_amounts(ordinary, extraordinary)?;
            }
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
