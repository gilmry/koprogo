use crate::core::{Node, Task};
use crate::ports::{NodeRepository, TaskDistributor, TaskRepository};
use async_trait::async_trait;
use std::sync::Arc;
use uuid::Uuid;

pub struct PostgresTaskDistributor {
    node_repo: Arc<dyn NodeRepository>,
    task_repo: Arc<dyn TaskRepository>,
}

impl PostgresTaskDistributor {
    pub fn new(
        node_repo: Arc<dyn NodeRepository>,
        task_repo: Arc<dyn TaskRepository>,
    ) -> Self {
        Self {
            node_repo,
            task_repo,
        }
    }
}

#[async_trait]
impl TaskDistributor for PostgresTaskDistributor {
    async fn assign_task(&self, task_id: Uuid) -> Result<Uuid, String> {
        // Find the task
        let mut task = self
            .task_repo
            .find_by_id(task_id)
            .await?
            .ok_or_else(|| "Task not found".to_string())?;

        // Find the best node
        let node = self
            .find_best_node(&task)
            .await?
            .ok_or_else(|| "No suitable node available".to_string())?;

        // Assign the task
        task.assign_to_node(node.id)?;
        self.task_repo.update(&task).await?;

        Ok(node.id)
    }

    async fn find_best_node(&self, _task: &Task) -> Result<Option<Node>, String> {
        // Get all active nodes sorted by eco score
        let nodes = self.node_repo.list_active().await?;

        if nodes.is_empty() {
            return Ok(None);
        }

        // Return the node with the highest eco score
        // In a real implementation, this would also consider:
        // - Node's current load
        // - Task requirements (CPU, memory, etc.)
        // - Geographic proximity
        // - Task priority
        Ok(nodes.into_iter().next())
    }

    async fn rebalance(&self) -> Result<(), String> {
        // In a real implementation, this would:
        // 1. Analyze current task distribution
        // 2. Identify overloaded/underloaded nodes
        // 3. Reassign tasks for better balance
        // For now, this is a no-op
        log::info!("Task rebalancing requested (not yet implemented)");
        Ok(())
    }
}
