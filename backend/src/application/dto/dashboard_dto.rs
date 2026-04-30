// Application DTOs: Dashboard
//
// Data Transfer Objects for dashboard statistics and recent transactions

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Accountant dashboard statistics.
///
/// MONETARY: amounts use rust_decimal::Decimal (cf. ADR-0007).
/// Percentages remain Decimal to preserve exactness on display.
#[derive(Debug, Serialize, Deserialize)]
pub struct AccountantDashboardStats {
    /// Total expenses for current month
    pub total_expenses_current_month: Decimal,

    /// Total paid expenses
    pub total_paid: Decimal,

    /// Percentage of expenses paid
    pub paid_percentage: Decimal,

    /// Total unpaid/pending expenses
    pub total_pending: Decimal,

    /// Percentage of expenses pending
    pub pending_percentage: Decimal,

    /// Number of owners with overdue payments
    pub owners_with_overdue: i64,
}

/// Transaction type for dashboard display
///
/// **Important**: Currently only displays expenses (payments made).
/// For a complete ACP (Association de Copropriétaires) accounting view, we would need:
/// - PaymentReceived: Appels de fonds paid by owners (classe 7 PCMN - Produits)
/// - PaymentMade: Expenses paid to suppliers (classe 6 PCMN - Charges)
///
/// **TODO**: Implement owner contributions tracking (appels de fonds) to show incoming payments
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TransactionType {
    /// Payment received from owner (appels de fonds) - NOT YET IMPLEMENTED
    PaymentReceived,
    /// Payment made to supplier (expenses)
    PaymentMade,
}

/// Recent transaction for dashboard
#[derive(Debug, Serialize, Deserialize)]
pub struct RecentTransaction {
    /// Transaction ID
    pub id: Uuid,

    /// Transaction type
    pub transaction_type: TransactionType,

    /// Transaction description
    pub description: String,

    /// Related entity (owner name, supplier, etc.)
    pub related_entity: Option<String>,

    /// Transaction amount (positive for received, negative for paid). Decimal exact.
    pub amount: Decimal,

    /// Transaction date
    pub date: DateTime<Utc>,
}
