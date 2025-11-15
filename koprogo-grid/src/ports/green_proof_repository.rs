use crate::core::GreenProof;
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait GreenProofRepository: Send + Sync {
    /// Creates a new green proof
    async fn create(&self, proof: &GreenProof) -> Result<GreenProof, String>;

    /// Finds a proof by its ID
    async fn find_by_id(&self, id: Uuid) -> Result<Option<GreenProof>, String>;

    /// Finds proofs by task ID
    async fn find_by_task(&self, task_id: Uuid) -> Result<Vec<GreenProof>, String>;

    /// Finds proofs by node ID
    async fn find_by_node(&self, node_id: Uuid) -> Result<Vec<GreenProof>, String>;

    /// Gets the latest proof (for blockchain chaining)
    async fn get_latest(&self) -> Result<Option<GreenProof>, String>;

    /// Lists all proofs with pagination
    async fn list(&self, limit: i64, offset: i64) -> Result<Vec<GreenProof>, String>;

    /// Verifies the blockchain integrity
    async fn verify_chain(&self) -> Result<bool, String>;
}
