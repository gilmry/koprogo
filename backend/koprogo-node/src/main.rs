// KoproGo Edge Node - Raspberry Pi AI inference server
use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info};
use uuid::Uuid;

mod mcp_edge;
mod grid_client;
mod model_manager;

use mcp_edge::*;
use grid_client::*;
use model_manager::*;

#[derive(Parser)]
#[command(name = "koprogo-node")]
#[command(about = "KoproGo Edge Node - Raspberry Pi AI server", long_about = None)]
struct Cli {
    /// Port to listen on
    #[arg(short, long, default_value = "3031")]
    port: u16,

    /// Model to load (e.g., llama3:8b-instruct-q4)
    #[arg(short, long, default_value = "llama3:8b-instruct-q4")]
    model: String,

    /// Models directory
    #[arg(long, default_value = "./models")]
    models_dir: String,

    /// Grid server URL (optional)
    #[arg(short, long)]
    grid_url: Option<String>,

    /// Enable MCP server
    #[arg(long, default_value = "true")]
    mcp: bool,
}

#[derive(Clone)]
struct AppState {
    model_manager: Arc<RwLock<ModelManager>>,
    grid_client: Option<Arc<GridClient>>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter("koprogo_node=info,tower_http=info")
        .init();

    let cli = Cli::parse();

    info!("🚀 Starting KoproGo Edge Node v{}", env!("CARGO_PKG_VERSION"));
    info!("📂 Models directory: {}", cli.models_dir);
    info!("🤖 Loading model: {}", cli.model);

    // Initialize model manager
    let model_manager = ModelManager::new(cli.models_dir.clone());
    model_manager.load_model(&cli.model).await?;

    info!("✅ Model loaded successfully");

    // Initialize grid client if URL provided
    let grid_client = cli.grid_url.map(|url| {
        info!("🌐 Connecting to grid: {}", url);
        Arc::new(GridClient::new(url))
    });

    let state = AppState {
        model_manager: Arc::new(RwLock::new(model_manager)),
        grid_client,
    };

    // Build router
    let app = Router::new()
        .route("/mcp/v1/chat", post(chat_handler))
        .route("/mcp/v1/models", get(models_handler))
        .route("/mcp/v1/health", get(health_handler))
        .route("/health", get(simple_health))
        .with_state(state)
        .layer(tower_http::trace::TraceLayer::new_for_http())
        .layer(tower_http::cors::CorsLayer::permissive());

    let addr = format!("0.0.0.0:{}", cli.port);
    info!("🎧 Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// POST /mcp/v1/chat - Chat completion endpoint
async fn chat_handler(
    State(state): State<AppState>,
    Json(request): Json<ChatRequest>,
) -> Result<Json<ChatResponse>, AppError> {
    let start = std::time::Instant::now();

    let model_manager = state.model_manager.read().await;

    // Validate model is loaded
    if !model_manager.is_model_loaded(&request.model) {
        return Err(AppError::ModelNotLoaded(request.model));
    }

    // Build prompt from messages
    let prompt = build_prompt(&request.messages);

    // Run inference
    let content = model_manager
        .infer(&request.model, &prompt, request.max_tokens, request.temperature)
        .await?;

    let latency_ms = start.elapsed().as_millis() as u64;

    // Estimate tokens (simple heuristic)
    let prompt_tokens = (prompt.len() / 4) as u32;
    let completion_tokens = (content.len() / 4) as u32;

    info!(
        "✅ Inference completed in {}ms ({} tokens)",
        latency_ms,
        prompt_tokens + completion_tokens
    );

    Ok(Json(ChatResponse {
        model: request.model,
        content,
        usage: TokenUsage {
            prompt_tokens,
            completion_tokens,
        },
    }))
}

/// GET /mcp/v1/models - List loaded models
async fn models_handler(State(state): State<AppState>) -> Json<ModelsResponse> {
    let model_manager = state.model_manager.read().await;
    let models = model_manager.list_models();

    Json(ModelsResponse { models })
}

/// GET /mcp/v1/health - Health check
async fn health_handler(State(state): State<AppState>) -> Json<HealthResponse> {
    let model_manager = state.model_manager.read().await;

    let mut sys = sysinfo::System::new_all();
    sys.refresh_all();

    let total_memory_mb = (sys.total_memory() / 1024 / 1024) as u32;
    let used_memory_mb = (sys.used_memory() / 1024 / 1024) as u32;

    Json(HealthResponse {
        node_url: "http://localhost:3031".to_string(),
        is_healthy: true,
        models_loaded: model_manager.list_models(),
        active_requests: 0, // TODO: track active requests
        total_memory_mb,
        used_memory_mb,
    })
}

/// GET /health - Simple health check
async fn simple_health() -> &'static str {
    "OK"
}

/// Build prompt from messages (simple chat template)
fn build_prompt(messages: &[Message]) -> String {
    let mut prompt = String::new();

    for message in messages {
        match message.role.as_str() {
            "system" => prompt.push_str(&format!("System: {}\n\n", message.content)),
            "user" => prompt.push_str(&format!("User: {}\n\n", message.content)),
            "assistant" => prompt.push_str(&format!("Assistant: {}\n\n", message.content)),
            _ => {}
        }
    }

    prompt.push_str("Assistant: ");
    prompt
}

// API types

#[derive(Debug, Deserialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
    max_tokens: Option<u32>,
    temperature: Option<f32>,
}

#[derive(Debug, Deserialize, Clone)]
struct Message {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct ChatResponse {
    model: String,
    content: String,
    usage: TokenUsage,
}

#[derive(Debug, Serialize)]
struct TokenUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
}

#[derive(Debug, Serialize)]
struct ModelsResponse {
    models: Vec<String>,
}

#[derive(Debug, Serialize)]
struct HealthResponse {
    node_url: String,
    is_healthy: bool,
    models_loaded: Vec<String>,
    active_requests: u32,
    total_memory_mb: u32,
    used_memory_mb: u32,
}

// Error handling

#[derive(Debug, thiserror::Error)]
enum AppError {
    #[error("Model not loaded: {0}")]
    ModelNotLoaded(String),

    #[error("Inference failed: {0}")]
    InferenceFailed(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::ModelNotLoaded(msg) => (StatusCode::NOT_FOUND, msg),
            AppError::InferenceFailed(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            AppError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };

        let body = serde_json::json!({
            "error": error_message
        });

        (status, Json(body)).into_response()
    }
}
