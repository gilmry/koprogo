use crate::application::dto::BookingStatisticsDto;
use crate::application::ports::ResourceBookingRepository;
use crate::domain::entities::{BookingStatus, RecurringPattern, ResourceBooking, ResourceType};
use crate::infrastructure::pool::DbPool;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::Row;
use uuid::Uuid;

pub struct PostgresResourceBookingRepository {
    pool: DbPool,
}

impl PostgresResourceBookingRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    /// Helper to convert database row to ResourceBooking entity
    fn row_to_entity(row: &sqlx::postgres::PgRow) -> Result<ResourceBooking, String> {
        // Parse ENUMs from database strings
        let resource_type_str: String = row
            .try_get("resource_type")
            .map_err(|e| format!("Failed to get resource_type: {}", e))?;
        let resource_type: ResourceType = serde_json::from_str(&format!("\"{}\"", resource_type_str))
            .map_err(|e| format!("Failed to parse resource_type: {}", e))?;

        let status_str: String = row
            .try_get("status")
            .map_err(|e| format!("Failed to get status: {}", e))?;
        let status: BookingStatus = serde_json::from_str(&format!("\"{}\"", status_str))
            .map_err(|e| format!("Failed to parse status: {}", e))?;

        let recurring_pattern_str: String = row
            .try_get("recurring_pattern")
            .map_err(|e| format!("Failed to get recurring_pattern: {}", e))?;
        let recurring_pattern: RecurringPattern =
            serde_json::from_str(&format!("\"{}\"", recurring_pattern_str))
                .map_err(|e| format!("Failed to parse recurring_pattern: {}", e))?;

        Ok(ResourceBooking {
            id: row
                .try_get("id")
                .map_err(|e| format!("Failed to get id: {}", e))?,
            building_id: row
                .try_get("building_id")
                .map_err(|e| format!("Failed to get building_id: {}", e))?,
            resource_type,
            resource_name: row
                .try_get("resource_name")
                .map_err(|e| format!("Failed to get resource_name: {}", e))?,
            booked_by: row
                .try_get("booked_by")
                .map_err(|e| format!("Failed to get booked_by: {}", e))?,
            start_time: row
                .try_get("start_time")
                .map_err(|e| format!("Failed to get start_time: {}", e))?,
            end_time: row
                .try_get("end_time")
                .map_err(|e| format!("Failed to get end_time: {}", e))?,
            status,
            notes: row
                .try_get("notes")
                .map_err(|e| format!("Failed to get notes: {}", e))?,
            recurring_pattern,
            recurrence_end_date: row
                .try_get("recurrence_end_date")
                .map_err(|e| format!("Failed to get recurrence_end_date: {}", e))?,
            created_at: row
                .try_get("created_at")
                .map_err(|e| format!("Failed to get created_at: {}", e))?,
            updated_at: row
                .try_get("updated_at")
                .map_err(|e| format!("Failed to get updated_at: {}", e))?,
        })
    }
}

#[async_trait]
impl ResourceBookingRepository for PostgresResourceBookingRepository {
    async fn create(&self, booking: &ResourceBooking) -> Result<ResourceBooking, String> {
        // Serialize ENUMs to strings for database
        let resource_type_str = serde_json::to_string(&booking.resource_type)
            .map_err(|e| format!("Failed to serialize resource_type: {}", e))?
            .trim_matches('"')
            .to_string();

        let status_str = serde_json::to_string(&booking.status)
            .map_err(|e| format!("Failed to serialize status: {}", e))?
            .trim_matches('"')
            .to_string();

        let recurring_pattern_str = serde_json::to_string(&booking.recurring_pattern)
            .map_err(|e| format!("Failed to serialize recurring_pattern: {}", e))?
            .trim_matches('"')
            .to_string();

        sqlx::query(
            r#"
            INSERT INTO resource_bookings (
                id, building_id, resource_type, resource_name, booked_by,
                start_time, end_time, status, notes, recurring_pattern,
                recurrence_end_date, created_at, updated_at
            )
            VALUES ($1, $2, $3::resource_type, $4, $5, $6, $7, $8::booking_status, $9,
                    $10::recurring_pattern, $11, $12, $13)
            "#,
        )
        .bind(booking.id)
        .bind(booking.building_id)
        .bind(&resource_type_str)
        .bind(&booking.resource_name)
        .bind(booking.booked_by)
        .bind(booking.start_time)
        .bind(booking.end_time)
        .bind(&status_str)
        .bind(&booking.notes)
        .bind(&recurring_pattern_str)
        .bind(booking.recurrence_end_date)
        .bind(booking.created_at)
        .bind(booking.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to create resource booking: {}", e))?;

        Ok(booking.clone())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<ResourceBooking>, String> {
        let row = sqlx::query(
            r#"
            SELECT id, building_id, resource_type, resource_name, booked_by,
                   start_time, end_time, status, notes, recurring_pattern,
                   recurrence_end_date, created_at, updated_at
            FROM resource_bookings
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Failed to find resource booking: {}", e))?;

        match row {
            Some(r) => Ok(Some(Self::row_to_entity(&r)?)),
            None => Ok(None),
        }
    }

    async fn find_by_building(&self, building_id: Uuid) -> Result<Vec<ResourceBooking>, String> {
        let rows = sqlx::query(
            r#"
            SELECT id, building_id, resource_type, resource_name, booked_by,
                   start_time, end_time, status, notes, recurring_pattern,
                   recurrence_end_date, created_at, updated_at
            FROM resource_bookings
            WHERE building_id = $1
            ORDER BY start_time ASC
            "#,
        )
        .bind(building_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to find bookings by building: {}", e))?;

        rows.iter().map(Self::row_to_entity).collect()
    }

    async fn find_by_building_and_resource_type(
        &self,
        building_id: Uuid,
        resource_type: ResourceType,
    ) -> Result<Vec<ResourceBooking>, String> {
        let resource_type_str = serde_json::to_string(&resource_type)
            .map_err(|e| format!("Failed to serialize resource_type: {}", e))?
            .trim_matches('"')
            .to_string();

        let rows = sqlx::query(
            r#"
            SELECT id, building_id, resource_type, resource_name, booked_by,
                   start_time, end_time, status, notes, recurring_pattern,
                   recurrence_end_date, created_at, updated_at
            FROM resource_bookings
            WHERE building_id = $1 AND resource_type = $2::resource_type
            ORDER BY start_time ASC
            "#,
        )
        .bind(building_id)
        .bind(&resource_type_str)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to find bookings by building and resource type: {}", e))?;

        rows.iter().map(Self::row_to_entity).collect()
    }

    async fn find_by_resource(
        &self,
        building_id: Uuid,
        resource_type: ResourceType,
        resource_name: &str,
    ) -> Result<Vec<ResourceBooking>, String> {
        let resource_type_str = serde_json::to_string(&resource_type)
            .map_err(|e| format!("Failed to serialize resource_type: {}", e))?
            .trim_matches('"')
            .to_string();

        let rows = sqlx::query(
            r#"
            SELECT id, building_id, resource_type, resource_name, booked_by,
                   start_time, end_time, status, notes, recurring_pattern,
                   recurrence_end_date, created_at, updated_at
            FROM resource_bookings
            WHERE building_id = $1 AND resource_type = $2::resource_type AND resource_name = $3
            ORDER BY start_time ASC
            "#,
        )
        .bind(building_id)
        .bind(&resource_type_str)
        .bind(resource_name)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to find bookings by resource: {}", e))?;

        rows.iter().map(Self::row_to_entity).collect()
    }

    async fn find_by_user(&self, user_id: Uuid) -> Result<Vec<ResourceBooking>, String> {
        let rows = sqlx::query(
            r#"
            SELECT id, building_id, resource_type, resource_name, booked_by,
                   start_time, end_time, status, notes, recurring_pattern,
                   recurrence_end_date, created_at, updated_at
            FROM resource_bookings
            WHERE booked_by = $1
            ORDER BY start_time DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to find bookings by user: {}", e))?;

        rows.iter().map(Self::row_to_entity).collect()
    }

    async fn find_by_user_and_status(
        &self,
        user_id: Uuid,
        status: BookingStatus,
    ) -> Result<Vec<ResourceBooking>, String> {
        let status_str = serde_json::to_string(&status)
            .map_err(|e| format!("Failed to serialize status: {}", e))?
            .trim_matches('"')
            .to_string();

        let rows = sqlx::query(
            r#"
            SELECT id, building_id, resource_type, resource_name, booked_by,
                   start_time, end_time, status, notes, recurring_pattern,
                   recurrence_end_date, created_at, updated_at
            FROM resource_bookings
            WHERE booked_by = $1 AND status = $2::booking_status
            ORDER BY start_time DESC
            "#,
        )
        .bind(user_id)
        .bind(&status_str)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to find bookings by user and status: {}", e))?;

        rows.iter().map(Self::row_to_entity).collect()
    }

    async fn find_by_building_and_status(
        &self,
        building_id: Uuid,
        status: BookingStatus,
    ) -> Result<Vec<ResourceBooking>, String> {
        let status_str = serde_json::to_string(&status)
            .map_err(|e| format!("Failed to serialize status: {}", e))?
            .trim_matches('"')
            .to_string();

        let rows = sqlx::query(
            r#"
            SELECT id, building_id, resource_type, resource_name, booked_by,
                   start_time, end_time, status, notes, recurring_pattern,
                   recurrence_end_date, created_at, updated_at
            FROM resource_bookings
            WHERE building_id = $1 AND status = $2::booking_status
            ORDER BY start_time ASC
            "#,
        )
        .bind(building_id)
        .bind(&status_str)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to find bookings by building and status: {}", e))?;

        rows.iter().map(Self::row_to_entity).collect()
    }

    async fn find_upcoming(
        &self,
        building_id: Uuid,
        limit: Option<i64>,
    ) -> Result<Vec<ResourceBooking>, String> {
        let limit_val = limit.unwrap_or(50);

        let rows = sqlx::query(
            r#"
            SELECT id, building_id, resource_type, resource_name, booked_by,
                   start_time, end_time, status, notes, recurring_pattern,
                   recurrence_end_date, created_at, updated_at
            FROM resource_bookings
            WHERE building_id = $1
              AND start_time > NOW()
              AND status IN ('Confirmed', 'Pending')
            ORDER BY start_time ASC
            LIMIT $2
            "#,
        )
        .bind(building_id)
        .bind(limit_val)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to find upcoming bookings: {}", e))?;

        rows.iter().map(Self::row_to_entity).collect()
    }

    async fn find_active(&self, building_id: Uuid) -> Result<Vec<ResourceBooking>, String> {
        let rows = sqlx::query(
            r#"
            SELECT id, building_id, resource_type, resource_name, booked_by,
                   start_time, end_time, status, notes, recurring_pattern,
                   recurrence_end_date, created_at, updated_at
            FROM resource_bookings
            WHERE building_id = $1
              AND status = 'Confirmed'
              AND start_time <= NOW()
              AND end_time > NOW()
            ORDER BY start_time ASC
            "#,
        )
        .bind(building_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to find active bookings: {}", e))?;

        rows.iter().map(Self::row_to_entity).collect()
    }

    async fn find_past(
        &self,
        building_id: Uuid,
        limit: Option<i64>,
    ) -> Result<Vec<ResourceBooking>, String> {
        let limit_val = limit.unwrap_or(50);

        let rows = sqlx::query(
            r#"
            SELECT id, building_id, resource_type, resource_name, booked_by,
                   start_time, end_time, status, notes, recurring_pattern,
                   recurrence_end_date, created_at, updated_at
            FROM resource_bookings
            WHERE building_id = $1
              AND end_time < NOW()
            ORDER BY start_time DESC
            LIMIT $2
            "#,
        )
        .bind(building_id)
        .bind(limit_val)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to find past bookings: {}", e))?;

        rows.iter().map(Self::row_to_entity).collect()
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
        let resource_type_str = serde_json::to_string(&resource_type)
            .map_err(|e| format!("Failed to serialize resource_type: {}", e))?
            .trim_matches('"')
            .to_string();

        // Conflict detection: start1 < end2 AND start2 < end1
        // Exclude cancelled, completed, and no-show bookings
        let rows = if let Some(exclude_id) = exclude_booking_id {
            sqlx::query(
                r#"
                SELECT id, building_id, resource_type, resource_name, booked_by,
                       start_time, end_time, status, notes, recurring_pattern,
                       recurrence_end_date, created_at, updated_at
                FROM resource_bookings
                WHERE building_id = $1
                  AND resource_type = $2::resource_type
                  AND resource_name = $3
                  AND status IN ('Pending', 'Confirmed')
                  AND start_time < $5
                  AND end_time > $4
                  AND id != $6
                ORDER BY start_time ASC
                "#,
            )
            .bind(building_id)
            .bind(&resource_type_str)
            .bind(resource_name)
            .bind(start_time)
            .bind(end_time)
            .bind(exclude_id)
            .fetch_all(&self.pool)
            .await
        } else {
            sqlx::query(
                r#"
                SELECT id, building_id, resource_type, resource_name, booked_by,
                       start_time, end_time, status, notes, recurring_pattern,
                       recurrence_end_date, created_at, updated_at
                FROM resource_bookings
                WHERE building_id = $1
                  AND resource_type = $2::resource_type
                  AND resource_name = $3
                  AND status IN ('Pending', 'Confirmed')
                  AND start_time < $5
                  AND end_time > $4
                ORDER BY start_time ASC
                "#,
            )
            .bind(building_id)
            .bind(&resource_type_str)
            .bind(resource_name)
            .bind(start_time)
            .bind(end_time)
            .fetch_all(&self.pool)
            .await
        }
        .map_err(|e| format!("Failed to find conflicting bookings: {}", e))?;

        rows.iter().map(Self::row_to_entity).collect()
    }

    async fn update(&self, booking: &ResourceBooking) -> Result<ResourceBooking, String> {
        let resource_type_str = serde_json::to_string(&booking.resource_type)
            .map_err(|e| format!("Failed to serialize resource_type: {}", e))?
            .trim_matches('"')
            .to_string();

        let status_str = serde_json::to_string(&booking.status)
            .map_err(|e| format!("Failed to serialize status: {}", e))?
            .trim_matches('"')
            .to_string();

        let recurring_pattern_str = serde_json::to_string(&booking.recurring_pattern)
            .map_err(|e| format!("Failed to serialize recurring_pattern: {}", e))?
            .trim_matches('"')
            .to_string();

        let result = sqlx::query(
            r#"
            UPDATE resource_bookings
            SET resource_type = $2::resource_type,
                resource_name = $3,
                start_time = $4,
                end_time = $5,
                status = $6::booking_status,
                notes = $7,
                recurring_pattern = $8::recurring_pattern,
                recurrence_end_date = $9,
                updated_at = $10
            WHERE id = $1
            "#,
        )
        .bind(booking.id)
        .bind(&resource_type_str)
        .bind(&booking.resource_name)
        .bind(booking.start_time)
        .bind(booking.end_time)
        .bind(&status_str)
        .bind(&booking.notes)
        .bind(&recurring_pattern_str)
        .bind(booking.recurrence_end_date)
        .bind(booking.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to update resource booking: {}", e))?;

        if result.rows_affected() == 0 {
            return Err("Resource booking not found".to_string());
        }

        Ok(booking.clone())
    }

    async fn delete(&self, id: Uuid) -> Result<(), String> {
        let result = sqlx::query(
            r#"
            DELETE FROM resource_bookings
            WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to delete resource booking: {}", e))?;

        if result.rows_affected() == 0 {
            return Err("Resource booking not found".to_string());
        }

        Ok(())
    }

    async fn count_by_building(&self, building_id: Uuid) -> Result<i64, String> {
        let row = sqlx::query(
            r#"
            SELECT COUNT(*) as count
            FROM resource_bookings
            WHERE building_id = $1
            "#,
        )
        .bind(building_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to count bookings by building: {}", e))?;

        let count: i64 = row
            .try_get("count")
            .map_err(|e| format!("Failed to get count: {}", e))?;
        Ok(count)
    }

    async fn count_by_building_and_status(
        &self,
        building_id: Uuid,
        status: BookingStatus,
    ) -> Result<i64, String> {
        let status_str = serde_json::to_string(&status)
            .map_err(|e| format!("Failed to serialize status: {}", e))?
            .trim_matches('"')
            .to_string();

        let row = sqlx::query(
            r#"
            SELECT COUNT(*) as count
            FROM resource_bookings
            WHERE building_id = $1 AND status = $2::booking_status
            "#,
        )
        .bind(building_id)
        .bind(&status_str)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to count bookings by building and status: {}", e))?;

        let count: i64 = row
            .try_get("count")
            .map_err(|e| format!("Failed to get count: {}", e))?;
        Ok(count)
    }

    async fn count_by_resource(
        &self,
        building_id: Uuid,
        resource_type: ResourceType,
        resource_name: &str,
    ) -> Result<i64, String> {
        let resource_type_str = serde_json::to_string(&resource_type)
            .map_err(|e| format!("Failed to serialize resource_type: {}", e))?
            .trim_matches('"')
            .to_string();

        let row = sqlx::query(
            r#"
            SELECT COUNT(*) as count
            FROM resource_bookings
            WHERE building_id = $1 AND resource_type = $2::resource_type AND resource_name = $3
            "#,
        )
        .bind(building_id)
        .bind(&resource_type_str)
        .bind(resource_name)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to count bookings by resource: {}", e))?;

        let count: i64 = row
            .try_get("count")
            .map_err(|e| format!("Failed to get count: {}", e))?;
        Ok(count)
    }

    async fn get_statistics(&self, building_id: Uuid) -> Result<BookingStatisticsDto, String> {
        // Get counts by status
        let total = self.count_by_building(building_id).await?;
        let confirmed = self
            .count_by_building_and_status(building_id, BookingStatus::Confirmed)
            .await?;
        let pending = self
            .count_by_building_and_status(building_id, BookingStatus::Pending)
            .await?;
        let completed = self
            .count_by_building_and_status(building_id, BookingStatus::Completed)
            .await?;
        let cancelled = self
            .count_by_building_and_status(building_id, BookingStatus::Cancelled)
            .await?;
        let no_show = self
            .count_by_building_and_status(building_id, BookingStatus::NoShow)
            .await?;

        // Get active bookings count (currently in progress)
        let active_bookings = self.find_active(building_id).await?.len() as i64;

        // Get upcoming bookings count (future)
        let upcoming_row = sqlx::query(
            r#"
            SELECT COUNT(*) as count
            FROM resource_bookings
            WHERE building_id = $1
              AND start_time > NOW()
              AND status IN ('Confirmed', 'Pending')
            "#,
        )
        .bind(building_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to count upcoming bookings: {}", e))?;

        let upcoming_bookings: i64 = upcoming_row
            .try_get("count")
            .map_err(|e| format!("Failed to get upcoming count: {}", e))?;

        // Calculate total hours booked
        let hours_row = sqlx::query(
            r#"
            SELECT COALESCE(SUM(EXTRACT(EPOCH FROM (end_time - start_time)) / 3600), 0) as total_hours
            FROM resource_bookings
            WHERE building_id = $1
              AND status IN ('Confirmed', 'Completed')
            "#,
        )
        .bind(building_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to calculate total hours booked: {}", e))?;

        let total_hours_booked: f64 = hours_row
            .try_get("total_hours")
            .map_err(|e| format!("Failed to get total_hours: {}", e))?;

        // Find most popular resource
        let popular_row = sqlx::query(
            r#"
            SELECT resource_name, COUNT(*) as booking_count
            FROM resource_bookings
            WHERE building_id = $1
            GROUP BY resource_name
            ORDER BY booking_count DESC
            LIMIT 1
            "#,
        )
        .bind(building_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Failed to find most popular resource: {}", e))?;

        let most_popular_resource = popular_row.map(|row| {
            row.try_get::<String, _>("resource_name")
                .unwrap_or_default()
        });

        Ok(BookingStatisticsDto {
            building_id,
            total_bookings: total,
            confirmed_bookings: confirmed,
            pending_bookings: pending,
            completed_bookings: completed,
            cancelled_bookings: cancelled,
            no_show_bookings: no_show,
            active_bookings,
            upcoming_bookings,
            total_hours_booked,
            most_popular_resource,
        })
    }
}
