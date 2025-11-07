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
