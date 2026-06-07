//! PostgreSQL implementation of [`ContractorEvaluationRepository`] (Story
//! 3.9 — FR34 FR35 INV-21 INV-24).
//!
//! All `sqlx::Error` paths are wrapped in `AppError::Database(_)` — no
//! `Result<_, String>` debt (CRITICAL.md #4 / #555).
//!
//! Evaluations are append-only: only [`save`] writes new rows. The DB
//! trigger `contractor_eval_no_mutation` (cf. migration `20260605070000`)
//! blocks any UPDATE / DELETE at the SQL boundary, so a misbehaving caller
//! cannot tamper with the audit trail.

use crate::application::error::AppError;
use crate::application::ports::ContractorEvaluationRepository;
use crate::domain::entities::{ContractorEvaluation, EvaluationScores};
use crate::infrastructure::database::pool::DbPool;
use async_trait::async_trait;
use sqlx::Row;
use uuid::Uuid;

pub struct PostgresContractorEvaluationRepository {
    pool: DbPool,
}

impl PostgresContractorEvaluationRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    fn row_to_evaluation(row: &sqlx::postgres::PgRow) -> Result<ContractorEvaluation, AppError> {
        let quality: i16 = row.get("score_quality");
        let timeliness: i16 = row.get("score_timeliness");
        let communication: i16 = row.get("score_communication");
        let cost_compliance: i16 = row.get("score_cost_compliance");
        let overall: i16 = row.get("score_overall");
        Ok(ContractorEvaluation {
            id: row.get("id"),
            contractor_user_id: row.get("contractor_user_id"),
            technical_spec_id: row.get("technical_spec_id"),
            linked_ticket_ids: row.get("linked_ticket_ids"),
            evaluator_user_id: row.get("evaluator_user_id"),
            scores: EvaluationScores {
                quality: quality as u8,
                timeliness: timeliness as u8,
                communication: communication as u8,
                cost_compliance: cost_compliance as u8,
                overall: overall as u8,
            },
            comment: row.get("comment"),
            created_at: row.get("created_at"),
        })
    }
}

#[async_trait]
impl ContractorEvaluationRepository for PostgresContractorEvaluationRepository {
    async fn save(&self, e: &ContractorEvaluation) -> Result<(), AppError> {
        sqlx::query(
            r#"
            INSERT INTO contractor_evaluations (
                id, contractor_user_id, technical_spec_id, linked_ticket_ids,
                evaluator_user_id,
                score_quality, score_timeliness, score_communication,
                score_cost_compliance, score_overall,
                comment, created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            "#,
        )
        .bind(e.id)
        .bind(e.contractor_user_id)
        .bind(e.technical_spec_id)
        .bind(&e.linked_ticket_ids)
        .bind(e.evaluator_user_id)
        .bind(e.scores.quality as i16)
        .bind(e.scores.timeliness as i16)
        .bind(e.scores.communication as i16)
        .bind(e.scores.cost_compliance as i16)
        .bind(e.scores.overall as i16)
        .bind(&e.comment)
        .bind(e.created_at)
        .execute(&self.pool)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;
        Ok(())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<ContractorEvaluation>, AppError> {
        let row = sqlx::query(
            r#"
            SELECT id, contractor_user_id, technical_spec_id, linked_ticket_ids,
                   evaluator_user_id,
                   score_quality, score_timeliness, score_communication,
                   score_cost_compliance, score_overall,
                   comment, created_at
            FROM contractor_evaluations
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        match row {
            None => Ok(None),
            Some(row) => Ok(Some(Self::row_to_evaluation(&row)?)),
        }
    }

    async fn list_for_contractor(
        &self,
        contractor_user_id: Uuid,
    ) -> Result<Vec<ContractorEvaluation>, AppError> {
        let rows = sqlx::query(
            r#"
            SELECT id, contractor_user_id, technical_spec_id, linked_ticket_ids,
                   evaluator_user_id,
                   score_quality, score_timeliness, score_communication,
                   score_cost_compliance, score_overall,
                   comment, created_at
            FROM contractor_evaluations
            WHERE contractor_user_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(contractor_user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        rows.iter().map(Self::row_to_evaluation).collect()
    }
}
