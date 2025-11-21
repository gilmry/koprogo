// Model Registry Port - Interface for managing available AI models
use crate::core::*;
use async_trait::async_trait;

/// Port for model registry - manages available models
#[async_trait]
pub trait ModelRegistry: Send + Sync {
    /// List all available models
    async fn list_models(&self) -> Result<Vec<ModelInfo>, String>;

    /// Get model by ID
    async fn get_model(&self, model_id: &str) -> Result<ModelInfo, String>;

    /// Register a new model
    async fn register_model(&self, model: ModelInfo) -> Result<ModelInfo, String>;

    /// Update model availability status
    async fn update_availability(&self, model_id: &str, is_available: bool) -> Result<(), String>;

    /// Check if model is available for edge execution
    async fn is_edge_compatible(&self, model_id: &str) -> Result<bool, String>;

    /// Get models by provider
    async fn get_models_by_provider(&self, provider: ModelProvider) -> Result<Vec<ModelInfo>, String>;
}
