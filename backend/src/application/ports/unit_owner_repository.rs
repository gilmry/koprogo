use crate::domain::entities::{LotHolder, UnitOwner};
use async_trait::async_trait;
use rust_decimal::Decimal;
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
    async fn get_total_ownership_percentage(&self, unit_id: Uuid) -> Result<Decimal, String>;

    /// Find active unit-owner relationship by unit and owner IDs
    async fn find_active_by_unit_and_owner(
        &self,
        unit_id: Uuid,
        owner_id: Uuid,
    ) -> Result<Option<UnitOwner>, String>;

    /// Get all active unit-owner relationships for a building
    /// Returns tuples of (unit_id, owner_id, ownership_percentage)
    /// Useful for calculating charge distributions
    /// Détentions actives d'un immeuble, avec le pourcentage BRUT de détention
    /// dans le lot (`unit_owners.ownership_percentage`).
    ///
    /// ⚠️ Ce pourcentage n'est PAS une quote-part de charge. Un propriétaire
    /// unique d'un lot vaut 1.0 quel que soit le poids de son lot dans
    /// l'immeuble. Ne l'utiliser que pour identifier QUI détient quoi
    /// (autorisations, éligibilité au vote), jamais pour répartir un montant :
    /// voir `find_active_quota_shares_by_building`.
    async fn find_active_by_building(
        &self,
        building_id: Uuid,
    ) -> Result<Vec<(Uuid, Uuid, Decimal)>, String>;

    /// Quotes-parts de CHARGE des détenteurs actifs d'un immeuble.
    ///
    /// Renvoie `(unit_id, owner_id, part)` où `part` est la fraction [0,1] du
    /// montant total qui incombe à ce copropriétaire :
    ///
    /// ```text
    /// part = (unit.quota / building.total_tantiemes) × ownership_percentage
    /// ```
    ///
    /// C'est la formule de l'Art. 3.84 CC, déjà implémentée et testée par
    /// `ChargeDistribution::resolve_owner_quota`. La somme des parts d'un
    /// immeuble conforme vaut 1.
    async fn find_active_quota_shares_by_building(
        &self,
        building_id: Uuid,
    ) -> Result<Vec<(Uuid, Uuid, Decimal)>, String>;

    /// Story H17 (Art. 3.87 §1 CC) — titularités actives d'un lot réduites aux
    /// attributs pertinents pour le calcul du droit de vote (`ownership_type`
    /// + `is_voting_representative`). Consommé par le gate vote (`cast_vote`)
    /// pour rejeter les lots démembrés/indivis sans représentant unique désigné
    /// (`VOTING_RIGHT_SUSPENDED`).
    async fn find_voting_holders_by_unit(&self, unit_id: Uuid) -> Result<Vec<LotHolder>, String>;
}
