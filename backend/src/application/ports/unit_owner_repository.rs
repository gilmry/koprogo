use crate::domain::entities::UnitOwner;
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait UnitOwnerRepository: Send + Sync {
    /// Create a new unit-owner relationship
    async fn create(&self, unit_owner: &UnitOwner) -> Result<UnitOwner, String>;

    /// Find a unit-owner relationship by ID
    async fn find_by_id(&self, id: Uuid) -> Result<Option<UnitOwner>, String>;

    /// Get all current owners of a unit (end_date IS NULL)
    async fn find_current_owners_by_unit(&self, unit_id: Uuid) -> Result<Vec<UnitOwner>, String>;

    /// Get all current units of an owner (end_date IS NULL)
    async fn find_current_units_by_owner(&self, owner_id: Uuid) -> Result<Vec<UnitOwner>, String>;

    /// Get ownership history of a unit (including past owners)
    async fn find_all_owners_by_unit(&self, unit_id: Uuid) -> Result<Vec<UnitOwner>, String>;

    /// Get ownership history of an owner (including past units)
    async fn find_all_units_by_owner(&self, owner_id: Uuid) -> Result<Vec<UnitOwner>, String>;

    /// Update a unit-owner relationship
    async fn update(&self, unit_owner: &UnitOwner) -> Result<UnitOwner, String>;

    /// Delete a unit-owner relationship
    async fn delete(&self, id: Uuid) -> Result<(), String>;

    /// Check if a unit has any active owners
    async fn has_active_owners(&self, unit_id: Uuid) -> Result<bool, String>;

    /// Get the total ownership percentage for a unit (should be <= 1.0)
    async fn get_total_ownership_percentage(&self, unit_id: Uuid) -> Result<f64, String>;

    /// Find active unit-owner relationship by unit and owner IDs
    async fn find_active_by_unit_and_owner(
        &self,
        unit_id: Uuid,
        owner_id: Uuid,
    ) -> Result<Option<UnitOwner>, String>;

    /// Get all active unit-owner relationships for a building
    /// Returns tuples of (unit_id, owner_id, ownership_percentage)
    /// Useful for calculating charge distributions
    async fn find_active_by_building(
        &self,
        building_id: Uuid,
    ) -> Result<Vec<(Uuid, Uuid, f64)>, String>;
}
