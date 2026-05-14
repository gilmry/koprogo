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

    /// Percentage of expenses paid (serialized as JSON number for frontend display)
    #[serde(with = "rust_decimal::serde::float")]
    pub paid_percentage: Decimal,

    /// Total unpaid/pending expenses
    pub total_pending: Decimal,

    /// Percentage of expenses pending (serialized as JSON number for frontend display)
    #[serde(with = "rust_decimal::serde::float")]
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

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;
    use serde_json::Value;

    fn sample(paid_pct: Decimal, pending_pct: Decimal) -> AccountantDashboardStats {
        AccountantDashboardStats {
            total_expenses_current_month: dec!(1000.00),
            total_paid: dec!(425.50),
            paid_percentage: paid_pct,
            total_pending: dec!(574.50),
            pending_percentage: pending_pct,
            owners_with_overdue: 3,
        }
    }

    // @happy — percentages serialize as JSON numbers (not strings)
    #[test]
    fn paid_and_pending_percentage_serialize_as_json_number() {
        let json: Value = serde_json::to_value(sample(dec!(42.5), dec!(57.5))).unwrap();
        assert!(
            json["paid_percentage"].is_number(),
            "paid_percentage must be JSON number, got {:?}",
            json["paid_percentage"]
        );
        assert!(
            json["pending_percentage"].is_number(),
            "pending_percentage must be JSON number, got {:?}",
            json["pending_percentage"]
        );
        assert_eq!(json["paid_percentage"].as_f64().unwrap(), 42.5);
    }

    // @edge — 0 and 100 boundaries
    #[test]
    fn percentage_zero_and_hundred_serialize_as_number() {
        let json: Value = serde_json::to_value(sample(dec!(0), dec!(100))).unwrap();
        assert_eq!(json["paid_percentage"].as_f64().unwrap(), 0.0);
        assert_eq!(json["pending_percentage"].as_f64().unwrap(), 100.0);
    }

    // @security — anti-regression: monetary fields stay JSON strings (no f64 on money rule)
    #[test]
    fn monetary_fields_remain_json_strings() {
        let json: Value = serde_json::to_value(sample(dec!(50), dec!(50))).unwrap();
        for field in ["total_expenses_current_month", "total_paid", "total_pending"] {
            assert!(
                json[field].is_string(),
                "{} must stay JSON string (Decimal exact), got {:?}",
                field,
                json[field]
            );
        }
    }

    // @negative — round-trip stays consistent (no precision drift breaking the contract)
    #[test]
    fn percentage_roundtrip_preserves_value() {
        let original = sample(dec!(33.33), dec!(66.67));
        let json = serde_json::to_string(&original).unwrap();
        let parsed: AccountantDashboardStats = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.paid_percentage, dec!(33.33));
        assert_eq!(parsed.pending_percentage, dec!(66.67));
    }
}
