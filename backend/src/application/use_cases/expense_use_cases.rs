use crate::application::dto::{CreateExpenseDto, ExpenseFilters, ExpenseResponseDto, PageRequest};
use crate::application::ports::ExpenseRepository;
use crate::domain::entities::Expense;
use chrono::DateTime;
use std::sync::Arc;
use uuid::Uuid;

pub struct ExpenseUseCases {
    repository: Arc<dyn ExpenseRepository>,
}

impl ExpenseUseCases {
    pub fn new(repository: Arc<dyn ExpenseRepository>) -> Self {
        Self { repository }
    }

    pub async fn create_expense(
        &self,
        dto: CreateExpenseDto,
    ) -> Result<ExpenseResponseDto, String> {
        let organization_id = Uuid::parse_str(&dto.organization_id)
            .map_err(|_| "Invalid organization_id format".to_string())?;
        let building_id = Uuid::parse_str(&dto.building_id)
            .map_err(|_| "Invalid building ID format".to_string())?;

        let expense_date = DateTime::parse_from_rfc3339(&dto.expense_date)
            .map_err(|_| "Invalid date format".to_string())?
            .with_timezone(&chrono::Utc);

        let expense = Expense::new(
            organization_id,
            building_id,
            dto.category,
            dto.description,
            dto.amount,
            expense_date,
            dto.supplier,
            dto.invoice_number,
            dto.account_code,
        )?;

        let created = self.repository.create(&expense).await?;
        Ok(self.to_response_dto(&created))
    }

    pub async fn get_expense(&self, id: Uuid) -> Result<Option<ExpenseResponseDto>, String> {
        let expense = self.repository.find_by_id(id).await?;
        Ok(expense.map(|e| self.to_response_dto(&e)))
    }

    pub async fn list_expenses_by_building(
        &self,
        building_id: Uuid,
    ) -> Result<Vec<ExpenseResponseDto>, String> {
        let expenses = self.repository.find_by_building(building_id).await?;
        Ok(expenses.iter().map(|e| self.to_response_dto(e)).collect())
    }

    pub async fn list_expenses_paginated(
        &self,
        page_request: &PageRequest,
        organization_id: Option<Uuid>,
    ) -> Result<(Vec<ExpenseResponseDto>, i64), String> {
        let filters = ExpenseFilters {
            organization_id,
            ..Default::default()
        };

        let (expenses, total) = self
            .repository
            .find_all_paginated(page_request, &filters)
            .await?;

        let dtos = expenses.iter().map(|e| self.to_response_dto(e)).collect();
        Ok((dtos, total))
    }

    pub async fn mark_as_paid(&self, id: Uuid) -> Result<ExpenseResponseDto, String> {
        let mut expense = self
            .repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| "Expense not found".to_string())?;

        expense.mark_as_paid()?;

        let updated = self.repository.update(&expense).await?;
        Ok(self.to_response_dto(&updated))
    }

    pub async fn mark_as_overdue(&self, id: Uuid) -> Result<ExpenseResponseDto, String> {
        let mut expense = self
            .repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| "Expense not found".to_string())?;

        expense.mark_as_overdue()?;

        let updated = self.repository.update(&expense).await?;
        Ok(self.to_response_dto(&updated))
    }

    pub async fn cancel_expense(&self, id: Uuid) -> Result<ExpenseResponseDto, String> {
        let mut expense = self
            .repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| "Expense not found".to_string())?;

        expense.cancel()?;

        let updated = self.repository.update(&expense).await?;
        Ok(self.to_response_dto(&updated))
    }

    pub async fn reactivate_expense(&self, id: Uuid) -> Result<ExpenseResponseDto, String> {
        let mut expense = self
            .repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| "Expense not found".to_string())?;

        expense.reactivate()?;

        let updated = self.repository.update(&expense).await?;
        Ok(self.to_response_dto(&updated))
    }

    pub async fn unpay_expense(&self, id: Uuid) -> Result<ExpenseResponseDto, String> {
        let mut expense = self
            .repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| "Expense not found".to_string())?;

        expense.unpay()?;

        let updated = self.repository.update(&expense).await?;
        Ok(self.to_response_dto(&updated))
    }

    fn to_response_dto(&self, expense: &Expense) -> ExpenseResponseDto {
        ExpenseResponseDto {
            id: expense.id.to_string(),
            building_id: expense.building_id.to_string(),
            category: expense.category.clone(),
            description: expense.description.clone(),
            amount: expense.amount,
            expense_date: expense.expense_date.to_rfc3339(),
            payment_status: expense.payment_status.clone(),
            supplier: expense.supplier.clone(),
            invoice_number: expense.invoice_number.clone(),
            account_code: expense.account_code.clone(),
        }
    }
}
