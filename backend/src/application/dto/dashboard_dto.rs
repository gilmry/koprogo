// Application DTOs: Dashboard
//
// Data Transfer Objects for dashboard statistics and recent transactions

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Accountant dashboard statistics
#[derive(Debug, Serialize, Deserialize)]
pub struct AccountantDashboardStats {
    /// Total expenses for current month
    pub total_expenses_current_month: f64,

    /// Total paid expenses
    pub total_paid: f64,

    /// Percentage of expenses paid
    pub paid_percentage: f64,

    /// Total unpaid/pending expenses
    pub total_pending: f64,

    /// Percentage of expenses pending
    pub pending_percentage: f64,

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

    /// Transaction amount (positive for received, negative for paid)
    pub amount: f64,

    /// Transaction date
    pub date: DateTime<Utc>,
}
