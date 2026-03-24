use crate::application::ports::contract_evaluation_repository::ContractEvaluationRepository;
use crate::domain::entities::contract_evaluation::ContractEvaluation;
use crate::infrastructure::database::pool::DbPool;
use async_trait::async_trait;
use sqlx::Row;
use std::collections::HashMap;
use uuid::Uuid;

pub struct PostgresContractEvaluationRepository {
    pool: DbPool,
}

impl PostgresContractEvaluationRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ContractEvaluationRepository for PostgresContractEvaluationRepository {
    async fn create(&self, evaluation: &ContractEvaluation) -> Result<ContractEvaluation, String> {
        sqlx::query(
            r#"
            INSERT INTO contract_evaluations (
                id, organization_id, service_provider_id, quote_id, ticket_id,
                evaluator_id, building_id, criteria, global_score, comments,
                would_recommend, is_legal_evaluation, is_anonymous, created_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            "#,
        )
        .bind(evaluation.id)
        .bind(evaluation.organization_id)
        .bind(evaluation.service_provider_id)
        .bind(evaluation.quote_id)
        .bind(evaluation.ticket_id)
        .bind(evaluation.evaluator_id)
        .bind(evaluation.building_id)
        .bind(
            serde_json::to_value(&evaluation.criteria)
                .map_err(|e| format!("Failed to serialize criteria: {}", e))?,
        )
        .bind(evaluation.global_score)
        .bind(&evaluation.comments)
        .bind(evaluation.would_recommend)
        .bind(evaluation.is_legal_evaluation)
        .bind(evaluation.is_anonymous)
        .bind(evaluation.created_at)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Database error creating contract evaluation: {}", e))?;

        Ok(evaluation.clone())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<ContractEvaluation>, String> {
        let row = sqlx::query(
            r#"
            SELECT id, organization_id, service_provider_id, quote_id, ticket_id,
                   evaluator_id, building_id, criteria, global_score, comments,
                   would_recommend, is_legal_evaluation, is_anonymous, created_at
            FROM contract_evaluations
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(row.map(|row| ContractEvaluation {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            service_provider_id: row.get("service_provider_id"),
            quote_id: row.get("quote_id"),
            ticket_id: row.get("ticket_id"),
            evaluator_id: row.get("evaluator_id"),
            building_id: row.get("building_id"),
            criteria: row
                .get::<serde_json::Value, _>("criteria")
                .as_object()
                .map(|obj| {
                    obj.iter()
                        .map(|(k, v)| (k.clone(), v.as_u64().unwrap_or(0) as u8))
                        .collect::<HashMap<String, u8>>()
                })
                .unwrap_or_default(),
            global_score: row.get("global_score"),
            comments: row.get("comments"),
            would_recommend: row.get("would_recommend"),
            is_legal_evaluation: row.get("is_legal_evaluation"),
            is_anonymous: row.get("is_anonymous"),
            created_at: row.get("created_at"),
        }))
    }

    async fn find_by_service_provider(
        &self,
        provider_id: Uuid,
        page: i64,
        per_page: i64,
    ) -> Result<Vec<ContractEvaluation>, String> {
        if page < 1 || per_page < 1 {
            return Err("Page and per_page must be >= 1".to_string());
        }

        let offset = (page - 1) * per_page;

        let rows = sqlx::query(
            r#"
            SELECT id, organization_id, service_provider_id, quote_id, ticket_id,
                   evaluator_id, building_id, criteria, global_score, comments,
                   would_recommend, is_legal_evaluation, is_anonymous, created_at
            FROM contract_evaluations
            WHERE service_provider_id = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(provider_id)
        .bind(per_page)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(rows
            .iter()
            .map(|row| ContractEvaluation {
                id: row.get("id"),
                organization_id: row.get("organization_id"),
                service_provider_id: row.get("service_provider_id"),
                quote_id: row.get("quote_id"),
                ticket_id: row.get("ticket_id"),
                evaluator_id: row.get("evaluator_id"),
                building_id: row.get("building_id"),
                criteria: row
                    .get::<serde_json::Value, _>("criteria")
                    .as_object()
                    .map(|obj| {
                        obj.iter()
                            .map(|(k, v)| (k.clone(), v.as_u64().unwrap_or(0) as u8))
                            .collect::<HashMap<String, u8>>()
                    })
                    .unwrap_or_default(),
                global_score: row.get("global_score"),
                comments: row.get("comments"),
                would_recommend: row.get("would_recommend"),
                is_legal_evaluation: row.get("is_legal_evaluation"),
                is_anonymous: row.get("is_anonymous"),
                created_at: row.get("created_at"),
            })
            .collect())
    }

    async fn find_by_quote(&self, quote_id: Uuid) -> Result<Vec<ContractEvaluation>, String> {
        let rows = sqlx::query(
            r#"
            SELECT id, organization_id, service_provider_id, quote_id, ticket_id,
                   evaluator_id, building_id, criteria, global_score, comments,
                   would_recommend, is_legal_evaluation, is_anonymous, created_at
            FROM contract_evaluations
            WHERE quote_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(quote_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(rows
            .iter()
            .map(|row| ContractEvaluation {
                id: row.get("id"),
                organization_id: row.get("organization_id"),
                service_provider_id: row.get("service_provider_id"),
                quote_id: row.get("quote_id"),
                ticket_id: row.get("ticket_id"),
                evaluator_id: row.get("evaluator_id"),
                building_id: row.get("building_id"),
                criteria: row
                    .get::<serde_json::Value, _>("criteria")
                    .as_object()
                    .map(|obj| {
                        obj.iter()
                            .map(|(k, v)| (k.clone(), v.as_u64().unwrap_or(0) as u8))
                            .collect::<HashMap<String, u8>>()
                    })
                    .unwrap_or_default(),
                global_score: row.get("global_score"),
                comments: row.get("comments"),
                would_recommend: row.get("would_recommend"),
                is_legal_evaluation: row.get("is_legal_evaluation"),
                is_anonymous: row.get("is_anonymous"),
                created_at: row.get("created_at"),
            })
            .collect())
    }

    async fn find_by_ticket(&self, ticket_id: Uuid) -> Result<Vec<ContractEvaluation>, String> {
        let rows = sqlx::query(
            r#"
            SELECT id, organization_id, service_provider_id, quote_id, ticket_id,
                   evaluator_id, building_id, criteria, global_score, comments,
                   would_recommend, is_legal_evaluation, is_anonymous, created_at
            FROM contract_evaluations
            WHERE ticket_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(ticket_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(rows
            .iter()
            .map(|row| ContractEvaluation {
                id: row.get("id"),
                organization_id: row.get("organization_id"),
                service_provider_id: row.get("service_provider_id"),
                quote_id: row.get("quote_id"),
                ticket_id: row.get("ticket_id"),
                evaluator_id: row.get("evaluator_id"),
                building_id: row.get("building_id"),
                criteria: row
                    .get::<serde_json::Value, _>("criteria")
                    .as_object()
                    .map(|obj| {
                        obj.iter()
                            .map(|(k, v)| (k.clone(), v.as_u64().unwrap_or(0) as u8))
                            .collect::<HashMap<String, u8>>()
                    })
                    .unwrap_or_default(),
                global_score: row.get("global_score"),
                comments: row.get("comments"),
                would_recommend: row.get("would_recommend"),
                is_legal_evaluation: row.get("is_legal_evaluation"),
                is_anonymous: row.get("is_anonymous"),
                created_at: row.get("created_at"),
            })
            .collect())
    }

    async fn find_by_building(
        &self,
        building_id: Uuid,
        page: i64,
        per_page: i64,
    ) -> Result<Vec<ContractEvaluation>, String> {
        if page < 1 || per_page < 1 {
            return Err("Page and per_page must be >= 1".to_string());
        }

        let offset = (page - 1) * per_page;

        let rows = sqlx::query(
            r#"
            SELECT id, organization_id, service_provider_id, quote_id, ticket_id,
                   evaluator_id, building_id, criteria, global_score, comments,
                   would_recommend, is_legal_evaluation, is_anonymous, created_at
            FROM contract_evaluations
            WHERE building_id = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(building_id)
        .bind(per_page)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(rows
            .iter()
            .map(|row| ContractEvaluation {
                id: row.get("id"),
                organization_id: row.get("organization_id"),
                service_provider_id: row.get("service_provider_id"),
                quote_id: row.get("quote_id"),
                ticket_id: row.get("ticket_id"),
                evaluator_id: row.get("evaluator_id"),
                building_id: row.get("building_id"),
                criteria: row
                    .get::<serde_json::Value, _>("criteria")
                    .as_object()
                    .map(|obj| {
                        obj.iter()
                            .map(|(k, v)| (k.clone(), v.as_u64().unwrap_or(0) as u8))
                            .collect::<HashMap<String, u8>>()
                    })
                    .unwrap_or_default(),
                global_score: row.get("global_score"),
                comments: row.get("comments"),
                would_recommend: row.get("would_recommend"),
                is_legal_evaluation: row.get("is_legal_evaluation"),
                is_anonymous: row.get("is_anonymous"),
                created_at: row.get("created_at"),
            })
            .collect())
    }

    async fn find_legal_evaluations(
        &self,
        building_id: Uuid,
        page: i64,
        per_page: i64,
    ) -> Result<Vec<ContractEvaluation>, String> {
        if page < 1 || per_page < 1 {
            return Err("Page and per_page must be >= 1".to_string());
        }

        let offset = (page - 1) * per_page;

        let rows = sqlx::query(
            r#"
            SELECT id, organization_id, service_provider_id, quote_id, ticket_id,
                   evaluator_id, building_id, criteria, global_score, comments,
                   would_recommend, is_legal_evaluation, is_anonymous, created_at
            FROM contract_evaluations
            WHERE building_id = $1 AND is_legal_evaluation = TRUE
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(building_id)
        .bind(per_page)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(rows
            .iter()
            .map(|row| ContractEvaluation {
                id: row.get("id"),
                organization_id: row.get("organization_id"),
                service_provider_id: row.get("service_provider_id"),
                quote_id: row.get("quote_id"),
                ticket_id: row.get("ticket_id"),
                evaluator_id: row.get("evaluator_id"),
                building_id: row.get("building_id"),
                criteria: row
                    .get::<serde_json::Value, _>("criteria")
                    .as_object()
                    .map(|obj| {
                        obj.iter()
                            .map(|(k, v)| (k.clone(), v.as_u64().unwrap_or(0) as u8))
                            .collect::<HashMap<String, u8>>()
                    })
                    .unwrap_or_default(),
                global_score: row.get("global_score"),
                comments: row.get("comments"),
                would_recommend: row.get("would_recommend"),
                is_legal_evaluation: row.get("is_legal_evaluation"),
                is_anonymous: row.get("is_anonymous"),
                created_at: row.get("created_at"),
            })
            .collect())
    }

    async fn update(&self, evaluation: &ContractEvaluation) -> Result<ContractEvaluation, String> {
        sqlx::query(
            r#"
            UPDATE contract_evaluations
            SET criteria = $1,
                global_score = $2,
                comments = $3,
                would_recommend = $4,
                is_legal_evaluation = $5,
                is_anonymous = $6,
                quote_id = $7,
                ticket_id = $8
            WHERE id = $9
            "#,
        )
        .bind(
            serde_json::to_value(&evaluation.criteria)
                .map_err(|e| format!("Failed to serialize criteria: {}", e))?,
        )
        .bind(evaluation.global_score)
        .bind(&evaluation.comments)
        .bind(evaluation.would_recommend)
        .bind(evaluation.is_legal_evaluation)
        .bind(evaluation.is_anonymous)
        .bind(evaluation.quote_id)
        .bind(evaluation.ticket_id)
        .bind(evaluation.id)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Database error updating contract evaluation: {}", e))?;

        Ok(evaluation.clone())
    }

    async fn delete(&self, id: Uuid) -> Result<(), String> {
        sqlx::query("DELETE FROM contract_evaluations WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("Database error deleting contract evaluation: {}", e))?;

        Ok(())
    }

    async fn count_by_service_provider(&self, provider_id: Uuid) -> Result<i64, String> {
        let row = sqlx::query(
            "SELECT COUNT(*) as count FROM contract_evaluations WHERE service_provider_id = $1",
        )
        .bind(provider_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(row.get::<i64, _>("count"))
    }

    async fn average_score_by_provider(&self, provider_id: Uuid) -> Result<Option<f64>, String> {
        let row = sqlx::query("SELECT AVG(global_score) as avg_score FROM contract_evaluations WHERE service_provider_id = $1")
            .bind(provider_id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| format!("Database error: {}", e))?;

        Ok(row.get::<Option<f64>, _>("avg_score"))
    }
}
