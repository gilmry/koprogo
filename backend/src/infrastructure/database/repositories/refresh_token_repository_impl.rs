use crate::application::ports::RefreshTokenRepository;
use crate::domain::entities::RefreshToken;
use crate::infrastructure::database::pool::DbPool;
use async_trait::async_trait;
use sqlx::Row;
use uuid::Uuid;

pub struct PostgresRefreshTokenRepository {
    pool: DbPool,
}

impl PostgresRefreshTokenRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl RefreshTokenRepository for PostgresRefreshTokenRepository {
    async fn create(&self, refresh_token: &RefreshToken) -> Result<RefreshToken, String> {
        sqlx::query(
            r#"
            INSERT INTO refresh_tokens (id, user_id, token, expires_at, revoked, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
        )
        .bind(refresh_token.id)
        .bind(refresh_token.user_id)
        .bind(&refresh_token.token)
        .bind(refresh_token.expires_at)
        .bind(refresh_token.revoked)
        .bind(refresh_token.created_at)
        .bind(refresh_token.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(refresh_token.clone())
    }

    async fn find_by_token(&self, token: &str) -> Result<Option<RefreshToken>, String> {
        let row = sqlx::query(
            r#"
            SELECT id, user_id, token, expires_at, revoked, created_at, updated_at
            FROM refresh_tokens
            WHERE token = $1
            "#,
        )
        .bind(token)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(row.map(|row| RefreshToken {
            id: row.get("id"),
            user_id: row.get("user_id"),
            token: row.get("token"),
            expires_at: row.get("expires_at"),
            revoked: row.get("revoked"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }))
    }

    async fn find_by_user_id(&self, user_id: Uuid) -> Result<Vec<RefreshToken>, String> {
        let rows = sqlx::query(
            r#"
            SELECT id, user_id, token, expires_at, revoked, created_at, updated_at
            FROM refresh_tokens
            WHERE user_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(rows
            .iter()
            .map(|row| RefreshToken {
                id: row.get("id"),
                user_id: row.get("user_id"),
                token: row.get("token"),
                expires_at: row.get("expires_at"),
                revoked: row.get("revoked"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            })
            .collect())
    }

    async fn revoke(&self, token: &str) -> Result<bool, String> {
        let result = sqlx::query(
            r#"
            UPDATE refresh_tokens
            SET revoked = true, updated_at = NOW()
            WHERE token = $1
            "#,
        )
        .bind(token)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(result.rows_affected() > 0)
    }

    async fn revoke_all_for_user(&self, user_id: Uuid) -> Result<u64, String> {
        let result = sqlx::query(
            r#"
            UPDATE refresh_tokens
            SET revoked = true, updated_at = NOW()
            WHERE user_id = $1
            "#,
        )
        .bind(user_id)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(result.rows_affected())
    }

    async fn delete_expired(&self) -> Result<u64, String> {
        let result = sqlx::query(
            r#"
            DELETE FROM refresh_tokens
            WHERE expires_at < NOW() OR revoked = true
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(result.rows_affected())
    }
}
