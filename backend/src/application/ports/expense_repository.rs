use crate::domain::entities::Expense;
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait ExpenseRepository: Send + Sync {
    async fn create(&self, expense: &Expense) -> Result<Expense, String>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Expense>, String>;
    async fn find_by_building(&self, building_id: Uuid) -> Result<Vec<Expense>, String>;
    async fn update(&self, expense: &Expense) -> Result<Expense, String>;
    async fn delete(&self, id: Uuid) -> Result<bool, String>;
}
