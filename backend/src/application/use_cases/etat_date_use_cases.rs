use crate::application::dto::{
    CreateEtatDateRequest, EtatDateResponse, EtatDateStatsResponse, PageRequest,
    UpdateEtatDateAdditionalDataRequest, UpdateEtatDateFinancialRequest,
};
use crate::application::ports::{
    BuildingRepository, EtatDateRepository, UnitOwnerRepository, UnitRepository,
};
use crate::domain::entities::{EtatDate, EtatDateLanguage, EtatDateStatus};
use rust_decimal::Decimal;
use std::sync::Arc;
use uuid::Uuid;

pub struct EtatDateUseCases {
    repository: Arc<dyn EtatDateRepository>,
    unit_repository: Arc<dyn UnitRepository>,
    building_repository: Arc<dyn BuildingRepository>,
    unit_owner_repository: Arc<dyn UnitOwnerRepository>,
}

impl EtatDateUseCases {
    pub fn new(
        repository: Arc<dyn EtatDateRepository>,
        unit_repository: Arc<dyn UnitRepository>,
        building_repository: Arc<dyn BuildingRepository>,
        unit_owner_repository: Arc<dyn UnitOwnerRepository>,
    ) -> Self {
        Self {
            repository,
            unit_repository,
            building_repository,
            unit_owner_repository,
        }
    }

    /// Create a new état daté request
    pub async fn create_etat_date(
        &self,
        request: CreateEtatDateRequest,
    ) -> Result<EtatDateResponse, String> {
        // Verify unit exists
        let unit = self
            .unit_repository
            .find_by_id(request.unit_id)
            .await?
            .ok_or_else(|| "Unit not found".to_string())?;

        // Verify building exists
        let building = self
            .building_repository
            .find_by_id(request.building_id)
            .await?
            .ok_or_else(|| "Building not found".to_string())?;

        // Get unit ownership info (quotes-parts)
        let unit_owners = self
            .unit_owner_repository
            .find_active_by_unit(request.unit_id)
            .await?;

        if unit_owners.is_empty() {
            return Err("Unit has no active owners".to_string());
        }

        // Calculate total quote-parts (should be 100% or close)
        let total_quota: Decimal = unit_owners.iter().map(|uo| uo.percentage).sum();

        // For simplicity, use total quota as both ordinary and extraordinary
        // In a real system, these might be stored separately per unit
        let ordinary_charges_quota = total_quota * Decimal::new(100, 0); // Convert to percentage
        let extraordinary_charges_quota = ordinary_charges_quota;

        // Create état daté
        let etat_date = EtatDate::new(
            request.organization_id,
            request.building_id,
            request.unit_id,
            request.reference_date,
            request.language,
            request.notary_name,
            request.notary_email,
            request.notary_phone,
            building.name.clone(),
            building.address.clone(),
            unit.unit_number.clone(),
            unit.floor.clone(),
            unit.area,
            ordinary_charges_quota,
            extraordinary_charges_quota,
        )?;

        let created = self.repository.create(&etat_date).await?;
        Ok(EtatDateResponse::from(created))
    }

    /// Get état daté by ID
    pub async fn get_etat_date(&self, id: Uuid) -> Result<Option<EtatDateResponse>, String> {
        let etat_date = self.repository.find_by_id(id).await?;
        Ok(etat_date.map(EtatDateResponse::from))
    }

    /// Get état daté by reference number
    pub async fn get_by_reference_number(
        &self,
        reference_number: &str,
    ) -> Result<Option<EtatDateResponse>, String> {
        let etat_date = self
            .repository
            .find_by_reference_number(reference_number)
            .await?;
        Ok(etat_date.map(EtatDateResponse::from))
    }

    /// List états datés for a unit
    pub async fn list_by_unit(&self, unit_id: Uuid) -> Result<Vec<EtatDateResponse>, String> {
        let etats = self.repository.find_by_unit(unit_id).await?;
        Ok(etats.into_iter().map(EtatDateResponse::from).collect())
    }

    /// List états datés for a building
    pub async fn list_by_building(
        &self,
        building_id: Uuid,
    ) -> Result<Vec<EtatDateResponse>, String> {
        let etats = self.repository.find_by_building(building_id).await?;
        Ok(etats.into_iter().map(EtatDateResponse::from).collect())
    }

    /// List états datés paginated
    pub async fn list_paginated(
        &self,
        page_request: &PageRequest,
        organization_id: Option<Uuid>,
        status: Option<EtatDateStatus>,
    ) -> Result<(Vec<EtatDateResponse>, i64), String> {
        let (etats, total) = self
            .repository
            .find_all_paginated(page_request, organization_id, status)
            .await?;

        let dtos = etats.into_iter().map(EtatDateResponse::from).collect();
        Ok((dtos, total))
    }

    /// Mark état daté as in progress
    pub async fn mark_in_progress(&self, id: Uuid) -> Result<EtatDateResponse, String> {
        let mut etat_date = self
            .repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| "État daté not found".to_string())?;

        etat_date.mark_in_progress()?;

        let updated = self.repository.update(&etat_date).await?;
        Ok(EtatDateResponse::from(updated))
    }

    /// Mark état daté as generated (with PDF file path)
    pub async fn mark_generated(
        &self,
        id: Uuid,
        pdf_file_path: String,
    ) -> Result<EtatDateResponse, String> {
        let mut etat_date = self
            .repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| "État daté not found".to_string())?;

        etat_date.mark_generated(pdf_file_path)?;

        let updated = self.repository.update(&etat_date).await?;
        Ok(EtatDateResponse::from(updated))
    }

    /// Mark état daté as delivered to notary
    pub async fn mark_delivered(&self, id: Uuid) -> Result<EtatDateResponse, String> {
        let mut etat_date = self
            .repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| "État daté not found".to_string())?;

        etat_date.mark_delivered()?;

        let updated = self.repository.update(&etat_date).await?;
        Ok(EtatDateResponse::from(updated))
    }

    /// Update financial data
    pub async fn update_financial_data(
        &self,
        id: Uuid,
        request: UpdateEtatDateFinancialRequest,
    ) -> Result<EtatDateResponse, String> {
        let mut etat_date = self
            .repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| "État daté not found".to_string())?;

        etat_date.update_financial_data(
            request.owner_balance,
            request.arrears_amount,
            request.monthly_provision_amount,
            request.total_balance,
            request.approved_works_unpaid,
        )?;

        let updated = self.repository.update(&etat_date).await?;
        Ok(EtatDateResponse::from(updated))
    }

    /// Update additional data (sections 7-16)
    pub async fn update_additional_data(
        &self,
        id: Uuid,
        request: UpdateEtatDateAdditionalDataRequest,
    ) -> Result<EtatDateResponse, String> {
        let mut etat_date = self
            .repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| "État daté not found".to_string())?;

        etat_date.update_additional_data(request.additional_data)?;

        let updated = self.repository.update(&etat_date).await?;
        Ok(EtatDateResponse::from(updated))
    }

    /// Get overdue états datés (>10 days, not generated yet)
    pub async fn list_overdue(
        &self,
        organization_id: Uuid,
    ) -> Result<Vec<EtatDateResponse>, String> {
        let etats = self.repository.find_overdue(organization_id).await?;
        Ok(etats.into_iter().map(EtatDateResponse::from).collect())
    }

    /// Get expired états datés (>3 months from reference date)
    pub async fn list_expired(
        &self,
        organization_id: Uuid,
    ) -> Result<Vec<EtatDateResponse>, String> {
        let etats = self.repository.find_expired(organization_id).await?;
        Ok(etats.into_iter().map(EtatDateResponse::from).collect())
    }

    /// Delete état daté
    pub async fn delete_etat_date(&self, id: Uuid) -> Result<bool, String> {
        self.repository.delete(id).await
    }

    /// Get statistics for dashboard
    pub async fn get_stats(
        &self,
        organization_id: Uuid,
    ) -> Result<EtatDateStatsResponse, String> {
        self.repository.get_stats(organization_id).await
    }
}
