// Domain Entity: Owner Contribution
//
// Represents payments made BY owners TO the ACP (incoming money = revenue)
// Complements Expense entity which represents payments made BY ACP TO suppliers (outgoing money = charges)
//
// Maps to PCMN classe 7 (Produits/Revenue)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Type of owner contribution
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ContributionType {
    /// Regular quarterly fees (appels de fonds ordinaires)
    Regular,
    /// Extraordinary fees for special works (appels de fonds extraordinaires)
    Extraordinary,
    /// Advance payment
    Advance,
    /// Adjustment (regularisation)
    Adjustment,
}

/// Payment status for contributions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ContributionPaymentStatus {
    /// Not yet paid
    Pending,
    /// Fully paid
    Paid,
    /// Partially paid
    Partial,
    /// Cancelled
    Cancelled,
}

/// Payment method for contributions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ContributionPaymentMethod {
    /// Bank transfer (virement)
    BankTransfer,
    /// Cash (espèces)
    Cash,
    /// Check (chèque)
    Check,
    /// Direct debit (domiciliation)
    Domiciliation,
}

/// Owner contribution (appel de fonds / cotisation)
///
/// Represents money paid BY owners TO the ACP (REVENUE - classe 7 PCMN)
/// This is the opposite of Expense which represents money paid BY ACP TO suppliers
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OwnerContribution {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub owner_id: Uuid,
    pub unit_id: Option<Uuid>,

    // Financial details
    pub description: String,
    pub amount: f64,

    // Accounting
    /// PCMN code (classe 7 - Produits)
    /// Examples: "7000" = regular fees, "7100" = extraordinary fees
    pub account_code: Option<String>,

    // Contribution details
    pub contribution_type: ContributionType,

    // Dates
    pub contribution_date: DateTime<Utc>, // When due/requested
    pub payment_date: Option<DateTime<Utc>>, // When actually paid

    // Payment details
    pub payment_method: Option<ContributionPaymentMethod>,
    pub payment_reference: Option<String>,

    // Status
    pub payment_status: ContributionPaymentStatus,

    // Link to collective call for funds (if generated from CallForFunds)
    pub call_for_funds_id: Option<Uuid>,

    // Metadata
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<Uuid>,
}

impl OwnerContribution {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        organization_id: Uuid,
        owner_id: Uuid,
        unit_id: Option<Uuid>,
        description: String,
        amount: f64,
        contribution_type: ContributionType,
        contribution_date: DateTime<Utc>,
        account_code: Option<String>,
    ) -> Result<Self, String> {
        // Validate amount is positive (revenue = money coming IN)
        if amount < 0.0 {
            return Err(
                "Contribution amount must be positive (revenue = money coming IN)".to_string(),
            );
        }

        // Validate description
        if description.trim().is_empty() {
            return Err("Description cannot be empty".to_string());
        }

        Ok(Self {
            id: Uuid::new_v4(),
            organization_id,
            owner_id,
            unit_id,
            description,
            amount,
            account_code,
            contribution_type,
            contribution_date,
            payment_date: None,
            payment_method: None,
            payment_reference: None,
            payment_status: ContributionPaymentStatus::Pending,
            call_for_funds_id: None,
            notes: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            created_by: None,
        })
    }

    /// Mark contribution as paid
    pub fn mark_as_paid(
        &mut self,
        payment_date: DateTime<Utc>,
        payment_method: ContributionPaymentMethod,
        payment_reference: Option<String>,
    ) {
        self.payment_date = Some(payment_date);
        self.payment_method = Some(payment_method);
        self.payment_reference = payment_reference;
        self.payment_status = ContributionPaymentStatus::Paid;
        self.updated_at = Utc::now();
    }

    /// Check if contribution is paid
    pub fn is_paid(&self) -> bool {
        self.payment_status == ContributionPaymentStatus::Paid
    }

    /// Check if contribution is overdue (not paid and past contribution_date)
    pub fn is_overdue(&self) -> bool {
        !self.is_paid() && Utc::now() > self.contribution_date
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::payment_method::PaymentMethod;

    #[test]
    fn test_create_contribution_success() {
        let contrib = OwnerContribution::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Some(Uuid::new_v4()),
            "Appel de fonds Q1 2025".to_string(),
            500.0,
            ContributionType::Regular,
            Utc::now(),
            Some("7000".to_string()),
        );

        assert!(contrib.is_ok());
        let contrib = contrib.unwrap();
        assert_eq!(contrib.amount, 500.0);
        assert_eq!(contrib.payment_status, ContributionPaymentStatus::Pending);
        assert!(!contrib.is_paid());
    }

    #[test]
    fn test_create_contribution_negative_amount() {
        let contrib = OwnerContribution::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            None,
            "Test".to_string(),
            -100.0, // Negative amount
            ContributionType::Regular,
            Utc::now(),
            None,
        );

        assert!(contrib.is_err());
        assert!(contrib.unwrap_err().contains("must be positive"));
    }

    #[test]
    fn test_create_contribution_empty_description() {
        let contrib = OwnerContribution::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            None,
            "   ".to_string(), // Empty description
            100.0,
            ContributionType::Regular,
            Utc::now(),
            None,
        );

        assert!(contrib.is_err());
        assert!(contrib.unwrap_err().contains("Description cannot be empty"));
    }

    #[test]
    fn test_mark_as_paid() {
        let mut contrib = OwnerContribution::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            None,
            "Test payment".to_string(),
            100.0,
            ContributionType::Regular,
            Utc::now(),
            None,
        )
        .unwrap();

        assert!(!contrib.is_paid());

        contrib.mark_as_paid(
            Utc::now(),
            PaymentMethod::BankTransfer,
            Some("REF-123".to_string()),
        );

        assert!(contrib.is_paid());
        assert!(contrib.payment_date.is_some());
        assert_eq!(contrib.payment_method, Some(PaymentMethod::BankTransfer));
        assert_eq!(contrib.payment_reference, Some("REF-123".to_string()));
    }

    #[test]
    fn test_is_overdue() {
        let past_date = Utc::now() - chrono::Duration::days(30);

        let contrib = OwnerContribution::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            None,
            "Overdue contribution".to_string(),
            100.0,
            ContributionType::Regular,
            past_date,
            None,
        )
        .unwrap();

        assert!(contrib.is_overdue());
    }
}
