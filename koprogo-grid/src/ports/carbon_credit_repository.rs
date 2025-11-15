use crate::core::{CarbonCredit, CreditStatus};
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait CarbonCreditRepository: Send + Sync {
    /// Creates a new carbon credit
    async fn create(&self, credit: &CarbonCredit) -> Result<CarbonCredit, String>;

    /// Finds a credit by its ID
    async fn find_by_id(&self, id: Uuid) -> Result<Option<CarbonCredit>, String>;

    /// Updates an existing credit
    async fn update(&self, credit: &CarbonCredit) -> Result<CarbonCredit, String>;

    /// Lists credits by node ID
    async fn find_by_node(&self, node_id: Uuid) -> Result<Vec<CarbonCredit>, String>;

    /// Lists credits by status
    async fn find_by_status(&self, status: CreditStatus) -> Result<Vec<CarbonCredit>, String>;

    /// Gets the total cooperative fund value
    async fn get_cooperative_fund(&self) -> Result<f64, String>;

    /// Gets statistics for a node
    async fn get_node_stats(&self, node_id: Uuid) -> Result<CreditStats, String>;
}

#[derive(Debug, Clone)]
pub struct CreditStats {
    pub total_credits: i64,
    pub total_kg_co2: f64,
    pub total_euro_value: f64,
    pub node_share_eur: f64,
    pub cooperative_share_eur: f64,
}
