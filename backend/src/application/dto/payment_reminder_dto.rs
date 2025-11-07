use crate::domain::entities::{DeliveryMethod, PaymentReminder, ReminderLevel, ReminderStatus};
use serde::{Deserialize, Serialize};
use validator::Validate;

/// DTO for creating a new payment reminder
#[derive(Debug, Deserialize, Validate, Clone)]
pub struct CreatePaymentReminderDto {
    pub organization_id: String,
    pub expense_id: String,
    pub owner_id: String,
    pub level: ReminderLevel,

    #[validate(range(min = 0.01))]
    pub amount_owed: f64,

    pub due_date: String, // ISO 8601 format

    #[validate(range(min = 0))]
    pub days_overdue: i64,
}

/// DTO for payment reminder response
#[derive(Debug, Serialize, Clone)]
pub struct PaymentReminderResponseDto {
    pub id: String,
    pub organization_id: String,
    pub expense_id: String,
    pub owner_id: String,
    pub level: ReminderLevel,
    pub status: ReminderStatus,
    pub amount_owed: f64,
    pub penalty_amount: f64,
    pub total_amount: f64,
    pub due_date: String,
    pub days_overdue: i64,
    pub delivery_method: DeliveryMethod,
    pub sent_date: Option<String>,
    pub opened_date: Option<String>,
    pub pdf_path: Option<String>,
    pub tracking_number: Option<String>,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<PaymentReminder> for PaymentReminderResponseDto {
    fn from(reminder: PaymentReminder) -> Self {
        Self {
            id: reminder.id.to_string(),
            organization_id: reminder.organization_id.to_string(),
            expense_id: reminder.expense_id.to_string(),
            owner_id: reminder.owner_id.to_string(),
            level: reminder.level,
            status: reminder.status,
            amount_owed: reminder.amount_owed,
            penalty_amount: reminder.penalty_amount,
            total_amount: reminder.total_amount,
            due_date: reminder.due_date.to_rfc3339(),
            days_overdue: reminder.days_overdue,
            delivery_method: reminder.delivery_method,
            sent_date: reminder.sent_date.map(|d| d.to_rfc3339()),
            opened_date: reminder.opened_date.map(|d| d.to_rfc3339()),
            pdf_path: reminder.pdf_path,
            tracking_number: reminder.tracking_number,
            notes: reminder.notes,
            created_at: reminder.created_at.to_rfc3339(),
            updated_at: reminder.updated_at.to_rfc3339(),
        }
    }
}

/// DTO for marking reminder as sent
#[derive(Debug, Deserialize, Validate, Clone)]
pub struct MarkReminderSentDto {
    pub pdf_path: Option<String>,
}

/// DTO for escalating a reminder
#[derive(Debug, Deserialize, Clone)]
pub struct EscalateReminderDto {
    pub reason: Option<String>,
}

/// DTO for cancelling a reminder
#[derive(Debug, Deserialize, Validate, Clone)]
pub struct CancelReminderDto {
    #[validate(length(min = 1))]
    pub reason: String,
}

/// DTO for adding tracking number
#[derive(Debug, Deserialize, Validate, Clone)]
pub struct AddTrackingNumberDto {
    #[validate(length(min = 1))]
    pub tracking_number: String,
}

/// DTO for payment recovery dashboard statistics
#[derive(Debug, Serialize, Clone)]
pub struct PaymentRecoveryStatsDto {
    pub total_owed: f64,
    pub total_penalties: f64,
    pub reminder_counts: Vec<ReminderLevelCountDto>,
    pub status_counts: Vec<ReminderStatusCountDto>,
}

#[derive(Debug, Serialize, Clone)]
pub struct ReminderLevelCountDto {
    pub level: ReminderLevel,
    pub count: i64,
}

#[derive(Debug, Serialize, Clone)]
pub struct ReminderStatusCountDto {
    pub status: ReminderStatus,
    pub count: i64,
}

/// DTO for overdue expense without reminder (for automated detection)
#[derive(Debug, Serialize, Clone)]
pub struct OverdueExpenseDto {
    pub expense_id: String,
    pub owner_id: String,
    pub days_overdue: i64,
    pub amount: f64,
    pub recommended_level: ReminderLevel,
}

impl OverdueExpenseDto {
    /// Create DTO with automatically determined reminder level
    pub fn new(expense_id: String, owner_id: String, days_overdue: i64, amount: f64) -> Self {
        let recommended_level = if days_overdue >= 60 {
            ReminderLevel::FormalNotice
        } else if days_overdue >= 30 {
            ReminderLevel::SecondReminder
        } else {
            ReminderLevel::FirstReminder
        };

        Self {
            expense_id,
            owner_id,
            days_overdue,
            amount,
            recommended_level,
        }
    }
}

/// DTO for bulk reminder creation
#[derive(Debug, Deserialize, Validate, Clone)]
pub struct BulkCreateRemindersDto {
    pub organization_id: String,

    #[validate(range(min = 15))]
    pub min_days_overdue: i64,
}

/// DTO for bulk reminder creation response
#[derive(Debug, Serialize, Clone)]
pub struct BulkCreateRemindersResponseDto {
    pub created_count: i32,
    pub skipped_count: i32,
    pub errors: Vec<String>,
    pub created_reminders: Vec<PaymentReminderResponseDto>,
}
