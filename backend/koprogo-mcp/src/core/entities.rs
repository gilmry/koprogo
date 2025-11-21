// MCP Domain Entities - Core business objects
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// MCP Request - Represents an AI inference request
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct McpRequest {
    pub id: Uuid,
    pub model: String,
    pub messages: Vec<Message>,
    pub context: Option<String>, // e.g., "copro:123"
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
    pub stream: bool,
    pub user_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

impl McpRequest {
    /// Create a new MCP request with validation
    pub fn new(
        model: String,
        messages: Vec<Message>,
        context: Option<String>,
    ) -> Result<Self, String> {
        // Validate model name (non-empty, alphanumeric + colon/dash)
        if model.trim().is_empty() {
            return Err("Model name cannot be empty".to_string());
        }

        if !model.chars().all(|c| c.is_alphanumeric() || c == ':' || c == '-' || c == '_') {
            return Err("Model name contains invalid characters".to_string());
        }

        // Validate messages (at least one)
        if messages.is_empty() {
            return Err("Messages cannot be empty".to_string());
        }

        // Validate context format if provided (should be "resource:id")
        if let Some(ref ctx) = context {
            if !ctx.contains(':') {
                return Err("Context must be in format 'resource:id'".to_string());
            }
        }

        Ok(Self {
            id: Uuid::new_v4(),
            model,
            messages,
            context,
            max_tokens: None,
            temperature: None,
            stream: false,
            user_id: None,
            created_at: Utc::now(),
        })
    }

    /// Set maximum tokens for response
    pub fn with_max_tokens(mut self, max_tokens: u32) -> Result<Self, String> {
        if max_tokens == 0 || max_tokens > 100_000 {
            return Err("max_tokens must be between 1 and 100,000".to_string());
        }
        self.max_tokens = Some(max_tokens);
        Ok(self)
    }

    /// Set temperature for response randomness
    pub fn with_temperature(mut self, temperature: f32) -> Result<Self, String> {
        if !(0.0..=2.0).contains(&temperature) {
            return Err("Temperature must be between 0.0 and 2.0".to_string());
        }
        self.temperature = Some(temperature);
        Ok(self)
    }

    /// Enable streaming response
    pub fn with_stream(mut self, stream: bool) -> Self {
        self.stream = stream;
        self
    }

    /// Set user ID for request tracking
    pub fn with_user_id(mut self, user_id: Uuid) -> Self {
        self.user_id = Some(user_id);
        self
    }

    /// Check if request should use edge node (local execution)
    pub fn should_use_edge(&self) -> bool {
        self.model.starts_with("llama") || self.model.contains("local")
    }

    /// Estimate token count (simple heuristic: ~4 chars per token)
    pub fn estimate_input_tokens(&self) -> usize {
        self.messages.iter()
            .map(|m| m.content.len())
            .sum::<usize>() / 4
    }
}

/// Message in a conversation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Message {
    pub role: MessageRole,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

impl Message {
    pub fn new(role: MessageRole, content: String) -> Self {
        Self {
            role,
            content,
            name: None,
        }
    }

    pub fn system(content: String) -> Self {
        Self::new(MessageRole::System, content)
    }

    pub fn user(content: String) -> Self {
        Self::new(MessageRole::User, content)
    }

    pub fn assistant(content: String) -> Self {
        Self::new(MessageRole::Assistant, content)
    }
}

/// Role of a message sender
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    System,
    User,
    Assistant,
}

/// MCP Response - AI model response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResponse {
    pub id: Uuid,
    pub request_id: Uuid,
    pub model: String,
    pub content: String,
    pub finish_reason: FinishReason,
    pub usage: TokenUsage,
    pub execution_info: ExecutionInfo,
    pub created_at: DateTime<Utc>,
}

impl McpResponse {
    pub fn new(
        request_id: Uuid,
        model: String,
        content: String,
        finish_reason: FinishReason,
        usage: TokenUsage,
        execution_info: ExecutionInfo,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            request_id,
            model,
            content,
            finish_reason,
            usage,
            execution_info,
            created_at: Utc::now(),
        }
    }

    /// Calculate CO2 emissions (grams) based on execution type
    /// - Edge (local): 0g CO2
    /// - Cloud API: ~0.3g CO2 per 1000 tokens (GPT-4 equivalent)
    pub fn calculate_co2_grams(&self) -> f64 {
        if self.execution_info.execution_type == ExecutionType::Edge {
            0.0
        } else {
            let total_tokens = self.usage.total_tokens as f64;
            (total_tokens / 1000.0) * 0.3
        }
    }
}

/// Reason why generation finished
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FinishReason {
    Stop,          // Natural completion
    Length,        // Max tokens reached
    ContentFilter, // Content policy violation
    Error,         // Error occurred
}

/// Token usage statistics
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

impl TokenUsage {
    pub fn new(prompt_tokens: u32, completion_tokens: u32) -> Self {
        Self {
            prompt_tokens,
            completion_tokens,
            total_tokens: prompt_tokens + completion_tokens,
        }
    }
}

/// Execution information (where/how the request was processed)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionInfo {
    pub execution_type: ExecutionType,
    pub node_id: Option<String>,
    pub latency_ms: u64,
    pub grid_task_id: Option<Uuid>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ExecutionType {
    Edge,  // Local Raspberry Pi
    Cloud, // External API (Claude, GPT-4)
    Grid,  // Distributed grid computing
}

/// Model information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub provider: ModelProvider,
    pub context_length: u32,
    pub is_available: bool,
    pub supports_streaming: bool,
    pub edge_compatible: bool,
    pub created_at: DateTime<Utc>,
}

impl ModelInfo {
    pub fn new(
        id: String,
        name: String,
        provider: ModelProvider,
        context_length: u32,
    ) -> Result<Self, String> {
        if id.trim().is_empty() {
            return Err("Model ID cannot be empty".to_string());
        }

        if name.trim().is_empty() {
            return Err("Model name cannot be empty".to_string());
        }

        if context_length == 0 {
            return Err("Context length must be greater than 0".to_string());
        }

        Ok(Self {
            id,
            name,
            provider,
            context_length,
            is_available: true,
            supports_streaming: true,
            edge_compatible: provider == ModelProvider::Local,
            created_at: Utc::now(),
        })
    }
}

/// AI Model provider
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ModelProvider {
    Local,     // llama.cpp, ONNX
    Anthropic, // Claude
    OpenAI,    // GPT-4
    Mistral,   // Mistral AI
}

/// MCP Task for grid computing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpTask {
    pub id: Uuid,
    pub task_type: TaskType,
    pub input_data: serde_json::Value,
    pub copro_id: Option<Uuid>,
    pub status: TaskStatus,
    pub result: Option<serde_json::Value>,
    pub assigned_node: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TaskType {
    OcrInvoice,       // OCR on invoice documents
    TranslateDocument, // Document translation
    SummarizeMeeting,  // Meeting minutes summarization
    PredictExpenses,   // Expense prediction model
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_mcp_request_success() {
        let messages = vec![Message::user("Hello".to_string())];
        let request = McpRequest::new(
            "llama3:8b".to_string(),
            messages.clone(),
            Some("copro:123".to_string()),
        );

        assert!(request.is_ok());
        let req = request.unwrap();
        assert_eq!(req.model, "llama3:8b");
        assert_eq!(req.messages, messages);
        assert_eq!(req.context, Some("copro:123".to_string()));
        assert!(req.should_use_edge());
    }

    #[test]
    fn test_create_mcp_request_empty_model() {
        let messages = vec![Message::user("Hello".to_string())];
        let request = McpRequest::new("".to_string(), messages, None);
        assert!(request.is_err());
        assert_eq!(request.unwrap_err(), "Model name cannot be empty");
    }

    #[test]
    fn test_create_mcp_request_invalid_model_chars() {
        let messages = vec![Message::user("Hello".to_string())];
        let request = McpRequest::new("model@invalid!".to_string(), messages, None);
        assert!(request.is_err());
    }

    #[test]
    fn test_create_mcp_request_empty_messages() {
        let request = McpRequest::new("llama3:8b".to_string(), vec![], None);
        assert!(request.is_err());
        assert_eq!(request.unwrap_err(), "Messages cannot be empty");
    }

    #[test]
    fn test_create_mcp_request_invalid_context() {
        let messages = vec![Message::user("Hello".to_string())];
        let request = McpRequest::new(
            "llama3:8b".to_string(),
            messages,
            Some("invalid_context".to_string()),
        );
        assert!(request.is_err());
        assert_eq!(request.unwrap_err(), "Context must be in format 'resource:id'");
    }

    #[test]
    fn test_mcp_request_with_max_tokens() {
        let messages = vec![Message::user("Hello".to_string())];
        let request = McpRequest::new("llama3:8b".to_string(), messages, None)
            .unwrap()
            .with_max_tokens(2000);

        assert!(request.is_ok());
        assert_eq!(request.unwrap().max_tokens, Some(2000));
    }

    #[test]
    fn test_mcp_request_invalid_max_tokens() {
        let messages = vec![Message::user("Hello".to_string())];
        let request = McpRequest::new("llama3:8b".to_string(), messages, None)
            .unwrap()
            .with_max_tokens(0);

        assert!(request.is_err());

        let messages = vec![Message::user("Hello".to_string())];
        let request2 = McpRequest::new("llama3:8b".to_string(), messages, None)
            .unwrap()
            .with_max_tokens(200_000);

        assert!(request2.is_err());
    }

    #[test]
    fn test_mcp_request_with_temperature() {
        let messages = vec![Message::user("Hello".to_string())];
        let request = McpRequest::new("llama3:8b".to_string(), messages, None)
            .unwrap()
            .with_temperature(0.7);

        assert!(request.is_ok());
        assert_eq!(request.unwrap().temperature, Some(0.7));
    }

    #[test]
    fn test_mcp_request_invalid_temperature() {
        let messages = vec![Message::user("Hello".to_string())];
        let request = McpRequest::new("llama3:8b".to_string(), messages, None)
            .unwrap()
            .with_temperature(-0.1);

        assert!(request.is_err());

        let messages = vec![Message::user("Hello".to_string())];
        let request2 = McpRequest::new("llama3:8b".to_string(), messages, None)
            .unwrap()
            .with_temperature(2.5);

        assert!(request2.is_err());
    }

    #[test]
    fn test_should_use_edge() {
        let messages = vec![Message::user("Hello".to_string())];

        let llama_req = McpRequest::new("llama3:8b".to_string(), messages.clone(), None).unwrap();
        assert!(llama_req.should_use_edge());

        let local_req = McpRequest::new("local-model".to_string(), messages.clone(), None).unwrap();
        assert!(local_req.should_use_edge());

        let cloud_req = McpRequest::new("claude-3-opus".to_string(), messages, None).unwrap();
        assert!(!cloud_req.should_use_edge());
    }

    #[test]
    fn test_estimate_input_tokens() {
        let messages = vec![
            Message::user("Hello".to_string()), // 5 chars
            Message::assistant("Hi there!".to_string()), // 9 chars
        ];
        let request = McpRequest::new("llama3:8b".to_string(), messages, None).unwrap();
        // (5 + 9) / 4 = 3.5 -> 3 tokens (integer division)
        assert_eq!(request.estimate_input_tokens(), 3);
    }

    #[test]
    fn test_message_constructors() {
        let sys_msg = Message::system("You are helpful".to_string());
        assert_eq!(sys_msg.role, MessageRole::System);

        let user_msg = Message::user("Hello".to_string());
        assert_eq!(user_msg.role, MessageRole::User);

        let assistant_msg = Message::assistant("Hi!".to_string());
        assert_eq!(assistant_msg.role, MessageRole::Assistant);
    }

    #[test]
    fn test_token_usage() {
        let usage = TokenUsage::new(100, 50);
        assert_eq!(usage.prompt_tokens, 100);
        assert_eq!(usage.completion_tokens, 50);
        assert_eq!(usage.total_tokens, 150);
    }

    #[test]
    fn test_calculate_co2_edge() {
        let exec_info = ExecutionInfo {
            execution_type: ExecutionType::Edge,
            node_id: Some("pi-001".to_string()),
            latency_ms: 50,
            grid_task_id: None,
        };

        let response = McpResponse::new(
            Uuid::new_v4(),
            "llama3:8b".to_string(),
            "Response".to_string(),
            FinishReason::Stop,
            TokenUsage::new(100, 50),
            exec_info,
        );

        assert_eq!(response.calculate_co2_grams(), 0.0);
    }

    #[test]
    fn test_calculate_co2_cloud() {
        let exec_info = ExecutionInfo {
            execution_type: ExecutionType::Cloud,
            node_id: None,
            latency_ms: 200,
            grid_task_id: None,
        };

        let response = McpResponse::new(
            Uuid::new_v4(),
            "claude-3-opus".to_string(),
            "Response".to_string(),
            FinishReason::Stop,
            TokenUsage::new(1000, 500), // 1500 tokens total
            exec_info,
        );

        // 1500 / 1000 * 0.3 = 0.45g CO2
        assert_eq!(response.calculate_co2_grams(), 0.45);
    }

    #[test]
    fn test_create_model_info_success() {
        let model = ModelInfo::new(
            "llama3:8b".to_string(),
            "Llama 3 8B Instruct".to_string(),
            ModelProvider::Local,
            8192,
        );

        assert!(model.is_ok());
        let m = model.unwrap();
        assert_eq!(m.id, "llama3:8b");
        assert!(m.edge_compatible);
        assert!(m.is_available);
    }

    #[test]
    fn test_create_model_info_validation() {
        let empty_id = ModelInfo::new(
            "".to_string(),
            "Test".to_string(),
            ModelProvider::Local,
            8192,
        );
        assert!(empty_id.is_err());

        let empty_name = ModelInfo::new(
            "test".to_string(),
            "".to_string(),
            ModelProvider::Local,
            8192,
        );
        assert!(empty_name.is_err());

        let zero_context = ModelInfo::new(
            "test".to_string(),
            "Test".to_string(),
            ModelProvider::Local,
            0,
        );
        assert!(zero_context.is_err());
    }
}
