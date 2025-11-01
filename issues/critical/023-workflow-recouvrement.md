# Issue #023 - Workflow Recouvrement Charges Impayées

**Priorité**: 🔴 CRITIQUE
**Estimation**: 6-8 heures
**Labels**: `enhancement`, `backend`, `frontend`, `critical`, `finance`, `automation`

---

## 📋 Description

Implémenter un **workflow automatisé de recouvrement** des charges impayées avec relances progressives, mise en demeure, et tracking complet. Actuellement, KoproGo permet de marquer des expenses comme `overdue`, mais sans processus de relance structuré.

**Contexte métier** : Le recouvrement des impayés est une tâche critique pour la santé financière d'une copropriété. Un système automatisé réduit les retards de paiement de 30-50% selon les études.

**Impact métier** : Amélioration trésorerie, réduction impayés, gain temps syndic.

---

## 🎯 Objectifs

- [ ] Workflow relances automatiques (3 niveaux)
- [ ] Génération lettres de relance PDF
- [ ] Envoi emails automatiques programmés
- [ ] Tracking complet historique relances
- [ ] Calcul pénalités de retard (selon règlement)
- [ ] Mise en demeure (avant contentieux)
- [ ] Dashboard impayés avec KPIs
- [ ] Export liste impayés pour comptable/avocat

---

## 📐 Spécifications Techniques

### Workflow Recouvrement (3 Niveaux)

#### Niveau 1 : Relance Amiable (J+15)
- **Délai** : 15 jours après échéance
- **Type** : Email automatique + lettre simple
- **Ton** : Courtois ("Nous vous rappelons...")
- **Contenu** : Montant dû, date échéance, IBAN

#### Niveau 2 : Relance Ferme (J+30)
- **Délai** : 30 jours après échéance
- **Type** : Email + lettre recommandée (optionnel)
- **Ton** : Ferme ("Nous constatons que...")
- **Contenu** : Montant + pénalités, date limite, conséquences

#### Niveau 3 : Mise en Demeure (J+60)
- **Délai** : 60 jours après échéance
- **Type** : Lettre recommandée AR (obligatoire)
- **Ton** : Juridique ("Mise en demeure de...")
- **Contenu** : Montant total + pénalités, délai 8 jours, mention contentieux

---

### 1. Domain Layer - Entity PaymentReminder

**Fichier** : `backend/src/domain/entities/payment_reminder.rs`

```rust
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentReminder {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub expense_id: Uuid,
    pub owner_id: Uuid,
    pub reminder_level: ReminderLevel,
    pub amount_due: f64,
    pub penalties: f64,
    pub total_amount: f64,
    pub sent_at: DateTime<Utc>,
    pub sent_via: Vec<ReminderChannel>,
    pub letter_pdf_path: Option<String>,
    pub response_deadline: NaiveDate,
    pub status: ReminderStatus,
    pub notes: Option<String>,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "reminder_level", rename_all = "snake_case")]
pub enum ReminderLevel {
    FirstReminder,      // Relance 1 (amiable)
    SecondReminder,     // Relance 2 (ferme)
    FormalNotice,       // Mise en demeure
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "reminder_channel", rename_all = "snake_case")]
pub enum ReminderChannel {
    Email,
    Post,               // Courrier simple
    RegisteredMail,     // Recommandé AR
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "reminder_status", rename_all = "snake_case")]
pub enum ReminderStatus {
    Sent,
    Paid,               // Payé suite à relance
    Ignored,            // Pas de réponse
    Escalated,          // Escaladé (contentieux)
}

impl PaymentReminder {
    pub fn new(
        organization_id: Uuid,
        expense_id: Uuid,
        owner_id: Uuid,
        reminder_level: ReminderLevel,
        amount_due: f64,
        penalty_rate: f64,
        created_by: Uuid,
    ) -> Result<Self, String> {
        if amount_due <= 0.0 {
            return Err("Amount due must be positive".to_string());
        }

        let penalties = Self::calculate_penalties(amount_due, &reminder_level, penalty_rate);
        let total_amount = amount_due + penalties;

        let response_deadline = Self::calculate_response_deadline(&reminder_level);

        Ok(Self {
            id: Uuid::new_v4(),
            organization_id,
            expense_id,
            owner_id,
            reminder_level,
            amount_due,
            penalties,
            total_amount,
            sent_at: Utc::now(),
            sent_via: vec![ReminderChannel::Email],
            letter_pdf_path: None,
            response_deadline,
            status: ReminderStatus::Sent,
            notes: None,
            created_by,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }

    fn calculate_penalties(amount: f64, level: &ReminderLevel, penalty_rate: f64) -> f64 {
        match level {
            ReminderLevel::FirstReminder => 0.0, // Pas de pénalités relance 1
            ReminderLevel::SecondReminder => amount * penalty_rate, // Ex: 5%
            ReminderLevel::FormalNotice => amount * (penalty_rate * 2.0), // Ex: 10%
        }
    }

    fn calculate_response_deadline(level: &ReminderLevel) -> NaiveDate {
        let today = chrono::Local::now().naive_local().date();
        match level {
            ReminderLevel::FirstReminder => today + chrono::Duration::days(15),
            ReminderLevel::SecondReminder => today + chrono::Duration::days(10),
            ReminderLevel::FormalNotice => today + chrono::Duration::days(8), // Délai légal
        }
    }

    pub fn mark_paid(&mut self) {
        self.status = ReminderStatus::Paid;
        self.updated_at = Utc::now();
    }

    pub fn escalate(&mut self) {
        self.status = ReminderStatus::Escalated;
        self.updated_at = Utc::now();
    }

    pub fn is_overdue(&self) -> bool {
        let today = chrono::Local::now().naive_local().date();
        today > self.response_deadline && self.status == ReminderStatus::Sent
    }
}
```

---

### 2. Application Layer - Recovery Use Cases

**Fichier** : `backend/src/application/use_cases/recovery_use_cases.rs`

```rust
use crate::domain::entities::payment_reminder::*;
use crate::domain::entities::expense::*;
use crate::application::ports::expense_repository::ExpenseRepository;
use crate::application::ports::reminder_repository::PaymentReminderRepository;
use crate::application::ports::email_service::EmailService;
use crate::application::ports::pdf_generator::PdfGenerator;
use std::sync::Arc;
use uuid::Uuid;
use chrono::{Utc, Duration};

pub struct RecoveryUseCases {
    expense_repo: Arc<dyn ExpenseRepository>,
    reminder_repo: Arc<dyn PaymentReminderRepository>,
    email_service: Arc<dyn EmailService>,
    pdf_generator: Arc<dyn PdfGenerator>,
}

impl RecoveryUseCases {
    pub fn new(
        expense_repo: Arc<dyn ExpenseRepository>,
        reminder_repo: Arc<dyn PaymentReminderRepository>,
        email_service: Arc<dyn EmailService>,
        pdf_generator: Arc<dyn PdfGenerator>,
    ) -> Self {
        Self {
            expense_repo,
            reminder_repo,
            email_service,
            pdf_generator,
        }
    }

    /// Processus automatique quotidien : scanner expenses impayées et envoyer relances
    pub async fn process_overdue_expenses(&self) -> Result<RecoveryReport, String> {
        let today = Utc::now().naive_utc().date();
        let mut report = RecoveryReport {
            processed: 0,
            first_reminders: 0,
            second_reminders: 0,
            formal_notices: 0,
            errors: Vec::new(),
        };

        // Récupérer toutes les expenses impayées
        let overdue_expenses = self.expense_repo.find_all_overdue().await?;

        for expense in overdue_expenses {
            let days_overdue = (today - expense.due_date).num_days();

            // Vérifier quelle relance envoyer
            let reminder_level = if days_overdue >= 60 {
                ReminderLevel::FormalNotice
            } else if days_overdue >= 30 {
                ReminderLevel::SecondReminder
            } else if days_overdue >= 15 {
                ReminderLevel::FirstReminder
            } else {
                continue; // Pas encore 15 jours de retard
            };

            // Vérifier si relance déjà envoyée pour ce niveau
            if self.reminder_already_sent(expense.id, &reminder_level).await? {
                continue;
            }

            // Envoyer relance
            match self.send_reminder(expense.id, expense.owner_id, reminder_level.clone()).await {
                Ok(_) => {
                    report.processed += 1;
                    match reminder_level {
                        ReminderLevel::FirstReminder => report.first_reminders += 1,
                        ReminderLevel::SecondReminder => report.second_reminders += 1,
                        ReminderLevel::FormalNotice => report.formal_notices += 1,
                    }
                }
                Err(e) => {
                    report.errors.push(format!("Expense {}: {}", expense.id, e));
                }
            }
        }

        Ok(report)
    }

    async fn reminder_already_sent(
        &self,
        expense_id: Uuid,
        level: &ReminderLevel,
    ) -> Result<bool, String> {
        let reminders = self.reminder_repo.find_by_expense(expense_id).await?;
        Ok(reminders.iter().any(|r| &r.reminder_level == level))
    }

    async fn send_reminder(
        &self,
        expense_id: Uuid,
        owner_id: Uuid,
        level: ReminderLevel,
    ) -> Result<PaymentReminder, String> {
        // 1. Récupérer expense et owner
        let expense = self.expense_repo.find_by_id(expense_id).await?
            .ok_or("Expense not found")?;

        // 2. Créer reminder
        let reminder = PaymentReminder::new(
            expense.organization_id,
            expense_id,
            owner_id,
            level.clone(),
            expense.amount,
            0.05, // 5% pénalités (configurable)
            Uuid::nil(), // System user
        )?;

        // 3. Générer PDF lettre
        let pdf_path = self.pdf_generator.generate_reminder_letter(&reminder, &expense).await?;

        // 4. Envoyer email
        self.email_service.send_payment_reminder_email(owner_id, &reminder, &pdf_path).await?;

        // 5. Sauvegarder reminder
        let mut saved_reminder = self.reminder_repo.create(&reminder).await?;
        saved_reminder.letter_pdf_path = Some(pdf_path);
        self.reminder_repo.update(&saved_reminder).await?;

        Ok(saved_reminder)
    }

    pub async fn get_overdue_summary(
        &self,
        building_id: Uuid,
    ) -> Result<OverdueSummary, String> {
        let expenses = self.expense_repo.find_overdue_by_building(building_id).await?;

        let total_overdue = expenses.iter().map(|e| e.amount).sum();
        let count = expenses.len();

        let by_level = self.group_by_reminder_level(expenses).await?;

        Ok(OverdueSummary {
            total_overdue,
            count,
            no_reminder: by_level.0,
            first_reminder: by_level.1,
            second_reminder: by_level.2,
            formal_notice: by_level.3,
        })
    }

    async fn group_by_reminder_level(
        &self,
        expenses: Vec<Expense>,
    ) -> Result<(usize, usize, usize, usize), String> {
        let mut no_reminder = 0;
        let mut first = 0;
        let mut second = 0;
        let mut formal = 0;

        for expense in expenses {
            let reminders = self.reminder_repo.find_by_expense(expense.id).await?;
            if reminders.is_empty() {
                no_reminder += 1;
            } else {
                let max_level = reminders.iter().map(|r| &r.reminder_level).max();
                match max_level {
                    Some(ReminderLevel::FirstReminder) => first += 1,
                    Some(ReminderLevel::SecondReminder) => second += 1,
                    Some(ReminderLevel::FormalNotice) => formal += 1,
                    None => no_reminder += 1,
                }
            }
        }

        Ok((no_reminder, first, second, formal))
    }
}

#[derive(Debug, Serialize)]
pub struct RecoveryReport {
    pub processed: usize,
    pub first_reminders: usize,
    pub second_reminders: usize,
    pub formal_notices: usize,
    pub errors: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct OverdueSummary {
    pub total_overdue: f64,
    pub count: usize,
    pub no_reminder: usize,
    pub first_reminder: usize,
    pub second_reminder: usize,
    pub formal_notice: usize,
}
```

---

### 3. Migration SQL

**Fichier** : `backend/migrations/20251101000005_create_payment_reminders.sql`

```sql
CREATE TYPE reminder_level AS ENUM ('first_reminder', 'second_reminder', 'formal_notice');
CREATE TYPE reminder_channel AS ENUM ('email', 'post', 'registered_mail');
CREATE TYPE reminder_status AS ENUM ('sent', 'paid', 'ignored', 'escalated');

CREATE TABLE payment_reminders (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    expense_id UUID NOT NULL REFERENCES expenses(id) ON DELETE CASCADE,
    owner_id UUID NOT NULL REFERENCES owners(id) ON DELETE CASCADE,
    reminder_level reminder_level NOT NULL,
    amount_due DECIMAL(12, 2) NOT NULL,
    penalties DECIMAL(12, 2) NOT NULL DEFAULT 0,
    total_amount DECIMAL(12, 2) NOT NULL,
    sent_at TIMESTAMPTZ NOT NULL,
    sent_via reminder_channel[] NOT NULL,
    letter_pdf_path TEXT,
    response_deadline DATE NOT NULL,
    status reminder_status NOT NULL DEFAULT 'sent',
    notes TEXT,
    created_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_reminders_expense ON payment_reminders(expense_id);
CREATE INDEX idx_reminders_owner ON payment_reminders(owner_id);
CREATE INDEX idx_reminders_status ON payment_reminders(status);
CREATE INDEX idx_reminders_deadline ON payment_reminders(response_deadline);
```

---

## ✅ Critères d'Acceptation

### Fonctionnels
- [ ] Relances automatiques J+15, J+30, J+60
- [ ] PDF lettres conformes (3 templates)
- [ ] Emails automatiques envoyés
- [ ] Calcul pénalités retard (configurable)
- [ ] Dashboard impayés avec KPIs
- [ ] Export CSV liste impayés

### Techniques
- [ ] Cron job quotidien (2h du matin)
- [ ] Tests unitaires (15+ tests)
- [ ] Tests E2E workflow complet
- [ ] Performance: traitement 100 impayés < 30s

---

## 🔗 Dépendances

- ✅ Expense entity exists
- ✅ Owner entity exists
- Issue #009 : Notifications (emails)
- Issue #047 : PDF Generation (lettres)

---

## 🚀 Checklist

- [ ] 1. Créer entity `payment_reminder.rs`
- [ ] 2. Migration SQL
- [ ] 3. Repository + use cases
- [ ] 4. PDF templates (3 niveaux)
- [ ] 5. Cron job quotidien
- [ ] 6. Handlers HTTP
- [ ] 7. Tests (15+ tests)
- [ ] 8. Frontend: dashboard impayés
- [ ] 9. Frontend: historique relances
- [ ] 10. Commit : `feat: implement automated payment recovery workflow`

---

**Créé le** : 2025-11-01
**Milestone** : v1.0 - Financial Automation
**Impact** : CRITIQUE - Réduction impayés 30-50%
