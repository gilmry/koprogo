use crate::application::ports::{OwnerRepository, UnitOwnerRepository, UnitRepository};
use crate::domain::entities::UnitOwner;
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

#[cfg(test)]
#[path = "unit_owner_use_cases_test.rs"]
mod unit_owner_use_cases_test;

pub struct UnitOwnerUseCases {
    unit_owner_repository: Arc<dyn UnitOwnerRepository>,
    unit_repository: Arc<dyn UnitRepository>,
    owner_repository: Arc<dyn OwnerRepository>,
}

impl UnitOwnerUseCases {
    pub fn new(
        unit_owner_repository: Arc<dyn UnitOwnerRepository>,
        unit_repository: Arc<dyn UnitRepository>,
        owner_repository: Arc<dyn OwnerRepository>,
    ) -> Self {
        Self {
            unit_owner_repository,
            unit_repository,
            owner_repository,
        }
    }

    /// Add an owner to a unit with specified ownership percentage
    pub async fn add_owner_to_unit(
        &self,
        unit_id: Uuid,
        owner_id: Uuid,
        ownership_percentage: f64,
        is_primary_contact: bool,
    ) -> Result<UnitOwner, String> {
        // Validate that unit exists
        self.unit_repository
            .find_by_id(unit_id)
            .await?
            .ok_or("Unit not found")?;

        // Validate that owner exists
        self.owner_repository
            .find_by_id(owner_id)
            .await?
            .ok_or("Owner not found")?;

        // Check if this owner is already active on this unit
        if let Some(_existing) = self
            .unit_owner_repository
            .find_active_by_unit_and_owner(unit_id, owner_id)
            .await?
        {
            return Err("Owner is already active on this unit".to_string());
        }

        // Validate total ownership percentage won't exceed 100%
        let current_total = self
            .unit_owner_repository
            .get_total_ownership_percentage(unit_id)
            .await?;

        if current_total + ownership_percentage > 1.0 {
            return Err(format!(
                "Total ownership would exceed 100% (current: {:.2}%, adding: {:.2}%)",
                current_total * 100.0,
                ownership_percentage * 100.0
            ));
        }

        // If this is primary contact, unset any existing primary contact
        if is_primary_contact {
            self.unset_all_primary_contacts(unit_id).await?;
        }

        // Create the unit-owner relationship
        let unit_owner =
            UnitOwner::new(unit_id, owner_id, ownership_percentage, is_primary_contact)?;

        self.unit_owner_repository.create(&unit_owner).await
    }

    /// Remove an owner from a unit (sets end_date to now)
    pub async fn remove_owner_from_unit(
        &self,
        unit_id: Uuid,
        owner_id: Uuid,
    ) -> Result<UnitOwner, String> {
        // Find the active relationship
        let mut unit_owner = self
            .unit_owner_repository
            .find_active_by_unit_and_owner(unit_id, owner_id)
            .await?
            .ok_or("Active unit-owner relationship not found")?;

        // End the ownership
        unit_owner.end_ownership(Utc::now())?;

        self.unit_owner_repository.update(&unit_owner).await
    }

    /// Update the ownership percentage for a unit-owner relationship
    pub async fn update_ownership_percentage(
        &self,
        unit_owner_id: Uuid,
        new_percentage: f64,
    ) -> Result<UnitOwner, String> {
        // Find the unit-owner relationship
        let mut unit_owner = self
            .unit_owner_repository
            .find_by_id(unit_owner_id)
            .await?
            .ok_or("Unit-owner relationship not found")?;

        // Validate it's still active
        if !unit_owner.is_active() {
            return Err("Cannot update percentage of ended ownership".to_string());
        }

        // Calculate what the new total would be
        let current_total = self
            .unit_owner_repository
            .get_total_ownership_percentage(unit_owner.unit_id)
            .await?;
        let old_percentage = unit_owner.ownership_percentage;
        let new_total = current_total - old_percentage + new_percentage;

        if new_total > 1.0 {
            return Err(format!(
                "Total ownership would exceed 100% (current: {:.2}%, new total: {:.2}%)",
                current_total * 100.0,
                new_total * 100.0
            ));
        }

        // Update the percentage
        unit_owner.update_percentage(new_percentage)?;

        self.unit_owner_repository.update(&unit_owner).await
    }

    /// Transfer ownership from one owner to another
    pub async fn transfer_ownership(
        &self,
        from_owner_id: Uuid,
        to_owner_id: Uuid,
        unit_id: Uuid,
    ) -> Result<(UnitOwner, UnitOwner), String> {
        // Validate that both owners exist
        self.owner_repository
            .find_by_id(from_owner_id)
            .await?
            .ok_or("Source owner not found")?;

        self.owner_repository
            .find_by_id(to_owner_id)
            .await?
            .ok_or("Target owner not found")?;

        // Get the active relationship from the source owner
        let mut from_relationship = self
            .unit_owner_repository
            .find_active_by_unit_and_owner(unit_id, from_owner_id)
            .await?
            .ok_or("Source owner does not own this unit")?;

        // Check if target owner already has an active relationship
        if let Some(_existing) = self
            .unit_owner_repository
            .find_active_by_unit_and_owner(unit_id, to_owner_id)
            .await?
        {
            return Err("Target owner already owns this unit".to_string());
        }

        // End the source ownership
        let transfer_date = Utc::now();
        from_relationship.end_ownership(transfer_date)?;

        // Create new ownership for target owner with same percentage
        let to_relationship = UnitOwner::new(
            unit_id,
            to_owner_id,
            from_relationship.ownership_percentage,
            from_relationship.is_primary_contact,
        )?;

        // Update both relationships
        let ended_relationship = self
            .unit_owner_repository
            .update(&from_relationship)
            .await?;
        let new_relationship = self.unit_owner_repository.create(&to_relationship).await?;

        Ok((ended_relationship, new_relationship))
    }

    /// Get all current owners of a unit
    pub async fn get_unit_owners(&self, unit_id: Uuid) -> Result<Vec<UnitOwner>, String> {
        // Validate unit exists
        self.unit_repository
            .find_by_id(unit_id)
            .await?
            .ok_or("Unit not found")?;

        self.unit_owner_repository
            .find_current_owners_by_unit(unit_id)
            .await
    }

    /// Get all current units owned by an owner
    pub async fn get_owner_units(&self, owner_id: Uuid) -> Result<Vec<UnitOwner>, String> {
        // Validate owner exists
        self.owner_repository
            .find_by_id(owner_id)
            .await?
            .ok_or("Owner not found")?;

        self.unit_owner_repository
            .find_current_units_by_owner(owner_id)
            .await
    }

    /// Get ownership history for a unit (including past owners)
    pub async fn get_unit_ownership_history(
        &self,
        unit_id: Uuid,
    ) -> Result<Vec<UnitOwner>, String> {
        // Validate unit exists
        self.unit_repository
            .find_by_id(unit_id)
            .await?
            .ok_or("Unit not found")?;

        self.unit_owner_repository
            .find_all_owners_by_unit(unit_id)
            .await
    }

    /// Get ownership history for an owner (including past units)
    pub async fn get_owner_ownership_history(
        &self,
        owner_id: Uuid,
    ) -> Result<Vec<UnitOwner>, String> {
        // Validate owner exists
        self.owner_repository
            .find_by_id(owner_id)
            .await?
            .ok_or("Owner not found")?;

        self.unit_owner_repository
            .find_all_units_by_owner(owner_id)
            .await
    }

    /// Set a unit-owner relationship as primary contact
    pub async fn set_primary_contact(&self, unit_owner_id: Uuid) -> Result<UnitOwner, String> {
        // Find the unit-owner relationship
        let mut unit_owner = self
            .unit_owner_repository
            .find_by_id(unit_owner_id)
            .await?
            .ok_or("Unit-owner relationship not found")?;

        // Validate it's still active
        if !unit_owner.is_active() {
            return Err("Cannot set primary contact for ended ownership".to_string());
        }

        // Unset all other primary contacts for this unit
        self.unset_all_primary_contacts(unit_owner.unit_id).await?;

        // Set this one as primary
        unit_owner.set_primary_contact(true);

        self.unit_owner_repository.update(&unit_owner).await
    }

    /// Get a specific unit-owner relationship by ID
    pub async fn get_unit_owner(&self, id: Uuid) -> Result<Option<UnitOwner>, String> {
        self.unit_owner_repository.find_by_id(id).await
    }

    /// Check if a unit has any active owners
    pub async fn has_active_owners(&self, unit_id: Uuid) -> Result<bool, String> {
        self.unit_owner_repository.has_active_owners(unit_id).await
    }

    /// Get the total ownership percentage for a unit
    pub async fn get_total_ownership_percentage(&self, unit_id: Uuid) -> Result<f64, String> {
        self.unit_owner_repository
            .get_total_ownership_percentage(unit_id)
            .await
    }

    // Helper method to unset all primary contacts for a unit
    async fn unset_all_primary_contacts(&self, unit_id: Uuid) -> Result<(), String> {
        let current_owners = self
            .unit_owner_repository
            .find_current_owners_by_unit(unit_id)
            .await?;

        for mut owner in current_owners {
            if owner.is_primary_contact {
                owner.set_primary_contact(false);
                self.unit_owner_repository.update(&owner).await?;
            }
        }

        Ok(())
    }
}
