use crate::application::dto::{
    BookingStatisticsDto, CreateResourceBookingDto, ResourceBookingResponseDto,
    UpdateResourceBookingDto,
};
use crate::application::ports::{OwnerRepository, ResourceBookingRepository};
use crate::domain::entities::{BookingStatus, ResourceBooking, ResourceType};
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

/// Use cases for resource booking operations
///
/// Orchestrates business logic for community space bookings with conflict detection,
/// owner name enrichment, and authorization checks.
pub struct ResourceBookingUseCases {
    booking_repo: Arc<dyn ResourceBookingRepository>,
    owner_repo: Arc<dyn OwnerRepository>,
}

impl ResourceBookingUseCases {
    pub fn new(
        booking_repo: Arc<dyn ResourceBookingRepository>,
        owner_repo: Arc<dyn OwnerRepository>,
    ) -> Self {
        Self {
            booking_repo,
            owner_repo,
        }
    }

    /// Create a new resource booking with conflict detection
    ///
    /// # Steps
    /// 1. Create booking entity (validates business rules)
    /// 2. Check for conflicts (overlapping bookings for same resource)
    /// 3. Persist booking
    /// 4. Enrich response with owner name
    ///
    /// # Authorization
    /// - Any authenticated owner can create bookings
    pub async fn create_booking(
        &self,
        booked_by: Uuid,
        dto: CreateResourceBookingDto,
    ) -> Result<ResourceBookingResponseDto, String> {
        // Create booking entity (validates business rules in constructor)
        let booking = ResourceBooking::new(
            dto.building_id,
            dto.resource_type.clone(),
            dto.resource_name.clone(),
            booked_by,
            dto.start_time,
            dto.end_time,
            dto.notes.clone(),
            dto.recurring_pattern.clone(),
            dto.recurrence_end_date,
            dto.max_duration_hours,
            dto.max_advance_days,
        )?;

        // Check for conflicts (overlapping bookings for same resource)
        let conflicts = self
            .booking_repo
            .find_conflicts(
                dto.building_id,
                dto.resource_type,
                &dto.resource_name,
                dto.start_time,
                dto.end_time,
                None, // No exclusion for new bookings
            )
            .await?;

        if !conflicts.is_empty() {
            return Err(format!(
                "Booking conflicts with {} existing booking(s) for this resource",
                conflicts.len()
            ));
        }

        // Persist booking
        let created = self.booking_repo.create(&booking).await?;

        // Enrich with owner name
        self.enrich_booking_response(created).await
    }

    /// Get booking by ID with owner name enrichment
    pub async fn get_booking(&self, booking_id: Uuid) -> Result<ResourceBookingResponseDto, String> {
        let booking = self
            .booking_repo
            .find_by_id(booking_id)
            .await?
            .ok_or("Booking not found".to_string())?;

        self.enrich_booking_response(booking).await
    }

    /// List all bookings for a building
    pub async fn list_building_bookings(
        &self,
        building_id: Uuid,
    ) -> Result<Vec<ResourceBookingResponseDto>, String> {
        let bookings = self.booking_repo.find_by_building(building_id).await?;
        self.enrich_bookings_response(bookings).await
    }

    /// List bookings by building and resource type
    pub async fn list_by_resource_type(
        &self,
        building_id: Uuid,
        resource_type: ResourceType,
    ) -> Result<Vec<ResourceBookingResponseDto>, String> {
        let bookings = self
            .booking_repo
            .find_by_building_and_resource_type(building_id, resource_type)
            .await?;
        self.enrich_bookings_response(bookings).await
    }

    /// List bookings for a specific resource (building + type + name)
    pub async fn list_by_resource(
        &self,
        building_id: Uuid,
        resource_type: ResourceType,
        resource_name: String,
    ) -> Result<Vec<ResourceBookingResponseDto>, String> {
        let bookings = self
            .booking_repo
            .find_by_resource(building_id, resource_type, &resource_name)
            .await?;
        self.enrich_bookings_response(bookings).await
    }

    /// List bookings by user (owner who made the booking)
    pub async fn list_user_bookings(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<ResourceBookingResponseDto>, String> {
        let bookings = self.booking_repo.find_by_user(user_id).await?;
        self.enrich_bookings_response(bookings).await
    }

    /// List bookings by user and status
    pub async fn list_user_bookings_by_status(
        &self,
        user_id: Uuid,
        status: BookingStatus,
    ) -> Result<Vec<ResourceBookingResponseDto>, String> {
        let bookings = self
            .booking_repo
            .find_by_user_and_status(user_id, status)
            .await?;
        self.enrich_bookings_response(bookings).await
    }

    /// List bookings by building and status
    pub async fn list_building_bookings_by_status(
        &self,
        building_id: Uuid,
        status: BookingStatus,
    ) -> Result<Vec<ResourceBookingResponseDto>, String> {
        let bookings = self
            .booking_repo
            .find_by_building_and_status(building_id, status)
            .await?;
        self.enrich_bookings_response(bookings).await
    }

    /// List upcoming bookings (future, confirmed or pending)
    pub async fn list_upcoming_bookings(
        &self,
        building_id: Uuid,
        limit: Option<i64>,
    ) -> Result<Vec<ResourceBookingResponseDto>, String> {
        let bookings = self.booking_repo.find_upcoming(building_id, limit).await?;
        self.enrich_bookings_response(bookings).await
    }

    /// List active bookings (currently in progress)
    pub async fn list_active_bookings(
        &self,
        building_id: Uuid,
    ) -> Result<Vec<ResourceBookingResponseDto>, String> {
        let bookings = self.booking_repo.find_active(building_id).await?;
        self.enrich_bookings_response(bookings).await
    }

    /// List past bookings
    pub async fn list_past_bookings(
        &self,
        building_id: Uuid,
        limit: Option<i64>,
    ) -> Result<Vec<ResourceBookingResponseDto>, String> {
        let bookings = self.booking_repo.find_past(building_id, limit).await?;
        self.enrich_bookings_response(bookings).await
    }

    /// Update booking details (resource_name, notes)
    ///
    /// Time changes require cancellation and rebooking to ensure conflict detection.
    ///
    /// # Authorization
    /// - Only booking owner can update their booking
    pub async fn update_booking(
        &self,
        booking_id: Uuid,
        updater_id: Uuid,
        dto: UpdateResourceBookingDto,
    ) -> Result<ResourceBookingResponseDto, String> {
        let mut booking = self
            .booking_repo
            .find_by_id(booking_id)
            .await?
            .ok_or("Booking not found".to_string())?;

        // Authorization: Only booking owner can update
        if booking.booked_by != updater_id {
            return Err("Only the booking owner can update this booking".to_string());
        }

        // Update details (validates business rules)
        booking.update_details(dto.resource_name, dto.notes)?;

        // Persist changes
        let updated = self.booking_repo.update(&booking).await?;

        // Enrich with owner name
        self.enrich_booking_response(updated).await
    }

    /// Cancel a booking
    ///
    /// # Authorization
    /// - Only booking owner can cancel their booking
    pub async fn cancel_booking(
        &self,
        booking_id: Uuid,
        canceller_id: Uuid,
    ) -> Result<ResourceBookingResponseDto, String> {
        let mut booking = self
            .booking_repo
            .find_by_id(booking_id)
            .await?
            .ok_or("Booking not found".to_string())?;

        // Cancel booking (validates authorization and state)
        booking.cancel(canceller_id)?;

        // Persist changes
        let updated = self.booking_repo.update(&booking).await?;

        // Enrich with owner name
        self.enrich_booking_response(updated).await
    }

    /// Complete a booking (typically auto-called after end_time)
    ///
    /// # Authorization
    /// - Admin only (for manual completion)
    pub async fn complete_booking(
        &self,
        booking_id: Uuid,
    ) -> Result<ResourceBookingResponseDto, String> {
        let mut booking = self
            .booking_repo
            .find_by_id(booking_id)
            .await?
            .ok_or("Booking not found".to_string())?;

        // Complete booking (validates state)
        booking.complete()?;

        // Persist changes
        let updated = self.booking_repo.update(&booking).await?;

        // Enrich with owner name
        self.enrich_booking_response(updated).await
    }

    /// Mark booking as no-show
    ///
    /// # Authorization
    /// - Admin only
    pub async fn mark_no_show(
        &self,
        booking_id: Uuid,
    ) -> Result<ResourceBookingResponseDto, String> {
        let mut booking = self
            .booking_repo
            .find_by_id(booking_id)
            .await?
            .ok_or("Booking not found".to_string())?;

        // Mark as no-show (validates state)
        booking.mark_no_show()?;

        // Persist changes
        let updated = self.booking_repo.update(&booking).await?;

        // Enrich with owner name
        self.enrich_booking_response(updated).await
    }

    /// Confirm a pending booking
    ///
    /// # Authorization
    /// - Admin only (for approval workflow)
    pub async fn confirm_booking(
        &self,
        booking_id: Uuid,
    ) -> Result<ResourceBookingResponseDto, String> {
        let mut booking = self
            .booking_repo
            .find_by_id(booking_id)
            .await?
            .ok_or("Booking not found".to_string())?;

        // Confirm booking (validates state)
        booking.confirm()?;

        // Persist changes
        let updated = self.booking_repo.update(&booking).await?;

        // Enrich with owner name
        self.enrich_booking_response(updated).await
    }

    /// Delete a booking
    ///
    /// # Authorization
    /// - Only booking owner can delete their booking
    /// - Or admin for any booking
    pub async fn delete_booking(
        &self,
        booking_id: Uuid,
        deleter_id: Uuid,
    ) -> Result<(), String> {
        let booking = self
            .booking_repo
            .find_by_id(booking_id)
            .await?
            .ok_or("Booking not found".to_string())?;

        // Authorization: Only booking owner can delete
        if booking.booked_by != deleter_id {
            return Err("Only the booking owner can delete this booking".to_string());
        }

        self.booking_repo.delete(booking_id).await
    }

    /// Check for booking conflicts
    ///
    /// Useful for frontend to preview conflicts before creating booking.
    ///
    /// Returns list of conflicting bookings with owner names.
    pub async fn check_conflicts(
        &self,
        building_id: Uuid,
        resource_type: ResourceType,
        resource_name: String,
        start_time: chrono::DateTime<Utc>,
        end_time: chrono::DateTime<Utc>,
        exclude_booking_id: Option<Uuid>,
    ) -> Result<Vec<ResourceBookingResponseDto>, String> {
        let conflicts = self
            .booking_repo
            .find_conflicts(
                building_id,
                resource_type,
                &resource_name,
                start_time,
                end_time,
                exclude_booking_id,
            )
            .await?;

        self.enrich_bookings_response(conflicts).await
    }

    /// Get booking statistics for a building
    pub async fn get_statistics(&self, building_id: Uuid) -> Result<BookingStatisticsDto, String> {
        self.booking_repo.get_statistics(building_id).await
    }

    /// Helper: Enrich single booking with owner name
    async fn enrich_booking_response(
        &self,
        booking: ResourceBooking,
    ) -> Result<ResourceBookingResponseDto, String> {
        // Fetch owner to get full name
        let owner = self
            .owner_repo
            .find_by_id(booking.booked_by)
            .await?
            .ok_or("Booking owner not found".to_string())?;

        let booked_by_name = format!("{} {}", owner.first_name, owner.last_name);

        Ok(ResourceBookingResponseDto::from_entity(
            booking,
            booked_by_name,
        ))
    }

    /// Helper: Enrich multiple bookings with owner names
    async fn enrich_bookings_response(
        &self,
        bookings: Vec<ResourceBooking>,
    ) -> Result<Vec<ResourceBookingResponseDto>, String> {
        let mut result = Vec::with_capacity(bookings.len());

        for booking in bookings {
            let enriched = self.enrich_booking_response(booking).await?;
            result.push(enriched);
        }

        Ok(result)
    }
}
