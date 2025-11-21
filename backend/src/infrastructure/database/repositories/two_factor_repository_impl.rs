use crate::application::ports::TwoFactorRepository;
use crate::domain::entities::TwoFactorSecret;
use async_trait::async_trait;
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

/// PostgreSQL implementation of TwoFactorRepository
pub struct PostgresTwoFactorRepository {
    pool: PgPool,
}

impl PostgresTwoFactorRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TwoFactorRepository for PostgresTwoFactorRepository {
    async fn create(&self, secret: &TwoFactorSecret) -> Result<TwoFactorSecret, String> {
        sqlx::query_as!(
            TwoFactorSecretRow,
            r#"
            INSERT INTO two_factor_secrets (
                id, user_id, secret_encrypted, backup_codes_encrypted,
                is_enabled, verified_at, last_used_at, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING
                id, user_id, secret_encrypted, backup_codes_encrypted,
                is_enabled, verified_at, last_used_at, created_at, updated_at
            "#,
            secret.id,
            secret.user_id,
            secret.secret_encrypted,
            &secret.backup_codes_encrypted,
            secret.is_enabled,
            secret.verified_at,
            secret.last_used_at,
            secret.created_at,
            secret.updated_at
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to create two factor secret: {}", e))?
        .try_into()
    }

    async fn find_by_user_id(&self, user_id: Uuid) -> Result<Option<TwoFactorSecret>, String> {
        let result = sqlx::query_as!(
            TwoFactorSecretRow,
            r#"
            SELECT
                id, user_id, secret_encrypted, backup_codes_encrypted,
                is_enabled, verified_at, last_used_at, created_at, updated_at
            FROM two_factor_secrets
            WHERE user_id = $1
            "#,
            user_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Failed to find two factor secret by user_id: {}", e))?;

        match result {
            Some(row) => Ok(Some(row.try_into()?)),
            None => Ok(None),
        }
    }

    async fn update(&self, secret: &TwoFactorSecret) -> Result<TwoFactorSecret, String> {
        let updated_at = Utc::now();

        sqlx::query_as!(
            TwoFactorSecretRow,
            r#"
            UPDATE two_factor_secrets
            SET
                secret_encrypted = $2,
                backup_codes_encrypted = $3,
                is_enabled = $4,
                verified_at = $5,
                last_used_at = $6,
                updated_at = $7
            WHERE id = $1
            RETURNING
                id, user_id, secret_encrypted, backup_codes_encrypted,
                is_enabled, verified_at, last_used_at, created_at, updated_at
            "#,
            secret.id,
            secret.secret_encrypted,
            &secret.backup_codes_encrypted,
            secret.is_enabled,
            secret.verified_at,
            secret.last_used_at,
            updated_at
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to update two factor secret: {}", e))?
        .try_into()
    }

    async fn delete(&self, user_id: Uuid) -> Result<(), String> {
        let result = sqlx::query!(
            r#"
            DELETE FROM two_factor_secrets
            WHERE user_id = $1
            "#,
            user_id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to delete two factor secret: {}", e))?;

        if result.rows_affected() == 0 {
            return Err(format!("Two factor secret not found for user {}", user_id));
        }

        Ok(())
    }

    async fn find_needing_reverification(&self) -> Result<Vec<TwoFactorSecret>, String> {
        sqlx::query_as!(
            TwoFactorSecretRow,
            r#"
            SELECT
                id, user_id, secret_encrypted, backup_codes_encrypted,
                is_enabled, verified_at, last_used_at, created_at, updated_at
            FROM two_factor_secrets
            WHERE is_enabled = true
              AND last_used_at < NOW() - INTERVAL '90 days'
            ORDER BY last_used_at ASC
            "#
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to find secrets needing reverification: {}", e))?
        .into_iter()
        .map(|row| row.try_into())
        .collect()
    }

    async fn find_with_low_backup_codes(&self) -> Result<Vec<TwoFactorSecret>, String> {
        sqlx::query_as!(
            TwoFactorSecretRow,
            r#"
            SELECT
                id, user_id, secret_encrypted, backup_codes_encrypted,
                is_enabled, verified_at, last_used_at, created_at, updated_at
            FROM two_factor_secrets
            WHERE is_enabled = true
              AND array_length(backup_codes_encrypted, 1) < 3
            ORDER BY array_length(backup_codes_encrypted, 1) ASC
            "#
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to find secrets with low backup codes: {}", e))?
        .into_iter()
        .map(|row| row.try_into())
        .collect()
    }
}

// ========================================
// Database row mapping
// ========================================

/// SQLx row struct for two_factor_secrets table
#[derive(Debug)]
struct TwoFactorSecretRow {
    id: Uuid,
    user_id: Uuid,
    secret_encrypted: String,
    backup_codes_encrypted: Vec<String>,
    is_enabled: bool,
    verified_at: Option<chrono::DateTime<Utc>>,
    last_used_at: Option<chrono::DateTime<Utc>>,
    created_at: chrono::DateTime<Utc>,
    updated_at: chrono::DateTime<Utc>,
}

impl TryFrom<TwoFactorSecretRow> for TwoFactorSecret {
    type Error = String;

    fn try_from(row: TwoFactorSecretRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: row.id,
            user_id: row.user_id,
            secret_encrypted: row.secret_encrypted,
            backup_codes_encrypted: row.backup_codes_encrypted,
            is_enabled: row.is_enabled,
            verified_at: row.verified_at,
            last_used_at: row.last_used_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::TwoFactorSecret;
    use testcontainers_modules::{postgres::Postgres, testcontainers::runners::AsyncRunner};

    async fn setup_test_db() -> PgPool {
        let container = Postgres::default().start().await.unwrap();
        let host_port = container.get_host_port_ipv4(5432).await.unwrap();
        let connection_string = format!(
            "postgres://postgres:postgres@127.0.0.1:{}/postgres",
            host_port
        );

        let pool = PgPool::connect(&connection_string).await.unwrap();

        // Run migrations
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();

        pool
    }

    #[tokio::test]
    async fn test_create_two_factor_secret() {
        let pool = setup_test_db().await;
        let repo = PostgresTwoFactorRepository::new(pool.clone());

        // Create test user first
        let user_id = Uuid::new_v4();
        sqlx::query(
            r#"
            INSERT INTO users (id, email, first_name, last_name, password_hash, is_active, created_at, updated_at)
            VALUES ($1, $2, 'Test', 'User', 'hash', true, NOW(), NOW())
            "#
        )
        .bind(user_id)
        .bind(format!("test-{}@example.com", user_id))
        .execute(&pool)
        .await
        .unwrap();

        let secret = TwoFactorSecret::new(user_id, "encrypted_secret".to_string())
            .unwrap()
            .with_backup_codes(vec![
                "code1".to_string(),
                "code2".to_string(),
                "code3".to_string(),
            ])
            .unwrap();

        let created = repo.create(&secret).await.unwrap();

        assert_eq!(created.user_id, user_id);
        assert_eq!(created.secret_encrypted, "encrypted_secret");
        assert_eq!(created.backup_codes_encrypted.len(), 3);
        assert!(!created.is_enabled);
    }

    #[tokio::test]
    async fn test_find_by_user_id() {
        let pool = setup_test_db().await;
        let repo = PostgresTwoFactorRepository::new(pool.clone());

        // Create test user
        let user_id = Uuid::new_v4();
        sqlx::query(
            r#"
            INSERT INTO users (id, email, first_name, last_name, password_hash, is_active, created_at, updated_at)
            VALUES ($1, $2, 'Test', 'User', 'hash', true, NOW(), NOW())
            "#
        )
        .bind(user_id)
        .bind(format!("test-{}@example.com", user_id))
        .execute(&pool)
        .await
        .unwrap();

        let secret = TwoFactorSecret::new(user_id, "encrypted_secret".to_string()).unwrap();

        repo.create(&secret).await.unwrap();

        let found = repo.find_by_user_id(user_id).await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().user_id, user_id);
    }

    #[tokio::test]
    async fn test_update_two_factor_secret() {
        let pool = setup_test_db().await;
        let repo = PostgresTwoFactorRepository::new(pool.clone());

        // Create test user
        let user_id = Uuid::new_v4();
        sqlx::query(
            r#"
            INSERT INTO users (id, email, first_name, last_name, password_hash, is_active, created_at, updated_at)
            VALUES ($1, $2, 'Test', 'User', 'hash', true, NOW(), NOW())
            "#
        )
        .bind(user_id)
        .bind(format!("test-{}@example.com", user_id))
        .execute(&pool)
        .await
        .unwrap();

        let mut secret = TwoFactorSecret::new(user_id, "encrypted_secret".to_string()).unwrap();

        let created = repo.create(&secret).await.unwrap();

        secret = created;
        secret.enable().unwrap();

        let updated = repo.update(&secret).await.unwrap();
        assert!(updated.is_enabled);
        assert!(updated.verified_at.is_some());
    }

    #[tokio::test]
    async fn test_delete_two_factor_secret() {
        let pool = setup_test_db().await;
        let repo = PostgresTwoFactorRepository::new(pool.clone());

        // Create test user
        let user_id = Uuid::new_v4();
        sqlx::query(
            r#"
            INSERT INTO users (id, email, first_name, last_name, password_hash, is_active, created_at, updated_at)
            VALUES ($1, $2, 'Test', 'User', 'hash', true, NOW(), NOW())
            "#
        )
        .bind(user_id)
        .bind(format!("test-{}@example.com", user_id))
        .execute(&pool)
        .await
        .unwrap();

        let secret = TwoFactorSecret::new(user_id, "encrypted_secret".to_string()).unwrap();

        repo.create(&secret).await.unwrap();
        repo.delete(user_id).await.unwrap();

        let found = repo.find_by_user_id(user_id).await.unwrap();
        assert!(found.is_none());
    }

    #[tokio::test]
    async fn test_find_needing_reverification() {
        let pool = setup_test_db().await;
        let repo = PostgresTwoFactorRepository::new(pool.clone());

        // Create test user
        let user_id = Uuid::new_v4();
        sqlx::query(
            r#"
            INSERT INTO users (id, email, first_name, last_name, password_hash, is_active, created_at, updated_at)
            VALUES ($1, $2, 'Test', 'User', 'hash', true, NOW(), NOW())
            "#
        )
        .bind(user_id)
        .bind(format!("test-{}@example.com", user_id))
        .execute(&pool)
        .await
        .unwrap();

        let mut secret = TwoFactorSecret::new(user_id, "encrypted_secret".to_string()).unwrap();

        secret.enable().unwrap();
        let created = repo.create(&secret).await.unwrap();

        // Manually set last_used_at to 91 days ago
        sqlx::query(
            "UPDATE two_factor_secrets SET last_used_at = NOW() - INTERVAL '91 days' WHERE id = $1"
        )
        .bind(created.id)
        .execute(&pool)
        .await
        .unwrap();

        let needing_reverification = repo.find_needing_reverification().await.unwrap();
        assert_eq!(needing_reverification.len(), 1);
        assert_eq!(needing_reverification[0].user_id, user_id);
    }

    #[tokio::test]
    async fn test_find_with_low_backup_codes() {
        let pool = setup_test_db().await;
        let repo = PostgresTwoFactorRepository::new(pool.clone());

        // Create test user
        let user_id = Uuid::new_v4();
        sqlx::query(
            r#"
            INSERT INTO users (id, email, first_name, last_name, password_hash, is_active, created_at, updated_at)
            VALUES ($1, $2, 'Test', 'User', 'hash', true, NOW(), NOW())
            "#
        )
        .bind(user_id)
        .bind(format!("test-{}@example.com", user_id))
        .execute(&pool)
        .await
        .unwrap();

        let mut secret = TwoFactorSecret::new(user_id, "encrypted_secret".to_string())
            .unwrap()
            .with_backup_codes(vec!["code1".to_string(), "code2".to_string()])
            .unwrap();

        secret.enable().unwrap();
        repo.create(&secret).await.unwrap();

        let low_codes = repo.find_with_low_backup_codes().await.unwrap();
        assert_eq!(low_codes.len(), 1);
        assert_eq!(low_codes[0].user_id, user_id);
    }
}
