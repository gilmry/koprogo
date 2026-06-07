//! PostgreSQL implementation of [`RoleDelegationRepository`] (Story 3.5).
//!
//! Persists delegated role assignments inside the existing `user_roles` table
//! using the two new columns introduced by migration
//! `20260605030000_extend_user_roles_delegation.sql`:
//! - `valid_until TIMESTAMPTZ` (NULL = permanent native row)
//! - `delegated_from_user_id UUID` (NULL = native row)

use crate::application::error::AppError;
use crate::application::ports::RoleDelegationRepository;
use crate::domain::entities::{UserRole, UserRoleAssignment};
use crate::infrastructure::pool::DbPool;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::Row;
use std::str::FromStr;
use uuid::Uuid;

pub struct PostgresRoleDelegationRepository {
    pool: DbPool,
}

impl PostgresRoleDelegationRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    fn row_to_assignment(row: &sqlx::postgres::PgRow) -> Result<UserRoleAssignment, AppError> {
        let role_str: String = row
            .try_get("role")
            .map_err(|e| AppError::Database(format!("Failed to read role: {}", e)))?;
        let role = UserRole::from_str(&role_str)
            .map_err(|e| AppError::Database(format!("Invalid role in DB: {}", e)))?;
        let valid_until: Option<DateTime<Utc>> = row
            .try_get("valid_until")
            .map_err(|e| AppError::Database(format!("Failed to read valid_until: {}", e)))?;
        let delegated_from_user_id: Option<Uuid> =
            row.try_get("delegated_from_user_id").map_err(|e| {
                AppError::Database(format!("Failed to read delegated_from_user_id: {}", e))
            })?;
        Ok(UserRoleAssignment {
            id: row
                .try_get("id")
                .map_err(|e| AppError::Database(format!("Failed to read id: {}", e)))?,
            user_id: row
                .try_get("user_id")
                .map_err(|e| AppError::Database(format!("Failed to read user_id: {}", e)))?,
            role,
            organization_id: row.try_get("organization_id").map_err(|e| {
                AppError::Database(format!("Failed to read organization_id: {}", e))
            })?,
            is_primary: row
                .try_get("is_primary")
                .map_err(|e| AppError::Database(format!("Failed to read is_primary: {}", e)))?,
            valid_until,
            delegated_from_user_id,
            created_at: row
                .try_get("created_at")
                .map_err(|e| AppError::Database(format!("Failed to read created_at: {}", e)))?,
            updated_at: row
                .try_get("updated_at")
                .map_err(|e| AppError::Database(format!("Failed to read updated_at: {}", e)))?,
        })
    }
}

#[async_trait]
impl RoleDelegationRepository for PostgresRoleDelegationRepository {
    async fn save(&self, a: &UserRoleAssignment) -> Result<(), AppError> {
        sqlx::query(
            r#"
            INSERT INTO user_roles (
                id, user_id, role, organization_id, is_primary,
                valid_until, delegated_from_user_id,
                created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
        )
        .bind(a.id)
        .bind(a.user_id)
        .bind(a.role.to_string())
        .bind(a.organization_id)
        .bind(a.is_primary)
        .bind(a.valid_until)
        .bind(a.delegated_from_user_id)
        .bind(a.created_at)
        .bind(a.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Database(format!("Failed to save delegation: {}", e)))?;
        Ok(())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<UserRoleAssignment>, AppError> {
        let row = sqlx::query(
            r#"
            SELECT id, user_id, role, organization_id, is_primary,
                   valid_until, delegated_from_user_id,
                   created_at, updated_at
            FROM user_roles
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Database(format!("Failed to find delegation: {}", e)))?;
        match row {
            None => Ok(None),
            Some(row) => Ok(Some(Self::row_to_assignment(&row)?)),
        }
    }

    async fn find_active_by_user_and_role(
        &self,
        user_id: Uuid,
        role: &UserRole,
        organization_id: Option<Uuid>,
    ) -> Result<Vec<UserRoleAssignment>, AppError> {
        // "Active" = either permanent (`valid_until IS NULL`) or still within
        // its validity window. NULL-safe equality on `organization_id`.
        let rows = sqlx::query(
            r#"
            SELECT id, user_id, role, organization_id, is_primary,
                   valid_until, delegated_from_user_id,
                   created_at, updated_at
            FROM user_roles
            WHERE user_id = $1
              AND role = $2
              AND (organization_id IS NOT DISTINCT FROM $3)
              AND (valid_until IS NULL OR valid_until > NOW())
            ORDER BY created_at ASC
            "#,
        )
        .bind(user_id)
        .bind(role.to_string())
        .bind(organization_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Database(format!("Failed to load assignments: {}", e)))?;

        rows.iter().map(Self::row_to_assignment).collect()
    }

    async fn list_delegations_of(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<UserRoleAssignment>, AppError> {
        let rows = sqlx::query(
            r#"
            SELECT id, user_id, role, organization_id, is_primary,
                   valid_until, delegated_from_user_id,
                   created_at, updated_at
            FROM user_roles
            WHERE delegated_from_user_id IS NOT NULL
              AND valid_until IS NOT NULL
              AND valid_until > NOW()
              AND (user_id = $1 OR delegated_from_user_id = $1)
            ORDER BY created_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Database(format!("Failed to list delegations: {}", e)))?;

        rows.iter().map(Self::row_to_assignment).collect()
    }

    async fn revoke(&self, id: Uuid) -> Result<(), AppError> {
        // We delete the row outright — delegations are typically short-lived
        // and the audit trail lives in `audit_log` (cf. Story 3.4 mandate
        // pattern but adapted for the lighter delegation lifecycle).
        // The DELETE is restricted to delegation rows (delegated_from_user_id
        // NOT NULL) to prevent accidentally wiping a native assignment.
        sqlx::query(
            r#"
            DELETE FROM user_roles
            WHERE id = $1 AND delegated_from_user_id IS NOT NULL
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Database(format!("Failed to revoke delegation: {}", e)))?;
        Ok(())
    }
}
