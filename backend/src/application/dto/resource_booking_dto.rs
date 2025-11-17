use crate::domain::entities::{BookingStatus, RecurringPattern, ResourceBooking, ResourceType};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// DTO for creating a new resource booking
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateResourceBookingDto {
    pub building_id: Uuid,
    pub resource_type: ResourceType,
    pub resource_name: String, // e.g., "Meeting Room A", "Laundry Room 1st Floor"
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    #[serde(default)]
    pub recurring_pattern: RecurringPattern, // Defaults to None
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recurrence_end_date: Option<DateTime<Utc>>,
    /// Optional max duration in hours (uses default if not provided)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_duration_hours: Option<i64>,
    /// Optional max advance booking in days (uses default if not provided)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_advance_days: Option<i64>,
}

/// DTO for updating booking details (resource_name, notes)
///
/// Time changes require cancellation and rebooking to ensure conflict detection.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdateResourceBookingDto {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

/// Response DTO for ResourceBooking with enriched data
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ResourceBookingResponseDto {
    pub id: Uuid,
    pub building_id: Uuid,
    pub resource_type: ResourceType,
    pub resource_name: String,
    pub booked_by: Uuid,
    pub booked_by_name: String, // Enriched: Owner full name
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub status: BookingStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    pub recurring_pattern: RecurringPattern,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recurrence_end_date: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    // Computed fields
    pub duration_hours: f64,
    pub is_active: bool,
    pub is_past: bool,
    pub is_future: bool,
    pub is_modifiable: bool,
    pub is_recurring: bool,
}

impl ResourceBookingResponseDto {
    /// Convert ResourceBooking entity to response DTO with owner name enrichment
    pub fn from_entity(booking: ResourceBooking, booked_by_name: String) -> Self {
        Self {
            id: booking.id,
            building_id: booking.building_id,
            resource_type: booking.resource_type.clone(),
            resource_name: booking.resource_name.clone(),
            booked_by: booking.booked_by,
            booked_by_name,
            start_time: booking.start_time,
            end_time: booking.end_time,
            status: booking.status.clone(),
            notes: booking.notes.clone(),
            recurring_pattern: booking.recurring_pattern.clone(),
            recurrence_end_date: booking.recurrence_end_date,
            created_at: booking.created_at,
            updated_at: booking.updated_at,
            // Computed fields
            duration_hours: booking.duration_hours(),
            is_active: booking.is_active(),
            is_past: booking.is_past(),
            is_future: booking.is_future(),
            is_modifiable: booking.is_modifiable(),
            is_recurring: booking.is_recurring(),
        }
    }
}

/// DTO for booking statistics
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BookingStatisticsDto {
    pub building_id: Uuid,
    pub total_bookings: i64,
    pub confirmed_bookings: i64,
    pub pending_bookings: i64,
    pub completed_bookings: i64,
    pub cancelled_bookings: i64,
    pub no_show_bookings: i64,
    pub active_bookings: i64,         // Currently in progress
    pub upcoming_bookings: i64,       // Future bookings
    pub total_hours_booked: f64,      // Sum of all booking durations
    pub most_popular_resource: Option<String>, // Most booked resource name
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_dto_serialization() {
        let dto = CreateResourceBookingDto {
            building_id: Uuid::new_v4(),
            resource_type: ResourceType::MeetingRoom,
            resource_name: "Meeting Room A".to_string(),
            start_time: Utc::now() + chrono::Duration::hours(2),
            end_time: Utc::now() + chrono::Duration::hours(4),
            notes: Some("Team meeting".to_string()),
            recurring_pattern: RecurringPattern::None,
            recurrence_end_date: None,
            max_duration_hours: None,
            max_advance_days: None,
        };

        let json = serde_json::to_string(&dto).unwrap();
        assert!(json.contains("Meeting Room A"));
    }

    #[test]
    fn test_response_dto_from_entity() {
        let building_id = Uuid::new_v4();
        let booked_by = Uuid::new_v4();
        let start_time = Utc::now() + chrono::Duration::hours(2);
        let end_time = start_time + chrono::Duration::hours(2);

        let booking = ResourceBooking::new(
            building_id,
            ResourceType::MeetingRoom,
            "Meeting Room A".to_string(),
            booked_by,
            start_time,
            end_time,
            Some("Team meeting".to_string()),
            RecurringPattern::None,
            None,
            None,
            None,
        )
        .unwrap();

        let dto = ResourceBookingResponseDto::from_entity(booking, "John Doe".to_string());

        assert_eq!(dto.resource_name, "Meeting Room A");
        assert_eq!(dto.booked_by_name, "John Doe");
        assert_eq!(dto.duration_hours, 2.0);
        assert!(dto.is_future);
        assert!(!dto.is_past);
        assert!(dto.is_modifiable);
    }
}
