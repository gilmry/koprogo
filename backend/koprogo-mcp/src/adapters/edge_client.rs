// Edge client - Communicates with koprogo-node (Raspberry Pi)
use crate::core::*;
use crate::ports::*;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{error, info, warn};

/// Edge client for communicating with local AI nodes
pub struct EdgeClient {
    client: Client,
    node_urls: Vec<String>,
    timeout: Duration,
}

impl EdgeClient {
    /// Create a new edge client
    pub fn new(node_urls: Vec<String>) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            node_urls,
            timeout: Duration::from_secs(30),
        }
    }

    /// Execute MCP request on edge node
    pub async fn execute_on_edge(&self, request: &McpRequest) -> Result<McpResponse, McpError> {
        if self.node_urls.is_empty() {
            return Err(McpError::EdgeUnavailable(
                "No edge nodes configured".to_string(),
            ));
        }

        // Try each node until one succeeds
        let mut last_error = None;
        for node_url in &self.node_urls {
            match self.try_node(node_url, request).await {
                Ok(response) => {
                    info!("Request executed on edge node: {}", node_url);
                    return Ok(response);
                }
                Err(e) => {
                    warn!("Failed to execute on node {}: {}", node_url, e);
                    last_error = Some(e);
                }
            }
        }

        Err(last_error.unwrap_or_else(|| {
            McpError::EdgeUnavailable("All edge nodes failed".to_string())
        }))
    }

    /// Try executing request on a specific node
    async fn try_node(&self, node_url: &str, request: &McpRequest) -> Result<McpResponse, McpError> {
        let url = format!("{}/mcp/v1/chat", node_url);

        let edge_request = EdgeChatRequest {
            model: request.model.clone(),
            messages: request.messages.clone(),
            max_tokens: request.max_tokens,
            temperature: request.temperature,
        };

        let start = std::time::Instant::now();

        let response = self
            .client
            .post(&url)
            .json(&edge_request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(McpError::ExecutionFailed(format!(
                "Node returned error: {}",
                error_text
            )));
        }

        let edge_response: EdgeChatResponse = response.json().await?;

        let latency_ms = start.elapsed().as_millis() as u64;

        // Convert edge response to MCP response
        Ok(McpResponse::new(
            request.id,
            edge_response.model,
            edge_response.content,
            FinishReason::Stop,
            TokenUsage::new(
                edge_response.usage.prompt_tokens,
                edge_response.usage.completion_tokens,
            ),
            ExecutionInfo {
                execution_type: ExecutionType::Edge,
                node_id: Some(node_url.to_string()),
                latency_ms,
                grid_task_id: None,
            },
        ))
    }

    /// Check health of edge nodes
    pub async fn check_health(&self) -> Vec<EdgeNodeHealth> {
        let mut healths = Vec::new();

        for node_url in &self.node_urls {
            let health = self.check_node_health(node_url).await;
            healths.push(health);
        }

        healths
    }

    /// Check health of a specific node
    async fn check_node_health(&self, node_url: &str) -> EdgeNodeHealth {
        let url = format!("{}/mcp/v1/health", node_url);

        match self.client.get(&url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    if let Ok(health) = response.json::<EdgeNodeHealth>().await {
                        health
                    } else {
                        EdgeNodeHealth {
                            node_url: node_url.to_string(),
                            is_healthy: false,
                            models_loaded: vec![],
                            active_requests: 0,
                            total_memory_mb: 0,
                            used_memory_mb: 0,
                        }
                    }
                } else {
                    EdgeNodeHealth {
                        node_url: node_url.to_string(),
                        is_healthy: false,
                        models_loaded: vec![],
                        active_requests: 0,
                        total_memory_mb: 0,
                        used_memory_mb: 0,
                    }
                }
            }
            Err(e) => {
                error!("Failed to check node health {}: {}", node_url, e);
                EdgeNodeHealth {
                    node_url: node_url.to_string(),
                    is_healthy: false,
                    models_loaded: vec![],
                    active_requests: 0,
                    total_memory_mb: 0,
                    used_memory_mb: 0,
                }
            }
        }
    }
}

/// Edge chat request (simplified for node API)
#[derive(Debug, Serialize)]
struct EdgeChatRequest {
    model: String,
    messages: Vec<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
}

/// Edge chat response
#[derive(Debug, Deserialize)]
struct EdgeChatResponse {
    model: String,
    content: String,
    usage: EdgeTokenUsage,
}

#[derive(Debug, Deserialize)]
struct EdgeTokenUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
}

/// Edge node health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeNodeHealth {
    pub node_url: String,
    pub is_healthy: bool,
    pub models_loaded: Vec<String>,
    pub active_requests: u32,
    pub total_memory_mb: u32,
    pub used_memory_mb: u32,
}
