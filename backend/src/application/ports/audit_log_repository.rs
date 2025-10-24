use crate::application::dto::PageRequest;
use crate::infrastructure::audit::{AuditEventType, AuditLogEntry};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Filters for querying audit logs
#[derive(Debug, Clone, Default)]
pub struct AuditLogFilters {
    pub user_id: Option<Uuid>,
    pub organization_id: Option<Uuid>,
    pub event_type: Option<AuditEventType>,
    pub success: Option<bool>,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub resource_type: Option<String>,
    pub resource_id: Option<Uuid>,
}

/// Port (interface) for audit log repository
/// Handles persistence of audit events for security, compliance, and debugging
#[async_trait]
pub trait AuditLogRepository: Send + Sync {
    /// Create a new audit log entry
    async fn create(&self, entry: &AuditLogEntry) -> Result<AuditLogEntry, String>;

    /// Find audit log by ID
    async fn find_by_id(&self, id: Uuid) -> Result<Option<AuditLogEntry>, String>;

    /// Find all audit logs with pagination and filters
    /// Returns tuple of (logs, total_count)
    async fn find_all_paginated(
        &self,
        page_request: &PageRequest,
        filters: &AuditLogFilters,
    ) -> Result<(Vec<AuditLogEntry>, i64), String>;

    /// Find recent audit logs (last N entries)
    async fn find_recent(&self, limit: i64) -> Result<Vec<AuditLogEntry>, String>;

    /// Find failed operations (for security monitoring)
    async fn find_failed_operations(
        &self,
        page_request: &PageRequest,
        organization_id: Option<Uuid>,
    ) -> Result<(Vec<AuditLogEntry>, i64), String>;

    /// Delete old audit logs (for data retention policies)
    /// Deletes logs older than the specified timestamp
    /// Returns number of deleted entries
    async fn delete_older_than(&self, timestamp: DateTime<Utc>) -> Result<i64, String>;

    /// Count audit logs by filters
    async fn count_by_filters(&self, filters: &AuditLogFilters) -> Result<i64, String>;
}
