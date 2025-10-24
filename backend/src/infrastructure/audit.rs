use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Audit log event types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditEventType {
    // Authentication events
    UserLogin,
    UserLogout,
    UserRegistration,
    TokenRefresh,
    AuthenticationFailed,

    // Data modification events
    BuildingCreated,
    BuildingUpdated,
    BuildingDeleted,
    UnitCreated,
    UnitAssignedToOwner,
    OwnerCreated,
    OwnerUpdated,
    ExpenseCreated,
    ExpenseMarkedPaid,
    MeetingCreated,
    MeetingCompleted,
    DocumentUploaded,
    DocumentDeleted,

    // Security events
    UnauthorizedAccess,
    RateLimitExceeded,
    InvalidToken,
}

/// Audit log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEntry {
    /// Unique ID for this audit entry
    pub id: Uuid,
    /// Timestamp of the event
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Type of event
    pub event_type: AuditEventType,
    /// User ID who performed the action (if authenticated)
    pub user_id: Option<Uuid>,
    /// Organization ID (for multi-tenant isolation)
    pub organization_id: Option<Uuid>,
    /// Resource type affected (e.g., "Building", "Unit")
    pub resource_type: Option<String>,
    /// Resource ID affected
    pub resource_id: Option<Uuid>,
    /// IP address of the client
    pub ip_address: Option<String>,
    /// User agent string
    pub user_agent: Option<String>,
    /// Additional metadata as JSON
    pub metadata: Option<serde_json::Value>,
    /// Success or failure
    pub success: bool,
    /// Error message if failed
    pub error_message: Option<String>,
}

impl AuditLogEntry {
    /// Create a new audit log entry
    pub fn new(
        event_type: AuditEventType,
        user_id: Option<Uuid>,
        organization_id: Option<Uuid>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            event_type,
            user_id,
            organization_id,
            resource_type: None,
            resource_id: None,
            ip_address: None,
            user_agent: None,
            metadata: None,
            success: true,
            error_message: None,
        }
    }

    /// Set resource information
    pub fn with_resource(mut self, resource_type: &str, resource_id: Uuid) -> Self {
        self.resource_type = Some(resource_type.to_string());
        self.resource_id = Some(resource_id);
        self
    }

    /// Set client information
    pub fn with_client_info(
        mut self,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Self {
        self.ip_address = ip_address;
        self.user_agent = user_agent;
        self
    }

    /// Set metadata
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = Some(metadata);
        self
    }

    /// Mark as failed with error message
    pub fn with_error(mut self, error_message: String) -> Self {
        self.success = false;
        self.error_message = Some(error_message);
        self
    }

    /// Log this entry (currently to stdout, can be extended to database/file)
    pub fn log(&self) {
        let log_message = format!(
            "[AUDIT] {} | {:?} | User: {:?} | Org: {:?} | Resource: {:?}/{:?} | Success: {} | IP: {:?}",
            self.timestamp.format("%Y-%m-%d %H:%M:%S"),
            self.event_type,
            self.user_id,
            self.organization_id,
            self.resource_type,
            self.resource_id,
            self.success,
            self.ip_address
        );

        if self.success {
            log::info!("{}", log_message);
        } else {
            log::warn!("{} | Error: {:?}", log_message, self.error_message);
        }

        // TODO: In production, write to:
        // - Database table (audit_logs)
        // - Rotating log files
        // - SIEM system (Security Information and Event Management)
    }
}

/// Helper macro to create and log audit entries
#[macro_export]
macro_rules! audit_log {
    ($event_type:expr, $user_id:expr, $org_id:expr) => {
        $crate::infrastructure::audit::AuditLogEntry::new($event_type, $user_id, $org_id).log()
    };
    ($event_type:expr, $user_id:expr, $org_id:expr, $resource_type:expr, $resource_id:expr) => {
        $crate::infrastructure::audit::AuditLogEntry::new($event_type, $user_id, $org_id)
            .with_resource($resource_type, $resource_id)
            .log()
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_log_creation() {
        let user_id = Uuid::new_v4();
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();

        let entry =
            AuditLogEntry::new(AuditEventType::BuildingCreated, Some(user_id), Some(org_id))
                .with_resource("Building", building_id)
                .with_client_info(Some("192.168.1.1".to_string()), None);

        assert_eq!(entry.user_id, Some(user_id));
        assert_eq!(entry.organization_id, Some(org_id));
        assert_eq!(entry.resource_id, Some(building_id));
        assert!(entry.success);
    }

    #[test]
    fn test_audit_log_with_error() {
        let entry = AuditLogEntry::new(AuditEventType::AuthenticationFailed, None, None)
            .with_error("Invalid credentials".to_string());

        assert!(!entry.success);
        assert_eq!(entry.error_message, Some("Invalid credentials".to_string()));
    }
}
