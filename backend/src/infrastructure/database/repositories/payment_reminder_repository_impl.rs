use crate::application::ports::PaymentReminderRepository;
use crate::domain::entities::{DeliveryMethod, PaymentReminder, ReminderLevel, ReminderStatus};
use crate::infrastructure::database::pool::DbPool;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::Row;
use uuid::Uuid;

pub struct PostgresPaymentReminderRepository {
    pool: DbPool,
}

impl PostgresPaymentReminderRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    /// Helper function to convert ReminderLevel enum to string for SQL
    fn level_to_str(level: &ReminderLevel) -> &'static str {
        match level {
            ReminderLevel::FirstReminder => "FirstReminder",
            ReminderLevel::SecondReminder => "SecondReminder",
            ReminderLevel::FormalNotice => "FormalNotice",
        }
    }

    /// Helper function to convert string to ReminderLevel enum
    fn str_to_level(s: &str) -> ReminderLevel {
        match s {
            "SecondReminder" => ReminderLevel::SecondReminder,
            "FormalNotice" => ReminderLevel::FormalNotice,
            _ => ReminderLevel::FirstReminder,
        }
    }

    /// Helper function to convert ReminderStatus enum to string for SQL
    fn status_to_str(status: &ReminderStatus) -> &'static str {
        match status {
            ReminderStatus::Pending => "Pending",
            ReminderStatus::Sent => "Sent",
            ReminderStatus::Opened => "Opened",
            ReminderStatus::Paid => "Paid",
            ReminderStatus::Escalated => "Escalated",
            ReminderStatus::Cancelled => "Cancelled",
        }
    }

    /// Helper function to convert string to ReminderStatus enum
    fn str_to_status(s: &str) -> ReminderStatus {
        match s {
            "Sent" => ReminderStatus::Sent,
            "Opened" => ReminderStatus::Opened,
            "Paid" => ReminderStatus::Paid,
            "Escalated" => ReminderStatus::Escalated,
            "Cancelled" => ReminderStatus::Cancelled,
            _ => ReminderStatus::Pending,
        }
    }

    /// Helper function to convert DeliveryMethod enum to string for SQL
    fn delivery_method_to_str(method: &DeliveryMethod) -> &'static str {
        match method {
            DeliveryMethod::Email => "Email",
            DeliveryMethod::RegisteredLetter => "RegisteredLetter",
            DeliveryMethod::Bailiff => "Bailiff",
        }
    }

    /// Helper function to convert string to DeliveryMethod enum
    fn str_to_delivery_method(s: &str) -> DeliveryMethod {
        match s {
            "RegisteredLetter" => DeliveryMethod::RegisteredLetter,
            "Bailiff" => DeliveryMethod::Bailiff,
            _ => DeliveryMethod::Email,
        }
    }

    /// Helper function to map SQL row to PaymentReminder entity
    fn row_to_reminder(&self, row: &sqlx::postgres::PgRow) -> PaymentReminder {
        let level_str: String = row.get("level");
        let status_str: String = row.get("status");
        let delivery_method_str: String = row.get("delivery_method");

        PaymentReminder {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            expense_id: row.get("expense_id"),
            owner_id: row.get("owner_id"),
            level: Self::str_to_level(&level_str),
            status: Self::str_to_status(&status_str),
            amount_owed: row.get("amount_owed"),
            penalty_amount: row.get("penalty_amount"),
            total_amount: row.get("total_amount"),
            due_date: row.get("due_date"),
            days_overdue: row.get::<i32, _>("days_overdue") as i64,
            delivery_method: Self::str_to_delivery_method(&delivery_method_str),
            sent_date: row.get("sent_date"),
            opened_date: row.get("opened_date"),
            pdf_path: row.get("pdf_path"),
            tracking_number: row.get("tracking_number"),
            notes: row.get("notes"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }
}

#[async_trait]
impl PaymentReminderRepository for PostgresPaymentReminderRepository {
    async fn create(&self, reminder: &PaymentReminder) -> Result<PaymentReminder, String> {
        sqlx::query(
            r#"
            INSERT INTO payment_reminders (
                id, organization_id, expense_id, owner_id, level, status,
                amount_owed, penalty_amount, total_amount, due_date, days_overdue,
                delivery_method, sent_date, opened_date, pdf_path, tracking_number, notes,
                created_at, updated_at
            )
            VALUES (
                $1, $2, $3, $4, CAST($5 AS reminder_level), CAST($6 AS reminder_status),
                $7, $8, $9, $10, $11,
                CAST($12 AS delivery_method), $13, $14, $15, $16, $17,
                $18, $19
            )
            "#,
        )
        .bind(reminder.id)
        .bind(reminder.organization_id)
        .bind(reminder.expense_id)
        .bind(reminder.owner_id)
        .bind(Self::level_to_str(&reminder.level))
        .bind(Self::status_to_str(&reminder.status))
        .bind(reminder.amount_owed)
        .bind(reminder.penalty_amount)
        .bind(reminder.total_amount)
        .bind(reminder.due_date)
        .bind(reminder.days_overdue as i32)
        .bind(Self::delivery_method_to_str(&reminder.delivery_method))
        .bind(reminder.sent_date)
        .bind(reminder.opened_date)
        .bind(&reminder.pdf_path)
        .bind(&reminder.tracking_number)
        .bind(&reminder.notes)
        .bind(reminder.created_at)
        .bind(reminder.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Database error creating reminder: {}", e))?;

        Ok(reminder.clone())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<PaymentReminder>, String> {
        let row = sqlx::query(
            r#"
            SELECT id, organization_id, expense_id, owner_id,
                   level::text AS level, status::text AS status,
                   amount_owed, penalty_amount, total_amount, due_date, days_overdue,
                   delivery_method::text AS delivery_method,
                   sent_date, opened_date, pdf_path, tracking_number, notes,
                   created_at, updated_at
            FROM payment_reminders
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Database error finding reminder: {}", e))?;

        Ok(row.as_ref().map(|r| self.row_to_reminder(r)))
    }

    async fn find_by_expense(&self, expense_id: Uuid) -> Result<Vec<PaymentReminder>, String> {
        let rows = sqlx::query(
            r#"
            SELECT id, organization_id, expense_id, owner_id,
                   level::text AS level, status::text AS status,
                   amount_owed, penalty_amount, total_amount, due_date, days_overdue,
                   delivery_method::text AS delivery_method,
                   sent_date, opened_date, pdf_path, tracking_number, notes,
                   created_at, updated_at
            FROM payment_reminders
            WHERE expense_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(expense_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Database error finding reminders by expense: {}", e))?;

        Ok(rows.iter().map(|r| self.row_to_reminder(r)).collect())
    }

    async fn find_by_owner(&self, owner_id: Uuid) -> Result<Vec<PaymentReminder>, String> {
        let rows = sqlx::query(
            r#"
            SELECT id, organization_id, expense_id, owner_id,
                   level::text AS level, status::text AS status,
                   amount_owed, penalty_amount, total_amount, due_date, days_overdue,
                   delivery_method::text AS delivery_method,
                   sent_date, opened_date, pdf_path, tracking_number, notes,
                   created_at, updated_at
            FROM payment_reminders
            WHERE owner_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(owner_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Database error finding reminders by owner: {}", e))?;

        Ok(rows.iter().map(|r| self.row_to_reminder(r)).collect())
    }

    async fn find_by_organization(
        &self,
        organization_id: Uuid,
    ) -> Result<Vec<PaymentReminder>, String> {
        let rows = sqlx::query(
            r#"
            SELECT id, organization_id, expense_id, owner_id,
                   level::text AS level, status::text AS status,
                   amount_owed, penalty_amount, total_amount, due_date, days_overdue,
                   delivery_method::text AS delivery_method,
                   sent_date, opened_date, pdf_path, tracking_number, notes,
                   created_at, updated_at
            FROM payment_reminders
            WHERE organization_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(organization_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Database error finding reminders by organization: {}", e))?;

        Ok(rows.iter().map(|r| self.row_to_reminder(r)).collect())
    }

    async fn find_by_status(&self, status: ReminderStatus) -> Result<Vec<PaymentReminder>, String> {
        let rows = sqlx::query(
            r#"
            SELECT id, organization_id, expense_id, owner_id,
                   level::text AS level, status::text AS status,
                   amount_owed, penalty_amount, total_amount, due_date, days_overdue,
                   delivery_method::text AS delivery_method,
                   sent_date, opened_date, pdf_path, tracking_number, notes,
                   created_at, updated_at
            FROM payment_reminders
            WHERE status = CAST($1 AS reminder_status)
            ORDER BY created_at DESC
            "#,
        )
        .bind(Self::status_to_str(&status))
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Database error finding reminders by status: {}", e))?;

        Ok(rows.iter().map(|r| self.row_to_reminder(r)).collect())
    }

    async fn find_by_organization_and_status(
        &self,
        organization_id: Uuid,
        status: ReminderStatus,
    ) -> Result<Vec<PaymentReminder>, String> {
        let rows = sqlx::query(
            r#"
            SELECT id, organization_id, expense_id, owner_id,
                   level::text AS level, status::text AS status,
                   amount_owed, penalty_amount, total_amount, due_date, days_overdue,
                   delivery_method::text AS delivery_method,
                   sent_date, opened_date, pdf_path, tracking_number, notes,
                   created_at, updated_at
            FROM payment_reminders
            WHERE organization_id = $1 AND status = CAST($2 AS reminder_status)
            ORDER BY created_at DESC
            "#,
        )
        .bind(organization_id)
        .bind(Self::status_to_str(&status))
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Database error finding reminders: {}", e))?;

        Ok(rows.iter().map(|r| self.row_to_reminder(r)).collect())
    }

    async fn find_pending_reminders(&self) -> Result<Vec<PaymentReminder>, String> {
        let rows = sqlx::query(
            r#"
            SELECT id, organization_id, expense_id, owner_id,
                   level::text AS level, status::text AS status,
                   amount_owed, penalty_amount, total_amount, due_date, days_overdue,
                   delivery_method::text AS delivery_method,
                   sent_date, opened_date, pdf_path, tracking_number, notes,
                   created_at, updated_at
            FROM payment_reminders
            WHERE status = 'Pending'::reminder_status
            ORDER BY created_at ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Database error finding pending reminders: {}", e))?;

        Ok(rows.iter().map(|r| self.row_to_reminder(r)).collect())
    }

    async fn find_reminders_needing_escalation(
        &self,
        cutoff_date: DateTime<Utc>,
    ) -> Result<Vec<PaymentReminder>, String> {
        let rows = sqlx::query(
            r#"
            SELECT id, organization_id, expense_id, owner_id,
                   level::text AS level, status::text AS status,
                   amount_owed, penalty_amount, total_amount, due_date, days_overdue,
                   delivery_method::text AS delivery_method,
                   sent_date, opened_date, pdf_path, tracking_number, notes,
                   created_at, updated_at
            FROM payment_reminders
            WHERE status IN ('Sent'::reminder_status, 'Opened'::reminder_status)
              AND sent_date <= $1
              AND level != 'FormalNotice'::reminder_level
            ORDER BY sent_date ASC
            "#,
        )
        .bind(cutoff_date)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Database error finding reminders needing escalation: {}", e))?;

        Ok(rows.iter().map(|r| self.row_to_reminder(r)).collect())
    }

    async fn find_latest_by_expense(
        &self,
        expense_id: Uuid,
    ) -> Result<Option<PaymentReminder>, String> {
        let row = sqlx::query(
            r#"
            SELECT id, organization_id, expense_id, owner_id,
                   level::text AS level, status::text AS status,
                   amount_owed, penalty_amount, total_amount, due_date, days_overdue,
                   delivery_method::text AS delivery_method,
                   sent_date, opened_date, pdf_path, tracking_number, notes,
                   created_at, updated_at
            FROM payment_reminders
            WHERE expense_id = $1
            ORDER BY created_at DESC
            LIMIT 1
            "#,
        )
        .bind(expense_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Database error finding latest reminder: {}", e))?;

        Ok(row.as_ref().map(|r| self.row_to_reminder(r)))
    }

    async fn find_active_by_owner(&self, owner_id: Uuid) -> Result<Vec<PaymentReminder>, String> {
        let rows = sqlx::query(
            r#"
            SELECT id, organization_id, expense_id, owner_id,
                   level::text AS level, status::text AS status,
                   amount_owed, penalty_amount, total_amount, due_date, days_overdue,
                   delivery_method::text AS delivery_method,
                   sent_date, opened_date, pdf_path, tracking_number, notes,
                   created_at, updated_at
            FROM payment_reminders
            WHERE owner_id = $1
              AND status NOT IN ('Paid'::reminder_status, 'Cancelled'::reminder_status)
            ORDER BY created_at DESC
            "#,
        )
        .bind(owner_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Database error finding active reminders: {}", e))?;

        Ok(rows.iter().map(|r| self.row_to_reminder(r)).collect())
    }

    async fn count_by_status(
        &self,
        organization_id: Uuid,
    ) -> Result<Vec<(ReminderStatus, i64)>, String> {
        let rows = sqlx::query(
            r#"
            SELECT status::text AS status, COUNT(*) as count
            FROM payment_reminders
            WHERE organization_id = $1
            GROUP BY status
            "#,
        )
        .bind(organization_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Database error counting reminders by status: {}", e))?;

        Ok(rows
            .iter()
            .map(|row| {
                let status_str: String = row.get("status");
                let count: i64 = row.get("count");
                (Self::str_to_status(&status_str), count)
            })
            .collect())
    }

    async fn get_total_owed_by_organization(&self, organization_id: Uuid) -> Result<f64, String> {
        let row = sqlx::query(
            r#"
            SELECT COALESCE(SUM(amount_owed), 0.0) as total
            FROM payment_reminders
            WHERE organization_id = $1
              AND status NOT IN ('Paid'::reminder_status, 'Cancelled'::reminder_status)
            "#,
        )
        .bind(organization_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Database error calculating total owed: {}", e))?;

        Ok(row.get("total"))
    }

    async fn get_total_penalties_by_organization(
        &self,
        organization_id: Uuid,
    ) -> Result<f64, String> {
        let row = sqlx::query(
            r#"
            SELECT COALESCE(SUM(penalty_amount), 0.0) as total
            FROM payment_reminders
            WHERE organization_id = $1
              AND status NOT IN ('Paid'::reminder_status, 'Cancelled'::reminder_status)
            "#,
        )
        .bind(organization_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Database error calculating total penalties: {}", e))?;

        Ok(row.get("total"))
    }

    async fn find_overdue_expenses_without_reminders(
        &self,
        organization_id: Uuid,
        min_days_overdue: i64,
    ) -> Result<Vec<(Uuid, Uuid, i64, f64)>, String> {
        let rows = sqlx::query(
            r#"
            SELECT
                e.id as expense_id,
                uo.owner_id,
                EXTRACT(DAY FROM (NOW() - e.expense_date))::bigint as days_overdue,
                e.amount
            FROM expenses e
            INNER JOIN units u ON e.building_id = (SELECT building_id FROM units WHERE id = u.id LIMIT 1)
            INNER JOIN unit_owners uo ON u.id = uo.unit_id AND uo.end_date IS NULL
            WHERE e.organization_id = $1
              AND e.payment_status = 'overdue'::payment_status
              AND EXTRACT(DAY FROM (NOW() - e.expense_date)) >= $2
              AND NOT EXISTS (
                  SELECT 1 FROM payment_reminders pr
                  WHERE pr.expense_id = e.id
                    AND pr.owner_id = uo.owner_id
                    AND pr.status NOT IN ('Paid'::reminder_status, 'Cancelled'::reminder_status)
              )
            ORDER BY days_overdue DESC
            "#,
        )
        .bind(organization_id)
        .bind(min_days_overdue as i32)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Database error finding overdue expenses: {}", e))?;

        Ok(rows
            .iter()
            .map(|row| {
                let expense_id: Uuid = row.get("expense_id");
                let owner_id: Uuid = row.get("owner_id");
                let days_overdue: i64 = row.get("days_overdue");
                let amount: f64 = row.get("amount");
                (expense_id, owner_id, days_overdue, amount)
            })
            .collect())
    }

    async fn update(&self, reminder: &PaymentReminder) -> Result<PaymentReminder, String> {
        sqlx::query(
            r#"
            UPDATE payment_reminders
            SET status = CAST($2 AS reminder_status),
                amount_owed = $3,
                penalty_amount = $4,
                total_amount = $5,
                days_overdue = $6,
                sent_date = $7,
                opened_date = $8,
                pdf_path = $9,
                tracking_number = $10,
                notes = $11,
                updated_at = $12
            WHERE id = $1
            "#,
        )
        .bind(reminder.id)
        .bind(Self::status_to_str(&reminder.status))
        .bind(reminder.amount_owed)
        .bind(reminder.penalty_amount)
        .bind(reminder.total_amount)
        .bind(reminder.days_overdue as i32)
        .bind(reminder.sent_date)
        .bind(reminder.opened_date)
        .bind(&reminder.pdf_path)
        .bind(&reminder.tracking_number)
        .bind(&reminder.notes)
        .bind(reminder.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Database error updating reminder: {}", e))?;

        Ok(reminder.clone())
    }

    async fn delete(&self, id: Uuid) -> Result<bool, String> {
        let result = sqlx::query("DELETE FROM payment_reminders WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("Database error deleting reminder: {}", e))?;

        Ok(result.rows_affected() > 0)
    }

    async fn get_dashboard_stats(
        &self,
        organization_id: Uuid,
    ) -> Result<(f64, f64, Vec<(ReminderLevel, i64)>), String> {
        let total_owed = self.get_total_owed_by_organization(organization_id).await?;
        let total_penalties = self
            .get_total_penalties_by_organization(organization_id)
            .await?;

        let rows = sqlx::query(
            r#"
            SELECT level::text AS level, COUNT(*) as count
            FROM payment_reminders
            WHERE organization_id = $1
              AND status NOT IN ('Paid'::reminder_status, 'Cancelled'::reminder_status)
            GROUP BY level
            "#,
        )
        .bind(organization_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Database error getting dashboard stats: {}", e))?;

        let level_counts = rows
            .iter()
            .map(|row| {
                let level_str: String = row.get("level");
                let count: i64 = row.get("count");
                (Self::str_to_level(&level_str), count)
            })
            .collect();

        Ok((total_owed, total_penalties, level_counts))
    }
}
