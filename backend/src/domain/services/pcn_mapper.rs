use crate::domain::entities::{Expense, ExpenseCategory};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Belgian Chart of Accounts (Plan Comptable Normalisé - PCN) Account
///
/// The PCN is the standard accounting framework used in Belgium.
/// - Class 6: Expenses (Charges) - 60x, 61x, 62x
/// - Class 7: Income (Produits) - 70x, 71x, 72x
///
/// For co-ownership buildings, we primarily use Class 6 accounts.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PcnAccount {
    /// PCN account code (e.g., "611" for maintenance, "615" for utilities)
    pub code: String,
    /// Account label in French (primary language in Belgium)
    pub label_fr: String,
    /// Account label in Dutch (Flemish - second official language in Belgium)
    pub label_nl: String,
}

impl PcnAccount {
    pub fn new(
        code: impl Into<String>,
        label_fr: impl Into<String>,
        label_nl: impl Into<String>,
    ) -> Self {
        Self {
            code: code.into(),
            label_fr: label_fr.into(),
            label_nl: label_nl.into(),
        }
    }
}

/// Belgian PCN Mapping Service
pub struct PcnMapper;

/// PCN Report Line - aggregated expenses for one PCN account
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PcnReportLine {
    /// PCN account
    pub account: PcnAccount,
    /// Total amount for this account
    pub total_amount: f64,
    /// Number of expense entries
    pub entry_count: usize,
}

impl PcnMapper {
    /// Map ExpenseCategory to Belgian PCN account
    /// Based on Belgian Chart of Accounts (PCN) Class 6: Expenses
    pub fn map_expense_to_pcn(category: &ExpenseCategory) -> PcnAccount {
        match category {
            ExpenseCategory::Works => PcnAccount::new(
                "610",
                "Travaux et grosses réparations",
                "Werken en grote herstellingen",
            ),
            ExpenseCategory::Maintenance => PcnAccount::new(
                "611",
                "Entretien et petites réparations",
                "Onderhoud en kleine herstellingen",
            ),
            ExpenseCategory::Repairs => {
                PcnAccount::new("612", "Réparations ordinaires", "Gewone herstellingen")
            }
            ExpenseCategory::Insurance => PcnAccount::new("613", "Assurances", "Verzekeringen"),
            ExpenseCategory::Cleaning => PcnAccount::new(
                "614",
                "Nettoyage et entretien courant",
                "Schoonmaak en lopend onderhoud",
            ),
            ExpenseCategory::Utilities => PcnAccount::new(
                "615",
                "Eau, énergie et chauffage",
                "Water, energie en verwarming",
            ),
            ExpenseCategory::Administration => PcnAccount::new(
                "620",
                "Frais de gestion et d'administration",
                "Beheer- en administratiekosten",
            ),
            ExpenseCategory::Other => {
                PcnAccount::new("619", "Autres charges diverses", "Overige diverse kosten")
            }
        }
    }

    /// Generate PCN report from a list of expenses
    /// Aggregates expenses by PCN account code
    /// Returns report lines sorted by PCN account code
    pub fn generate_report(expenses: &[Expense]) -> Vec<PcnReportLine> {
        let mut aggregated: HashMap<String, (PcnAccount, f64, usize)> = HashMap::new();

        // Aggregate expenses by PCN account code
        for expense in expenses {
            let account = Self::map_expense_to_pcn(&expense.category);
            let entry = aggregated
                .entry(account.code.clone())
                .or_insert((account, 0.0, 0));
            entry.1 += expense.amount;
            entry.2 += 1;
        }

        // Convert to Vec<PcnReportLine> and sort by account code
        let mut report: Vec<PcnReportLine> = aggregated
            .into_iter()
            .map(|(_, (account, total_amount, entry_count))| PcnReportLine {
                account,
                total_amount,
                entry_count,
            })
            .collect();

        report.sort_by(|a, b| a.account.code.cmp(&b.account.code));
        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_maintenance_to_pcn_611() {
        let account = PcnMapper::map_expense_to_pcn(&ExpenseCategory::Maintenance);

        assert_eq!(account.code, "611");
        assert!(account.label_fr.contains("Entretien"));
        assert!(!account.label_nl.is_empty());
    }

    #[test]
    fn test_map_utilities_to_pcn_615() {
        let account = PcnMapper::map_expense_to_pcn(&ExpenseCategory::Utilities);

        assert_eq!(account.code, "615");
        assert!(account.label_fr.contains("Eau") || account.label_fr.contains("Énergie"));
    }

    #[test]
    fn test_map_insurance_to_pcn_613() {
        let account = PcnMapper::map_expense_to_pcn(&ExpenseCategory::Insurance);

        assert_eq!(account.code, "613");
        assert!(account.label_fr.contains("Assurance"));
    }

    #[test]
    fn test_map_repairs_to_pcn_612() {
        let account = PcnMapper::map_expense_to_pcn(&ExpenseCategory::Repairs);

        assert_eq!(account.code, "612");
        assert!(account.label_fr.contains("Réparation"));
    }

    #[test]
    fn test_map_cleaning_to_pcn_614() {
        let account = PcnMapper::map_expense_to_pcn(&ExpenseCategory::Cleaning);

        assert_eq!(account.code, "614");
        assert!(account.label_fr.contains("Nettoyage"));
    }

    #[test]
    fn test_map_administration_to_pcn_620() {
        let account = PcnMapper::map_expense_to_pcn(&ExpenseCategory::Administration);

        assert_eq!(account.code, "620");
        assert!(
            account.label_fr.to_lowercase().contains("administration")
                || account.label_fr.to_lowercase().contains("gestion")
        );
    }

    #[test]
    fn test_map_works_to_pcn_610() {
        let account = PcnMapper::map_expense_to_pcn(&ExpenseCategory::Works);

        assert_eq!(account.code, "610");
        assert!(account.label_fr.contains("Travaux"));
    }

    #[test]
    fn test_map_other_to_pcn_619() {
        let account = PcnMapper::map_expense_to_pcn(&ExpenseCategory::Other);

        assert_eq!(account.code, "619");
        assert!(account.label_fr.contains("Divers") || account.label_fr.contains("Autre"));
    }

    // Helper function to create test expenses
    fn create_test_expense(category: ExpenseCategory, amount: f64) -> Expense {
        use chrono::Utc;
        use uuid::Uuid;

        let description = format!("Test expense for {:?}", category);
        Expense::new(
            Uuid::new_v4(), // building_id
            category,
            description,
            amount,
            Utc::now(),
            Some("Test Supplier".to_string()),
            Some("INV-001".to_string()),
        )
        .unwrap()
    }

    #[test]
    fn test_generate_report_empty_expenses() {
        let expenses: Vec<Expense> = vec![];
        let report = PcnMapper::generate_report(&expenses);

        assert!(report.is_empty());
    }

    #[test]
    fn test_generate_report_single_category() {
        let expenses = vec![
            create_test_expense(ExpenseCategory::Maintenance, 100.0),
            create_test_expense(ExpenseCategory::Maintenance, 150.0),
        ];

        let report = PcnMapper::generate_report(&expenses);

        assert_eq!(report.len(), 1);
        assert_eq!(report[0].account.code, "611");
        assert_eq!(report[0].total_amount, 250.0);
        assert_eq!(report[0].entry_count, 2);
    }

    #[test]
    fn test_generate_report_multiple_categories() {
        let expenses = vec![
            create_test_expense(ExpenseCategory::Maintenance, 100.0),
            create_test_expense(ExpenseCategory::Utilities, 50.0),
            create_test_expense(ExpenseCategory::Maintenance, 150.0),
            create_test_expense(ExpenseCategory::Insurance, 200.0),
        ];

        let report = PcnMapper::generate_report(&expenses);

        assert_eq!(report.len(), 3);

        // Find specific accounts in report
        let maintenance_line = report.iter().find(|l| l.account.code == "611").unwrap();
        assert_eq!(maintenance_line.total_amount, 250.0);
        assert_eq!(maintenance_line.entry_count, 2);

        let utilities_line = report.iter().find(|l| l.account.code == "615").unwrap();
        assert_eq!(utilities_line.total_amount, 50.0);
        assert_eq!(utilities_line.entry_count, 1);

        let insurance_line = report.iter().find(|l| l.account.code == "613").unwrap();
        assert_eq!(insurance_line.total_amount, 200.0);
        assert_eq!(insurance_line.entry_count, 1);
    }

    #[test]
    fn test_generate_report_sorted_by_account_code() {
        let expenses = vec![
            create_test_expense(ExpenseCategory::Administration, 100.0), // 620
            create_test_expense(ExpenseCategory::Works, 200.0),          // 610
            create_test_expense(ExpenseCategory::Utilities, 50.0),       // 615
        ];

        let report = PcnMapper::generate_report(&expenses);

        assert_eq!(report.len(), 3);
        // Should be sorted by account code: 610, 615, 620
        assert_eq!(report[0].account.code, "610");
        assert_eq!(report[1].account.code, "615");
        assert_eq!(report[2].account.code, "620");
    }
}
