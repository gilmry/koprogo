use crate::application::ports::UserRepository;
use crate::domain::entities::{User, UserRole};
use crate::infrastructure::pool::DbPool;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::Row;
use uuid::Uuid;

pub struct PostgresUserRepository {
    pool: DbPool,
}

impl PostgresUserRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

const USER_COLUMNS: &str = "id, email, password_hash, first_name, last_name, role, organization_id, is_active, processing_restricted, processing_restricted_at, marketing_opt_out, marketing_opt_out_at, created_at, updated_at";

fn row_to_user(row: &sqlx::postgres::PgRow) -> Result<User, String> {
    let role_str: String = row.get("role");
    let role = role_str
        .parse::<UserRole>()
        .map_err(|e| format!("Invalid role: {}", e))?;

    Ok(User {
        id: row.get("id"),
        email: row.get("email"),
        password_hash: row.get("password_hash"),
        first_name: row.get("first_name"),
        last_name: row.get("last_name"),
        role,
        organization_id: row.get("organization_id"),
        is_active: row.get("is_active"),
        processing_restricted: row.get("processing_restricted"),
        processing_restricted_at: row.get::<Option<DateTime<Utc>>, _>("processing_restricted_at"),
        marketing_opt_out: row.get("marketing_opt_out"),
        marketing_opt_out_at: row.get::<Option<DateTime<Utc>>, _>("marketing_opt_out_at"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    })
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
        let sql = format!("SELECT {} FROM users WHERE id = $1", USER_COLUMNS);
        let result = sqlx::query(&sql)
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| format!("Failed to find user: {}", e))?;

        match result {
            Some(row) => Ok(Some(row_to_user(&row)?)),
            None => Ok(None),
        }
    }

    async fn find_by_email(&self, email: &str) -> Result<Option<User>, String> {
        let sql = format!("SELECT {} FROM users WHERE email = $1", USER_COLUMNS);
        let result = sqlx::query(&sql)
            .bind(email)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| format!("Failed to find user by email: {}", e))?;

        match result {
            Some(row) => Ok(Some(row_to_user(&row)?)),
            None => Ok(None),
        }
    }

    async fn find_all(&self) -> Result<Vec<User>, String> {
        let sql = format!(
            "SELECT {} FROM users ORDER BY created_at DESC",
            USER_COLUMNS
        );
        let rows = sqlx::query(&sql)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| format!("Failed to fetch users: {}", e))?;

        let mut users = Vec::new();
        for row in &rows {
            if let Ok(user) = row_to_user(row) {
                users.push(user);
            }
        }

        Ok(users)
    }

    async fn find_by_organization(&self, org_id: Uuid) -> Result<Vec<User>, String> {
        let sql = format!(
            "SELECT {} FROM users WHERE organization_id = $1 ORDER BY created_at DESC",
            USER_COLUMNS
        );
        let rows = sqlx::query(&sql)
            .bind(org_id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| format!("Failed to fetch users by organization: {}", e))?;

        let mut users = Vec::new();
        for row in &rows {
            if let Ok(user) = row_to_user(row) {
                users.push(user);
            }
        }

        Ok(users)
    }

    async fn update(&self, user: &User) -> Result<User, String> {
        sqlx::query(
            r#"
            UPDATE users
            SET email = $2, first_name = $3, last_name = $4, role = $5,
                organization_id = $6, is_active = $7,
                processing_restricted = $8, processing_restricted_at = $9,
                marketing_opt_out = $10, marketing_opt_out_at = $11,
                updated_at = $12
            WHERE id = $1
            "#,
        )
        .bind(user.id)
        .bind(&user.email)
        .bind(&user.first_name)
        .bind(&user.last_name)
        .bind(user.role.to_string())
        .bind(user.organization_id)
        .bind(user.is_active)
        .bind(user.processing_restricted)
        .bind(user.processing_restricted_at)
        .bind(user.marketing_opt_out)
        .bind(user.marketing_opt_out_at)
        .bind(user.updated_at)
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
