use crate::application::dto::{
    AddTrackingNumberDto, BulkCreateRemindersDto, BulkCreateRemindersResponseDto,
    CancelReminderDto, CreatePaymentReminderDto, EscalateReminderDto, MarkReminderSentDto,
    OverdueExpenseDto, PaymentRecoveryStatsDto, PaymentReminderResponseDto, ReminderLevelCountDto,
    ReminderStatusCountDto,
};
use crate::application::ports::{ExpenseRepository, PaymentReminderRepository};
use crate::domain::entities::{PaymentReminder, PaymentStatus, ReminderStatus};
use chrono::{DateTime, Utc};
use std::sync::Arc;
use uuid::Uuid;

pub struct PaymentReminderUseCases {
    reminder_repository: Arc<dyn PaymentReminderRepository>,
    expense_repository: Arc<dyn ExpenseRepository>,
}

impl PaymentReminderUseCases {
    pub fn new(
        reminder_repository: Arc<dyn PaymentReminderRepository>,
        expense_repository: Arc<dyn ExpenseRepository>,
    ) -> Self {
        Self {
            reminder_repository,
            expense_repository,
        }
    }

    /// Create a new payment reminder
    pub async fn create_reminder(
        &self,
        dto: CreatePaymentReminderDto,
    ) -> Result<PaymentReminderResponseDto, String> {
        let organization_id = Uuid::parse_str(&dto.organization_id)
            .map_err(|_| "Invalid organization_id format".to_string())?;
        let expense_id = Uuid::parse_str(&dto.expense_id)
            .map_err(|_| "Invalid expense_id format".to_string())?;
        let owner_id =
            Uuid::parse_str(&dto.owner_id).map_err(|_| "Invalid owner_id format".to_string())?;

        let due_date = DateTime::parse_from_rfc3339(&dto.due_date)
            .map_err(|_| "Invalid date format".to_string())?
            .with_timezone(&Utc);

        // Verify expense exists and is not paid
        let expense = self
            .expense_repository
            .find_by_id(expense_id)
            .await?
            .ok_or_else(|| "Expense not found".to_string())?;

        if expense.payment_status == PaymentStatus::Paid {
            return Err("Cannot create reminder for paid expense".to_string());
        }

        // Check if reminder already exists for this expense and owner at this level
        let existing_reminders = self.reminder_repository.find_by_expense(expense_id).await?;

        if existing_reminders.iter().any(|r| {
            r.owner_id == owner_id
                && r.level == dto.level
                && r.status != ReminderStatus::Cancelled
                && r.status != ReminderStatus::Paid
        }) {
            return Err(format!(
                "Active reminder already exists for this expense at {:?} level",
                dto.level
            ));
        }

        let reminder = PaymentReminder::new(
            organization_id,
            expense_id,
            owner_id,
            dto.level,
            dto.amount_owed,
            due_date,
            dto.days_overdue,
        )?;

        let created = self.reminder_repository.create(&reminder).await?;
        Ok(created.into())
    }

    /// Get reminder by ID
    pub async fn get_reminder(
        &self,
        id: Uuid,
    ) -> Result<Option<PaymentReminderResponseDto>, String> {
        let reminder = self.reminder_repository.find_by_id(id).await?;
        Ok(reminder.map(|r| r.into()))
    }

    /// List all reminders for an expense
    pub async fn list_by_expense(
        &self,
        expense_id: Uuid,
    ) -> Result<Vec<PaymentReminderResponseDto>, String> {
        let reminders = self.reminder_repository.find_by_expense(expense_id).await?;
        Ok(reminders.into_iter().map(|r| r.into()).collect())
    }

    /// List all reminders for an owner
    pub async fn list_by_owner(
        &self,
        owner_id: Uuid,
    ) -> Result<Vec<PaymentReminderResponseDto>, String> {
        let reminders = self.reminder_repository.find_by_owner(owner_id).await?;
        Ok(reminders.into_iter().map(|r| r.into()).collect())
    }

    /// List all reminders for an organization
    pub async fn list_by_organization(
        &self,
        organization_id: Uuid,
    ) -> Result<Vec<PaymentReminderResponseDto>, String> {
        let reminders = self
            .reminder_repository
            .find_by_organization(organization_id)
            .await?;
        Ok(reminders.into_iter().map(|r| r.into()).collect())
    }

    /// List active (non-paid, non-cancelled) reminders for an owner
    pub async fn list_active_by_owner(
        &self,
        owner_id: Uuid,
    ) -> Result<Vec<PaymentReminderResponseDto>, String> {
        let reminders = self
            .reminder_repository
            .find_active_by_owner(owner_id)
            .await?;
        Ok(reminders.into_iter().map(|r| r.into()).collect())
    }

    /// Mark reminder as sent
    pub async fn mark_as_sent(
        &self,
        id: Uuid,
        dto: MarkReminderSentDto,
    ) -> Result<PaymentReminderResponseDto, String> {
        let mut reminder = self
            .reminder_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| "Reminder not found".to_string())?;

        reminder.mark_as_sent(dto.pdf_path)?;

        let updated = self.reminder_repository.update(&reminder).await?;
        Ok(updated.into())
    }

    /// Mark reminder as opened (email opened)
    pub async fn mark_as_opened(&self, id: Uuid) -> Result<PaymentReminderResponseDto, String> {
        let mut reminder = self
            .reminder_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| "Reminder not found".to_string())?;

        reminder.mark_as_opened()?;

        let updated = self.reminder_repository.update(&reminder).await?;
        Ok(updated.into())
    }

    /// Mark reminder as paid
    pub async fn mark_as_paid(&self, id: Uuid) -> Result<PaymentReminderResponseDto, String> {
        let mut reminder = self
            .reminder_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| "Reminder not found".to_string())?;

        reminder.mark_as_paid()?;

        let updated = self.reminder_repository.update(&reminder).await?;
        Ok(updated.into())
    }

    /// Cancel a reminder
    pub async fn cancel_reminder(
        &self,
        id: Uuid,
        dto: CancelReminderDto,
    ) -> Result<PaymentReminderResponseDto, String> {
        let mut reminder = self
            .reminder_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| "Reminder not found".to_string())?;

        reminder.cancel(dto.reason)?;

        let updated = self.reminder_repository.update(&reminder).await?;
        Ok(updated.into())
    }

    /// Escalate a reminder to next level
    pub async fn escalate_reminder(
        &self,
        id: Uuid,
        _dto: EscalateReminderDto,
    ) -> Result<Option<PaymentReminderResponseDto>, String> {
        let mut reminder = self
            .reminder_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| "Reminder not found".to_string())?;

        let next_level = reminder.escalate()?;

        let updated = self.reminder_repository.update(&reminder).await?;

        // If there's a next level, optionally create a new reminder automatically
        if let Some(level) = next_level {
            // Calculate new days overdue based on next level
            let days_overdue = (Utc::now() - reminder.due_date).num_days();

            // Create next level reminder
            let next_reminder = PaymentReminder::new(
                reminder.organization_id,
                reminder.expense_id,
                reminder.owner_id,
                level,
                reminder.amount_owed,
                reminder.due_date,
                days_overdue,
            )?;

            let created = self.reminder_repository.create(&next_reminder).await?;
            return Ok(Some(created.into()));
        }

        Ok(Some(updated.into()))
    }

    /// Add tracking number to a reminder (for registered letters)
    pub async fn add_tracking_number(
        &self,
        id: Uuid,
        dto: AddTrackingNumberDto,
    ) -> Result<PaymentReminderResponseDto, String> {
        let mut reminder = self
            .reminder_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| "Reminder not found".to_string())?;

        reminder.set_tracking_number(dto.tracking_number)?;

        let updated = self.reminder_repository.update(&reminder).await?;
        Ok(updated.into())
    }

    /// Find all pending reminders (to be sent)
    pub async fn find_pending_reminders(&self) -> Result<Vec<PaymentReminderResponseDto>, String> {
        let reminders = self.reminder_repository.find_pending_reminders().await?;
        Ok(reminders.into_iter().map(|r| r.into()).collect())
    }

    /// Find reminders needing escalation (sent >15 days ago)
    pub async fn find_reminders_needing_escalation(
        &self,
    ) -> Result<Vec<PaymentReminderResponseDto>, String> {
        let cutoff_date = Utc::now() - chrono::Duration::days(15);
        let reminders = self
            .reminder_repository
            .find_reminders_needing_escalation(cutoff_date)
            .await?;

        // Filter to only those that actually need escalation
        let needs_escalation: Vec<PaymentReminder> = reminders
            .into_iter()
            .filter(|r| r.needs_escalation(Utc::now()))
            .collect();

        Ok(needs_escalation.into_iter().map(|r| r.into()).collect())
    }

    /// Get payment recovery statistics for an organization
    pub async fn get_recovery_stats(
        &self,
        organization_id: Uuid,
    ) -> Result<PaymentRecoveryStatsDto, String> {
        let (total_owed, total_penalties, level_counts) = self
            .reminder_repository
            .get_dashboard_stats(organization_id)
            .await?;

        let status_counts = self
            .reminder_repository
            .count_by_status(organization_id)
            .await?;

        Ok(PaymentRecoveryStatsDto {
            total_owed,
            total_penalties,
            reminder_counts: level_counts
                .into_iter()
                .map(|(level, count)| ReminderLevelCountDto { level, count })
                .collect(),
            status_counts: status_counts
                .into_iter()
                .map(|(status, count)| ReminderStatusCountDto { status, count })
                .collect(),
        })
    }

    /// Find overdue expenses without reminders (for automated detection)
    pub async fn find_overdue_expenses_without_reminders(
        &self,
        organization_id: Uuid,
        min_days_overdue: i64,
    ) -> Result<Vec<OverdueExpenseDto>, String> {
        let results = self
            .reminder_repository
            .find_overdue_expenses_without_reminders(organization_id, min_days_overdue)
            .await?;

        Ok(results
            .into_iter()
            .map(|(expense_id, owner_id, days_overdue, amount)| {
                OverdueExpenseDto::new(
                    expense_id.to_string(),
                    owner_id.to_string(),
                    days_overdue,
                    amount,
                )
            })
            .collect())
    }

    /// Bulk create reminders for all overdue expenses
    pub async fn bulk_create_reminders(
        &self,
        dto: BulkCreateRemindersDto,
    ) -> Result<BulkCreateRemindersResponseDto, String> {
        let organization_id = Uuid::parse_str(&dto.organization_id)
            .map_err(|_| "Invalid organization_id format".to_string())?;

        let overdue_list = self
            .find_overdue_expenses_without_reminders(organization_id, dto.min_days_overdue)
            .await?;

        let mut created_count = 0;
        let mut skipped_count = 0;
        let mut errors = Vec::new();
        let mut created_reminders = Vec::new();

        for overdue in overdue_list {
            let expense_id = Uuid::parse_str(&overdue.expense_id);
            let owner_id = Uuid::parse_str(&overdue.owner_id);

            if expense_id.is_err() || owner_id.is_err() {
                errors.push(
                    "Invalid UUID format for expense_id or owner_id in overdue item".to_string(),
                );
                skipped_count += 1;
                continue;
            }

            let expense_id = expense_id.unwrap();
            let owner_id = owner_id.unwrap();

            // Get expense to get due date
            let expense_result = self.expense_repository.find_by_id(expense_id).await;

            match expense_result {
                Ok(Some(expense)) => {
                    let due_date = expense.expense_date;

                    let create_dto = CreatePaymentReminderDto {
                        organization_id: organization_id.to_string(),
                        expense_id: expense_id.to_string(),
                        owner_id: owner_id.to_string(),
                        level: overdue.recommended_level,
                        amount_owed: overdue.amount,
                        due_date: due_date.to_rfc3339(),
                        days_overdue: overdue.days_overdue,
                    };

                    match self.create_reminder(create_dto).await {
                        Ok(reminder) => {
                            created_count += 1;
                            created_reminders.push(reminder);
                        }
                        Err(e) => {
                            errors.push(format!(
                                "Error creating reminder for expense {}: {}",
                                expense_id, e
                            ));
                            skipped_count += 1;
                        }
                    }
                }
                Ok(None) => {
                    errors.push(format!("Expense {} not found", expense_id));
                    skipped_count += 1;
                }
                Err(e) => {
                    errors.push(format!("Error fetching expense {}: {}", expense_id, e));
                    skipped_count += 1;
                }
            }
        }

        Ok(BulkCreateRemindersResponseDto {
            created_count,
            skipped_count,
            errors,
            created_reminders,
        })
    }

    /// Process automatic escalations (called by cron job)
    pub async fn process_automatic_escalations(&self) -> Result<i32, String> {
        let reminders = self.find_reminders_needing_escalation().await?;
        let mut escalated_count = 0;

        for reminder_dto in reminders {
            let id =
                Uuid::parse_str(&reminder_dto.id).map_err(|_| "Invalid reminder ID".to_string())?;

            match self
                .escalate_reminder(id, EscalateReminderDto { reason: None })
                .await
            {
                Ok(_) => escalated_count += 1,
                Err(e) => {
                    eprintln!("Error escalating reminder {}: {}", id, e);
                }
            }
        }

        Ok(escalated_count)
    }

    /// Recalculate penalties for all active reminders (called periodically)
    pub async fn recalculate_all_penalties(&self, organization_id: Uuid) -> Result<i32, String> {
        let reminders = self
            .reminder_repository
            .find_by_organization_and_status(organization_id, ReminderStatus::Sent)
            .await?;

        let mut updated_count = 0;

        for mut reminder in reminders {
            let current_days = (Utc::now() - reminder.due_date).num_days();
            if current_days != reminder.days_overdue {
                reminder.recalculate_penalties(current_days);
                self.reminder_repository.update(&reminder).await?;
                updated_count += 1;
            }
        }

        Ok(updated_count)
    }

    /// Delete a reminder
    pub async fn delete_reminder(&self, id: Uuid) -> Result<bool, String> {
        self.reminder_repository.delete(id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::ports::PaymentReminderRepository;
    use crate::domain::entities::ReminderLevel;
    use async_trait::async_trait;
    use std::collections::HashMap;
    use std::sync::Mutex;

    // Mock repository for testing
    #[allow(dead_code)]
    struct MockPaymentReminderRepository {
        reminders: Mutex<HashMap<Uuid, PaymentReminder>>,
    }

    #[allow(dead_code)]
    impl MockPaymentReminderRepository {
        fn new() -> Self {
            Self {
                reminders: Mutex::new(HashMap::new()),
            }
        }
    }

    #[async_trait]
    impl PaymentReminderRepository for MockPaymentReminderRepository {
        async fn create(&self, reminder: &PaymentReminder) -> Result<PaymentReminder, String> {
            let mut reminders = self.reminders.lock().unwrap();
            reminders.insert(reminder.id, reminder.clone());
            Ok(reminder.clone())
        }

        async fn find_by_id(&self, id: Uuid) -> Result<Option<PaymentReminder>, String> {
            let reminders = self.reminders.lock().unwrap();
            Ok(reminders.get(&id).cloned())
        }

        async fn find_by_expense(&self, expense_id: Uuid) -> Result<Vec<PaymentReminder>, String> {
            let reminders = self.reminders.lock().unwrap();
            Ok(reminders
                .values()
                .filter(|r| r.expense_id == expense_id)
                .cloned()
                .collect())
        }

        async fn find_by_owner(&self, owner_id: Uuid) -> Result<Vec<PaymentReminder>, String> {
            let reminders = self.reminders.lock().unwrap();
            Ok(reminders
                .values()
                .filter(|r| r.owner_id == owner_id)
                .cloned()
                .collect())
        }

        async fn find_by_organization(
            &self,
            organization_id: Uuid,
        ) -> Result<Vec<PaymentReminder>, String> {
            let reminders = self.reminders.lock().unwrap();
            Ok(reminders
                .values()
                .filter(|r| r.organization_id == organization_id)
                .cloned()
                .collect())
        }

        async fn find_by_status(
            &self,
            status: ReminderStatus,
        ) -> Result<Vec<PaymentReminder>, String> {
            let reminders = self.reminders.lock().unwrap();
            Ok(reminders
                .values()
                .filter(|r| r.status == status)
                .cloned()
                .collect())
        }

        async fn find_by_organization_and_status(
            &self,
            organization_id: Uuid,
            status: ReminderStatus,
        ) -> Result<Vec<PaymentReminder>, String> {
            let reminders = self.reminders.lock().unwrap();
            Ok(reminders
                .values()
                .filter(|r| r.organization_id == organization_id && r.status == status)
                .cloned()
                .collect())
        }

        async fn find_pending_reminders(&self) -> Result<Vec<PaymentReminder>, String> {
            self.find_by_status(ReminderStatus::Pending).await
        }

        async fn find_reminders_needing_escalation(
            &self,
            _cutoff_date: DateTime<Utc>,
        ) -> Result<Vec<PaymentReminder>, String> {
            Ok(vec![])
        }

        async fn find_latest_by_expense(
            &self,
            _expense_id: Uuid,
        ) -> Result<Option<PaymentReminder>, String> {
            Ok(None)
        }

        async fn find_active_by_owner(
            &self,
            _owner_id: Uuid,
        ) -> Result<Vec<PaymentReminder>, String> {
            Ok(vec![])
        }

        async fn count_by_status(
            &self,
            _organization_id: Uuid,
        ) -> Result<Vec<(ReminderStatus, i64)>, String> {
            Ok(vec![])
        }

        async fn get_total_owed_by_organization(
            &self,
            _organization_id: Uuid,
        ) -> Result<f64, String> {
            Ok(0.0)
        }

        async fn get_total_penalties_by_organization(
            &self,
            _organization_id: Uuid,
        ) -> Result<f64, String> {
            Ok(0.0)
        }

        async fn find_overdue_expenses_without_reminders(
            &self,
            _organization_id: Uuid,
            _min_days_overdue: i64,
        ) -> Result<Vec<(Uuid, Uuid, i64, f64)>, String> {
            Ok(vec![])
        }

        async fn update(&self, reminder: &PaymentReminder) -> Result<PaymentReminder, String> {
            let mut reminders = self.reminders.lock().unwrap();
            reminders.insert(reminder.id, reminder.clone());
            Ok(reminder.clone())
        }

        async fn delete(&self, id: Uuid) -> Result<bool, String> {
            let mut reminders = self.reminders.lock().unwrap();
            Ok(reminders.remove(&id).is_some())
        }

        async fn get_dashboard_stats(
            &self,
            _organization_id: Uuid,
        ) -> Result<(f64, f64, Vec<(ReminderLevel, i64)>), String> {
            Ok((0.0, 0.0, vec![]))
        }
    }

    // TODO: Add more comprehensive tests
}
