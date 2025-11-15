// Model manager - Handles loading and managing AI models
use crate::mcp_edge::McpEdge;
use anyhow::Result;
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::{error, info};

/// Manages AI models on the edge node
pub struct ModelManager {
    models_dir: PathBuf,
    loaded_models: HashMap<String, McpEdge>,
}

impl ModelManager {
    pub fn new(models_dir: String) -> Self {
        Self {
            models_dir: PathBuf::from(models_dir),
            loaded_models: HashMap::new(),
        }
    }

    /// Load a model from disk
    pub async fn load_model(&mut self, model_name: &str) -> Result<()> {
        if self.loaded_models.contains_key(model_name) {
            info!("Model {} already loaded", model_name);
            return Ok(());
        }

        let model_path = self.resolve_model_path(model_name)?;

        info!("Loading model: {} from {:?}", model_name, model_path);

        let mcp_edge = McpEdge::new(model_path.to_string_lossy().to_string());
        mcp_edge.load_model().await?;

        self.loaded_models.insert(model_name.to_string(), mcp_edge);

        info!("✅ Model {} loaded successfully", model_name);
        Ok(())
    }

    /// Unload a model from memory
    pub async fn unload_model(&mut self, model_name: &str) -> Result<()> {
        if let Some(model) = self.loaded_models.remove(model_name) {
            model.unload_model().await?;
            info!("Model {} unloaded", model_name);
        }

        Ok(())
    }

    /// Check if a model is loaded
    pub fn is_model_loaded(&self, model_name: &str) -> bool {
        self.loaded_models.contains_key(model_name)
    }

    /// List all loaded models
    pub fn list_models(&self) -> Vec<String> {
        self.loaded_models.keys().cloned().collect()
    }

    /// Run inference on a model
    pub async fn infer(
        &self,
        model_name: &str,
        prompt: &str,
        max_tokens: Option<u32>,
        temperature: Option<f32>,
    ) -> Result<String> {
        let model = self
            .loaded_models
            .get(model_name)
            .ok_or_else(|| anyhow::anyhow!("Model not loaded: {}", model_name))?;

        model.infer(prompt, max_tokens, temperature).await
    }

    /// Resolve model name to file path
    fn resolve_model_path(&self, model_name: &str) -> Result<PathBuf> {
        // Map model names to GGUF files
        let filename = match model_name {
            "llama3:8b" | "llama3:8b-instruct-q4" => "llama3-8b-instruct-q4.gguf",
            "mistral:7b" | "mistral:7b-instruct-q4" => "mistral-7b-instruct-q4.gguf",
            "phi-2:2.7b" | "phi-2:2.7b-q4" => "phi-2-2.7b-q4.gguf",
            _ => {
                // Try to use model_name directly as filename
                if model_name.ends_with(".gguf") {
                    model_name
                } else {
                    return Err(anyhow::anyhow!("Unknown model: {}", model_name));
                }
            }
        };

        let path = self.models_dir.join(filename);

        // Check if file exists
        if !path.exists() {
            error!("Model file not found: {:?}", path);
            error!("Available files in {:?}:", self.models_dir);

            if let Ok(entries) = std::fs::read_dir(&self.models_dir) {
                for entry in entries.flatten() {
                    error!("  - {:?}", entry.file_name());
                }
            }

            return Err(anyhow::anyhow!(
                "Model file not found: {:?}. Please download the model to the models directory.",
                path
            ));
        }

        Ok(path)
    }
}

/*
 * MODEL DOWNLOAD INSTRUCTIONS:
 *
 * To use real models, download GGUF files to the models/ directory:
 *
 * 1. Llama 3 8B Instruct Q4 (~4.5GB):
 *    wget -P models/ https://huggingface.co/QuantFactory/Meta-Llama-3-8B-Instruct-GGUF/resolve/main/Meta-Llama-3-8B-Instruct.Q4_K_M.gguf
 *    mv models/Meta-Llama-3-8B-Instruct.Q4_K_M.gguf models/llama3-8b-instruct-q4.gguf
 *
 * 2. Mistral 7B Instruct Q4 (~4GB):
 *    wget -P models/ https://huggingface.co/TheBloke/Mistral-7B-Instruct-v0.2-GGUF/resolve/main/mistral-7b-instruct-v0.2.Q4_K_M.gguf
 *    mv models/mistral-7b-instruct-v0.2.Q4_K_M.gguf models/mistral-7b-instruct-q4.gguf
 *
 * 3. Phi-2 2.7B Q4 (~1.6GB) - Best for Raspberry Pi 4GB:
 *    wget -P models/ https://huggingface.co/TheBloke/phi-2-GGUF/resolve/main/phi-2.Q4_K_M.gguf
 *    mv models/phi-2.Q4_K_M.gguf models/phi-2-2.7b-q4.gguf
 *
 * Note: These models require significant RAM:
 * - Raspberry Pi 4/5 8GB: Can run 8B models
 * - Raspberry Pi 4 4GB: Limited to 2-3B models
 * - For production, use mmap to reduce RAM usage
 */
