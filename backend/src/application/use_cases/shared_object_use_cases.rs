use crate::application::dto::{
    BorrowObjectDto, CategoryObjectCount, CreateSharedObjectDto, SharedObjectResponseDto,
    SharedObjectStatisticsDto, SharedObjectSummaryDto, UpdateSharedObjectDto,
};
use crate::application::ports::{OwnerRepository, SharedObjectRepository};
use crate::domain::entities::{SharedObject, SharedObjectCategory};
use std::sync::Arc;
use uuid::Uuid;

pub struct SharedObjectUseCases {
    shared_object_repo: Arc<dyn SharedObjectRepository>,
    owner_repo: Arc<dyn OwnerRepository>,
}

impl SharedObjectUseCases {
    pub fn new(
        shared_object_repo: Arc<dyn SharedObjectRepository>,
        owner_repo: Arc<dyn OwnerRepository>,
    ) -> Self {
        Self {
            shared_object_repo,
            owner_repo,
        }
    }

    /// Create a new shared object
    ///
    /// # Authorization
    /// - Owner must exist in the system
    pub async fn create_shared_object(
        &self,
        owner_id: Uuid,
        dto: CreateSharedObjectDto,
    ) -> Result<SharedObjectResponseDto, String> {
        // Verify owner exists
        let owner = self
            .owner_repo
            .find_by_id(owner_id)
            .await?
            .ok_or("Owner not found".to_string())?;

        // Create shared object entity (validates business rules)
        let object = SharedObject::new(
            owner_id,
            dto.building_id,
            dto.object_category,
            dto.object_name,
            dto.description,
            dto.condition,
            dto.is_available,
            dto.rental_credits_per_day,
            dto.deposit_credits,
            dto.borrowing_duration_days,
            dto.photos,
            dto.location_details,
            dto.usage_instructions,
        )?;

        // Persist object
        let created = self.shared_object_repo.create(&object).await?;

        // Return enriched response
        let owner_name = format!("{} {}", owner.first_name, owner.last_name);
        Ok(SharedObjectResponseDto::from_shared_object(
            created, owner_name, None,
        ))
    }

    /// Get shared object by ID with owner/borrower name enrichment
    pub async fn get_shared_object(
        &self,
        object_id: Uuid,
    ) -> Result<SharedObjectResponseDto, String> {
        let object = self
            .shared_object_repo
            .find_by_id(object_id)
            .await?
            .ok_or("Shared object not found".to_string())?;

        // Enrich with owner name
        let owner = self
            .owner_repo
            .find_by_id(object.owner_id)
            .await?
            .ok_or("Owner not found".to_string())?;
        let owner_name = format!("{} {}", owner.first_name, owner.last_name);

        // Enrich with borrower name if borrowed
        let borrower_name = if let Some(borrower_id) = object.current_borrower_id {
            let borrower = self.owner_repo.find_by_id(borrower_id).await?;
            borrower.map(|b| format!("{} {}", b.first_name, b.last_name))
        } else {
            None
        };

        Ok(SharedObjectResponseDto::from_shared_object(
            object,
            owner_name,
            borrower_name,
        ))
    }

    /// List all shared objects for a building
    ///
    /// # Returns
    /// - Objects sorted by available (DESC), object_name (ASC)
    pub async fn list_building_objects(
        &self,
        building_id: Uuid,
    ) -> Result<Vec<SharedObjectSummaryDto>, String> {
        let objects = self
            .shared_object_repo
            .find_by_building(building_id)
            .await?;
        self.enrich_objects_summary(objects).await
    }

    /// List available shared objects for a building (marketplace view)
    ///
    /// # Returns
    /// - Only available objects (is_available = true)
    pub async fn list_available_objects(
        &self,
        building_id: Uuid,
    ) -> Result<Vec<SharedObjectSummaryDto>, String> {
        let objects = self
            .shared_object_repo
            .find_available_by_building(building_id)
            .await?;
        self.enrich_objects_summary(objects).await
    }

    /// List borrowed shared objects for a building
    pub async fn list_borrowed_objects(
        &self,
        building_id: Uuid,
    ) -> Result<Vec<SharedObjectSummaryDto>, String> {
        let objects = self
            .shared_object_repo
            .find_borrowed_by_building(building_id)
            .await?;
        self.enrich_objects_summary(objects).await
    }

    /// List overdue shared objects for a building
    pub async fn list_overdue_objects(
        &self,
        building_id: Uuid,
    ) -> Result<Vec<SharedObjectSummaryDto>, String> {
        let objects = self
            .shared_object_repo
            .find_overdue_by_building(building_id)
            .await?;
        self.enrich_objects_summary(objects).await
    }

    /// List all shared objects created by an owner
    pub async fn list_owner_objects(
        &self,
        owner_id: Uuid,
    ) -> Result<Vec<SharedObjectSummaryDto>, String> {
        let objects = self.shared_object_repo.find_by_owner(owner_id).await?;
        self.enrich_objects_summary(objects).await
    }

    /// List all shared objects currently borrowed by a user
    pub async fn list_user_borrowed_objects(
        &self,
        borrower_id: Uuid,
    ) -> Result<Vec<SharedObjectSummaryDto>, String> {
        let objects = self
            .shared_object_repo
            .find_borrowed_by_user(borrower_id)
            .await?;
        self.enrich_objects_summary(objects).await
    }

    /// List shared objects by category (Tools, Books, Electronics, etc.)
    pub async fn list_objects_by_category(
        &self,
        building_id: Uuid,
        category: SharedObjectCategory,
    ) -> Result<Vec<SharedObjectSummaryDto>, String> {
        let objects = self
            .shared_object_repo
            .find_by_category(building_id, category)
            .await?;
        self.enrich_objects_summary(objects).await
    }

    /// List free/volunteer shared objects for a building
    pub async fn list_free_objects(
        &self,
        building_id: Uuid,
    ) -> Result<Vec<SharedObjectSummaryDto>, String> {
        let objects = self
            .shared_object_repo
            .find_free_by_building(building_id)
            .await?;
        self.enrich_objects_summary(objects).await
    }

    /// Update a shared object
    ///
    /// # Authorization
    /// - Only owner can update their object
    /// - Cannot update if currently borrowed
    pub async fn update_shared_object(
        &self,
        object_id: Uuid,
        actor_id: Uuid,
        dto: UpdateSharedObjectDto,
    ) -> Result<SharedObjectResponseDto, String> {
        let mut object = self
            .shared_object_repo
            .find_by_id(object_id)
            .await?
            .ok_or("Shared object not found".to_string())?;

        // Authorization: only owner can update
        if object.owner_id != actor_id {
            return Err("Unauthorized: only owner can update object".to_string());
        }

        // Update object (domain validates business rules including borrowed check)
        object.update(
            dto.object_name,
            dto.description,
            dto.condition,
            dto.is_available,
            dto.rental_credits_per_day,
            dto.deposit_credits,
            dto.borrowing_duration_days,
            dto.photos,
            dto.location_details,
            dto.usage_instructions,
        )?;

        // Persist changes
        let updated = self.shared_object_repo.update(&object).await?;

        // Return enriched response
        self.get_shared_object(updated.id).await
    }

    /// Mark shared object as available
    ///
    /// # Authorization
    /// - Only owner can mark their object as available
    pub async fn mark_object_available(
        &self,
        object_id: Uuid,
        actor_id: Uuid,
    ) -> Result<SharedObjectResponseDto, String> {
        let mut object = self
            .shared_object_repo
            .find_by_id(object_id)
            .await?
            .ok_or("Shared object not found".to_string())?;

        // Authorization: only owner can mark available
        if object.owner_id != actor_id {
            return Err("Unauthorized: only owner can mark object as available".to_string());
        }

        // Mark available (checks not borrowed)
        object.mark_available()?;

        // Persist changes
        let updated = self.shared_object_repo.update(&object).await?;

        // Return enriched response
        self.get_shared_object(updated.id).await
    }

    /// Mark shared object as unavailable
    ///
    /// # Authorization
    /// - Only owner can mark their object as unavailable
    pub async fn mark_object_unavailable(
        &self,
        object_id: Uuid,
        actor_id: Uuid,
    ) -> Result<SharedObjectResponseDto, String> {
        let mut object = self
            .shared_object_repo
            .find_by_id(object_id)
            .await?
            .ok_or("Shared object not found".to_string())?;

        // Authorization: only owner can mark unavailable
        if object.owner_id != actor_id {
            return Err("Unauthorized: only owner can mark object as unavailable".to_string());
        }

        // Mark unavailable
        object.mark_unavailable();

        // Persist changes
        let updated = self.shared_object_repo.update(&object).await?;

        // Return enriched response
        self.get_shared_object(updated.id).await
    }

    /// Borrow a shared object
    ///
    /// # Authorization
    /// - Borrower must not be the owner
    /// - Object must be available
    ///
    /// # SEL Integration
    /// - For paid objects, this would integrate with OwnerCreditBalanceRepository
    /// - Deposit is held, rental fee calculated on return
    pub async fn borrow_object(
        &self,
        object_id: Uuid,
        borrower_id: Uuid,
        dto: BorrowObjectDto,
    ) -> Result<SharedObjectResponseDto, String> {
        let mut object = self
            .shared_object_repo
            .find_by_id(object_id)
            .await?
            .ok_or("Shared object not found".to_string())?;

        // Borrow object (validates business rules: owner != borrower, is_available, etc.)
        object.borrow(borrower_id, dto.duration_days)?;

        // TODO: For paid objects (rental_credits_per_day > 0), integrate with SEL:
        // - Hold deposit_credits from borrower's credit balance
        // - This would require OwnerCreditBalanceRepository

        // Persist changes
        let updated = self.shared_object_repo.update(&object).await?;

        // Return enriched response
        self.get_shared_object(updated.id).await
    }

    /// Return a borrowed object
    ///
    /// # Authorization
    /// - Only borrower can return object
    ///
    /// # SEL Integration
    /// - For paid objects, calculates rental fee based on actual days
    /// - Transfers credits from borrower to owner
    /// - Refunds deposit
    pub async fn return_object(
        &self,
        object_id: Uuid,
        returner_id: Uuid,
    ) -> Result<SharedObjectResponseDto, String> {
        let mut object = self
            .shared_object_repo
            .find_by_id(object_id)
            .await?
            .ok_or("Shared object not found".to_string())?;

        // TODO: For paid objects, calculate final rental cost and transfer SEL credits
        // let (rental_cost, deposit) = object.calculate_total_cost();
        // This would require OwnerCreditBalanceRepository

        // Return object (validates only borrower can return)
        object.return_object(returner_id)?;

        // Persist changes
        let updated = self.shared_object_repo.update(&object).await?;

        // Return enriched response
        self.get_shared_object(updated.id).await
    }

    /// Delete a shared object
    ///
    /// # Authorization
    /// - Only owner can delete their object
    /// - Cannot delete if currently borrowed
    pub async fn delete_shared_object(
        &self,
        object_id: Uuid,
        actor_id: Uuid,
    ) -> Result<(), String> {
        let object = self
            .shared_object_repo
            .find_by_id(object_id)
            .await?
            .ok_or("Shared object not found".to_string())?;

        // Authorization: only owner can delete
        if object.owner_id != actor_id {
            return Err("Unauthorized: only owner can delete object".to_string());
        }

        // Business rule: cannot delete if borrowed
        if object.is_borrowed() {
            return Err("Cannot delete object while it is borrowed".to_string());
        }

        // Delete object
        self.shared_object_repo.delete(object_id).await?;

        Ok(())
    }

    /// Get shared object statistics for a building
    pub async fn get_object_statistics(
        &self,
        building_id: Uuid,
    ) -> Result<SharedObjectStatisticsDto, String> {
        let total_objects = self
            .shared_object_repo
            .count_by_building(building_id)
            .await?;
        let available_objects = self
            .shared_object_repo
            .count_available_by_building(building_id)
            .await?;
        let borrowed_objects = self
            .shared_object_repo
            .count_borrowed_by_building(building_id)
            .await?;
        let overdue_objects = self
            .shared_object_repo
            .count_overdue_by_building(building_id)
            .await?;

        // Calculate free/paid objects
        let objects = self
            .shared_object_repo
            .find_by_building(building_id)
            .await?;
        let free_objects = objects.iter().filter(|o| o.is_free()).count() as i64;
        let paid_objects = total_objects - free_objects;

        // Count by category
        let mut objects_by_category = Vec::new();
        for category in [
            SharedObjectCategory::Tools,
            SharedObjectCategory::Books,
            SharedObjectCategory::Electronics,
            SharedObjectCategory::Sports,
            SharedObjectCategory::Gardening,
            SharedObjectCategory::Kitchen,
            SharedObjectCategory::Baby,
            SharedObjectCategory::Other,
        ] {
            let count = self
                .shared_object_repo
                .count_by_category(building_id, category.clone())
                .await?;
            if count > 0 {
                objects_by_category.push(CategoryObjectCount { category, count });
            }
        }

        Ok(SharedObjectStatisticsDto {
            total_objects,
            available_objects,
            borrowed_objects,
            overdue_objects,
            free_objects,
            paid_objects,
            objects_by_category,
        })
    }

    /// Helper method to enrich objects with owner names
    async fn enrich_objects_summary(
        &self,
        objects: Vec<SharedObject>,
    ) -> Result<Vec<SharedObjectSummaryDto>, String> {
        let mut enriched = Vec::new();

        for object in objects {
            // Get owner name
            let owner = self.owner_repo.find_by_id(object.owner_id).await?;
            let owner_name = if let Some(owner) = owner {
                format!("{} {}", owner.first_name, owner.last_name)
            } else {
                "Unknown Owner".to_string()
            };

            enriched.push(SharedObjectSummaryDto::from_shared_object(
                object, owner_name,
            ));
        }

        Ok(enriched)
    }
}
