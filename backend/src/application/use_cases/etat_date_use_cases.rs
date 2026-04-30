use crate::application::dto::{
    CreateEtatDateRequest, EtatDateResponse, EtatDateStatsResponse, PageRequest,
    UpdateEtatDateAdditionalDataRequest, UpdateEtatDateFinancialRequest,
};
use crate::application::ports::{
    BuildingRepository, EtatDateRepository, UnitOwnerRepository, UnitRepository,
};
use crate::domain::entities::{EtatDate, EtatDateStatus};
use f64;
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
            .find_current_owners_by_unit(request.unit_id)
            .await?;

        if unit_owners.is_empty() {
            return Err("Unit has no active owners".to_string());
        }

        // Calculate total quote-parts (should be 100% or close)
        let total_quota: f64 = unit_owners.iter().map(|uo| uo.ownership_percentage).sum();

        // For simplicity, use total quota as both ordinary and extraordinary
        // In a real system, these might be stored separately per unit
        let ordinary_charges_quota = total_quota * 100.0; // Convert to ownership_percentage
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
            unit.floor.map(|f| f.to_string()),
            Some(unit.surface_area),
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
    pub async fn get_stats(&self, organization_id: Uuid) -> Result<EtatDateStatsResponse, String> {
        self.repository.get_stats(organization_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::dto::EtatDateStatsResponse;
    use crate::application::ports::{
        BuildingRepository, EtatDateRepository, UnitOwnerRepository, UnitRepository,
    };
    use crate::domain::entities::{
        Building, EtatDate, EtatDateLanguage, EtatDateStatus, Unit, UnitOwner, UnitType,
    };
    use chrono::Utc;
    use mockall::mock;
    use mockall::predicate::*;

    // --- Mock repositories ---

    mock! {
        pub EtatDateRepo {}

        #[async_trait::async_trait]
        impl EtatDateRepository for EtatDateRepo {
            async fn create(&self, etat_date: &EtatDate) -> Result<EtatDate, String>;
            async fn find_by_id(&self, id: Uuid) -> Result<Option<EtatDate>, String>;
            async fn find_by_reference_number(&self, reference_number: &str) -> Result<Option<EtatDate>, String>;
            async fn find_by_unit(&self, unit_id: Uuid) -> Result<Vec<EtatDate>, String>;
            async fn find_by_building(&self, building_id: Uuid) -> Result<Vec<EtatDate>, String>;
            async fn find_all_paginated(
                &self,
                page_request: &PageRequest,
                organization_id: Option<Uuid>,
                status: Option<EtatDateStatus>,
            ) -> Result<(Vec<EtatDate>, i64), String>;
            async fn find_overdue(&self, organization_id: Uuid) -> Result<Vec<EtatDate>, String>;
            async fn find_expired(&self, organization_id: Uuid) -> Result<Vec<EtatDate>, String>;
            async fn update(&self, etat_date: &EtatDate) -> Result<EtatDate, String>;
            async fn delete(&self, id: Uuid) -> Result<bool, String>;
            async fn get_stats(&self, organization_id: Uuid) -> Result<EtatDateStatsResponse, String>;
            async fn count_by_status(&self, organization_id: Uuid, status: EtatDateStatus) -> Result<i64, String>;
        }
    }

    mock! {
        pub UnitRepo {}

        #[async_trait::async_trait]
        impl UnitRepository for UnitRepo {
            async fn create(&self, unit: &Unit) -> Result<Unit, String>;
            async fn find_by_id(&self, id: Uuid) -> Result<Option<Unit>, String>;
            async fn find_by_building(&self, building_id: Uuid) -> Result<Vec<Unit>, String>;
            async fn find_by_owner(&self, owner_id: Uuid) -> Result<Vec<Unit>, String>;
            async fn find_all_paginated(
                &self,
                page_request: &crate::application::dto::PageRequest,
                filters: &crate::application::dto::UnitFilters,
            ) -> Result<(Vec<Unit>, i64), String>;
            async fn update(&self, unit: &Unit) -> Result<Unit, String>;
            async fn delete(&self, id: Uuid) -> Result<bool, String>;
        }
    }

    mock! {
        pub BuildingRepo {}

        #[async_trait::async_trait]
        impl BuildingRepository for BuildingRepo {
            async fn create(&self, building: &Building) -> Result<Building, String>;
            async fn find_by_id(&self, id: Uuid) -> Result<Option<Building>, String>;
            async fn find_by_slug(&self, slug: &str) -> Result<Option<Building>, String>;
            async fn find_all(&self) -> Result<Vec<Building>, String>;
            async fn find_all_paginated(
                &self,
                page_request: &crate::application::dto::PageRequest,
                filters: &crate::application::dto::BuildingFilters,
            ) -> Result<(Vec<Building>, i64), String>;
            async fn update(&self, building: &Building) -> Result<Building, String>;
            async fn delete(&self, id: Uuid) -> Result<bool, String>;
        }
    }

    mock! {
        pub UnitOwnerRepo {}

        #[async_trait::async_trait]
        impl UnitOwnerRepository for UnitOwnerRepo {
            async fn create(&self, unit_owner: &UnitOwner) -> Result<UnitOwner, String>;
            async fn find_by_id(&self, id: Uuid) -> Result<Option<UnitOwner>, String>;
            async fn find_current_owners_by_unit(&self, unit_id: Uuid) -> Result<Vec<UnitOwner>, String>;
            async fn find_current_units_by_owner(&self, owner_id: Uuid) -> Result<Vec<UnitOwner>, String>;
            async fn find_all_owners_by_unit(&self, unit_id: Uuid) -> Result<Vec<UnitOwner>, String>;
            async fn find_all_units_by_owner(&self, owner_id: Uuid) -> Result<Vec<UnitOwner>, String>;
            async fn update(&self, unit_owner: &UnitOwner) -> Result<UnitOwner, String>;
            async fn delete(&self, id: Uuid) -> Result<(), String>;
            async fn has_active_owners(&self, unit_id: Uuid) -> Result<bool, String>;
            async fn get_total_ownership_percentage(&self, unit_id: Uuid) -> Result<f64, String>;
            async fn find_active_by_unit_and_owner(&self, unit_id: Uuid, owner_id: Uuid) -> Result<Option<UnitOwner>, String>;
            async fn find_active_by_building(&self, building_id: Uuid) -> Result<Vec<(Uuid, Uuid, f64)>, String>;
        }
    }

    // --- Helpers ---

    fn make_building(org_id: Uuid) -> Building {
        Building::new(
            org_id,
            "Résidence Les Jardins".to_string(),
            "Rue de la Loi 123".to_string(),
            "Bruxelles".to_string(),
            "1000".to_string(),
            "Belgium".to_string(),
            25,
            1000,
            Some(2020),
        )
        .unwrap()
    }

    fn make_unit(org_id: Uuid, building_id: Uuid) -> Unit {
        Unit::new(
            org_id,
            building_id,
            "101".to_string(),
            UnitType::Apartment,
            Some(1),
            85.0,
            50.0,
        )
        .unwrap()
    }

    fn make_unit_owner(unit_id: Uuid) -> UnitOwner {
        UnitOwner::new(unit_id, Uuid::new_v4(), 0.5, true).unwrap()
    }

    fn make_etat_date(org_id: Uuid, building_id: Uuid, unit_id: Uuid) -> EtatDate {
        EtatDate::new(
            org_id,
            building_id,
            unit_id,
            Utc::now(),
            EtatDateLanguage::Fr,
            "Maitre Dupont".to_string(),
            "dupont@notaire.be".to_string(),
            Some("+32 2 123 4567".to_string()),
            "Residence Les Jardins".to_string(),
            "Rue de la Loi 123, 1000 Bruxelles".to_string(),
            "101".to_string(),
            Some("1".to_string()),
            Some(85.0),
            50.0,
            50.0,
        )
        .unwrap()
    }

    fn make_create_request(
        org_id: Uuid,
        building_id: Uuid,
        unit_id: Uuid,
    ) -> CreateEtatDateRequest {
        CreateEtatDateRequest {
            organization_id: org_id,
            building_id,
            unit_id,
            reference_date: Utc::now(),
            language: EtatDateLanguage::Fr,
            notary_name: "Maitre Dupont".to_string(),
            notary_email: "dupont@notaire.be".to_string(),
            notary_phone: Some("+32 2 123 4567".to_string()),
        }
    }

    fn build_use_cases(
        etat_repo: MockEtatDateRepo,
        unit_repo: MockUnitRepo,
        building_repo: MockBuildingRepo,
        uo_repo: MockUnitOwnerRepo,
    ) -> EtatDateUseCases {
        EtatDateUseCases::new(
            Arc::new(etat_repo),
            Arc::new(unit_repo),
            Arc::new(building_repo),
            Arc::new(uo_repo),
        )
    }

    // --- Tests ---

    #[tokio::test]
    async fn test_create_etat_date_success() {
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let unit_id = Uuid::new_v4();

        let mut etat_repo = MockEtatDateRepo::new();
        let mut unit_repo = MockUnitRepo::new();
        let mut building_repo = MockBuildingRepo::new();
        let mut uo_repo = MockUnitOwnerRepo::new();

        // Mock unit exists
        let unit = make_unit(org_id, building_id);
        unit_repo
            .expect_find_by_id()
            .with(eq(unit_id))
            .times(1)
            .returning(move |_| Ok(Some(unit.clone())));

        // Mock building exists
        let building = make_building(org_id);
        building_repo
            .expect_find_by_id()
            .with(eq(building_id))
            .times(1)
            .returning(move |_| Ok(Some(building.clone())));

        // Mock unit owners exist with 50% ownership
        let uo = make_unit_owner(unit_id);
        uo_repo
            .expect_find_current_owners_by_unit()
            .with(eq(unit_id))
            .times(1)
            .returning(move |_| Ok(vec![uo.clone()]));

        // Mock create returns the entity
        etat_repo
            .expect_create()
            .times(1)
            .returning(|ed| Ok(ed.clone()));

        let use_cases = build_use_cases(etat_repo, unit_repo, building_repo, uo_repo);
        let request = make_create_request(org_id, building_id, unit_id);

        let result = use_cases.create_etat_date(request).await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.status, EtatDateStatus::Requested);
        assert!(response.reference_number.starts_with("ED-"));
        assert_eq!(response.notary_name, "Maitre Dupont");
        assert_eq!(response.notary_email, "dupont@notaire.be");
    }

    #[tokio::test]
    async fn test_create_etat_date_unit_not_found() {
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let unit_id = Uuid::new_v4();

        let etat_repo = MockEtatDateRepo::new();
        let mut unit_repo = MockUnitRepo::new();
        let building_repo = MockBuildingRepo::new();
        let uo_repo = MockUnitOwnerRepo::new();

        // Mock unit not found
        unit_repo
            .expect_find_by_id()
            .with(eq(unit_id))
            .times(1)
            .returning(|_| Ok(None));

        let use_cases = build_use_cases(etat_repo, unit_repo, building_repo, uo_repo);
        let request = make_create_request(org_id, building_id, unit_id);

        let result = use_cases.create_etat_date(request).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Unit not found");
    }

    #[tokio::test]
    async fn test_create_etat_date_no_active_owners() {
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let unit_id = Uuid::new_v4();

        let etat_repo = MockEtatDateRepo::new();
        let mut unit_repo = MockUnitRepo::new();
        let mut building_repo = MockBuildingRepo::new();
        let mut uo_repo = MockUnitOwnerRepo::new();

        let unit = make_unit(org_id, building_id);
        unit_repo
            .expect_find_by_id()
            .with(eq(unit_id))
            .times(1)
            .returning(move |_| Ok(Some(unit.clone())));

        let building = make_building(org_id);
        building_repo
            .expect_find_by_id()
            .with(eq(building_id))
            .times(1)
            .returning(move |_| Ok(Some(building.clone())));

        // No active owners
        uo_repo
            .expect_find_current_owners_by_unit()
            .with(eq(unit_id))
            .times(1)
            .returning(|_| Ok(vec![]));

        let use_cases = build_use_cases(etat_repo, unit_repo, building_repo, uo_repo);
        let request = make_create_request(org_id, building_id, unit_id);

        let result = use_cases.create_etat_date(request).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Unit has no active owners");
    }

    #[tokio::test]
    async fn test_find_by_id_success() {
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let unit_id = Uuid::new_v4();
        let etat_date = make_etat_date(org_id, building_id, unit_id);
        let etat_id = etat_date.id;

        let mut etat_repo = MockEtatDateRepo::new();
        etat_repo
            .expect_find_by_id()
            .with(eq(etat_id))
            .times(1)
            .returning(move |_| Ok(Some(etat_date.clone())));

        let use_cases = build_use_cases(
            etat_repo,
            MockUnitRepo::new(),
            MockBuildingRepo::new(),
            MockUnitOwnerRepo::new(),
        );

        let result = use_cases.get_etat_date(etat_id).await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.is_some());
        assert_eq!(response.unwrap().id, etat_id);
    }

    #[tokio::test]
    async fn test_find_by_reference_number_success() {
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let unit_id = Uuid::new_v4();
        let etat_date = make_etat_date(org_id, building_id, unit_id);
        let ref_number = etat_date.reference_number.clone();

        let mut etat_repo = MockEtatDateRepo::new();
        etat_repo
            .expect_find_by_reference_number()
            .with(eq(ref_number.clone()))
            .times(1)
            .returning(move |_| Ok(Some(etat_date.clone())));

        let use_cases = build_use_cases(
            etat_repo,
            MockUnitRepo::new(),
            MockBuildingRepo::new(),
            MockUnitOwnerRepo::new(),
        );

        let result = use_cases.get_by_reference_number(&ref_number).await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.is_some());
        assert_eq!(response.unwrap().reference_number, ref_number);
    }

    #[tokio::test]
    async fn test_mark_in_progress_success() {
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let unit_id = Uuid::new_v4();
        let etat_date = make_etat_date(org_id, building_id, unit_id);
        let etat_id = etat_date.id;

        let mut etat_repo = MockEtatDateRepo::new();

        // find_by_id returns a Requested etat date
        etat_repo
            .expect_find_by_id()
            .with(eq(etat_id))
            .times(1)
            .returning(move |_| Ok(Some(etat_date.clone())));

        // update returns the updated entity
        etat_repo
            .expect_update()
            .times(1)
            .returning(|ed| Ok(ed.clone()));

        let use_cases = build_use_cases(
            etat_repo,
            MockUnitRepo::new(),
            MockBuildingRepo::new(),
            MockUnitOwnerRepo::new(),
        );

        let result = use_cases.mark_in_progress(etat_id).await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.status, EtatDateStatus::InProgress);
    }

    #[tokio::test]
    async fn test_mark_generated_with_pdf_path() {
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let unit_id = Uuid::new_v4();
        let mut etat_date = make_etat_date(org_id, building_id, unit_id);
        // Must be InProgress to transition to Generated
        etat_date.status = EtatDateStatus::InProgress;
        let etat_id = etat_date.id;

        let mut etat_repo = MockEtatDateRepo::new();

        etat_repo
            .expect_find_by_id()
            .with(eq(etat_id))
            .times(1)
            .returning(move |_| Ok(Some(etat_date.clone())));

        etat_repo
            .expect_update()
            .times(1)
            .returning(|ed| Ok(ed.clone()));

        let use_cases = build_use_cases(
            etat_repo,
            MockUnitRepo::new(),
            MockBuildingRepo::new(),
            MockUnitOwnerRepo::new(),
        );

        let pdf_path = "/documents/etat-date/ED-2026-001.pdf".to_string();
        let result = use_cases.mark_generated(etat_id, pdf_path.clone()).await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.status, EtatDateStatus::Generated);
        assert_eq!(response.pdf_file_path, Some(pdf_path));
        assert!(response.generated_date.is_some());
    }

    #[tokio::test]
    async fn test_mark_delivered_to_notary() {
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let unit_id = Uuid::new_v4();
        let mut etat_date = make_etat_date(org_id, building_id, unit_id);
        // Must be Generated to transition to Delivered
        etat_date.status = EtatDateStatus::Generated;
        etat_date.generated_date = Some(Utc::now());
        etat_date.pdf_file_path = Some("/documents/etat-date/ED-2026-001.pdf".to_string());
        let etat_id = etat_date.id;

        let mut etat_repo = MockEtatDateRepo::new();

        etat_repo
            .expect_find_by_id()
            .with(eq(etat_id))
            .times(1)
            .returning(move |_| Ok(Some(etat_date.clone())));

        etat_repo
            .expect_update()
            .times(1)
            .returning(|ed| Ok(ed.clone()));

        let use_cases = build_use_cases(
            etat_repo,
            MockUnitRepo::new(),
            MockBuildingRepo::new(),
            MockUnitOwnerRepo::new(),
        );

        let result = use_cases.mark_delivered(etat_id).await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.status, EtatDateStatus::Delivered);
        assert!(response.delivered_date.is_some());
    }

    #[tokio::test]
    async fn test_update_financial_data_success() {
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let unit_id = Uuid::new_v4();
        let etat_date = make_etat_date(org_id, building_id, unit_id);
        let etat_id = etat_date.id;

        let mut etat_repo = MockEtatDateRepo::new();

        etat_repo
            .expect_find_by_id()
            .with(eq(etat_id))
            .times(1)
            .returning(move |_| Ok(Some(etat_date.clone())));

        etat_repo
            .expect_update()
            .times(1)
            .returning(|ed| Ok(ed.clone()));

        let use_cases = build_use_cases(
            etat_repo,
            MockUnitRepo::new(),
            MockBuildingRepo::new(),
            MockUnitOwnerRepo::new(),
        );

        let request = UpdateEtatDateFinancialRequest {
            owner_balance: -1250.50,
            arrears_amount: 800.0,
            monthly_provision_amount: 150.0,
            total_balance: -1250.50,
            approved_works_unpaid: 3500.0,
        };

        let result = use_cases.update_financial_data(etat_id, request).await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.owner_balance, -1250.50);
        assert_eq!(response.arrears_amount, 800.0);
        assert_eq!(response.monthly_provision_amount, 150.0);
        assert_eq!(response.total_balance, -1250.50);
        assert_eq!(response.approved_works_unpaid, 3500.0);
    }

    #[tokio::test]
    async fn test_list_overdue_returns_old_requests() {
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let unit_id = Uuid::new_v4();

        let mut overdue_etat = make_etat_date(org_id, building_id, unit_id);
        // Simulate a request made 16 days ago (>15 days = overdue per Art. 3.94 CC)
        overdue_etat.requested_date = Utc::now() - chrono::Duration::days(16);

        let mut etat_repo = MockEtatDateRepo::new();
        etat_repo
            .expect_find_overdue()
            .with(eq(org_id))
            .times(1)
            .returning(move |_| Ok(vec![overdue_etat.clone()]));

        let use_cases = build_use_cases(
            etat_repo,
            MockUnitRepo::new(),
            MockBuildingRepo::new(),
            MockUnitOwnerRepo::new(),
        );

        let result = use_cases.list_overdue(org_id).await;
        assert!(result.is_ok());
        let items = result.unwrap();
        assert_eq!(items.len(), 1);
        assert!(items[0].is_overdue);
        assert!(items[0].days_since_request >= 16);
    }

    #[tokio::test]
    async fn test_list_expired_returns_old_etats() {
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let unit_id = Uuid::new_v4();

        let mut expired_etat = make_etat_date(org_id, building_id, unit_id);
        // Simulate reference date >3 months ago (>90 days = expired)
        expired_etat.reference_date = Utc::now() - chrono::Duration::days(100);

        let mut etat_repo = MockEtatDateRepo::new();
        etat_repo
            .expect_find_expired()
            .with(eq(org_id))
            .times(1)
            .returning(move |_| Ok(vec![expired_etat.clone()]));

        let use_cases = build_use_cases(
            etat_repo,
            MockUnitRepo::new(),
            MockBuildingRepo::new(),
            MockUnitOwnerRepo::new(),
        );

        let result = use_cases.list_expired(org_id).await;
        assert!(result.is_ok());
        let items = result.unwrap();
        assert_eq!(items.len(), 1);
        assert!(items[0].is_expired);
    }

    #[tokio::test]
    async fn test_mark_in_progress_not_found() {
        let etat_id = Uuid::new_v4();

        let mut etat_repo = MockEtatDateRepo::new();
        etat_repo
            .expect_find_by_id()
            .with(eq(etat_id))
            .times(1)
            .returning(|_| Ok(None));

        let use_cases = build_use_cases(
            etat_repo,
            MockUnitRepo::new(),
            MockBuildingRepo::new(),
            MockUnitOwnerRepo::new(),
        );

        let result = use_cases.mark_in_progress(etat_id).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }
}
