use crate::application::dto::PageRequest;
use crate::application::ports::{AuditLogFilters, AuditLogRepository};
use crate::infrastructure::audit::{AuditEventType, AuditLogEntry};
use crate::infrastructure::database::pool::DbPool;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::Row;
use uuid::Uuid;

pub struct PostgresAuditLogRepository {
    pool: DbPool,
}

impl PostgresAuditLogRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    /// Convert AuditEventType to string for database storage
    fn event_type_to_string(event_type: &AuditEventType) -> String {
        format!("{:?}", event_type)
    }

    /// Convert string from database to AuditEventType
    fn string_to_event_type(s: &str) -> AuditEventType {
        match s {
            "UserLogin" => AuditEventType::UserLogin,
            "UserLogout" => AuditEventType::UserLogout,
            "UserRegistration" => AuditEventType::UserRegistration,
            "TokenRefresh" => AuditEventType::TokenRefresh,
            "AuthenticationFailed" => AuditEventType::AuthenticationFailed,
            "BuildingCreated" => AuditEventType::BuildingCreated,
            "BuildingUpdated" => AuditEventType::BuildingUpdated,
            "BuildingDeleted" => AuditEventType::BuildingDeleted,
            "UnitCreated" => AuditEventType::UnitCreated,
            "UnitAssignedToOwner" => AuditEventType::UnitAssignedToOwner,
            "OwnerCreated" => AuditEventType::OwnerCreated,
            "OwnerUpdated" => AuditEventType::OwnerUpdated,
            "ExpenseCreated" => AuditEventType::ExpenseCreated,
            "ExpenseMarkedPaid" => AuditEventType::ExpenseMarkedPaid,
            "MeetingCreated" => AuditEventType::MeetingCreated,
            "MeetingCompleted" => AuditEventType::MeetingCompleted,
            "DocumentUploaded" => AuditEventType::DocumentUploaded,
            "DocumentDeleted" => AuditEventType::DocumentDeleted,
            "UnauthorizedAccess" => AuditEventType::UnauthorizedAccess,
            "RateLimitExceeded" => AuditEventType::RateLimitExceeded,
            "InvalidToken" => AuditEventType::InvalidToken,
            _ => AuditEventType::UnauthorizedAccess, // Default fallback
        }
    }

    /// Map database row to AuditLogEntry
    fn row_to_entry(row: &sqlx::postgres::PgRow) -> AuditLogEntry {
        let event_type_str: String = row.get("event_type");
        let metadata_json: Option<serde_json::Value> = row.get("metadata");

        AuditLogEntry {
            id: row.get("id"),
            timestamp: row.get("timestamp"),
            event_type: Self::string_to_event_type(&event_type_str),
            user_id: row.get("user_id"),
            organization_id: row.get("organization_id"),
            resource_type: row.get("resource_type"),
            resource_id: row.get("resource_id"),
            ip_address: row.get("ip_address"),
            user_agent: row.get("user_agent"),
            metadata: metadata_json,
            success: row.get("success"),
            error_message: row.get("error_message"),
        }
    }
}

#[async_trait]
impl AuditLogRepository for PostgresAuditLogRepository {
    async fn create(&self, entry: &AuditLogEntry) -> Result<AuditLogEntry, String> {
        let event_type_str = Self::event_type_to_string(&entry.event_type);

        sqlx::query(
            r#"
            INSERT INTO audit_logs (
                id, timestamp, event_type, user_id, organization_id,
                resource_type, resource_id, ip_address, user_agent,
                metadata, success, error_message, created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            "#,
        )
        .bind(entry.id)
        .bind(entry.timestamp)
        .bind(event_type_str)
        .bind(entry.user_id)
        .bind(entry.organization_id)
        .bind(&entry.resource_type)
        .bind(entry.resource_id)
        .bind(&entry.ip_address)
        .bind(&entry.user_agent)
        .bind(&entry.metadata)
        .bind(entry.success)
        .bind(&entry.error_message)
        .bind(Utc::now())
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(entry.clone())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<AuditLogEntry>, String> {
        let row = sqlx::query(
            r#"
            SELECT id, timestamp, event_type, user_id, organization_id,
                   resource_type, resource_id, ip_address, user_agent,
                   metadata, success, error_message
            FROM audit_logs
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(row.map(|r| Self::row_to_entry(&r)))
    }

    async fn find_all_paginated(
        &self,
        page_request: &PageRequest,
        filters: &AuditLogFilters,
    ) -> Result<(Vec<AuditLogEntry>, i64), String> {
        let limit = page_request.per_page.min(100);
        let offset = (page_request.page - 1) * limit;

        // Build WHERE clause dynamically based on filters
        let mut where_clauses = Vec::new();
        let mut param_index = 1;

        if filters.user_id.is_some() {
            where_clauses.push(format!("user_id = ${}", param_index));
            param_index += 1;
        }
        if filters.organization_id.is_some() {
            where_clauses.push(format!("organization_id = ${}", param_index));
            param_index += 1;
        }
        if filters.event_type.is_some() {
            where_clauses.push(format!("event_type = ${}", param_index));
            param_index += 1;
        }
        if filters.success.is_some() {
            where_clauses.push(format!("success = ${}", param_index));
            param_index += 1;
        }
        if filters.start_date.is_some() {
            where_clauses.push(format!("timestamp >= ${}", param_index));
            param_index += 1;
        }
        if filters.end_date.is_some() {
            where_clauses.push(format!("timestamp <= ${}", param_index));
            param_index += 1;
        }
        if filters.resource_type.is_some() {
            where_clauses.push(format!("resource_type = ${}", param_index));
            param_index += 1;
        }
        if filters.resource_id.is_some() {
            where_clauses.push(format!("resource_id = ${}", param_index));
            param_index += 1;
        }

        let where_clause = if where_clauses.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", where_clauses.join(" AND "))
        };

        // Count total matching records
        let count_query = format!(
            "SELECT COUNT(*) as count FROM audit_logs {}",
            where_clause
        );
        let mut count_query_builder = sqlx::query(&count_query);

        // Bind parameters for count query
        if let Some(user_id) = filters.user_id {
            count_query_builder = count_query_builder.bind(user_id);
        }
        if let Some(org_id) = filters.organization_id {
            count_query_builder = count_query_builder.bind(org_id);
        }
        if let Some(ref event_type) = filters.event_type {
            count_query_builder = count_query_builder.bind(Self::event_type_to_string(event_type));
        }
        if let Some(success) = filters.success {
            count_query_builder = count_query_builder.bind(success);
        }
        if let Some(start_date) = filters.start_date {
            count_query_builder = count_query_builder.bind(start_date);
        }
        if let Some(end_date) = filters.end_date {
            count_query_builder = count_query_builder.bind(end_date);
        }
        if let Some(ref resource_type) = filters.resource_type {
            count_query_builder = count_query_builder.bind(resource_type);
        }
        if let Some(resource_id) = filters.resource_id {
            count_query_builder = count_query_builder.bind(resource_id);
        }

        let count_row = count_query_builder
            .fetch_one(&self.pool)
            .await
            .map_err(|e| format!("Database error: {}", e))?;
        let total: i64 = count_row.get("count");

        // Fetch paginated results
        let data_query = format!(
            r#"
            SELECT id, timestamp, event_type, user_id, organization_id,
                   resource_type, resource_id, ip_address, user_agent,
                   metadata, success, error_message
            FROM audit_logs
            {}
            ORDER BY timestamp DESC
            LIMIT ${} OFFSET ${}
            "#,
            where_clause,
            param_index,
            param_index + 1
        );

        let mut data_query_builder = sqlx::query(&data_query);

        // Bind parameters for data query
        if let Some(user_id) = filters.user_id {
            data_query_builder = data_query_builder.bind(user_id);
        }
        if let Some(org_id) = filters.organization_id {
            data_query_builder = data_query_builder.bind(org_id);
        }
        if let Some(ref event_type) = filters.event_type {
            data_query_builder = data_query_builder.bind(Self::event_type_to_string(event_type));
        }
        if let Some(success) = filters.success {
            data_query_builder = data_query_builder.bind(success);
        }
        if let Some(start_date) = filters.start_date {
            data_query_builder = data_query_builder.bind(start_date);
        }
        if let Some(end_date) = filters.end_date {
            data_query_builder = data_query_builder.bind(end_date);
        }
        if let Some(ref resource_type) = filters.resource_type {
            data_query_builder = data_query_builder.bind(resource_type);
        }
        if let Some(resource_id) = filters.resource_id {
            data_query_builder = data_query_builder.bind(resource_id);
        }

        data_query_builder = data_query_builder.bind(limit).bind(offset);

        let rows = data_query_builder
            .fetch_all(&self.pool)
            .await
            .map_err(|e| format!("Database error: {}", e))?;

        let entries: Vec<AuditLogEntry> = rows.iter().map(|r| Self::row_to_entry(r)).collect();

        Ok((entries, total))
    }

    async fn find_recent(&self, limit: i64) -> Result<Vec<AuditLogEntry>, String> {
        let rows = sqlx::query(
            r#"
            SELECT id, timestamp, event_type, user_id, organization_id,
                   resource_type, resource_id, ip_address, user_agent,
                   metadata, success, error_message
            FROM audit_logs
            ORDER BY timestamp DESC
            LIMIT $1
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(rows.iter().map(|r| Self::row_to_entry(r)).collect())
    }

    async fn find_failed_operations(
        &self,
        page_request: &PageRequest,
        organization_id: Option<Uuid>,
    ) -> Result<(Vec<AuditLogEntry>, i64), String> {
        let limit = page_request.per_page.min(100);
        let offset = (page_request.page - 1) * limit;

        let where_clause = if organization_id.is_some() {
            "WHERE success = false AND organization_id = $1"
        } else {
            "WHERE success = false"
        };

        // Count
        let count_query = format!("SELECT COUNT(*) as count FROM audit_logs {}", where_clause);
        let count_row = if let Some(org_id) = organization_id {
            sqlx::query(&count_query)
                .bind(org_id)
                .fetch_one(&self.pool)
                .await
                .map_err(|e| format!("Database error: {}", e))?
        } else {
            sqlx::query(&count_query)
                .fetch_one(&self.pool)
                .await
                .map_err(|e| format!("Database error: {}", e))?
        };
        let total: i64 = count_row.get("count");

        // Fetch data
        let data_query = format!(
            r#"
            SELECT id, timestamp, event_type, user_id, organization_id,
                   resource_type, resource_id, ip_address, user_agent,
                   metadata, success, error_message
            FROM audit_logs
            {}
            ORDER BY timestamp DESC
            LIMIT $2 OFFSET $3
            "#,
            where_clause
        );

        let rows = if let Some(org_id) = organization_id {
            sqlx::query(&data_query)
                .bind(org_id)
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await
                .map_err(|e| format!("Database error: {}", e))?
        } else {
            sqlx::query(&data_query)
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await
                .map_err(|e| format!("Database error: {}", e))?
        };

        let entries: Vec<AuditLogEntry> = rows.iter().map(|r| Self::row_to_entry(r)).collect();

        Ok((entries, total))
    }

    async fn delete_older_than(&self, timestamp: DateTime<Utc>) -> Result<i64, String> {
        let result = sqlx::query(
            r#"
            DELETE FROM audit_logs
            WHERE timestamp < $1
            "#,
        )
        .bind(timestamp)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(result.rows_affected() as i64)
    }

    async fn count_by_filters(&self, filters: &AuditLogFilters) -> Result<i64, String> {
        let mut where_clauses = Vec::new();
        let mut param_index = 1;

        if filters.user_id.is_some() {
            where_clauses.push(format!("user_id = ${}", param_index));
            param_index += 1;
        }
        if filters.organization_id.is_some() {
            where_clauses.push(format!("organization_id = ${}", param_index));
            param_index += 1;
        }
        if filters.event_type.is_some() {
            where_clauses.push(format!("event_type = ${}", param_index));
            param_index += 1;
        }
        if filters.success.is_some() {
            where_clauses.push(format!("success = ${}", param_index));
            param_index += 1;
        }
        if filters.start_date.is_some() {
            where_clauses.push(format!("timestamp >= ${}", param_index));
            param_index += 1;
        }
        if filters.end_date.is_some() {
            where_clauses.push(format!("timestamp <= ${}", param_index));
            param_index += 1;
        }
        if filters.resource_type.is_some() {
            where_clauses.push(format!("resource_type = ${}", param_index));
            param_index += 1;
        }
        if filters.resource_id.is_some() {
            where_clauses.push(format!("resource_id = ${}", param_index));
        }

        let where_clause = if where_clauses.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", where_clauses.join(" AND "))
        };

        let count_query = format!("SELECT COUNT(*) as count FROM audit_logs {}", where_clause);
        let mut query_builder = sqlx::query(&count_query);

        // Bind parameters
        if let Some(user_id) = filters.user_id {
            query_builder = query_builder.bind(user_id);
        }
        if let Some(org_id) = filters.organization_id {
            query_builder = query_builder.bind(org_id);
        }
        if let Some(ref event_type) = filters.event_type {
            query_builder = query_builder.bind(Self::event_type_to_string(event_type));
        }
        if let Some(success) = filters.success {
            query_builder = query_builder.bind(success);
        }
        if let Some(start_date) = filters.start_date {
            query_builder = query_builder.bind(start_date);
        }
        if let Some(end_date) = filters.end_date {
            query_builder = query_builder.bind(end_date);
        }
        if let Some(ref resource_type) = filters.resource_type {
            query_builder = query_builder.bind(resource_type);
        }
        if let Some(resource_id) = filters.resource_id {
            query_builder = query_builder.bind(resource_id);
        }

        let row = query_builder
            .fetch_one(&self.pool)
            .await
            .map_err(|e| format!("Database error: {}", e))?;

        Ok(row.get("count"))
    }
}
