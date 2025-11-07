// Infrastructure: PostgreSQL Account Repository Implementation
//
// CREDITS & ATTRIBUTION:
// This implementation is inspired by the Noalyss project (https://gitlab.com/noalyss/noalyss)
// Noalyss is a free accounting software for Belgian and French accounting
// License: GPL-2.0-or-later (GNU General Public License version 2 or later)
// Copyright: (C) 1989, 1991 Free Software Foundation, Inc.
// Copyright: Dany De Bontridder <dany@alchimerys.eu>
//
// The SQL queries and approach are based on Noalyss' Acc_Plan_SQL and Tmp_Pcmn_SQL classes.
// See: include/database/acc_plan_sql.class.php in Noalyss repository

use crate::application::ports::AccountRepository;
use crate::domain::entities::{Account, AccountType};
use async_trait::async_trait;
use sqlx::{PgPool, Row};
use uuid::Uuid;

/// PostgreSQL implementation of AccountRepository
///
/// Manages accounts in the Belgian PCMN (Plan Comptable Minimum Normalisé).
/// Inspired by Noalyss PostgreSQL schema and repository pattern.
pub struct PostgresAccountRepository {
    pool: PgPool,
}

impl PostgresAccountRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AccountRepository for PostgresAccountRepository {
    async fn create(&self, account: &Account) -> Result<Account, String> {
        let result = sqlx::query!(
            r#"
            INSERT INTO accounts (id, code, label, parent_code, account_type, direct_use, organization_id, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5::account_type, $6, $7, $8, $9)
            RETURNING id, code, label, parent_code, account_type AS "account_type: AccountType",
                      direct_use, organization_id, created_at, updated_at
            "#,
            account.id,
            account.code,
            account.label,
            account.parent_code,
            account.account_type as AccountType,
            account.direct_use,
            account.organization_id,
            account.created_at,
            account.updated_at
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to create account: {}", e))?;

        Ok(Account {
            id: result.id,
            code: result.code,
            label: result.label,
            parent_code: result.parent_code,
            account_type: result.account_type,
            direct_use: result.direct_use,
            organization_id: result.organization_id,
            created_at: result.created_at.and_utc(),
            updated_at: result.updated_at.and_utc(),
        })
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Account>, String> {
        let result = sqlx::query!(
            r#"
            SELECT id, code, label, parent_code, account_type AS "account_type: AccountType",
                   direct_use, organization_id, created_at, updated_at
            FROM accounts
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Failed to find account by id: {}", e))?;

        Ok(result.map(|row| Account {
            id: row.id,
            code: row.code,
            label: row.label,
            parent_code: row.parent_code,
            account_type: row.account_type,
            direct_use: row.direct_use,
            organization_id: row.organization_id,
            created_at: row.created_at.and_utc(),
            updated_at: row.updated_at.and_utc(),
        }))
    }

    async fn find_by_code(
        &self,
        code: &str,
        organization_id: Uuid,
    ) -> Result<Option<Account>, String> {
        let result = sqlx::query!(
            r#"
            SELECT id, code, label, parent_code, account_type AS "account_type: AccountType",
                   direct_use, organization_id, created_at, updated_at
            FROM accounts
            WHERE code = $1 AND organization_id = $2
            "#,
            code,
            organization_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Failed to find account by code: {}", e))?;

        Ok(result.map(|row| Account {
            id: row.id,
            code: row.code,
            label: row.label,
            parent_code: row.parent_code,
            account_type: row.account_type,
            direct_use: row.direct_use,
            organization_id: row.organization_id,
            created_at: row.created_at.and_utc(),
            updated_at: row.updated_at.and_utc(),
        }))
    }

    async fn find_by_organization(
        &self,
        organization_id: Uuid,
    ) -> Result<Vec<Account>, String> {
        let rows = sqlx::query!(
            r#"
            SELECT id, code, label, parent_code, account_type AS "account_type: AccountType",
                   direct_use, organization_id, created_at, updated_at
            FROM accounts
            WHERE organization_id = $1
            ORDER BY code ASC
            "#,
            organization_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to find accounts by organization: {}", e))?;

        Ok(rows
            .into_iter()
            .map(|row| Account {
                id: row.id,
                code: row.code,
                label: row.label,
                parent_code: row.parent_code,
                account_type: row.account_type,
                direct_use: row.direct_use,
                organization_id: row.organization_id,
                created_at: row.created_at.and_utc(),
                updated_at: row.updated_at.and_utc(),
            })
            .collect())
    }

    async fn find_by_type(
        &self,
        account_type: AccountType,
        organization_id: Uuid,
    ) -> Result<Vec<Account>, String> {
        let rows = sqlx::query!(
            r#"
            SELECT id, code, label, parent_code, account_type AS "account_type: AccountType",
                   direct_use, organization_id, created_at, updated_at
            FROM accounts
            WHERE account_type = $1::account_type AND organization_id = $2
            ORDER BY code ASC
            "#,
            account_type as AccountType,
            organization_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to find accounts by type: {}", e))?;

        Ok(rows
            .into_iter()
            .map(|row| Account {
                id: row.id,
                code: row.code,
                label: row.label,
                parent_code: row.parent_code,
                account_type: row.account_type,
                direct_use: row.direct_use,
                organization_id: row.organization_id,
                created_at: row.created_at.and_utc(),
                updated_at: row.updated_at.and_utc(),
            })
            .collect())
    }

    async fn find_by_parent_code(
        &self,
        parent_code: &str,
        organization_id: Uuid,
    ) -> Result<Vec<Account>, String> {
        let rows = sqlx::query!(
            r#"
            SELECT id, code, label, parent_code, account_type AS "account_type: AccountType",
                   direct_use, organization_id, created_at, updated_at
            FROM accounts
            WHERE parent_code = $1 AND organization_id = $2
            ORDER BY code ASC
            "#,
            parent_code,
            organization_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to find accounts by parent code: {}", e))?;

        Ok(rows
            .into_iter()
            .map(|row| Account {
                id: row.id,
                code: row.code,
                label: row.label,
                parent_code: row.parent_code,
                account_type: row.account_type,
                direct_use: row.direct_use,
                organization_id: row.organization_id,
                created_at: row.created_at.and_utc(),
                updated_at: row.updated_at.and_utc(),
            })
            .collect())
    }

    async fn find_direct_use_accounts(
        &self,
        organization_id: Uuid,
    ) -> Result<Vec<Account>, String> {
        let rows = sqlx::query!(
            r#"
            SELECT id, code, label, parent_code, account_type AS "account_type: AccountType",
                   direct_use, organization_id, created_at, updated_at
            FROM accounts
            WHERE direct_use = true AND organization_id = $1
            ORDER BY code ASC
            "#,
            organization_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to find direct use accounts: {}", e))?;

        Ok(rows
            .into_iter()
            .map(|row| Account {
                id: row.id,
                code: row.code,
                label: row.label,
                parent_code: row.parent_code,
                account_type: row.account_type,
                direct_use: row.direct_use,
                organization_id: row.organization_id,
                created_at: row.created_at.and_utc(),
                updated_at: row.updated_at.and_utc(),
            })
            .collect())
    }

    async fn search_by_code_pattern(
        &self,
        code_pattern: &str,
        organization_id: Uuid,
    ) -> Result<Vec<Account>, String> {
        let rows = sqlx::query!(
            r#"
            SELECT id, code, label, parent_code, account_type AS "account_type: AccountType",
                   direct_use, organization_id, created_at, updated_at
            FROM accounts
            WHERE code LIKE $1 AND organization_id = $2
            ORDER BY code ASC
            "#,
            code_pattern,
            organization_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to search accounts by code pattern: {}", e))?;

        Ok(rows
            .into_iter()
            .map(|row| Account {
                id: row.id,
                code: row.code,
                label: row.label,
                parent_code: row.parent_code,
                account_type: row.account_type,
                direct_use: row.direct_use,
                organization_id: row.organization_id,
                created_at: row.created_at.and_utc(),
                updated_at: row.updated_at.and_utc(),
            })
            .collect())
    }

    async fn update(&self, account: &Account) -> Result<Account, String> {
        let result = sqlx::query!(
            r#"
            UPDATE accounts
            SET code = $2, label = $3, parent_code = $4, account_type = $5::account_type,
                direct_use = $6, updated_at = $7
            WHERE id = $1
            RETURNING id, code, label, parent_code, account_type AS "account_type: AccountType",
                      direct_use, organization_id, created_at, updated_at
            "#,
            account.id,
            account.code,
            account.label,
            account.parent_code,
            account.account_type as AccountType,
            account.direct_use,
            account.updated_at
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to update account: {}", e))?;

        Ok(Account {
            id: result.id,
            code: result.code,
            label: result.label,
            parent_code: result.parent_code,
            account_type: result.account_type,
            direct_use: result.direct_use,
            organization_id: result.organization_id,
            created_at: result.created_at.and_utc(),
            updated_at: result.updated_at.and_utc(),
        })
    }

    async fn delete(&self, id: Uuid) -> Result<(), String> {
        // Validation inspired by Noalyss Acc_Plan_SQL::delete()
        // Check if account has children
        let has_children = sqlx::query!(
            "SELECT COUNT(*) as count FROM accounts WHERE parent_code = (SELECT code FROM accounts WHERE id = $1)",
            id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to check for child accounts: {}", e))?;

        if has_children.count.unwrap_or(0) > 0 {
            return Err(
                "Cannot delete account: it has child accounts. Delete children first."
                    .to_string(),
            );
        }

        // Check if account is used in expenses
        let is_used = sqlx::query!(
            "SELECT COUNT(*) as count FROM expenses WHERE account_code = (SELECT code FROM accounts WHERE id = $1)",
            id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to check account usage: {}", e))?;

        if is_used.count.unwrap_or(0) > 0 {
            return Err(
                "Cannot delete account: it is used in expense transactions. Archive instead."
                    .to_string(),
            );
        }

        // Proceed with deletion
        let result = sqlx::query!("DELETE FROM accounts WHERE id = $1", id)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("Failed to delete account: {}", e))?;

        if result.rows_affected() == 0 {
            return Err("Account not found".to_string());
        }

        Ok(())
    }

    async fn exists(&self, code: &str, organization_id: Uuid) -> Result<bool, String> {
        let result = sqlx::query!(
            "SELECT COUNT(*) as count FROM accounts WHERE code = $1 AND organization_id = $2",
            code,
            organization_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to check account existence: {}", e))?;

        Ok(result.count.unwrap_or(0) > 0)
    }

    async fn count_by_organization(&self, organization_id: Uuid) -> Result<i64, String> {
        let result = sqlx::query!(
            "SELECT COUNT(*) as count FROM accounts WHERE organization_id = $1",
            organization_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to count accounts: {}", e))?;

        Ok(result.count.unwrap_or(0))
    }
}
