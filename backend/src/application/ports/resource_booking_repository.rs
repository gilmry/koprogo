use crate::application::dto::BookingStatisticsDto;
use crate::domain::entities::{BookingStatus, ResourceBooking, ResourceType};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Repository trait for ResourceBooking persistence operations
///
/// Defines the contract for storing and retrieving resource bookings.
/// Implementations must handle conflict detection, statistics, and filtering.
#[async_trait]
pub trait ResourceBookingRepository: Send + Sync {
    /// Create a new booking
    async fn create(&self, booking: &ResourceBooking) -> Result<ResourceBooking, String>;

    /// Find booking by ID
    async fn find_by_id(&self, id: Uuid) -> Result<Option<ResourceBooking>, String>;

    /// Find all bookings for a building
    async fn find_by_building(&self, building_id: Uuid) -> Result<Vec<ResourceBooking>, String>;

    /// Find all bookings for a building with a specific resource type
    async fn find_by_building_and_resource_type(
        &self,
        building_id: Uuid,
        resource_type: ResourceType,
    ) -> Result<Vec<ResourceBooking>, String>;

    /// Find all bookings for a specific resource (building_id, resource_type, resource_name)
    async fn find_by_resource(
        &self,
        building_id: Uuid,
        resource_type: ResourceType,
        resource_name: &str,
    ) -> Result<Vec<ResourceBooking>, String>;

    /// Find bookings by user (owner who made the booking)
    async fn find_by_user(&self, user_id: Uuid) -> Result<Vec<ResourceBooking>, String>;

    /// Find bookings by user and status
    async fn find_by_user_and_status(
        &self,
        user_id: Uuid,
        status: BookingStatus,
    ) -> Result<Vec<ResourceBooking>, String>;

    /// Find bookings by building and status
    async fn find_by_building_and_status(
        &self,
        building_id: Uuid,
        status: BookingStatus,
    ) -> Result<Vec<ResourceBooking>, String>;

    /// Find upcoming bookings (start_time in the future, status Confirmed or Pending)
    async fn find_upcoming(
        &self,
        building_id: Uuid,
        limit: Option<i64>,
    ) -> Result<Vec<ResourceBooking>, String>;

    /// Find active bookings (currently in progress: now >= start_time AND now < end_time)
    async fn find_active(&self, building_id: Uuid) -> Result<Vec<ResourceBooking>, String>;

    /// Find past bookings (end_time < now)
    async fn find_past(
        &self,
        building_id: Uuid,
        limit: Option<i64>,
    ) -> Result<Vec<ResourceBooking>, String>;

    /// Find conflicting bookings for a time range and resource
    ///
    /// Returns bookings that overlap with the given time range for the same resource.
    /// Excludes cancelled, completed, and no-show bookings.
    /// Excludes the booking with exclude_booking_id (useful for update conflict checks).
    ///
    /// Conflict logic: start1 < end2 AND start2 < end1
    async fn find_conflicts(
        &self,
        building_id: Uuid,
        resource_type: ResourceType,
        resource_name: &str,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
        exclude_booking_id: Option<Uuid>,
    ) -> Result<Vec<ResourceBooking>, String>;

    /// Update booking
    async fn update(&self, booking: &ResourceBooking) -> Result<ResourceBooking, String>;

    /// Delete booking
    async fn delete(&self, id: Uuid) -> Result<(), String>;

    /// Count total bookings for a building
    async fn count_by_building(&self, building_id: Uuid) -> Result<i64, String>;

    /// Count bookings by building and status
    async fn count_by_building_and_status(
        &self,
        building_id: Uuid,
        status: BookingStatus,
    ) -> Result<i64, String>;

    /// Count bookings by resource
    async fn count_by_resource(
        &self,
        building_id: Uuid,
        resource_type: ResourceType,
        resource_name: &str,
    ) -> Result<i64, String>;

    /// Get booking statistics for a building
    async fn get_statistics(&self, building_id: Uuid) -> Result<BookingStatisticsDto, String>;
}
