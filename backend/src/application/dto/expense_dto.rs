use crate::domain::entities::{ExpenseCategory, PaymentStatus};
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Deserialize, Validate, Clone)]
pub struct CreateExpenseDto {
    pub organization_id: String,
    pub building_id: String,
    pub category: ExpenseCategory,

    #[validate(length(min = 1))]
    pub description: String,

    #[validate(range(min = 0.01))]
    pub amount: f64,

    pub expense_date: String,
    pub supplier: Option<String>,
    pub invoice_number: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ExpenseResponseDto {
    pub id: String,
    pub building_id: String,
    pub category: ExpenseCategory,
    pub description: String,
    pub amount: f64,
    pub expense_date: String,
    pub payment_status: PaymentStatus,
    pub supplier: Option<String>,
    pub invoice_number: Option<String>,
}
