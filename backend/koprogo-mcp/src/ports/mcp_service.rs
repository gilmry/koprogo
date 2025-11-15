// MCP Service Port - Interface for AI inference execution
use crate::core::*;
use async_trait::async_trait;
use uuid::Uuid;

/// Port for MCP service - handles AI model inference
#[async_trait]
pub trait McpService: Send + Sync {
    /// Execute a chat completion request
    async fn chat(&self, request: McpRequest) -> Result<McpResponse, McpError>;

    /// Execute a chat completion with streaming
    async fn chat_stream(
        &self,
        request: McpRequest,
    ) -> Result<tokio::sync::mpsc::Receiver<StreamChunk>, McpError>;

    /// Execute a grid computing task
    async fn execute_task(&self, task: McpTask) -> Result<McpTask, McpError>;

    /// Get task status
    async fn get_task_status(&self, task_id: Uuid) -> Result<TaskStatus, McpError>;

    /// Health check for MCP service
    async fn health_check(&self) -> Result<HealthStatus, McpError>;
}

/// Stream chunk for streaming responses
#[derive(Debug, Clone)]
pub struct StreamChunk {
    pub content: String,
    pub finish_reason: Option<FinishReason>,
}

/// Health status of MCP service
#[derive(Debug, Clone)]
pub struct HealthStatus {
    pub is_healthy: bool,
    pub edge_nodes_available: usize,
    pub active_tasks: usize,
    pub avg_latency_ms: u64,
}

/// MCP-specific errors
#[derive(Debug, thiserror::Error)]
pub enum McpError {
    #[error("Model not found: {0}")]
    ModelNotFound(String),

    #[error("Model unavailable: {0}")]
    ModelUnavailable(String),

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Execution failed: {0}")]
    ExecutionFailed(String),

    #[error("Edge node unavailable: {0}")]
    EdgeUnavailable(String),

    #[error("Task not found: {0}")]
    TaskNotFound(Uuid),

    #[error("Timeout: {0}")]
    Timeout(String),

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
}
