use crate::application::ports::UserRoleRepository;
use crate::domain::entities::{UserRole, UserRoleAssignment};
use crate::infrastructure::pool::DbPool;
use async_trait::async_trait;
use sqlx::Row;
use uuid::Uuid;

pub struct PostgresUserRoleRepository {
    pool: DbPool,
}

impl PostgresUserRoleRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    fn map_row(row: sqlx::postgres::PgRow) -> Result<UserRoleAssignment, String> {
        let role: UserRole = row
            .try_get::<String, _>("role")
            .map_err(|e| format!("Failed to read role: {}", e))?
            .parse()
            .map_err(|e| format!("Invalid role: {}", e))?;

        Ok(UserRoleAssignment {
            id: row
                .try_get("id")
                .map_err(|e| format!("Failed to read id: {}", e))?,
            user_id: row
                .try_get("user_id")
                .map_err(|e| format!("Failed to read user_id: {}", e))?,
            role,
            organization_id: row
                .try_get("organization_id")
                .map_err(|e| format!("Failed to read organization_id: {}", e))?,
            is_primary: row
                .try_get("is_primary")
                .map_err(|e| format!("Failed to read is_primary: {}", e))?,
            created_at: row
                .try_get("created_at")
                .map_err(|e| format!("Failed to read created_at: {}", e))?,
            updated_at: row
                .try_get("updated_at")
                .map_err(|e| format!("Failed to read updated_at: {}", e))?,
        })
    }
}

#[async_trait]
impl UserRoleRepository for PostgresUserRoleRepository {
    async fn create(&self, assignment: &UserRoleAssignment) -> Result<UserRoleAssignment, String> {
        let row = sqlx::query(
            r#"
            INSERT INTO user_roles (id, user_id, role, organization_id, is_primary, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, user_id, role, organization_id, is_primary, created_at, updated_at
            "#,
        )
        .bind(assignment.id)
        .bind(assignment.user_id)
        .bind(assignment.role.to_string())
        .bind(assignment.organization_id)
        .bind(assignment.is_primary)
        .bind(assignment.created_at)
        .bind(assignment.updated_at)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to create user role: {}", e))?;

        Self::map_row(row)
    }

    async fn list_for_user(&self, user_id: Uuid) -> Result<Vec<UserRoleAssignment>, String> {
        let rows = sqlx::query(
            r#"
            SELECT id, user_id, role, organization_id, is_primary, created_at, updated_at
            FROM user_roles
            WHERE user_id = $1
            ORDER BY is_primary DESC, created_at ASC
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to list user roles: {}", e))?;

        rows.into_iter()
            .map(Self::map_row)
            .collect::<Result<Vec<_>, _>>()
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<UserRoleAssignment>, String> {
        let row = sqlx::query(
            r#"
            SELECT id, user_id, role, organization_id, is_primary, created_at, updated_at
            FROM user_roles
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Failed to find user role: {}", e))?;

        match row {
            Some(row) => Ok(Some(Self::map_row(row)?)),
            None => Ok(None),
        }
    }

    async fn set_primary_role(
        &self,
        user_id: Uuid,
        role_id: Uuid,
    ) -> Result<UserRoleAssignment, String> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| format!("Failed to begin transaction: {}", e))?;

        sqlx::query(
            r#"
            UPDATE user_roles
            SET is_primary = false, updated_at = NOW()
            WHERE user_id = $1
            "#,
        )
        .bind(user_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| format!("Failed to clear primary roles: {}", e))?;

        let row = sqlx::query(
            r#"
            UPDATE user_roles
            SET is_primary = true, updated_at = NOW()
            WHERE id = $1 AND user_id = $2
            RETURNING id, user_id, role, organization_id, is_primary, created_at, updated_at
            "#,
        )
        .bind(role_id)
        .bind(user_id)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| format!("Failed to set primary role: {}", e))?;

        tx.commit()
            .await
            .map_err(|e| format!("Failed to commit transaction: {}", e))?;

        Self::map_row(row)
    }
}
