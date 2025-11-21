use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Represents a computational task in the grid
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: Uuid,
    pub task_type: TaskType,
    pub status: TaskStatus,
    pub assigned_node_id: Option<Uuid>,
    pub data_url: String,
    pub result_hash: Option<String>,
    pub deadline: DateTime<Utc>,
    pub energy_used_wh: Option<f64>,
    pub carbon_credits_awarded: Option<f64>,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskType {
    MlTrain,      // Machine learning training
    DataHash,     // Data hashing/verification
    Render,       // Image/video rendering
    Scientific,   // Scientific computation
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Pending,
    Assigned,
    InProgress,
    Completed,
    Failed,
    Expired,
}

impl Task {
    /// Creates a new task with validation
    pub fn new(
        task_type: TaskType,
        data_url: String,
        deadline_minutes: i64,
    ) -> Result<Self, String> {
        if data_url.trim().is_empty() {
            return Err("Data URL cannot be empty".to_string());
        }

        if deadline_minutes <= 0 {
            return Err("Deadline must be in the future".to_string());
        }

        let now = Utc::now();
        let deadline = now + chrono::Duration::minutes(deadline_minutes);

        Ok(Task {
            id: Uuid::new_v4(),
            task_type,
            status: TaskStatus::Pending,
            assigned_node_id: None,
            data_url,
            result_hash: None,
            deadline,
            energy_used_wh: None,
            carbon_credits_awarded: None,
            created_at: now,
            completed_at: None,
        })
    }

    /// Assigns the task to a specific node
    pub fn assign_to_node(&mut self, node_id: Uuid) -> Result<(), String> {
        if self.status != TaskStatus::Pending {
            return Err(format!(
                "Cannot assign task in status {:?}",
                self.status
            ));
        }

        self.assigned_node_id = Some(node_id);
        self.status = TaskStatus::Assigned;
        Ok(())
    }

    /// Marks the task as in progress
    pub fn start(&mut self) -> Result<(), String> {
        if self.status != TaskStatus::Assigned {
            return Err(format!(
                "Cannot start task in status {:?}",
                self.status
            ));
        }

        if self.is_expired() {
            self.status = TaskStatus::Expired;
            return Err("Task has expired".to_string());
        }

        self.status = TaskStatus::InProgress;
        Ok(())
    }

    /// Completes the task with results
    pub fn complete(&mut self, result_hash: String, energy_used_wh: f64) -> Result<(), String> {
        if self.status != TaskStatus::InProgress {
            return Err(format!(
                "Cannot complete task in status {:?}",
                self.status
            ));
        }

        if result_hash.trim().is_empty() {
            return Err("Result hash cannot be empty".to_string());
        }

        if energy_used_wh < 0.0 {
            return Err("Energy used cannot be negative".to_string());
        }

        // Calculate carbon credits based on energy used
        // Formula: 1 Wh of green energy = 0.0005 kg CO2 avoided = 0.0005 carbon credits
        let carbon_credits = energy_used_wh * 0.0005;

        self.result_hash = Some(result_hash);
        self.energy_used_wh = Some(energy_used_wh);
        self.carbon_credits_awarded = Some(carbon_credits);
        self.status = TaskStatus::Completed;
        self.completed_at = Some(Utc::now());
        Ok(())
    }

    /// Marks the task as failed
    pub fn fail(&mut self, reason: &str) -> Result<(), String> {
        if self.status == TaskStatus::Completed {
            return Err("Cannot fail a completed task".to_string());
        }

        log::warn!("Task {} failed: {}", self.id, reason);
        self.status = TaskStatus::Failed;
        Ok(())
    }

    /// Checks if the task has expired
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.deadline
    }

    /// Returns the estimated reward in euros
    pub fn estimated_reward(&self) -> f64 {
        match self.task_type {
            TaskType::MlTrain => 0.05,
            TaskType::DataHash => 0.01,
            TaskType::Render => 0.03,
            TaskType::Scientific => 0.04,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_task_success() {
        let task = Task::new(
            TaskType::MlTrain,
            "s3://bucket/data.csv".to_string(),
            60,
        );

        assert!(task.is_ok());
        let task = task.unwrap();
        assert_eq!(task.task_type, TaskType::MlTrain);
        assert_eq!(task.status, TaskStatus::Pending);
        assert!(task.assigned_node_id.is_none());
    }

    #[test]
    fn test_create_task_empty_url() {
        let result = Task::new(TaskType::MlTrain, "".to_string(), 60);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Data URL cannot be empty");
    }

    #[test]
    fn test_assign_task() {
        let mut task = Task::new(
            TaskType::MlTrain,
            "s3://bucket/data.csv".to_string(),
            60,
        )
        .unwrap();

        let node_id = Uuid::new_v4();
        let result = task.assign_to_node(node_id);

        assert!(result.is_ok());
        assert_eq!(task.status, TaskStatus::Assigned);
        assert_eq!(task.assigned_node_id, Some(node_id));
    }

    #[test]
    fn test_start_task() {
        let mut task = Task::new(
            TaskType::MlTrain,
            "s3://bucket/data.csv".to_string(),
            60,
        )
        .unwrap();

        task.assign_to_node(Uuid::new_v4()).unwrap();
        let result = task.start();

        assert!(result.is_ok());
        assert_eq!(task.status, TaskStatus::InProgress);
    }

    #[test]
    fn test_complete_task() {
        let mut task = Task::new(
            TaskType::MlTrain,
            "s3://bucket/data.csv".to_string(),
            60,
        )
        .unwrap();

        task.assign_to_node(Uuid::new_v4()).unwrap();
        task.start().unwrap();

        let result = task.complete("abc123def456".to_string(), 12.5);

        assert!(result.is_ok());
        assert_eq!(task.status, TaskStatus::Completed);
        assert_eq!(task.result_hash, Some("abc123def456".to_string()));
        assert_eq!(task.energy_used_wh, Some(12.5));
        // 12.5 * 0.0005 = 0.00625
        assert!((task.carbon_credits_awarded.unwrap() - 0.00625).abs() < 0.0001);
    }

    #[test]
    fn test_fail_task() {
        let mut task = Task::new(
            TaskType::MlTrain,
            "s3://bucket/data.csv".to_string(),
            60,
        )
        .unwrap();

        task.assign_to_node(Uuid::new_v4()).unwrap();
        task.start().unwrap();

        let result = task.fail("Network error");

        assert!(result.is_ok());
        assert_eq!(task.status, TaskStatus::Failed);
    }

    #[test]
    fn test_estimated_reward() {
        let task_ml = Task::new(TaskType::MlTrain, "url".to_string(), 60).unwrap();
        assert!((task_ml.estimated_reward() - 0.05).abs() < 0.001);

        let task_hash = Task::new(TaskType::DataHash, "url".to_string(), 60).unwrap();
        assert!((task_hash.estimated_reward() - 0.01).abs() < 0.001);
    }
}
