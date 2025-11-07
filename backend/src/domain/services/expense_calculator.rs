use crate::domain::entities::{Expense, Unit};

/// Service de domaine pour calculer la répartition des charges
pub struct ExpenseCalculator;

impl ExpenseCalculator {
    /// Calcule le montant dû par un lot selon sa quote-part
    pub fn calculate_unit_share(expense: &Expense, unit: &Unit) -> f64 {
        expense.amount * (unit.quota / 1000.0)
    }

    /// Calcule le total des charges pour un ensemble de dépenses
    pub fn calculate_total_expenses(expenses: &[Expense]) -> f64 {
        expenses.iter().map(|e| e.amount).sum()
    }

    /// Calcule le montant total payé
    pub fn calculate_paid_expenses(expenses: &[Expense]) -> f64 {
        expenses
            .iter()
            .filter(|e| e.is_paid())
            .map(|e| e.amount)
            .sum()
    }

    /// Calcule le montant total impayé
    pub fn calculate_unpaid_expenses(expenses: &[Expense]) -> f64 {
        expenses
            .iter()
            .filter(|e| !e.is_paid())
            .map(|e| e.amount)
            .sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::{ExpenseCategory, UnitType};
    use chrono::Utc;
    use uuid::Uuid;

    #[test]
    fn test_calculate_unit_share() {
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();

        let expense = Expense::new(
            org_id,
            building_id,
            ExpenseCategory::Maintenance,
            "Test".to_string(),
            1000.0,
            Utc::now(),
            None,
            None,
            None, // account_code
        )
        .unwrap();

        let unit = Unit::new(
            org_id,
            building_id,
            "A101".to_string(),
            UnitType::Apartment,
            Some(1),
            75.0,
            50.0, // 50/1000 = 5%
        )
        .unwrap();

        let share = ExpenseCalculator::calculate_unit_share(&expense, &unit);
        assert_eq!(share, 50.0); // 5% de 1000€ = 50€
    }

    #[test]
    fn test_calculate_total_expenses() {
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();

        let expenses = vec![
            Expense::new(
                org_id,
                building_id,
                ExpenseCategory::Maintenance,
                "Test 1".to_string(),
                100.0,
                Utc::now(),
                None,
                None,
                None, // account_code
            )
            .unwrap(),
            Expense::new(
                org_id,
                building_id,
                ExpenseCategory::Repairs,
                "Test 2".to_string(),
                200.0,
                Utc::now(),
                None,
                None,
                None, // account_code
            )
            .unwrap(),
        ];

        let total = ExpenseCalculator::calculate_total_expenses(&expenses);
        assert_eq!(total, 300.0);
    }

    #[test]
    fn test_calculate_paid_and_unpaid() {
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();

        let mut expense1 = Expense::new(
            org_id,
            building_id,
            ExpenseCategory::Maintenance,
            "Test 1".to_string(),
            100.0,
            Utc::now(),
            None,
            None,
            None, // account_code
        )
        .unwrap();
        let _ = expense1.mark_as_paid();

        let expense2 = Expense::new(
            org_id,
            building_id,
            ExpenseCategory::Repairs,
            "Test 2".to_string(),
            200.0,
            Utc::now(),
            None,
            None,
            None, // account_code
        )
        .unwrap();

        let expenses = vec![expense1, expense2];

        let paid = ExpenseCalculator::calculate_paid_expenses(&expenses);
        let unpaid = ExpenseCalculator::calculate_unpaid_expenses(&expenses);

        assert_eq!(paid, 100.0);
        assert_eq!(unpaid, 200.0);
    }
}
