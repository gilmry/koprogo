use crate::application::dto::{ExpenseFilters, PageRequest};
use crate::domain::entities::Expense;
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait ExpenseRepository: Send + Sync {
    async fn create(&self, expense: &Expense) -> Result<Expense, String>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Expense>, String>;
    async fn find_by_building(&self, building_id: Uuid) -> Result<Vec<Expense>, String>;

    /// Find all expenses with pagination and filters
    /// Returns tuple of (expenses, total_count)
    async fn find_all_paginated(
        &self,
        page_request: &PageRequest,
        filters: &ExpenseFilters,
    ) -> Result<(Vec<Expense>, i64), String>;

    async fn update(&self, expense: &Expense) -> Result<Expense, String>;
    async fn delete(&self, id: Uuid) -> Result<bool, String>;
}
