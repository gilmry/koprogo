use crate::application::dto::{
    BuildingFilters, BuildingResponseDto, CreateBuildingDto, PageRequest, UpdateBuildingDto,
};
use crate::application::ports::BuildingRepository;
use crate::domain::entities::Building;
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
        let organization_id = Uuid::parse_str(&dto.organization_id)
            .map_err(|_| "Invalid organization_id format".to_string())?;

        let building = Building::new(
            organization_id,
            dto.name,
            dto.address,
            dto.city,
            dto.postal_code,
            dto.country,
            dto.total_units,
            dto.construction_year,
        )?;

        let created = self.repository.create(&building).await?;
        Ok(self.to_response_dto(&created))
    }

    pub async fn get_building(&self, id: Uuid) -> Result<Option<BuildingResponseDto>, String> {
        let building = self.repository.find_by_id(id).await?;
        Ok(building.map(|b| self.to_response_dto(&b)))
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

        // Update organization if provided (SuperAdmin feature)
        if let Some(org_id_str) = dto.organization_id {
            let org_id = Uuid::parse_str(&org_id_str)
                .map_err(|_| "Invalid organization_id format".to_string())?;
            building.organization_id = org_id;
        }

        building.update_info(
            dto.name,
            dto.address,
            dto.city,
            dto.postal_code,
            dto.country,
            dto.total_units,
            dto.construction_year,
        );

        let updated = self.repository.update(&building).await?;
        Ok(self.to_response_dto(&updated))
    }

    pub async fn delete_building(&self, id: Uuid) -> Result<bool, String> {
        self.repository.delete(id).await
    }

    fn to_response_dto(&self, building: &Building) -> BuildingResponseDto {
        BuildingResponseDto {
            id: building.id.to_string(),
            organization_id: building.organization_id.to_string(),
            name: building.name.clone(),
            address: building.address.clone(),
            city: building.city.clone(),
            postal_code: building.postal_code.clone(),
            country: building.country.clone(),
            total_units: building.total_units,
            construction_year: building.construction_year,
            created_at: building.created_at.to_rfc3339(),
            updated_at: building.updated_at.to_rfc3339(),
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
        }
    }

    #[tokio::test]
    async fn test_create_building_success() {
        let mut mock_repo = MockBuildingRepo::new();

        mock_repo.expect_create().returning(|b| Ok(b.clone()));

        let use_cases = BuildingUseCases::new(Arc::new(mock_repo));

        let dto = CreateBuildingDto {
            organization_id: Uuid::new_v4().to_string(),
            name: "Test Building".to_string(),
            address: "123 Test St".to_string(),
            city: "Paris".to_string(),
            postal_code: "75001".to_string(),
            country: "France".to_string(),
            total_units: 10,
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
            organization_id: Uuid::new_v4().to_string(),
            name: "".to_string(), // Invalid: empty name
            address: "123 Test St".to_string(),
            city: "Paris".to_string(),
            postal_code: "75001".to_string(),
            country: "France".to_string(),
            total_units: 10,
            construction_year: Some(2000),
        };

        let result = use_cases.create_building(dto).await;
        assert!(result.is_err());
    }
}
