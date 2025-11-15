use serde::{Deserialize, Serialize};
use uuid::Uuid;

// Request DTOs
#[derive(Debug, Deserialize)]
pub struct RegisterNodeRequest {
    pub node_id: Option<Uuid>,
    pub name: String,
    pub cpu_cores: u32,
    pub has_solar: bool,
    pub location: String,
}

#[derive(Debug, Deserialize)]
pub struct HeartbeatRequest {
    pub node_id: Uuid,
    pub cpu_usage: f64,
    pub solar_watts: f64,
}

#[derive(Debug, Deserialize)]
pub struct CreateTaskRequest {
    pub task_type: String,
    pub data_url: String,
    pub deadline_minutes: i64,
}

#[derive(Debug, Deserialize)]
pub struct ReportTaskRequest {
    pub task_id: Uuid,
    pub result_hash: String,
    pub energy_used_wh: f64,
    pub solar_contribution_wh: f64,
}

// Response DTOs
#[derive(Debug, Serialize)]
pub struct NodeResponse {
    pub id: Uuid,
    pub name: String,
    pub cpu_cores: u32,
    pub has_solar: bool,
    pub location: String,
    pub status: String,
    pub eco_score: f64,
    pub total_energy_saved_wh: f64,
    pub total_carbon_credits: f64,
}

#[derive(Debug, Serialize)]
pub struct TaskResponse {
    pub id: Uuid,
    pub task_type: String,
    pub status: String,
    pub data_url: String,
    pub deadline: String,
    pub estimated_reward: f64,
}

#[derive(Debug, Serialize)]
pub struct StatsResponse {
    pub nodes: NodeStatsResponse,
    pub tasks: TaskStatsResponse,
    pub cooperative_fund_eur: f64,
}

#[derive(Debug, Serialize)]
pub struct NodeStatsResponse {
    pub total_nodes: i64,
    pub active_nodes: i64,
    pub total_cpu_cores: i64,
    pub nodes_with_solar: i64,
    pub total_energy_saved_wh: f64,
    pub total_carbon_credits: f64,
}

#[derive(Debug, Serialize)]
pub struct TaskStatsResponse {
    pub total_tasks: i64,
    pub pending_tasks: i64,
    pub completed_tasks: i64,
    pub failed_tasks: i64,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}
