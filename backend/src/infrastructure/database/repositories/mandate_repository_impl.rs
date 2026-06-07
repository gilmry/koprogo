//! PostgreSQL implementation of [`MandateRepository`] (Story 3.4).
//!
//! All `sqlx::Error` paths are wrapped in `AppError::Database(_)` — no
//! `Result<_, String>` debt (CRITICAL.md #4 / #555).

use crate::application::error::AppError;
use crate::application::ports::MandateRepository;
use crate::domain::entities::{Mandate, MandateKind, MandateScope};
use crate::infrastructure::database::pool::DbPool;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::Row;
use std::str::FromStr;
use uuid::Uuid;

pub struct PostgresMandateRepository {
    pool: DbPool,
}

impl PostgresMandateRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    fn row_to_mandate(row: &sqlx::postgres::PgRow) -> Result<Mandate, AppError> {
        let kind_str: String = row.get("kind");
        let kind = MandateKind::from_str(&kind_str)?;
        let scope_kind_str: String = row.get("scope_kind");
        let scope_id: Uuid = row.get("scope_id");
        let scope = MandateScope::from_parts(&scope_kind_str, scope_id)?;
        Ok(Mandate {
            id: row.get("id"),
            subject_user_id: row.get("subject_user_id"),
            kind,
            scope,
            issued_by: row.get("issued_by"),
            reason: row.get("reason"),
            valid_from: row.get("valid_from"),
            valid_until: row.get("valid_until"),
            revoked_at: row.get("revoked_at"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }
}

#[async_trait]
impl MandateRepository for PostgresMandateRepository {
    async fn save(&self, mandate: &Mandate) -> Result<(), AppError> {
        sqlx::query(
            r#"
            INSERT INTO mandates (
                id, subject_user_id, kind, scope_kind, scope_id,
                issued_by, reason, valid_from, valid_until, revoked_at,
                created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            "#,
        )
        .bind(mandate.id)
        .bind(mandate.subject_user_id)
        .bind(mandate.kind.to_string())
        .bind(mandate.scope.kind_str())
        .bind(mandate.scope.id())
        .bind(mandate.issued_by)
        .bind(&mandate.reason)
        .bind(mandate.valid_from)
        .bind(mandate.valid_until)
        .bind(mandate.revoked_at)
        .bind(mandate.created_at)
        .bind(mandate.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Mandate>, AppError> {
        let row = sqlx::query(
            r#"
            SELECT id, subject_user_id, kind, scope_kind, scope_id,
                   issued_by, reason, valid_from, valid_until, revoked_at,
                   created_at, updated_at
            FROM mandates
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        match row {
            None => Ok(None),
            Some(row) => Ok(Some(Self::row_to_mandate(&row)?)),
        }
    }

    async fn list_active_for_subject(
        &self,
        subject_user_id: Uuid,
    ) -> Result<Vec<Mandate>, AppError> {
        let rows = sqlx::query(
            r#"
            SELECT id, subject_user_id, kind, scope_kind, scope_id,
                   issued_by, reason, valid_from, valid_until, revoked_at,
                   created_at, updated_at
            FROM mandates
            WHERE subject_user_id = $1
              AND revoked_at IS NULL
              AND valid_from <= NOW()
              AND valid_until > NOW()
            ORDER BY valid_until ASC
            "#,
        )
        .bind(subject_user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        rows.iter().map(Self::row_to_mandate).collect()
    }

    async fn list_for_scope(&self, scope: &MandateScope) -> Result<Vec<Mandate>, AppError> {
        let rows = sqlx::query(
            r#"
            SELECT id, subject_user_id, kind, scope_kind, scope_id,
                   issued_by, reason, valid_from, valid_until, revoked_at,
                   created_at, updated_at
            FROM mandates
            WHERE scope_kind = $1 AND scope_id = $2
            ORDER BY created_at DESC
            "#,
        )
        .bind(scope.kind_str())
        .bind(scope.id())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        rows.iter().map(Self::row_to_mandate).collect()
    }

    async fn revoke(&self, id: Uuid, revoked_at: DateTime<Utc>) -> Result<(), AppError> {
        // Idempotent: only updates rows that are not already revoked.
        sqlx::query(
            r#"
            UPDATE mandates
            SET revoked_at = $2, updated_at = $2
            WHERE id = $1 AND revoked_at IS NULL
            "#,
        )
        .bind(id)
        .bind(revoked_at)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(())
    }
}
