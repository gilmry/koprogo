use crate::domain::entities::{NotificationChannel, NotificationPreference, NotificationType};
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait NotificationPreferenceRepository: Send + Sync {
    /// Create a new notification preference
    async fn create(
        &self,
        preference: &NotificationPreference,
    ) -> Result<NotificationPreference, String>;

    /// Find preference by ID
    async fn find_by_id(&self, id: Uuid) -> Result<Option<NotificationPreference>, String>;

    /// Find preference by user and notification type
    async fn find_by_user_and_type(
        &self,
        user_id: Uuid,
        notification_type: NotificationType,
    ) -> Result<Option<NotificationPreference>, String>;

    /// Find all preferences for a user
    async fn find_by_user(&self, user_id: Uuid) -> Result<Vec<NotificationPreference>, String>;

    /// Update preference
    async fn update(
        &self,
        preference: &NotificationPreference,
    ) -> Result<NotificationPreference, String>;

    /// Delete preference
    async fn delete(&self, id: Uuid) -> Result<bool, String>;

    /// Check if user has enabled a specific channel for a notification type
    async fn is_channel_enabled(
        &self,
        user_id: Uuid,
        notification_type: NotificationType,
        channel: NotificationChannel,
    ) -> Result<bool, String>;

    /// Create default preferences for a new user
    async fn create_defaults_for_user(&self, user_id: Uuid) -> Result<Vec<NotificationPreference>, String>;
}
