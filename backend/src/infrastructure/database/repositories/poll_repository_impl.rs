use crate::application::dto::{PageRequest, PollFilters};
use crate::application::ports::{PollRepository, PollStatistics};
use crate::domain::entities::{Poll, PollOption, PollStatus, PollType};
use crate::infrastructure::database::pool::DbPool;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde_json;
use sqlx::Row;
use uuid::Uuid;

pub struct PostgresPollRepository {
    pool: DbPool,
}

impl PostgresPollRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    /// Convert PollType to database string format
    fn poll_type_to_string(poll_type: &PollType) -> &'static str {
        match poll_type {
            PollType::YesNo => "yes_no",
            PollType::MultipleChoice => "multiple_choice",
            PollType::Rating => "rating",
            PollType::OpenEnded => "open_ended",
        }
    }

    /// Parse PollType from database string
    fn parse_poll_type(s: &str) -> PollType {
        match s {
            "yes_no" => PollType::YesNo,
            "multiple_choice" => PollType::MultipleChoice,
            "rating" => PollType::Rating,
            "open_ended" => PollType::OpenEnded,
            _ => PollType::YesNo, // Default fallback
        }
    }

    /// Convert PollStatus to database string format
    fn poll_status_to_string(status: &PollStatus) -> &'static str {
        match status {
            PollStatus::Draft => "draft",
            PollStatus::Active => "active",
            PollStatus::Closed => "closed",
            PollStatus::Cancelled => "cancelled",
        }
    }

    /// Parse PollStatus from database string
    fn parse_poll_status(s: &str) -> PollStatus {
        match s {
            "draft" => PollStatus::Draft,
            "active" => PollStatus::Active,
            "closed" => PollStatus::Closed,
            "cancelled" => PollStatus::Cancelled,
            _ => PollStatus::Draft, // Default fallback
        }
    }

    /// Map database row to Poll entity
    fn row_to_poll(&self, row: &sqlx::postgres::PgRow) -> Result<Poll, String> {
        // Parse options from JSONB
        let options_json: serde_json::Value = row
            .try_get("options")
            .map_err(|e| format!("Failed to get options: {}", e))?;

        let options: Vec<PollOption> = serde_json::from_value(options_json)
            .map_err(|e| format!("Failed to deserialize options: {}", e))?;

        Ok(Poll {
            id: row
                .try_get("id")
                .map_err(|e| format!("Failed to get id: {}", e))?,
            building_id: row
                .try_get("building_id")
                .map_err(|e| format!("Failed to get building_id: {}", e))?,
            created_by: row
                .try_get("created_by")
                .map_err(|e| format!("Failed to get created_by: {}", e))?,
            title: row
                .try_get("title")
                .map_err(|e| format!("Failed to get title: {}", e))?,
            description: row
                .try_get("description")
                .map_err(|e| format!("Failed to get description: {}", e))?,
            poll_type: Self::parse_poll_type(
                row.try_get("poll_type")
                    .map_err(|e| format!("Failed to get poll_type: {}", e))?,
            ),
            options,
            is_anonymous: row
                .try_get("is_anonymous")
                .map_err(|e| format!("Failed to get is_anonymous: {}", e))?,
            allow_multiple_votes: row
                .try_get("allow_multiple_votes")
                .map_err(|e| format!("Failed to get allow_multiple_votes: {}", e))?,
            require_all_owners: row
                .try_get("require_all_owners")
                .map_err(|e| format!("Failed to get require_all_owners: {}", e))?,
            starts_at: row
                .try_get("starts_at")
                .map_err(|e| format!("Failed to get starts_at: {}", e))?,
            ends_at: row
                .try_get("ends_at")
                .map_err(|e| format!("Failed to get ends_at: {}", e))?,
            status: Self::parse_poll_status(
                row.try_get("status")
                    .map_err(|e| format!("Failed to get status: {}", e))?,
            ),
            total_eligible_voters: row
                .try_get("total_eligible_voters")
                .map_err(|e| format!("Failed to get total_eligible_voters: {}", e))?,
            total_votes_cast: row
                .try_get("total_votes_cast")
                .map_err(|e| format!("Failed to get total_votes_cast: {}", e))?,
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
impl PollRepository for PostgresPollRepository {
    async fn create(&self, poll: &Poll) -> Result<Poll, String> {
        let poll_type_str = Self::poll_type_to_string(&poll.poll_type);
        let status_str = Self::poll_status_to_string(&poll.status);

        // Serialize options to JSON
        let options_json = serde_json::to_value(&poll.options)
            .map_err(|e| format!("Failed to serialize options: {}", e))?;

        sqlx::query(
            r#"
            INSERT INTO polls (
                id, building_id, created_by, title, description, poll_type, options,
                is_anonymous, allow_multiple_votes, require_all_owners,
                starts_at, ends_at, status, total_eligible_voters, total_votes_cast,
                created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17)
            "#,
        )
        .bind(poll.id)
        .bind(poll.building_id)
        .bind(poll.created_by)
        .bind(&poll.title)
        .bind(&poll.description)
        .bind(poll_type_str)
        .bind(options_json)
        .bind(poll.is_anonymous)
        .bind(poll.allow_multiple_votes)
        .bind(poll.require_all_owners)
        .bind(poll.starts_at)
        .bind(poll.ends_at)
        .bind(status_str)
        .bind(poll.total_eligible_voters)
        .bind(poll.total_votes_cast)
        .bind(poll.created_at)
        .bind(poll.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to insert poll: {}", e))?;

        Ok(poll.clone())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Poll>, String> {
        let row = sqlx::query(
            r#"
            SELECT * FROM polls WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Failed to fetch poll: {}", e))?;

        match row {
            Some(r) => Ok(Some(self.row_to_poll(&r)?)),
            None => Ok(None),
        }
    }

    async fn find_by_building(&self, building_id: Uuid) -> Result<Vec<Poll>, String> {
        let rows = sqlx::query(
            r#"
            SELECT * FROM polls 
            WHERE building_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(building_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to fetch polls: {}", e))?;

        rows.iter()
            .map(|row| self.row_to_poll(row))
            .collect::<Result<Vec<Poll>, String>>()
    }

    async fn find_by_created_by(&self, created_by: Uuid) -> Result<Vec<Poll>, String> {
        let rows = sqlx::query(
            r#"
            SELECT * FROM polls 
            WHERE created_by = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(created_by)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to fetch polls: {}", e))?;

        rows.iter()
            .map(|row| self.row_to_poll(row))
            .collect::<Result<Vec<Poll>, String>>()
    }

    async fn find_all_paginated(
        &self,
        page_request: &PageRequest,
        filters: &PollFilters,
    ) -> Result<(Vec<Poll>, i64), String> {
        let offset = (page_request.page - 1) * page_request.per_page;

        // Build WHERE clause dynamically
        let mut where_clauses = Vec::new();
        let mut bind_index = 1;

        if filters.building_id.is_some() {
            where_clauses.push(format!("building_id = ${}", bind_index));
            bind_index += 1;
        }

        if filters.created_by.is_some() {
            where_clauses.push(format!("created_by = ${}", bind_index));
            bind_index += 1;
        }

        if filters.status.is_some() {
            where_clauses.push(format!("status = ${}", bind_index));
            bind_index += 1;
        }

        if filters.poll_type.is_some() {
            where_clauses.push(format!("poll_type = ${}", bind_index));
            bind_index += 1;
        }

        if filters.ends_before.is_some() {
            where_clauses.push(format!("ends_at < ${}", bind_index));
            bind_index += 1;
        }

        if filters.ends_after.is_some() {
            where_clauses.push(format!("ends_at > ${}", bind_index));
            bind_index += 1;
        }

        let where_sql = if where_clauses.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", where_clauses.join(" AND "))
        };

        // Count total
        let count_query = format!("SELECT COUNT(*) as count FROM polls {}", where_sql);
        let mut count_query_builder = sqlx::query(&count_query);

        if let Some(ref building_id) = filters.building_id {
            let id = Uuid::parse_str(building_id)
                .map_err(|_| "Invalid building_id format".to_string())?;
            count_query_builder = count_query_builder.bind(id);
        }
        if let Some(ref created_by) = filters.created_by {
            let id = Uuid::parse_str(created_by)
                .map_err(|_| "Invalid created_by format".to_string())?;
            count_query_builder = count_query_builder.bind(id);
        }
        if let Some(ref status) = filters.status {
            count_query_builder = count_query_builder.bind(Self::poll_status_to_string(status));
        }
        if let Some(ref poll_type) = filters.poll_type {
            count_query_builder = count_query_builder.bind(Self::poll_type_to_string(poll_type));
        }
        if let Some(ref ends_before) = filters.ends_before {
            let date = DateTime::parse_from_rfc3339(ends_before)
                .map_err(|_| "Invalid ends_before format".to_string())?
                .with_timezone(&Utc);
            count_query_builder = count_query_builder.bind(date);
        }
        if let Some(ref ends_after) = filters.ends_after {
            let date = DateTime::parse_from_rfc3339(ends_after)
                .map_err(|_| "Invalid ends_after format".to_string())?
                .with_timezone(&Utc);
            count_query_builder = count_query_builder.bind(date);
        }

        let count_row = count_query_builder
            .fetch_one(&self.pool)
            .await
            .map_err(|e| format!("Failed to count polls: {}", e))?;
        let total: i64 = count_row.try_get("count")
            .map_err(|e| format!("Failed to get count: {}", e))?;

        // Fetch paginated results
        let data_query = format!(
            "SELECT * FROM polls {} ORDER BY created_at DESC LIMIT ${} OFFSET ${}",
            where_sql,
            bind_index,
            bind_index + 1
        );
        let mut data_query_builder = sqlx::query(&data_query);

        if let Some(ref building_id) = filters.building_id {
            let id = Uuid::parse_str(building_id)
                .map_err(|_| "Invalid building_id format".to_string())?;
            data_query_builder = data_query_builder.bind(id);
        }
        if let Some(ref created_by) = filters.created_by {
            let id = Uuid::parse_str(created_by)
                .map_err(|_| "Invalid created_by format".to_string())?;
            data_query_builder = data_query_builder.bind(id);
        }
        if let Some(ref status) = filters.status {
            data_query_builder = data_query_builder.bind(Self::poll_status_to_string(status));
        }
        if let Some(ref poll_type) = filters.poll_type {
            data_query_builder = data_query_builder.bind(Self::poll_type_to_string(poll_type));
        }
        if let Some(ref ends_before) = filters.ends_before {
            let date = DateTime::parse_from_rfc3339(ends_before)
                .map_err(|_| "Invalid ends_before format".to_string())?
                .with_timezone(&Utc);
            data_query_builder = data_query_builder.bind(date);
        }
        if let Some(ref ends_after) = filters.ends_after {
            let date = DateTime::parse_from_rfc3339(ends_after)
                .map_err(|_| "Invalid ends_after format".to_string())?
                .with_timezone(&Utc);
            data_query_builder = data_query_builder.bind(date);
        }

        data_query_builder = data_query_builder.bind(page_request.per_page).bind(offset);

        let rows = data_query_builder
            .fetch_all(&self.pool)
            .await
            .map_err(|e| format!("Failed to fetch polls: {}", e))?;

        let polls = rows
            .iter()
            .map(|row| self.row_to_poll(row))
            .collect::<Result<Vec<Poll>, String>>()?;

        Ok((polls, total))
    }

    async fn find_active(&self, building_id: Uuid) -> Result<Vec<Poll>, String> {
        let now = Utc::now();

        let rows = sqlx::query(
            r#"
            SELECT * FROM polls 
            WHERE building_id = $1 
              AND status = 'active'
              AND starts_at <= $2
              AND ends_at > $2
            ORDER BY created_at DESC
            "#,
        )
        .bind(building_id)
        .bind(now)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to fetch active polls: {}", e))?;

        rows.iter()
            .map(|row| self.row_to_poll(row))
            .collect::<Result<Vec<Poll>, String>>()
    }

    async fn find_by_status(
        &self,
        building_id: Uuid,
        status: &str,
    ) -> Result<Vec<Poll>, String> {
        let rows = sqlx::query(
            r#"
            SELECT * FROM polls 
            WHERE building_id = $1 AND status = $2
            ORDER BY created_at DESC
            "#,
        )
        .bind(building_id)
        .bind(status)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to fetch polls by status: {}", e))?;

        rows.iter()
            .map(|row| self.row_to_poll(row))
            .collect::<Result<Vec<Poll>, String>>()
    }

    async fn find_expired_active(&self) -> Result<Vec<Poll>, String> {
        let now = Utc::now();

        let rows = sqlx::query(
            r#"
            SELECT * FROM polls 
            WHERE status = 'active' AND ends_at <= $1
            ORDER BY ends_at ASC
            "#,
        )
        .bind(now)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to fetch expired polls: {}", e))?;

        rows.iter()
            .map(|row| self.row_to_poll(row))
            .collect::<Result<Vec<Poll>, String>>()
    }

    async fn update(&self, poll: &Poll) -> Result<Poll, String> {
        let poll_type_str = Self::poll_type_to_string(&poll.poll_type);
        let status_str = Self::poll_status_to_string(&poll.status);

        // Serialize options to JSON
        let options_json = serde_json::to_value(&poll.options)
            .map_err(|e| format!("Failed to serialize options: {}", e))?;

        sqlx::query(
            r#"
            UPDATE polls SET
                title = $2,
                description = $3,
                poll_type = $4,
                options = $5,
                is_anonymous = $6,
                allow_multiple_votes = $7,
                require_all_owners = $8,
                starts_at = $9,
                ends_at = $10,
                status = $11,
                total_eligible_voters = $12,
                total_votes_cast = $13,
                updated_at = $14
            WHERE id = $1
            "#,
        )
        .bind(poll.id)
        .bind(&poll.title)
        .bind(&poll.description)
        .bind(poll_type_str)
        .bind(options_json)
        .bind(poll.is_anonymous)
        .bind(poll.allow_multiple_votes)
        .bind(poll.require_all_owners)
        .bind(poll.starts_at)
        .bind(poll.ends_at)
        .bind(status_str)
        .bind(poll.total_eligible_voters)
        .bind(poll.total_votes_cast)
        .bind(poll.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to update poll: {}", e))?;

        Ok(poll.clone())
    }

    async fn delete(&self, id: Uuid) -> Result<bool, String> {
        let result = sqlx::query(
            r#"
            DELETE FROM polls WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to delete poll: {}", e))?;

        Ok(result.rows_affected() > 0)
    }

    async fn get_building_statistics(&self, building_id: Uuid) -> Result<PollStatistics, String> {
        let row = sqlx::query(
            r#"
            SELECT 
                COUNT(*) as total_polls,
                COUNT(*) FILTER (WHERE status = 'active') as active_polls,
                COUNT(*) FILTER (WHERE status = 'closed') as closed_polls,
                COALESCE(AVG(CASE 
                    WHEN total_eligible_voters > 0 
                    THEN (total_votes_cast::float / total_eligible_voters::float) * 100.0
                    ELSE 0
                END), 0.0) as avg_participation
            FROM polls
            WHERE building_id = $1
            "#,
        )
        .bind(building_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to fetch poll statistics: {}", e))?;

        Ok(PollStatistics {
            total_polls: row.try_get("total_polls")
                .map_err(|e| format!("Failed to get total_polls: {}", e))?,
            active_polls: row.try_get("active_polls")
                .map_err(|e| format!("Failed to get active_polls: {}", e))?,
            closed_polls: row.try_get("closed_polls")
                .map_err(|e| format!("Failed to get closed_polls: {}", e))?,
            average_participation_rate: row.try_get("avg_participation")
                .map_err(|e| format!("Failed to get avg_participation: {}", e))?,
        })
    }
}
