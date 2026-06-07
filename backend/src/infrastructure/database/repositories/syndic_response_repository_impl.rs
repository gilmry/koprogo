//! PostgreSQL implementation of [`SyndicResponseRepository`] (Story 3.7 —
//! FR32 INV-23).
//!
//! Append-only by construction: only [`Self::save`] writes new rows. There
//! is no `update` / `delete` method on the trait. The DB trigger
//! `syndic_responses_no_update` (cf. migration `20260605050000`) ensures
//! the same guarantee at the SQL boundary.

use crate::application::error::AppError;
use crate::application::ports::SyndicResponseRepository;
use crate::domain::entities::SyndicResponse;
use crate::infrastructure::database::pool::DbPool;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::Row;
use uuid::Uuid;

pub struct PostgresSyndicResponseRepository {
    pool: DbPool,
}

impl PostgresSyndicResponseRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    fn row_to_response(row: &sqlx::postgres::PgRow) -> SyndicResponse {
        SyndicResponse {
            id: row.get("id"),
            ticket_id: row.get("ticket_id"),
            syndic_user_id: row.get("syndic_user_id"),
            body: row.get("body"),
            action_proposed: row.get("action_proposed"),
            created_at: row.get("created_at"),
        }
    }
}

#[async_trait]
impl SyndicResponseRepository for PostgresSyndicResponseRepository {
    async fn save(&self, response: &SyndicResponse) -> Result<(), AppError> {
        sqlx::query(
            r#"
            INSERT INTO syndic_responses (
                id, ticket_id, syndic_user_id, body, action_proposed, created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(response.id)
        .bind(response.ticket_id)
        .bind(response.syndic_user_id)
        .bind(&response.body)
        .bind(response.action_proposed.as_deref())
        .bind(response.created_at)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(())
    }

    async fn list_for_ticket(&self, ticket_id: Uuid) -> Result<Vec<SyndicResponse>, AppError> {
        let rows = sqlx::query(
            r#"
            SELECT id, ticket_id, syndic_user_id, body, action_proposed, created_at
            FROM syndic_responses
            WHERE ticket_id = $1
            ORDER BY created_at ASC
            "#,
        )
        .bind(ticket_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(rows.iter().map(Self::row_to_response).collect())
    }

    async fn find_overdue_tickets(&self, now: DateTime<Utc>) -> Result<Vec<Uuid>, AppError> {
        let rows = sqlx::query(
            r#"
            SELECT id
            FROM tickets
            WHERE sla_due_at IS NOT NULL
              AND sla_due_at <= $1
              AND sla_escalated_at IS NULL
            "#,
        )
        .bind(now)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(rows.iter().map(|r| r.get::<Uuid, _>("id")).collect())
    }

    async fn mark_ticket_escalated(
        &self,
        ticket_id: Uuid,
        escalated_at: DateTime<Utc>,
    ) -> Result<(), AppError> {
        // Idempotency: only the first write succeeds (CAS on NULL).
        sqlx::query(
            r#"
            UPDATE tickets
            SET sla_escalated_at = $2
            WHERE id = $1 AND sla_escalated_at IS NULL
            "#,
        )
        .bind(ticket_id)
        .bind(escalated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(())
    }
}
