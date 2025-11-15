// MCP Repository Port - Interface for persistence
use crate::core::*;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Port for MCP request/response logging and persistence
#[async_trait]
pub trait McpRepository: Send + Sync {
    /// Log an MCP request
    async fn log_request(&self, request: &McpRequest) -> Result<(), String>;

    /// Log an MCP response
    async fn log_response(&self, response: &McpResponse) -> Result<(), String>;

    /// Get request by ID
    async fn get_request(&self, request_id: Uuid) -> Result<McpRequest, String>;

    /// Get response for a request
    async fn get_response(&self, request_id: Uuid) -> Result<McpResponse, String>;

    /// Get request history for a user
    async fn get_user_history(
        &self,
        user_id: Uuid,
        limit: usize,
    ) -> Result<Vec<(McpRequest, Option<McpResponse>)>, String>;

    /// Get request history for a context (e.g., copro:123)
    async fn get_context_history(
        &self,
        context: &str,
        limit: usize,
    ) -> Result<Vec<(McpRequest, Option<McpResponse>)>, String>;

    /// Get statistics for a time period
    async fn get_statistics(
        &self,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> Result<McpStatistics, String>;
}

/// MCP usage statistics
#[derive(Debug, Clone)]
pub struct McpStatistics {
    pub total_requests: u64,
    pub total_tokens: u64,
    pub total_co2_grams: f64,
    pub edge_requests: u64,
    pub cloud_requests: u64,
    pub grid_requests: u64,
    pub avg_latency_ms: f64,
    pub models_used: Vec<String>,
}
