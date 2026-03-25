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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::UnitType;
    use async_trait::async_trait;
    use std::collections::HashMap;
    use std::sync::Mutex;

    struct MockUnitRepository {
        items: Mutex<HashMap<Uuid, Unit>>,
    }

    impl MockUnitRepository {
        fn new() -> Self {
            Self {
                items: Mutex::new(HashMap::new()),
            }
        }
    }

    #[async_trait]
    impl UnitRepository for MockUnitRepository {
        async fn create(&self, unit: &Unit) -> Result<Unit, String> {
            let mut items = self.items.lock().unwrap();
            items.insert(unit.id, unit.clone());
            Ok(unit.clone())
        }

        async fn find_by_id(&self, id: Uuid) -> Result<Option<Unit>, String> {
            let items = self.items.lock().unwrap();
            Ok(items.get(&id).cloned())
        }

        async fn find_by_building(&self, building_id: Uuid) -> Result<Vec<Unit>, String> {
            let items = self.items.lock().unwrap();
            Ok(items
                .values()
                .filter(|u| u.building_id == building_id)
                .cloned()
                .collect())
        }

        async fn find_by_owner(&self, owner_id: Uuid) -> Result<Vec<Unit>, String> {
            let items = self.items.lock().unwrap();
            Ok(items
                .values()
                .filter(|u| u.owner_id == Some(owner_id))
                .cloned()
                .collect())
        }

        async fn find_all_paginated(
            &self,
            page_request: &PageRequest,
            _filters: &UnitFilters,
        ) -> Result<(Vec<Unit>, i64), String> {
            let items = self.items.lock().unwrap();
            let all: Vec<Unit> = items.values().cloned().collect();
            let total = all.len() as i64;
            let offset = page_request.offset() as usize;
            let limit = page_request.limit() as usize;
            let page = all.into_iter().skip(offset).take(limit).collect();
            Ok((page, total))
        }

        async fn update(&self, unit: &Unit) -> Result<Unit, String> {
            let mut items = self.items.lock().unwrap();
            items.insert(unit.id, unit.clone());
            Ok(unit.clone())
        }

        async fn delete(&self, id: Uuid) -> Result<bool, String> {
            let mut items = self.items.lock().unwrap();
            Ok(items.remove(&id).is_some())
        }
    }

    fn make_use_cases(repo: MockUnitRepository) -> UnitUseCases {
        UnitUseCases::new(Arc::new(repo))
    }

    fn make_create_dto(org_id: Uuid, building_id: Uuid) -> CreateUnitDto {
        CreateUnitDto {
            organization_id: org_id.to_string(),
            building_id: building_id.to_string(),
            unit_number: "A101".to_string(),
            unit_type: UnitType::Apartment,
            floor: Some(1),
            surface_area: 85.0,
            quota: 50.0,
        }
    }

    #[tokio::test]
    async fn test_create_unit_success() {
        let repo = MockUnitRepository::new();
        let use_cases = make_use_cases(repo);
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();

        let result = use_cases
            .create_unit(make_create_dto(org_id, building_id))
            .await;

        assert!(result.is_ok());
        let dto = result.unwrap();
        assert_eq!(dto.unit_number, "A101");
        assert_eq!(dto.surface_area, 85.0);
        assert_eq!(dto.quota, 50.0);
        assert_eq!(dto.building_id, building_id.to_string());
        assert!(dto.owner_id.is_none());
    }

    #[tokio::test]
    async fn test_create_unit_invalid_building_id() {
        let repo = MockUnitRepository::new();
        let use_cases = make_use_cases(repo);
        let org_id = Uuid::new_v4();

        let dto = CreateUnitDto {
            organization_id: org_id.to_string(),
            building_id: "not-a-valid-uuid".to_string(),
            unit_number: "A101".to_string(),
            unit_type: UnitType::Apartment,
            floor: Some(1),
            surface_area: 85.0,
            quota: 50.0,
        };

        let result = use_cases.create_unit(dto).await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Invalid building ID format");
    }

    #[tokio::test]
    async fn test_get_unit() {
        let repo = MockUnitRepository::new();
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let unit = Unit::new(
            org_id,
            building_id,
            "B202".to_string(),
            UnitType::Parking,
            Some(-1),
            15.0,
            10.0,
        )
        .unwrap();
        let unit_id = unit.id;
        repo.items.lock().unwrap().insert(unit.id, unit);

        let use_cases = make_use_cases(repo);
        let result = use_cases.get_unit(unit_id).await;

        assert!(result.is_ok());
        let dto = result.unwrap();
        assert!(dto.is_some());
        let dto = dto.unwrap();
        assert_eq!(dto.unit_number, "B202");
        assert_eq!(dto.surface_area, 15.0);
    }

    #[tokio::test]
    async fn test_list_units_by_building() {
        let repo = MockUnitRepository::new();
        let org_id = Uuid::new_v4();
        let building_a = Uuid::new_v4();
        let building_b = Uuid::new_v4();

        let unit1 = Unit::new(
            org_id,
            building_a,
            "A101".to_string(),
            UnitType::Apartment,
            Some(1),
            80.0,
            40.0,
        )
        .unwrap();
        let unit2 = Unit::new(
            org_id,
            building_a,
            "A102".to_string(),
            UnitType::Apartment,
            Some(1),
            65.0,
            30.0,
        )
        .unwrap();
        let unit3 = Unit::new(
            org_id,
            building_b,
            "B101".to_string(),
            UnitType::Commercial,
            Some(0),
            120.0,
            100.0,
        )
        .unwrap();

        {
            let mut items = repo.items.lock().unwrap();
            items.insert(unit1.id, unit1);
            items.insert(unit2.id, unit2);
            items.insert(unit3.id, unit3);
        }

        let use_cases = make_use_cases(repo);
        let result = use_cases.list_units_by_building(building_a).await;

        assert!(result.is_ok());
        let units = result.unwrap();
        assert_eq!(units.len(), 2);
        assert!(units
            .iter()
            .all(|u| u.building_id == building_a.to_string()));
    }

    #[tokio::test]
    async fn test_delete_unit() {
        let repo = MockUnitRepository::new();
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let unit = Unit::new(
            org_id,
            building_id,
            "A101".to_string(),
            UnitType::Apartment,
            Some(1),
            80.0,
            50.0,
        )
        .unwrap();
        let unit_id = unit.id;
        repo.items.lock().unwrap().insert(unit.id, unit);

        let use_cases = make_use_cases(repo);
        let result = use_cases.delete_unit(unit_id).await;

        assert!(result.is_ok());
        assert!(result.unwrap());

        // Verify it is gone
        let get_result = use_cases.get_unit(unit_id).await;
        assert!(get_result.is_ok());
        assert!(get_result.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_assign_owner() {
        let repo = MockUnitRepository::new();
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let unit = Unit::new(
            org_id,
            building_id,
            "A101".to_string(),
            UnitType::Apartment,
            Some(1),
            80.0,
            50.0,
        )
        .unwrap();
        let unit_id = unit.id;
        repo.items.lock().unwrap().insert(unit.id, unit);

        let use_cases = make_use_cases(repo);
        let owner_id = Uuid::new_v4();
        let result = use_cases.assign_owner(unit_id, owner_id).await;

        assert!(result.is_ok());
        let dto = result.unwrap();
        assert_eq!(dto.owner_id, Some(owner_id.to_string()));
    }
}
