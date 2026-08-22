use crate::application::dto::{
    BuildingFilters, BuildingResponseDto, CreateBuildingDto, PageRequest, UpdateBuildingDto,
};
use crate::application::error::AppError;
use crate::application::ports::BuildingRepository;
use crate::domain::entities::{Building, BuildingMetrics};
use std::sync::Arc;
use uuid::Uuid;

pub struct BuildingUseCases {
    repository: Arc<dyn BuildingRepository>,
}

impl BuildingUseCases {
    pub fn new(repository: Arc<dyn BuildingRepository>) -> Self {
        Self { repository }
    }

    pub async fn create_building(
        &self,
        dto: CreateBuildingDto,
    ) -> Result<BuildingResponseDto, String> {
        // Story 1.2 — Building::acp_id (FK vers acps.id, anciennement
        // organization_id). Le DTO expose désormais `acp_id` (renommé) ;
        // le handler résout l'ACP à partir de l'organisation du JWT pour
        // les non-superadmins (cf. building_handlers::create_building).
        let acp_id =
            Uuid::parse_str(&dto.acp_id).map_err(|_| "Invalid acp_id format".to_string())?;

        let building = Building::new(
            acp_id,
            dto.name,
            dto.address,
            dto.city,
            dto.postal_code,
            dto.country,
            dto.total_units,
            dto.total_tantiemes.unwrap_or(1000),
            dto.construction_year,
        )?;

        let created = self.repository.create(&building).await?;
        Ok(self.to_response_dto(&created))
    }

    pub async fn get_building(&self, id: Uuid) -> Result<Option<BuildingResponseDto>, String> {
        let building = self.repository.find_by_id(id).await?;
        Ok(building.map(|b| self.to_response_dto(&b)))
    }

    /// Story 1.4 — Get building + metrics (count units + SUM quotas) +
    /// is_conformant + delta. Retour typé `AppError` (cluster #555).
    ///
    /// `Ok(None)` quand l'id n'existe pas (le handler le mappe en 404).
    /// Toute erreur infra remonte en `AppError::Internal` via `From<String>`.
    pub async fn get_building_with_metrics(
        &self,
        id: Uuid,
    ) -> Result<Option<BuildingResponseDto>, AppError> {
        let pair = self
            .repository
            .find_by_id_with_metrics(id)
            .await
            .map_err(AppError::from)?;
        Ok(pair.map(|(b, m)| Self::to_response_dto_with_metrics(&b, &m)))
    }

    pub async fn list_buildings(&self) -> Result<Vec<BuildingResponseDto>, String> {
        let buildings = self.repository.find_all().await?;
        Ok(buildings.iter().map(|b| self.to_response_dto(b)).collect())
    }

    pub async fn list_buildings_paginated(
        &self,
        page_request: &PageRequest,
        organization_id: Option<Uuid>,
    ) -> Result<(Vec<BuildingResponseDto>, i64), String> {
        let filters = BuildingFilters {
            organization_id,
            ..Default::default()
        };

        let (buildings, total) = self
            .repository
            .find_all_paginated(page_request, &filters)
            .await?;

        let dtos = buildings.iter().map(|b| self.to_response_dto(b)).collect();
        Ok((dtos, total))
    }

    /// Liste paginée avec filtrage Owner (BUG-WF14-2)
    /// Si owner_user_id est Some, filtre les buildings où le user possède un lot
    pub async fn list_buildings_paginated_for_user(
        &self,
        page_request: &PageRequest,
        organization_id: Option<Uuid>,
        owner_user_id: Option<Uuid>,
        search: Option<String>,
    ) -> Result<(Vec<BuildingResponseDto>, i64), String> {
        let filters = BuildingFilters {
            organization_id,
            owner_user_id,
            search,
            ..Default::default()
        };

        let (buildings, total) = self
            .repository
            .find_all_paginated(page_request, &filters)
            .await?;

        let dtos = buildings.iter().map(|b| self.to_response_dto(b)).collect();
        Ok((dtos, total))
    }

    pub async fn update_building(
        &self,
        id: Uuid,
        dto: UpdateBuildingDto,
    ) -> Result<BuildingResponseDto, String> {
        let mut building = self
            .repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| "Building not found".to_string())?;

        // Story 1.2 — Réaffectation ACP (SuperAdmin uniquement).
        if let Some(acp_id_str) = dto.acp_id {
            let acp_id =
                Uuid::parse_str(&acp_id_str).map_err(|_| "Invalid acp_id format".to_string())?;
            building.acp_id = acp_id;
        }

        building.update_info(
            dto.name,
            dto.address,
            dto.city,
            dto.postal_code,
            dto.country,
            dto.total_units,
            dto.total_tantiemes.unwrap_or(1000),
            dto.construction_year,
        );

        let updated = self.repository.update(&building).await?;
        Ok(self.to_response_dto(&updated))
    }

    pub async fn delete_building(&self, id: Uuid) -> Result<bool, String> {
        self.repository.delete(id).await
    }

    /// Find building by URL slug (for public pages - Issue #92)
    pub async fn find_by_slug(&self, slug: &str) -> Result<Option<Building>, String> {
        self.repository.find_by_slug(slug).await
    }

    fn to_response_dto(&self, building: &Building) -> BuildingResponseDto {
        // Story 1.4 : par défaut on retourne des métriques vides — les
        // callers historiques (list, update) ne paient pas le coût d'un
        // JOIN. Le path GET unique passe par `to_response_dto_with_metrics`
        // pour exposer is_conformant + delta réels.
        Self::to_response_dto_with_metrics(building, &BuildingMetrics::empty())
    }

    /// Story 1.4 — Variante exposant les métriques réelles (count units +
    /// SUM quotas) + `is_conformant` + delta Decimal-as-string.
    ///
    /// Strict Decimal : `quota_sum`/`quota_delta` sérialisés via `to_string()`
    /// (jamais `to_f64`) — cf. mémoire `no-f64-in-money` + ADR-0007.
    fn to_response_dto_with_metrics(
        building: &Building,
        metrics: &BuildingMetrics,
    ) -> BuildingResponseDto {
        let is_conformant = building.is_conformant(metrics);
        // Track H Story H1 — `quota_delta` est désormais méthode d'instance
        // (acte de base lu sur `self.total_tantiemes`).
        let delta = building.quota_delta(metrics);
        BuildingResponseDto {
            id: building.id.to_string(),
            acp_id: building.acp_id.to_string(),
            name: building.name.clone(),
            address: building.address.clone(),
            city: building.city.clone(),
            postal_code: building.postal_code.clone(),
            country: building.country.clone(),
            total_units: building.total_units,
            total_tantiemes: building.total_tantiemes,
            construction_year: building.construction_year,
            created_at: building.created_at.to_rfc3339(),
            updated_at: building.updated_at.to_rfc3339(),
            units_count: metrics.units_count,
            quota_sum: metrics.quota_sum.to_string(),
            is_conformant,
            quota_delta: delta.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::ports::BuildingRepository;
    use async_trait::async_trait;
    use mockall::mock;

    mock! {
        BuildingRepo {}

        #[async_trait]
        impl BuildingRepository for BuildingRepo {
            async fn create(&self, building: &Building) -> Result<Building, String>;
            async fn find_by_id(&self, id: Uuid) -> Result<Option<Building>, String>;
            async fn find_all(&self) -> Result<Vec<Building>, String>;
            async fn find_all_paginated(
                &self,
                page_request: &PageRequest,
                filters: &BuildingFilters,
            ) -> Result<(Vec<Building>, i64), String>;
            async fn update(&self, building: &Building) -> Result<Building, String>;
            async fn delete(&self, id: Uuid) -> Result<bool, String>;
            async fn find_by_slug(&self, slug: &str) -> Result<Option<Building>, String>;
            async fn find_by_id_with_metrics(
                &self,
                id: Uuid,
            ) -> Result<Option<(Building, BuildingMetrics)>, String>;
        }
    }

    #[tokio::test]
    async fn test_create_building_success() {
        let mut mock_repo = MockBuildingRepo::new();

        mock_repo.expect_create().returning(|b| Ok(b.clone()));

        let use_cases = BuildingUseCases::new(Arc::new(mock_repo));

        let dto = CreateBuildingDto {
            acp_id: Uuid::new_v4().to_string(),
            name: "Test Building".to_string(),
            address: "123 Test St".to_string(),
            city: "Paris".to_string(),
            postal_code: "75001".to_string(),
            country: "France".to_string(),
            total_units: 10,
            total_tantiemes: Some(1000),
            construction_year: Some(2000),
        };

        let result = use_cases.create_building(dto).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_create_building_validation_fails() {
        let mock_repo = MockBuildingRepo::new();
        let use_cases = BuildingUseCases::new(Arc::new(mock_repo));

        let dto = CreateBuildingDto {
            acp_id: Uuid::new_v4().to_string(),
            name: "".to_string(), // Invalid: empty name
            address: "123 Test St".to_string(),
            city: "Paris".to_string(),
            postal_code: "75001".to_string(),
            country: "France".to_string(),
            total_units: 10,
            total_tantiemes: Some(1000),
            construction_year: Some(2000),
        };

        let result = use_cases.create_building(dto).await;
        assert!(result.is_err());
    }
}
