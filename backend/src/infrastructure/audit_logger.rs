use crate::application::ports::AuditLogRepository;
use crate::infrastructure::audit::AuditLogEntry;
use std::sync::Arc;

/// AuditLogger manages both stdout logging and database persistence
pub struct AuditLogger {
    repository: Option<Arc<dyn AuditLogRepository>>,
}

impl AuditLogger {
    /// Create a new AuditLogger with optional database persistence
    pub fn new(repository: Option<Arc<dyn AuditLogRepository>>) -> Self {
        Self { repository }
    }

    /// Log an audit entry to stdout and optionally persist to database
    pub async fn log(&self, entry: &AuditLogEntry) {
        // Always log to stdout for real-time monitoring
        entry.log();

        // Persist to database if repository is available
        if let Some(repo) = &self.repository {
            if let Err(e) = repo.create(entry).await {
                log::error!("[AUDIT] Failed to persist audit log to database: {}", e);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::audit::{AuditEventType, AuditLogEntry};
    use uuid::Uuid;

    #[tokio::test]
    async fn test_audit_logger_without_repository() {
        let logger = AuditLogger::new(None);
        let entry = AuditLogEntry::new(
            AuditEventType::UserLogin,
            Some(Uuid::new_v4()),
            Some(Uuid::new_v4()),
        );

        // Should not panic even without repository
        logger.log(&entry).await;
    }
}
