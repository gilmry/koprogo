// Grid client - Communicates with grid computing network
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::{error, info};
use uuid::Uuid;

/// Grid client for distributed computing
pub struct GridClient {
    server_url: String,
    client: Client,
}

impl GridClient {
    pub fn new(server_url: String) -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            server_url,
            client,
        }
    }

    /// Register this node with the grid
    pub async fn register_node(&self, node_id: &str, capabilities: NodeCapabilities) -> Result<(), String> {
        let url = format!("{}/grid/v1/nodes/register", self.server_url);

        let request = RegisterNodeRequest {
            node_id: node_id.to_string(),
            capabilities,
        };

        match self.client.post(&url).json(&request).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    info!("✅ Node registered with grid");
                    Ok(())
                } else {
                    let error_text = response.text().await.unwrap_or_default();
                    Err(format!("Failed to register: {}", error_text))
                }
            }
            Err(e) => {
                error!("Failed to register with grid: {}", e);
                Err(e.to_string())
            }
        }
    }

    /// Poll for tasks from the grid
    pub async fn poll_task(&self, node_id: &str) -> Result<Option<GridTask>, String> {
        let url = format!("{}/grid/v1/tasks/poll?node_id={}", self.server_url, node_id);

        match self.client.get(&url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    let task: Option<GridTask> = response.json().await.map_err(|e| e.to_string())?;
                    if task.is_some() {
                        info!("📥 Received task from grid");
                    }
                    Ok(task)
                } else if response.status() == 204 {
                    // No tasks available
                    Ok(None)
                } else {
                    Err(format!("Failed to poll tasks: {}", response.status()))
                }
            }
            Err(e) => {
                error!("Failed to poll grid: {}", e);
                Err(e.to_string())
            }
        }
    }

    /// Submit task result back to grid
    pub async fn submit_result(&self, task_id: Uuid, result: serde_json::Value) -> Result<(), String> {
        let url = format!("{}/grid/v1/tasks/{}/result", self.server_url, task_id);

        let request = TaskResultRequest { result };

        match self.client.post(&url).json(&request).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    info!("✅ Task result submitted");
                    Ok(())
                } else {
                    Err(format!("Failed to submit result: {}", response.status()))
                }
            }
            Err(e) => {
                error!("Failed to submit result: {}", e);
                Err(e.to_string())
            }
        }
    }

    /// Report node heartbeat
    pub async fn heartbeat(&self, node_id: &str, stats: NodeStats) -> Result<(), String> {
        let url = format!("{}/grid/v1/nodes/{}/heartbeat", self.server_url, node_id);

        match self.client.post(&url).json(&stats).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    Ok(())
                } else {
                    Err(format!("Heartbeat failed: {}", response.status()))
                }
            }
            Err(e) => {
                error!("Heartbeat failed: {}", e);
                Err(e.to_string())
            }
        }
    }
}

#[derive(Debug, Serialize)]
struct RegisterNodeRequest {
    node_id: String,
    capabilities: NodeCapabilities,
}

#[derive(Debug, Clone, Serialize)]
pub struct NodeCapabilities {
    pub cpu_cores: u32,
    pub total_memory_mb: u32,
    pub models_available: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct GridTask {
    pub id: Uuid,
    pub task_type: String,
    pub input_data: serde_json::Value,
}

#[derive(Debug, Serialize)]
struct TaskResultRequest {
    result: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct NodeStats {
    pub cpu_usage_percent: f32,
    pub memory_used_mb: u32,
    pub active_tasks: u32,
}

/*
 * GRID COMPUTING INTEGRATION NOTES:
 *
 * The grid computing network allows distributing heavy AI tasks across
 * multiple Raspberry Pi nodes. This is useful for:
 *
 * 1. OCR on large PDF documents
 * 2. Batch processing of invoices
 * 3. Document translation
 * 4. Meeting minutes summarization
 *
 * Architecture:
 * - Grid Server (central coordinator): Assigns tasks to nodes
 * - Edge Nodes (Raspberry Pi): Execute tasks and report results
 * - Proof of Green: Validates carbon-neutral execution
 *
 * Task flow:
 * 1. Client submits task to grid server (POST /grid/v1/tasks)
 * 2. Grid server queues task
 * 3. Edge nodes poll for tasks (GET /grid/v1/tasks/poll)
 * 4. Node executes task (local AI inference)
 * 5. Node submits result (POST /grid/v1/tasks/{id}/result)
 * 6. Client retrieves result (GET /grid/v1/tasks/{id})
 *
 * Rewards:
 * - Nodes earn MCP tokens for completed tasks
 * - CO₂ savings are tracked and contribute to solidarity fund
 * - Passive income for copro members who run edge nodes
 *
 * Security:
 * - Tasks are signed and verified
 * - Results include proof of execution
 * - Anti-fraud: Multiple nodes can verify same task
 */
