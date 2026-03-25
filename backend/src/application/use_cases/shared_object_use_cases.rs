use crate::application::dto::{
    BorrowObjectDto, CategoryObjectCount, CreateSharedObjectDto, SharedObjectResponseDto,
    SharedObjectStatisticsDto, SharedObjectSummaryDto, UpdateSharedObjectDto,
};
use crate::application::ports::{
    OwnerCreditBalanceRepository, OwnerRepository, SharedObjectRepository,
};
use crate::domain::entities::{SharedObject, SharedObjectCategory};
use std::sync::Arc;
use uuid::Uuid;

pub struct SharedObjectUseCases {
    shared_object_repo: Arc<dyn SharedObjectRepository>,
    owner_repo: Arc<dyn OwnerRepository>,
    credit_balance_repo: Arc<dyn OwnerCreditBalanceRepository>,
}

impl SharedObjectUseCases {
    pub fn new(
        shared_object_repo: Arc<dyn SharedObjectRepository>,
        owner_repo: Arc<dyn OwnerRepository>,
        credit_balance_repo: Arc<dyn OwnerCreditBalanceRepository>,
    ) -> Self {
        Self {
            shared_object_repo,
            owner_repo,
            credit_balance_repo,
        }
    }

    /// Resolve user_id to owner via organization lookup
    async fn resolve_owner(
        &self,
        user_id: Uuid,
        organization_id: Uuid,
    ) -> Result<crate::domain::entities::Owner, String> {
        self.owner_repo
            .find_by_user_id_and_organization(user_id, organization_id)
            .await?
            .ok_or_else(|| "Owner not found for this user in the organization".to_string())
    }

    /// Create a new shared object
    ///
    /// # Authorization
    /// - Must be an owner in the organization (owners lend to other owners/tenants)
    pub async fn create_shared_object(
        &self,
        user_id: Uuid,
        organization_id: Uuid,
        dto: CreateSharedObjectDto,
    ) -> Result<SharedObjectResponseDto, String> {
        let owner = self.resolve_owner(user_id, organization_id).await?;
        let owner_id = owner.id;

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

        let created = self.shared_object_repo.create(&object).await?;

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
        user_id: Uuid,
        organization_id: Uuid,
    ) -> Result<Vec<SharedObjectSummaryDto>, String> {
        let owner = self.resolve_owner(user_id, organization_id).await?;
        let objects = self
            .shared_object_repo
            .find_borrowed_by_user(owner.id)
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
        user_id: Uuid,
        organization_id: Uuid,
        dto: UpdateSharedObjectDto,
    ) -> Result<SharedObjectResponseDto, String> {
        let owner = self.resolve_owner(user_id, organization_id).await?;
        let mut object = self
            .shared_object_repo
            .find_by_id(object_id)
            .await?
            .ok_or("Shared object not found".to_string())?;

        // Authorization: only owner can update
        if object.owner_id != owner.id {
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
        user_id: Uuid,
        organization_id: Uuid,
    ) -> Result<SharedObjectResponseDto, String> {
        let owner = self.resolve_owner(user_id, organization_id).await?;
        let mut object = self
            .shared_object_repo
            .find_by_id(object_id)
            .await?
            .ok_or("Shared object not found".to_string())?;

        // Authorization: only owner can mark available
        if object.owner_id != owner.id {
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
        user_id: Uuid,
        organization_id: Uuid,
    ) -> Result<SharedObjectResponseDto, String> {
        let owner = self.resolve_owner(user_id, organization_id).await?;
        let mut object = self
            .shared_object_repo
            .find_by_id(object_id)
            .await?
            .ok_or("Shared object not found".to_string())?;

        // Authorization: only owner can mark unavailable
        if object.owner_id != owner.id {
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
    /// - For paid objects, holds deposit from borrower's credit balance
    /// - Rental fee calculated on return based on actual days borrowed
    pub async fn borrow_object(
        &self,
        object_id: Uuid,
        user_id: Uuid,
        organization_id: Uuid,
        dto: BorrowObjectDto,
    ) -> Result<SharedObjectResponseDto, String> {
        let borrower = self.resolve_owner(user_id, organization_id).await?;
        let borrower_id = borrower.id;
        let mut object = self
            .shared_object_repo
            .find_by_id(object_id)
            .await?
            .ok_or("Shared object not found".to_string())?;

        // SEL Integration: Hold deposit for paid objects
        if let Some(deposit_amount) = object.deposit_credits {
            if deposit_amount > 0 {
                // Get or create borrower's credit balance
                let mut borrower_balance = self
                    .credit_balance_repo
                    .get_or_create(borrower_id, object.building_id)
                    .await?;

                // Check if borrower has sufficient balance (allowing negative for trust-based)
                // But we still validate to warn if balance goes too negative
                let new_balance = borrower_balance.balance - deposit_amount;
                if new_balance < -100 {
                    // Trust limit: -100 credits
                    return Err(format!(
                        "Insufficient credit balance. Deposit required: {} credits. \
                        Your balance: {} credits. Trust limit: -100 credits.",
                        deposit_amount, borrower_balance.balance
                    ));
                }

                // Hold deposit (spend from borrower's balance)
                borrower_balance.spend_credits(deposit_amount)?;

                // Persist balance change
                self.credit_balance_repo.update(&borrower_balance).await?;
            }
        }

        // Borrow object (validates business rules: owner != borrower, is_available, etc.)
        object.borrow(borrower_id, dto.duration_days)?;

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
    /// - Transfers rental credits from borrower to owner
    /// - Refunds deposit to borrower
    pub async fn return_object(
        &self,
        object_id: Uuid,
        user_id: Uuid,
        organization_id: Uuid,
    ) -> Result<SharedObjectResponseDto, String> {
        let returner = self.resolve_owner(user_id, organization_id).await?;
        let returner_id = returner.id;
        let mut object = self
            .shared_object_repo
            .find_by_id(object_id)
            .await?
            .ok_or("Shared object not found".to_string())?;

        // SEL Integration: Calculate and transfer credits for paid objects
        let (rental_cost, deposit) = object.calculate_total_cost();

        if rental_cost > 0 || deposit > 0 {
            let borrower_id = object
                .current_borrower_id
                .ok_or("No current borrower to refund".to_string())?;
            let owner_id = object.owner_id;

            // Get or create borrower's credit balance
            let mut borrower_balance = self
                .credit_balance_repo
                .get_or_create(borrower_id, object.building_id)
                .await?;

            // Get or create owner's credit balance
            let mut owner_balance = self
                .credit_balance_repo
                .get_or_create(owner_id, object.building_id)
                .await?;

            // Transfer rental cost from borrower to owner
            if rental_cost > 0 {
                borrower_balance.spend_credits(rental_cost)?;
                owner_balance.earn_credits(rental_cost)?;
            }

            // Refund deposit to borrower
            if deposit > 0 {
                borrower_balance.earn_credits(deposit)?;
            }

            // Persist balance changes
            self.credit_balance_repo.update(&borrower_balance).await?;
            self.credit_balance_repo.update(&owner_balance).await?;
        }

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
        user_id: Uuid,
        organization_id: Uuid,
    ) -> Result<(), String> {
        let owner = self.resolve_owner(user_id, organization_id).await?;
        let object = self
            .shared_object_repo
            .find_by_id(object_id)
            .await?
            .ok_or("Shared object not found".to_string())?;

        // Authorization: only owner can delete
        if object.owner_id != owner.id {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::dto::{OwnerFilters, PageRequest};
    use crate::application::ports::{
        OwnerCreditBalanceRepository, OwnerRepository, SharedObjectRepository,
    };
    use crate::domain::entities::{
        ObjectCondition, Owner, OwnerCreditBalance, SharedObject, SharedObjectCategory,
    };
    use async_trait::async_trait;
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};
    use uuid::Uuid;

    // ── Mock SharedObjectRepository ─────────────────────────────────────────
    struct MockSharedObjectRepo {
        objects: Mutex<HashMap<Uuid, SharedObject>>,
    }

    impl MockSharedObjectRepo {
        fn new() -> Self {
            Self {
                objects: Mutex::new(HashMap::new()),
            }
        }
    }

    #[async_trait]
    impl SharedObjectRepository for MockSharedObjectRepo {
        async fn create(&self, object: &SharedObject) -> Result<SharedObject, String> {
            let mut map = self.objects.lock().unwrap();
            map.insert(object.id, object.clone());
            Ok(object.clone())
        }

        async fn find_by_id(&self, id: Uuid) -> Result<Option<SharedObject>, String> {
            let map = self.objects.lock().unwrap();
            Ok(map.get(&id).cloned())
        }

        async fn find_by_building(
            &self,
            building_id: Uuid,
        ) -> Result<Vec<SharedObject>, String> {
            let map = self.objects.lock().unwrap();
            Ok(map
                .values()
                .filter(|o| o.building_id == building_id)
                .cloned()
                .collect())
        }

        async fn find_available_by_building(
            &self,
            building_id: Uuid,
        ) -> Result<Vec<SharedObject>, String> {
            let map = self.objects.lock().unwrap();
            Ok(map
                .values()
                .filter(|o| o.building_id == building_id && o.is_available)
                .cloned()
                .collect())
        }

        async fn find_borrowed_by_building(
            &self,
            building_id: Uuid,
        ) -> Result<Vec<SharedObject>, String> {
            let map = self.objects.lock().unwrap();
            Ok(map
                .values()
                .filter(|o| o.building_id == building_id && o.is_borrowed())
                .cloned()
                .collect())
        }

        async fn find_overdue_by_building(
            &self,
            building_id: Uuid,
        ) -> Result<Vec<SharedObject>, String> {
            let map = self.objects.lock().unwrap();
            Ok(map
                .values()
                .filter(|o| o.building_id == building_id && o.is_overdue())
                .cloned()
                .collect())
        }

        async fn find_by_owner(&self, owner_id: Uuid) -> Result<Vec<SharedObject>, String> {
            let map = self.objects.lock().unwrap();
            Ok(map
                .values()
                .filter(|o| o.owner_id == owner_id)
                .cloned()
                .collect())
        }

        async fn find_borrowed_by_user(
            &self,
            borrower_id: Uuid,
        ) -> Result<Vec<SharedObject>, String> {
            let map = self.objects.lock().unwrap();
            Ok(map
                .values()
                .filter(|o| o.current_borrower_id == Some(borrower_id))
                .cloned()
                .collect())
        }

        async fn find_by_category(
            &self,
            building_id: Uuid,
            category: SharedObjectCategory,
        ) -> Result<Vec<SharedObject>, String> {
            let map = self.objects.lock().unwrap();
            Ok(map
                .values()
                .filter(|o| o.building_id == building_id && o.object_category == category)
                .cloned()
                .collect())
        }

        async fn find_free_by_building(
            &self,
            building_id: Uuid,
        ) -> Result<Vec<SharedObject>, String> {
            let map = self.objects.lock().unwrap();
            Ok(map
                .values()
                .filter(|o| o.building_id == building_id && o.is_free())
                .cloned()
                .collect())
        }

        async fn update(&self, object: &SharedObject) -> Result<SharedObject, String> {
            let mut map = self.objects.lock().unwrap();
            map.insert(object.id, object.clone());
            Ok(object.clone())
        }

        async fn delete(&self, id: Uuid) -> Result<(), String> {
            let mut map = self.objects.lock().unwrap();
            map.remove(&id);
            Ok(())
        }

        async fn count_by_building(&self, building_id: Uuid) -> Result<i64, String> {
            let map = self.objects.lock().unwrap();
            Ok(map
                .values()
                .filter(|o| o.building_id == building_id)
                .count() as i64)
        }

        async fn count_available_by_building(&self, building_id: Uuid) -> Result<i64, String> {
            let map = self.objects.lock().unwrap();
            Ok(map
                .values()
                .filter(|o| o.building_id == building_id && o.is_available)
                .count() as i64)
        }

        async fn count_borrowed_by_building(&self, building_id: Uuid) -> Result<i64, String> {
            let map = self.objects.lock().unwrap();
            Ok(map
                .values()
                .filter(|o| o.building_id == building_id && o.is_borrowed())
                .count() as i64)
        }

        async fn count_overdue_by_building(&self, building_id: Uuid) -> Result<i64, String> {
            let map = self.objects.lock().unwrap();
            Ok(map
                .values()
                .filter(|o| o.building_id == building_id && o.is_overdue())
                .count() as i64)
        }

        async fn count_by_category(
            &self,
            building_id: Uuid,
            category: SharedObjectCategory,
        ) -> Result<i64, String> {
            let map = self.objects.lock().unwrap();
            Ok(map
                .values()
                .filter(|o| o.building_id == building_id && o.object_category == category)
                .count() as i64)
        }
    }

    // ── Mock OwnerRepository ────────────────────────────────────────────────
    struct MockOwnerRepo {
        owners: Mutex<HashMap<Uuid, Owner>>,
    }

    impl MockOwnerRepo {
        fn new() -> Self {
            Self {
                owners: Mutex::new(HashMap::new()),
            }
        }
        fn add_owner(&self, owner: Owner) {
            self.owners.lock().unwrap().insert(owner.id, owner);
        }
    }

    #[async_trait]
    impl OwnerRepository for MockOwnerRepo {
        async fn create(&self, owner: &Owner) -> Result<Owner, String> {
            self.owners.lock().unwrap().insert(owner.id, owner.clone());
            Ok(owner.clone())
        }
        async fn find_by_id(&self, id: Uuid) -> Result<Option<Owner>, String> {
            Ok(self.owners.lock().unwrap().get(&id).cloned())
        }
        async fn find_by_user_id(&self, user_id: Uuid) -> Result<Option<Owner>, String> {
            Ok(self.owners.lock().unwrap().values().find(|o| o.user_id == Some(user_id)).cloned())
        }
        async fn find_by_user_id_and_organization(&self, user_id: Uuid, org_id: Uuid) -> Result<Option<Owner>, String> {
            Ok(self.owners.lock().unwrap().values().find(|o| o.user_id == Some(user_id) && o.organization_id == org_id).cloned())
        }
        async fn find_by_email(&self, email: &str) -> Result<Option<Owner>, String> {
            Ok(self.owners.lock().unwrap().values().find(|o| o.email == email).cloned())
        }
        async fn find_all(&self) -> Result<Vec<Owner>, String> {
            Ok(self.owners.lock().unwrap().values().cloned().collect())
        }
        async fn find_all_paginated(&self, _p: &PageRequest, _f: &OwnerFilters) -> Result<(Vec<Owner>, i64), String> {
            let all: Vec<_> = self.owners.lock().unwrap().values().cloned().collect();
            let c = all.len() as i64;
            Ok((all, c))
        }
        async fn update(&self, owner: &Owner) -> Result<Owner, String> {
            self.owners.lock().unwrap().insert(owner.id, owner.clone());
            Ok(owner.clone())
        }
        async fn delete(&self, id: Uuid) -> Result<bool, String> {
            Ok(self.owners.lock().unwrap().remove(&id).is_some())
        }
    }

    // ── Mock OwnerCreditBalanceRepository ────────────────────────────────────
    struct MockCreditBalanceRepo {
        balances: Mutex<HashMap<(Uuid, Uuid), OwnerCreditBalance>>,
    }

    impl MockCreditBalanceRepo {
        fn new() -> Self {
            Self {
                balances: Mutex::new(HashMap::new()),
            }
        }
    }

    #[async_trait]
    impl OwnerCreditBalanceRepository for MockCreditBalanceRepo {
        async fn create(&self, balance: &OwnerCreditBalance) -> Result<OwnerCreditBalance, String> {
            let mut map = self.balances.lock().unwrap();
            map.insert((balance.owner_id, balance.building_id), balance.clone());
            Ok(balance.clone())
        }
        async fn find_by_owner_and_building(&self, owner_id: Uuid, building_id: Uuid) -> Result<Option<OwnerCreditBalance>, String> {
            Ok(self.balances.lock().unwrap().get(&(owner_id, building_id)).cloned())
        }
        async fn find_by_building(&self, building_id: Uuid) -> Result<Vec<OwnerCreditBalance>, String> {
            Ok(self.balances.lock().unwrap().values().filter(|b| b.building_id == building_id).cloned().collect())
        }
        async fn find_by_owner(&self, owner_id: Uuid) -> Result<Vec<OwnerCreditBalance>, String> {
            Ok(self.balances.lock().unwrap().values().filter(|b| b.owner_id == owner_id).cloned().collect())
        }
        async fn get_or_create(&self, owner_id: Uuid, building_id: Uuid) -> Result<OwnerCreditBalance, String> {
            let mut map = self.balances.lock().unwrap();
            let key = (owner_id, building_id);
            if let Some(existing) = map.get(&key) {
                Ok(existing.clone())
            } else {
                let balance = OwnerCreditBalance::new(owner_id, building_id);
                map.insert(key, balance.clone());
                Ok(balance)
            }
        }
        async fn update(&self, balance: &OwnerCreditBalance) -> Result<OwnerCreditBalance, String> {
            let mut map = self.balances.lock().unwrap();
            map.insert((balance.owner_id, balance.building_id), balance.clone());
            Ok(balance.clone())
        }
        async fn delete(&self, owner_id: Uuid, building_id: Uuid) -> Result<bool, String> {
            Ok(self.balances.lock().unwrap().remove(&(owner_id, building_id)).is_some())
        }
        async fn get_leaderboard(&self, _building_id: Uuid, _limit: i32) -> Result<Vec<OwnerCreditBalance>, String> {
            Ok(vec![])
        }
        async fn count_active_participants(&self, _building_id: Uuid) -> Result<i64, String> {
            Ok(0)
        }
        async fn get_total_credits_in_circulation(&self, _building_id: Uuid) -> Result<i32, String> {
            Ok(0)
        }
    }

    // ── Helpers ─────────────────────────────────────────────────────────────
    fn create_test_owner(user_id: Uuid, org_id: Uuid) -> Owner {
        let mut owner = Owner::new(
            org_id, "Jean".to_string(), "Dupont".to_string(),
            "jean@test.com".to_string(), None,
            "Rue Test".to_string(), "Brussels".to_string(),
            "1000".to_string(), "Belgium".to_string(),
        ).unwrap();
        owner.user_id = Some(user_id);
        owner
    }

    fn setup() -> (SharedObjectUseCases, Uuid, Uuid, Uuid, Uuid) {
        let user_id = Uuid::new_v4();
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();

        let obj_repo = Arc::new(MockSharedObjectRepo::new());
        let owner_repo = Arc::new(MockOwnerRepo::new());
        let credit_repo = Arc::new(MockCreditBalanceRepo::new());

        let owner = create_test_owner(user_id, org_id);
        let owner_id = owner.id;
        owner_repo.add_owner(owner);

        let uc = SharedObjectUseCases::new(
            obj_repo as Arc<dyn SharedObjectRepository>,
            owner_repo as Arc<dyn OwnerRepository>,
            credit_repo as Arc<dyn OwnerCreditBalanceRepository>,
        );

        (uc, user_id, org_id, building_id, owner_id)
    }

    fn make_create_dto(building_id: Uuid) -> CreateSharedObjectDto {
        CreateSharedObjectDto {
            building_id,
            object_category: SharedObjectCategory::Tools,
            object_name: "Power Drill".to_string(),
            description: "18V cordless drill with battery".to_string(),
            condition: ObjectCondition::Good,
            is_available: true,
            rental_credits_per_day: Some(2),
            deposit_credits: Some(10),
            borrowing_duration_days: Some(7),
            photos: None,
            location_details: Some("Basement".to_string()),
            usage_instructions: None,
        }
    }

    // ── Tests ───────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_create_shared_object_success() {
        let (uc, user_id, org_id, building_id, _) = setup();
        let dto = make_create_dto(building_id);
        let result = uc.create_shared_object(user_id, org_id, dto).await;
        assert!(result.is_ok());
        let resp = result.unwrap();
        assert_eq!(resp.object_name, "Power Drill");
        assert_eq!(resp.owner_name, "Jean Dupont");
    }

    #[tokio::test]
    async fn test_get_shared_object_success() {
        let (uc, user_id, org_id, building_id, _) = setup();
        let dto = make_create_dto(building_id);
        let created = uc.create_shared_object(user_id, org_id, dto).await.unwrap();

        let result = uc.get_shared_object(created.id).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().id, created.id);
    }

    #[tokio::test]
    async fn test_get_shared_object_not_found() {
        let (uc, _, _, _, _) = setup();
        let result = uc.get_shared_object(Uuid::new_v4()).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Shared object not found");
    }

    #[tokio::test]
    async fn test_delete_shared_object_success() {
        let (uc, user_id, org_id, building_id, _) = setup();
        let dto = make_create_dto(building_id);
        let created = uc.create_shared_object(user_id, org_id, dto).await.unwrap();

        let result = uc.delete_shared_object(created.id, user_id, org_id).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_delete_shared_object_wrong_owner() {
        let (uc, user_id, org_id, building_id, _) = setup();
        let dto = make_create_dto(building_id);
        let created = uc.create_shared_object(user_id, org_id, dto).await.unwrap();

        // Another user tries to delete -- will fail because user not found as owner
        let other = Uuid::new_v4();
        let result = uc.delete_shared_object(created.id, other, org_id).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Owner not found"));
    }

    #[tokio::test]
    async fn test_list_building_objects() {
        let (uc, user_id, org_id, building_id, _) = setup();

        let dto1 = make_create_dto(building_id);
        let mut dto2 = make_create_dto(building_id);
        dto2.object_name = "Hammer".to_string();
        dto2.description = "Steel hammer for nails".to_string();

        uc.create_shared_object(user_id, org_id, dto1).await.unwrap();
        uc.create_shared_object(user_id, org_id, dto2).await.unwrap();

        let result = uc.list_building_objects(building_id).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 2);
    }

    #[tokio::test]
    async fn test_owner_not_found() {
        let obj_repo = Arc::new(MockSharedObjectRepo::new());
        let owner_repo = Arc::new(MockOwnerRepo::new());
        let credit_repo = Arc::new(MockCreditBalanceRepo::new());
        // No owner added

        let uc = SharedObjectUseCases::new(
            obj_repo as Arc<dyn SharedObjectRepository>,
            owner_repo as Arc<dyn OwnerRepository>,
            credit_repo as Arc<dyn OwnerCreditBalanceRepository>,
        );

        let dto = make_create_dto(Uuid::new_v4());
        let result = uc.create_shared_object(Uuid::new_v4(), Uuid::new_v4(), dto).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Owner not found"));
    }
}
