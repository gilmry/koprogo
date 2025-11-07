use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Niveau de relance de paiement
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ReminderLevel {
    FirstReminder,  // J+15 - Rappel aimable
    SecondReminder, // J+30 - Relance ferme
    FormalNotice,   // J+60 - Mise en demeure légale
}

impl ReminderLevel {
    /// Nombre de jours après la date d'échéance pour chaque niveau
    pub fn days_after_due_date(&self) -> i64 {
        match self {
            ReminderLevel::FirstReminder => 15,
            ReminderLevel::SecondReminder => 30,
            ReminderLevel::FormalNotice => 60,
        }
    }

    /// Prochain niveau de relance (None si dernier niveau atteint)
    pub fn next_level(&self) -> Option<ReminderLevel> {
        match self {
            ReminderLevel::FirstReminder => Some(ReminderLevel::SecondReminder),
            ReminderLevel::SecondReminder => Some(ReminderLevel::FormalNotice),
            ReminderLevel::FormalNotice => None, // Dernier niveau - passer à huissier
        }
    }

    /// Ton du message pour chaque niveau
    pub fn tone(&self) -> &'static str {
        match self {
            ReminderLevel::FirstReminder => "aimable",
            ReminderLevel::SecondReminder => "ferme",
            ReminderLevel::FormalNotice => "juridique",
        }
    }
}

/// Statut d'une relance
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ReminderStatus {
    Pending,  // En attente d'envoi
    Sent,     // Envoyée
    Opened,   // Email ouvert par le destinataire
    Paid,     // Paiement reçu après relance
    Escalated, // Escaladé au niveau supérieur
    Cancelled, // Annulé (paiement reçu avant envoi)
}

/// Méthode d'envoi de la relance
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DeliveryMethod {
    Email,
    RegisteredLetter, // Lettre recommandée
    Bailiff,          // Huissier de justice
}

/// Représente une relance de paiement pour charges impayées
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PaymentReminder {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub expense_id: Uuid,
    pub owner_id: Uuid,
    pub level: ReminderLevel,
    pub status: ReminderStatus,
    pub amount_owed: f64,       // Montant dû (en euros)
    pub penalty_amount: f64,    // Pénalités de retard (8% annuel en Belgique)
    pub total_amount: f64,      // Montant total (owed + penalties)
    pub due_date: DateTime<Utc>, // Date d'échéance originale de la charge
    pub days_overdue: i64,      // Nombre de jours de retard
    pub delivery_method: DeliveryMethod,
    pub sent_date: Option<DateTime<Utc>>,
    pub opened_date: Option<DateTime<Utc>>,
    pub pdf_path: Option<String>, // Chemin vers le PDF de la lettre
    pub tracking_number: Option<String>, // Numéro de suivi (lettre recommandée)
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl PaymentReminder {
    /// Taux légal de pénalité de retard en Belgique (8% annuel)
    pub const BELGIAN_PENALTY_RATE: f64 = 0.08;

    /// Crée une nouvelle relance de paiement
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        organization_id: Uuid,
        expense_id: Uuid,
        owner_id: Uuid,
        level: ReminderLevel,
        amount_owed: f64,
        due_date: DateTime<Utc>,
        days_overdue: i64,
    ) -> Result<Self, String> {
        // Validation des business rules
        if amount_owed <= 0.0 {
            return Err("Amount owed must be greater than 0".to_string());
        }

        if days_overdue < 0 {
            return Err("Days overdue cannot be negative".to_string());
        }

        // Vérifier que le niveau de relance correspond au nombre de jours de retard
        let expected_days = level.days_after_due_date();
        if days_overdue < expected_days {
            return Err(format!(
                "Cannot create {} reminder before {} days overdue (currently {} days)",
                match level {
                    ReminderLevel::FirstReminder => "first",
                    ReminderLevel::SecondReminder => "second",
                    ReminderLevel::FormalNotice => "formal notice",
                },
                expected_days,
                days_overdue
            ));
        }

        // Calculer les pénalités de retard (taux légal belge: 8% annuel)
        let penalty_amount = Self::calculate_penalty(amount_owed, days_overdue);
        let total_amount = amount_owed + penalty_amount;

        // Déterminer la méthode de livraison selon le niveau
        let delivery_method = match level {
            ReminderLevel::FirstReminder => DeliveryMethod::Email,
            ReminderLevel::SecondReminder => DeliveryMethod::Email,
            ReminderLevel::FormalNotice => DeliveryMethod::RegisteredLetter,
        };

        let now = Utc::now();
        Ok(Self {
            id: Uuid::new_v4(),
            organization_id,
            expense_id,
            owner_id,
            level,
            status: ReminderStatus::Pending,
            amount_owed,
            penalty_amount,
            total_amount,
            due_date,
            days_overdue,
            delivery_method,
            sent_date: None,
            opened_date: None,
            pdf_path: None,
            tracking_number: None,
            notes: None,
            created_at: now,
            updated_at: now,
        })
    }

    /// Calcule les pénalités de retard selon le taux légal belge (8% annuel)
    /// Formule: pénalité = montant * 0.08 * (jours_retard / 365)
    pub fn calculate_penalty(amount: f64, days_overdue: i64) -> f64 {
        if days_overdue <= 0 {
            return 0.0;
        }
        let yearly_penalty = amount * Self::BELGIAN_PENALTY_RATE;
        let daily_penalty = yearly_penalty / 365.0;
        (daily_penalty * days_overdue as f64 * 100.0).round() / 100.0 // Arrondi à 2 décimales
    }

    /// Marque la relance comme envoyée
    pub fn mark_as_sent(&mut self, pdf_path: Option<String>) -> Result<(), String> {
        if self.status != ReminderStatus::Pending {
            return Err(format!(
                "Cannot mark reminder as sent: current status is {:?}",
                self.status
            ));
        }

        self.status = ReminderStatus::Sent;
        self.sent_date = Some(Utc::now());
        self.pdf_path = pdf_path;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Marque la relance comme ouverte (email ouvert)
    pub fn mark_as_opened(&mut self) -> Result<(), String> {
        if self.status != ReminderStatus::Sent {
            return Err(format!(
                "Cannot mark reminder as opened: must be sent first (current status: {:?})",
                self.status
            ));
        }

        self.status = ReminderStatus::Opened;
        self.opened_date = Some(Utc::now());
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Marque la relance comme payée
    pub fn mark_as_paid(&mut self) -> Result<(), String> {
        match self.status {
            ReminderStatus::Sent | ReminderStatus::Opened | ReminderStatus::Pending => {
                self.status = ReminderStatus::Paid;
                self.updated_at = Utc::now();
                Ok(())
            }
            ReminderStatus::Paid => Err("Reminder is already marked as paid".to_string()),
            ReminderStatus::Escalated => {
                Err("Cannot mark escalated reminder as paid".to_string())
            }
            ReminderStatus::Cancelled => {
                Err("Cannot mark cancelled reminder as paid".to_string())
            }
        }
    }

    /// Escalade vers le niveau de relance supérieur
    pub fn escalate(&mut self) -> Result<Option<ReminderLevel>, String> {
        if self.status == ReminderStatus::Paid || self.status == ReminderStatus::Cancelled {
            return Err(format!(
                "Cannot escalate reminder with status {:?}",
                self.status
            ));
        }

        self.status = ReminderStatus::Escalated;
        self.updated_at = Utc::now();
        Ok(self.level.next_level())
    }

    /// Annule la relance (paiement reçu avant envoi)
    pub fn cancel(&mut self, reason: String) -> Result<(), String> {
        if self.status == ReminderStatus::Sent || self.status == ReminderStatus::Opened {
            return Err("Cannot cancel reminder that has already been sent".to_string());
        }

        self.status = ReminderStatus::Cancelled;
        self.notes = Some(reason);
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Ajoute un numéro de suivi (pour lettre recommandée)
    pub fn set_tracking_number(&mut self, tracking_number: String) -> Result<(), String> {
        if self.delivery_method != DeliveryMethod::RegisteredLetter {
            return Err("Tracking number is only valid for registered letters".to_string());
        }

        self.tracking_number = Some(tracking_number);
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Vérifie si la relance nécessite une escalade
    pub fn needs_escalation(&self, current_date: DateTime<Utc>) -> bool {
        if self.status != ReminderStatus::Sent && self.status != ReminderStatus::Opened {
            return false;
        }

        if let Some(sent_date) = self.sent_date {
            let days_since_sent = (current_date - sent_date).num_days();
            // Escalader si pas de réponse après 15 jours
            days_since_sent >= 15 && self.level.next_level().is_some()
        } else {
            false
        }
    }

    /// Recalcule les pénalités en fonction du nombre de jours actuel
    pub fn recalculate_penalties(&mut self, current_days_overdue: i64) {
        self.days_overdue = current_days_overdue;
        self.penalty_amount = Self::calculate_penalty(self.amount_owed, current_days_overdue);
        self.total_amount = self.amount_owed + self.penalty_amount;
        self.updated_at = Utc::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_payment_reminder_success() {
        let org_id = Uuid::new_v4();
        let expense_id = Uuid::new_v4();
        let owner_id = Uuid::new_v4();
        let due_date = Utc::now() - chrono::Duration::days(20);

        let reminder = PaymentReminder::new(
            org_id,
            expense_id,
            owner_id,
            ReminderLevel::FirstReminder,
            100.0,
            due_date,
            20,
        );

        assert!(reminder.is_ok());
        let reminder = reminder.unwrap();
        assert_eq!(reminder.status, ReminderStatus::Pending);
        assert_eq!(reminder.level, ReminderLevel::FirstReminder);
        assert_eq!(reminder.delivery_method, DeliveryMethod::Email);
    }

    #[test]
    fn test_create_reminder_too_early() {
        let org_id = Uuid::new_v4();
        let expense_id = Uuid::new_v4();
        let owner_id = Uuid::new_v4();
        let due_date = Utc::now() - chrono::Duration::days(10);

        let reminder = PaymentReminder::new(
            org_id,
            expense_id,
            owner_id,
            ReminderLevel::FirstReminder,
            100.0,
            due_date,
            10, // Moins de 15 jours
        );

        assert!(reminder.is_err());
        assert!(reminder
            .unwrap_err()
            .contains("Cannot create first reminder before"));
    }

    #[test]
    fn test_calculate_penalty() {
        // 100€, 30 jours de retard, taux 8% annuel
        // Pénalité = 100 * 0.08 * (30/365) = 0.66€
        let penalty = PaymentReminder::calculate_penalty(100.0, 30);
        assert!((penalty - 0.66).abs() < 0.01);

        // 1000€, 365 jours de retard (1 an)
        // Pénalité = 1000 * 0.08 * 1 = 80€
        let penalty = PaymentReminder::calculate_penalty(1000.0, 365);
        assert!((penalty - 80.0).abs() < 0.01);
    }

    #[test]
    fn test_mark_as_sent() {
        let org_id = Uuid::new_v4();
        let expense_id = Uuid::new_v4();
        let owner_id = Uuid::new_v4();
        let due_date = Utc::now() - chrono::Duration::days(20);

        let mut reminder = PaymentReminder::new(
            org_id,
            expense_id,
            owner_id,
            ReminderLevel::FirstReminder,
            100.0,
            due_date,
            20,
        )
        .unwrap();

        let result = reminder.mark_as_sent(Some("/path/to/pdf".to_string()));
        assert!(result.is_ok());
        assert_eq!(reminder.status, ReminderStatus::Sent);
        assert!(reminder.sent_date.is_some());
        assert_eq!(reminder.pdf_path, Some("/path/to/pdf".to_string()));
    }

    #[test]
    fn test_escalate() {
        let org_id = Uuid::new_v4();
        let expense_id = Uuid::new_v4();
        let owner_id = Uuid::new_v4();
        let due_date = Utc::now() - chrono::Duration::days(20);

        let mut reminder = PaymentReminder::new(
            org_id,
            expense_id,
            owner_id,
            ReminderLevel::FirstReminder,
            100.0,
            due_date,
            20,
        )
        .unwrap();

        reminder.mark_as_sent(None).unwrap();

        let next_level = reminder.escalate().unwrap();
        assert_eq!(next_level, Some(ReminderLevel::SecondReminder));
        assert_eq!(reminder.status, ReminderStatus::Escalated);
    }

    #[test]
    fn test_reminder_level_days() {
        assert_eq!(ReminderLevel::FirstReminder.days_after_due_date(), 15);
        assert_eq!(ReminderLevel::SecondReminder.days_after_due_date(), 30);
        assert_eq!(ReminderLevel::FormalNotice.days_after_due_date(), 60);
    }

    #[test]
    fn test_needs_escalation() {
        let org_id = Uuid::new_v4();
        let expense_id = Uuid::new_v4();
        let owner_id = Uuid::new_v4();
        let due_date = Utc::now() - chrono::Duration::days(20);

        let mut reminder = PaymentReminder::new(
            org_id,
            expense_id,
            owner_id,
            ReminderLevel::FirstReminder,
            100.0,
            due_date,
            20,
        )
        .unwrap();

        // Pas d'escalade si pas envoyé
        assert!(!reminder.needs_escalation(Utc::now()));

        // Marquer comme envoyé
        reminder.mark_as_sent(None).unwrap();

        // Pas d'escalade immédiatement après envoi
        assert!(!reminder.needs_escalation(Utc::now()));

        // Escalade nécessaire après 15 jours
        let future_date = Utc::now() + chrono::Duration::days(16);
        assert!(reminder.needs_escalation(future_date));
    }

    #[test]
    fn test_recalculate_penalties() {
        let org_id = Uuid::new_v4();
        let expense_id = Uuid::new_v4();
        let owner_id = Uuid::new_v4();
        let due_date = Utc::now() - chrono::Duration::days(20);

        let mut reminder = PaymentReminder::new(
            org_id,
            expense_id,
            owner_id,
            ReminderLevel::FirstReminder,
            100.0,
            due_date,
            20,
        )
        .unwrap();

        let initial_penalty = reminder.penalty_amount;

        // Recalculer avec plus de jours de retard
        reminder.recalculate_penalties(40);

        assert_eq!(reminder.days_overdue, 40);
        assert!(reminder.penalty_amount > initial_penalty);
        assert_eq!(reminder.total_amount, reminder.amount_owed + reminder.penalty_amount);
    }

    #[test]
    fn test_formal_notice_uses_registered_letter() {
        let org_id = Uuid::new_v4();
        let expense_id = Uuid::new_v4();
        let owner_id = Uuid::new_v4();
        let due_date = Utc::now() - chrono::Duration::days(70);

        let reminder = PaymentReminder::new(
            org_id,
            expense_id,
            owner_id,
            ReminderLevel::FormalNotice,
            100.0,
            due_date,
            70,
        )
        .unwrap();

        assert_eq!(reminder.delivery_method, DeliveryMethod::RegisteredLetter);
    }
}
