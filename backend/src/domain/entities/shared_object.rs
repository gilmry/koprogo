use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Category for shared objects
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SharedObjectCategory {
    /// Tools and equipment (drill, ladder, hammer, saw, etc.)
    Tools,
    /// Books and magazines
    Books,
    /// Electronics (projector, camera, tablet, etc.)
    Electronics,
    /// Sports equipment (bike, skis, tennis racket, etc.)
    Sports,
    /// Gardening tools and equipment (mower, trimmer, etc.)
    Gardening,
    /// Kitchen appliances (mixer, pressure cooker, etc.)
    Kitchen,
    /// Baby and children items (stroller, car seat, toys, etc.)
    Baby,
    /// Other shared objects
    Other,
}

/// Condition of shared object
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum ObjectCondition {
    /// Excellent condition (like new)
    Excellent,
    /// Good condition (minor wear)
    Good,
    /// Fair condition (visible wear but functional)
    Fair,
    /// Used condition (significant wear)
    Used,
}

/// Shared object for community equipment sharing
///
/// Represents an object that a building resident can lend to other members.
/// Integrates with SEL (Local Exchange Trading System) for optional credit-based rental.
///
/// # Business Rules
/// - object_name must be 3-100 characters
/// - description max 1000 characters
/// - rental_credits_per_day: 0-20 (0 = free, reasonable daily rate)
/// - deposit_credits: 0-100 (security deposit)
/// - borrowing_duration_days: 1-90 (max borrowing period)
/// - Only owner can update/delete object
/// - Only available objects can be borrowed
/// - Borrower must return object before borrowing another from same owner
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedObject {
    pub id: Uuid,
    pub owner_id: Uuid,
    pub building_id: Uuid,
    pub object_category: SharedObjectCategory,
    pub object_name: String,
    pub description: String,
    pub condition: ObjectCondition,
    pub is_available: bool,
    /// Rental rate in SEL credits per day (0 = free, None = not specified)
    pub rental_credits_per_day: Option<i32>,
    /// Security deposit in SEL credits (refunded on return)
    pub deposit_credits: Option<i32>,
    /// Maximum borrowing duration in days (1-90)
    pub borrowing_duration_days: Option<i32>,
    /// Current borrower (if borrowed)
    pub current_borrower_id: Option<Uuid>,
    /// When object was borrowed
    pub borrowed_at: Option<DateTime<Utc>>,
    /// When object is due back
    pub due_back_at: Option<DateTime<Utc>>,
    /// Photo URLs (optional)
    pub photos: Option<Vec<String>>,
    /// Pickup location details (optional)
    pub location_details: Option<String>,
    /// Usage instructions (optional)
    pub usage_instructions: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl SharedObject {
    /// Create a new shared object
    ///
    /// # Validation
    /// - object_name: 3-100 characters
    /// - description: max 1000 characters
    /// - rental_credits_per_day: 0-20 if provided
    /// - deposit_credits: 0-100 if provided
    /// - borrowing_duration_days: 1-90 if provided
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        owner_id: Uuid,
        building_id: Uuid,
        object_category: SharedObjectCategory,
        object_name: String,
        description: String,
        condition: ObjectCondition,
        is_available: bool,
        rental_credits_per_day: Option<i32>,
        deposit_credits: Option<i32>,
        borrowing_duration_days: Option<i32>,
        photos: Option<Vec<String>>,
        location_details: Option<String>,
        usage_instructions: Option<String>,
    ) -> Result<Self, String> {
        // Validate object_name
        if object_name.len() < 3 {
            return Err("Object name must be at least 3 characters".to_string());
        }
        if object_name.len() > 100 {
            return Err("Object name cannot exceed 100 characters".to_string());
        }

        // Validate description
        if description.trim().is_empty() {
            return Err("Description cannot be empty".to_string());
        }
        if description.len() > 1000 {
            return Err("Description cannot exceed 1000 characters".to_string());
        }

        // Validate rental_credits_per_day (0-20 credits/day)
        if let Some(rate) = rental_credits_per_day {
            if rate < 0 {
                return Err("Rental rate cannot be negative".to_string());
            }
            if rate > 20 {
                return Err("Rental rate cannot exceed 20 credits per day".to_string());
            }
        }

        // Validate deposit_credits (0-100 credits)
        if let Some(deposit) = deposit_credits {
            if deposit < 0 {
                return Err("Deposit cannot be negative".to_string());
            }
            if deposit > 100 {
                return Err("Deposit cannot exceed 100 credits".to_string());
            }
        }

        // Validate borrowing_duration_days (1-90 days)
        if let Some(duration) = borrowing_duration_days {
            if duration < 1 {
                return Err("Borrowing duration must be at least 1 day".to_string());
            }
            if duration > 90 {
                return Err("Borrowing duration cannot exceed 90 days".to_string());
            }
        }

        let now = Utc::now();

        Ok(Self {
            id: Uuid::new_v4(),
            owner_id,
            building_id,
            object_category,
            object_name,
            description,
            condition,
            is_available,
            rental_credits_per_day,
            deposit_credits,
            borrowing_duration_days,
            current_borrower_id: None,
            borrowed_at: None,
            due_back_at: None,
            photos,
            location_details,
            usage_instructions,
            created_at: now,
            updated_at: now,
        })
    }

    /// Update shared object information
    ///
    /// # Validation
    /// - Same validation rules as new()
    /// - Cannot update if currently borrowed
    #[allow(clippy::too_many_arguments)]
    pub fn update(
        &mut self,
        object_name: Option<String>,
        description: Option<String>,
        condition: Option<ObjectCondition>,
        is_available: Option<bool>,
        rental_credits_per_day: Option<Option<i32>>,
        deposit_credits: Option<Option<i32>>,
        borrowing_duration_days: Option<Option<i32>>,
        photos: Option<Option<Vec<String>>>,
        location_details: Option<Option<String>>,
        usage_instructions: Option<Option<String>>,
    ) -> Result<(), String> {
        // Cannot update if currently borrowed
        if self.is_borrowed() {
            return Err("Cannot update object while it is borrowed".to_string());
        }

        // Update object_name if provided
        if let Some(name) = object_name {
            if name.len() < 3 {
                return Err("Object name must be at least 3 characters".to_string());
            }
            if name.len() > 100 {
                return Err("Object name cannot exceed 100 characters".to_string());
            }
            self.object_name = name;
        }

        // Update description if provided
        if let Some(desc) = description {
            if desc.trim().is_empty() {
                return Err("Description cannot be empty".to_string());
            }
            if desc.len() > 1000 {
                return Err("Description cannot exceed 1000 characters".to_string());
            }
            self.description = desc;
        }

        // Update condition if provided
        if let Some(cond) = condition {
            self.condition = cond;
        }

        // Update availability if provided
        if let Some(available) = is_available {
            self.is_available = available;
        }

        // Update rental_credits_per_day if provided
        if let Some(rate_opt) = rental_credits_per_day {
            if let Some(rate) = rate_opt {
                if rate < 0 {
                    return Err("Rental rate cannot be negative".to_string());
                }
                if rate > 20 {
                    return Err("Rental rate cannot exceed 20 credits per day".to_string());
                }
            }
            self.rental_credits_per_day = rate_opt;
        }

        // Update deposit_credits if provided
        if let Some(deposit_opt) = deposit_credits {
            if let Some(deposit) = deposit_opt {
                if deposit < 0 {
                    return Err("Deposit cannot be negative".to_string());
                }
                if deposit > 100 {
                    return Err("Deposit cannot exceed 100 credits".to_string());
                }
            }
            self.deposit_credits = deposit_opt;
        }

        // Update borrowing_duration_days if provided
        if let Some(duration_opt) = borrowing_duration_days {
            if let Some(duration) = duration_opt {
                if duration < 1 {
                    return Err("Borrowing duration must be at least 1 day".to_string());
                }
                if duration > 90 {
                    return Err("Borrowing duration cannot exceed 90 days".to_string());
                }
            }
            self.borrowing_duration_days = duration_opt;
        }

        // Update photos if provided
        if let Some(photos_opt) = photos {
            self.photos = photos_opt;
        }

        // Update location_details if provided
        if let Some(location_opt) = location_details {
            self.location_details = location_opt;
        }

        // Update usage_instructions if provided
        if let Some(instructions_opt) = usage_instructions {
            self.usage_instructions = instructions_opt;
        }

        self.updated_at = Utc::now();
        Ok(())
    }

    /// Mark object as available for borrowing
    pub fn mark_available(&mut self) -> Result<(), String> {
        if self.is_borrowed() {
            return Err("Cannot mark as available while borrowed".to_string());
        }
        self.is_available = true;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Mark object as unavailable for borrowing
    pub fn mark_unavailable(&mut self) {
        self.is_available = false;
        self.updated_at = Utc::now();
    }

    /// Borrow object
    ///
    /// # Validation
    /// - Object must be available
    /// - Borrower cannot be the owner
    pub fn borrow(&mut self, borrower_id: Uuid, duration_days: Option<i32>) -> Result<(), String> {
        if !self.is_available {
            return Err("Object is not available for borrowing".to_string());
        }

        if self.is_borrowed() {
            return Err("Object is already borrowed".to_string());
        }

        if borrower_id == self.owner_id {
            return Err("Owner cannot borrow their own object".to_string());
        }

        let duration = duration_days
            .or(self.borrowing_duration_days)
            .unwrap_or(7); // Default 7 days

        if duration < 1 || duration > 90 {
            return Err("Borrowing duration must be between 1 and 90 days".to_string());
        }

        let now = Utc::now();
        let due_back = now + Duration::days(duration as i64);

        self.current_borrower_id = Some(borrower_id);
        self.borrowed_at = Some(now);
        self.due_back_at = Some(due_back);
        self.is_available = false;
        self.updated_at = now;

        Ok(())
    }

    /// Return borrowed object
    ///
    /// # Validation
    /// - Object must be borrowed
    /// - Only borrower can return
    pub fn return_object(&mut self, returner_id: Uuid) -> Result<(), String> {
        if !self.is_borrowed() {
            return Err("Object is not currently borrowed".to_string());
        }

        if self.current_borrower_id != Some(returner_id) {
            return Err("Only borrower can return object".to_string());
        }

        self.current_borrower_id = None;
        self.borrowed_at = None;
        self.due_back_at = None;
        self.is_available = true;
        self.updated_at = Utc::now();

        Ok(())
    }

    /// Check if object is currently borrowed
    pub fn is_borrowed(&self) -> bool {
        self.current_borrower_id.is_some()
    }

    /// Check if object is free (no rental fee)
    pub fn is_free(&self) -> bool {
        self.rental_credits_per_day.is_none() || self.rental_credits_per_day == Some(0)
    }

    /// Check if object is overdue
    pub fn is_overdue(&self) -> bool {
        if let Some(due_back) = self.due_back_at {
            Utc::now() > due_back
        } else {
            false
        }
    }

    /// Calculate total rental cost for actual borrowing period
    ///
    /// Returns (rental_cost, deposit)
    pub fn calculate_total_cost(&self) -> (i32, i32) {
        let rental_cost = if let (Some(borrowed), Some(rate)) =
            (self.borrowed_at, self.rental_credits_per_day)
        {
            let days_borrowed = (Utc::now() - borrowed).num_days() + 1; // At least 1 day
            (days_borrowed as i32) * rate
        } else {
            0
        };

        let deposit = self.deposit_credits.unwrap_or(0);

        (rental_cost, deposit)
    }

    /// Calculate days overdue
    pub fn days_overdue(&self) -> i32 {
        if let Some(due_back) = self.due_back_at {
            let overdue_duration = Utc::now() - due_back;
            if overdue_duration.num_days() > 0 {
                return overdue_duration.num_days() as i32;
            }
        }
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_shared_object_success() {
        let owner_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();

        let object = SharedObject::new(
            owner_id,
            building_id,
            SharedObjectCategory::Tools,
            "Power Drill".to_string(),
            "18V cordless drill with battery".to_string(),
            ObjectCondition::Good,
            true,
            Some(2),  // 2 credits/day
            Some(10), // 10 credits deposit
            Some(7),  // Max 7 days
            None,
            Some("Basement storage room".to_string()),
            Some("Charge battery before use".to_string()),
        );

        assert!(object.is_ok());
        let object = object.unwrap();
        assert_eq!(object.owner_id, owner_id);
        assert_eq!(object.object_category, SharedObjectCategory::Tools);
        assert!(object.is_available);
        assert!(!object.is_free());
        assert!(!object.is_borrowed());
    }

    #[test]
    fn test_object_name_too_short_fails() {
        let owner_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();

        let result = SharedObject::new(
            owner_id,
            building_id,
            SharedObjectCategory::Books,
            "AB".to_string(), // Too short (< 3 chars)
            "Description".to_string(),
            ObjectCondition::Excellent,
            true,
            None,
            None,
            None,
            None,
            None,
            None,
        );

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Object name must be at least 3 characters"
        );
    }

    #[test]
    fn test_rental_rate_exceeds_20_fails() {
        let owner_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();

        let result = SharedObject::new(
            owner_id,
            building_id,
            SharedObjectCategory::Electronics,
            "Projector".to_string(),
            "HD projector".to_string(),
            ObjectCondition::Excellent,
            true,
            Some(25), // Exceeds 20 credits/day
            None,
            None,
            None,
            None,
            None,
        );

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Rental rate cannot exceed 20 credits per day"
        );
    }

    #[test]
    fn test_borrow_object_success() {
        let owner_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let borrower_id = Uuid::new_v4();

        let mut object = SharedObject::new(
            owner_id,
            building_id,
            SharedObjectCategory::Sports,
            "Mountain Bike".to_string(),
            "26 inch mountain bike".to_string(),
            ObjectCondition::Good,
            true,
            Some(5),
            Some(20),
            Some(3), // Max 3 days
            None,
            None,
            None,
        )
        .unwrap();

        let result = object.borrow(borrower_id, Some(3));
        assert!(result.is_ok());
        assert!(object.is_borrowed());
        assert!(!object.is_available);
        assert_eq!(object.current_borrower_id, Some(borrower_id));
        assert!(object.borrowed_at.is_some());
        assert!(object.due_back_at.is_some());
    }

    #[test]
    fn test_owner_cannot_borrow_own_object() {
        let owner_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();

        let mut object = SharedObject::new(
            owner_id,
            building_id,
            SharedObjectCategory::Gardening,
            "Lawn Mower".to_string(),
            "Electric lawn mower".to_string(),
            ObjectCondition::Excellent,
            true,
            Some(3),
            None,
            Some(1),
            None,
            None,
            None,
        )
        .unwrap();

        let result = object.borrow(owner_id, Some(1)); // Owner tries to borrow
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Owner cannot borrow their own object");
    }

    #[test]
    fn test_return_object_success() {
        let owner_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let borrower_id = Uuid::new_v4();

        let mut object = SharedObject::new(
            owner_id,
            building_id,
            SharedObjectCategory::Kitchen,
            "Mixer".to_string(),
            "Stand mixer".to_string(),
            ObjectCondition::Good,
            true,
            None, // Free
            None,
            Some(7),
            None,
            None,
            None,
        )
        .unwrap();

        object.borrow(borrower_id, Some(7)).unwrap();
        assert!(object.is_borrowed());

        let result = object.return_object(borrower_id);
        assert!(result.is_ok());
        assert!(!object.is_borrowed());
        assert!(object.is_available);
        assert_eq!(object.current_borrower_id, None);
    }

    #[test]
    fn test_only_borrower_can_return() {
        let owner_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let borrower_id = Uuid::new_v4();
        let other_user_id = Uuid::new_v4();

        let mut object = SharedObject::new(
            owner_id,
            building_id,
            SharedObjectCategory::Baby,
            "Stroller".to_string(),
            "Baby stroller".to_string(),
            ObjectCondition::Fair,
            true,
            None,
            None,
            Some(14),
            None,
            None,
            None,
        )
        .unwrap();

        object.borrow(borrower_id, Some(14)).unwrap();

        let result = object.return_object(other_user_id); // Wrong user
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Only borrower can return object");
    }

    #[test]
    fn test_is_free() {
        let owner_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();

        // Free (None)
        let object1 = SharedObject::new(
            owner_id,
            building_id,
            SharedObjectCategory::Books,
            "Book Title".to_string(),
            "Description".to_string(),
            ObjectCondition::Good,
            true,
            None, // Free
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap();
        assert!(object1.is_free());

        // Free (0 credits)
        let object2 = SharedObject::new(
            owner_id,
            building_id,
            SharedObjectCategory::Books,
            "Book Title".to_string(),
            "Description".to_string(),
            ObjectCondition::Good,
            true,
            Some(0), // Explicitly 0 credits
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap();
        assert!(object2.is_free());

        // Not free
        let object3 = SharedObject::new(
            owner_id,
            building_id,
            SharedObjectCategory::Electronics,
            "Camera".to_string(),
            "DSLR camera".to_string(),
            ObjectCondition::Excellent,
            true,
            Some(10),
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap();
        assert!(!object3.is_free());
    }

    #[test]
    fn test_calculate_total_cost() {
        let owner_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let borrower_id = Uuid::new_v4();

        let mut object = SharedObject::new(
            owner_id,
            building_id,
            SharedObjectCategory::Tools,
            "Chainsaw".to_string(),
            "Gas chainsaw".to_string(),
            ObjectCondition::Good,
            true,
            Some(5),  // 5 credits/day
            Some(30), // 30 credits deposit
            Some(3),
            None,
            None,
            None,
        )
        .unwrap();

        object.borrow(borrower_id, Some(3)).unwrap();

        let (rental_cost, deposit) = object.calculate_total_cost();
        assert_eq!(deposit, 30);
        assert!(rental_cost >= 5); // At least 1 day * 5 credits
    }

    #[test]
    fn test_cannot_update_while_borrowed() {
        let owner_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let borrower_id = Uuid::new_v4();

        let mut object = SharedObject::new(
            owner_id,
            building_id,
            SharedObjectCategory::Other,
            "Item".to_string(),
            "Description".to_string(),
            ObjectCondition::Good,
            true,
            None,
            None,
            Some(7),
            None,
            None,
            None,
        )
        .unwrap();

        object.borrow(borrower_id, Some(7)).unwrap();

        let result = object.update(
            Some("New Name".to_string()),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        );

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Cannot update object while it is borrowed"
        );
    }

    #[test]
    fn test_mark_available_while_borrowed_fails() {
        let owner_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let borrower_id = Uuid::new_v4();

        let mut object = SharedObject::new(
            owner_id,
            building_id,
            SharedObjectCategory::Sports,
            "Tennis Racket".to_string(),
            "Professional racket".to_string(),
            ObjectCondition::Excellent,
            true,
            Some(2),
            None,
            Some(7),
            None,
            None,
            None,
        )
        .unwrap();

        object.borrow(borrower_id, Some(7)).unwrap();

        let result = object.mark_available();
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Cannot mark as available while borrowed"
        );
    }
}
