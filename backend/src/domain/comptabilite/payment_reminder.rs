use chrono::{DateTime, Utc};
use rust_decimal::{Decimal, RoundingStrategy};
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Niveau de relance de paiement
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, utoipa::ToSchema)]
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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, utoipa::ToSchema)]
pub enum ReminderStatus {
    Pending,   // En attente d'envoi
    Sent,      // Envoyée
    Opened,    // Email ouvert par le destinataire
    Paid,      // Paiement reçu après relance
    Escalated, // Escaladé au niveau supérieur
    Cancelled, // Annulé (paiement reçu avant envoi)
}

/// Méthode d'envoi de la relance
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, utoipa::ToSchema)]
pub enum DeliveryMethod {
    Email,
    RegisteredLetter, // Lettre recommandée
    Bailiff,          // Huissier de justice
}

/// Représente une relance de paiement pour charges impayées
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PaymentReminder {
    pub id: Uuid,

    /// L'ACP au profit de laquelle la somme est réclamée.
    ///
    /// Art. 3.86 § 3 : « Le syndic peut prendre toutes les mesures judiciaires
    /// et extrajudiciaires pour la récupération des charges ». Il recouvre
    /// **pour** l'association ; la créance et les pénalités lui reviennent.
    /// Cf. ADR-0045.
    pub acp_id: Uuid,

    /// Le syndic qui a émis la relance, conservé comme trace d'auteur.
    pub organization_id: Uuid,
    pub expense_id: Uuid,
    pub owner_id: Uuid,
    pub level: ReminderLevel,
    pub status: ReminderStatus,
    /// Montant dû (en euros). `Decimal` — ADR-0007/0008 : montant opposable.
    pub amount_owed: Decimal,
    /// Pénalités de retard (taux légal civil belge, 4,5 % annuel en 2026).
    pub penalty_amount: Decimal,
    /// Montant total (`amount_owed + penalty_amount`). L'égalité est garantie
    /// par un `CHECK` en base, exact depuis le passage en `NUMERIC`.
    pub total_amount: Decimal,
    pub due_date: DateTime<Utc>, // Date d'échéance originale de la charge
    pub days_overdue: i64,       // Nombre de jours de retard
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
    /// Taux légal civil de pénalité de retard en Belgique (Moniteur belge)
    /// Ce taux est publié annuellement par Arrêté Royal.
    /// 2024: 5.25%, 2025: 4.0%, 2026: 4.5%
    /// A mettre à jour chaque année selon publication au Moniteur belge.
    pub const BELGIAN_PENALTY_RATE: Decimal = dec!(0.045);

    /// Nombre de jours de l'année servant de base au prorata du taux annuel.
    const DAYS_PER_YEAR: Decimal = dec!(365);

    /// Crée une nouvelle relance de paiement
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        acp_id: Uuid,
        organization_id: Uuid,
        expense_id: Uuid,
        owner_id: Uuid,
        level: ReminderLevel,
        amount_owed: Decimal,
        due_date: DateTime<Utc>,
        days_overdue: i64,
    ) -> Result<Self, String> {
        // Validation des business rules
        if amount_owed <= Decimal::ZERO {
            return Err("Amount owed must be greater than 0".to_string());
        }

        // Borne au centime : reprise de l'invariant que portait
        // `#[validate(range(min = 0.01))]` côté DTO avant #661 — `validator` ne
        // sachant pas borner un `Decimal`, la règle descend dans le domaine
        // plutôt que de disparaître. Une relance pour moins d'un centime n'a
        // aucun sens (frais d'envoi, lettre recommandée, huissier).
        if amount_owed < dec!(0.01) {
            return Err("Amount owed must be at least 0.01".to_string());
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

        // Calculer les pénalités de retard (taux légal civil belge: 4.5% annuel en 2026)
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
            acp_id,
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

    /// Calcule les pénalités de retard selon le taux légal civil belge
    /// (4,5 % annuel en 2026).
    ///
    /// Formule : pénalité = montant × 0,045 × (jours_retard / 365), arrondie au
    /// centime.
    ///
    /// Calcul en `Decimal` (suite #661) : c'est un montant **réclamé à un
    /// copropriétaire**, et l'ancienne version arrondissait via
    /// `(x * 100.0).round() / 100.0` en `f64` — un motif qui produit des écarts
    /// d'un centime sur des valeurs parfaitement ordinaires, et qui n'arrondit
    /// pas au plus proche de façon fiable près des demis.
    ///
    /// L'arrondi est **`MidpointAwayFromZero`** (arrondi commercial : 0,005 €
    /// donne 0,01 €), et non le « banker's rounding » que `round_dp` applique
    /// par défaut : sur une somme due, arrondir la moitié vers le pair n'a
    /// aucun fondement, et diverge de ce que produit un tableur.
    pub fn calculate_penalty(amount: Decimal, days_overdue: i64) -> Decimal {
        if days_overdue <= 0 {
            return Decimal::ZERO;
        }
        let yearly_penalty = amount * Self::BELGIAN_PENALTY_RATE;
        let daily_penalty = yearly_penalty / Self::DAYS_PER_YEAR;
        (daily_penalty * Decimal::from(days_overdue))
            .round_dp_with_strategy(2, RoundingStrategy::MidpointAwayFromZero)
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
            ReminderStatus::Escalated => Err("Cannot mark escalated reminder as paid".to_string()),
            ReminderStatus::Cancelled => Err("Cannot mark cancelled reminder as paid".to_string()),
        }
    }

    /// Escalade vers le niveau de relance supérieur
    /// Vérifie sans muter qu'une escalade est permise.
    ///
    /// Extrait de `escalate` pour que l'appelant puisse contrôler l'ordre :
    /// la couche application doit refuser un dossier soldé AVANT de construire
    /// le niveau suivant, sinon un dossier payé se voit reprocher son délai
    /// plutôt que son statut — un message qui envoie chercher le problème au
    /// mauvais endroit.
    pub fn can_escalate(&self) -> Result<(), String> {
        if self.status == ReminderStatus::Paid || self.status == ReminderStatus::Cancelled {
            return Err(format!(
                "Cannot escalate reminder with status {:?}",
                self.status
            ));
        }
        Ok(())
    }

    pub fn escalate(&mut self) -> Result<Option<ReminderLevel>, String> {
        self.can_escalate()?;

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

impl crate::domain::services::PieceDeGestion for PaymentReminder {
    fn acp_id(&self) -> Uuid {
        self.acp_id
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
            Uuid::new_v4(), // acp_id
            org_id,
            expense_id,
            owner_id,
            ReminderLevel::FirstReminder,
            dec!(100),
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
            Uuid::new_v4(), // acp_id
            org_id,
            expense_id,
            owner_id,
            ReminderLevel::FirstReminder,
            dec!(100),
            due_date,
            10, // Moins de 15 jours
        );

        assert!(reminder.is_err());
        assert!(reminder
            .unwrap_err()
            .contains("Cannot create first reminder before"));
    }

    /// @happy — le calcul de pénalité au taux légal civil belge.
    ///
    /// Assertions en **égalité `Decimal` exacte** : les tolérances `< 0.01`
    /// précédentes acceptaient un écart d'un centime sur une somme réclamée à
    /// un copropriétaire, c'est-à-dire précisément l'erreur qu'un calcul en
    /// `f64` produit.
    #[test]
    fn test_calculate_penalty() {
        // 100€, 30 jours : 100 × 0,045 × (30/365) = 0,369863… → 0,37 €
        assert_eq!(
            PaymentReminder::calculate_penalty(dec!(100), 30),
            dec!(0.37)
        );

        // 1000€, 365 jours (1 an pile) : 1000 × 0,045 = 45,00 € exactement.
        assert_eq!(
            PaymentReminder::calculate_penalty(dec!(1000), 365),
            dec!(45.00)
        );
    }

    /// @edge — aucune pénalité sans retard, et pas de valeur négative.
    #[test]
    fn test_calculate_penalty_no_overdue_days() {
        assert_eq!(
            PaymentReminder::calculate_penalty(dec!(100), 0),
            Decimal::ZERO
        );
        assert_eq!(
            PaymentReminder::calculate_penalty(dec!(100), -5),
            Decimal::ZERO
        );
    }

    /// @edge — l'arrondi au centime est **commercial** (`MidpointAwayFromZero`),
    /// pas « banker's rounding ».
    ///
    /// Cas construit pour tomber exactement sur un demi-centime :
    /// 8,11111…€ × 0,045 × (1/365) n'est pas un demi ; on utilise donc un
    /// montant qui produit une fraction se terminant par 5 au millième.
    /// 1000 € sur 81 jours → 1000 × 0,045 × 81/365 = 9,98630136…€ → 9,99 €.
    #[test]
    fn test_calculate_penalty_rounds_to_the_cent() {
        assert_eq!(
            PaymentReminder::calculate_penalty(dec!(1000), 81),
            dec!(9.99)
        );

        // Le résultat n'a jamais plus de 2 décimales — invariant de la colonne
        // NUMERIC(12,2) qui le stocke.
        let p = PaymentReminder::calculate_penalty(dec!(1234.56), 137);
        assert_eq!(p.scale(), 2, "la pénalité doit être arrondie au centime");
    }

    /// @security — un montant dû doit valoir au moins un centime. Cet invariant
    /// était porté par `#[validate(range(min = 0.01))]` sur le DTO ; `validator`
    /// ne sachant pas borner un `Decimal`, il vit désormais dans le domaine —
    /// où il s'applique à TOUS les appelants, pas seulement à la route HTTP.
    #[test]
    fn test_reminder_rejects_amounts_below_one_cent() {
        let due_date = Utc::now() - chrono::Duration::days(20);
        for amount in [dec!(0), dec!(-100), dec!(0.009)] {
            let r = PaymentReminder::new(
                Uuid::new_v4(), // acp_id
                Uuid::new_v4(),
                Uuid::new_v4(),
                Uuid::new_v4(),
                ReminderLevel::FirstReminder,
                amount,
                due_date,
                20,
            );
            assert!(r.is_err(), "montant {amount} aurait dû être rejeté");
        }
    }

    /// @negative — le total reste exactement la somme de ses composantes.
    /// C'est l'égalité que la contrainte `CHECK` vérifie en base : en
    /// `DOUBLE PRECISION` elle pouvait échouer sur une ligne valide.
    #[test]
    fn test_total_amount_equals_owed_plus_penalty_exactly() {
        let due_date = Utc::now() - chrono::Duration::days(90);
        let r = PaymentReminder::new(
            Uuid::new_v4(), // acp_id
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            ReminderLevel::FormalNotice,
            dec!(1234.56),
            due_date,
            90,
        )
        .unwrap();

        assert_eq!(r.total_amount, r.amount_owed + r.penalty_amount);
        assert_eq!(r.amount_owed, dec!(1234.56));
    }

    #[test]
    fn test_mark_as_sent() {
        let org_id = Uuid::new_v4();
        let expense_id = Uuid::new_v4();
        let owner_id = Uuid::new_v4();
        let due_date = Utc::now() - chrono::Duration::days(20);

        let mut reminder = PaymentReminder::new(
            Uuid::new_v4(), // acp_id
            org_id,
            expense_id,
            owner_id,
            ReminderLevel::FirstReminder,
            dec!(100),
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
            Uuid::new_v4(), // acp_id
            org_id,
            expense_id,
            owner_id,
            ReminderLevel::FirstReminder,
            dec!(100),
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
            Uuid::new_v4(), // acp_id
            org_id,
            expense_id,
            owner_id,
            ReminderLevel::FirstReminder,
            dec!(100),
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
            Uuid::new_v4(), // acp_id
            org_id,
            expense_id,
            owner_id,
            ReminderLevel::FirstReminder,
            dec!(100),
            due_date,
            20,
        )
        .unwrap();

        let initial_penalty = reminder.penalty_amount;

        // Recalculer avec plus de jours de retard
        reminder.recalculate_penalties(40);

        assert_eq!(reminder.days_overdue, 40);
        assert!(reminder.penalty_amount > initial_penalty);
        assert_eq!(
            reminder.total_amount,
            reminder.amount_owed + reminder.penalty_amount
        );
    }

    #[test]
    fn test_formal_notice_uses_registered_letter() {
        let org_id = Uuid::new_v4();
        let expense_id = Uuid::new_v4();
        let owner_id = Uuid::new_v4();
        let due_date = Utc::now() - chrono::Duration::days(70);

        let reminder = PaymentReminder::new(
            Uuid::new_v4(), // acp_id
            org_id,
            expense_id,
            owner_id,
            ReminderLevel::FormalNotice,
            dec!(100),
            due_date,
            70,
        )
        .unwrap();

        assert_eq!(reminder.delivery_method, DeliveryMethod::RegisteredLetter);
    }
}
