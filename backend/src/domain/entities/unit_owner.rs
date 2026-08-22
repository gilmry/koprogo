use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use uuid::Uuid;

/// UnitOwner represents the ownership relationship between a Unit and an Owner
/// This entity supports:
/// - Multiple owners per unit (co-ownership, indivision)
/// - Multiple units per owner (owner in multiple buildings)
/// - Ownership percentage tracking
/// - Historical ownership tracking (start_date, end_date)
///
/// MONETARY-ADJACENT: ownership_percentage uses rust_decimal::Decimal (cf. ADR-0007).
/// Quote-parts drive charge distribution; rounding errors propagate to invoices.
#[derive(Debug, Clone)]
pub struct UnitOwner {
    pub id: Uuid,
    pub unit_id: Uuid,
    pub owner_id: Uuid,

    /// Ownership percentage (0.0 to 1.0). Decimal exact (cf. ADR-0007).
    /// Example: dec!(0.5) = 50%, dec!(1.0) = 100%
    pub ownership_percentage: Decimal,

    /// Date when ownership started
    pub start_date: DateTime<Utc>,

    /// Date when ownership ended (None = current owner)
    pub end_date: Option<DateTime<Utc>>,

    /// Is this owner the primary contact for this unit?
    pub is_primary_contact: bool,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl UnitOwner {
    /// Create a new UnitOwner relationship
    pub fn new(
        unit_id: Uuid,
        owner_id: Uuid,
        ownership_percentage: Decimal,
        is_primary_contact: bool,
    ) -> Result<Self, String> {
        // Validate ownership percentage
        if ownership_percentage <= Decimal::ZERO || ownership_percentage > Decimal::ONE {
            return Err("Ownership percentage must be between 0 and 1".to_string());
        }

        Ok(Self {
            id: Uuid::new_v4(),
            unit_id,
            owner_id,
            ownership_percentage,
            start_date: Utc::now(),
            end_date: None,
            is_primary_contact,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }

    /// Create a new UnitOwner with a specific start date
    pub fn new_with_start_date(
        unit_id: Uuid,
        owner_id: Uuid,
        ownership_percentage: Decimal,
        is_primary_contact: bool,
        start_date: DateTime<Utc>,
    ) -> Result<Self, String> {
        if ownership_percentage <= Decimal::ZERO || ownership_percentage > Decimal::ONE {
            return Err("Ownership percentage must be between 0 and 1".to_string());
        }

        Ok(Self {
            id: Uuid::new_v4(),
            unit_id,
            owner_id,
            ownership_percentage,
            start_date,
            end_date: None,
            is_primary_contact,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }

    /// Check if this ownership is currently active
    pub fn is_active(&self) -> bool {
        self.end_date.is_none()
    }

    /// End this ownership relationship
    pub fn end_ownership(&mut self, end_date: DateTime<Utc>) -> Result<(), String> {
        if end_date <= self.start_date {
            return Err("End date must be after start date".to_string());
        }

        self.end_date = Some(end_date);
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Update ownership percentage
    pub fn update_percentage(&mut self, new_percentage: Decimal) -> Result<(), String> {
        if new_percentage <= Decimal::ZERO || new_percentage > Decimal::ONE {
            return Err("Ownership percentage must be between 0 and 1".to_string());
        }

        self.ownership_percentage = new_percentage;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Set as primary contact
    pub fn set_primary_contact(&mut self, is_primary: bool) {
        self.is_primary_contact = is_primary;
        self.updated_at = Utc::now();
    }
}

// ============================================================================
// Story H17 (Track H, CL3) — Représentant de vote / suspension (Art. 3.87 §1).
//
// Un lot peut appartenir à plusieurs titulaires (indivision) OU être démembré
// (usufruit/nue-propriété, emphytéose, superficie). Dans ce cas le droit de
// vote est SUSPENDU jusqu'à désignation d'un représentant unique (mandataire
// commun). Logique domaine pure (zéro I/O), consommée par le gate vote (H10)
// et le recalcul de quorum (H9). Cf. ADR-0011 + spec H17.
// ============================================================================

/// Nature de la titularité d'une ligne `unit_owners` (Art. 3.87 §1 CC).
/// Détermine si le lot vote directement ou requiert un représentant unique.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OwnershipType {
    /// Pleine propriété — titulaire unique, vote direct.
    #[default]
    FullOwner,
    /// Usufruitier (démembrement).
    Usufruct,
    /// Nu-propriétaire (démembrement).
    BareOwner,
    /// Co-titulaire en indivision.
    Indivisaire,
    /// Emphytéote (bail emphytéotique).
    Emphyteote,
    /// Superficiaire (droit de superficie).
    Superficiaire,
}

impl OwnershipType {
    /// Vrai uniquement pour la pleine propriété (seul cas votant sans
    /// représentant désigné quand le lot est mono-titulaire).
    pub fn is_full_ownership(&self) -> bool {
        matches!(self, OwnershipType::FullOwner)
    }
}

impl std::fmt::Display for OwnershipType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Valeurs alignées sur le CHECK SQL (migration 20260621000000).
        let s = match self {
            OwnershipType::FullOwner => "full_owner",
            OwnershipType::Usufruct => "usufruct",
            OwnershipType::BareOwner => "bare_owner",
            OwnershipType::Indivisaire => "indivisaire",
            OwnershipType::Emphyteote => "emphyteote",
            OwnershipType::Superficiaire => "superficiaire",
        };
        f.write_str(s)
    }
}

impl std::str::FromStr for OwnershipType {
    type Err = VotingRightError;

    /// Parse strict (mémoire `validate-before-compute`) : toute valeur hors
    /// enum → erreur typée, jamais un fallback silencieux.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "full_owner" => Ok(OwnershipType::FullOwner),
            "usufruct" => Ok(OwnershipType::Usufruct),
            "bare_owner" => Ok(OwnershipType::BareOwner),
            "indivisaire" => Ok(OwnershipType::Indivisaire),
            "emphyteote" => Ok(OwnershipType::Emphyteote),
            "superficiaire" => Ok(OwnershipType::Superficiaire),
            other => Err(VotingRightError::UnknownOwnershipType(other.to_string())),
        }
    }
}

/// Statut du droit de vote d'un lot (Art. 3.87 §1 CC).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VotingRightStatus {
    /// Le lot peut voter (mono-plein-propriétaire OU représentant désigné).
    Active,
    /// Vote suspendu : lot démembré/indivis sans représentant unique désigné.
    Suspended,
}

/// Une ligne de titularité d'un lot, réduite aux attributs pertinents pour le
/// calcul du droit de vote (Art. 3.87 §1). Value object pur.
///
/// Volontairement découplé de `UnitOwner` (persistance) : le gate vote et le
/// checker quorum n'ont besoin que de `(ownership_type, is_voting_representative)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LotHolder {
    pub ownership_type: OwnershipType,
    pub is_voting_representative: bool,
}

impl LotHolder {
    pub fn new(ownership_type: OwnershipType, is_voting_representative: bool) -> Self {
        Self {
            ownership_type,
            is_voting_representative,
        }
    }
}

/// Erreurs typées du calcul de droit de vote (Art. 3.87 §1 CC).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VotingRightError {
    /// Chaîne de titularité inconnue (hors enum / CHECK DB).
    UnknownOwnershipType(String),
    /// Plus d'un représentant de vote désigné pour un même lot. Art. 3.87 §1 :
    /// les titulaires désignent UN représentant unique.
    MultipleRepresentatives { unit_id: Uuid, count: usize },
}

impl std::fmt::Display for VotingRightError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VotingRightError::UnknownOwnershipType(s) => {
                write!(f, "Type de titularité inconnu : '{}'", s)
            }
            VotingRightError::MultipleRepresentatives { unit_id, count } => write!(
                f,
                "Lot {} : {} représentants de vote désignés, un seul autorisé (Art. 3.87 §1 CC)",
                unit_id, count
            ),
        }
    }
}

impl std::error::Error for VotingRightError {}

/// Droit de vote suspendu (Art. 3.87 §1 CC) : lot démembré/indivis sans
/// représentant unique désigné. Erreur typée → `AppError` 422
/// `VOTING_RIGHT_SUSPENDED` (bridge dans `application/error.rs`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VotingRightSuspendedError {
    pub unit_id: Uuid,
}

impl std::fmt::Display for VotingRightSuspendedError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Droit de vote suspendu pour le lot {} : lot démembré/indivis sans \
             représentant unique désigné (Art. 3.87 §1 CC)",
            self.unit_id
        )
    }
}

impl std::error::Error for VotingRightSuspendedError {}

/// Détermine le statut du droit de vote d'un lot à partir de ses titulaires
/// **actifs** (Art. 3.87 §1 CC). Domaine pur, déterministe.
///
/// - Aucun titulaire qualifié (rétro-compat des lots pré-H17) → `Active`
///   (lot supposé en pleine propriété mono-titulaire).
/// - Au moins un représentant de vote désigné → `Active` (le mandataire
///   unique exerce le vote du lot).
/// - Un seul titulaire en pleine propriété → `Active`.
/// - Sinon (indivision OU démembrement sans représentant) → `Suspended`.
pub fn voting_right_status(holders: &[LotHolder]) -> VotingRightStatus {
    if holders.is_empty() {
        return VotingRightStatus::Active;
    }
    if holders.iter().any(|h| h.is_voting_representative) {
        return VotingRightStatus::Active;
    }
    if holders.len() == 1 && holders[0].ownership_type.is_full_ownership() {
        return VotingRightStatus::Active;
    }
    VotingRightStatus::Suspended
}

/// Vérifie qu'**au plus un** représentant de vote est désigné pour le lot
/// (Art. 3.87 §1 : représentant UNIQUE). Erreur typée si ≥ 2 (à appeler lors
/// de la désignation d'un représentant).
pub fn assert_single_voting_representative(
    unit_id: Uuid,
    holders: &[LotHolder],
) -> Result<(), VotingRightError> {
    let count = holders
        .iter()
        .filter(|h| h.is_voting_representative)
        .count();
    if count >= 2 {
        return Err(VotingRightError::MultipleRepresentatives { unit_id, count });
    }
    Ok(())
}

/// Garde d'application (gate vote H10/H17) : un lot dont le vote est suspendu
/// ne peut pas voter (Art. 3.87 §1). Erreur typée → 422 `VOTING_RIGHT_SUSPENDED`.
pub fn assert_voting_right_active(
    unit_id: Uuid,
    holders: &[LotHolder],
) -> Result<(), VotingRightSuspendedError> {
    match voting_right_status(holders) {
        VotingRightStatus::Active => Ok(()),
        VotingRightStatus::Suspended => Err(VotingRightSuspendedError { unit_id }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_create_unit_owner() {
        let unit_id = Uuid::new_v4();
        let owner_id = Uuid::new_v4();

        let unit_owner = UnitOwner::new(unit_id, owner_id, dec!(0.5), true).unwrap();

        assert_eq!(unit_owner.unit_id, unit_id);
        assert_eq!(unit_owner.owner_id, owner_id);
        assert_eq!(unit_owner.ownership_percentage, dec!(0.5));
        assert!(unit_owner.is_primary_contact);
        assert!(unit_owner.is_active());
    }

    #[test]
    fn test_invalid_ownership_percentage() {
        let unit_id = Uuid::new_v4();
        let owner_id = Uuid::new_v4();

        // Test percentage > 1.0
        let result = UnitOwner::new(unit_id, owner_id, dec!(1.5), false);
        assert!(result.is_err());

        // Test percentage <= 0
        let result = UnitOwner::new(unit_id, owner_id, Decimal::ZERO, false);
        assert!(result.is_err());

        let result = UnitOwner::new(unit_id, owner_id, dec!(-0.5), false);
        assert!(result.is_err());
    }

    #[test]
    fn test_end_ownership() {
        let unit_id = Uuid::new_v4();
        let owner_id = Uuid::new_v4();

        let mut unit_owner = UnitOwner::new(unit_id, owner_id, Decimal::ONE, true).unwrap();

        assert!(unit_owner.is_active());

        let end_date = Utc::now() + chrono::Duration::days(1);
        unit_owner.end_ownership(end_date).unwrap();

        assert!(!unit_owner.is_active());
        assert_eq!(unit_owner.end_date, Some(end_date));
    }

    #[test]
    fn test_invalid_end_date() {
        let unit_id = Uuid::new_v4();
        let owner_id = Uuid::new_v4();

        let mut unit_owner = UnitOwner::new(unit_id, owner_id, Decimal::ONE, true).unwrap();

        // End date before start date should fail
        let invalid_end_date = unit_owner.start_date - chrono::Duration::days(1);
        let result = unit_owner.end_ownership(invalid_end_date);

        assert!(result.is_err());
    }

    #[test]
    fn test_update_percentage() {
        let unit_id = Uuid::new_v4();
        let owner_id = Uuid::new_v4();

        let mut unit_owner = UnitOwner::new(unit_id, owner_id, dec!(0.5), true).unwrap();

        unit_owner.update_percentage(dec!(0.75)).unwrap();
        assert_eq!(unit_owner.ownership_percentage, dec!(0.75));

        // Invalid percentage
        let result = unit_owner.update_percentage(dec!(1.5));
        assert!(result.is_err());
    }

    #[test]
    fn test_update_percentage_boundary_values() {
        let unit_id = Uuid::new_v4();
        let owner_id = Uuid::new_v4();

        let mut unit_owner = UnitOwner::new(unit_id, owner_id, dec!(0.5), false).unwrap();

        // Test boundary: exactly 1.0 (100%) is valid
        assert!(unit_owner.update_percentage(Decimal::ONE).is_ok());
        assert_eq!(unit_owner.ownership_percentage, Decimal::ONE);

        // Test boundary: 0.0 is invalid
        assert!(unit_owner.update_percentage(Decimal::ZERO).is_err());

        // Test boundary: 0.0001 (0.01%) is valid
        assert!(unit_owner.update_percentage(dec!(0.0001)).is_ok());
        assert_eq!(unit_owner.ownership_percentage, dec!(0.0001));

        // Test boundary: 1.0001 is invalid
        assert!(unit_owner.update_percentage(dec!(1.0001)).is_err());

        // Test negative values
        assert!(unit_owner.update_percentage(dec!(-0.5)).is_err());
    }

    #[test]
    fn test_set_primary_contact() {
        let unit_id = Uuid::new_v4();
        let owner_id = Uuid::new_v4();

        let mut unit_owner = UnitOwner::new(unit_id, owner_id, dec!(0.5), false).unwrap();

        assert!(!unit_owner.is_primary_contact);

        unit_owner.set_primary_contact(true);
        assert!(unit_owner.is_primary_contact);

        unit_owner.set_primary_contact(false);
        assert!(!unit_owner.is_primary_contact);
    }

    #[test]
    fn test_ownership_percentage_precision() {
        let unit_id = Uuid::new_v4();
        let owner_id = Uuid::new_v4();

        // Test with 4 decimal places (common for co-ownership)
        let unit_owner = UnitOwner::new(unit_id, owner_id, dec!(0.3333), false).unwrap();
        assert_eq!(unit_owner.ownership_percentage, dec!(0.3333));

        // Test with very small percentage
        let unit_owner = UnitOwner::new(unit_id, owner_id, dec!(0.0001), false).unwrap();
        assert_eq!(unit_owner.ownership_percentage, dec!(0.0001));
    }

    #[test]
    fn test_end_ownership_updates_end_date() {
        let unit_id = Uuid::new_v4();
        let owner_id = Uuid::new_v4();

        let mut unit_owner = UnitOwner::new(unit_id, owner_id, Decimal::ONE, true).unwrap();

        assert!(unit_owner.end_date.is_none());

        let end_date = Utc::now() + chrono::Duration::days(30);
        unit_owner.end_ownership(end_date).unwrap();

        assert!(unit_owner.end_date.is_some());
        assert_eq!(unit_owner.end_date.unwrap(), end_date);
    }

    #[test]
    fn test_cannot_end_ownership_twice() {
        let unit_id = Uuid::new_v4();
        let owner_id = Uuid::new_v4();

        let mut unit_owner = UnitOwner::new(unit_id, owner_id, Decimal::ONE, true).unwrap();

        let first_end = Utc::now() + chrono::Duration::days(1);
        unit_owner.end_ownership(first_end).unwrap();

        // Should still work, just updates the date
        let second_end = Utc::now() + chrono::Duration::days(2);
        let result = unit_owner.end_ownership(second_end);
        assert!(result.is_ok());
        assert_eq!(unit_owner.end_date.unwrap(), second_end);
    }

    #[test]
    fn test_timestamps_are_set() {
        let unit_id = Uuid::new_v4();
        let owner_id = Uuid::new_v4();

        let before = Utc::now();
        let unit_owner = UnitOwner::new(unit_id, owner_id, dec!(0.5), false).unwrap();
        let after = Utc::now();

        // created_at should be between before and after
        assert!(unit_owner.created_at >= before);
        assert!(unit_owner.created_at <= after);

        // updated_at should initially equal created_at (within millisecond precision)
        let diff = (unit_owner.created_at - unit_owner.updated_at)
            .num_milliseconds()
            .abs();
        assert!(diff < 1);
    }

    #[test]
    fn test_updated_at_changes_on_modification() {
        let unit_id = Uuid::new_v4();
        let owner_id = Uuid::new_v4();

        let mut unit_owner = UnitOwner::new(unit_id, owner_id, dec!(0.5), false).unwrap();
        let original_updated_at = unit_owner.updated_at;

        // Wait a tiny bit to ensure timestamp changes
        std::thread::sleep(std::time::Duration::from_millis(10));

        unit_owner.update_percentage(dec!(0.6)).unwrap();
        assert!(unit_owner.updated_at > original_updated_at);

        let previous_updated = unit_owner.updated_at;
        std::thread::sleep(std::time::Duration::from_millis(10));

        unit_owner.set_primary_contact(true);
        assert!(unit_owner.updated_at > previous_updated);
    }

    #[test]
    fn test_100_percent_ownership_is_valid() {
        let unit_id = Uuid::new_v4();
        let owner_id = Uuid::new_v4();

        let unit_owner = UnitOwner::new(unit_id, owner_id, Decimal::ONE, true).unwrap();
        assert_eq!(unit_owner.ownership_percentage, Decimal::ONE);
    }

    #[test]
    fn test_multiple_owners_scenario_percentages() {
        let unit_id = Uuid::new_v4();
        let owner1_id = Uuid::new_v4();
        let owner2_id = Uuid::new_v4();
        let owner3_id = Uuid::new_v4();

        // Scenario: 3 co-owners with 50%, 30%, 20%
        let owner1 = UnitOwner::new(unit_id, owner1_id, dec!(0.5), true).unwrap();
        let owner2 = UnitOwner::new(unit_id, owner2_id, dec!(0.3), false).unwrap();
        let owner3 = UnitOwner::new(unit_id, owner3_id, dec!(0.2), false).unwrap();

        assert_eq!(owner1.ownership_percentage, dec!(0.5));
        assert_eq!(owner2.ownership_percentage, dec!(0.3));
        assert_eq!(owner3.ownership_percentage, dec!(0.2));

        // Total should be 1.0 EXACTLY (Decimal — pas IEEE 754).
        let total =
            owner1.ownership_percentage + owner2.ownership_percentage + owner3.ownership_percentage;
        assert_eq!(total, Decimal::ONE);
    }

    // ------------------------------------------------------------------------
    // Story H17 (CL3) — Représentant de vote / suspension (Art. 3.87 §1 CC).
    // TDD 4 catégories : @happy / @edge / @security / @negative.
    // ------------------------------------------------------------------------

    use std::str::FromStr;

    fn holder(t: OwnershipType, rep: bool) -> LotHolder {
        LotHolder::new(t, rep)
    }

    /// @happy — lot mono-plein-propriétaire → vote actif.
    #[test]
    fn happy_voting_active_mono_full_owner() {
        let holders = [holder(OwnershipType::FullOwner, false)];
        assert_eq!(voting_right_status(&holders), VotingRightStatus::Active);
        assert!(assert_voting_right_active(Uuid::new_v4(), &holders).is_ok());
    }

    /// @happy — lot avec représentant de vote désigné → vote actif.
    #[test]
    fn happy_voting_active_with_designated_representative() {
        // Indivision (2 titulaires) mais un représentant désigné → actif.
        let holders = [
            holder(OwnershipType::Indivisaire, true),
            holder(OwnershipType::Indivisaire, false),
        ];
        assert_eq!(voting_right_status(&holders), VotingRightStatus::Active);
        assert!(assert_voting_right_active(Uuid::new_v4(), &holders).is_ok());
    }

    /// @happy — rétro-compat : aucune titularité qualifiée → actif (lot supposé
    /// pleine propriété mono-titulaire ; aucune régression de vote pré-H17).
    #[test]
    fn happy_voting_active_legacy_no_holders() {
        assert_eq!(voting_right_status(&[]), VotingRightStatus::Active);
        assert!(assert_voting_right_active(Uuid::new_v4(), &[]).is_ok());
    }

    /// @edge — lot démembré usufruit/nue-propriété AVEC représentant désigné
    /// (ici l'usufruitier) → actif.
    #[test]
    fn edge_voting_active_usufruct_with_representative() {
        let holders = [
            holder(OwnershipType::Usufruct, true),
            holder(OwnershipType::BareOwner, false),
        ];
        assert_eq!(voting_right_status(&holders), VotingRightStatus::Active);
    }

    /// @edge — emphytéote/superficiaire seul SANS représentant → suspendu
    /// (démembrement, pas pleine propriété).
    #[test]
    fn edge_voting_suspended_single_dismembered_holder() {
        for t in [OwnershipType::Emphyteote, OwnershipType::Superficiaire] {
            let holders = [holder(t, false)];
            assert_eq!(
                voting_right_status(&holders),
                VotingRightStatus::Suspended,
                "type {t} seul sans représentant doit suspendre le vote"
            );
        }
    }

    /// @edge — round-trip Display ⇄ FromStr aligné sur le CHECK SQL.
    #[test]
    fn edge_ownership_type_display_fromstr_roundtrip() {
        for t in [
            OwnershipType::FullOwner,
            OwnershipType::Usufruct,
            OwnershipType::BareOwner,
            OwnershipType::Indivisaire,
            OwnershipType::Emphyteote,
            OwnershipType::Superficiaire,
        ] {
            let s = t.to_string();
            assert_eq!(OwnershipType::from_str(&s).unwrap(), t);
        }
    }

    /// @security — lot indivis SANS représentant → suspendu + gate rejette le
    /// vote avec erreur typée `VotingRightSuspendedError` (→ 422).
    #[test]
    fn security_voting_suspended_indivision_without_representative() {
        let unit_id = Uuid::new_v4();
        let holders = [
            holder(OwnershipType::Indivisaire, false),
            holder(OwnershipType::Indivisaire, false),
        ];
        assert_eq!(voting_right_status(&holders), VotingRightStatus::Suspended);
        let err = assert_voting_right_active(unit_id, &holders).unwrap_err();
        assert_eq!(err.unit_id, unit_id);
    }

    /// @security — lot démembré (usufruit + nue-propriété) SANS représentant →
    /// suspendu (le contournement « voter quand même » est bloqué).
    #[test]
    fn security_voting_suspended_dismembered_without_representative() {
        let holders = [
            holder(OwnershipType::Usufruct, false),
            holder(OwnershipType::BareOwner, false),
        ];
        assert_eq!(voting_right_status(&holders), VotingRightStatus::Suspended);
        assert!(assert_voting_right_active(Uuid::new_v4(), &holders).is_err());
    }

    /// @negative — désignation de 2 représentants pour un même lot → rejet typé
    /// (Art. 3.87 §1 : représentant unique), pas de panic.
    #[test]
    fn negative_multiple_voting_representatives_rejected() {
        let unit_id = Uuid::new_v4();
        let holders = [
            holder(OwnershipType::Indivisaire, true),
            holder(OwnershipType::Indivisaire, true),
        ];
        let err = assert_single_voting_representative(unit_id, &holders).unwrap_err();
        match err {
            VotingRightError::MultipleRepresentatives { unit_id: u, count } => {
                assert_eq!(u, unit_id);
                assert_eq!(count, 2);
            }
            other => panic!("attendu MultipleRepresentatives, obtenu {other:?}"),
        }
        // Un seul représentant (ou zéro) passe.
        assert!(assert_single_voting_representative(
            unit_id,
            &[holder(OwnershipType::Indivisaire, true)]
        )
        .is_ok());
    }

    /// @negative — type de titularité inconnu → erreur typée, jamais un
    /// fallback silencieux.
    #[test]
    fn negative_unknown_ownership_type_rejected() {
        let err = OwnershipType::from_str("locataire").unwrap_err();
        assert_eq!(
            err,
            VotingRightError::UnknownOwnershipType("locataire".to_string())
        );
    }
}
