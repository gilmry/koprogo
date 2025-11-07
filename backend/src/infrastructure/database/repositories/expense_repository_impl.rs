use crate::application::dto::{ExpenseFilters, PageRequest};
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
            INSERT INTO expenses (id, organization_id, building_id, category, description, amount, expense_date, payment_status, supplier, invoice_number, account_code, created_at, updated_at)
            VALUES ($1, $2, $3, CAST($4 AS expense_category), $5, $6, $7, CAST($8 AS payment_status), $9, $10, $11, $12, $13)
            "#,
        )
        .bind(expense.id)
        .bind(expense.organization_id)
        .bind(expense.building_id)
        .bind(category_str)
        .bind(&expense.description)
        .bind(expense.amount)
        .bind(expense.expense_date)
        .bind(status_str)
        .bind(&expense.supplier)
        .bind(&expense.invoice_number)
        .bind(&expense.account_code)
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
            SELECT id, organization_id, building_id,
                   category::text AS category, description, amount, expense_date,
                   payment_status::text AS payment_status, supplier, invoice_number, account_code, created_at, updated_at
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
                organization_id: row.get("organization_id"),
                building_id: row.get("building_id"),
                category,
                description: row.get("description"),
                amount: row.get("amount"),
                expense_date: row.get("expense_date"),
                payment_status,
                supplier: row.get("supplier"),
                invoice_number: row.get("invoice_number"),
                account_code: row.get("account_code"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            }
        }))
    }

    async fn find_by_building(&self, building_id: Uuid) -> Result<Vec<Expense>, String> {
        let rows = sqlx::query(
            r#"
            SELECT id, organization_id, building_id,
                   category::text AS category, description, amount, expense_date,
                   payment_status::text AS payment_status, supplier, invoice_number, account_code, created_at, updated_at
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
                    organization_id: row.get("organization_id"),
                    building_id: row.get("building_id"),
                    category,
                    description: row.get("description"),
                    amount: row.get("amount"),
                    expense_date: row.get("expense_date"),
                    payment_status,
                    supplier: row.get("supplier"),
                    invoice_number: row.get("invoice_number"),
                    account_code: row.get("account_code"),
                    created_at: row.get("created_at"),
                    updated_at: row.get("updated_at"),
                }
            })
            .collect())
    }

    async fn find_all_paginated(
        &self,
        page_request: &PageRequest,
        filters: &ExpenseFilters,
    ) -> Result<(Vec<Expense>, i64), String> {
        // Validate page request
        page_request.validate()?;

        // Build WHERE clause dynamically
        let mut where_clauses = Vec::new();
        let mut param_count = 0;

        if filters.building_id.is_some() {
            param_count += 1;
            where_clauses.push(format!("building_id = ${}", param_count));
        }

        if filters.category.is_some() {
            param_count += 1;
            where_clauses.push(format!("category = ${}", param_count));
        }

        if filters.status.is_some() {
            param_count += 1;
            where_clauses.push(format!("payment_status = ${}", param_count));
        }

        if filters.date_from.is_some() {
            param_count += 1;
            where_clauses.push(format!("expense_date >= ${}", param_count));
        }

        if filters.date_to.is_some() {
            param_count += 1;
            where_clauses.push(format!("expense_date <= ${}", param_count));
        }

        if filters.min_amount.is_some() {
            param_count += 1;
            where_clauses.push(format!("amount >= ${}", param_count));
        }

        if filters.max_amount.is_some() {
            param_count += 1;
            where_clauses.push(format!("amount <= ${}", param_count));
        }

        let where_clause = if where_clauses.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", where_clauses.join(" AND "))
        };

        // Validate sort column (whitelist)
        let allowed_columns = ["expense_date", "amount", "created_at", "payment_status"];
        let sort_column = page_request.sort_by.as_deref().unwrap_or("expense_date");

        if !allowed_columns.contains(&sort_column) {
            return Err(format!("Invalid sort column: {}", sort_column));
        }

        // Count total items
        let count_query = format!("SELECT COUNT(*) FROM expenses {}", where_clause);
        let mut count_query = sqlx::query_scalar::<_, i64>(&count_query);

        if let Some(building_id) = filters.building_id {
            count_query = count_query.bind(building_id);
        }
        if let Some(category) = &filters.category {
            count_query = count_query.bind(category);
        }
        if let Some(status) = &filters.status {
            count_query = count_query.bind(status);
        }
        if let Some(date_from) = filters.date_from {
            count_query = count_query.bind(date_from);
        }
        if let Some(date_to) = filters.date_to {
            count_query = count_query.bind(date_to);
        }
        if let Some(min_amount) = filters.min_amount {
            count_query = count_query.bind(min_amount);
        }
        if let Some(max_amount) = filters.max_amount {
            count_query = count_query.bind(max_amount);
        }

        let total_items = count_query
            .fetch_one(&self.pool)
            .await
            .map_err(|e| format!("Database error: {}", e))?;

        // Fetch paginated data
        param_count += 1;
        let limit_param = param_count;
        param_count += 1;
        let offset_param = param_count;

        let data_query = format!(
            "SELECT id, organization_id, building_id, category::text AS category, description, amount, expense_date, payment_status::text AS payment_status, supplier, invoice_number, account_code, created_at, updated_at \
             FROM expenses {} ORDER BY {} {} LIMIT ${} OFFSET ${}",
            where_clause,
            sort_column,
            page_request.order.to_sql(),
            limit_param,
            offset_param
        );

        let mut data_query = sqlx::query(&data_query);

        if let Some(building_id) = filters.building_id {
            data_query = data_query.bind(building_id);
        }
        if let Some(category) = &filters.category {
            data_query = data_query.bind(category);
        }
        if let Some(status) = &filters.status {
            data_query = data_query.bind(status);
        }
        if let Some(date_from) = filters.date_from {
            data_query = data_query.bind(date_from);
        }
        if let Some(date_to) = filters.date_to {
            data_query = data_query.bind(date_to);
        }
        if let Some(min_amount) = filters.min_amount {
            data_query = data_query.bind(min_amount);
        }
        if let Some(max_amount) = filters.max_amount {
            data_query = data_query.bind(max_amount);
        }

        data_query = data_query
            .bind(page_request.limit())
            .bind(page_request.offset());

        let rows = data_query
            .fetch_all(&self.pool)
            .await
            .map_err(|e| format!("Database error: {}", e))?;

        let expenses: Vec<Expense> = rows
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
                    organization_id: row.get("organization_id"),
                    building_id: row.get("building_id"),
                    category,
                    description: row.get("description"),
                    amount: row.get("amount"),
                    expense_date: row.get("expense_date"),
                    payment_status,
                    supplier: row.get("supplier"),
                    invoice_number: row.get("invoice_number"),
                    account_code: row.get("account_code"),
                    created_at: row.get("created_at"),
                    updated_at: row.get("updated_at"),
                }
            })
            .collect();

        Ok((expenses, total_items))
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
            SET payment_status = CAST($2 AS payment_status), updated_at = $3
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
