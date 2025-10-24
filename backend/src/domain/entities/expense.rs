use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Catégorie de charges
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ExpenseCategory {
    Maintenance,    // Entretien
    Repairs,        // Réparations
    Insurance,      // Assurance
    Utilities,      // Charges courantes (eau, électricité)
    Cleaning,       // Nettoyage
    Administration, // Administration
    Works,          // Travaux
    Other,
}

/// Statut de paiement
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PaymentStatus {
    Pending,
    Paid,
    Overdue,
    Cancelled,
}

/// Représente une charge de copropriété
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Expense {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub building_id: Uuid,
    pub category: ExpenseCategory,
    pub description: String,
    pub amount: f64, // en euros
    pub expense_date: DateTime<Utc>,
    pub payment_status: PaymentStatus,
    pub supplier: Option<String>,
    pub invoice_number: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Expense {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        organization_id: Uuid,
        building_id: Uuid,
        category: ExpenseCategory,
        description: String,
        amount: f64,
        expense_date: DateTime<Utc>,
        supplier: Option<String>,
        invoice_number: Option<String>,
    ) -> Result<Self, String> {
        if description.is_empty() {
            return Err("Description cannot be empty".to_string());
        }
        if amount <= 0.0 {
            return Err("Amount must be greater than 0".to_string());
        }

        let now = Utc::now();
        Ok(Self {
            id: Uuid::new_v4(),
            organization_id,
            building_id,
            category,
            description,
            amount,
            expense_date,
            payment_status: PaymentStatus::Pending,
            supplier,
            invoice_number,
            created_at: now,
            updated_at: now,
        })
    }

    pub fn mark_as_paid(&mut self) {
        self.payment_status = PaymentStatus::Paid;
        self.updated_at = Utc::now();
    }

    pub fn mark_as_overdue(&mut self) {
        self.payment_status = PaymentStatus::Overdue;
        self.updated_at = Utc::now();
    }

    pub fn cancel(&mut self) {
        self.payment_status = PaymentStatus::Cancelled;
        self.updated_at = Utc::now();
    }

    pub fn is_paid(&self) -> bool {
        self.payment_status == PaymentStatus::Paid
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_expense_success() {
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let expense = Expense::new(
            org_id,
            building_id,
            ExpenseCategory::Maintenance,
            "Entretien ascenseur".to_string(),
            500.0,
            Utc::now(),
            Some("ACME Elevators".to_string()),
            Some("INV-2024-001".to_string()),
        );

        assert!(expense.is_ok());
        let expense = expense.unwrap();
        assert_eq!(expense.organization_id, org_id);
        assert_eq!(expense.amount, 500.0);
        assert_eq!(expense.payment_status, PaymentStatus::Pending);
    }

    #[test]
    fn test_create_expense_negative_amount_fails() {
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let expense = Expense::new(
            org_id,
            building_id,
            ExpenseCategory::Maintenance,
            "Test".to_string(),
            -100.0,
            Utc::now(),
            None,
            None,
        );

        assert!(expense.is_err());
    }

    #[test]
    fn test_mark_expense_as_paid() {
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let mut expense = Expense::new(
            org_id,
            building_id,
            ExpenseCategory::Maintenance,
            "Test".to_string(),
            100.0,
            Utc::now(),
            None,
            None,
        )
        .unwrap();

        assert!(!expense.is_paid());
        expense.mark_as_paid();
        assert!(expense.is_paid());
    }
}
