use crate::domain::entities::{ObjectCondition, SharedObject, SharedObjectCategory};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// DTO for creating a new shared object
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateSharedObjectDto {
    pub building_id: Uuid,
    pub object_category: SharedObjectCategory,
    pub object_name: String,
    pub description: String,
    pub condition: ObjectCondition,
    pub is_available: bool,
    pub rental_credits_per_day: Option<i32>, // 0-20 (SEL integration)
    pub deposit_credits: Option<i32>,        // 0-100
    pub borrowing_duration_days: Option<i32>, // 1-90
    pub photos: Option<Vec<String>>,
    pub location_details: Option<String>,
    pub usage_instructions: Option<String>,
}

/// DTO for updating a shared object
#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateSharedObjectDto {
    pub object_name: Option<String>,
    pub description: Option<String>,
    pub condition: Option<ObjectCondition>,
    pub is_available: Option<bool>,
    pub rental_credits_per_day: Option<Option<i32>>,
    pub deposit_credits: Option<Option<i32>>,
    pub borrowing_duration_days: Option<Option<i32>>,
    pub photos: Option<Option<Vec<String>>>,
    pub location_details: Option<Option<String>>,
    pub usage_instructions: Option<Option<String>>,
}

/// DTO for borrowing an object
#[derive(Debug, Serialize, Deserialize)]
pub struct BorrowObjectDto {
    pub duration_days: Option<i32>, // Override default duration
}

/// Complete shared object response with owner/borrower information
#[derive(Debug, Serialize, Clone)]
pub struct SharedObjectResponseDto {
    pub id: Uuid,
    pub owner_id: Uuid,
    pub owner_name: String, // Enriched from Owner
    pub building_id: Uuid,
    pub object_category: SharedObjectCategory,
    pub object_name: String,
    pub description: String,
    pub condition: ObjectCondition,
    pub is_available: bool,
    pub rental_credits_per_day: Option<i32>,
    pub deposit_credits: Option<i32>,
    pub borrowing_duration_days: Option<i32>,
    pub current_borrower_id: Option<Uuid>,
    pub current_borrower_name: Option<String>, // Enriched from Owner
    pub borrowed_at: Option<DateTime<Utc>>,
    pub due_back_at: Option<DateTime<Utc>>,
    pub photos: Option<Vec<String>>,
    pub location_details: Option<String>,
    pub usage_instructions: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    // Computed fields
    pub is_free: bool,
    pub is_borrowed: bool,
    pub is_overdue: bool,
    pub days_overdue: i32,
}

impl SharedObjectResponseDto {
    /// Create from SharedObject with owner/borrower name enrichment
    pub fn from_shared_object(
        object: SharedObject,
        owner_name: String,
        borrower_name: Option<String>,
    ) -> Self {
        let is_free = object.is_free();
        let is_borrowed = object.is_borrowed();
        let is_overdue = object.is_overdue();
        let days_overdue = object.days_overdue();

        Self {
            id: object.id,
            owner_id: object.owner_id,
            owner_name,
            building_id: object.building_id,
            object_category: object.object_category,
            object_name: object.object_name,
            description: object.description,
            condition: object.condition,
            is_available: object.is_available,
            rental_credits_per_day: object.rental_credits_per_day,
            deposit_credits: object.deposit_credits,
            borrowing_duration_days: object.borrowing_duration_days,
            current_borrower_id: object.current_borrower_id,
            current_borrower_name: borrower_name,
            borrowed_at: object.borrowed_at,
            due_back_at: object.due_back_at,
            photos: object.photos,
            location_details: object.location_details,
            usage_instructions: object.usage_instructions,
            created_at: object.created_at,
            updated_at: object.updated_at,
            is_free,
            is_borrowed,
            is_overdue,
            days_overdue,
        }
    }
}

/// Summary shared object view for lists
#[derive(Debug, Serialize, Clone)]
pub struct SharedObjectSummaryDto {
    pub id: Uuid,
    pub owner_id: Uuid,
    pub owner_name: String, // Enriched from Owner
    pub building_id: Uuid,
    pub object_category: SharedObjectCategory,
    pub object_name: String,
    pub condition: ObjectCondition,
    pub is_available: bool,
    pub rental_credits_per_day: Option<i32>,
    pub deposit_credits: Option<i32>,
    pub current_borrower_id: Option<Uuid>,
    pub due_back_at: Option<DateTime<Utc>>,
    pub is_free: bool,
    pub is_borrowed: bool,
    pub is_overdue: bool,
}

impl SharedObjectSummaryDto {
    /// Create from SharedObject with owner name enrichment
    pub fn from_shared_object(object: SharedObject, owner_name: String) -> Self {
        let is_free = object.is_free();
        let is_borrowed = object.is_borrowed();
        let is_overdue = object.is_overdue();

        Self {
            id: object.id,
            owner_id: object.owner_id,
            owner_name,
            building_id: object.building_id,
            object_category: object.object_category,
            object_name: object.object_name,
            condition: object.condition,
            is_available: object.is_available,
            rental_credits_per_day: object.rental_credits_per_day,
            deposit_credits: object.deposit_credits,
            current_borrower_id: object.current_borrower_id,
            due_back_at: object.due_back_at,
            is_free,
            is_borrowed,
            is_overdue,
        }
    }
}

/// Statistics for building shared objects
#[derive(Debug, Serialize)]
pub struct SharedObjectStatisticsDto {
    pub total_objects: i64,
    pub available_objects: i64,
    pub borrowed_objects: i64,
    pub overdue_objects: i64,
    pub free_objects: i64,
    pub paid_objects: i64,
    pub objects_by_category: Vec<CategoryObjectCount>,
}

/// Category count for statistics
#[derive(Debug, Serialize)]
pub struct CategoryObjectCount {
    pub category: SharedObjectCategory,
    pub count: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shared_object_response_dto_from_shared_object() {
        let object = SharedObject::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            SharedObjectCategory::Tools,
            "Power Drill".to_string(),
            "18V cordless drill".to_string(),
            ObjectCondition::Good,
            true,
            Some(2),
            Some(10),
            Some(7),
            None,
            None,
            None,
        )
        .unwrap();

        let dto = SharedObjectResponseDto::from_shared_object(
            object.clone(),
            "John Doe".to_string(),
            None,
        );

        assert_eq!(dto.owner_name, "John Doe");
        assert_eq!(dto.object_name, "Power Drill");
        assert!(!dto.is_free);
        assert!(!dto.is_borrowed);
        assert!(!dto.is_overdue);
    }

    #[test]
    fn test_shared_object_summary_dto_from_shared_object() {
        let object = SharedObject::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            SharedObjectCategory::Books,
            "Book Title".to_string(),
            "Description".to_string(),
            ObjectCondition::Excellent,
            true,
            None, // Free
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap();

        let dto =
            SharedObjectSummaryDto::from_shared_object(object.clone(), "Jane Smith".to_string());

        assert_eq!(dto.owner_name, "Jane Smith");
        assert_eq!(dto.object_name, "Book Title");
        assert!(dto.is_free);
        assert!(!dto.is_borrowed);
    }
}
