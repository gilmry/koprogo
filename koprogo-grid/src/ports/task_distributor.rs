use crate::core::{Node, Task};
use async_trait::async_trait;
use uuid::Uuid;

/// Service responsible for intelligent task distribution
#[async_trait]
pub trait TaskDistributor: Send + Sync {
    /// Assigns a task to the most suitable node
    /// Selection criteria:
    /// - Node eco score (higher is better)
    /// - Node availability (CPU cores, current load)
    /// - Geographic proximity (if applicable)
    async fn assign_task(&self, task_id: Uuid) -> Result<Uuid, String>;

    /// Finds the best node for a given task
    async fn find_best_node(&self, task: &Task) -> Result<Option<Node>, String>;

    /// Rebalances tasks across nodes (for optimization)
    async fn rebalance(&self) -> Result<(), String>;
}
