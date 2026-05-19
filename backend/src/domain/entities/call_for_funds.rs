// Domain Entity: Call for Funds (Appel de Fonds)
//
// Represents a collective payment request sent by the Syndic to all owners
// This is the "master" entity that generates individual OwnerContribution records
//
// MONETARY: total_amount uses rust_decimal::Decimal (cf. ADR-0007).

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::ContributionType;

/// Status of the call for funds
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum CallForFundsStatus {
    /// Draft - not yet sent
    Draft,
    /// Sent to owners
    Sent,
    /// Partially paid
    Partial,
    /// Fully paid by all owners
    Completed,
    /// Cancelled
    Cancelled,
}

/// Call for Funds (Appel de Fonds Collectif)
///
/// Represents a payment request sent by the Syndic to all owners of a building
/// Automatically generates individual OwnerContribution records based on ownership percentages
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CallForFunds {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub building_id: Uuid,

    // Description
    pub title: String,
    pub description: String,

    // Financial details
    pub total_amount: Decimal, // Total amount to be collected from ALL owners

    // Type
    pub contribution_type: ContributionType,

    // Dates
    pub call_date: DateTime<Utc>,         // When the call is issued
    pub due_date: DateTime<Utc>,          // Payment deadline
    pub sent_date: Option<DateTime<Utc>>, // When actually sent to owners

    // Status
    pub status: CallForFundsStatus,

    // Accounting
    pub account_code: Option<String>, // PCMN code (classe 7)

    // Metadata
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<Uuid>,
}

/// Domain-typed validation error for calls for funds (appel de fonds).
///
/// Pure domain type — no infra/application dependency (hexagonal purity).
/// Précédent `JournalEntryError`/`ChargeDistributionError` (#433 / WP-A6
/// EXP-008) → 400 validation, jamais 500 Internal.
#[derive(Debug, Clone, PartialEq)]
pub enum CallForFundsError {
    /// Montant total non strictement positif.
    NonPositiveTotalAmount,
    /// Titre vide.
    EmptyTitle,
    /// Description vide.
    EmptyDescription,
    /// Date d'échéance ≤ date d'appel (fenêtre de paiement invalide).
    DueDateNotAfterCallDate,
}

impl std::fmt::Display for CallForFundsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NonPositiveTotalAmount => write!(f, "Total amount must be positive"),
            Self::EmptyTitle => write!(f, "Title cannot be empty"),
            Self::EmptyDescription => write!(f, "Description cannot be empty"),
            Self::DueDateNotAfterCallDate => {
                write!(f, "Due date must be after call date")
            }
        }
    }
}

impl std::error::Error for CallForFundsError {}

/// Bridge : use-cases/ports `Result<_, String>` inchangés (cascade
/// String→AppError = slice large différée, précédent WP-A3/A4/A5).
impl From<CallForFundsError> for String {
    fn from(e: CallForFundsError) -> String {
        e.to_string()
    }
}

impl CallForFunds {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        organization_id: Uuid,
        building_id: Uuid,
        title: String,
        description: String,
        total_amount: Decimal,
        contribution_type: ContributionType,
        call_date: DateTime<Utc>,
        due_date: DateTime<Utc>,
        account_code: Option<String>,
    ) -> Result<Self, CallForFundsError> {
        // Validate total amount is positive
        if total_amount <= Decimal::ZERO {
            return Err(CallForFundsError::NonPositiveTotalAmount);
        }

        // Validate title
        if title.trim().is_empty() {
            return Err(CallForFundsError::EmptyTitle);
        }

        // Validate description
        if description.trim().is_empty() {
            return Err(CallForFundsError::EmptyDescription);
        }

        // Validate dates
        if due_date <= call_date {
            return Err(CallForFundsError::DueDateNotAfterCallDate);
        }

        Ok(Self {
            id: Uuid::new_v4(),
            organization_id,
            building_id,
            title,
            description,
            total_amount,
            contribution_type,
            call_date,
            due_date,
            sent_date: None,
            status: CallForFundsStatus::Draft,
            account_code,
            notes: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            created_by: None,
        })
    }

    /// Mark as sent to owners
    pub fn mark_as_sent(&mut self) {
        self.sent_date = Some(Utc::now());
        self.status = CallForFundsStatus::Sent;
        self.updated_at = Utc::now();
    }

    /// Mark as completed (all owners paid)
    pub fn mark_as_completed(&mut self) {
        self.status = CallForFundsStatus::Completed;
        self.updated_at = Utc::now();
    }

    /// Mark as cancelled
    pub fn cancel(&mut self) {
        self.status = CallForFundsStatus::Cancelled;
        self.updated_at = Utc::now();
    }

    /// Check if overdue (past due date and not completed)
    pub fn is_overdue(&self) -> bool {
        self.status != CallForFundsStatus::Completed
            && self.status != CallForFundsStatus::Cancelled
            && Utc::now() > self.due_date
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::ContributionType;

    #[test]
    fn test_create_call_for_funds_success() {
        let call_date = Utc::now();
        let due_date = call_date + chrono::Duration::days(30);

        let call = CallForFunds::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "Appel de fonds Q1 2025".to_string(),
            "Charges courantes trimestrielles".to_string(),
            rust_decimal_macros::dec!(5000),
            ContributionType::Regular,
            call_date,
            due_date,
            Some("7000".to_string()),
        );

        assert!(call.is_ok());
        let call = call.unwrap();
        assert_eq!(call.total_amount, rust_decimal_macros::dec!(5000));
        assert_eq!(call.status, CallForFundsStatus::Draft);
    }

    #[test]
    fn test_create_call_negative_amount() {
        let call_date = Utc::now();
        let due_date = call_date + chrono::Duration::days(30);

        let call = CallForFunds::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "Test".to_string(),
            "Test".to_string(),
            rust_decimal_macros::dec!(-100),
            ContributionType::Regular,
            call_date,
            due_date,
            None,
        );

        assert!(matches!(
            call.unwrap_err(),
            CallForFundsError::NonPositiveTotalAmount
        ));
    }

    #[test]
    fn test_create_call_invalid_dates() {
        let call_date = Utc::now();
        let due_date = call_date - chrono::Duration::days(1); // Due date BEFORE call date

        let call = CallForFunds::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "Test".to_string(),
            "Test".to_string(),
            rust_decimal_macros::dec!(100),
            ContributionType::Regular,
            call_date,
            due_date,
            None,
        );

        assert!(matches!(
            call.unwrap_err(),
            CallForFundsError::DueDateNotAfterCallDate
        ));
    }

    #[test]
    fn test_mark_as_sent() {
        let call_date = Utc::now();
        let due_date = call_date + chrono::Duration::days(30);

        let mut call = CallForFunds::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "Test".to_string(),
            "Test".to_string(),
            rust_decimal_macros::dec!(100),
            ContributionType::Regular,
            call_date,
            due_date,
            None,
        )
        .unwrap();

        assert_eq!(call.status, CallForFundsStatus::Draft);
        assert!(call.sent_date.is_none());

        call.mark_as_sent();

        assert_eq!(call.status, CallForFundsStatus::Sent);
        assert!(call.sent_date.is_some());
    }

    #[test]
    fn test_is_overdue() {
        let call_date = Utc::now() - chrono::Duration::days(60);
        let due_date = Utc::now() - chrono::Duration::days(30); // 30 days ago

        let call = CallForFunds::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "Overdue call".to_string(),
            "Test".to_string(),
            rust_decimal_macros::dec!(100),
            ContributionType::Regular,
            call_date,
            due_date,
            None,
        )
        .unwrap();

        assert!(call.is_overdue());
    }

    // ------------------------------------------------------------------------
    // 4 catégories #433/WP-A6 EXP-008 — erreur typée (CRITICAL.md #3).
    // Entité déjà Decimal (ADR-0007) ; ce WP type l'erreur domaine.
    // ------------------------------------------------------------------------

    fn mk(amount: Decimal, days: i64) -> Result<CallForFunds, CallForFundsError> {
        let call_date = Utc::now();
        CallForFunds::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "Appel".to_string(),
            "Charges".to_string(),
            amount,
            ContributionType::Regular,
            call_date,
            call_date + chrono::Duration::days(days),
            None,
        )
    }

    /// @happy — appel nominal : total_amount Decimal exact.
    #[test]
    fn happy_total_amount_decimal_exact() {
        let c = mk(rust_decimal_macros::dec!(9876.54), 30).unwrap();
        assert_eq!(c.total_amount, rust_decimal_macros::dec!(9876.54));
        assert_eq!(c.status, CallForFundsStatus::Draft);
    }

    /// @edge — montant minimal strictement positif accepté ; exactitude
    /// Decimal sur cumul (0.1+0.2=0.3, f64 échoue).
    #[test]
    fn edge_min_positive_and_decimal_exactness() {
        assert!(mk(rust_decimal_macros::dec!(0.01), 1).is_ok());
        let c = mk(
            rust_decimal_macros::dec!(0.1) + rust_decimal_macros::dec!(0.2),
            7,
        )
        .unwrap();
        assert_eq!(c.total_amount, rust_decimal_macros::dec!(0.3));
    }

    /// @negative — total ≤ 0, titre/description vides, échéance ≤ appel
    /// rejetés (erreurs typées, pas de panic).
    #[test]
    fn negative_invalid_inputs_rejected() {
        assert!(matches!(
            mk(Decimal::ZERO, 30).unwrap_err(),
            CallForFundsError::NonPositiveTotalAmount
        ));
        assert!(matches!(
            mk(rust_decimal_macros::dec!(100), -1).unwrap_err(),
            CallForFundsError::DueDateNotAfterCallDate
        ));
    }

    /// @security — un appel de fonds falsifié à montant nul/négatif
    /// (collecte fantôme) ne peut jamais être créé.
    #[test]
    fn security_tampered_nonpositive_amount_rejected() {
        assert!(matches!(
            mk(rust_decimal_macros::dec!(-1), 30).unwrap_err(),
            CallForFundsError::NonPositiveTotalAmount
        ));
    }
}
