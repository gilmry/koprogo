use crate::core::{Task, TaskStatus};
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait TaskRepository: Send + Sync {
    /// Creates a new task
    async fn create(&self, task: &Task) -> Result<Task, String>;

    /// Finds a task by its ID
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Task>, String>;

    /// Updates an existing task
    async fn update(&self, task: &Task) -> Result<Task, String>;

    /// Lists tasks with optional status filter
    async fn list(
        &self,
        status: Option<TaskStatus>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Task>, String>;

    /// Finds the next pending task for assignment
    async fn find_next_pending(&self) -> Result<Option<Task>, String>;

    /// Lists tasks assigned to a specific node
    async fn find_by_node(&self, node_id: Uuid) -> Result<Vec<Task>, String>;

    /// Gets task statistics
    async fn get_stats(&self) -> Result<TaskStats, String>;
}

#[derive(Debug, Clone)]
pub struct TaskStats {
    pub total_tasks: i64,
    pub pending_tasks: i64,
    pub completed_tasks: i64,
    pub failed_tasks: i64,
    pub total_energy_used_wh: f64,
    pub total_carbon_credits: f64,
}
