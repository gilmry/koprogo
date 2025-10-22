use crate::application::dto::{CreateUnitDto, UnitResponseDto};
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
        let building_id = Uuid::parse_str(&dto.building_id)
            .map_err(|_| "Invalid building ID format".to_string())?;

        let unit = Unit::new(
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

    pub async fn list_units_by_building(&self, building_id: Uuid) -> Result<Vec<UnitResponseDto>, String> {
        let units = self.repository.find_by_building(building_id).await?;
        Ok(units.iter().map(|u| self.to_response_dto(u)).collect())
    }

    pub async fn assign_owner(&self, unit_id: Uuid, owner_id: Uuid) -> Result<UnitResponseDto, String> {
        let mut unit = self
            .repository
            .find_by_id(unit_id)
            .await?
            .ok_or_else(|| "Unit not found".to_string())?;

        unit.assign_owner(owner_id);

        let updated = self.repository.update(&unit).await?;
        Ok(self.to_response_dto(&updated))
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
