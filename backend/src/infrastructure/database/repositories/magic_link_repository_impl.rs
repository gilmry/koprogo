//! PostgreSQL implementation of [`MagicLinkRepository`] (Story 3.2).
//!
//! All `sqlx::Error` paths are wrapped in `AppError::Database(_)` — no
//! `Result<_, String>` debt (CRITICAL.md #4, issue #555).

use crate::application::error::AppError;
use crate::application::ports::MagicLinkRepository;
use crate::domain::entities::{MagicLink, MagicLinkScopeKind};
use crate::infrastructure::database::pool::DbPool;
use async_trait::async_trait;
use sqlx::Row;
use std::str::FromStr;
use uuid::Uuid;

pub struct PostgresMagicLinkRepository {
    pool: DbPool,
}

impl PostgresMagicLinkRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl MagicLinkRepository for PostgresMagicLinkRepository {
    async fn save(&self, link: &MagicLink) -> Result<(), AppError> {
        sqlx::query(
            r#"
            INSERT INTO magic_links (
                id, token_hash, subject_user_id, scope_kind, scope_id,
                issued_by, expires_at, consumed_at, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
        )
        .bind(link.id)
        .bind(&link.token_hash)
        .bind(link.subject_user_id)
        .bind(link.scope_kind.to_string())
        .bind(link.scope_id)
        .bind(link.issued_by)
        .bind(link.expires_at)
        .bind(link.consumed_at)
        .bind(link.created_at)
        .bind(link.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(())
    }

    async fn find_by_token_hash(&self, token_hash: &str) -> Result<Option<MagicLink>, AppError> {
        let row = sqlx::query(
            r#"
            SELECT id, token_hash, subject_user_id, scope_kind, scope_id,
                   issued_by, expires_at, consumed_at, created_at, updated_at
            FROM magic_links
            WHERE token_hash = $1
            "#,
        )
        .bind(token_hash)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        match row {
            None => Ok(None),
            Some(row) => {
                let scope_kind_str: String = row.get("scope_kind");
                let scope_kind = MagicLinkScopeKind::from_str(&scope_kind_str)?;
                Ok(Some(MagicLink {
                    id: row.get("id"),
                    token_hash: row.get("token_hash"),
                    subject_user_id: row.get("subject_user_id"),
                    scope_kind,
                    scope_id: row.get("scope_id"),
                    issued_by: row.get("issued_by"),
                    expires_at: row.get("expires_at"),
                    consumed_at: row.get("consumed_at"),
                    created_at: row.get("created_at"),
                    updated_at: row.get("updated_at"),
                }))
            }
        }
    }

    async fn mark_consumed(&self, id: Uuid) -> Result<(), AppError> {
        // Atomic single-use guard: only updates rows that have NOT been consumed.
        // Concurrent attempts will see rows_affected == 0 for the loser.
        let result = sqlx::query(
            r#"
            UPDATE magic_links
            SET consumed_at = NOW(), updated_at = NOW()
            WHERE id = $1 AND consumed_at IS NULL
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(AppError::MagicLinkAlreadyConsumed);
        }
        Ok(())
    }
}
