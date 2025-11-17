use crate::application::ports::UserRepository;
use crate::domain::entities::{User, UserRole};
use crate::infrastructure::pool::DbPool;
use async_trait::async_trait;
use uuid::Uuid;

pub struct PostgresUserRepository {
    pool: DbPool,
}

impl PostgresUserRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserRepository for PostgresUserRepository {
    async fn create(&self, user: &User) -> Result<User, String> {
        sqlx::query!(
            r#"
            INSERT INTO users (id, email, password_hash, first_name, last_name, role, organization_id, is_active, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
            user.id,
            user.email,
            user.password_hash,
            user.first_name,
            user.last_name,
            user.role.to_string(),
            user.organization_id,
            user.is_active,
            user.created_at,
            user.updated_at,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to create user: {}", e))?;

        Ok(user.clone())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, String> {
        let result = sqlx::query!(
            r#"
            SELECT id, email, password_hash, first_name, last_name, role, organization_id, is_active, created_at, updated_at
            FROM users
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Failed to find user: {}", e))?;

        match result {
            Some(row) => {
                let role = row
                    .role
                    .parse::<UserRole>()
                    .map_err(|e| format!("Invalid role: {}", e))?;

                Ok(Some(User {
                    id: row.id,
                    email: row.email,
                    password_hash: row.password_hash,
                    first_name: row.first_name,
                    last_name: row.last_name,
                    role,
                    organization_id: row.organization_id,
                    is_active: row.is_active,
                    created_at: row.created_at,
                    updated_at: row.updated_at,
                }))
            }
            None => Ok(None),
        }
    }

    async fn find_by_email(&self, email: &str) -> Result<Option<User>, String> {
        let result = sqlx::query!(
            r#"
            SELECT id, email, password_hash, first_name, last_name, role, organization_id, is_active, created_at, updated_at
            FROM users
            WHERE email = $1
            "#,
            email
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Failed to find user by email: {}", e))?;

        match result {
            Some(row) => {
                let role = row
                    .role
                    .parse::<UserRole>()
                    .map_err(|e| format!("Invalid role: {}", e))?;

                Ok(Some(User {
                    id: row.id,
                    email: row.email,
                    password_hash: row.password_hash,
                    first_name: row.first_name,
                    last_name: row.last_name,
                    role,
                    organization_id: row.organization_id,
                    is_active: row.is_active,
                    created_at: row.created_at,
                    updated_at: row.updated_at,
                }))
            }
            None => Ok(None),
        }
    }

    async fn find_all(&self) -> Result<Vec<User>, String> {
        let rows = sqlx::query!(
            r#"
            SELECT id, email, password_hash, first_name, last_name, role, organization_id, is_active, created_at, updated_at
            FROM users
            ORDER BY created_at DESC
            "#
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to fetch users: {}", e))?;

        let users = rows
            .into_iter()
            .filter_map(|row| {
                let role = row.role.parse::<UserRole>().ok()?;
                Some(User {
                    id: row.id,
                    email: row.email,
                    password_hash: row.password_hash,
                    first_name: row.first_name,
                    last_name: row.last_name,
                    role,
                    organization_id: row.organization_id,
                    is_active: row.is_active,
                    processing_restricted: false, // Default: no restriction
                    processing_restricted_at: None,
                    marketing_opt_out: false, // Default: opt-in
                    marketing_opt_out_at: None,
                    created_at: row.created_at,
                    updated_at: row.updated_at,
                })
            })
            .collect();

        Ok(users)
    }

    async fn find_by_organization(&self, org_id: Uuid) -> Result<Vec<User>, String> {
        let rows = sqlx::query!(
            r#"
            SELECT id, email, password_hash, first_name, last_name, role, organization_id, is_active, created_at, updated_at
            FROM users
            WHERE organization_id = $1
            ORDER BY created_at DESC
            "#,
            org_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to fetch users by organization: {}", e))?;

        let users = rows
            .into_iter()
            .filter_map(|row| {
                let role = row.role.parse::<UserRole>().ok()?;
                Some(User {
                    id: row.id,
                    email: row.email,
                    password_hash: row.password_hash,
                    first_name: row.first_name,
                    last_name: row.last_name,
                    role,
                    organization_id: row.organization_id,
                    is_active: row.is_active,
                    processing_restricted: false, // Default: no restriction
                    processing_restricted_at: None,
                    marketing_opt_out: false, // Default: opt-in
                    marketing_opt_out_at: None,
                    created_at: row.created_at,
                    updated_at: row.updated_at,
                })
            })
            .collect();

        Ok(users)
    }

    async fn update(&self, user: &User) -> Result<User, String> {
        sqlx::query!(
            r#"
            UPDATE users
            SET email = $2, first_name = $3, last_name = $4, role = $5,
                organization_id = $6, is_active = $7, updated_at = $8
            WHERE id = $1
            "#,
            user.id,
            user.email,
            user.first_name,
            user.last_name,
            user.role.to_string(),
            user.organization_id,
            user.is_active,
            user.updated_at,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to update user: {}", e))?;

        Ok(user.clone())
    }

    async fn delete(&self, id: Uuid) -> Result<bool, String> {
        let result = sqlx::query!(
            r#"
            DELETE FROM users
            WHERE id = $1
            "#,
            id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to delete user: {}", e))?;

        Ok(result.rows_affected() > 0)
    }

    async fn count_by_organization(&self, org_id: Uuid) -> Result<i64, String> {
        let result = sqlx::query!(
            r#"
            SELECT COUNT(*) as count
            FROM users
            WHERE organization_id = $1
            "#,
            org_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to count users: {}", e))?;

        Ok(result.count.unwrap_or(0))
    }
}
