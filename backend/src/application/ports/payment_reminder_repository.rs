use crate::domain::entities::{PaymentReminder, ReminderLevel, ReminderStatus};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Repository trait for payment reminder persistence operations
#[async_trait]
pub trait PaymentReminderRepository: Send + Sync {
    /// Create a new payment reminder
    async fn create(&self, reminder: &PaymentReminder) -> Result<PaymentReminder, String>;

    /// Find a reminder by ID
    async fn find_by_id(&self, id: Uuid) -> Result<Option<PaymentReminder>, String>;

    /// Find all reminders for a specific expense
    async fn find_by_expense(&self, expense_id: Uuid) -> Result<Vec<PaymentReminder>, String>;

    /// Find all reminders for a specific owner
    async fn find_by_owner(&self, owner_id: Uuid) -> Result<Vec<PaymentReminder>, String>;

    /// Find all reminders for an organization
    async fn find_by_organization(&self, organization_id: Uuid)
        -> Result<Vec<PaymentReminder>, String>;

    /// Find all reminders with a specific status
    async fn find_by_status(&self, status: ReminderStatus) -> Result<Vec<PaymentReminder>, String>;

    /// Find all reminders with a specific status for an organization
    async fn find_by_organization_and_status(
        &self,
        organization_id: Uuid,
        status: ReminderStatus,
    ) -> Result<Vec<PaymentReminder>, String>;

    /// Find all pending reminders that should be sent (status = Pending)
    async fn find_pending_reminders(&self) -> Result<Vec<PaymentReminder>, String>;

    /// Find all sent reminders that need escalation (sent > 15 days ago)
    async fn find_reminders_needing_escalation(
        &self,
        cutoff_date: DateTime<Utc>,
    ) -> Result<Vec<PaymentReminder>, String>;

    /// Find the latest reminder for a specific expense
    async fn find_latest_by_expense(
        &self,
        expense_id: Uuid,
    ) -> Result<Option<PaymentReminder>, String>;

    /// Find all active (non-paid, non-cancelled) reminders for an owner
    async fn find_active_by_owner(&self, owner_id: Uuid) -> Result<Vec<PaymentReminder>, String>;

    /// Get statistics: count reminders by status for an organization
    async fn count_by_status(
        &self,
        organization_id: Uuid,
    ) -> Result<Vec<(ReminderStatus, i64)>, String>;

    /// Get statistics: total amount owed by organization
    async fn get_total_owed_by_organization(
        &self,
        organization_id: Uuid,
    ) -> Result<f64, String>;

    /// Get statistics: total penalties by organization
    async fn get_total_penalties_by_organization(
        &self,
        organization_id: Uuid,
    ) -> Result<f64, String>;

    /// Get overdue expenses without reminders (for automated detection)
    /// Returns list of (expense_id, owner_id, days_overdue, amount)
    async fn find_overdue_expenses_without_reminders(
        &self,
        organization_id: Uuid,
        min_days_overdue: i64,
    ) -> Result<Vec<(Uuid, Uuid, i64, f64)>, String>;

    /// Update a reminder
    async fn update(&self, reminder: &PaymentReminder) -> Result<PaymentReminder, String>;

    /// Delete a reminder
    async fn delete(&self, id: Uuid) -> Result<bool, String>;

    /// Get payment recovery dashboard data for an organization
    /// Returns: (total_owed, total_penalties, reminder_count_by_level)
    async fn get_dashboard_stats(
        &self,
        organization_id: Uuid,
    ) -> Result<(f64, f64, Vec<(ReminderLevel, i64)>), String>;
}
