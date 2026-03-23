use async_trait::async_trait;
use uuid::Uuid;
use crate::domain::entities::contract_evaluation::ContractEvaluation;

#[async_trait]
pub trait ContractEvaluationRepository: Send + Sync {
    async fn create(&self, evaluation: &ContractEvaluation) -> Result<ContractEvaluation, String>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<ContractEvaluation>, String>;
    async fn find_by_service_provider(&self, provider_id: Uuid, page: i64, per_page: i64) -> Result<Vec<ContractEvaluation>, String>;
    async fn find_by_quote(&self, quote_id: Uuid) -> Result<Vec<ContractEvaluation>, String>;
    async fn find_by_ticket(&self, ticket_id: Uuid) -> Result<Vec<ContractEvaluation>, String>;
    async fn find_by_building(&self, building_id: Uuid, page: i64, per_page: i64) -> Result<Vec<ContractEvaluation>, String>;
    async fn find_legal_evaluations(&self, building_id: Uuid, page: i64, per_page: i64) -> Result<Vec<ContractEvaluation>, String>;
    async fn update(&self, evaluation: &ContractEvaluation) -> Result<ContractEvaluation, String>;
    async fn delete(&self, id: Uuid) -> Result<(), String>;
    async fn count_by_service_provider(&self, provider_id: Uuid) -> Result<i64, String>;
    async fn average_score_by_provider(&self, provider_id: Uuid) -> Result<Option<f64>, String>;
}
