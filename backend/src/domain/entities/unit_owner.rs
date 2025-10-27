use chrono::{DateTime, Utc};
use uuid::Uuid;

/// UnitOwner represents the ownership relationship between a Unit and an Owner
/// This entity supports:
/// - Multiple owners per unit (co-ownership, indivision)
/// - Multiple units per owner (owner in multiple buildings)
/// - Ownership percentage tracking
/// - Historical ownership tracking (start_date, end_date)
#[derive(Debug, Clone)]
pub struct UnitOwner {
    pub id: Uuid,
    pub unit_id: Uuid,
    pub owner_id: Uuid,

    /// Ownership percentage (0.0 to 1.0)
    /// Example: 0.5 = 50%, 1.0 = 100%
    pub ownership_percentage: f64,

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
        ownership_percentage: f64,
        is_primary_contact: bool,
    ) -> Result<Self, String> {
        // Validate ownership percentage
        if ownership_percentage <= 0.0 || ownership_percentage > 1.0 {
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
        ownership_percentage: f64,
        is_primary_contact: bool,
        start_date: DateTime<Utc>,
    ) -> Result<Self, String> {
        if ownership_percentage <= 0.0 || ownership_percentage > 1.0 {
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
    pub fn update_percentage(&mut self, new_percentage: f64) -> Result<(), String> {
        if new_percentage <= 0.0 || new_percentage > 1.0 {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_unit_owner() {
        let unit_id = Uuid::new_v4();
        let owner_id = Uuid::new_v4();

        let unit_owner = UnitOwner::new(unit_id, owner_id, 0.5, true).unwrap();

        assert_eq!(unit_owner.unit_id, unit_id);
        assert_eq!(unit_owner.owner_id, owner_id);
        assert_eq!(unit_owner.ownership_percentage, 0.5);
        assert!(unit_owner.is_primary_contact);
        assert!(unit_owner.is_active());
    }

    #[test]
    fn test_invalid_ownership_percentage() {
        let unit_id = Uuid::new_v4();
        let owner_id = Uuid::new_v4();

        // Test percentage > 1.0
        let result = UnitOwner::new(unit_id, owner_id, 1.5, false);
        assert!(result.is_err());

        // Test percentage <= 0
        let result = UnitOwner::new(unit_id, owner_id, 0.0, false);
        assert!(result.is_err());

        let result = UnitOwner::new(unit_id, owner_id, -0.5, false);
        assert!(result.is_err());
    }

    #[test]
    fn test_end_ownership() {
        let unit_id = Uuid::new_v4();
        let owner_id = Uuid::new_v4();

        let mut unit_owner = UnitOwner::new(unit_id, owner_id, 1.0, true).unwrap();

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

        let mut unit_owner = UnitOwner::new(unit_id, owner_id, 1.0, true).unwrap();

        // End date before start date should fail
        let invalid_end_date = unit_owner.start_date - chrono::Duration::days(1);
        let result = unit_owner.end_ownership(invalid_end_date);

        assert!(result.is_err());
    }

    #[test]
    fn test_update_percentage() {
        let unit_id = Uuid::new_v4();
        let owner_id = Uuid::new_v4();

        let mut unit_owner = UnitOwner::new(unit_id, owner_id, 0.5, true).unwrap();

        unit_owner.update_percentage(0.75).unwrap();
        assert_eq!(unit_owner.ownership_percentage, 0.75);

        // Invalid percentage
        let result = unit_owner.update_percentage(1.5);
        assert!(result.is_err());
    }

    #[test]
    fn test_update_percentage_boundary_values() {
        let unit_id = Uuid::new_v4();
        let owner_id = Uuid::new_v4();

        let mut unit_owner = UnitOwner::new(unit_id, owner_id, 0.5, false).unwrap();

        // Test boundary: exactly 1.0 (100%) is valid
        assert!(unit_owner.update_percentage(1.0).is_ok());
        assert_eq!(unit_owner.ownership_percentage, 1.0);

        // Test boundary: 0.0 is invalid
        assert!(unit_owner.update_percentage(0.0).is_err());

        // Test boundary: 0.0001 (0.01%) is valid
        assert!(unit_owner.update_percentage(0.0001).is_ok());
        assert_eq!(unit_owner.ownership_percentage, 0.0001);

        // Test boundary: 1.0001 is invalid
        assert!(unit_owner.update_percentage(1.0001).is_err());

        // Test negative values
        assert!(unit_owner.update_percentage(-0.5).is_err());
    }

    #[test]
    fn test_set_primary_contact() {
        let unit_id = Uuid::new_v4();
        let owner_id = Uuid::new_v4();

        let mut unit_owner = UnitOwner::new(unit_id, owner_id, 0.5, false).unwrap();

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
        let unit_owner = UnitOwner::new(unit_id, owner_id, 0.3333, false).unwrap();
        assert_eq!(unit_owner.ownership_percentage, 0.3333);

        // Test with very small percentage
        let unit_owner = UnitOwner::new(unit_id, owner_id, 0.0001, false).unwrap();
        assert_eq!(unit_owner.ownership_percentage, 0.0001);
    }

    #[test]
    fn test_end_ownership_updates_end_date() {
        let unit_id = Uuid::new_v4();
        let owner_id = Uuid::new_v4();

        let mut unit_owner = UnitOwner::new(unit_id, owner_id, 1.0, true).unwrap();

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

        let mut unit_owner = UnitOwner::new(unit_id, owner_id, 1.0, true).unwrap();

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
        let unit_owner = UnitOwner::new(unit_id, owner_id, 0.5, false).unwrap();
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

        let mut unit_owner = UnitOwner::new(unit_id, owner_id, 0.5, false).unwrap();
        let original_updated_at = unit_owner.updated_at;

        // Wait a tiny bit to ensure timestamp changes
        std::thread::sleep(std::time::Duration::from_millis(10));

        unit_owner.update_percentage(0.6).unwrap();
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

        let unit_owner = UnitOwner::new(unit_id, owner_id, 1.0, true).unwrap();
        assert_eq!(unit_owner.ownership_percentage, 1.0);
    }

    #[test]
    fn test_multiple_owners_scenario_percentages() {
        let unit_id = Uuid::new_v4();
        let owner1_id = Uuid::new_v4();
        let owner2_id = Uuid::new_v4();
        let owner3_id = Uuid::new_v4();

        // Scenario: 3 co-owners with 50%, 30%, 20%
        let owner1 = UnitOwner::new(unit_id, owner1_id, 0.5, true).unwrap();
        let owner2 = UnitOwner::new(unit_id, owner2_id, 0.3, false).unwrap();
        let owner3 = UnitOwner::new(unit_id, owner3_id, 0.2, false).unwrap();

        assert_eq!(owner1.ownership_percentage, 0.5);
        assert_eq!(owner2.ownership_percentage, 0.3);
        assert_eq!(owner3.ownership_percentage, 0.2);

        // Total should be 1.0
        let total =
            owner1.ownership_percentage + owner2.ownership_percentage + owner3.ownership_percentage;
        assert!((total - 1.0).abs() < f64::EPSILON);
    }
}
