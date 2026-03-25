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

    /// Resolve user_id to owner via organization lookup
    async fn resolve_owner(
        &self,
        user_id: Uuid,
        organization_id: Uuid,
    ) -> Result<crate::domain::entities::Owner, String> {
        self.owner_repo
            .find_by_user_id_and_organization(user_id, organization_id)
            .await?
            .ok_or_else(|| "Owner not found for this user in the organization".to_string())
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
        user_id: Uuid,
        organization_id: Uuid,
        dto: CreateResourceBookingDto,
    ) -> Result<ResourceBookingResponseDto, String> {
        let owner = self.resolve_owner(user_id, organization_id).await?;
        let booked_by = owner.id;
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
    pub async fn get_booking(
        &self,
        booking_id: Uuid,
    ) -> Result<ResourceBookingResponseDto, String> {
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
        organization_id: Uuid,
    ) -> Result<Vec<ResourceBookingResponseDto>, String> {
        let owner = self.resolve_owner(user_id, organization_id).await?;
        let bookings = self.booking_repo.find_by_user(owner.id).await?;
        self.enrich_bookings_response(bookings).await
    }

    /// List bookings by user and status
    pub async fn list_user_bookings_by_status(
        &self,
        user_id: Uuid,
        organization_id: Uuid,
        status: BookingStatus,
    ) -> Result<Vec<ResourceBookingResponseDto>, String> {
        let owner = self.resolve_owner(user_id, organization_id).await?;
        let bookings = self
            .booking_repo
            .find_by_user_and_status(owner.id, status)
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
        user_id: Uuid,
        organization_id: Uuid,
        dto: UpdateResourceBookingDto,
    ) -> Result<ResourceBookingResponseDto, String> {
        let owner = self.resolve_owner(user_id, organization_id).await?;
        let mut booking = self
            .booking_repo
            .find_by_id(booking_id)
            .await?
            .ok_or("Booking not found".to_string())?;

        // Authorization: Only booking owner can update
        if booking.booked_by != owner.id {
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
        user_id: Uuid,
        organization_id: Uuid,
    ) -> Result<ResourceBookingResponseDto, String> {
        let owner = self.resolve_owner(user_id, organization_id).await?;
        let mut booking = self
            .booking_repo
            .find_by_id(booking_id)
            .await?
            .ok_or("Booking not found".to_string())?;

        // Cancel booking (validates authorization and state)
        booking.cancel(owner.id)?;

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
        user_id: Uuid,
        organization_id: Uuid,
    ) -> Result<(), String> {
        let owner = self.resolve_owner(user_id, organization_id).await?;
        let booking = self
            .booking_repo
            .find_by_id(booking_id)
            .await?
            .ok_or("Booking not found".to_string())?;

        // Authorization: Only booking owner can delete
        if booking.booked_by != owner.id {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::dto::{BookingStatisticsDto, OwnerFilters, PageRequest};
    use crate::application::ports::{OwnerRepository, ResourceBookingRepository};
    use crate::domain::entities::{
        BookingStatus, Owner, RecurringPattern, ResourceBooking, ResourceType,
    };
    use async_trait::async_trait;
    use chrono::{DateTime, Duration, Utc};
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};
    use uuid::Uuid;

    // ── Mock ResourceBookingRepository ──────────────────────────────────────
    struct MockBookingRepo {
        bookings: Mutex<HashMap<Uuid, ResourceBooking>>,
    }

    impl MockBookingRepo {
        fn new() -> Self {
            Self {
                bookings: Mutex::new(HashMap::new()),
            }
        }
    }

    #[async_trait]
    impl ResourceBookingRepository for MockBookingRepo {
        async fn create(&self, booking: &ResourceBooking) -> Result<ResourceBooking, String> {
            let mut map = self.bookings.lock().unwrap();
            map.insert(booking.id, booking.clone());
            Ok(booking.clone())
        }

        async fn find_by_id(&self, id: Uuid) -> Result<Option<ResourceBooking>, String> {
            let map = self.bookings.lock().unwrap();
            Ok(map.get(&id).cloned())
        }

        async fn find_by_building(
            &self,
            building_id: Uuid,
        ) -> Result<Vec<ResourceBooking>, String> {
            let map = self.bookings.lock().unwrap();
            Ok(map
                .values()
                .filter(|b| b.building_id == building_id)
                .cloned()
                .collect())
        }

        async fn find_by_building_and_resource_type(
            &self,
            building_id: Uuid,
            resource_type: ResourceType,
        ) -> Result<Vec<ResourceBooking>, String> {
            let map = self.bookings.lock().unwrap();
            Ok(map
                .values()
                .filter(|b| b.building_id == building_id && b.resource_type == resource_type)
                .cloned()
                .collect())
        }

        async fn find_by_resource(
            &self,
            building_id: Uuid,
            resource_type: ResourceType,
            resource_name: &str,
        ) -> Result<Vec<ResourceBooking>, String> {
            let map = self.bookings.lock().unwrap();
            Ok(map
                .values()
                .filter(|b| {
                    b.building_id == building_id
                        && b.resource_type == resource_type
                        && b.resource_name == resource_name
                })
                .cloned()
                .collect())
        }

        async fn find_by_user(&self, user_id: Uuid) -> Result<Vec<ResourceBooking>, String> {
            let map = self.bookings.lock().unwrap();
            Ok(map
                .values()
                .filter(|b| b.booked_by == user_id)
                .cloned()
                .collect())
        }

        async fn find_by_user_and_status(
            &self,
            user_id: Uuid,
            status: BookingStatus,
        ) -> Result<Vec<ResourceBooking>, String> {
            let map = self.bookings.lock().unwrap();
            Ok(map
                .values()
                .filter(|b| b.booked_by == user_id && b.status == status)
                .cloned()
                .collect())
        }

        async fn find_by_building_and_status(
            &self,
            building_id: Uuid,
            status: BookingStatus,
        ) -> Result<Vec<ResourceBooking>, String> {
            let map = self.bookings.lock().unwrap();
            Ok(map
                .values()
                .filter(|b| b.building_id == building_id && b.status == status)
                .cloned()
                .collect())
        }

        async fn find_upcoming(
            &self,
            building_id: Uuid,
            _limit: Option<i64>,
        ) -> Result<Vec<ResourceBooking>, String> {
            let map = self.bookings.lock().unwrap();
            let now = Utc::now();
            Ok(map
                .values()
                .filter(|b| {
                    b.building_id == building_id
                        && b.start_time > now
                        && matches!(b.status, BookingStatus::Pending | BookingStatus::Confirmed)
                })
                .cloned()
                .collect())
        }

        async fn find_active(&self, building_id: Uuid) -> Result<Vec<ResourceBooking>, String> {
            let map = self.bookings.lock().unwrap();
            let now = Utc::now();
            Ok(map
                .values()
                .filter(|b| {
                    b.building_id == building_id
                        && b.status == BookingStatus::Confirmed
                        && now >= b.start_time
                        && now < b.end_time
                })
                .cloned()
                .collect())
        }

        async fn find_past(
            &self,
            building_id: Uuid,
            _limit: Option<i64>,
        ) -> Result<Vec<ResourceBooking>, String> {
            let map = self.bookings.lock().unwrap();
            let now = Utc::now();
            Ok(map
                .values()
                .filter(|b| b.building_id == building_id && b.end_time < now)
                .cloned()
                .collect())
        }

        async fn find_conflicts(
            &self,
            building_id: Uuid,
            resource_type: ResourceType,
            resource_name: &str,
            start_time: DateTime<Utc>,
            end_time: DateTime<Utc>,
            exclude_booking_id: Option<Uuid>,
        ) -> Result<Vec<ResourceBooking>, String> {
            let map = self.bookings.lock().unwrap();
            Ok(map
                .values()
                .filter(|b| {
                    b.building_id == building_id
                        && b.resource_type == resource_type
                        && b.resource_name == resource_name
                        && matches!(b.status, BookingStatus::Pending | BookingStatus::Confirmed)
                        && b.start_time < end_time
                        && start_time < b.end_time
                        && exclude_booking_id.map_or(true, |id| b.id != id)
                })
                .cloned()
                .collect())
        }

        async fn update(&self, booking: &ResourceBooking) -> Result<ResourceBooking, String> {
            let mut map = self.bookings.lock().unwrap();
            map.insert(booking.id, booking.clone());
            Ok(booking.clone())
        }

        async fn delete(&self, id: Uuid) -> Result<(), String> {
            let mut map = self.bookings.lock().unwrap();
            map.remove(&id);
            Ok(())
        }

        async fn count_by_building(&self, building_id: Uuid) -> Result<i64, String> {
            let map = self.bookings.lock().unwrap();
            Ok(map
                .values()
                .filter(|b| b.building_id == building_id)
                .count() as i64)
        }

        async fn count_by_building_and_status(
            &self,
            building_id: Uuid,
            status: BookingStatus,
        ) -> Result<i64, String> {
            let map = self.bookings.lock().unwrap();
            Ok(map
                .values()
                .filter(|b| b.building_id == building_id && b.status == status)
                .count() as i64)
        }

        async fn count_by_resource(
            &self,
            building_id: Uuid,
            resource_type: ResourceType,
            resource_name: &str,
        ) -> Result<i64, String> {
            let map = self.bookings.lock().unwrap();
            Ok(map
                .values()
                .filter(|b| {
                    b.building_id == building_id
                        && b.resource_type == resource_type
                        && b.resource_name == resource_name
                })
                .count() as i64)
        }

        async fn get_statistics(&self, building_id: Uuid) -> Result<BookingStatisticsDto, String> {
            Ok(BookingStatisticsDto {
                building_id,
                total_bookings: 0,
                confirmed_bookings: 0,
                pending_bookings: 0,
                cancelled_bookings: 0,
                completed_bookings: 0,
                no_show_bookings: 0,
                active_bookings: 0,
                upcoming_bookings: 0,
                total_hours_booked: 0.0,
                most_popular_resource: None,
            })
        }
    }

    // ── Mock OwnerRepository ────────────────────────────────────────────────
    struct MockOwnerRepo {
        owners: Mutex<HashMap<Uuid, Owner>>,
    }

    impl MockOwnerRepo {
        fn new() -> Self {
            Self {
                owners: Mutex::new(HashMap::new()),
            }
        }

        fn add_owner(&self, owner: Owner) {
            let mut map = self.owners.lock().unwrap();
            map.insert(owner.id, owner);
        }
    }

    #[async_trait]
    impl OwnerRepository for MockOwnerRepo {
        async fn create(&self, owner: &Owner) -> Result<Owner, String> {
            let mut map = self.owners.lock().unwrap();
            map.insert(owner.id, owner.clone());
            Ok(owner.clone())
        }

        async fn find_by_id(&self, id: Uuid) -> Result<Option<Owner>, String> {
            let map = self.owners.lock().unwrap();
            Ok(map.get(&id).cloned())
        }

        async fn find_by_user_id(&self, user_id: Uuid) -> Result<Option<Owner>, String> {
            let map = self.owners.lock().unwrap();
            Ok(map.values().find(|o| o.user_id == Some(user_id)).cloned())
        }

        async fn find_by_user_id_and_organization(
            &self,
            user_id: Uuid,
            organization_id: Uuid,
        ) -> Result<Option<Owner>, String> {
            let map = self.owners.lock().unwrap();
            Ok(map
                .values()
                .find(|o| o.user_id == Some(user_id) && o.organization_id == organization_id)
                .cloned())
        }

        async fn find_by_email(&self, email: &str) -> Result<Option<Owner>, String> {
            let map = self.owners.lock().unwrap();
            Ok(map.values().find(|o| o.email == email).cloned())
        }

        async fn find_all(&self) -> Result<Vec<Owner>, String> {
            let map = self.owners.lock().unwrap();
            Ok(map.values().cloned().collect())
        }

        async fn find_all_paginated(
            &self,
            _page_request: &PageRequest,
            _filters: &OwnerFilters,
        ) -> Result<(Vec<Owner>, i64), String> {
            let map = self.owners.lock().unwrap();
            let all: Vec<_> = map.values().cloned().collect();
            let count = all.len() as i64;
            Ok((all, count))
        }

        async fn update(&self, owner: &Owner) -> Result<Owner, String> {
            let mut map = self.owners.lock().unwrap();
            map.insert(owner.id, owner.clone());
            Ok(owner.clone())
        }

        async fn delete(&self, id: Uuid) -> Result<bool, String> {
            let mut map = self.owners.lock().unwrap();
            Ok(map.remove(&id).is_some())
        }
    }

    // ── Helpers ─────────────────────────────────────────────────────────────
    fn create_test_owner(user_id: Uuid, organization_id: Uuid) -> Owner {
        let mut owner = Owner::new(
            organization_id,
            "Jean".to_string(),
            "Dupont".to_string(),
            "jean@test.com".to_string(),
            None,
            "Rue Test 1".to_string(),
            "Brussels".to_string(),
            "1000".to_string(),
            "Belgium".to_string(),
        )
        .unwrap();
        owner.user_id = Some(user_id);
        owner
    }

    fn setup_use_cases() -> (
        ResourceBookingUseCases,
        Uuid,
        Uuid,
        Uuid,
        Arc<MockBookingRepo>,
    ) {
        let user_id = Uuid::new_v4();
        let organization_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();

        let booking_repo = Arc::new(MockBookingRepo::new());
        let owner_repo = Arc::new(MockOwnerRepo::new());

        let owner = create_test_owner(user_id, organization_id);
        owner_repo.add_owner(owner);

        let use_cases = ResourceBookingUseCases::new(
            booking_repo.clone() as Arc<dyn ResourceBookingRepository>,
            owner_repo as Arc<dyn OwnerRepository>,
        );

        (
            use_cases,
            user_id,
            organization_id,
            building_id,
            booking_repo,
        )
    }

    fn make_create_dto(building_id: Uuid) -> CreateResourceBookingDto {
        let start_time = Utc::now() + Duration::hours(2);
        let end_time = start_time + Duration::hours(2);
        CreateResourceBookingDto {
            building_id,
            resource_type: ResourceType::MeetingRoom,
            resource_name: "Meeting Room A".to_string(),
            start_time,
            end_time,
            notes: Some("Team standup".to_string()),
            recurring_pattern: RecurringPattern::None,
            recurrence_end_date: None,
            max_duration_hours: None,
            max_advance_days: None,
        }
    }

    // ── Tests ───────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_create_booking_success() {
        let (use_cases, user_id, org_id, building_id, _) = setup_use_cases();
        let dto = make_create_dto(building_id);

        let result = use_cases.create_booking(user_id, org_id, dto).await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.building_id, building_id);
        assert_eq!(response.resource_name, "Meeting Room A");
        assert_eq!(response.booked_by_name, "Jean Dupont");
    }

    #[tokio::test]
    async fn test_create_booking_conflict_detected() {
        let (use_cases, user_id, org_id, building_id, _) = setup_use_cases();
        let dto = make_create_dto(building_id);

        // First booking succeeds
        use_cases
            .create_booking(user_id, org_id, dto.clone())
            .await
            .unwrap();

        // Second booking for same resource and time range should fail
        let result = use_cases.create_booking(user_id, org_id, dto).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("conflicts with"));
    }

    #[tokio::test]
    async fn test_get_booking_success() {
        let (use_cases, user_id, org_id, building_id, _) = setup_use_cases();
        let dto = make_create_dto(building_id);

        let created = use_cases
            .create_booking(user_id, org_id, dto)
            .await
            .unwrap();

        let result = use_cases.get_booking(created.id).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().id, created.id);
    }

    #[tokio::test]
    async fn test_get_booking_not_found() {
        let (use_cases, _, _, _, _) = setup_use_cases();
        let result = use_cases.get_booking(Uuid::new_v4()).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Booking not found");
    }

    #[tokio::test]
    async fn test_cancel_booking_success() {
        let (use_cases, user_id, org_id, building_id, _) = setup_use_cases();
        let dto = make_create_dto(building_id);

        let created = use_cases
            .create_booking(user_id, org_id, dto)
            .await
            .unwrap();

        let result = use_cases.cancel_booking(created.id, user_id, org_id).await;
        assert!(result.is_ok());
        let cancelled = result.unwrap();
        assert_eq!(cancelled.status, BookingStatus::Cancelled);
    }

    #[tokio::test]
    async fn test_cancel_booking_wrong_user() {
        let (use_cases, user_id, org_id, building_id, _) = setup_use_cases();
        let dto = make_create_dto(building_id);

        let created = use_cases
            .create_booking(user_id, org_id, dto)
            .await
            .unwrap();

        // Try cancelling with a different user who is not registered as owner
        let other_user = Uuid::new_v4();
        let result = use_cases
            .cancel_booking(created.id, other_user, org_id)
            .await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Owner not found"));
    }

    #[tokio::test]
    async fn test_delete_booking_success() {
        let (use_cases, user_id, org_id, building_id, _) = setup_use_cases();
        let dto = make_create_dto(building_id);

        let created = use_cases
            .create_booking(user_id, org_id, dto)
            .await
            .unwrap();

        let result = use_cases.delete_booking(created.id, user_id, org_id).await;
        assert!(result.is_ok());

        // Confirm it's gone
        let fetch = use_cases.get_booking(created.id).await;
        assert!(fetch.is_err());
    }

    #[tokio::test]
    async fn test_confirm_booking_success() {
        let (use_cases, user_id, org_id, building_id, _booking_repo) = setup_use_cases();
        let dto = make_create_dto(building_id);

        let created = use_cases
            .create_booking(user_id, org_id, dto)
            .await
            .unwrap();

        // Manually confirm via the use case
        let result = use_cases.confirm_booking(created.id).await;
        assert!(result.is_ok());
        let confirmed = result.unwrap();
        assert_eq!(confirmed.status, BookingStatus::Confirmed);

        // Now we can complete it
        let completed = use_cases.complete_booking(created.id).await;
        assert!(completed.is_ok());
        assert_eq!(completed.unwrap().status, BookingStatus::Completed);
    }

    #[tokio::test]
    async fn test_list_building_bookings() {
        let (use_cases, user_id, org_id, building_id, _) = setup_use_cases();

        let dto1 = make_create_dto(building_id);

        let mut dto2 = make_create_dto(building_id);
        dto2.resource_name = "Meeting Room B".to_string();

        use_cases
            .create_booking(user_id, org_id, dto1)
            .await
            .unwrap();
        use_cases
            .create_booking(user_id, org_id, dto2)
            .await
            .unwrap();

        let result = use_cases.list_building_bookings(building_id).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 2);
    }

    #[tokio::test]
    async fn test_owner_not_found_for_user() {
        let booking_repo = Arc::new(MockBookingRepo::new());
        let owner_repo = Arc::new(MockOwnerRepo::new());
        // Do not add any owner
        let use_cases = ResourceBookingUseCases::new(
            booking_repo as Arc<dyn ResourceBookingRepository>,
            owner_repo as Arc<dyn OwnerRepository>,
        );

        let building_id = Uuid::new_v4();
        let dto = make_create_dto(building_id);
        let result = use_cases
            .create_booking(Uuid::new_v4(), Uuid::new_v4(), dto)
            .await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Owner not found"));
    }
}
