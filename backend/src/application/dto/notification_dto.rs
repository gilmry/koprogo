use crate::domain::entities::{
    Notification, NotificationChannel, NotificationPreference, NotificationPriority,
    NotificationStatus, NotificationType,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Notification Response DTO
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NotificationResponse {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub user_id: Uuid,
    pub notification_type: NotificationType,
    pub channel: NotificationChannel,
    pub priority: NotificationPriority,
    pub status: NotificationStatus,
    pub title: String,
    pub message: String,
    pub link_url: Option<String>,
    pub metadata: Option<String>,
    pub sent_at: Option<DateTime<Utc>>,
    pub read_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub error_message: Option<String>,
}

impl From<Notification> for NotificationResponse {
    fn from(notification: Notification) -> Self {
        Self {
            id: notification.id,
            organization_id: notification.organization_id,
            user_id: notification.user_id,
            notification_type: notification.notification_type,
            channel: notification.channel,
            priority: notification.priority,
            status: notification.status,
            title: notification.title,
            message: notification.message,
            link_url: notification.link_url,
            metadata: notification.metadata,
            sent_at: notification.sent_at,
            read_at: notification.read_at,
            created_at: notification.created_at,
            error_message: notification.error_message,
        }
    }
}

/// Create Notification Request
#[derive(Debug, Deserialize)]
pub struct CreateNotificationRequest {
    pub user_id: Uuid,
    pub notification_type: NotificationType,
    pub channel: NotificationChannel,
    pub priority: NotificationPriority,
    pub title: String,
    pub message: String,
    pub link_url: Option<String>,
    pub metadata: Option<String>,
}

/// Mark Notification as Read Request (for in-app only)
#[derive(Debug, Deserialize)]
pub struct MarkReadRequest {
    // Empty - just marks the notification as read
}

/// Update Notification Preference Request
#[derive(Debug, Deserialize)]
pub struct UpdatePreferenceRequest {
    pub email_enabled: Option<bool>,
    pub in_app_enabled: Option<bool>,
    pub push_enabled: Option<bool>,
}

/// Notification Preference Response DTO
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NotificationPreferenceResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub notification_type: NotificationType,
    pub email_enabled: bool,
    pub in_app_enabled: bool,
    pub push_enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<NotificationPreference> for NotificationPreferenceResponse {
    fn from(preference: NotificationPreference) -> Self {
        Self {
            id: preference.id,
            user_id: preference.user_id,
            notification_type: preference.notification_type,
            email_enabled: preference.email_enabled,
            in_app_enabled: preference.in_app_enabled,
            push_enabled: preference.push_enabled,
            created_at: preference.created_at,
            updated_at: preference.updated_at,
        }
    }
}

/// Notification Statistics
#[derive(Debug, Serialize)]
pub struct NotificationStats {
    pub total: i64,
    pub unread: i64,
    pub pending: i64,
    pub sent: i64,
    pub failed: i64,
}
