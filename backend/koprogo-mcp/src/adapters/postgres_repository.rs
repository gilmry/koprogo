// PostgreSQL implementation of MCP repository
use crate::core::*;
use crate::ports::*;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{PgPool, Row};
use uuid::Uuid;

pub struct PostgresMcpRepository {
    pool: PgPool,
}

impl PostgresMcpRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl McpRepository for PostgresMcpRepository {
    async fn log_request(&self, request: &McpRequest) -> Result<(), String> {
        let messages_json = serde_json::to_value(&request.messages)
            .map_err(|e| format!("Failed to serialize messages: {}", e))?;

        sqlx::query!(
            r#"
            INSERT INTO mcp_requests (
                id, model, messages, context, max_tokens, temperature,
                stream, user_id, created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
            request.id,
            request.model,
            messages_json,
            request.context,
            request.max_tokens.map(|t| t as i32),
            request.temperature,
            request.stream,
            request.user_id,
            request.created_at,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to log request: {}", e))?;

        Ok(())
    }

    async fn log_response(&self, response: &McpResponse) -> Result<(), String> {
        let exec_info_json = serde_json::to_value(&response.execution_info)
            .map_err(|e| format!("Failed to serialize execution_info: {}", e))?;

        sqlx::query!(
            r#"
            INSERT INTO mcp_responses (
                id, request_id, model, content, finish_reason,
                prompt_tokens, completion_tokens, total_tokens,
                execution_info, created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
            response.id,
            response.request_id,
            response.model,
            response.content,
            serde_json::to_string(&response.finish_reason).unwrap(),
            response.usage.prompt_tokens as i32,
            response.usage.completion_tokens as i32,
            response.usage.total_tokens as i32,
            exec_info_json,
            response.created_at,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to log response: {}", e))?;

        Ok(())
    }

    async fn get_request(&self, request_id: Uuid) -> Result<McpRequest, String> {
        let row = sqlx::query!(
            r#"
            SELECT id, model, messages, context, max_tokens, temperature,
                   stream, user_id, created_at
            FROM mcp_requests
            WHERE id = $1
            "#,
            request_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Request not found: {}", e))?;

        let messages: Vec<Message> = serde_json::from_value(row.messages)
            .map_err(|e| format!("Failed to deserialize messages: {}", e))?;

        Ok(McpRequest {
            id: row.id,
            model: row.model,
            messages,
            context: row.context,
            max_tokens: row.max_tokens.map(|t| t as u32),
            temperature: row.temperature,
            stream: row.stream,
            user_id: row.user_id,
            created_at: row.created_at,
        })
    }

    async fn get_response(&self, request_id: Uuid) -> Result<McpResponse, String> {
        let row = sqlx::query!(
            r#"
            SELECT id, request_id, model, content, finish_reason,
                   prompt_tokens, completion_tokens, total_tokens,
                   execution_info, created_at
            FROM mcp_responses
            WHERE request_id = $1
            "#,
            request_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Response not found: {}", e))?;

        let finish_reason: FinishReason = serde_json::from_str(&row.finish_reason)
            .map_err(|e| format!("Failed to deserialize finish_reason: {}", e))?;

        let execution_info: ExecutionInfo = serde_json::from_value(row.execution_info)
            .map_err(|e| format!("Failed to deserialize execution_info: {}", e))?;

        Ok(McpResponse {
            id: row.id,
            request_id: row.request_id,
            model: row.model,
            content: row.content,
            finish_reason,
            usage: TokenUsage::new(
                row.prompt_tokens as u32,
                row.completion_tokens as u32,
            ),
            execution_info,
            created_at: row.created_at,
        })
    }

    async fn get_user_history(
        &self,
        user_id: Uuid,
        limit: usize,
    ) -> Result<Vec<(McpRequest, Option<McpResponse>)>, String> {
        let rows = sqlx::query!(
            r#"
            SELECT
                r.id, r.model, r.messages, r.context, r.max_tokens, r.temperature,
                r.stream, r.user_id, r.created_at,
                resp.id as resp_id, resp.content, resp.finish_reason,
                resp.prompt_tokens, resp.completion_tokens, resp.total_tokens,
                resp.execution_info, resp.created_at as resp_created_at
            FROM mcp_requests r
            LEFT JOIN mcp_responses resp ON r.id = resp.request_id
            WHERE r.user_id = $1
            ORDER BY r.created_at DESC
            LIMIT $2
            "#,
            user_id,
            limit as i64
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to fetch user history: {}", e))?;

        let mut results = Vec::new();
        for row in rows {
            let messages: Vec<Message> = serde_json::from_value(row.messages)
                .map_err(|e| format!("Failed to deserialize messages: {}", e))?;

            let request = McpRequest {
                id: row.id,
                model: row.model,
                messages,
                context: row.context,
                max_tokens: row.max_tokens.map(|t| t as u32),
                temperature: row.temperature,
                stream: row.stream,
                user_id: row.user_id,
                created_at: row.created_at,
            };

            let response = if let (Some(resp_id), Some(content), Some(finish_reason), Some(exec_info)) =
                (row.resp_id, row.content, row.finish_reason, row.execution_info)
            {
                let finish_reason: FinishReason = serde_json::from_str(&finish_reason)
                    .map_err(|e| format!("Failed to deserialize finish_reason: {}", e))?;

                let execution_info: ExecutionInfo = serde_json::from_value(exec_info)
                    .map_err(|e| format!("Failed to deserialize execution_info: {}", e))?;

                Some(McpResponse {
                    id: resp_id,
                    request_id: row.id,
                    model: request.model.clone(),
                    content,
                    finish_reason,
                    usage: TokenUsage::new(
                        row.prompt_tokens.unwrap() as u32,
                        row.completion_tokens.unwrap() as u32,
                    ),
                    execution_info,
                    created_at: row.resp_created_at.unwrap(),
                })
            } else {
                None
            };

            results.push((request, response));
        }

        Ok(results)
    }

    async fn get_context_history(
        &self,
        context: &str,
        limit: usize,
    ) -> Result<Vec<(McpRequest, Option<McpResponse>)>, String> {
        let rows = sqlx::query!(
            r#"
            SELECT
                r.id, r.model, r.messages, r.context, r.max_tokens, r.temperature,
                r.stream, r.user_id, r.created_at,
                resp.id as resp_id, resp.content, resp.finish_reason,
                resp.prompt_tokens, resp.completion_tokens, resp.total_tokens,
                resp.execution_info, resp.created_at as resp_created_at
            FROM mcp_requests r
            LEFT JOIN mcp_responses resp ON r.id = resp.request_id
            WHERE r.context = $1
            ORDER BY r.created_at DESC
            LIMIT $2
            "#,
            context,
            limit as i64
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to fetch context history: {}", e))?;

        let mut results = Vec::new();
        for row in rows {
            let messages: Vec<Message> = serde_json::from_value(row.messages)
                .map_err(|e| format!("Failed to deserialize messages: {}", e))?;

            let request = McpRequest {
                id: row.id,
                model: row.model,
                messages,
                context: row.context,
                max_tokens: row.max_tokens.map(|t| t as u32),
                temperature: row.temperature,
                stream: row.stream,
                user_id: row.user_id,
                created_at: row.created_at,
            };

            let response = if let (Some(resp_id), Some(content), Some(finish_reason), Some(exec_info)) =
                (row.resp_id, row.content, row.finish_reason, row.execution_info)
            {
                let finish_reason: FinishReason = serde_json::from_str(&finish_reason)
                    .map_err(|e| format!("Failed to deserialize finish_reason: {}", e))?;

                let execution_info: ExecutionInfo = serde_json::from_value(exec_info)
                    .map_err(|e| format!("Failed to deserialize execution_info: {}", e))?;

                Some(McpResponse {
                    id: resp_id,
                    request_id: row.id,
                    model: request.model.clone(),
                    content,
                    finish_reason,
                    usage: TokenUsage::new(
                        row.prompt_tokens.unwrap() as u32,
                        row.completion_tokens.unwrap() as u32,
                    ),
                    execution_info,
                    created_at: row.resp_created_at.unwrap(),
                })
            } else {
                None
            };

            results.push((request, response));
        }

        Ok(results)
    }

    async fn get_statistics(
        &self,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> Result<McpStatistics, String> {
        let row = sqlx::query!(
            r#"
            SELECT
                COUNT(DISTINCT r.id) as total_requests,
                COALESCE(SUM(resp.total_tokens), 0) as total_tokens,
                COUNT(DISTINCT r.model) as models_count
            FROM mcp_requests r
            LEFT JOIN mcp_responses resp ON r.id = resp.request_id
            WHERE r.created_at BETWEEN $1 AND $2
            "#,
            start_date,
            end_date
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to fetch statistics: {}", e))?;

        let exec_stats = sqlx::query!(
            r#"
            SELECT
                execution_info->>'execution_type' as exec_type,
                COUNT(*) as count,
                AVG((execution_info->>'latency_ms')::bigint) as avg_latency
            FROM mcp_responses
            WHERE created_at BETWEEN $1 AND $2
            GROUP BY execution_info->>'execution_type'
            "#,
            start_date,
            end_date
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to fetch execution stats: {}", e))?;

        let mut edge_requests = 0u64;
        let mut cloud_requests = 0u64;
        let mut grid_requests = 0u64;
        let mut total_latency = 0f64;
        let mut latency_count = 0u64;

        for stat in exec_stats {
            let count = stat.count.unwrap_or(0) as u64;
            if let Some(exec_type) = stat.exec_type {
                match exec_type.as_str() {
                    "edge" => edge_requests = count,
                    "cloud" => cloud_requests = count,
                    "grid" => grid_requests = count,
                    _ => {}
                }
            }

            if let Some(Some(latency)) = stat.avg_latency {
                total_latency += latency;
                latency_count += 1;
            }
        }

        let models_used = sqlx::query_scalar!(
            r#"
            SELECT DISTINCT model
            FROM mcp_requests
            WHERE created_at BETWEEN $1 AND $2
            "#,
            start_date,
            end_date
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to fetch models: {}", e))?;

        let total_tokens = row.total_tokens.unwrap_or(0) as u64;
        let total_co2_grams = (cloud_requests as f64 / 1000.0) * 0.3 * (total_tokens as f64);

        Ok(McpStatistics {
            total_requests: row.total_requests.unwrap_or(0) as u64,
            total_tokens,
            total_co2_grams,
            edge_requests,
            cloud_requests,
            grid_requests,
            avg_latency_ms: if latency_count > 0 {
                total_latency / latency_count as f64
            } else {
                0.0
            },
            models_used,
        })
    }
}

/// PostgreSQL implementation of ModelRegistry
pub struct PostgresModelRegistry {
    pool: PgPool,
}

impl PostgresModelRegistry {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ModelRegistry for PostgresModelRegistry {
    async fn list_models(&self) -> Result<Vec<ModelInfo>, String> {
        let rows = sqlx::query!(
            r#"
            SELECT id, name, provider, context_length, is_available,
                   supports_streaming, edge_compatible, created_at
            FROM mcp_models
            ORDER BY name
            "#
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to list models: {}", e))?;

        let mut models = Vec::new();
        for row in rows {
            let provider: ModelProvider = serde_json::from_str(&row.provider)
                .map_err(|e| format!("Failed to deserialize provider: {}", e))?;

            let model = ModelInfo {
                id: row.id,
                name: row.name,
                provider,
                context_length: row.context_length as u32,
                is_available: row.is_available,
                supports_streaming: row.supports_streaming,
                edge_compatible: row.edge_compatible,
                created_at: row.created_at,
            };
            models.push(model);
        }

        Ok(models)
    }

    async fn get_model(&self, model_id: &str) -> Result<ModelInfo, String> {
        let row = sqlx::query!(
            r#"
            SELECT id, name, provider, context_length, is_available,
                   supports_streaming, edge_compatible, created_at
            FROM mcp_models
            WHERE id = $1
            "#,
            model_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Model not found: {}", e))?;

        let provider: ModelProvider = serde_json::from_str(&row.provider)
            .map_err(|e| format!("Failed to deserialize provider: {}", e))?;

        Ok(ModelInfo {
            id: row.id,
            name: row.name,
            provider,
            context_length: row.context_length as u32,
            is_available: row.is_available,
            supports_streaming: row.supports_streaming,
            edge_compatible: row.edge_compatible,
            created_at: row.created_at,
        })
    }

    async fn register_model(&self, model: ModelInfo) -> Result<ModelInfo, String> {
        let provider_json = serde_json::to_string(&model.provider)
            .map_err(|e| format!("Failed to serialize provider: {}", e))?;

        sqlx::query!(
            r#"
            INSERT INTO mcp_models (
                id, name, provider, context_length, is_available,
                supports_streaming, edge_compatible, created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
            model.id,
            model.name,
            provider_json,
            model.context_length as i32,
            model.is_available,
            model.supports_streaming,
            model.edge_compatible,
            model.created_at,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to register model: {}", e))?;

        Ok(model)
    }

    async fn update_availability(&self, model_id: &str, is_available: bool) -> Result<(), String> {
        sqlx::query!(
            r#"
            UPDATE mcp_models
            SET is_available = $1
            WHERE id = $2
            "#,
            is_available,
            model_id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to update availability: {}", e))?;

        Ok(())
    }

    async fn is_edge_compatible(&self, model_id: &str) -> Result<bool, String> {
        let row = sqlx::query!(
            r#"
            SELECT edge_compatible
            FROM mcp_models
            WHERE id = $1
            "#,
            model_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Model not found: {}", e))?;

        Ok(row.edge_compatible)
    }

    async fn get_models_by_provider(&self, provider: ModelProvider) -> Result<Vec<ModelInfo>, String> {
        let provider_json = serde_json::to_string(&provider)
            .map_err(|e| format!("Failed to serialize provider: {}", e))?;

        let rows = sqlx::query!(
            r#"
            SELECT id, name, provider, context_length, is_available,
                   supports_streaming, edge_compatible, created_at
            FROM mcp_models
            WHERE provider = $1
            ORDER BY name
            "#,
            provider_json
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to fetch models by provider: {}", e))?;

        let mut models = Vec::new();
        for row in rows {
            let model = ModelInfo {
                id: row.id,
                name: row.name,
                provider,
                context_length: row.context_length as u32,
                is_available: row.is_available,
                supports_streaming: row.supports_streaming,
                edge_compatible: row.edge_compatible,
                created_at: row.created_at,
            };
            models.push(model);
        }

        Ok(models)
    }
}
