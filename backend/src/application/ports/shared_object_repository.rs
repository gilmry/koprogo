use crate::domain::entities::{SharedObject, SharedObjectCategory};
use async_trait::async_trait;
use uuid::Uuid;

/// Repository port for SharedObject aggregate
#[async_trait]
pub trait SharedObjectRepository: Send + Sync {
    /// Create a new shared object
    async fn create(&self, object: &SharedObject) -> Result<SharedObject, String>;

    /// Find shared object by ID
    async fn find_by_id(&self, id: Uuid) -> Result<Option<SharedObject>, String>;

    /// Find all shared objects for a building
    async fn find_by_building(&self, building_id: Uuid) -> Result<Vec<SharedObject>, String>;

    /// Find all available shared objects for a building (not currently borrowed)
    async fn find_available_by_building(
        &self,
        building_id: Uuid,
    ) -> Result<Vec<SharedObject>, String>;

    /// Find all borrowed shared objects for a building
    async fn find_borrowed_by_building(
        &self,
        building_id: Uuid,
    ) -> Result<Vec<SharedObject>, String>;

    /// Find all overdue shared objects for a building
    async fn find_overdue_by_building(&self, building_id: Uuid)
        -> Result<Vec<SharedObject>, String>;

    /// Find all shared objects by owner
    async fn find_by_owner(&self, owner_id: Uuid) -> Result<Vec<SharedObject>, String>;

    /// Find all shared objects borrowed by a specific user
    async fn find_borrowed_by_user(&self, borrower_id: Uuid)
        -> Result<Vec<SharedObject>, String>;

    /// Find shared objects by category (Tools, Books, Electronics, etc.)
    async fn find_by_category(
        &self,
        building_id: Uuid,
        category: SharedObjectCategory,
    ) -> Result<Vec<SharedObject>, String>;

    /// Find free/volunteer shared objects for a building (rental_credits_per_day IS NULL OR = 0)
    async fn find_free_by_building(&self, building_id: Uuid)
        -> Result<Vec<SharedObject>, String>;

    /// Update an existing shared object
    async fn update(&self, object: &SharedObject) -> Result<SharedObject, String>;

    /// Delete a shared object
    async fn delete(&self, id: Uuid) -> Result<(), String>;

    /// Count total shared objects for a building
    async fn count_by_building(&self, building_id: Uuid) -> Result<i64, String>;

    /// Count available shared objects for a building
    async fn count_available_by_building(&self, building_id: Uuid) -> Result<i64, String>;

    /// Count borrowed shared objects for a building
    async fn count_borrowed_by_building(&self, building_id: Uuid) -> Result<i64, String>;

    /// Count overdue shared objects for a building
    async fn count_overdue_by_building(&self, building_id: Uuid) -> Result<i64, String>;

    /// Count shared objects by category (for statistics)
    async fn count_by_category(
        &self,
        building_id: Uuid,
        category: SharedObjectCategory,
    ) -> Result<i64, String>;
}
