use crate::core::Node;
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait NodeRepository: Send + Sync {
    /// Creates a new node in the grid
    async fn create(&self, node: &Node) -> Result<Node, String>;

    /// Finds a node by its ID
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Node>, String>;

    /// Updates an existing node
    async fn update(&self, node: &Node) -> Result<Node, String>;

    /// Deletes a node by ID
    async fn delete(&self, id: Uuid) -> Result<(), String>;

    /// Lists all nodes with pagination
    async fn list(&self, limit: i64, offset: i64) -> Result<Vec<Node>, String>;

    /// Lists active nodes (received heartbeat in last 5 minutes)
    async fn list_active(&self) -> Result<Vec<Node>, String>;

    /// Finds nodes by location
    async fn find_by_location(&self, location: &str) -> Result<Vec<Node>, String>;

    /// Gets total statistics for all nodes
    async fn get_total_stats(&self) -> Result<NodeStats, String>;
}

#[derive(Debug, Clone)]
pub struct NodeStats {
    pub total_nodes: i64,
    pub active_nodes: i64,
    pub total_cpu_cores: i64,
    pub nodes_with_solar: i64,
    pub total_energy_saved_wh: f64,
    pub total_carbon_credits: f64,
}
