use crate::application::ports::ExpenseRepository;
use crate::domain::entities::{Expense, ExpenseCategory, PaymentStatus};
use crate::infrastructure::database::pool::DbPool;
use async_trait::async_trait;
use sqlx::Row;
use uuid::Uuid;

pub struct PostgresExpenseRepository {
    pool: DbPool,
}

impl PostgresExpenseRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ExpenseRepository for PostgresExpenseRepository {
    async fn create(&self, expense: &Expense) -> Result<Expense, String> {
        let category_str = match expense.category {
            ExpenseCategory::Maintenance => "maintenance",
            ExpenseCategory::Repairs => "repairs",
            ExpenseCategory::Insurance => "insurance",
            ExpenseCategory::Utilities => "utilities",
            ExpenseCategory::Cleaning => "cleaning",
            ExpenseCategory::Administration => "administration",
            ExpenseCategory::Works => "works",
            ExpenseCategory::Other => "other",
        };

        let status_str = match expense.payment_status {
            PaymentStatus::Pending => "pending",
            PaymentStatus::Paid => "paid",
            PaymentStatus::Overdue => "overdue",
            PaymentStatus::Cancelled => "cancelled",
        };

        sqlx::query(
            r#"
            INSERT INTO expenses (id, building_id, category, description, amount, expense_date, payment_status, supplier, invoice_number, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            "#,
        )
        .bind(expense.id)
        .bind(expense.building_id)
        .bind(category_str)
        .bind(&expense.description)
        .bind(expense.amount)
        .bind(expense.expense_date)
        .bind(status_str)
        .bind(&expense.supplier)
        .bind(&expense.invoice_number)
        .bind(expense.created_at)
        .bind(expense.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(expense.clone())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Expense>, String> {
        let row = sqlx::query(
            r#"
            SELECT id, building_id, category, description, amount, expense_date, payment_status, supplier, invoice_number, created_at, updated_at
            FROM expenses
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(row.map(|row| {
            let category_str: String = row.get("category");
            let category = match category_str.as_str() {
                "maintenance" => ExpenseCategory::Maintenance,
                "repairs" => ExpenseCategory::Repairs,
                "insurance" => ExpenseCategory::Insurance,
                "utilities" => ExpenseCategory::Utilities,
                "cleaning" => ExpenseCategory::Cleaning,
                "administration" => ExpenseCategory::Administration,
                "works" => ExpenseCategory::Works,
                _ => ExpenseCategory::Other,
            };

            let status_str: String = row.get("payment_status");
            let payment_status = match status_str.as_str() {
                "paid" => PaymentStatus::Paid,
                "overdue" => PaymentStatus::Overdue,
                "cancelled" => PaymentStatus::Cancelled,
                _ => PaymentStatus::Pending,
            };

            Expense {
                id: row.get("id"),
                building_id: row.get("building_id"),
                category,
                description: row.get("description"),
                amount: row.get("amount"),
                expense_date: row.get("expense_date"),
                payment_status,
                supplier: row.get("supplier"),
                invoice_number: row.get("invoice_number"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            }
        }))
    }

    async fn find_by_building(&self, building_id: Uuid) -> Result<Vec<Expense>, String> {
        let rows = sqlx::query(
            r#"
            SELECT id, building_id, category, description, amount, expense_date, payment_status, supplier, invoice_number, created_at, updated_at
            FROM expenses
            WHERE building_id = $1
            ORDER BY expense_date DESC
            "#,
        )
        .bind(building_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(rows
            .iter()
            .map(|row| {
                let category_str: String = row.get("category");
                let category = match category_str.as_str() {
                    "maintenance" => ExpenseCategory::Maintenance,
                    "repairs" => ExpenseCategory::Repairs,
                    "insurance" => ExpenseCategory::Insurance,
                    "utilities" => ExpenseCategory::Utilities,
                    "cleaning" => ExpenseCategory::Cleaning,
                    "administration" => ExpenseCategory::Administration,
                    "works" => ExpenseCategory::Works,
                    _ => ExpenseCategory::Other,
                };

                let status_str: String = row.get("payment_status");
                let payment_status = match status_str.as_str() {
                    "paid" => PaymentStatus::Paid,
                    "overdue" => PaymentStatus::Overdue,
                    "cancelled" => PaymentStatus::Cancelled,
                    _ => PaymentStatus::Pending,
                };

                Expense {
                    id: row.get("id"),
                    building_id: row.get("building_id"),
                    category,
                    description: row.get("description"),
                    amount: row.get("amount"),
                    expense_date: row.get("expense_date"),
                    payment_status,
                    supplier: row.get("supplier"),
                    invoice_number: row.get("invoice_number"),
                    created_at: row.get("created_at"),
                    updated_at: row.get("updated_at"),
                }
            })
            .collect())
    }

    async fn update(&self, expense: &Expense) -> Result<Expense, String> {
        let status_str = match expense.payment_status {
            PaymentStatus::Pending => "pending",
            PaymentStatus::Paid => "paid",
            PaymentStatus::Overdue => "overdue",
            PaymentStatus::Cancelled => "cancelled",
        };

        sqlx::query(
            r#"
            UPDATE expenses
            SET payment_status = $2, updated_at = $3
            WHERE id = $1
            "#,
        )
        .bind(expense.id)
        .bind(status_str)
        .bind(expense.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(expense.clone())
    }

    async fn delete(&self, id: Uuid) -> Result<bool, String> {
        let result = sqlx::query("DELETE FROM expenses WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("Database error: {}", e))?;

        Ok(result.rows_affected() > 0)
    }
}
