use crate::application::dto::ChargeDistributionResponseDto;
use crate::application::ports::{
    ChargeDistributionRepository, ExpenseRepository, UnitOwnerRepository,
};
use crate::domain::entities::{ApprovalStatus, ChargeDistribution};
use std::sync::Arc;
use uuid::Uuid;

pub struct ChargeDistributionUseCases {
    distribution_repository: Arc<dyn ChargeDistributionRepository>,
    expense_repository: Arc<dyn ExpenseRepository>,
    unit_owner_repository: Arc<dyn UnitOwnerRepository>,
}

impl ChargeDistributionUseCases {
    pub fn new(
        distribution_repository: Arc<dyn ChargeDistributionRepository>,
        expense_repository: Arc<dyn ExpenseRepository>,
        unit_owner_repository: Arc<dyn UnitOwnerRepository>,
    ) -> Self {
        Self {
            distribution_repository,
            expense_repository,
            unit_owner_repository,
        }
    }

    /// Calculer et sauvegarder la répartition des charges pour une facture approuvée
    pub async fn calculate_and_save_distribution(
        &self,
        expense_id: Uuid,
    ) -> Result<Vec<ChargeDistributionResponseDto>, String> {
        // 1. Récupérer la facture
        let expense = self
            .expense_repository
            .find_by_id(expense_id)
            .await?
            .ok_or_else(|| "Expense/Invoice not found".to_string())?;

        // 2. Vérifier que la facture est approuvée
        if expense.approval_status != ApprovalStatus::Approved {
            return Err(format!(
                "Cannot calculate distribution for non-approved invoice (status: {:?})",
                expense.approval_status
            ));
        }

        // 3. Récupérer le montant TTC à répartir
        let total_amount = expense.amount_incl_vat.unwrap_or(expense.amount);

        // 4. Récupérer toutes les relations unit-owner actives pour ce bâtiment
        let unit_ownerships = self
            .unit_owner_repository
            .find_active_by_building(expense.building_id)
            .await?;

        if unit_ownerships.is_empty() {
            return Err("No active unit-owner relationships found for this building".to_string());
        }

        // 5. Calculer les distributions
        let distributions =
            ChargeDistribution::calculate_distributions(expense_id, total_amount, unit_ownerships)?;

        // 6. Sauvegarder en masse
        let saved_distributions = self
            .distribution_repository
            .create_bulk(&distributions)
            .await?;

        // 7. Convertir en DTOs
        Ok(saved_distributions
            .iter()
            .map(|d| self.to_response_dto(d))
            .collect())
    }

    /// Récupérer la répartition d'une facture
    pub async fn get_distribution_by_expense(
        &self,
        expense_id: Uuid,
    ) -> Result<Vec<ChargeDistributionResponseDto>, String> {
        let distributions = self
            .distribution_repository
            .find_by_expense(expense_id)
            .await?;
        Ok(distributions
            .iter()
            .map(|d| self.to_response_dto(d))
            .collect())
    }

    /// Récupérer toutes les distributions pour un propriétaire
    pub async fn get_distributions_by_owner(
        &self,
        owner_id: Uuid,
    ) -> Result<Vec<ChargeDistributionResponseDto>, String> {
        let distributions = self.distribution_repository.find_by_owner(owner_id).await?;
        Ok(distributions
            .iter()
            .map(|d| self.to_response_dto(d))
            .collect())
    }

    /// Récupérer le montant total dû par un propriétaire
    pub async fn get_total_due_by_owner(&self, owner_id: Uuid) -> Result<f64, String> {
        self.distribution_repository
            .get_total_due_by_owner(owner_id)
            .await
    }

    /// Supprimer les distributions d'une facture (si annulée)
    pub async fn delete_distribution_by_expense(&self, expense_id: Uuid) -> Result<(), String> {
        self.distribution_repository
            .delete_by_expense(expense_id)
            .await
    }

    fn to_response_dto(&self, distribution: &ChargeDistribution) -> ChargeDistributionResponseDto {
        ChargeDistributionResponseDto {
            id: distribution.id.to_string(),
            expense_id: distribution.expense_id.to_string(),
            unit_id: distribution.unit_id.to_string(),
            owner_id: distribution.owner_id.to_string(),
            quota_percentage: distribution.quota_percentage,
            amount_due: distribution.amount_due,
            created_at: distribution.created_at.to_rfc3339(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::dto::{ExpenseFilters, PageRequest};
    use crate::application::ports::{
        ChargeDistributionRepository, ExpenseRepository, UnitOwnerRepository,
    };
    use crate::domain::entities::{
        ApprovalStatus, ChargeDistribution, Expense, ExpenseCategory, PaymentStatus, UnitOwner,
    };
    use async_trait::async_trait;
    use chrono::Utc;
    use std::collections::HashMap;
    use std::sync::Mutex;

    // ========== Mock ChargeDistributionRepository ==========

    struct MockChargeDistributionRepository {
        distributions: Mutex<HashMap<Uuid, ChargeDistribution>>,
    }

    impl MockChargeDistributionRepository {
        fn new() -> Self {
            Self {
                distributions: Mutex::new(HashMap::new()),
            }
        }
    }

    #[async_trait]
    impl ChargeDistributionRepository for MockChargeDistributionRepository {
        async fn create(
            &self,
            distribution: &ChargeDistribution,
        ) -> Result<ChargeDistribution, String> {
            let mut distributions = self.distributions.lock().unwrap();
            distributions.insert(distribution.id, distribution.clone());
            Ok(distribution.clone())
        }

        async fn create_bulk(
            &self,
            distributions: &[ChargeDistribution],
        ) -> Result<Vec<ChargeDistribution>, String> {
            let mut store = self.distributions.lock().unwrap();
            let mut result = Vec::new();
            for d in distributions {
                store.insert(d.id, d.clone());
                result.push(d.clone());
            }
            Ok(result)
        }

        async fn find_by_id(&self, id: Uuid) -> Result<Option<ChargeDistribution>, String> {
            let distributions = self.distributions.lock().unwrap();
            Ok(distributions.get(&id).cloned())
        }

        async fn find_by_expense(
            &self,
            expense_id: Uuid,
        ) -> Result<Vec<ChargeDistribution>, String> {
            let distributions = self.distributions.lock().unwrap();
            Ok(distributions
                .values()
                .filter(|d| d.expense_id == expense_id)
                .cloned()
                .collect())
        }

        async fn find_by_unit(&self, unit_id: Uuid) -> Result<Vec<ChargeDistribution>, String> {
            let distributions = self.distributions.lock().unwrap();
            Ok(distributions
                .values()
                .filter(|d| d.unit_id == unit_id)
                .cloned()
                .collect())
        }

        async fn find_by_owner(&self, owner_id: Uuid) -> Result<Vec<ChargeDistribution>, String> {
            let distributions = self.distributions.lock().unwrap();
            Ok(distributions
                .values()
                .filter(|d| d.owner_id == owner_id)
                .cloned()
                .collect())
        }

        async fn delete_by_expense(&self, expense_id: Uuid) -> Result<(), String> {
            let mut distributions = self.distributions.lock().unwrap();
            distributions.retain(|_, d| d.expense_id != expense_id);
            Ok(())
        }

        async fn get_total_due_by_owner(&self, owner_id: Uuid) -> Result<f64, String> {
            let distributions = self.distributions.lock().unwrap();
            let total = distributions
                .values()
                .filter(|d| d.owner_id == owner_id)
                .map(|d| d.amount_due)
                .sum();
            Ok(total)
        }
    }

    // ========== Mock ExpenseRepository ==========

    struct MockExpenseRepository {
        expenses: Mutex<HashMap<Uuid, Expense>>,
    }

    impl MockExpenseRepository {
        fn new() -> Self {
            Self {
                expenses: Mutex::new(HashMap::new()),
            }
        }

        fn with_expense(expense: Expense) -> Self {
            let mut map = HashMap::new();
            map.insert(expense.id, expense);
            Self {
                expenses: Mutex::new(map),
            }
        }
    }

    #[async_trait]
    impl ExpenseRepository for MockExpenseRepository {
        async fn create(&self, expense: &Expense) -> Result<Expense, String> {
            let mut expenses = self.expenses.lock().unwrap();
            expenses.insert(expense.id, expense.clone());
            Ok(expense.clone())
        }

        async fn find_by_id(&self, id: Uuid) -> Result<Option<Expense>, String> {
            let expenses = self.expenses.lock().unwrap();
            Ok(expenses.get(&id).cloned())
        }

        async fn find_by_building(&self, building_id: Uuid) -> Result<Vec<Expense>, String> {
            let expenses = self.expenses.lock().unwrap();
            Ok(expenses
                .values()
                .filter(|e| e.building_id == building_id)
                .cloned()
                .collect())
        }

        async fn find_all_paginated(
            &self,
            _page_request: &PageRequest,
            _filters: &ExpenseFilters,
        ) -> Result<(Vec<Expense>, i64), String> {
            let expenses = self.expenses.lock().unwrap();
            let all: Vec<Expense> = expenses.values().cloned().collect();
            let count = all.len() as i64;
            Ok((all, count))
        }

        async fn update(&self, expense: &Expense) -> Result<Expense, String> {
            let mut expenses = self.expenses.lock().unwrap();
            expenses.insert(expense.id, expense.clone());
            Ok(expense.clone())
        }

        async fn delete(&self, id: Uuid) -> Result<bool, String> {
            let mut expenses = self.expenses.lock().unwrap();
            Ok(expenses.remove(&id).is_some())
        }
    }

    // ========== Mock UnitOwnerRepository ==========

    /// (unit_id, owner_id, percentage) tuples grouped by building_id
    type BuildingOwnerships = HashMap<Uuid, Vec<(Uuid, Uuid, f64)>>;

    struct MockUnitOwnerRepository {
        /// Stores active building ownerships: building_id -> Vec<(unit_id, owner_id, percentage)>
        building_ownerships: Mutex<BuildingOwnerships>,
    }

    impl MockUnitOwnerRepository {
        fn new() -> Self {
            Self {
                building_ownerships: Mutex::new(HashMap::new()),
            }
        }

        fn with_building_ownerships(building_id: Uuid, ownerships: Vec<(Uuid, Uuid, f64)>) -> Self {
            let mut map = HashMap::new();
            map.insert(building_id, ownerships);
            Self {
                building_ownerships: Mutex::new(map),
            }
        }
    }

    #[async_trait]
    impl UnitOwnerRepository for MockUnitOwnerRepository {
        async fn create(&self, _unit_owner: &UnitOwner) -> Result<UnitOwner, String> {
            unimplemented!("not needed for charge distribution tests")
        }

        async fn find_by_id(&self, _id: Uuid) -> Result<Option<UnitOwner>, String> {
            unimplemented!("not needed for charge distribution tests")
        }

        async fn find_current_owners_by_unit(
            &self,
            _unit_id: Uuid,
        ) -> Result<Vec<UnitOwner>, String> {
            unimplemented!("not needed for charge distribution tests")
        }

        async fn find_current_units_by_owner(
            &self,
            _owner_id: Uuid,
        ) -> Result<Vec<UnitOwner>, String> {
            unimplemented!("not needed for charge distribution tests")
        }

        async fn find_all_owners_by_unit(&self, _unit_id: Uuid) -> Result<Vec<UnitOwner>, String> {
            unimplemented!("not needed for charge distribution tests")
        }

        async fn find_all_units_by_owner(&self, _owner_id: Uuid) -> Result<Vec<UnitOwner>, String> {
            unimplemented!("not needed for charge distribution tests")
        }

        async fn update(&self, _unit_owner: &UnitOwner) -> Result<UnitOwner, String> {
            unimplemented!("not needed for charge distribution tests")
        }

        async fn delete(&self, _id: Uuid) -> Result<(), String> {
            unimplemented!("not needed for charge distribution tests")
        }

        async fn has_active_owners(&self, _unit_id: Uuid) -> Result<bool, String> {
            unimplemented!("not needed for charge distribution tests")
        }

        async fn get_total_ownership_percentage(&self, _unit_id: Uuid) -> Result<f64, String> {
            unimplemented!("not needed for charge distribution tests")
        }

        async fn find_active_by_unit_and_owner(
            &self,
            _unit_id: Uuid,
            _owner_id: Uuid,
        ) -> Result<Option<UnitOwner>, String> {
            unimplemented!("not needed for charge distribution tests")
        }

        async fn find_active_by_building(
            &self,
            building_id: Uuid,
        ) -> Result<Vec<(Uuid, Uuid, f64)>, String> {
            let ownerships = self.building_ownerships.lock().unwrap();
            Ok(ownerships.get(&building_id).cloned().unwrap_or_default())
        }
    }

    // ========== Helpers ==========

    fn make_approved_expense(building_id: Uuid, amount_incl_vat: f64) -> Expense {
        let now = Utc::now();
        Expense {
            id: Uuid::new_v4(),
            organization_id: Uuid::new_v4(),
            building_id,
            category: ExpenseCategory::Maintenance,
            description: "Elevator maintenance".to_string(),
            amount: amount_incl_vat,
            amount_excl_vat: Some(amount_incl_vat / 1.21),
            vat_rate: Some(21.0),
            vat_amount: Some(amount_incl_vat - amount_incl_vat / 1.21),
            amount_incl_vat: Some(amount_incl_vat),
            expense_date: now,
            invoice_date: Some(now),
            due_date: None,
            paid_date: None,
            approval_status: ApprovalStatus::Approved,
            submitted_at: Some(now),
            approved_by: Some(Uuid::new_v4()),
            approved_at: Some(now),
            rejection_reason: None,
            payment_status: PaymentStatus::Pending,
            supplier: Some("Schindler SA".to_string()),
            invoice_number: Some("INV-001".to_string()),
            account_code: Some("611002".to_string()),
            contractor_report_id: None,
            created_at: now,
            updated_at: now,
        }
    }

    fn make_draft_expense(building_id: Uuid) -> Expense {
        let now = Utc::now();
        Expense {
            id: Uuid::new_v4(),
            organization_id: Uuid::new_v4(),
            building_id,
            category: ExpenseCategory::Maintenance,
            description: "Draft expense".to_string(),
            amount: 1000.0,
            amount_excl_vat: None,
            vat_rate: None,
            vat_amount: None,
            amount_incl_vat: None,
            expense_date: now,
            invoice_date: None,
            due_date: None,
            paid_date: None,
            approval_status: ApprovalStatus::Draft,
            submitted_at: None,
            approved_by: None,
            approved_at: None,
            rejection_reason: None,
            payment_status: PaymentStatus::Pending,
            supplier: None,
            invoice_number: None,
            account_code: None,
            contractor_report_id: None,
            created_at: now,
            updated_at: now,
        }
    }

    fn make_use_cases(
        dist_repo: MockChargeDistributionRepository,
        expense_repo: MockExpenseRepository,
        unit_owner_repo: MockUnitOwnerRepository,
    ) -> ChargeDistributionUseCases {
        ChargeDistributionUseCases::new(
            Arc::new(dist_repo),
            Arc::new(expense_repo),
            Arc::new(unit_owner_repo),
        )
    }

    // ========== Tests ==========

    #[tokio::test]
    async fn test_calculate_and_save_distribution_success() {
        let building_id = Uuid::new_v4();
        let unit1_id = Uuid::new_v4();
        let unit2_id = Uuid::new_v4();
        let owner1_id = Uuid::new_v4();
        let owner2_id = Uuid::new_v4();

        let expense = make_approved_expense(building_id, 1000.0);
        let expense_id = expense.id;

        let ownerships = vec![
            (unit1_id, owner1_id, 0.60), // 60%
            (unit2_id, owner2_id, 0.40), // 40%
        ];

        let dist_repo = MockChargeDistributionRepository::new();
        let expense_repo = MockExpenseRepository::with_expense(expense);
        let unit_owner_repo =
            MockUnitOwnerRepository::with_building_ownerships(building_id, ownerships);

        let uc = make_use_cases(dist_repo, expense_repo, unit_owner_repo);

        let result = uc.calculate_and_save_distribution(expense_id).await;
        assert!(result.is_ok());

        let distributions = result.unwrap();
        assert_eq!(distributions.len(), 2);

        // Verify amounts: 60% of 1000 = 600, 40% of 1000 = 400
        let total_amount: f64 = distributions.iter().map(|d| d.amount_due).sum();
        assert!((total_amount - 1000.0).abs() < 0.01);

        // Verify all distributions reference the correct expense
        assert!(distributions
            .iter()
            .all(|d| d.expense_id == expense_id.to_string()));
    }

    #[tokio::test]
    async fn test_calculate_and_save_distribution_non_approved_expense() {
        let building_id = Uuid::new_v4();
        let expense = make_draft_expense(building_id);
        let expense_id = expense.id;

        let dist_repo = MockChargeDistributionRepository::new();
        let expense_repo = MockExpenseRepository::with_expense(expense);
        let unit_owner_repo = MockUnitOwnerRepository::new();

        let uc = make_use_cases(dist_repo, expense_repo, unit_owner_repo);

        let result = uc.calculate_and_save_distribution(expense_id).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("non-approved invoice"));
    }

    #[tokio::test]
    async fn test_calculate_and_save_distribution_expense_not_found() {
        let dist_repo = MockChargeDistributionRepository::new();
        let expense_repo = MockExpenseRepository::new(); // empty
        let unit_owner_repo = MockUnitOwnerRepository::new();

        let uc = make_use_cases(dist_repo, expense_repo, unit_owner_repo);

        let result = uc.calculate_and_save_distribution(Uuid::new_v4()).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Expense/Invoice not found");
    }

    #[tokio::test]
    async fn test_calculate_and_save_distribution_no_active_owners() {
        let building_id = Uuid::new_v4();
        let expense = make_approved_expense(building_id, 1000.0);
        let expense_id = expense.id;

        let dist_repo = MockChargeDistributionRepository::new();
        let expense_repo = MockExpenseRepository::with_expense(expense);
        // Empty ownerships for the building
        let unit_owner_repo =
            MockUnitOwnerRepository::with_building_ownerships(building_id, vec![]);

        let uc = make_use_cases(dist_repo, expense_repo, unit_owner_repo);

        let result = uc.calculate_and_save_distribution(expense_id).await;
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "No active unit-owner relationships found for this building"
        );
    }

    #[tokio::test]
    async fn test_get_distribution_by_expense() {
        let building_id = Uuid::new_v4();
        let unit1_id = Uuid::new_v4();
        let owner1_id = Uuid::new_v4();

        let expense = make_approved_expense(building_id, 500.0);
        let expense_id = expense.id;

        let ownerships = vec![(unit1_id, owner1_id, 1.0)]; // 100%

        let dist_repo = MockChargeDistributionRepository::new();
        let expense_repo = MockExpenseRepository::with_expense(expense);
        let unit_owner_repo =
            MockUnitOwnerRepository::with_building_ownerships(building_id, ownerships);

        let uc = make_use_cases(dist_repo, expense_repo, unit_owner_repo);

        // First calculate and save
        uc.calculate_and_save_distribution(expense_id)
            .await
            .unwrap();

        // Then retrieve by expense
        let result = uc.get_distribution_by_expense(expense_id).await;
        assert!(result.is_ok());
        let distributions = result.unwrap();
        assert_eq!(distributions.len(), 1);
        assert_eq!(distributions[0].expense_id, expense_id.to_string());
        assert_eq!(distributions[0].quota_percentage, 1.0);
        assert!((distributions[0].amount_due - 500.0).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_get_distribution_by_expense_empty() {
        let dist_repo = MockChargeDistributionRepository::new();
        let expense_repo = MockExpenseRepository::new();
        let unit_owner_repo = MockUnitOwnerRepository::new();

        let uc = make_use_cases(dist_repo, expense_repo, unit_owner_repo);

        let result = uc.get_distribution_by_expense(Uuid::new_v4()).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_get_total_due_by_owner() {
        let building_id = Uuid::new_v4();
        let unit1_id = Uuid::new_v4();
        let unit2_id = Uuid::new_v4();
        let owner1_id = Uuid::new_v4();
        let owner2_id = Uuid::new_v4();

        // Create 2 approved expenses
        let expense1 = make_approved_expense(building_id, 1000.0);
        let expense1_id = expense1.id;
        let expense2 = make_approved_expense(building_id, 2000.0);
        let expense2_id = expense2.id;

        let ownerships = vec![
            (unit1_id, owner1_id, 0.60), // 60%
            (unit2_id, owner2_id, 0.40), // 40%
        ];

        let dist_repo = MockChargeDistributionRepository::new();
        let mut expense_map = HashMap::new();
        expense_map.insert(expense1.id, expense1);
        expense_map.insert(expense2.id, expense2);
        let expense_repo = MockExpenseRepository {
            expenses: Mutex::new(expense_map),
        };
        let unit_owner_repo =
            MockUnitOwnerRepository::with_building_ownerships(building_id, ownerships);

        let uc = make_use_cases(dist_repo, expense_repo, unit_owner_repo);

        // Calculate distributions for both expenses
        uc.calculate_and_save_distribution(expense1_id)
            .await
            .unwrap();
        uc.calculate_and_save_distribution(expense2_id)
            .await
            .unwrap();

        // Owner 1 owes 60% of 1000 + 60% of 2000 = 600 + 1200 = 1800
        let total = uc.get_total_due_by_owner(owner1_id).await.unwrap();
        assert!((total - 1800.0).abs() < 0.01);

        // Owner 2 owes 40% of 1000 + 40% of 2000 = 400 + 800 = 1200
        let total = uc.get_total_due_by_owner(owner2_id).await.unwrap();
        assert!((total - 1200.0).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_get_total_due_by_owner_no_distributions() {
        let dist_repo = MockChargeDistributionRepository::new();
        let expense_repo = MockExpenseRepository::new();
        let unit_owner_repo = MockUnitOwnerRepository::new();

        let uc = make_use_cases(dist_repo, expense_repo, unit_owner_repo);

        let total = uc.get_total_due_by_owner(Uuid::new_v4()).await.unwrap();
        assert_eq!(total, 0.0);
    }

    #[tokio::test]
    async fn test_calculate_distribution_uses_amount_incl_vat() {
        let building_id = Uuid::new_v4();
        let unit_id = Uuid::new_v4();
        let owner_id = Uuid::new_v4();

        // Expense with amount_incl_vat = 1210, amount = 1000 (backward compat)
        let now = Utc::now();
        let expense = Expense {
            id: Uuid::new_v4(),
            organization_id: Uuid::new_v4(),
            building_id,
            category: ExpenseCategory::Utilities,
            description: "Electricity".to_string(),
            amount: 1000.0,
            amount_excl_vat: Some(1000.0),
            vat_rate: Some(21.0),
            vat_amount: Some(210.0),
            amount_incl_vat: Some(1210.0),
            expense_date: now,
            invoice_date: Some(now),
            due_date: None,
            paid_date: None,
            approval_status: ApprovalStatus::Approved,
            submitted_at: Some(now),
            approved_by: Some(Uuid::new_v4()),
            approved_at: Some(now),
            rejection_reason: None,
            payment_status: PaymentStatus::Pending,
            supplier: None,
            invoice_number: None,
            account_code: None,
            contractor_report_id: None,
            created_at: now,
            updated_at: now,
        };
        let expense_id = expense.id;

        let ownerships = vec![(unit_id, owner_id, 1.0)]; // 100%

        let dist_repo = MockChargeDistributionRepository::new();
        let expense_repo = MockExpenseRepository::with_expense(expense);
        let unit_owner_repo =
            MockUnitOwnerRepository::with_building_ownerships(building_id, ownerships);

        let uc = make_use_cases(dist_repo, expense_repo, unit_owner_repo);

        let result = uc
            .calculate_and_save_distribution(expense_id)
            .await
            .unwrap();

        // Should use amount_incl_vat (1210), not amount (1000)
        assert_eq!(result.len(), 1);
        assert!((result[0].amount_due - 1210.0).abs() < 0.01);
    }
}
