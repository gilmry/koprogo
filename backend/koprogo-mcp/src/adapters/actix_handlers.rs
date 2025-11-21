// Actix-web handlers for MCP API endpoints
use crate::core::*;
use crate::ports::*;
use actix_web::{web, HttpResponse, Result as ActixResult};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

/// Chat completion request (API contract)
#[derive(Debug, Deserialize)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(default)]
    pub stream: bool,
}

/// Chat completion response (API contract)
#[derive(Debug, Serialize)]
pub struct ChatResponse {
    pub id: Uuid,
    pub model: String,
    pub content: String,
    pub finish_reason: String,
    pub usage: ApiTokenUsage,
    pub execution_info: ApiExecutionInfo,
}

#[derive(Debug, Serialize)]
pub struct ApiTokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Debug, Serialize)]
pub struct ApiExecutionInfo {
    pub execution_type: String,
    pub node_id: Option<String>,
    pub latency_ms: u64,
    pub co2_grams: f64,
}

/// Models list response
#[derive(Debug, Serialize)]
pub struct ModelsResponse {
    pub models: Vec<ModelDto>,
}

#[derive(Debug, Serialize)]
pub struct ModelDto {
    pub id: String,
    pub name: String,
    pub provider: String,
    pub context_length: u32,
    pub is_available: bool,
    pub edge_compatible: bool,
}

/// Task execution request
#[derive(Debug, Deserialize)]
pub struct ExecuteTaskRequest {
    pub task_type: TaskType,
    pub input_data: serde_json::Value,
    pub copro_id: Option<Uuid>,
}

/// Task response
#[derive(Debug, Serialize)]
pub struct TaskResponse {
    pub id: Uuid,
    pub status: String,
    pub result: Option<serde_json::Value>,
}

/// Statistics response
#[derive(Debug, Serialize)]
pub struct StatisticsResponse {
    pub total_requests: u64,
    pub total_tokens: u64,
    pub total_co2_grams: f64,
    pub co2_saved_grams: f64,
    pub edge_requests: u64,
    pub cloud_requests: u64,
    pub grid_requests: u64,
    pub avg_latency_ms: f64,
    pub models_used: Vec<String>,
}

/// POST /mcp/v1/chat - Chat completion endpoint
pub async fn chat_handler(
    mcp_service: web::Data<Arc<dyn McpService>>,
    repository: web::Data<Arc<dyn McpRepository>>,
    request: web::Json<ChatRequest>,
) -> ActixResult<HttpResponse> {
    // Build MCP request
    let mut mcp_request = McpRequest::new(
        request.model.clone(),
        request.messages.clone(),
        request.context.clone(),
    )
    .map_err(|e| actix_web::error::ErrorBadRequest(e))?;

    if let Some(max_tokens) = request.max_tokens {
        mcp_request = mcp_request
            .with_max_tokens(max_tokens)
            .map_err(|e| actix_web::error::ErrorBadRequest(e))?;
    }

    if let Some(temperature) = request.temperature {
        mcp_request = mcp_request
            .with_temperature(temperature)
            .map_err(|e| actix_web::error::ErrorBadRequest(e))?;
    }

    mcp_request = mcp_request.with_stream(request.stream);

    // Log request
    repository
        .log_request(&mcp_request)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    // Execute request
    let response = mcp_service
        .chat(mcp_request)
        .await
        .map_err(|e| match e {
            McpError::ModelNotFound(msg) => actix_web::error::ErrorNotFound(msg),
            McpError::InvalidRequest(msg) => actix_web::error::ErrorBadRequest(msg),
            McpError::RateLimitExceeded => actix_web::error::ErrorTooManyRequests("Rate limit exceeded"),
            _ => actix_web::error::ErrorInternalServerError(e.to_string()),
        })?;

    // Log response
    repository
        .log_response(&response)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    // Convert to API response
    let api_response = ChatResponse {
        id: response.id,
        model: response.model.clone(),
        content: response.content.clone(),
        finish_reason: format!("{:?}", response.finish_reason).to_lowercase(),
        usage: ApiTokenUsage {
            prompt_tokens: response.usage.prompt_tokens,
            completion_tokens: response.usage.completion_tokens,
            total_tokens: response.usage.total_tokens,
        },
        execution_info: ApiExecutionInfo {
            execution_type: format!("{:?}", response.execution_info.execution_type).to_lowercase(),
            node_id: response.execution_info.node_id.clone(),
            latency_ms: response.execution_info.latency_ms,
            co2_grams: response.calculate_co2_grams(),
        },
    };

    Ok(HttpResponse::Ok().json(api_response))
}

/// GET /mcp/v1/models - List available models
pub async fn list_models_handler(
    registry: web::Data<Arc<dyn ModelRegistry>>,
) -> ActixResult<HttpResponse> {
    let models = registry
        .list_models()
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    let model_dtos: Vec<ModelDto> = models
        .into_iter()
        .map(|m| ModelDto {
            id: m.id,
            name: m.name,
            provider: format!("{:?}", m.provider).to_lowercase(),
            context_length: m.context_length,
            is_available: m.is_available,
            edge_compatible: m.edge_compatible,
        })
        .collect();

    Ok(HttpResponse::Ok().json(ModelsResponse { models: model_dtos }))
}

/// POST /mcp/v1/execute - Execute grid task
pub async fn execute_task_handler(
    mcp_service: web::Data<Arc<dyn McpService>>,
    request: web::Json<ExecuteTaskRequest>,
) -> ActixResult<HttpResponse> {
    let task = McpTask {
        id: Uuid::new_v4(),
        task_type: request.task_type,
        input_data: request.input_data.clone(),
        copro_id: request.copro_id,
        status: TaskStatus::Pending,
        result: None,
        assigned_node: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let executed_task = mcp_service
        .execute_task(task)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    Ok(HttpResponse::Ok().json(TaskResponse {
        id: executed_task.id,
        status: format!("{:?}", executed_task.status).to_lowercase(),
        result: executed_task.result,
    }))
}

/// GET /mcp/v1/tasks/{id} - Get task status
pub async fn get_task_handler(
    mcp_service: web::Data<Arc<dyn McpService>>,
    task_id: web::Path<Uuid>,
) -> ActixResult<HttpResponse> {
    let status = mcp_service
        .get_task_status(*task_id)
        .await
        .map_err(|e| match e {
            McpError::TaskNotFound(id) => actix_web::error::ErrorNotFound(format!("Task {} not found", id)),
            _ => actix_web::error::ErrorInternalServerError(e.to_string()),
        })?;

    Ok(HttpResponse::Ok().json(TaskResponse {
        id: *task_id,
        status: format!("{:?}", status).to_lowercase(),
        result: None,
    }))
}

/// GET /mcp/v1/stats - Get MCP statistics
pub async fn stats_handler(
    repository: web::Data<Arc<dyn McpRepository>>,
    query: web::Query<StatsQuery>,
) -> ActixResult<HttpResponse> {
    let start_date = query
        .start_date
        .unwrap_or_else(|| chrono::Utc::now() - chrono::Duration::days(30));
    let end_date = query.end_date.unwrap_or_else(chrono::Utc::now);

    let stats = repository
        .get_statistics(start_date, end_date)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    // Calculate CO2 savings (edge vs cloud)
    let co2_saved = (stats.edge_requests as f64 / 1000.0) * 0.3 * (stats.total_tokens as f64);

    Ok(HttpResponse::Ok().json(StatisticsResponse {
        total_requests: stats.total_requests,
        total_tokens: stats.total_tokens,
        total_co2_grams: stats.total_co2_grams,
        co2_saved_grams: co2_saved,
        edge_requests: stats.edge_requests,
        cloud_requests: stats.cloud_requests,
        grid_requests: stats.grid_requests,
        avg_latency_ms: stats.avg_latency_ms,
        models_used: stats.models_used,
    }))
}

#[derive(Debug, Deserialize)]
pub struct StatsQuery {
    pub start_date: Option<chrono::DateTime<chrono::Utc>>,
    pub end_date: Option<chrono::DateTime<chrono::Utc>>,
}

/// GET /mcp/v1/history - Get request history
pub async fn history_handler(
    repository: web::Data<Arc<dyn McpRepository>>,
    query: web::Query<HistoryQuery>,
) -> ActixResult<HttpResponse> {
    let limit = query.limit.unwrap_or(50).min(200);

    let history = if let Some(user_id) = query.user_id {
        repository
            .get_user_history(user_id, limit)
            .await
            .map_err(|e| actix_web::error::ErrorInternalServerError(e))?
    } else if let Some(ref context) = query.context {
        repository
            .get_context_history(context, limit)
            .await
            .map_err(|e| actix_web::error::ErrorInternalServerError(e))?
    } else {
        return Err(actix_web::error::ErrorBadRequest(
            "Either user_id or context must be provided",
        ));
    };

    Ok(HttpResponse::Ok().json(history))
}

#[derive(Debug, Deserialize)]
pub struct HistoryQuery {
    pub user_id: Option<Uuid>,
    pub context: Option<String>,
    pub limit: Option<usize>,
}

/// GET /mcp/v1/health - Health check
pub async fn health_handler(
    mcp_service: web::Data<Arc<dyn McpService>>,
) -> ActixResult<HttpResponse> {
    let health = mcp_service
        .health_check()
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "is_healthy": health.is_healthy,
        "edge_nodes_available": health.edge_nodes_available,
        "active_tasks": health.active_tasks,
        "avg_latency_ms": health.avg_latency_ms,
    })))
}
