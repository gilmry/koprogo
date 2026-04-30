// Domain Entity: Journal Entry
//
// CREDITS & ATTRIBUTION:
// This implementation is inspired by the Noalyss project (https://gitlab.com/noalyss/noalyss)
// Noalyss is a free accounting software for Belgian and French accounting
// License: GPL-2.0-or-later (GNU General Public License version 2 or later)
// Copyright: (C) 1989, 1991 Free Software Foundation, Inc.
// Copyright: Dany De Bontridder <dany@alchimerys.eu>
//
// Inspired by Noalyss `jrn` table structure
//
// MONETARY: debit/credit use rust_decimal::Decimal (cf. ADR-0007).
// Tolerance for double-entry balance: dec!(0.011).

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Journal Entry represents a complete accounting transaction
/// with balanced debit and credit lines (double-entry bookkeeping).
///
/// Each entry contains multiple lines (JournalEntryLine) where:
/// - Sum of debits = Sum of credits (enforced by database trigger)
/// - Each line affects one account
///
/// Example: Recording a 1,210€ utility expense (1,000€ + 210€ VAT 21%):
/// - Debit: 6100 (Utilities) 1,000€
/// - Debit: 4110 (VAT Recoverable) 210€
/// - Credit: 4400 (Suppliers) 1,210€
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalEntry {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// Optional link to building for building-specific accounting
    pub building_id: Option<Uuid>,
    /// Date when the transaction occurred (not when recorded)
    pub entry_date: DateTime<Utc>,
    /// Human-readable description (e.g., "Facture eau janvier 2025")
    pub description: Option<String>,
    /// Reference to source document (invoice number, receipt, etc.)
    pub document_ref: Option<String>,
    /// Journal type: ACH (Purchases), VEN (Sales), FIN (Financial), ODS (Miscellaneous)
    /// Inspired by Noalyss journal categories
    pub journal_type: Option<String>,
    /// Optional link to the expense that generated this entry
    pub expense_id: Option<Uuid>,
    /// Optional link to the owner contribution that generated this entry
    pub contribution_id: Option<Uuid>,
    /// Lines composing this entry (debits and credits)
    pub lines: Vec<JournalEntryLine>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<Uuid>,
}

/// Individual debit or credit line within a journal entry
///
/// Implements double-entry bookkeeping rule: each line is EITHER debit OR credit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalEntryLine {
    pub id: Uuid,
    pub journal_entry_id: Uuid,
    pub organization_id: Uuid,
    /// PCMN account code (e.g., "6100", "4400", "5500")
    pub account_code: String,
    /// Debit amount (increases assets/expenses, decreases liabilities/revenue)
    pub debit: Decimal,
    /// Credit amount (decreases assets/expenses, increases liabilities/revenue)
    pub credit: Decimal,
    /// Optional description specific to this line
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Tolerance for double-entry balance check (1 centime + epsilon).
const BALANCE_TOLERANCE: Decimal = dec!(0.011);

impl JournalEntry {
    /// Create a new journal entry with validation
    ///
    /// # Arguments
    /// - `organization_id`: Organization owning this entry
    /// - `entry_date`: Transaction date
    /// - `description`: Human-readable description
    /// - `lines`: Debit and credit lines (must balance)
    ///
    /// # Returns
    /// - `Ok(JournalEntry)` if lines balance (within 0.01€ tolerance)
    /// - `Err(String)` if validation fails
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        organization_id: Uuid,
        building_id: Option<Uuid>,
        entry_date: DateTime<Utc>,
        description: Option<String>,
        document_ref: Option<String>,
        journal_type: Option<String>,
        expense_id: Option<Uuid>,
        contribution_id: Option<Uuid>,
        lines: Vec<JournalEntryLine>,
        created_by: Option<Uuid>,
    ) -> Result<Self, String> {
        // Validate lines balance
        Self::validate_lines_balance(&lines)?;

        // Validate each line
        for line in &lines {
            Self::validate_line(line)?;
        }

        // Validate journal_type if provided (Noalyss-inspired)
        if let Some(ref jtype) = journal_type {
            if !["ACH", "VEN", "FIN", "ODS"].contains(&jtype.as_str()) {
                return Err(format!(
                    "Invalid journal type: {}. Must be one of: ACH (Purchases), VEN (Sales), FIN (Financial), ODS (Miscellaneous)",
                    jtype
                ));
            }
        }

        let now = Utc::now();
        Ok(Self {
            id: Uuid::new_v4(),
            organization_id,
            building_id,
            entry_date,
            description,
            document_ref,
            journal_type,
            expense_id,
            contribution_id,
            lines,
            created_at: now,
            updated_at: now,
            created_by,
        })
    }

    /// Validate that debits equal credits (with small rounding tolerance)
    fn validate_lines_balance(lines: &[JournalEntryLine]) -> Result<(), String> {
        if lines.is_empty() {
            return Err("Journal entry must have at least one line".to_string());
        }

        let total_debits: Decimal = lines.iter().map(|l| l.debit).sum();
        let total_credits: Decimal = lines.iter().map(|l| l.credit).sum();

        let difference = (total_debits - total_credits).abs();
        if difference > BALANCE_TOLERANCE {
            return Err(format!(
                "Journal entry is unbalanced: debits={}€, credits={}€, difference={}€ (tolerance: {}€)",
                total_debits, total_credits, difference, BALANCE_TOLERANCE
            ));
        }

        Ok(())
    }

    /// Validate an individual line
    fn validate_line(line: &JournalEntryLine) -> Result<(), String> {
        // Must be EITHER debit OR credit (not both, not neither)
        if line.debit > Decimal::ZERO && line.credit > Decimal::ZERO {
            return Err("Line cannot have both debit and credit".to_string());
        }

        if line.debit == Decimal::ZERO && line.credit == Decimal::ZERO {
            return Err("Line must have either debit or credit".to_string());
        }

        // Amounts must be non-negative
        if line.debit < Decimal::ZERO || line.credit < Decimal::ZERO {
            return Err("Debit and credit amounts must be non-negative".to_string());
        }

        // Account code required
        if line.account_code.trim().is_empty() {
            return Err("Account code is required".to_string());
        }

        Ok(())
    }

    /// Calculate total debits for this entry
    pub fn total_debits(&self) -> Decimal {
        self.lines.iter().map(|l| l.debit).sum()
    }

    /// Calculate total credits for this entry
    pub fn total_credits(&self) -> Decimal {
        self.lines.iter().map(|l| l.credit).sum()
    }

    /// Check if this entry is balanced (debits = credits)
    pub fn is_balanced(&self) -> bool {
        (self.total_debits() - self.total_credits()).abs() <= BALANCE_TOLERANCE
    }
}

impl JournalEntryLine {
    /// Create a new debit line
    pub fn new_debit(
        journal_entry_id: Uuid,
        organization_id: Uuid,
        account_code: String,
        amount: Decimal,
        description: Option<String>,
    ) -> Result<Self, String> {
        if amount <= Decimal::ZERO {
            return Err("Debit amount must be positive".to_string());
        }

        Ok(Self {
            id: Uuid::new_v4(),
            journal_entry_id,
            organization_id,
            account_code,
            debit: amount,
            credit: Decimal::ZERO,
            description,
            created_at: Utc::now(),
        })
    }

    /// Create a new credit line
    pub fn new_credit(
        journal_entry_id: Uuid,
        organization_id: Uuid,
        account_code: String,
        amount: Decimal,
        description: Option<String>,
    ) -> Result<Self, String> {
        if amount <= Decimal::ZERO {
            return Err("Credit amount must be positive".to_string());
        }

        Ok(Self {
            id: Uuid::new_v4(),
            journal_entry_id,
            organization_id,
            account_code,
            debit: Decimal::ZERO,
            credit: amount,
            description,
            created_at: Utc::now(),
        })
    }

    /// Get the amount (whether debit or credit)
    pub fn amount(&self) -> Decimal {
        if self.debit > Decimal::ZERO {
            self.debit
        } else {
            self.credit
        }
    }

    /// Check if this is a debit line
    pub fn is_debit(&self) -> bool {
        self.debit > Decimal::ZERO
    }

    /// Check if this is a credit line
    pub fn is_credit(&self) -> bool {
        self.credit > Decimal::ZERO
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_journal_entry_balanced() {
        let org_id = Uuid::new_v4();
        let entry_id = Uuid::new_v4();

        // Utility expense: 1,000€ + 210€ VAT = 1,210€
        let lines = vec![
            JournalEntryLine::new_debit(
                entry_id,
                org_id,
                "6100".to_string(),
                dec!(1000),
                Some("Utilities".to_string()),
            )
            .unwrap(),
            JournalEntryLine::new_debit(
                entry_id,
                org_id,
                "4110".to_string(),
                dec!(210),
                Some("VAT 21%".to_string()),
            )
            .unwrap(),
            JournalEntryLine::new_credit(
                entry_id,
                org_id,
                "4400".to_string(),
                dec!(1210),
                Some("Supplier".to_string()),
            )
            .unwrap(),
        ];

        let entry = JournalEntry::new(
            org_id,
            None, // building_id
            Utc::now(),
            Some("Facture eau".to_string()),
            Some("INV-2025-001".to_string()),
            Some("ACH".to_string()), // journal_type
            None,                    // expense_id
            None,                    // contribution_id
            lines,
            None, // created_by
        );

        assert!(entry.is_ok());
        let entry = entry.unwrap();
        assert!(entry.is_balanced());
        assert_eq!(entry.total_debits(), dec!(1210));
        assert_eq!(entry.total_credits(), dec!(1210));
    }

    #[test]
    fn test_journal_entry_unbalanced() {
        let org_id = Uuid::new_v4();
        let entry_id = Uuid::new_v4();

        // Unbalanced: 1,000€ debit vs 900€ credit
        let lines = vec![
            JournalEntryLine::new_debit(entry_id, org_id, "6100".to_string(), dec!(1000), None)
                .unwrap(),
            JournalEntryLine::new_credit(entry_id, org_id, "4400".to_string(), dec!(900), None)
                .unwrap(),
        ];

        let entry = JournalEntry::new(
            org_id,
            None, // building_id
            Utc::now(),
            Some("Test".to_string()),
            None, // document_ref
            None, // journal_type
            None, // expense_id
            None, // contribution_id
            lines,
            None, // created_by
        );

        assert!(entry.is_err());
        assert!(entry.unwrap_err().contains("unbalanced"));
    }

    #[test]
    fn test_journal_entry_line_cannot_have_both_debit_and_credit() {
        let org_id = Uuid::new_v4();
        let entry_id = Uuid::new_v4();

        // Invalid line with both debit and credit
        let invalid_line = JournalEntryLine {
            id: Uuid::new_v4(),
            journal_entry_id: entry_id,
            organization_id: org_id,
            account_code: "6100".to_string(),
            debit: dec!(100),
            credit: dec!(100), // Invalid!
            description: None,
            created_at: Utc::now(),
        };

        let entry = JournalEntry::new(
            org_id,
            None,
            Utc::now(),
            Some("Test".to_string()),
            None,
            None,
            None,
            None,
            vec![invalid_line],
            None,
        );

        assert!(entry.is_err());
        assert!(entry.unwrap_err().contains("both debit and credit"));
    }

    #[test]
    fn test_journal_entry_line_must_have_amount() {
        let org_id = Uuid::new_v4();
        let entry_id = Uuid::new_v4();

        // Invalid line with neither debit nor credit
        let invalid_line = JournalEntryLine {
            id: Uuid::new_v4(),
            journal_entry_id: entry_id,
            organization_id: org_id,
            account_code: "6100".to_string(),
            debit: Decimal::ZERO,
            credit: Decimal::ZERO, // Invalid!
            description: None,
            created_at: Utc::now(),
        };

        let entry = JournalEntry::new(
            org_id,
            None,
            Utc::now(),
            Some("Test".to_string()),
            None,
            None,
            None,
            None,
            vec![invalid_line],
            None,
        );

        assert!(entry.is_err());
        assert!(entry.unwrap_err().contains("either debit or credit"));
    }

    #[test]
    fn test_rounding_tolerance() {
        let org_id = Uuid::new_v4();
        let entry_id = Uuid::new_v4();

        // Small rounding difference (0.01€) should be accepted
        let lines = vec![
            JournalEntryLine::new_debit(entry_id, org_id, "6100".to_string(), dec!(100.33), None)
                .unwrap(),
            JournalEntryLine::new_credit(
                entry_id,
                org_id,
                "4400".to_string(),
                dec!(100.34), // 0.01€ difference
                None,
            )
            .unwrap(),
        ];

        let entry = JournalEntry::new(
            org_id,
            None,
            Utc::now(),
            Some("Test rounding".to_string()),
            None,
            None,
            None,
            None,
            lines,
            None,
        );

        if entry.is_err() {
            eprintln!("Error: {:?}", entry.as_ref().err());
        }
        assert!(entry.is_ok());
        assert!(entry.unwrap().is_balanced());
    }

    /// @edge — Decimal exactness preserved on cumulative sums (ADR-0007).
    /// IEEE 754 fails this: 0.1 + 0.2 != 0.3 in f64.
    #[test]
    fn edge_decimal_exactness_preserved_on_cumul() {
        let org_id = Uuid::new_v4();
        let entry_id = Uuid::new_v4();

        let lines = vec![
            JournalEntryLine::new_debit(entry_id, org_id, "6100".to_string(), dec!(0.1), None)
                .unwrap(),
            JournalEntryLine::new_debit(entry_id, org_id, "6101".to_string(), dec!(0.2), None)
                .unwrap(),
            JournalEntryLine::new_credit(entry_id, org_id, "4400".to_string(), dec!(0.3), None)
                .unwrap(),
        ];

        let entry = JournalEntry::new(
            org_id,
            None,
            Utc::now(),
            None,
            None,
            None,
            None,
            None,
            lines,
            None,
        )
        .expect("0.1 + 0.2 = 0.3 must balance exactly with Decimal");

        assert_eq!(entry.total_debits(), dec!(0.3));
        assert_eq!(entry.total_credits(), dec!(0.3));
        assert!(entry.is_balanced());
    }

    /// @negative — Negative debit must be rejected.
    #[test]
    fn negative_debit_amount_rejected() {
        let result = JournalEntryLine::new_debit(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "6100".to_string(),
            dec!(-1),
            None,
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("must be positive"));
    }
}
