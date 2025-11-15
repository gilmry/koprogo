use crate::core::{Task, TaskStatus, TaskType};
use crate::ports::{TaskRepository, TaskStats};
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

pub struct PostgresTaskRepository {
    pool: PgPool,
}

impl PostgresTaskRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TaskRepository for PostgresTaskRepository {
    async fn create(&self, task: &Task) -> Result<Task, String> {
        let task_type_str = task_type_to_str(task.task_type);
        let status_str = task_status_to_str(task.status);

        sqlx::query_as!(
            TaskRow,
            r#"
            INSERT INTO grid_tasks (
                id, task_type, status, assigned_node_id, data_url, result_hash,
                deadline, energy_used_wh, carbon_credits_awarded, created_at, completed_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING *
            "#,
            task.id,
            task_type_str,
            status_str,
            task.assigned_node_id,
            task.data_url,
            task.result_hash,
            task.deadline,
            task.energy_used_wh,
            task.carbon_credits_awarded,
            task.created_at,
            task.completed_at
        )
        .fetch_one(&self.pool)
        .await
        .map(|row| row.into())
        .map_err(|e| format!("Failed to create task: {}", e))
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Task>, String> {
        sqlx::query_as!(TaskRow, "SELECT * FROM grid_tasks WHERE id = $1", id)
            .fetch_optional(&self.pool)
            .await
            .map(|opt| opt.map(|row| row.into()))
            .map_err(|e| format!("Failed to find task: {}", e))
    }

    async fn update(&self, task: &Task) -> Result<Task, String> {
        let task_type_str = task_type_to_str(task.task_type);
        let status_str = task_status_to_str(task.status);

        sqlx::query_as!(
            TaskRow,
            r#"
            UPDATE grid_tasks
            SET task_type = $2, status = $3, assigned_node_id = $4, data_url = $5,
                result_hash = $6, deadline = $7, energy_used_wh = $8,
                carbon_credits_awarded = $9, completed_at = $10
            WHERE id = $1
            RETURNING *
            "#,
            task.id,
            task_type_str,
            status_str,
            task.assigned_node_id,
            task.data_url,
            task.result_hash,
            task.deadline,
            task.energy_used_wh,
            task.carbon_credits_awarded,
            task.completed_at
        )
        .fetch_one(&self.pool)
        .await
        .map(|row| row.into())
        .map_err(|e| format!("Failed to update task: {}", e))
    }

    async fn list(
        &self,
        status: Option<TaskStatus>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Task>, String> {
        if let Some(status) = status {
            let status_str = task_status_to_str(status);
            sqlx::query_as!(
                TaskRow,
                "SELECT * FROM grid_tasks WHERE status = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3",
                status_str,
                limit,
                offset
            )
            .fetch_all(&self.pool)
            .await
        } else {
            sqlx::query_as!(
                TaskRow,
                "SELECT * FROM grid_tasks ORDER BY created_at DESC LIMIT $1 OFFSET $2",
                limit,
                offset
            )
            .fetch_all(&self.pool)
            .await
        }
        .map(|rows| rows.into_iter().map(|row| row.into()).collect())
        .map_err(|e| format!("Failed to list tasks: {}", e))
    }

    async fn find_next_pending(&self) -> Result<Option<Task>, String> {
        sqlx::query_as!(
            TaskRow,
            r#"
            SELECT * FROM grid_tasks
            WHERE status = 'pending' AND deadline > NOW()
            ORDER BY deadline ASC
            LIMIT 1
            "#
        )
        .fetch_optional(&self.pool)
        .await
        .map(|opt| opt.map(|row| row.into()))
        .map_err(|e| format!("Failed to find next pending task: {}", e))
    }

    async fn find_by_node(&self, node_id: Uuid) -> Result<Vec<Task>, String> {
        sqlx::query_as!(
            TaskRow,
            "SELECT * FROM grid_tasks WHERE assigned_node_id = $1",
            node_id
        )
        .fetch_all(&self.pool)
        .await
        .map(|rows| rows.into_iter().map(|row| row.into()).collect())
        .map_err(|e| format!("Failed to find tasks by node: {}", e))
    }

    async fn get_stats(&self) -> Result<TaskStats, String> {
        let stats = sqlx::query!(
            r#"
            SELECT
                COUNT(*) as "total_tasks!",
                COUNT(*) FILTER (WHERE status = 'pending') as "pending_tasks!",
                COUNT(*) FILTER (WHERE status = 'completed') as "completed_tasks!",
                COUNT(*) FILTER (WHERE status = 'failed') as "failed_tasks!",
                COALESCE(SUM(energy_used_wh), 0) as "total_energy_used_wh!",
                COALESCE(SUM(carbon_credits_awarded), 0) as "total_carbon_credits!"
            FROM grid_tasks
            "#
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to get task stats: {}", e))?;

        Ok(TaskStats {
            total_tasks: stats.total_tasks,
            pending_tasks: stats.pending_tasks,
            completed_tasks: stats.completed_tasks,
            failed_tasks: stats.failed_tasks,
            total_energy_used_wh: stats.total_energy_used_wh,
            total_carbon_credits: stats.total_carbon_credits,
        })
    }
}

// Helper functions
fn task_type_to_str(task_type: TaskType) -> &'static str {
    match task_type {
        TaskType::MlTrain => "ml_train",
        TaskType::DataHash => "data_hash",
        TaskType::Render => "render",
        TaskType::Scientific => "scientific",
    }
}

fn task_status_to_str(status: TaskStatus) -> &'static str {
    match status {
        TaskStatus::Pending => "pending",
        TaskStatus::Assigned => "assigned",
        TaskStatus::InProgress => "in_progress",
        TaskStatus::Completed => "completed",
        TaskStatus::Failed => "failed",
        TaskStatus::Expired => "expired",
    }
}

// Database row
#[derive(Debug)]
struct TaskRow {
    id: Uuid,
    task_type: String,
    status: String,
    assigned_node_id: Option<Uuid>,
    data_url: String,
    result_hash: Option<String>,
    deadline: chrono::DateTime<chrono::Utc>,
    energy_used_wh: Option<f64>,
    carbon_credits_awarded: Option<f64>,
    created_at: chrono::DateTime<chrono::Utc>,
    completed_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl From<TaskRow> for Task {
    fn from(row: TaskRow) -> Self {
        let task_type = match row.task_type.as_str() {
            "ml_train" => TaskType::MlTrain,
            "data_hash" => TaskType::DataHash,
            "render" => TaskType::Render,
            "scientific" => TaskType::Scientific,
            _ => TaskType::DataHash,
        };

        let status = match row.status.as_str() {
            "pending" => TaskStatus::Pending,
            "assigned" => TaskStatus::Assigned,
            "in_progress" => TaskStatus::InProgress,
            "completed" => TaskStatus::Completed,
            "failed" => TaskStatus::Failed,
            "expired" => TaskStatus::Expired,
            _ => TaskStatus::Pending,
        };

        Task {
            id: row.id,
            task_type,
            status,
            assigned_node_id: row.assigned_node_id,
            data_url: row.data_url,
            result_hash: row.result_hash,
            deadline: row.deadline,
            energy_used_wh: row.energy_used_wh,
            carbon_credits_awarded: row.carbon_credits_awarded,
            created_at: row.created_at,
            completed_at: row.completed_at,
        }
    }
}
