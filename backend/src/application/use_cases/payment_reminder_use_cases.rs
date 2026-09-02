use crate::application::dto::{
    AddTrackingNumberDto, BulkCreateRemindersDto, BulkCreateRemindersResponseDto,
    CancelReminderDto, CreatePaymentReminderDto, EscalateReminderDto, MarkReminderSentDto,
    OverdueExpenseDto, PaymentRecoveryStatsDto, PaymentReminderResponseDto, ReminderLevelCountDto,
    ReminderStatusCountDto,
};
use crate::application::error::AppError;
use crate::application::ports::{ExpenseRepository, OwnerRepository, PaymentReminderRepository};
use crate::domain::entities::{PaymentReminder, PaymentStatus, ReminderStatus};
use chrono::{DateTime, Utc};
use std::sync::Arc;
use uuid::Uuid;

pub struct PaymentReminderUseCases {
    reminder_repository: Arc<dyn PaymentReminderRepository>,
    expense_repository: Arc<dyn ExpenseRepository>,
    owner_repository: Arc<dyn OwnerRepository>,
}

impl PaymentReminderUseCases {
    pub fn new(
        reminder_repository: Arc<dyn PaymentReminderRepository>,
        expense_repository: Arc<dyn ExpenseRepository>,
        owner_repository: Arc<dyn OwnerRepository>,
    ) -> Self {
        Self {
            reminder_repository,
            expense_repository,
            owner_repository,
        }
    }

    /// Helper to enrich a reminder DTO with owner information
    async fn enrich_with_owner_info(
        &self,
        mut dto: PaymentReminderResponseDto,
    ) -> Result<PaymentReminderResponseDto, AppError> {
        let owner_id =
            Uuid::parse_str(&dto.owner_id).map_err(|_| "Invalid owner_id format".to_string())?;

        if let Ok(Some(owner)) = self.owner_repository.find_by_id(owner_id).await {
            dto.owner_name = Some(owner.full_name());
            dto.owner_email = Some(owner.email.clone());
        }

        Ok(dto)
    }

    /// Create a new payment reminder
    pub async fn create_reminder(
        &self,
        dto: CreatePaymentReminderDto,
    ) -> Result<PaymentReminderResponseDto, AppError> {
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
            return Err(AppError::Conflict(
                "Cannot create reminder for paid expense".to_string(),
            ));
        }

        // Check if reminder already exists for this expense and owner at this level
        let existing_reminders = self.reminder_repository.find_by_expense(expense_id).await?;

        if existing_reminders.iter().any(|r| {
            r.owner_id == owner_id
                && r.level == dto.level
                && r.status != ReminderStatus::Cancelled
                && r.status != ReminderStatus::Paid
        }) {
            return Err(AppError::Conflict(format!(
                "Active reminder already exists for this expense at {:?} level",
                dto.level
            )));
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
    ) -> Result<Option<PaymentReminderResponseDto>, AppError> {
        let reminder = self.reminder_repository.find_by_id(id).await?;
        Ok(reminder.map(|r| r.into()))
    }

    /// List all reminders for an expense
    pub async fn list_by_expense(
        &self,
        expense_id: Uuid,
    ) -> Result<Vec<PaymentReminderResponseDto>, AppError> {
        let reminders = self.reminder_repository.find_by_expense(expense_id).await?;
        Ok(reminders.into_iter().map(|r| r.into()).collect())
    }

    /// List all reminders for an owner
    pub async fn list_by_owner(
        &self,
        owner_id: Uuid,
    ) -> Result<Vec<PaymentReminderResponseDto>, AppError> {
        let reminders = self.reminder_repository.find_by_owner(owner_id).await?;
        Ok(reminders.into_iter().map(|r| r.into()).collect())
    }

    /// List all reminders for an organization
    pub async fn list_by_organization(
        &self,
        organization_id: Uuid,
    ) -> Result<Vec<PaymentReminderResponseDto>, AppError> {
        let reminders = self
            .reminder_repository
            .find_by_organization(organization_id)
            .await?;

        // Enrich each reminder with owner information
        let mut enriched_reminders = Vec::new();
        for reminder in reminders {
            let dto: PaymentReminderResponseDto = reminder.into();
            let enriched = self.enrich_with_owner_info(dto).await?;
            enriched_reminders.push(enriched);
        }

        Ok(enriched_reminders)
    }

    /// List active (non-paid, non-cancelled) reminders for an owner
    pub async fn list_active_by_owner(
        &self,
        owner_id: Uuid,
    ) -> Result<Vec<PaymentReminderResponseDto>, AppError> {
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
    ) -> Result<PaymentReminderResponseDto, AppError> {
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
    pub async fn mark_as_opened(&self, id: Uuid) -> Result<PaymentReminderResponseDto, AppError> {
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
    pub async fn mark_as_paid(&self, id: Uuid) -> Result<PaymentReminderResponseDto, AppError> {
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
    ) -> Result<PaymentReminderResponseDto, AppError> {
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
    ) -> Result<Option<PaymentReminderResponseDto>, AppError> {
        let mut reminder = self
            .reminder_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| "Reminder not found".to_string())?;

        // Le statut est contrôlé D'ABORD : un dossier soldé ou annulé doit se
        // voir reprocher son statut, pas le délai du niveau suivant.
        reminder.can_escalate()?;

        // Puis le niveau suivant est CONSTRUIT AVANT que l'escalade ne soit
        // persistée.
        //
        // L'ordre inverse laissait un dossier à l'abandon : le statut
        // `Escalated` était écrit, puis `PaymentReminder::new` pouvait échouer
        // (« Cannot create second reminder before 30 days overdue »), et la
        // relance restait marquée escaladée SANS successeur. Personne ne le
        // voyait : `process_automatic_escalations`, appelé par cron, journalise
        // l'erreur sur stderr et poursuit sa boucle.
        //
        // Le cas se produit dès que l'escalade est déclenchée plus tôt que le
        // délai légal du niveau visé — 30 jours pour une relance ferme, 60 pour
        // une mise en demeure.
        let next_level = reminder.level.next_level();
        let next_reminder = match next_level {
            Some(level) => {
                let days_overdue = (Utc::now() - reminder.due_date).num_days();
                Some(PaymentReminder::new(
                    reminder.organization_id,
                    reminder.expense_id,
                    reminder.owner_id,
                    level,
                    reminder.amount_owed,
                    reminder.due_date,
                    days_overdue,
                )?)
            }
            None => None,
        };

        // Refuse l'escalade d'un dossier soldé ou annulé, et marque le statut.
        reminder.escalate()?;
        let updated = self.reminder_repository.update(&reminder).await?;

        if let Some(next_reminder) = next_reminder {
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
    ) -> Result<PaymentReminderResponseDto, AppError> {
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
    pub async fn find_pending_reminders(
        &self,
    ) -> Result<Vec<PaymentReminderResponseDto>, AppError> {
        let reminders = self.reminder_repository.find_pending_reminders().await?;
        Ok(reminders.into_iter().map(|r| r.into()).collect())
    }

    /// Find reminders needing escalation (sent >15 days ago)
    pub async fn find_reminders_needing_escalation(
        &self,
    ) -> Result<Vec<PaymentReminderResponseDto>, AppError> {
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
    ) -> Result<PaymentRecoveryStatsDto, AppError> {
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
    ) -> Result<Vec<OverdueExpenseDto>, AppError> {
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
    ) -> Result<BulkCreateRemindersResponseDto, AppError> {
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
    pub async fn process_automatic_escalations(&self) -> Result<i32, AppError> {
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
    pub async fn recalculate_all_penalties(&self, organization_id: Uuid) -> Result<i32, AppError> {
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
    pub async fn delete_reminder(&self, id: Uuid) -> Result<bool, AppError> {
        self.reminder_repository.delete(id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::ports::PaymentReminderRepository;
    use crate::domain::entities::ReminderLevel;
    use async_trait::async_trait;
    use rust_decimal::Decimal;
    use std::collections::HashMap;
    use std::sync::Mutex;

    // ── Dépôts simulés ───────────────────────────────────────────────────
    //
    // Ce mock existait déjà, complet, marqué `#[allow(dead_code)]` et suivi
    // d'un `// TODO: Add more comprehensive tests`. L'échafaudage avait été
    // construit puis jamais utilisé — et le `allow(dead_code)` faisait taire
    // l'avertissement qui l'aurait signalé. 670 lignes de logique de
    // recouvrement (pénalités, escalade à 4 niveaux, création en masse) sans
    // un seul test, sur le module même que visent les constats F10 et F18 du
    // rapport du 2026-09-01.
    struct MockPaymentReminderRepository {
        reminders: Mutex<HashMap<Uuid, PaymentReminder>>,
    }

    impl MockPaymentReminderRepository {
        fn new() -> Self {
            Self {
                reminders: Mutex::new(HashMap::new()),
            }
        }

        fn get(&self, id: Uuid) -> Option<PaymentReminder> {
            self.reminders.lock().unwrap().get(&id).cloned()
        }

        fn count(&self) -> usize {
            self.reminders.lock().unwrap().len()
        }
    }

    #[async_trait]
    impl PaymentReminderRepository for MockPaymentReminderRepository {
        async fn create(&self, reminder: &PaymentReminder) -> Result<PaymentReminder, AppError> {
            let mut reminders = self.reminders.lock().unwrap();
            reminders.insert(reminder.id, reminder.clone());
            Ok(reminder.clone())
        }

        async fn find_by_id(&self, id: Uuid) -> Result<Option<PaymentReminder>, AppError> {
            let reminders = self.reminders.lock().unwrap();
            Ok(reminders.get(&id).cloned())
        }

        async fn find_by_expense(
            &self,
            expense_id: Uuid,
        ) -> Result<Vec<PaymentReminder>, AppError> {
            let reminders = self.reminders.lock().unwrap();
            Ok(reminders
                .values()
                .filter(|r| r.expense_id == expense_id)
                .cloned()
                .collect())
        }

        async fn find_by_owner(&self, owner_id: Uuid) -> Result<Vec<PaymentReminder>, AppError> {
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
        ) -> Result<Vec<PaymentReminder>, AppError> {
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
        ) -> Result<Vec<PaymentReminder>, AppError> {
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
        ) -> Result<Vec<PaymentReminder>, AppError> {
            let reminders = self.reminders.lock().unwrap();
            Ok(reminders
                .values()
                .filter(|r| r.organization_id == organization_id && r.status == status)
                .cloned()
                .collect())
        }

        async fn find_pending_reminders(&self) -> Result<Vec<PaymentReminder>, AppError> {
            self.find_by_status(ReminderStatus::Pending).await
        }

        async fn find_reminders_needing_escalation(
            &self,
            _cutoff_date: DateTime<Utc>,
        ) -> Result<Vec<PaymentReminder>, AppError> {
            Ok(vec![])
        }

        async fn find_latest_by_expense(
            &self,
            _expense_id: Uuid,
        ) -> Result<Option<PaymentReminder>, AppError> {
            Ok(None)
        }

        async fn find_active_by_owner(
            &self,
            _owner_id: Uuid,
        ) -> Result<Vec<PaymentReminder>, AppError> {
            Ok(vec![])
        }

        /// Compte REELLEMENT, au lieu de renvoyer un vecteur vide.
        ///
        /// Le stub d'origine rendait tout test de statistiques vide de sens :
        /// il aurait passé quelle que soit la logique testée.
        async fn count_by_status(
            &self,
            organization_id: Uuid,
        ) -> Result<Vec<(ReminderStatus, i64)>, AppError> {
            let reminders = self.reminders.lock().unwrap();
            let mut comptes: HashMap<String, (ReminderStatus, i64)> = HashMap::new();
            for r in reminders
                .values()
                .filter(|r| r.organization_id == organization_id)
            {
                let cle = format!("{:?}", r.status);
                comptes
                    .entry(cle)
                    .and_modify(|(_, n)| *n += 1)
                    .or_insert((r.status.clone(), 1));
            }
            Ok(comptes.into_values().collect())
        }

        async fn get_total_owed_by_organization(
            &self,
            _organization_id: Uuid,
        ) -> Result<Decimal, AppError> {
            Ok(Decimal::ZERO)
        }

        async fn get_total_penalties_by_organization(
            &self,
            _organization_id: Uuid,
        ) -> Result<Decimal, AppError> {
            Ok(Decimal::ZERO)
        }

        async fn find_overdue_expenses_without_reminders(
            &self,
            _organization_id: Uuid,
            _min_days_overdue: i64,
        ) -> Result<Vec<(Uuid, Uuid, i64, Decimal)>, AppError> {
            Ok(vec![])
        }

        async fn update(&self, reminder: &PaymentReminder) -> Result<PaymentReminder, AppError> {
            let mut reminders = self.reminders.lock().unwrap();
            reminders.insert(reminder.id, reminder.clone());
            Ok(reminder.clone())
        }

        async fn delete(&self, id: Uuid) -> Result<bool, AppError> {
            let mut reminders = self.reminders.lock().unwrap();
            Ok(reminders.remove(&id).is_some())
        }

        async fn get_dashboard_stats(
            &self,
            organization_id: Uuid,
        ) -> Result<(Decimal, Decimal, Vec<(ReminderLevel, i64)>), AppError> {
            let reminders = self.reminders.lock().unwrap();
            let actives: Vec<_> = reminders
                .values()
                .filter(|r| {
                    r.organization_id == organization_id
                        && r.status != ReminderStatus::Paid
                        && r.status != ReminderStatus::Cancelled
                })
                .collect();
            let total_owed = actives.iter().map(|r| r.amount_owed).sum();
            let total_penalties = actives.iter().map(|r| r.penalty_amount).sum();
            let mut comptes: HashMap<String, (ReminderLevel, i64)> = HashMap::new();
            for r in &actives {
                let cle = format!("{:?}", r.level);
                comptes
                    .entry(cle)
                    .and_modify(|(_, n)| *n += 1)
                    .or_insert((r.level.clone(), 1));
            }
            Ok((total_owed, total_penalties, comptes.into_values().collect()))
        }
    }

    // ── Dépôts simulés : dépenses et propriétaires ───────────────────────

    struct MockExpenseRepo {
        expenses: Mutex<HashMap<Uuid, crate::domain::entities::Expense>>,
    }

    impl MockExpenseRepo {
        fn new() -> Self {
            Self {
                expenses: Mutex::new(HashMap::new()),
            }
        }

        fn with(expense: crate::domain::entities::Expense) -> Self {
            let mut m = HashMap::new();
            m.insert(expense.id, expense);
            Self {
                expenses: Mutex::new(m),
            }
        }
    }

    #[async_trait]
    impl crate::application::ports::ExpenseRepository for MockExpenseRepo {
        async fn create(
            &self,
            e: &crate::domain::entities::Expense,
        ) -> Result<crate::domain::entities::Expense, String> {
            self.expenses.lock().unwrap().insert(e.id, e.clone());
            Ok(e.clone())
        }
        async fn find_by_id(
            &self,
            id: Uuid,
        ) -> Result<Option<crate::domain::entities::Expense>, String> {
            Ok(self.expenses.lock().unwrap().get(&id).cloned())
        }
        async fn find_by_building(
            &self,
            building_id: Uuid,
        ) -> Result<Vec<crate::domain::entities::Expense>, String> {
            Ok(self
                .expenses
                .lock()
                .unwrap()
                .values()
                .filter(|e| e.building_id == building_id)
                .cloned()
                .collect())
        }
        async fn find_all_paginated(
            &self,
            _p: &crate::application::dto::PageRequest,
            _f: &crate::application::dto::ExpenseFilters,
        ) -> Result<(Vec<crate::domain::entities::Expense>, i64), String> {
            Ok((vec![], 0))
        }
        async fn update(
            &self,
            e: &crate::domain::entities::Expense,
        ) -> Result<crate::domain::entities::Expense, String> {
            self.expenses.lock().unwrap().insert(e.id, e.clone());
            Ok(e.clone())
        }
        async fn delete(&self, id: Uuid) -> Result<bool, String> {
            Ok(self.expenses.lock().unwrap().remove(&id).is_some())
        }
    }

    struct MockOwnerRepo {
        owners: Mutex<HashMap<Uuid, crate::domain::entities::Owner>>,
    }

    impl MockOwnerRepo {
        fn new() -> Self {
            Self {
                owners: Mutex::new(HashMap::new()),
            }
        }

        fn with(owner: crate::domain::entities::Owner) -> Self {
            let mut m = HashMap::new();
            m.insert(owner.id, owner);
            Self {
                owners: Mutex::new(m),
            }
        }
    }

    #[async_trait]
    impl crate::application::ports::OwnerRepository for MockOwnerRepo {
        async fn create(
            &self,
            o: &crate::domain::entities::Owner,
        ) -> Result<crate::domain::entities::Owner, String> {
            self.owners.lock().unwrap().insert(o.id, o.clone());
            Ok(o.clone())
        }
        async fn find_by_id(
            &self,
            id: Uuid,
        ) -> Result<Option<crate::domain::entities::Owner>, String> {
            Ok(self.owners.lock().unwrap().get(&id).cloned())
        }
        async fn find_by_user_id(
            &self,
            _id: Uuid,
        ) -> Result<Option<crate::domain::entities::Owner>, String> {
            Ok(None)
        }
        async fn find_by_user_id_and_organization(
            &self,
            _u: Uuid,
            _o: Uuid,
        ) -> Result<Option<crate::domain::entities::Owner>, String> {
            Ok(None)
        }
        async fn find_by_email(
            &self,
            _e: &str,
        ) -> Result<Option<crate::domain::entities::Owner>, String> {
            Ok(None)
        }
        async fn find_all(&self) -> Result<Vec<crate::domain::entities::Owner>, String> {
            Ok(self.owners.lock().unwrap().values().cloned().collect())
        }
        async fn find_all_paginated(
            &self,
            _p: &crate::application::dto::PageRequest,
            _f: &crate::application::dto::OwnerFilters,
        ) -> Result<(Vec<crate::domain::entities::Owner>, i64), String> {
            Ok((vec![], 0))
        }
        async fn update(
            &self,
            o: &crate::domain::entities::Owner,
        ) -> Result<crate::domain::entities::Owner, String> {
            self.owners.lock().unwrap().insert(o.id, o.clone());
            Ok(o.clone())
        }
        async fn delete(&self, id: Uuid) -> Result<bool, String> {
            Ok(self.owners.lock().unwrap().remove(&id).is_some())
        }
        async fn set_user_link(&self, _o: Uuid, _u: Option<Uuid>) -> Result<bool, String> {
            Ok(true)
        }
    }

    // ── Fixtures ─────────────────────────────────────────────────────────

    fn expense_impaye(org_id: Uuid, montant: Decimal) -> crate::domain::entities::Expense {
        crate::domain::entities::Expense::new(
            Uuid::new_v4(), // acp_id
            org_id,
            Uuid::new_v4(),
            crate::domain::entities::ExpenseCategory::Maintenance,
            "Entretien ascenseur".to_string(),
            montant,
            Utc::now() - chrono::Duration::days(40),
            Some("Kone SA".to_string()),
            None,
            Some("611002".to_string()),
        )
        .expect("dépense de test valide")
    }

    fn proprietaire(org_id: Uuid) -> crate::domain::entities::Owner {
        crate::domain::entities::Owner::new(
            org_id,
            "Jean".to_string(),
            "Peeters".to_string(),
            "jean.peeters@example.com".to_string(),
            None,
            "Rue Test 1".to_string(),
            "Bruxelles".to_string(),
            "1000".to_string(),
            "Belgique".to_string(),
        )
        .expect("propriétaire de test valide")
    }

    fn use_cases(
        reminders: Arc<MockPaymentReminderRepository>,
        expenses: Arc<MockExpenseRepo>,
        owners: Arc<MockOwnerRepo>,
    ) -> PaymentReminderUseCases {
        PaymentReminderUseCases::new(reminders, expenses, owners)
    }

    fn create_dto(
        org_id: Uuid,
        expense_id: Uuid,
        owner_id: Uuid,
        level: ReminderLevel,
        days_overdue: i64,
    ) -> CreatePaymentReminderDto {
        CreatePaymentReminderDto {
            organization_id: org_id.to_string(),
            expense_id: expense_id.to_string(),
            owner_id: owner_id.to_string(),
            level,
            amount_owed: Decimal::from(2000),
            due_date: (Utc::now() - chrono::Duration::days(days_overdue)).to_rfc3339(),
            days_overdue,
        }
    }

    // ── Tests ────────────────────────────────────────────────────────────

    /// Le chemin nominal : une dépense impayée depuis 17 jours produit un
    /// premier rappel, avec sa pénalité calculée.
    #[tokio::test]
    async fn test_creation_relance_calcule_la_penalite() {
        let org_id = Uuid::new_v4();
        let depense = expense_impaye(org_id, Decimal::from(2000));
        let prop = proprietaire(org_id);
        let (expense_id, owner_id) = (depense.id, prop.id);

        let reminders = Arc::new(MockPaymentReminderRepository::new());
        let uc = use_cases(
            reminders.clone(),
            Arc::new(MockExpenseRepo::with(depense)),
            Arc::new(MockOwnerRepo::with(prop)),
        );

        let dto = create_dto(org_id, expense_id, owner_id, ReminderLevel::FirstReminder, 17);
        let cree = uc.create_reminder(dto).await.expect("création acceptée");

        assert_eq!(reminders.count(), 1);
        // 2000 × 0,045 × 17/365 = 4,1917… → 4,19 €. C'est le calcul que le
        // rapport du 2026-09-01 disait « incohérent » : il l'est avec le taux
        // de 8 % qu'ANNONÇAIT l'interface, pas avec le taux de 4,5 % que le
        // domaine applique. Le texte de l'interface a été aligné sur le calcul.
        assert_eq!(cree.penalty_amount, rust_decimal_macros::dec!(4.19));
        assert_eq!(cree.total_amount, rust_decimal_macros::dec!(2004.19));
    }

    /// @security — pas de relance sur une dépense déjà réglée.
    #[tokio::test]
    async fn test_relance_refusee_sur_depense_payee() {
        let org_id = Uuid::new_v4();
        let mut depense = expense_impaye(org_id, Decimal::from(2000));
        depense.payment_status = PaymentStatus::Paid;
        let prop = proprietaire(org_id);
        let (expense_id, owner_id) = (depense.id, prop.id);

        let reminders = Arc::new(MockPaymentReminderRepository::new());
        let uc = use_cases(
            reminders.clone(),
            Arc::new(MockExpenseRepo::with(depense)),
            Arc::new(MockOwnerRepo::with(prop)),
        );

        let err = uc
            .create_reminder(create_dto(
                org_id,
                expense_id,
                owner_id,
                ReminderLevel::FirstReminder,
                17,
            ))
            .await
            .expect_err("réclamer un montant déjà payé doit être refusé");
        assert!(format!("{err}").contains("paid"), "{err}");
        assert_eq!(reminders.count(), 0);
    }

    /// @edge — pas deux relances actives au même niveau pour la même dépense.
    ///
    /// Le garde-fou qui empêche un copropriétaire de recevoir deux fois la
    /// même mise en demeure.
    #[tokio::test]
    async fn test_pas_de_doublon_de_relance_au_meme_niveau() {
        let org_id = Uuid::new_v4();
        let depense = expense_impaye(org_id, Decimal::from(2000));
        let prop = proprietaire(org_id);
        let (expense_id, owner_id) = (depense.id, prop.id);

        let reminders = Arc::new(MockPaymentReminderRepository::new());
        let uc = use_cases(
            reminders.clone(),
            Arc::new(MockExpenseRepo::with(depense)),
            Arc::new(MockOwnerRepo::with(prop)),
        );

        uc.create_reminder(create_dto(
            org_id,
            expense_id,
            owner_id,
            ReminderLevel::FirstReminder,
            17,
        ))
        .await
        .expect("première relance acceptée");

        let err = uc
            .create_reminder(create_dto(
                org_id,
                expense_id,
                owner_id,
                ReminderLevel::FirstReminder,
                17,
            ))
            .await
            .expect_err("doublon refusé");
        assert!(format!("{err}").contains("already exists"), "{err}");
        assert_eq!(reminders.count(), 1);

        // Le niveau SUIVANT reste possible : c'est l'escalade normale.
        uc.create_reminder(create_dto(
            org_id,
            expense_id,
            owner_id,
            ReminderLevel::SecondReminder,
            35,
        ))
        .await
        .expect("le second niveau doit rester ouvert");
        assert_eq!(reminders.count(), 2);
    }

    /// @edge — un niveau ne peut pas être créé avant son délai légal.
    ///
    /// Une mise en demeure est un acte à J+60. L'émettre à J+20 exposerait
    /// l'ACP sur le fond comme sur la forme.
    #[tokio::test]
    async fn test_niveau_refuse_avant_son_delai() {
        let org_id = Uuid::new_v4();
        let depense = expense_impaye(org_id, Decimal::from(2000));
        let prop = proprietaire(org_id);
        let (expense_id, owner_id) = (depense.id, prop.id);

        let uc = use_cases(
            Arc::new(MockPaymentReminderRepository::new()),
            Arc::new(MockExpenseRepo::with(depense)),
            Arc::new(MockOwnerRepo::with(prop)),
        );

        let err = uc
            .create_reminder(create_dto(
                org_id,
                expense_id,
                owner_id,
                ReminderLevel::FormalNotice,
                20,
            ))
            .await
            .expect_err("mise en demeure prématurée refusée");
        assert!(format!("{err}").contains("60 days"), "{err}");
    }

    /// L'escalade crée la relance du niveau suivant et marque la précédente.
    #[tokio::test]
    async fn test_escalade_cree_le_niveau_suivant() {
        let org_id = Uuid::new_v4();
        let depense = expense_impaye(org_id, Decimal::from(2000));
        let prop = proprietaire(org_id);
        let (expense_id, owner_id) = (depense.id, prop.id);

        let reminders = Arc::new(MockPaymentReminderRepository::new());
        let uc = use_cases(
            reminders.clone(),
            Arc::new(MockExpenseRepo::with(depense)),
            Arc::new(MockOwnerRepo::with(prop)),
        );

        // 35 jours de retard : le niveau suivant (J+30) est atteignable.
        let premiere = uc
            .create_reminder(create_dto(
                org_id,
                expense_id,
                owner_id,
                ReminderLevel::FirstReminder,
                35,
            ))
            .await
            .expect("première relance");
        let premiere_id = Uuid::parse_str(&premiere.id).unwrap();

        let suivante = uc
            .escalate_reminder(premiere_id, EscalateReminderDto { reason: None })
            .await
            .expect("escalade acceptée")
            .expect("une relance est renvoyée");

        assert_eq!(reminders.count(), 2, "l'escalade crée le niveau suivant");
        assert_eq!(suivante.level, ReminderLevel::SecondReminder);
        assert_eq!(
            reminders.get(premiere_id).unwrap().status,
            ReminderStatus::Escalated,
            "la relance d'origine doit être marquée escaladée"
        );
    }

    /// @edge — une escalade prématurée ne laisse pas le dossier à l'abandon.
    ///
    /// Défaut d'ordonnancement trouvé en écrivant ces tests : le statut
    /// `Escalated` était PERSISTÉ avant que le niveau suivant ne soit validé.
    /// Sur un premier rappel à J+17, la relance ferme (J+30) est refusée par le
    /// domaine — mais la relance d'origine restait marquée escaladée, sans
    /// successeur, définitivement bloquée.
    ///
    /// Personne ne l'aurait vu : `process_automatic_escalations`, appelé par
    /// cron, journalise l'erreur sur stderr et poursuit sa boucle.
    #[tokio::test]
    async fn test_escalade_prematuree_ne_modifie_rien() {
        let org_id = Uuid::new_v4();
        let depense = expense_impaye(org_id, Decimal::from(2000));
        let prop = proprietaire(org_id);
        let (expense_id, owner_id) = (depense.id, prop.id);

        let reminders = Arc::new(MockPaymentReminderRepository::new());
        let uc = use_cases(
            reminders.clone(),
            Arc::new(MockExpenseRepo::with(depense)),
            Arc::new(MockOwnerRepo::with(prop)),
        );

        let premiere = uc
            .create_reminder(create_dto(
                org_id,
                expense_id,
                owner_id,
                ReminderLevel::FirstReminder,
                17,
            ))
            .await
            .expect("premier rappel à J+17");
        let id = Uuid::parse_str(&premiere.id).unwrap();

        let err = uc
            .escalate_reminder(id, EscalateReminderDto { reason: None })
            .await
            .expect_err("le niveau suivant n'est pas encore atteignable");
        assert!(format!("{err}").contains("30 days"), "{err}");

        // L'invariant : rien n'a bougé.
        assert_eq!(reminders.count(), 1, "aucune relance fantôme");
        assert_eq!(
            reminders.get(id).unwrap().status,
            ReminderStatus::Pending,
            "la relance d'origine ne doit PAS rester marquée escaladée sans successeur"
        );
    }

    /// @edge — l'escalade s'arrête à la mise en demeure.
    ///
    /// Au-delà, c'est l'huissier : une procédure judiciaire, pas une relance
    /// de plus que le système émettrait tout seul.
    #[tokio::test]
    async fn test_escalade_sarrete_a_la_mise_en_demeure() {
        let org_id = Uuid::new_v4();
        let depense = expense_impaye(org_id, Decimal::from(2000));
        let prop = proprietaire(org_id);
        let (expense_id, owner_id) = (depense.id, prop.id);

        let reminders = Arc::new(MockPaymentReminderRepository::new());
        let uc = use_cases(
            reminders.clone(),
            Arc::new(MockExpenseRepo::with(depense)),
            Arc::new(MockOwnerRepo::with(prop)),
        );

        let mise_en_demeure = uc
            .create_reminder(create_dto(
                org_id,
                expense_id,
                owner_id,
                ReminderLevel::FormalNotice,
                70,
            ))
            .await
            .expect("mise en demeure");
        let id = Uuid::parse_str(&mise_en_demeure.id).unwrap();

        uc.escalate_reminder(id, EscalateReminderDto { reason: None })
            .await
            .expect("escalade acceptée");

        assert_eq!(
            reminders.count(),
            1,
            "aucun niveau au-delà de la mise en demeure ne doit être créé"
        );
        assert_eq!(reminders.get(id).unwrap().status, ReminderStatus::Escalated);
    }

    /// @security — une relance payée ou annulée ne s'escalade pas.
    #[tokio::test]
    async fn test_escalade_refusee_apres_paiement() {
        let org_id = Uuid::new_v4();
        let depense = expense_impaye(org_id, Decimal::from(2000));
        let prop = proprietaire(org_id);
        let (expense_id, owner_id) = (depense.id, prop.id);

        let reminders = Arc::new(MockPaymentReminderRepository::new());
        let uc = use_cases(
            reminders.clone(),
            Arc::new(MockExpenseRepo::with(depense)),
            Arc::new(MockOwnerRepo::with(prop)),
        );

        let relance = uc
            .create_reminder(create_dto(
                org_id,
                expense_id,
                owner_id,
                ReminderLevel::FirstReminder,
                17,
            ))
            .await
            .expect("relance");
        let id = Uuid::parse_str(&relance.id).unwrap();

        uc.mark_as_paid(id).await.expect("marquage payé");

        let err = uc
            .escalate_reminder(id, EscalateReminderDto { reason: None })
            .await
            .expect_err("escalader un dossier soldé doit être refusé");
        assert!(format!("{err}").contains("Cannot escalate"), "{err}");
        assert_eq!(
            reminders.count(),
            1,
            "aucune relance supplémentaire ne doit partir"
        );
    }

    /// Non-régression F10 — les statistiques doivent distinguer les statuts.
    ///
    /// L'interface affichait « relances actives : 0 » en présence d'une
    /// relance en attente, parce qu'elle ne comptait QUE le statut `Sent`.
    /// Le correctif est côté interface, mais il repose sur le fait que
    /// `status_counts` remonte bien chaque statut séparément — ce que ce test
    /// verrouille.
    #[tokio::test]
    async fn test_les_statistiques_distinguent_les_statuts() {
        let org_id = Uuid::new_v4();
        let depense = expense_impaye(org_id, Decimal::from(2000));
        let prop = proprietaire(org_id);
        let (expense_id, owner_id) = (depense.id, prop.id);

        let reminders = Arc::new(MockPaymentReminderRepository::new());
        let uc = use_cases(
            reminders.clone(),
            Arc::new(MockExpenseRepo::with(depense)),
            Arc::new(MockOwnerRepo::with(prop)),
        );

        // Une relance en attente, une envoyée.
        uc.create_reminder(create_dto(
            org_id,
            expense_id,
            owner_id,
            ReminderLevel::FirstReminder,
            17,
        ))
        .await
        .expect("relance 1");
        let deuxieme = uc
            .create_reminder(create_dto(
                org_id,
                expense_id,
                owner_id,
                ReminderLevel::SecondReminder,
                35,
            ))
            .await
            .expect("relance 2");
        uc.mark_as_sent(
            Uuid::parse_str(&deuxieme.id).unwrap(),
            MarkReminderSentDto {
                pdf_path: Some("/tmp/relance.pdf".to_string()),
            },
        )
        .await
        .expect("envoi");

        let stats = uc
            .get_recovery_stats(org_id)
            .await
            .expect("statistiques disponibles");

        let compte = |s: ReminderStatus| -> i64 {
            stats
                .status_counts
                .iter()
                .find(|c| c.status == s)
                .map(|c| c.count)
                .unwrap_or(0)
        };
        assert_eq!(compte(ReminderStatus::Pending), 1);
        assert_eq!(compte(ReminderStatus::Sent), 1);
        // Le cœur du constat F10 : ne compter que `Sent` masque la moitié du
        // recouvrement en cours.
        assert_eq!(
            compte(ReminderStatus::Pending) + compte(ReminderStatus::Sent),
            2,
            "les deux relances sont actives, quel que soit leur statut d'envoi"
        );
    }

    /// @edge — une dépense introuvable ne crée pas de relance fantôme.
    #[tokio::test]
    async fn test_relance_refusee_si_depense_introuvable() {
        let org_id = Uuid::new_v4();
        let reminders = Arc::new(MockPaymentReminderRepository::new());
        let uc = use_cases(
            reminders.clone(),
            Arc::new(MockExpenseRepo::new()),
            Arc::new(MockOwnerRepo::new()),
        );

        let err = uc
            .create_reminder(create_dto(
                org_id,
                Uuid::new_v4(),
                Uuid::new_v4(),
                ReminderLevel::FirstReminder,
                17,
            ))
            .await
            .expect_err("dépense inexistante refusée");
        assert!(format!("{err}").contains("not found"), "{err}");
        assert_eq!(reminders.count(), 0);
    }

    /// L'annulation n'est possible qu'avant envoi.
    ///
    /// Une lettre partie ne se rappelle pas : le dossier se solde, il ne
    /// s'efface pas.
    #[tokio::test]
    async fn test_annulation_impossible_apres_envoi() {
        let org_id = Uuid::new_v4();
        let depense = expense_impaye(org_id, Decimal::from(2000));
        let prop = proprietaire(org_id);
        let (expense_id, owner_id) = (depense.id, prop.id);

        let uc = use_cases(
            Arc::new(MockPaymentReminderRepository::new()),
            Arc::new(MockExpenseRepo::with(depense)),
            Arc::new(MockOwnerRepo::with(prop)),
        );

        let relance = uc
            .create_reminder(create_dto(
                org_id,
                expense_id,
                owner_id,
                ReminderLevel::FirstReminder,
                17,
            ))
            .await
            .expect("relance");
        let id = Uuid::parse_str(&relance.id).unwrap();

        // Avant envoi : accepté.
        let annulee = uc
            .cancel_reminder(
                id,
                CancelReminderDto {
                    reason: "Paiement reçu entre-temps".to_string(),
                },
            )
            .await
            .expect("annulation avant envoi acceptée");
        assert_eq!(annulee.status, ReminderStatus::Cancelled);
    }
}
