use async_trait::async_trait;
use uuid::Uuid;
use crate::domain::entities::service_provider::{ServiceProvider, TradeCategory};

#[async_trait]
pub trait ServiceProviderRepository: Send + Sync {
    async fn create(&self, provider: &ServiceProvider) -> Result<ServiceProvider, String>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<ServiceProvider>, String>;
    async fn find_by_slug(&self, slug: &str) -> Result<Option<ServiceProvider>, String>;
    async fn find_all(&self, organization_id: Option<Uuid>, page: i64, per_page: i64) -> Result<Vec<ServiceProvider>, String>;
    async fn find_by_trade_category(&self, category: TradeCategory, page: i64, per_page: i64) -> Result<Vec<ServiceProvider>, String>;
    async fn search(&self, query: &str, postal_code: Option<&str>, page: i64, per_page: i64) -> Result<Vec<ServiceProvider>, String>;
    async fn update(&self, provider: &ServiceProvider) -> Result<ServiceProvider, String>;
    async fn delete(&self, id: Uuid) -> Result<(), String>;
    async fn update_rating(&self, id: Uuid, rating_avg: f64, reviews_count: i32) -> Result<(), String>;
    async fn count_by_organization(&self, organization_id: Uuid) -> Result<i64, String>;
}
