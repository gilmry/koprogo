// MCP Domain Services - Business logic orchestration
use super::entities::*;
use uuid::Uuid;

/// Domain service for MCP request validation and routing
pub struct McpRequestService;

impl McpRequestService {
    /// Determine optimal execution strategy for a request
    pub fn determine_execution_strategy(request: &McpRequest) -> ExecutionStrategy {
        // Edge strategy: local models on Raspberry Pi
        if request.should_use_edge() {
            return ExecutionStrategy::Edge;
        }

        // Grid strategy: heavy tasks (OCR, batch processing)
        let estimated_tokens = request.estimate_input_tokens();
        if estimated_tokens > 10_000 {
            return ExecutionStrategy::Grid;
        }

        // Cloud strategy: API models (Claude, GPT-4)
        ExecutionStrategy::Cloud
    }

    /// Validate request against model capabilities
    pub fn validate_request_for_model(
        request: &McpRequest,
        model: &ModelInfo,
    ) -> Result<(), String> {
        if !model.is_available {
            return Err(format!("Model '{}' is not available", model.id));
        }

        let estimated_tokens = request.estimate_input_tokens();
        if estimated_tokens > model.context_length as usize {
            return Err(format!(
                "Request tokens ({}) exceed model context length ({})",
                estimated_tokens, model.context_length
            ));
        }

        if request.stream && !model.supports_streaming {
            return Err(format!("Model '{}' does not support streaming", model.id));
        }

        Ok(())
    }

    /// Calculate priority score for request queuing
    pub fn calculate_priority(request: &McpRequest) -> u8 {
        let mut priority = 50; // Base priority

        // Higher priority for smaller requests
        let tokens = request.estimate_input_tokens();
        if tokens < 500 {
            priority += 20;
        } else if tokens > 5000 {
            priority -= 10;
        }

        // Higher priority for edge requests (local = faster)
        if request.should_use_edge() {
            priority += 15;
        }

        // Context-aware priority (copro context = higher)
        if request.context.is_some() {
            priority += 10;
        }

        priority.min(100)
    }
}

/// Execution strategy for MCP requests
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionStrategy {
    Edge,  // Local Raspberry Pi execution
    Cloud, // External API call
    Grid,  // Distributed grid computing
}

/// Domain service for carbon footprint calculation
pub struct CarbonFootprintService;

impl CarbonFootprintService {
    /// Calculate CO2 savings by using edge vs cloud
    /// Returns (cloud_co2_grams, edge_co2_grams, savings_grams)
    pub fn calculate_savings(total_tokens: u32) -> (f64, f64, f64) {
        let cloud_co2 = (total_tokens as f64 / 1000.0) * 0.3;
        let edge_co2 = 0.0; // Edge is carbon-neutral (solar-powered Pi)
        let savings = cloud_co2 - edge_co2;

        (cloud_co2, edge_co2, savings)
    }

    /// Estimate monthly CO2 savings for a copro
    pub fn estimate_monthly_savings(avg_requests_per_day: u32, avg_tokens_per_request: u32) -> f64 {
        let total_tokens_per_month = avg_requests_per_day * avg_tokens_per_request * 30;
        let (_, _, savings_per_token) = Self::calculate_savings(avg_tokens_per_request);
        savings_per_token * (total_tokens_per_month as f64 / avg_tokens_per_request as f64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_determine_execution_strategy_edge() {
        let request = McpRequest::new(
            "llama3:8b".to_string(),
            vec![Message::user("Hello".to_string())],
            None,
        ).unwrap();

        assert_eq!(
            McpRequestService::determine_execution_strategy(&request),
            ExecutionStrategy::Edge
        );
    }

    #[test]
    fn test_determine_execution_strategy_cloud() {
        let request = McpRequest::new(
            "claude-3-opus".to_string(),
            vec![Message::user("Hello".to_string())],
            None,
        ).unwrap();

        assert_eq!(
            McpRequestService::determine_execution_strategy(&request),
            ExecutionStrategy::Cloud
        );
    }

    #[test]
    fn test_determine_execution_strategy_grid() {
        // Create a large message to exceed 10k tokens
        let large_content = "word ".repeat(10_000); // ~50k chars = ~12.5k tokens
        let request = McpRequest::new(
            "claude-3-opus".to_string(),
            vec![Message::user(large_content)],
            None,
        ).unwrap();

        assert_eq!(
            McpRequestService::determine_execution_strategy(&request),
            ExecutionStrategy::Grid
        );
    }

    #[test]
    fn test_validate_request_for_model_success() {
        let request = McpRequest::new(
            "llama3:8b".to_string(),
            vec![Message::user("Hello".to_string())],
            None,
        ).unwrap();

        let model = ModelInfo::new(
            "llama3:8b".to_string(),
            "Llama 3 8B".to_string(),
            ModelProvider::Local,
            8192,
        ).unwrap();

        assert!(McpRequestService::validate_request_for_model(&request, &model).is_ok());
    }

    #[test]
    fn test_validate_request_model_unavailable() {
        let request = McpRequest::new(
            "llama3:8b".to_string(),
            vec![Message::user("Hello".to_string())],
            None,
        ).unwrap();

        let mut model = ModelInfo::new(
            "llama3:8b".to_string(),
            "Llama 3 8B".to_string(),
            ModelProvider::Local,
            8192,
        ).unwrap();
        model.is_available = false;

        let result = McpRequestService::validate_request_for_model(&request, &model);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not available"));
    }

    #[test]
    fn test_validate_request_exceeds_context_length() {
        let large_content = "word ".repeat(5000);
        let request = McpRequest::new(
            "llama3:8b".to_string(),
            vec![Message::user(large_content)],
            None,
        ).unwrap();

        let model = ModelInfo::new(
            "llama3:8b".to_string(),
            "Llama 3 8B".to_string(),
            ModelProvider::Local,
            1000, // Small context
        ).unwrap();

        let result = McpRequestService::validate_request_for_model(&request, &model);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("exceed model context length"));
    }

    #[test]
    fn test_validate_request_streaming_not_supported() {
        let request = McpRequest::new(
            "llama3:8b".to_string(),
            vec![Message::user("Hello".to_string())],
            None,
        ).unwrap()
        .with_stream(true);

        let mut model = ModelInfo::new(
            "llama3:8b".to_string(),
            "Llama 3 8B".to_string(),
            ModelProvider::Local,
            8192,
        ).unwrap();
        model.supports_streaming = false;

        let result = McpRequestService::validate_request_for_model(&request, &model);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("does not support streaming"));
    }

    #[test]
    fn test_calculate_priority_small_request() {
        let request = McpRequest::new(
            "llama3:8b".to_string(),
            vec![Message::user("Hi".to_string())], // Small message
            Some("copro:123".to_string()),
        ).unwrap();

        let priority = McpRequestService::calculate_priority(&request);
        // Base(50) + small(20) + edge(15) + context(10) = 95
        assert_eq!(priority, 95);
    }

    #[test]
    fn test_calculate_priority_large_cloud_request() {
        let large_content = "word ".repeat(2000);
        let request = McpRequest::new(
            "claude-3-opus".to_string(),
            vec![Message::user(large_content)],
            None,
        ).unwrap();

        let priority = McpRequestService::calculate_priority(&request);
        // Base(50) - large(10) = 40
        assert_eq!(priority, 40);
    }

    #[test]
    fn test_calculate_priority_capped_at_100() {
        let request = McpRequest::new(
            "llama3:8b".to_string(),
            vec![Message::user("Hi".to_string())],
            Some("copro:123".to_string()),
        ).unwrap();

        let priority = McpRequestService::calculate_priority(&request);
        assert!(priority <= 100);
    }

    #[test]
    fn test_calculate_savings() {
        let (cloud, edge, savings) = CarbonFootprintService::calculate_savings(1000);
        assert_eq!(cloud, 0.3); // 1000/1000 * 0.3 = 0.3g
        assert_eq!(edge, 0.0);
        assert_eq!(savings, 0.3);
    }

    #[test]
    fn test_estimate_monthly_savings() {
        // 100 requests/day, 500 tokens/request, 30 days
        let savings = CarbonFootprintService::estimate_monthly_savings(100, 500);
        // 100 * 30 = 3000 requests/month
        // 3000 * (500/1000 * 0.3) = 3000 * 0.15 = 450g CO2 saved
        assert_eq!(savings, 450.0);
    }
}
