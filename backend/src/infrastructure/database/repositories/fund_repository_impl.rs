//! PostgreSQL implementation of [`FundRepository`] (issue #635, ADR-0054).
//!
//! All `sqlx::Error` paths are wrapped in `AppError::Database(_)` — no
//! `Result<_, String>` debt (CRITICAL.md #4 / #555).

use crate::application::error::AppError;
use crate::application::ports::FundRepository;
use crate::domain::entities::{Fund, FundKind, FundReassignment};
use crate::infrastructure::database::pool::DbPool;
use async_trait::async_trait;
use sqlx::Row;
use uuid::Uuid;

pub struct PostgresFundRepository {
    pool: DbPool,
}

impl PostgresFundRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    fn kind_str(kind: FundKind) -> &'static str {
        match kind {
            FundKind::WorkingCapital => "working_capital",
            FundKind::Reserve => "reserve",
            FundKind::Earmarked => "earmarked",
        }
    }

    fn row_to_fund(row: &sqlx::postgres::PgRow) -> Fund {
        let kind_str: String = row.get("kind");
        let kind = match kind_str.as_str() {
            "reserve" => FundKind::Reserve,
            "earmarked" => FundKind::Earmarked,
            _ => FundKind::WorkingCapital,
        };
        Fund {
            id: row.get("id"),
            acp_id: row.get("acp_id"),
            kind,
            name: row.get("name"),
            purpose: row.try_get("purpose").ok(),
            target_amount: row.try_get("target_amount").ok(),
            balance: row.get("balance"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    fn row_to_reassignment(row: &sqlx::postgres::PgRow) -> FundReassignment {
        FundReassignment {
            id: row.get("id"),
            fund_id: row.get("fund_id"),
            previous_purpose: row.get("previous_purpose"),
            new_purpose: row.get("new_purpose"),
            amount: row.get("amount"),
            resolution_id: row.get("resolution_id"),
            reassigned_at: row.get("reassigned_at"),
        }
    }
}

#[async_trait]
impl FundRepository for PostgresFundRepository {
    async fn create(&self, fund: &Fund) -> Result<Fund, AppError> {
        sqlx::query(
            r#"
            INSERT INTO funds (
                id, acp_id, kind, name, purpose, target_amount, balance,
                created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
        )
        .bind(fund.id)
        .bind(fund.acp_id)
        .bind(Self::kind_str(fund.kind))
        .bind(&fund.name)
        .bind(&fund.purpose)
        .bind(fund.target_amount)
        .bind(fund.balance)
        .bind(fund.created_at)
        .bind(fund.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(fund.clone())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Fund>, AppError> {
        let row = sqlx::query(
            r#"
            SELECT id, acp_id, kind, name, purpose, target_amount, balance,
                   created_at, updated_at
            FROM funds
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(row.map(|r| Self::row_to_fund(&r)))
    }

    async fn find_by_acp_id(&self, acp_id: Uuid) -> Result<Vec<Fund>, AppError> {
        let rows = sqlx::query(
            r#"
            SELECT id, acp_id, kind, name, purpose, target_amount, balance,
                   created_at, updated_at
            FROM funds
            WHERE acp_id = $1
            ORDER BY created_at ASC
            "#,
        )
        .bind(acp_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(rows.iter().map(Self::row_to_fund).collect())
    }

    async fn update(&self, fund: &Fund) -> Result<Fund, AppError> {
        sqlx::query(
            r#"
            UPDATE funds
            SET name = $2,
                purpose = $3,
                target_amount = $4,
                balance = $5,
                updated_at = $6
            WHERE id = $1
            "#,
        )
        .bind(fund.id)
        .bind(&fund.name)
        .bind(&fund.purpose)
        .bind(fund.target_amount)
        .bind(fund.balance)
        .bind(fund.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(fund.clone())
    }

    async fn record_reassignment(&self, reassignment: &FundReassignment) -> Result<(), AppError> {
        sqlx::query(
            r#"
            INSERT INTO fund_reassignments (
                id, fund_id, previous_purpose, new_purpose, amount,
                resolution_id, reassigned_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
        )
        .bind(reassignment.id)
        .bind(reassignment.fund_id)
        .bind(&reassignment.previous_purpose)
        .bind(&reassignment.new_purpose)
        .bind(reassignment.amount)
        .bind(reassignment.resolution_id)
        .bind(reassignment.reassigned_at)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(())
    }

    async fn list_reassignments(&self, fund_id: Uuid) -> Result<Vec<FundReassignment>, AppError> {
        let rows = sqlx::query(
            r#"
            SELECT id, fund_id, previous_purpose, new_purpose, amount,
                   resolution_id, reassigned_at
            FROM fund_reassignments
            WHERE fund_id = $1
            ORDER BY reassigned_at DESC
            "#,
        )
        .bind(fund_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(rows.iter().map(Self::row_to_reassignment).collect())
    }
}
