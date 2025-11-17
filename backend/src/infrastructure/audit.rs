use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Audit log event types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
    JournalEntryCreated,
    JournalEntryDeleted,
    UnitCreated,
    UnitUpdated,
    UnitDeleted,
    UnitAssignedToOwner,
    UnitOwnerCreated,
    UnitOwnerUpdated,
    UnitOwnerDeleted,
    OwnerCreated,
    OwnerUpdated,
    ExpenseCreated,
    ExpenseMarkedPaid,
    MeetingCreated,
    MeetingCompleted,
    DocumentUploaded,
    DocumentDeleted,

    // Board management events
    BoardMemberElected,
    BoardMemberRemoved,
    BoardMemberMandateRenewed,
    BoardDecisionCreated,
    BoardDecisionUpdated,
    BoardDecisionCompleted,
    BoardDecisionNotesAdded,

    // Voting events (Issue #46 - Phase 2)
    ResolutionCreated,
    ResolutionDeleted,
    VoteCast,
    VoteChanged,
    VotingClosed,

    // Ticketing events (Issue #85 - Phase 2)
    TicketCreated,
    TicketAssigned,
    TicketStatusChanged,
    TicketResolved,
    TicketClosed,
    TicketCancelled,
    TicketReopened,
    TicketDeleted,

    // Notification events (Issue #86 - Phase 2)
    NotificationCreated,
    NotificationRead,
    NotificationDeleted,
    NotificationPreferenceUpdated,

    // Payment events (Issue #84 - Phase 2)
    PaymentCreated,
    PaymentProcessing,
    PaymentRequiresAction,
    PaymentSucceeded,
    PaymentFailed,
    PaymentCancelled,
    PaymentRefunded,
    PaymentDeleted,

    // Payment method events (Issue #84 - Phase 2)
    PaymentMethodCreated,
    PaymentMethodUpdated,
    PaymentMethodSetDefault,
    PaymentMethodDeactivated,
    PaymentMethodReactivated,
    PaymentMethodDeleted,

    // Convocation events (Issue #88 - Phase 2)
    ConvocationCreated,
    ConvocationScheduled,
    ConvocationSent,
    ConvocationCancelled,
    ConvocationDeleted,
    ConvocationReminderSent,
    ConvocationAttendanceUpdated,
    ConvocationProxySet,

    // Quote events (Contractor Quotes Module - Issue #91 - Phase 2)
    QuoteCreated,
    QuoteSubmitted,
    QuoteUnderReview,
    QuoteAccepted,
    QuoteRejected,
    QuoteWithdrawn,
    QuoteExpired,
    QuoteRatingUpdated,
    QuoteComparisonPerformed,
    QuoteDeleted,

    // SEL events (Local Exchange Trading System - Issue #49 - Phase 2)
    ExchangeCreated,
    ExchangeRequested,
    ExchangeStarted,
    ExchangeCompleted,
    ExchangeCancelled,
    ExchangeProviderRated,
    ExchangeRequesterRated,
    ExchangeDeleted,
    CreditBalanceUpdated,
    CreditBalanceCreated,

    // Notice events (Community Notice Board - Issue #49 - Phase 2)
    NoticeCreated,
    NoticeUpdated,
    NoticePublished,
    NoticeArchived,
    NoticePinned,
    NoticeUnpinned,
    NoticeExpirationSet,
    NoticeExpired,
    NoticeDeleted,

    // Skill events (Skills Directory - Issue #49 - Phase 3)
    SkillCreated,
    SkillUpdated,
    SkillMarkedAvailable,
    SkillMarkedUnavailable,
    SkillDeleted,

    // Shared Object events (Object Sharing Library - Issue #49 - Phase 4)
    SharedObjectCreated,
    SharedObjectUpdated,
    SharedObjectMarkedAvailable,
    SharedObjectMarkedUnavailable,
    SharedObjectBorrowed,
    SharedObjectReturned,
    SharedObjectDeleted,

    // Resource Booking events (Resource Booking Calendar - Issue #49 - Phase 5)
    ResourceBookingCreated,
    ResourceBookingUpdated,
    ResourceBookingCancelled,
    ResourceBookingCompleted,
    ResourceBookingNoShow,
    ResourceBookingConfirmed,
    ResourceBookingDeleted,

    // Gamification events (Achievements & Challenges - Issue #49 - Phase 6)
    AchievementCreated,
    AchievementUpdated,
    AchievementDeleted,
    AchievementAwarded,
    ChallengeCreated,
    ChallengeActivated,
    ChallengeUpdated,
    ChallengeCompleted,
    ChallengeCancelled,
    ChallengeDeleted,
    ChallengeProgressIncremented,
    ChallengeProgressCompleted,

    // Payment reminder events
    PaymentReminderCreated,
    PaymentReminderSent,
    PaymentReminderOpened,
    PaymentReminderPaid,
    PaymentReminderCancelled,
    PaymentReminderEscalated,
    PaymentReminderTrackingAdded,
    PaymentRemindersBulkCreated,
    PaymentReminderDeleted,

    // État Daté events (Belgian legal requirement for property sales)
    EtatDateCreated,
    EtatDateInProgress,
    EtatDateGenerated,
    EtatDateDelivered,
    EtatDateFinancialUpdate,
    EtatDateAdditionalDataUpdate,
    EtatDateDeleted,

    // Budget events (Annual budget management)
    BudgetCreated,
    BudgetUpdated,
    BudgetSubmitted,
    BudgetApproved,
    BudgetRejected,
    BudgetArchived,
    BudgetDeleted,

    // Security events
    UnauthorizedAccess,
    RateLimitExceeded,
    InvalidToken,

    // GDPR events (Data Privacy Compliance - Article 30: Records of Processing)
    GdprDataExported,
    GdprDataExportFailed,
    GdprDataErased,
    GdprDataErasureFailed,
    GdprErasureCheckRequested,
    // GDPR Article 16: Right to Rectification
    GdprDataRectified,
    GdprDataRectificationFailed,
    // GDPR Article 18: Right to Restriction of Processing
    GdprProcessingRestricted,
    GdprProcessingRestrictionFailed,
    // GDPR Article 21: Right to Object (Marketing)
    GdprMarketingOptOut,
    GdprMarketingOptIn,
    GdprMarketingPreferenceChangeFailed,

    // Accounting events
    AccountCreated,
    AccountUpdated,
    AccountDeleted,
    BelgianPCMNSeeded,

    // Financial reporting events
    ReportGenerated,
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

    /// Add details to metadata as a string
    pub fn with_details(mut self, details: String) -> Self {
        let details_json = serde_json::json!({ "details": details });
        self.metadata = Some(details_json);
        self
    }

    /// Log this entry (currently to stdout, can be extended to database/file)
    pub fn log(&self) {
        // Redact sensitive information for logging (GDPR compliance)
        let redact_presence = |present| if present { "[REDACTED]" } else { "None" };
        let redacted_user = redact_presence(self.user_id.is_some());
        let redacted_org = redact_presence(self.organization_id.is_some());
        let redacted_resource_id = redact_presence(self.resource_id.is_some());
        let redacted_ip = redact_presence(self.ip_address.is_some());
        let redacted_error = self.error_message.as_ref().map(|_| "[REDACTED]");

        let log_message = format!(
            "[AUDIT] {} | {:?} | User: {} | Org: {} | Resource: {}/{} | Success: {} | IP: {}",
            self.timestamp.format("%Y-%m-%d %H:%M:%S"),
            self.event_type,
            redacted_user,
            redacted_org,
            self.resource_type.as_deref().unwrap_or("None"),
            redacted_resource_id,
            self.success,
            redacted_ip
        );

        if self.success {
            log::info!("{}", log_message);
        } else {
            log::warn!(
                "{} | Error: {}",
                log_message,
                redacted_error.unwrap_or("None")
            );
        }

        // TODO: In production, write full (unredacted) audit data to:
        // - Database table (audit_logs) with encryption at rest
        // - Rotating log files in secure location with restricted access
        // - SIEM system (Security Information and Event Management)
        // Note: Full audit data (including IP, error messages) should only be
        // stored in secure, access-controlled systems for compliance and forensics
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
