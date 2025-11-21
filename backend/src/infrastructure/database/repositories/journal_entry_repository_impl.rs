// Infrastructure: PostgreSQL Journal Entry Repository Implementation
//
// CREDITS & ATTRIBUTION:
// This implementation is inspired by the Noalyss project (https://gitlab.com/noalyss/noalyss)
// Noalyss is a free accounting software for Belgian and French accounting
// License: GPL-2.0-or-later (GNU General Public License version 2 or later)
// Copyright: (C) 1989, 1991 Free Software Foundation, Inc.
// Copyright: Dany De Bontridder <dany@alchimerys.eu>

use crate::application::ports::JournalEntryRepository;
use crate::domain::entities::{JournalEntry, JournalEntryLine};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use std::collections::HashMap;
use uuid::Uuid;

pub struct PostgresJournalEntryRepository {
    pool: PgPool,
}

impl PostgresJournalEntryRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Load all lines for a journal entry
    async fn load_lines(&self, journal_entry_id: Uuid) -> Result<Vec<JournalEntryLine>, String> {
        let lines = sqlx::query_as!(
            JournalEntryLineRow,
            r#"
            SELECT
                id,
                journal_entry_id,
                organization_id,
                account_code,
                debit,
                credit,
                description,
                created_at
            FROM journal_entry_lines
            WHERE journal_entry_id = $1
            ORDER BY created_at
            "#,
            journal_entry_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to load journal entry lines: {}", e))?;

        Ok(lines.into_iter().map(Into::into).collect())
    }
}

#[async_trait]
impl JournalEntryRepository for PostgresJournalEntryRepository {
    async fn create(&self, entry: &JournalEntry) -> Result<JournalEntry, String> {
        // Validate entry is balanced before inserting
        if !entry.is_balanced() {
            return Err(format!(
                "Journal entry is unbalanced: debits={:.2}€ credits={:.2}€",
                entry.total_debits(),
                entry.total_credits()
            ));
        }

        // Start transaction
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| format!("Failed to begin transaction: {}", e))?;

        // Insert journal entry
        let entry_row = sqlx::query_as!(
            JournalEntryRow,
            r#"
            INSERT INTO journal_entries (
                organization_id, building_id, entry_date, description, document_ref,
                journal_type, expense_id, contribution_id, created_by
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING id, organization_id, building_id, entry_date, description, document_ref,
                      journal_type, expense_id, contribution_id, created_at, updated_at, created_by
            "#,
            entry.organization_id,
            entry.building_id,
            entry.entry_date,
            entry.description,
            entry.document_ref,
            entry.journal_type,
            entry.expense_id,
            entry.contribution_id,
            entry.created_by
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| format!("Failed to insert journal entry: {}", e))?;

        // Insert journal entry lines
        for line in &entry.lines {
            sqlx::query!(
                r#"
                INSERT INTO journal_entry_lines (
                    journal_entry_id, organization_id, account_code,
                    debit, credit, description
                )
                VALUES ($1, $2, $3, $4, $5, $6)
                "#,
                entry_row.id,
                line.organization_id,
                line.account_code,
                rust_decimal::Decimal::from_f64_retain(line.debit).unwrap_or_default(),
                rust_decimal::Decimal::from_f64_retain(line.credit).unwrap_or_default(),
                line.description
            )
            .execute(&mut *tx)
            .await
            .map_err(|e| format!("Failed to insert journal entry line: {}", e))?;
        }

        // Commit transaction (database trigger validates balance)
        tx.commit()
            .await
            .map_err(|e| format!("Failed to commit transaction: {}", e))?;

        // Load lines to return complete entry
        let lines = self.load_lines(entry_row.id).await?;

        Ok(entry_row.into_journal_entry(lines))
    }

    async fn find_by_organization(
        &self,
        organization_id: Uuid,
    ) -> Result<Vec<JournalEntry>, String> {
        let entry_rows = sqlx::query_as!(
            JournalEntryRow,
            r#"
            SELECT
                id, organization_id, building_id, entry_date, description, document_ref,
                journal_type, expense_id, contribution_id, created_at, updated_at, created_by
            FROM journal_entries
            WHERE organization_id = $1
            ORDER BY entry_date DESC, created_at DESC
            "#,
            organization_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to find journal entries: {}", e))?;

        let mut entries = Vec::new();
        for row in entry_rows {
            let lines = self.load_lines(row.id).await?;
            entries.push(row.into_journal_entry(lines));
        }

        Ok(entries)
    }

    async fn find_by_expense(&self, expense_id: Uuid) -> Result<Vec<JournalEntry>, String> {
        let entry_rows = sqlx::query_as!(
            JournalEntryRow,
            r#"
            SELECT
                id, organization_id, building_id, entry_date, description, document_ref,
                journal_type, expense_id, contribution_id, created_at, updated_at, created_by
            FROM journal_entries
            WHERE expense_id = $1
            ORDER BY entry_date DESC, created_at DESC
            "#,
            expense_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to find journal entries for expense: {}", e))?;

        let mut entries = Vec::new();
        for row in entry_rows {
            let lines = self.load_lines(row.id).await?;
            entries.push(row.into_journal_entry(lines));
        }

        Ok(entries)
    }

    async fn find_by_date_range(
        &self,
        organization_id: Uuid,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> Result<Vec<JournalEntry>, String> {
        let entry_rows = sqlx::query_as!(
            JournalEntryRow,
            r#"
            SELECT
                id, organization_id, building_id, entry_date, description, document_ref,
                journal_type, expense_id, contribution_id, created_at, updated_at, created_by
            FROM journal_entries
            WHERE organization_id = $1
              AND entry_date >= $2
              AND entry_date <= $3
            ORDER BY entry_date, created_at
            "#,
            organization_id,
            start_date,
            end_date
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to find journal entries by date range: {}", e))?;

        let mut entries = Vec::new();
        for row in entry_rows {
            let lines = self.load_lines(row.id).await?;
            entries.push(row.into_journal_entry(lines));
        }

        Ok(entries)
    }

    async fn calculate_account_balances(
        &self,
        organization_id: Uuid,
    ) -> Result<HashMap<String, f64>, String> {
        // Use the account_balances view created in migration
        let balances = sqlx::query!(
            r#"
            SELECT account_code, balance
            FROM account_balances
            WHERE organization_id = $1
            "#,
            organization_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to calculate account balances: {}", e))?;

        let mut result = HashMap::new();
        for row in balances {
            if let Some(code) = row.account_code {
                result.insert(
                    code,
                    row.balance
                        .map(|b| b.to_string().parse::<f64>().unwrap_or(0.0))
                        .unwrap_or(0.0),
                );
            }
        }

        Ok(result)
    }

    async fn calculate_account_balances_for_period(
        &self,
        organization_id: Uuid,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> Result<HashMap<String, f64>, String> {
        // Similar to calculate_account_balances but filtered by date
        let balances = sqlx::query!(
            r#"
            SELECT
                jel.account_code,
                a.account_type as "account_type: String",
                SUM(jel.debit) as total_debit,
                SUM(jel.credit) as total_credit
            FROM journal_entry_lines jel
            JOIN journal_entries je ON je.id = jel.journal_entry_id
            JOIN accounts a ON a.organization_id = jel.organization_id
                           AND a.code = jel.account_code
            WHERE jel.organization_id = $1
              AND je.entry_date >= $2
              AND je.entry_date <= $3
            GROUP BY jel.account_code, a.account_type
            "#,
            organization_id,
            start_date,
            end_date
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to calculate account balances for period: {}", e))?;

        let mut result = HashMap::new();
        for row in balances {
            let total_debit = row
                .total_debit
                .map(|d| d.to_string().parse::<f64>().unwrap_or(0.0))
                .unwrap_or(0.0);
            let total_credit = row
                .total_credit
                .map(|c| c.to_string().parse::<f64>().unwrap_or(0.0))
                .unwrap_or(0.0);

            // Calculate balance based on account type
            let balance = match row.account_type.as_str() {
                "ASSET" | "EXPENSE" => total_debit - total_credit,
                "LIABILITY" | "REVENUE" => total_credit - total_debit,
                _ => 0.0,
            };

            result.insert(row.account_code, balance);
        }

        Ok(result)
    }

    async fn find_lines_by_account(
        &self,
        organization_id: Uuid,
        account_code: &str,
    ) -> Result<Vec<JournalEntryLine>, String> {
        let lines = sqlx::query_as!(
            JournalEntryLineRow,
            r#"
            SELECT
                jel.id,
                jel.journal_entry_id,
                jel.organization_id,
                jel.account_code,
                jel.debit,
                jel.credit,
                jel.description,
                jel.created_at
            FROM journal_entry_lines jel
            WHERE jel.organization_id = $1
              AND jel.account_code = $2
            ORDER BY jel.created_at
            "#,
            organization_id,
            account_code
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to find lines by account: {}", e))?;

        Ok(lines.into_iter().map(Into::into).collect())
    }

    async fn validate_balance(&self, entry_id: Uuid) -> Result<bool, String> {
        let result = sqlx::query!(
            r#"
            SELECT
                SUM(debit) as total_debits,
                SUM(credit) as total_credits
            FROM journal_entry_lines
            WHERE journal_entry_id = $1
            "#,
            entry_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to validate balance: {}", e))?;

        let total_debits = result
            .total_debits
            .map(|d| d.to_string().parse::<f64>().unwrap_or(0.0))
            .unwrap_or(0.0);
        let total_credits = result
            .total_credits
            .map(|c| c.to_string().parse::<f64>().unwrap_or(0.0))
            .unwrap_or(0.0);

        Ok((total_debits - total_credits).abs() <= 0.01)
    }

    async fn calculate_account_balances_for_building(
        &self,
        organization_id: Uuid,
        building_id: Uuid,
    ) -> Result<HashMap<String, f64>, String> {
        // Calculate balances from journal entries linked to expenses or contributions for this building
        let rows = sqlx::query!(
            r#"
            SELECT
                jel.account_code,
                SUM(jel.debit) as total_debit,
                SUM(jel.credit) as total_credit
            FROM journal_entry_lines jel
            JOIN journal_entries je ON jel.journal_entry_id = je.id
            LEFT JOIN expenses e ON je.expense_id = e.id
            WHERE jel.organization_id = $1
              AND (e.building_id = $2 OR e.building_id IS NULL)
            GROUP BY jel.account_code
            "#,
            organization_id,
            building_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to calculate building balances: {}", e))?;

        let mut balances = HashMap::new();
        for row in rows {
            let debit = row
                .total_debit
                .map(|d| d.to_string().parse::<f64>().unwrap_or(0.0))
                .unwrap_or(0.0);
            let credit = row
                .total_credit
                .map(|c| c.to_string().parse::<f64>().unwrap_or(0.0))
                .unwrap_or(0.0);

            // For expenses/assets: positive balance = debit - credit
            // For revenue/liabilities: positive balance = credit - debit
            // We'll determine this based on account class
            let account_code = &row.account_code;
            let balance = if account_code.starts_with('6')
                || account_code.starts_with('2')
                || account_code.starts_with('3')
                || account_code.starts_with('4')
                || account_code.starts_with('5')
            {
                debit - credit // Assets/Expenses
            } else {
                credit - debit // Liabilities/Revenue (class 1, 7)
            };

            balances.insert(row.account_code.clone(), balance);
        }

        Ok(balances)
    }

    async fn calculate_account_balances_for_building_and_period(
        &self,
        organization_id: Uuid,
        building_id: Uuid,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> Result<HashMap<String, f64>, String> {
        // Calculate balances from journal entries linked to expenses or contributions for this building and period
        let rows = sqlx::query!(
            r#"
            SELECT
                jel.account_code,
                SUM(jel.debit) as total_debit,
                SUM(jel.credit) as total_credit
            FROM journal_entry_lines jel
            JOIN journal_entries je ON jel.journal_entry_id = je.id
            LEFT JOIN expenses e ON je.expense_id = e.id
            WHERE jel.organization_id = $1
              AND (e.building_id = $2 OR e.building_id IS NULL)
              AND je.entry_date >= $3
              AND je.entry_date <= $4
            GROUP BY jel.account_code
            "#,
            organization_id,
            building_id,
            start_date,
            end_date
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to calculate building period balances: {}", e))?;

        let mut balances = HashMap::new();
        for row in rows {
            let debit = row
                .total_debit
                .map(|d| d.to_string().parse::<f64>().unwrap_or(0.0))
                .unwrap_or(0.0);
            let credit = row
                .total_credit
                .map(|c| c.to_string().parse::<f64>().unwrap_or(0.0))
                .unwrap_or(0.0);

            let account_code = &row.account_code;
            let balance = if account_code.starts_with('6')
                || account_code.starts_with('2')
                || account_code.starts_with('3')
                || account_code.starts_with('4')
                || account_code.starts_with('5')
            {
                debit - credit // Assets/Expenses
            } else {
                credit - debit // Liabilities/Revenue
            };

            balances.insert(row.account_code.clone(), balance);
        }

        Ok(balances)
    }

    async fn create_manual_entry(
        &self,
        entry: &JournalEntry,
        lines: &[JournalEntryLine],
    ) -> Result<(), String> {
        // Start transaction with deferred constraints
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| format!("Failed to begin transaction: {}", e))?;

        sqlx::query("SET CONSTRAINTS ALL DEFERRED")
            .execute(&mut *tx)
            .await
            .map_err(|e| format!("Failed to defer constraints: {}", e))?;

        // Insert journal entry header
        sqlx::query!(
            r#"
            INSERT INTO journal_entries (
                id, organization_id, building_id, entry_date, description,
                document_ref, journal_type, expense_id, contribution_id, created_at, updated_at, created_by
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            "#,
            entry.id,
            entry.organization_id,
            entry.building_id,
            entry.entry_date,
            entry.description,
            entry.document_ref,
            entry.journal_type,
            entry.expense_id,
            entry.contribution_id,
            entry.created_at,
            entry.updated_at,
            entry.created_by
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| format!("Failed to insert journal entry: {}", e))?;

        // Insert journal entry lines
        for line in lines {
            sqlx::query!(
                r#"
                INSERT INTO journal_entry_lines (
                    id, journal_entry_id, organization_id, account_code,
                    debit, credit, description, created_at
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                "#,
                line.id,
                line.journal_entry_id,
                line.organization_id,
                line.account_code,
                rust_decimal::Decimal::from_f64_retain(line.debit).unwrap_or_default(),
                rust_decimal::Decimal::from_f64_retain(line.credit).unwrap_or_default(),
                line.description,
                line.created_at
            )
            .execute(&mut *tx)
            .await
            .map_err(|e| format!("Failed to insert journal entry line: {}", e))?;
        }

        // Commit transaction
        tx.commit()
            .await
            .map_err(|e| format!("Failed to commit transaction: {}", e))?;

        Ok(())
    }

    async fn list_entries(
        &self,
        organization_id: Uuid,
        building_id: Option<Uuid>,
        journal_type: Option<String>,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<JournalEntry>, String> {
        let rows = sqlx::query_as!(
            JournalEntryRow,
            r#"
            SELECT
                id, organization_id, building_id, entry_date, description,
                document_ref, journal_type, expense_id, contribution_id, created_at, updated_at, created_by
            FROM journal_entries
            WHERE organization_id = $1
              AND ($2::uuid IS NULL OR building_id = $2)
              AND ($3::text IS NULL OR journal_type = $3)
              AND ($4::timestamptz IS NULL OR entry_date >= $4)
              AND ($5::timestamptz IS NULL OR entry_date <= $5)
            ORDER BY entry_date DESC, created_at DESC
            LIMIT $6 OFFSET $7
            "#,
            organization_id,
            building_id,
            journal_type,
            start_date,
            end_date,
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to list journal entries: {}", e))?;

        Ok(rows
            .into_iter()
            .map(|row| row.into_journal_entry(vec![]))
            .collect())
    }

    async fn find_by_id(
        &self,
        entry_id: Uuid,
        organization_id: Uuid,
    ) -> Result<JournalEntry, String> {
        let row = sqlx::query_as!(
            JournalEntryRow,
            r#"
            SELECT
                id, organization_id, building_id, entry_date, description,
                document_ref, journal_type, expense_id, contribution_id, created_at, updated_at, created_by
            FROM journal_entries
            WHERE id = $1 AND organization_id = $2
            "#,
            entry_id,
            organization_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Journal entry not found: {}", e))?;

        Ok(row.into_journal_entry(vec![]))
    }

    async fn find_lines_by_entry(
        &self,
        entry_id: Uuid,
        organization_id: Uuid,
    ) -> Result<Vec<JournalEntryLine>, String> {
        let rows = sqlx::query_as!(
            JournalEntryLineRow,
            r#"
            SELECT
                id, journal_entry_id, organization_id, account_code,
                debit, credit, description, created_at
            FROM journal_entry_lines
            WHERE journal_entry_id = $1 AND organization_id = $2
            ORDER BY created_at ASC
            "#,
            entry_id,
            organization_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to fetch journal entry lines: {}", e))?;

        Ok(rows.into_iter().map(JournalEntryLine::from).collect())
    }

    async fn delete_entry(&self, entry_id: Uuid, organization_id: Uuid) -> Result<(), String> {
        // Start transaction
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| format!("Failed to begin transaction: {}", e))?;

        // Delete journal entry lines first (foreign key constraint)
        sqlx::query!(
            r#"
            DELETE FROM journal_entry_lines
            WHERE journal_entry_id = $1 AND organization_id = $2
            "#,
            entry_id,
            organization_id
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| format!("Failed to delete journal entry lines: {}", e))?;

        // Delete journal entry
        let result = sqlx::query!(
            r#"
            DELETE FROM journal_entries
            WHERE id = $1 AND organization_id = $2
            "#,
            entry_id,
            organization_id
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| format!("Failed to delete journal entry: {}", e))?;

        if result.rows_affected() == 0 {
            return Err("Journal entry not found or already deleted".to_string());
        }

        // Commit transaction
        tx.commit()
            .await
            .map_err(|e| format!("Failed to commit transaction: {}", e))?;

        Ok(())
    }
}

// Database row structs
#[derive(Debug)]
struct JournalEntryRow {
    id: Uuid,
    organization_id: Uuid,
    building_id: Option<Uuid>,
    entry_date: DateTime<Utc>,
    description: Option<String>,
    document_ref: Option<String>,
    journal_type: Option<String>,
    expense_id: Option<Uuid>,
    contribution_id: Option<Uuid>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    created_by: Option<Uuid>,
}

impl JournalEntryRow {
    fn into_journal_entry(self, lines: Vec<JournalEntryLine>) -> JournalEntry {
        JournalEntry {
            id: self.id,
            organization_id: self.organization_id,
            building_id: self.building_id,
            entry_date: self.entry_date,
            description: self.description,
            document_ref: self.document_ref,
            journal_type: self.journal_type,
            expense_id: self.expense_id,
            contribution_id: self.contribution_id,
            lines,
            created_at: self.created_at,
            updated_at: self.updated_at,
            created_by: self.created_by,
        }
    }
}

#[derive(Debug)]
struct JournalEntryLineRow {
    id: Uuid,
    journal_entry_id: Uuid,
    organization_id: Uuid,
    account_code: String,
    debit: rust_decimal::Decimal,
    credit: rust_decimal::Decimal,
    description: Option<String>,
    created_at: DateTime<Utc>,
}

impl From<JournalEntryLineRow> for JournalEntryLine {
    fn from(row: JournalEntryLineRow) -> Self {
        Self {
            id: row.id,
            journal_entry_id: row.journal_entry_id,
            organization_id: row.organization_id,
            account_code: row.account_code,
            debit: row.debit.to_string().parse::<f64>().unwrap_or(0.0),
            credit: row.credit.to_string().parse::<f64>().unwrap_or(0.0),
            description: row.description,
            created_at: row.created_at,
        }
    }
}
