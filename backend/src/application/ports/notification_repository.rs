use crate::domain::entities::{Notification, NotificationChannel, NotificationStatus, NotificationType};
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait NotificationRepository: Send + Sync {
    /// Create a new notification
    async fn create(&self, notification: &Notification) -> Result<Notification, String>;

    /// Find notification by ID
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Notification>, String>;

    /// Find notifications by user
    async fn find_by_user(&self, user_id: Uuid) -> Result<Vec<Notification>, String>;

    /// Find notifications by user and status
    async fn find_by_user_and_status(
        &self,
        user_id: Uuid,
        status: NotificationStatus,
    ) -> Result<Vec<Notification>, String>;

    /// Find notifications by user and channel
    async fn find_by_user_and_channel(
        &self,
        user_id: Uuid,
        channel: NotificationChannel,
    ) -> Result<Vec<Notification>, String>;

    /// Find unread in-app notifications for user
    async fn find_unread_by_user(&self, user_id: Uuid) -> Result<Vec<Notification>, String>;

    /// Find pending notifications (to be sent)
    async fn find_pending(&self) -> Result<Vec<Notification>, String>;

    /// Find failed notifications (for retry)
    async fn find_failed(&self) -> Result<Vec<Notification>, String>;

    /// Find notifications by organization
    async fn find_by_organization(
        &self,
        organization_id: Uuid,
    ) -> Result<Vec<Notification>, String>;

    /// Update notification
    async fn update(&self, notification: &Notification) -> Result<Notification, String>;

    /// Delete notification
    async fn delete(&self, id: Uuid) -> Result<bool, String>;

    /// Count unread notifications for user
    async fn count_unread_by_user(&self, user_id: Uuid) -> Result<i64, String>;

    /// Count notifications by user and status
    async fn count_by_user_and_status(
        &self,
        user_id: Uuid,
        status: NotificationStatus,
    ) -> Result<i64, String>;

    /// Mark all in-app notifications as read for user
    async fn mark_all_read_by_user(&self, user_id: Uuid) -> Result<i64, String>;

    /// Delete old notifications (cleanup)
    async fn delete_older_than(&self, days: i64) -> Result<i64, String>;
}
