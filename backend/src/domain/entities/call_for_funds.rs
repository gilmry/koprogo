// Domain Entity: Call for Funds (Appel de Fonds)
//
// Represents a collective payment request sent by the Syndic to all owners
// This is the "master" entity that generates individual OwnerContribution records

use chrono::{DateTime, Utc};
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
    pub total_amount: f64, // Total amount to be collected from ALL owners

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

impl CallForFunds {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        organization_id: Uuid,
        building_id: Uuid,
        title: String,
        description: String,
        total_amount: f64,
        contribution_type: ContributionType,
        call_date: DateTime<Utc>,
        due_date: DateTime<Utc>,
        account_code: Option<String>,
    ) -> Result<Self, String> {
        // Validate total amount is positive
        if total_amount <= 0.0 {
            return Err("Total amount must be positive".to_string());
        }

        // Validate title
        if title.trim().is_empty() {
            return Err("Title cannot be empty".to_string());
        }

        // Validate description
        if description.trim().is_empty() {
            return Err("Description cannot be empty".to_string());
        }

        // Validate dates
        if due_date <= call_date {
            return Err("Due date must be after call date".to_string());
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
            5000.0,
            ContributionType::Regular,
            call_date,
            due_date,
            Some("7000".to_string()),
        );

        assert!(call.is_ok());
        let call = call.unwrap();
        assert_eq!(call.total_amount, 5000.0);
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
            -100.0,
            ContributionType::Regular,
            call_date,
            due_date,
            None,
        );

        assert!(call.is_err());
        assert!(call.unwrap_err().contains("must be positive"));
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
            100.0,
            ContributionType::Regular,
            call_date,
            due_date,
            None,
        );

        assert!(call.is_err());
        assert!(call.unwrap_err().contains("Due date must be after"));
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
            100.0,
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
            100.0,
            ContributionType::Regular,
            call_date,
            due_date,
            None,
        )
        .unwrap();

        assert!(call.is_overdue());
    }
}
