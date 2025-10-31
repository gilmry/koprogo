use crate::application::dto::{
    CreateUnitDto, PageRequest, UnitFilters, UnitResponseDto, UpdateUnitDto,
};
use crate::application::ports::UnitRepository;
use crate::domain::entities::Unit;
use std::sync::Arc;
use uuid::Uuid;

pub struct UnitUseCases {
    repository: Arc<dyn UnitRepository>,
}

impl UnitUseCases {
    pub fn new(repository: Arc<dyn UnitRepository>) -> Self {
        Self { repository }
    }

    pub async fn create_unit(&self, dto: CreateUnitDto) -> Result<UnitResponseDto, String> {
        let organization_id = Uuid::parse_str(&dto.organization_id)
            .map_err(|_| "Invalid organization_id format".to_string())?;
        let building_id = Uuid::parse_str(&dto.building_id)
            .map_err(|_| "Invalid building ID format".to_string())?;

        let unit = Unit::new(
            organization_id,
            building_id,
            dto.unit_number,
            dto.unit_type,
            dto.floor,
            dto.surface_area,
            dto.quota,
        )?;

        let created = self.repository.create(&unit).await?;
        Ok(self.to_response_dto(&created))
    }

    pub async fn get_unit(&self, id: Uuid) -> Result<Option<UnitResponseDto>, String> {
        let unit = self.repository.find_by_id(id).await?;
        Ok(unit.map(|u| self.to_response_dto(&u)))
    }

    pub async fn list_units_by_building(
        &self,
        building_id: Uuid,
    ) -> Result<Vec<UnitResponseDto>, String> {
        let units = self.repository.find_by_building(building_id).await?;
        Ok(units.iter().map(|u| self.to_response_dto(u)).collect())
    }

    pub async fn list_units_paginated(
        &self,
        page_request: &PageRequest,
        organization_id: Option<Uuid>,
    ) -> Result<(Vec<UnitResponseDto>, i64), String> {
        let filters = UnitFilters {
            organization_id,
            ..Default::default()
        };

        let (units, total) = self
            .repository
            .find_all_paginated(page_request, &filters)
            .await?;

        let dtos = units.iter().map(|u| self.to_response_dto(u)).collect();
        Ok((dtos, total))
    }

    pub async fn update_unit(
        &self,
        id: Uuid,
        dto: UpdateUnitDto,
    ) -> Result<UnitResponseDto, String> {
        // Get existing unit
        let mut unit = self
            .repository
            .find_by_id(id)
            .await?
            .ok_or("Unit not found".to_string())?;

        // Update unit fields
        unit.unit_number = dto.unit_number;
        unit.unit_type = dto.unit_type;
        unit.floor = Some(dto.floor);
        unit.surface_area = dto.surface_area;
        unit.quota = dto.quota;
        unit.updated_at = chrono::Utc::now();

        // Validate the updated unit
        unit.validate_update()?;

        // Save updated unit
        let updated = self.repository.update(&unit).await?;
        Ok(self.to_response_dto(&updated))
    }

    pub async fn assign_owner(
        &self,
        unit_id: Uuid,
        owner_id: Uuid,
    ) -> Result<UnitResponseDto, String> {
        let mut unit = self
            .repository
            .find_by_id(unit_id)
            .await?
            .ok_or_else(|| "Unit not found".to_string())?;

        unit.assign_owner(owner_id);

        let updated = self.repository.update(&unit).await?;
        Ok(self.to_response_dto(&updated))
    }

    pub async fn delete_unit(&self, id: Uuid) -> Result<bool, String> {
        // Check if unit exists
        let _unit = self
            .repository
            .find_by_id(id)
            .await?
            .ok_or("Unit not found".to_string())?;

        // Delete the unit
        self.repository.delete(id).await
    }

    fn to_response_dto(&self, unit: &Unit) -> UnitResponseDto {
        UnitResponseDto {
            id: unit.id.to_string(),
            building_id: unit.building_id.to_string(),
            unit_number: unit.unit_number.clone(),
            unit_type: unit.unit_type.clone(),
            floor: unit.floor,
            surface_area: unit.surface_area,
            quota: unit.quota,
            owner_id: unit.owner_id.map(|id| id.to_string()),
        }
    }
}
