//! PostgreSQL implementation of [`NotaryLinkRepository`] (Issue #855).
//!
//! All `sqlx::Error` paths are wrapped in `AppError::Database(_)` — no
//! `Result<_, String>` debt (CRITICAL.md #4).

use crate::application::error::AppError;
use crate::application::ports::NotaryLinkRepository;
use crate::domain::entities::NotaryLink;
use crate::infrastructure::database::pool::DbPool;
use async_trait::async_trait;
use sqlx::Row;
use uuid::Uuid;

pub struct PostgresNotaryLinkRepository {
    pool: DbPool,
}

impl PostgresNotaryLinkRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    fn row_to_link(row: sqlx::postgres::PgRow) -> NotaryLink {
        NotaryLink {
            id: row.get("id"),
            token_hash: row.get("token_hash"),
            etat_date_id: row.get("etat_date_id"),
            issued_by: row.get("issued_by"),
            expires_at: row.get("expires_at"),
            revoked_at: row.get("revoked_at"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }
}

#[async_trait]
impl NotaryLinkRepository for PostgresNotaryLinkRepository {
    async fn save(&self, link: &NotaryLink) -> Result<(), AppError> {
        sqlx::query(
            r#"
            INSERT INTO notary_links (
                id, token_hash, etat_date_id, issued_by, expires_at,
                revoked_at, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
        )
        .bind(link.id)
        .bind(&link.token_hash)
        .bind(link.etat_date_id)
        .bind(link.issued_by)
        .bind(link.expires_at)
        .bind(link.revoked_at)
        .bind(link.created_at)
        .bind(link.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(())
    }

    async fn find_by_token_hash(&self, token_hash: &str) -> Result<Option<NotaryLink>, AppError> {
        let row = sqlx::query(
            r#"
            SELECT id, token_hash, etat_date_id, issued_by, expires_at,
                   revoked_at, created_at, updated_at
            FROM notary_links
            WHERE token_hash = $1
            "#,
        )
        .bind(token_hash)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(row.map(Self::row_to_link))
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<NotaryLink>, AppError> {
        let row = sqlx::query(
            r#"
            SELECT id, token_hash, etat_date_id, issued_by, expires_at,
                   revoked_at, created_at, updated_at
            FROM notary_links
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(row.map(Self::row_to_link))
    }

    async fn list_by_etat_date(&self, etat_date_id: Uuid) -> Result<Vec<NotaryLink>, AppError> {
        let rows = sqlx::query(
            r#"
            SELECT id, token_hash, etat_date_id, issued_by, expires_at,
                   revoked_at, created_at, updated_at
            FROM notary_links
            WHERE etat_date_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(etat_date_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(rows.into_iter().map(Self::row_to_link).collect())
    }

    async fn update(&self, link: &NotaryLink) -> Result<(), AppError> {
        sqlx::query(
            r#"
            UPDATE notary_links
            SET expires_at = $2, revoked_at = $3, updated_at = $4
            WHERE id = $1
            "#,
        )
        .bind(link.id)
        .bind(link.expires_at)
        .bind(link.revoked_at)
        .bind(link.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(())
    }
}
