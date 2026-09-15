use crate::application::error::AppError;
use crate::application::ports::{OwnerRepository, UnitOwnerRepository, UnitRepository};
use crate::domain::entities::{
    assert_single_voting_representative, LotHolder, OwnershipType, UnitOwner,
};
use chrono::Utc;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::sync::Arc;
use uuid::Uuid;

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
        ownership_percentage: Decimal,
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

        // Validate total ownership percentage won't exceed 100% (Art. 577-2 §4 CC)
        let current_total = self
            .unit_owner_repository
            .get_total_ownership_percentage(unit_id)
            .await?;

        // CRITICAL: This validation MUST block if total > 100.0% (Belgian legal requirement)
        if current_total + ownership_percentage > Decimal::ONE {
            return Err(format!(
                "Total ownership would exceed 100% (Art. 577-2 §4 CC). \
                 Current: {}%, adding: {}%, total would be: {}%",
                current_total * dec!(100),
                ownership_percentage * dec!(100),
                (current_total + ownership_percentage) * dec!(100)
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
        new_percentage: Decimal,
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

        // CRITICAL: This validation MUST block if total > 100.0% (Art. 577-2 §4 CC)
        if new_total > Decimal::ONE {
            return Err(format!(
                "Total ownership would exceed 100% (Art. 577-2 §4 CC). \
                 Current without this owner: {}%, new percentage: {}%, total would be: {}%",
                (current_total - old_percentage) * dec!(100),
                new_percentage * dec!(100),
                new_total * dec!(100)
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
    pub async fn get_total_ownership_percentage(&self, unit_id: Uuid) -> Result<Decimal, String> {
        self.unit_owner_repository
            .get_total_ownership_percentage(unit_id)
            .await
    }

    /// Désigne `owner_id` comme représentant de vote unique du lot `unit_id`
    /// (Art. 3.87 §1 CC, #848). Un lot à plusieurs titulaires actifs (couple,
    /// succession — le cas le plus ordinaire) voit son vote SUSPENDU tant
    /// qu'aucun d'eux n'est désigné ; cette méthode livre la désignation qui
    /// lève cette suspension (`voting_right_status` redevient `Active`).
    ///
    /// Idempotent : redésigner le représentant déjà en place est un no-op, pas
    /// une seconde désignation.
    ///
    /// Refuse (`AppError::Conflict`, via `VotingRightError::MultipleRepresentatives`)
    /// si un AUTRE titulaire du lot est déjà désigné : l'Art. 3.87 §1 prévoit
    /// un représentant UNIQUE. C'est le contrôle dormant nommé par #848 —
    /// `assert_single_voting_representative` — appelé ici pour la première
    /// fois en production, sur l'état PROSPECTIF (les titulaires actuels plus
    /// la désignation qui vient), avant toute écriture.
    pub async fn designate_voting_representative(
        &self,
        unit_id: Uuid,
        owner_id: Uuid,
    ) -> Result<UnitOwner, AppError> {
        // La désignation ne peut porter que sur une titularité ACTIVE de CE
        // lot : ni un rattachement clos, ni un autre lot que celui visé par la
        // route (cf. handler — `verify_unit_org_access` filtre déjà le
        // cloisonnement organisation, ceci filtre la cohérence des données).
        let target = self
            .unit_owner_repository
            .find_active_by_unit_and_owner(unit_id, owner_id)
            .await
            .map_err(AppError::from)?
            .ok_or_else(|| {
                AppError::NotFound(format!(
                    "Aucune titularité active de l'owner {owner_id} sur le lot {unit_id}"
                ))
            })?;

        if self
            .unit_owner_repository
            .is_voting_representative(target.id)
            .await
            .map_err(AppError::from)?
        {
            // Déjà désigné : rien à écrire, rien à contrôler à nouveau.
            return Ok(target);
        }

        // État prospectif = titulaires actuels (aucun n'est le représentant
        // visé, on vient de le vérifier) + la désignation qui vient. Le type
        // de titularité du titulaire ajouté n'entre pas dans le calcul
        // d'`assert_single_voting_representative` (il ne compte que
        // `is_voting_representative`) : `FullOwner` par défaut n'introduit
        // aucun biais.
        let mut prospective: Vec<LotHolder> = self
            .unit_owner_repository
            .find_voting_holders_by_unit(unit_id)
            .await
            .map_err(AppError::from)?;
        prospective.push(LotHolder::new(OwnershipType::default(), true));
        assert_single_voting_representative(unit_id, &prospective).map_err(AppError::from)?;

        self.unit_owner_repository
            .set_voting_representative(target.id)
            .await
            .map_err(AppError::from)?;

        Ok(target)
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

// Déclaré en BAS de fichier, pas en haut : la garde `garde_controles_dormants`
// coupe chaque fichier à la première occurrence textuelle de l'attribut
// cfg(test), et ne scanne que ce qui précède pour trouver les appels de
// production. Cet attribut en tête de fichier aurait fait disparaître TOUT
// l'`impl UnitOwnerUseCases` ci-dessus — donc l'appel de
// `assert_single_voting_representative` (#848) — de son scan, malgré un appel
// bien réel. Repéré en écrivant cette story ; aucun autre fichier de
// `use_cases/` ne déclare son module de test de cette façon en tête.
#[cfg(test)]
#[path = "unit_owner_use_cases_test.rs"]
mod unit_owner_use_cases_test;
