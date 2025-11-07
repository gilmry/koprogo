use super::*;
use crate::application::ports::{OwnerRepository, UnitOwnerRepository, UnitRepository};
use crate::domain::entities::unit::UnitType;
use crate::domain::entities::{Owner, Unit, UnitOwner};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

// Mock UnitOwnerRepository
#[derive(Clone)]
struct MockUnitOwnerRepository {
    unit_owners: Arc<Mutex<HashMap<Uuid, UnitOwner>>>,
}

impl MockUnitOwnerRepository {
    fn new() -> Self {
        Self {
            unit_owners: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl UnitOwnerRepository for MockUnitOwnerRepository {
    async fn create(&self, unit_owner: &UnitOwner) -> Result<UnitOwner, String> {
        let mut store = self.unit_owners.lock().unwrap();
        store.insert(unit_owner.id, unit_owner.clone());
        Ok(unit_owner.clone())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<UnitOwner>, String> {
        let store = self.unit_owners.lock().unwrap();
        Ok(store.get(&id).cloned())
    }

    async fn find_current_owners_by_unit(&self, unit_id: Uuid) -> Result<Vec<UnitOwner>, String> {
        let store = self.unit_owners.lock().unwrap();
        let owners: Vec<UnitOwner> = store
            .values()
            .filter(|uo| uo.unit_id == unit_id && uo.end_date.is_none())
            .cloned()
            .collect();
        Ok(owners)
    }

    async fn find_current_units_by_owner(&self, owner_id: Uuid) -> Result<Vec<UnitOwner>, String> {
        let store = self.unit_owners.lock().unwrap();
        let units: Vec<UnitOwner> = store
            .values()
            .filter(|uo| uo.owner_id == owner_id && uo.end_date.is_none())
            .cloned()
            .collect();
        Ok(units)
    }

    async fn find_all_owners_by_unit(&self, unit_id: Uuid) -> Result<Vec<UnitOwner>, String> {
        let store = self.unit_owners.lock().unwrap();
        let owners: Vec<UnitOwner> = store
            .values()
            .filter(|uo| uo.unit_id == unit_id)
            .cloned()
            .collect();
        Ok(owners)
    }

    async fn find_all_units_by_owner(&self, owner_id: Uuid) -> Result<Vec<UnitOwner>, String> {
        let store = self.unit_owners.lock().unwrap();
        let units: Vec<UnitOwner> = store
            .values()
            .filter(|uo| uo.owner_id == owner_id)
            .cloned()
            .collect();
        Ok(units)
    }

    async fn update(&self, unit_owner: &UnitOwner) -> Result<UnitOwner, String> {
        let mut store = self.unit_owners.lock().unwrap();
        store.insert(unit_owner.id, unit_owner.clone());
        Ok(unit_owner.clone())
    }

    async fn delete(&self, id: Uuid) -> Result<(), String> {
        let mut store = self.unit_owners.lock().unwrap();
        store.remove(&id);
        Ok(())
    }

    async fn has_active_owners(&self, unit_id: Uuid) -> Result<bool, String> {
        let owners = self.find_current_owners_by_unit(unit_id).await?;
        Ok(!owners.is_empty())
    }

    async fn get_total_ownership_percentage(&self, unit_id: Uuid) -> Result<f64, String> {
        let owners = self.find_current_owners_by_unit(unit_id).await?;
        let total: f64 = owners.iter().map(|o| o.ownership_percentage).sum();
        Ok(total)
    }

    async fn find_active_by_unit_and_owner(
        &self,
        unit_id: Uuid,
        owner_id: Uuid,
    ) -> Result<Option<UnitOwner>, String> {
        let store = self.unit_owners.lock().unwrap();
        Ok(store
            .values()
            .find(|uo| uo.unit_id == unit_id && uo.owner_id == owner_id && uo.end_date.is_none())
            .cloned())
    }

    async fn find_active_by_building(
        &self,
        _building_id: Uuid,
    ) -> Result<Vec<(Uuid, Uuid, f64)>, String> {
        Ok(vec![])
    }
}

// Mock UnitRepository
#[derive(Clone)]
struct MockUnitRepository {
    units: Arc<Mutex<HashMap<Uuid, Unit>>>,
}

impl MockUnitRepository {
    fn new() -> Self {
        Self {
            units: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    fn add_unit(&self, unit: Unit) {
        let mut store = self.units.lock().unwrap();
        store.insert(unit.id, unit);
    }
}

#[async_trait]
impl UnitRepository for MockUnitRepository {
    async fn create(&self, _unit: &Unit) -> Result<Unit, String> {
        unimplemented!()
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Unit>, String> {
        let store = self.units.lock().unwrap();
        Ok(store.get(&id).cloned())
    }

    async fn find_by_building(&self, _building_id: Uuid) -> Result<Vec<Unit>, String> {
        unimplemented!()
    }

    async fn find_by_owner(&self, _owner_id: Uuid) -> Result<Vec<Unit>, String> {
        unimplemented!()
    }

    async fn find_all_paginated(
        &self,
        _page_request: &crate::application::dto::PageRequest,
        _filters: &crate::application::dto::UnitFilters,
    ) -> Result<(Vec<Unit>, i64), String> {
        unimplemented!()
    }

    async fn update(&self, _unit: &Unit) -> Result<Unit, String> {
        unimplemented!()
    }

    async fn delete(&self, _id: Uuid) -> Result<bool, String> {
        unimplemented!()
    }
}

// Mock OwnerRepository
#[derive(Clone)]
struct MockOwnerRepository {
    owners: Arc<Mutex<HashMap<Uuid, Owner>>>,
}

impl MockOwnerRepository {
    fn new() -> Self {
        Self {
            owners: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    fn add_owner(&self, owner: Owner) {
        let mut store = self.owners.lock().unwrap();
        store.insert(owner.id, owner);
    }
}

#[async_trait]
impl OwnerRepository for MockOwnerRepository {
    async fn create(&self, _owner: &Owner) -> Result<Owner, String> {
        unimplemented!()
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Owner>, String> {
        let store = self.owners.lock().unwrap();
        Ok(store.get(&id).cloned())
    }

    async fn find_by_email(&self, _email: &str) -> Result<Option<Owner>, String> {
        unimplemented!()
    }

    async fn find_all(&self) -> Result<Vec<Owner>, String> {
        unimplemented!()
    }

    async fn find_all_paginated(
        &self,
        _page_request: &crate::application::dto::PageRequest,
        _filters: &crate::application::dto::OwnerFilters,
    ) -> Result<(Vec<Owner>, i64), String> {
        unimplemented!()
    }

    async fn update(&self, _owner: &Owner) -> Result<Owner, String> {
        unimplemented!()
    }

    async fn delete(&self, _id: Uuid) -> Result<bool, String> {
        unimplemented!()
    }
}

// Helper to create test data
fn create_test_unit(org_id: Uuid, building_id: Uuid) -> Unit {
    Unit::new(
        org_id,
        building_id,
        "A101".to_string(),
        UnitType::Apartment,
        Some(1),
        75.5,
        100.0,
    )
    .unwrap()
}

fn create_test_owner(org_id: Uuid) -> Owner {
    Owner::new(
        org_id,
        "Jean".to_string(),
        "Dupont".to_string(),
        "jean.dupont@example.com".to_string(),
        Some("+32 123 456 789".to_string()),
        "Rue de la Loi 1".to_string(),
        "Brussels".to_string(),
        "1000".to_string(),
        "Belgium".to_string(),
    )
    .unwrap()
}

fn setup() -> (
    UnitOwnerUseCases,
    MockUnitOwnerRepository,
    MockUnitRepository,
    MockOwnerRepository,
) {
    let unit_owner_repo = MockUnitOwnerRepository::new();
    let unit_repo = MockUnitRepository::new();
    let owner_repo = MockOwnerRepository::new();

    let use_cases = UnitOwnerUseCases::new(
        Arc::new(unit_owner_repo.clone()),
        Arc::new(unit_repo.clone()),
        Arc::new(owner_repo.clone()),
    );

    (use_cases, unit_owner_repo, unit_repo, owner_repo)
}

// TESTS: add_owner_to_unit

#[tokio::test]
async fn test_add_owner_to_unit_success() {
    let (use_cases, _uo_repo, unit_repo, owner_repo) = setup();

    let org_id = Uuid::new_v4();
    let building_id = Uuid::new_v4();
    let unit = create_test_unit(org_id, building_id);
    let owner = create_test_owner(org_id);

    unit_repo.add_unit(unit.clone());
    owner_repo.add_owner(owner.clone());

    let result = use_cases
        .add_owner_to_unit(unit.id, owner.id, 1.0, true)
        .await;

    assert!(result.is_ok());
    let unit_owner = result.unwrap();
    assert_eq!(unit_owner.unit_id, unit.id);
    assert_eq!(unit_owner.owner_id, owner.id);
    assert_eq!(unit_owner.ownership_percentage, 1.0);
    assert!(unit_owner.is_primary_contact);
}

#[tokio::test]
async fn test_add_owner_to_unit_nonexistent_unit() {
    let (use_cases, _uo_repo, _unit_repo, owner_repo) = setup();

    let org_id = Uuid::new_v4();
    let owner = create_test_owner(org_id);
    owner_repo.add_owner(owner.clone());

    let fake_unit_id = Uuid::new_v4();
    let result = use_cases
        .add_owner_to_unit(fake_unit_id, owner.id, 1.0, true)
        .await;

    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Unit not found");
}

#[tokio::test]
async fn test_add_owner_to_unit_nonexistent_owner() {
    let (use_cases, _uo_repo, unit_repo, _owner_repo) = setup();

    let org_id = Uuid::new_v4();
    let building_id = Uuid::new_v4();
    let unit = create_test_unit(org_id, building_id);
    unit_repo.add_unit(unit.clone());

    let fake_owner_id = Uuid::new_v4();
    let result = use_cases
        .add_owner_to_unit(unit.id, fake_owner_id, 1.0, true)
        .await;

    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Owner not found");
}

#[tokio::test]
async fn test_add_owner_to_unit_exceeds_100_percent() {
    let (use_cases, _uo_repo, unit_repo, owner_repo) = setup();

    let org_id = Uuid::new_v4();
    let building_id = Uuid::new_v4();
    let unit = create_test_unit(org_id, building_id);
    let owner1 = create_test_owner(org_id);
    let mut owner2 = create_test_owner(org_id);
    owner2.id = Uuid::new_v4();
    owner2.email = "marie.martin@example.com".to_string();

    unit_repo.add_unit(unit.clone());
    owner_repo.add_owner(owner1.clone());
    owner_repo.add_owner(owner2.clone());

    // Add first owner with 70%
    use_cases
        .add_owner_to_unit(unit.id, owner1.id, 0.7, true)
        .await
        .unwrap();

    // Try to add second owner with 50% (total would be 120%)
    let result = use_cases
        .add_owner_to_unit(unit.id, owner2.id, 0.5, false)
        .await;

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("exceed 100%"));
}

#[tokio::test]
async fn test_add_owner_to_unit_already_active() {
    let (use_cases, _uo_repo, unit_repo, owner_repo) = setup();

    let org_id = Uuid::new_v4();
    let building_id = Uuid::new_v4();
    let unit = create_test_unit(org_id, building_id);
    let owner = create_test_owner(org_id);

    unit_repo.add_unit(unit.clone());
    owner_repo.add_owner(owner.clone());

    // Add owner first time
    use_cases
        .add_owner_to_unit(unit.id, owner.id, 1.0, true)
        .await
        .unwrap();

    // Try to add same owner again
    let result = use_cases
        .add_owner_to_unit(unit.id, owner.id, 0.5, false)
        .await;

    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Owner is already active on this unit");
}

#[tokio::test]
async fn test_add_multiple_owners_to_unit() {
    let (use_cases, _uo_repo, unit_repo, owner_repo) = setup();

    let org_id = Uuid::new_v4();
    let building_id = Uuid::new_v4();
    let unit = create_test_unit(org_id, building_id);

    let owner1 = create_test_owner(org_id);
    let mut owner2 = create_test_owner(org_id);
    owner2.id = Uuid::new_v4();
    owner2.email = "marie@example.com".to_string();
    let mut owner3 = create_test_owner(org_id);
    owner3.id = Uuid::new_v4();
    owner3.email = "pierre@example.com".to_string();

    unit_repo.add_unit(unit.clone());
    owner_repo.add_owner(owner1.clone());
    owner_repo.add_owner(owner2.clone());
    owner_repo.add_owner(owner3.clone());

    // Add 3 owners: 50%, 30%, 20%
    let uo1 = use_cases
        .add_owner_to_unit(unit.id, owner1.id, 0.5, true)
        .await
        .unwrap();
    let uo2 = use_cases
        .add_owner_to_unit(unit.id, owner2.id, 0.3, false)
        .await
        .unwrap();
    let uo3 = use_cases
        .add_owner_to_unit(unit.id, owner3.id, 0.2, false)
        .await
        .unwrap();

    assert_eq!(uo1.ownership_percentage, 0.5);
    assert_eq!(uo2.ownership_percentage, 0.3);
    assert_eq!(uo3.ownership_percentage, 0.2);

    // Verify total
    let total = use_cases
        .get_total_ownership_percentage(unit.id)
        .await
        .unwrap();
    assert!((total - 1.0).abs() < 0.0001);
}

// TESTS: remove_owner_from_unit

#[tokio::test]
async fn test_remove_owner_from_unit_success() {
    let (use_cases, _uo_repo, unit_repo, owner_repo) = setup();

    let org_id = Uuid::new_v4();
    let building_id = Uuid::new_v4();
    let unit = create_test_unit(org_id, building_id);
    let owner = create_test_owner(org_id);

    unit_repo.add_unit(unit.clone());
    owner_repo.add_owner(owner.clone());

    // Add owner
    use_cases
        .add_owner_to_unit(unit.id, owner.id, 1.0, true)
        .await
        .unwrap();

    // Remove owner
    let result = use_cases.remove_owner_from_unit(unit.id, owner.id).await;

    assert!(result.is_ok());
    let ended = result.unwrap();
    assert!(!ended.is_active());
    assert!(ended.end_date.is_some());
}

#[tokio::test]
async fn test_remove_nonexistent_relationship() {
    let (use_cases, _uo_repo, unit_repo, owner_repo) = setup();

    let org_id = Uuid::new_v4();
    let building_id = Uuid::new_v4();
    let unit = create_test_unit(org_id, building_id);
    let owner = create_test_owner(org_id);

    unit_repo.add_unit(unit.clone());
    owner_repo.add_owner(owner.clone());

    // Try to remove without adding first
    let result = use_cases.remove_owner_from_unit(unit.id, owner.id).await;

    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        "Active unit-owner relationship not found"
    );
}

// TESTS: update_ownership_percentage

#[tokio::test]
async fn test_update_ownership_percentage_success() {
    let (use_cases, _uo_repo, unit_repo, owner_repo) = setup();

    let org_id = Uuid::new_v4();
    let building_id = Uuid::new_v4();
    let unit = create_test_unit(org_id, building_id);
    let owner = create_test_owner(org_id);

    unit_repo.add_unit(unit.clone());
    owner_repo.add_owner(owner.clone());

    let uo = use_cases
        .add_owner_to_unit(unit.id, owner.id, 0.5, true)
        .await
        .unwrap();

    // Update percentage
    let result = use_cases.update_ownership_percentage(uo.id, 0.75).await;

    assert!(result.is_ok());
    let updated = result.unwrap();
    assert_eq!(updated.ownership_percentage, 0.75);
}

#[tokio::test]
async fn test_update_percentage_exceeds_limit() {
    let (use_cases, _uo_repo, unit_repo, owner_repo) = setup();

    let org_id = Uuid::new_v4();
    let building_id = Uuid::new_v4();
    let unit = create_test_unit(org_id, building_id);
    let owner1 = create_test_owner(org_id);
    let mut owner2 = create_test_owner(org_id);
    owner2.id = Uuid::new_v4();
    owner2.email = "owner2@example.com".to_string();

    unit_repo.add_unit(unit.clone());
    owner_repo.add_owner(owner1.clone());
    owner_repo.add_owner(owner2.clone());

    let uo1 = use_cases
        .add_owner_to_unit(unit.id, owner1.id, 0.6, true)
        .await
        .unwrap();
    use_cases
        .add_owner_to_unit(unit.id, owner2.id, 0.4, false)
        .await
        .unwrap();

    // Try to increase owner1 to 0.7 (would make total 1.1)
    let result = use_cases.update_ownership_percentage(uo1.id, 0.7).await;

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("exceed 100%"));
}

// TESTS: transfer_ownership

#[tokio::test]
async fn test_transfer_ownership_success() {
    let (use_cases, _uo_repo, unit_repo, owner_repo) = setup();

    let org_id = Uuid::new_v4();
    let building_id = Uuid::new_v4();
    let unit = create_test_unit(org_id, building_id);
    let from_owner = create_test_owner(org_id);
    let mut to_owner = create_test_owner(org_id);
    to_owner.id = Uuid::new_v4();
    to_owner.email = "buyer@example.com".to_string();

    unit_repo.add_unit(unit.clone());
    owner_repo.add_owner(from_owner.clone());
    owner_repo.add_owner(to_owner.clone());

    // Add initial owner
    use_cases
        .add_owner_to_unit(unit.id, from_owner.id, 1.0, true)
        .await
        .unwrap();

    // Transfer
    let result = use_cases
        .transfer_ownership(from_owner.id, to_owner.id, unit.id)
        .await;

    assert!(result.is_ok());
    let (ended, created) = result.unwrap();

    assert!(!ended.is_active());
    assert_eq!(ended.owner_id, from_owner.id);

    assert!(created.is_active());
    assert_eq!(created.owner_id, to_owner.id);
    assert_eq!(created.ownership_percentage, 1.0);
    assert_eq!(created.is_primary_contact, ended.is_primary_contact);
}

#[tokio::test]
async fn test_transfer_ownership_target_already_owns() {
    let (use_cases, _uo_repo, unit_repo, owner_repo) = setup();

    let org_id = Uuid::new_v4();
    let building_id = Uuid::new_v4();
    let unit = create_test_unit(org_id, building_id);
    let owner1 = create_test_owner(org_id);
    let mut owner2 = create_test_owner(org_id);
    owner2.id = Uuid::new_v4();
    owner2.email = "owner2@example.com".to_string();

    unit_repo.add_unit(unit.clone());
    owner_repo.add_owner(owner1.clone());
    owner_repo.add_owner(owner2.clone());

    // Both owners already own parts
    use_cases
        .add_owner_to_unit(unit.id, owner1.id, 0.5, true)
        .await
        .unwrap();
    use_cases
        .add_owner_to_unit(unit.id, owner2.id, 0.5, false)
        .await
        .unwrap();

    // Try to transfer from owner1 to owner2
    let result = use_cases
        .transfer_ownership(owner1.id, owner2.id, unit.id)
        .await;

    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Target owner already owns this unit");
}

// TESTS: get_unit_owners & get_owner_units

#[tokio::test]
async fn test_get_unit_owners() {
    let (use_cases, _uo_repo, unit_repo, owner_repo) = setup();

    let org_id = Uuid::new_v4();
    let building_id = Uuid::new_v4();
    let unit = create_test_unit(org_id, building_id);
    let owner = create_test_owner(org_id);

    unit_repo.add_unit(unit.clone());
    owner_repo.add_owner(owner.clone());

    use_cases
        .add_owner_to_unit(unit.id, owner.id, 1.0, true)
        .await
        .unwrap();

    let result = use_cases.get_unit_owners(unit.id).await;

    assert!(result.is_ok());
    let owners = result.unwrap();
    assert_eq!(owners.len(), 1);
    assert_eq!(owners[0].owner_id, owner.id);
}

#[tokio::test]
async fn test_get_owner_units() {
    let (use_cases, _uo_repo, unit_repo, owner_repo) = setup();

    let org_id = Uuid::new_v4();
    let building_id = Uuid::new_v4();
    let unit1 = create_test_unit(org_id, building_id);
    let mut unit2 = create_test_unit(org_id, building_id);
    unit2.id = Uuid::new_v4();
    unit2.unit_number = "B202".to_string();

    let owner = create_test_owner(org_id);

    unit_repo.add_unit(unit1.clone());
    unit_repo.add_unit(unit2.clone());
    owner_repo.add_owner(owner.clone());

    use_cases
        .add_owner_to_unit(unit1.id, owner.id, 1.0, true)
        .await
        .unwrap();
    use_cases
        .add_owner_to_unit(unit2.id, owner.id, 1.0, true)
        .await
        .unwrap();

    let result = use_cases.get_owner_units(owner.id).await;

    assert!(result.is_ok());
    let units = result.unwrap();
    assert_eq!(units.len(), 2);
}

// TESTS: set_primary_contact

#[tokio::test]
async fn test_set_primary_contact() {
    let (use_cases, _uo_repo, unit_repo, owner_repo) = setup();

    let org_id = Uuid::new_v4();
    let building_id = Uuid::new_v4();
    let unit = create_test_unit(org_id, building_id);
    let owner1 = create_test_owner(org_id);
    let mut owner2 = create_test_owner(org_id);
    owner2.id = Uuid::new_v4();
    owner2.email = "owner2@example.com".to_string();

    unit_repo.add_unit(unit.clone());
    owner_repo.add_owner(owner1.clone());
    owner_repo.add_owner(owner2.clone());

    let uo1 = use_cases
        .add_owner_to_unit(unit.id, owner1.id, 0.5, true)
        .await
        .unwrap();
    let uo2 = use_cases
        .add_owner_to_unit(unit.id, owner2.id, 0.5, false)
        .await
        .unwrap();

    assert!(uo1.is_primary_contact);
    assert!(!uo2.is_primary_contact);

    // Set owner2 as primary
    let result = use_cases.set_primary_contact(uo2.id).await;

    assert!(result.is_ok());
    let updated = result.unwrap();
    assert!(updated.is_primary_contact);

    // Verify owner1 is no longer primary
    let uo1_updated = use_cases.get_unit_owner(uo1.id).await.unwrap().unwrap();
    assert!(!uo1_updated.is_primary_contact);
}

// TESTS: Edge cases

#[tokio::test]
async fn test_total_ownership_percentage() {
    let (use_cases, _uo_repo, unit_repo, owner_repo) = setup();

    let org_id = Uuid::new_v4();
    let building_id = Uuid::new_v4();
    let unit = create_test_unit(org_id, building_id);
    let owner = create_test_owner(org_id);

    unit_repo.add_unit(unit.clone());
    owner_repo.add_owner(owner.clone());

    // Initially 0%
    let total = use_cases
        .get_total_ownership_percentage(unit.id)
        .await
        .unwrap();
    assert_eq!(total, 0.0);

    // Add 60%
    use_cases
        .add_owner_to_unit(unit.id, owner.id, 0.6, true)
        .await
        .unwrap();

    let total = use_cases
        .get_total_ownership_percentage(unit.id)
        .await
        .unwrap();
    assert_eq!(total, 0.6);
}

#[tokio::test]
async fn test_has_active_owners() {
    let (use_cases, _uo_repo, unit_repo, owner_repo) = setup();

    let org_id = Uuid::new_v4();
    let building_id = Uuid::new_v4();
    let unit = create_test_unit(org_id, building_id);
    let owner = create_test_owner(org_id);

    unit_repo.add_unit(unit.clone());
    owner_repo.add_owner(owner.clone());

    // Initially no owners
    let has_owners = use_cases.has_active_owners(unit.id).await.unwrap();
    assert!(!has_owners);

    // Add owner
    use_cases
        .add_owner_to_unit(unit.id, owner.id, 1.0, true)
        .await
        .unwrap();

    let has_owners = use_cases.has_active_owners(unit.id).await.unwrap();
    assert!(has_owners);

    // Remove owner
    use_cases
        .remove_owner_from_unit(unit.id, owner.id)
        .await
        .unwrap();

    let has_owners = use_cases.has_active_owners(unit.id).await.unwrap();
    assert!(!has_owners);
}

#[tokio::test]
async fn test_ownership_history() {
    let (use_cases, _uo_repo, unit_repo, owner_repo) = setup();

    let org_id = Uuid::new_v4();
    let building_id = Uuid::new_v4();
    let unit = create_test_unit(org_id, building_id);
    let owner1 = create_test_owner(org_id);
    let mut owner2 = create_test_owner(org_id);
    owner2.id = Uuid::new_v4();
    owner2.email = "owner2@example.com".to_string();

    unit_repo.add_unit(unit.clone());
    owner_repo.add_owner(owner1.clone());
    owner_repo.add_owner(owner2.clone());

    // owner1 owns it first
    use_cases
        .add_owner_to_unit(unit.id, owner1.id, 1.0, true)
        .await
        .unwrap();

    // Transfer to owner2
    use_cases
        .transfer_ownership(owner1.id, owner2.id, unit.id)
        .await
        .unwrap();

    // Check history
    let history = use_cases.get_unit_ownership_history(unit.id).await.unwrap();

    assert_eq!(history.len(), 2);
    // One active, one ended
    let active_count = history.iter().filter(|uo| uo.is_active()).count();
    let ended_count = history.iter().filter(|uo| !uo.is_active()).count();

    assert_eq!(active_count, 1);
    assert_eq!(ended_count, 1);
}
