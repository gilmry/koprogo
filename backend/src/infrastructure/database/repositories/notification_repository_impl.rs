use crate::application::ports::NotificationRepository;
use crate::domain::entities::{
    Notification, NotificationChannel, NotificationPriority, NotificationStatus, NotificationType,
};
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

/// PostgreSQL implementation of NotificationRepository
pub struct PostgresNotificationRepository {
    pool: PgPool,
}

impl PostgresNotificationRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Convert NotificationType enum to database string
    fn type_to_db(notification_type: &NotificationType) -> &'static str {
        match notification_type {
            NotificationType::ExpenseCreated => "ExpenseCreated",
            NotificationType::MeetingConvocation => "MeetingConvocation",
            NotificationType::PaymentReceived => "PaymentReceived",
            NotificationType::TicketResolved => "TicketResolved",
            NotificationType::DocumentAdded => "DocumentAdded",
            NotificationType::BoardMessage => "BoardMessage",
            NotificationType::PaymentReminder => "PaymentReminder",
            NotificationType::BudgetApproved => "BudgetApproved",
            NotificationType::ResolutionVote => "ResolutionVote",
            NotificationType::System => "System",
        }
    }

    /// Convert database string to NotificationType enum
    fn type_from_db(s: &str) -> Result<NotificationType, String> {
        match s {
            "ExpenseCreated" => Ok(NotificationType::ExpenseCreated),
            "MeetingConvocation" => Ok(NotificationType::MeetingConvocation),
            "PaymentReceived" => Ok(NotificationType::PaymentReceived),
            "TicketResolved" => Ok(NotificationType::TicketResolved),
            "DocumentAdded" => Ok(NotificationType::DocumentAdded),
            "BoardMessage" => Ok(NotificationType::BoardMessage),
            "PaymentReminder" => Ok(NotificationType::PaymentReminder),
            "BudgetApproved" => Ok(NotificationType::BudgetApproved),
            "ResolutionVote" => Ok(NotificationType::ResolutionVote),
            "System" => Ok(NotificationType::System),
            _ => Err(format!("Invalid notification type: {}", s)),
        }
    }

    /// Convert NotificationChannel enum to database string
    fn channel_to_db(channel: &NotificationChannel) -> &'static str {
        match channel {
            NotificationChannel::Email => "Email",
            NotificationChannel::InApp => "InApp",
            NotificationChannel::Push => "Push",
        }
    }

    /// Convert database string to NotificationChannel enum
    fn channel_from_db(s: &str) -> Result<NotificationChannel, String> {
        match s {
            "Email" => Ok(NotificationChannel::Email),
            "InApp" => Ok(NotificationChannel::InApp),
            "Push" => Ok(NotificationChannel::Push),
            _ => Err(format!("Invalid notification channel: {}", s)),
        }
    }

    /// Convert NotificationPriority enum to database string
    fn priority_to_db(priority: &NotificationPriority) -> &'static str {
        match priority {
            NotificationPriority::Low => "Low",
            NotificationPriority::Medium => "Medium",
            NotificationPriority::High => "High",
            NotificationPriority::Critical => "Critical",
        }
    }

    /// Convert database string to NotificationPriority enum
    fn priority_from_db(s: &str) -> Result<NotificationPriority, String> {
        match s {
            "Low" => Ok(NotificationPriority::Low),
            "Medium" => Ok(NotificationPriority::Medium),
            "High" => Ok(NotificationPriority::High),
            "Critical" => Ok(NotificationPriority::Critical),
            _ => Err(format!("Invalid notification priority: {}", s)),
        }
    }

    /// Convert NotificationStatus enum to database string
    fn status_to_db(status: &NotificationStatus) -> &'static str {
        match status {
            NotificationStatus::Pending => "Pending",
            NotificationStatus::Sent => "Sent",
            NotificationStatus::Failed => "Failed",
            NotificationStatus::Read => "Read",
        }
    }

    /// Convert database string to NotificationStatus enum
    fn status_from_db(s: &str) -> Result<NotificationStatus, String> {
        match s {
            "Pending" => Ok(NotificationStatus::Pending),
            "Sent" => Ok(NotificationStatus::Sent),
            "Failed" => Ok(NotificationStatus::Failed),
            "Read" => Ok(NotificationStatus::Read),
            _ => Err(format!("Invalid notification status: {}", s)),
        }
    }
}

#[async_trait]
impl NotificationRepository for PostgresNotificationRepository {
    async fn create(&self, notification: &Notification) -> Result<Notification, String> {
        let type_str = Self::type_to_db(&notification.notification_type);
        let channel_str = Self::channel_to_db(&notification.channel);
        let priority_str = Self::priority_to_db(&notification.priority);
        let status_str = Self::status_to_db(&notification.status);

        let row = sqlx::query!(
            r#"
            INSERT INTO notifications (
                id, organization_id, user_id, notification_type, channel, priority, status,
                title, message, link_url, metadata, sent_at, read_at, created_at, error_message
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
            RETURNING id, organization_id, user_id, notification_type, channel, priority, status,
                      title, message, link_url, metadata, sent_at, read_at, created_at, error_message
            "#,
            notification.id,
            notification.organization_id,
            notification.user_id,
            type_str,
            channel_str,
            priority_str,
            status_str,
            notification.title,
            notification.message,
            notification.link_url,
            notification.metadata.as_ref().map(|m| serde_json::from_str::<serde_json::Value>(m).ok()).flatten(),
            notification.sent_at,
            notification.read_at,
            notification.created_at,
            notification.error_message
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Database error creating notification: {}", e))?;

        Ok(Notification {
            id: row.id,
            organization_id: row.organization_id,
            user_id: row.user_id,
            notification_type: Self::type_from_db(&row.notification_type)?,
            channel: Self::channel_from_db(&row.channel)?,
            priority: Self::priority_from_db(&row.priority)?,
            status: Self::status_from_db(&row.status)?,
            title: row.title,
            message: row.message,
            link_url: row.link_url,
            metadata: row.metadata.map(|m| m.to_string()),
            sent_at: row.sent_at,
            read_at: row.read_at,
            created_at: row.created_at,
            error_message: row.error_message,
        })
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Notification>, String> {
        let row = sqlx::query!(
            r#"
            SELECT id, organization_id, user_id, notification_type, channel, priority, status,
                   title, message, link_url, metadata, sent_at, read_at, created_at, error_message
            FROM notifications
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Database error finding notification: {}", e))?;

        match row {
            Some(r) => Ok(Some(Notification {
                id: r.id,
                organization_id: r.organization_id,
                user_id: r.user_id,
                notification_type: Self::type_from_db(&r.notification_type)?,
                channel: Self::channel_from_db(&r.channel)?,
                priority: Self::priority_from_db(&r.priority)?,
                status: Self::status_from_db(&r.status)?,
                title: r.title,
                message: r.message,
                link_url: r.link_url,
                metadata: r.metadata.map(|m| m.to_string()),
                sent_at: r.sent_at,
                read_at: r.read_at,
                created_at: r.created_at,
                error_message: r.error_message,
            })),
            None => Ok(None),
        }
    }

    async fn find_by_user(&self, user_id: Uuid) -> Result<Vec<Notification>, String> {
        let rows = sqlx::query!(
            r#"
            SELECT id, organization_id, user_id, notification_type, channel, priority, status,
                   title, message, link_url, metadata, sent_at, read_at, created_at, error_message
            FROM notifications
            WHERE user_id = $1
            ORDER BY created_at DESC
            "#,
            user_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Database error finding notifications by user: {}", e))?;

        rows.into_iter()
            .map(|r| {
                Ok(Notification {
                    id: r.id,
                    organization_id: r.organization_id,
                    user_id: r.user_id,
                    notification_type: Self::type_from_db(&r.notification_type)?,
                    channel: Self::channel_from_db(&r.channel)?,
                    priority: Self::priority_from_db(&r.priority)?,
                    status: Self::status_from_db(&r.status)?,
                    title: r.title,
                    message: r.message,
                    link_url: r.link_url,
                    metadata: r.metadata.map(|m| m.to_string()),
                    sent_at: r.sent_at,
                    read_at: r.read_at,
                    created_at: r.created_at,
                    error_message: r.error_message,
                })
            })
            .collect()
    }

    async fn find_by_user_and_status(
        &self,
        user_id: Uuid,
        status: NotificationStatus,
    ) -> Result<Vec<Notification>, String> {
        let status_str = Self::status_to_db(&status);

        let rows = sqlx::query!(
            r#"
            SELECT id, organization_id, user_id, notification_type, channel, priority, status,
                   title, message, link_url, metadata, sent_at, read_at, created_at, error_message
            FROM notifications
            WHERE user_id = $1 AND status = $2
            ORDER BY created_at DESC
            "#,
            user_id,
            status_str
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            format!(
                "Database error finding notifications by user and status: {}",
                e
            )
        })?;

        rows.into_iter()
            .map(|r| {
                Ok(Notification {
                    id: r.id,
                    organization_id: r.organization_id,
                    user_id: r.user_id,
                    notification_type: Self::type_from_db(&r.notification_type)?,
                    channel: Self::channel_from_db(&r.channel)?,
                    priority: Self::priority_from_db(&r.priority)?,
                    status: Self::status_from_db(&r.status)?,
                    title: r.title,
                    message: r.message,
                    link_url: r.link_url,
                    metadata: r.metadata.map(|m| m.to_string()),
                    sent_at: r.sent_at,
                    read_at: r.read_at,
                    created_at: r.created_at,
                    error_message: r.error_message,
                })
            })
            .collect()
    }

    async fn find_by_user_and_channel(
        &self,
        user_id: Uuid,
        channel: NotificationChannel,
    ) -> Result<Vec<Notification>, String> {
        let channel_str = Self::channel_to_db(&channel);

        let rows = sqlx::query!(
            r#"
            SELECT id, organization_id, user_id, notification_type, channel, priority, status,
                   title, message, link_url, metadata, sent_at, read_at, created_at, error_message
            FROM notifications
            WHERE user_id = $1 AND channel = $2
            ORDER BY created_at DESC
            "#,
            user_id,
            channel_str
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            format!(
                "Database error finding notifications by user and channel: {}",
                e
            )
        })?;

        rows.into_iter()
            .map(|r| {
                Ok(Notification {
                    id: r.id,
                    organization_id: r.organization_id,
                    user_id: r.user_id,
                    notification_type: Self::type_from_db(&r.notification_type)?,
                    channel: Self::channel_from_db(&r.channel)?,
                    priority: Self::priority_from_db(&r.priority)?,
                    status: Self::status_from_db(&r.status)?,
                    title: r.title,
                    message: r.message,
                    link_url: r.link_url,
                    metadata: r.metadata.map(|m| m.to_string()),
                    sent_at: r.sent_at,
                    read_at: r.read_at,
                    created_at: r.created_at,
                    error_message: r.error_message,
                })
            })
            .collect()
    }

    async fn find_unread_by_user(&self, user_id: Uuid) -> Result<Vec<Notification>, String> {
        let rows = sqlx::query!(
            r#"
            SELECT id, organization_id, user_id, notification_type, channel, priority, status,
                   title, message, link_url, metadata, sent_at, read_at, created_at, error_message
            FROM notifications
            WHERE user_id = $1 AND channel = 'InApp' AND status = 'Sent' AND read_at IS NULL
            ORDER BY created_at DESC
            "#,
            user_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Database error finding unread notifications: {}", e))?;

        rows.into_iter()
            .map(|r| {
                Ok(Notification {
                    id: r.id,
                    organization_id: r.organization_id,
                    user_id: r.user_id,
                    notification_type: Self::type_from_db(&r.notification_type)?,
                    channel: Self::channel_from_db(&r.channel)?,
                    priority: Self::priority_from_db(&r.priority)?,
                    status: Self::status_from_db(&r.status)?,
                    title: r.title,
                    message: r.message,
                    link_url: r.link_url,
                    metadata: r.metadata.map(|m| m.to_string()),
                    sent_at: r.sent_at,
                    read_at: r.read_at,
                    created_at: r.created_at,
                    error_message: r.error_message,
                })
            })
            .collect()
    }

    async fn find_pending(&self) -> Result<Vec<Notification>, String> {
        let rows = sqlx::query!(
            r#"
            SELECT id, organization_id, user_id, notification_type, channel, priority, status,
                   title, message, link_url, metadata, sent_at, read_at, created_at, error_message
            FROM notifications
            WHERE status = 'Pending'
            ORDER BY priority DESC, created_at ASC
            "#
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Database error finding pending notifications: {}", e))?;

        rows.into_iter()
            .map(|r| {
                Ok(Notification {
                    id: r.id,
                    organization_id: r.organization_id,
                    user_id: r.user_id,
                    notification_type: Self::type_from_db(&r.notification_type)?,
                    channel: Self::channel_from_db(&r.channel)?,
                    priority: Self::priority_from_db(&r.priority)?,
                    status: Self::status_from_db(&r.status)?,
                    title: r.title,
                    message: r.message,
                    link_url: r.link_url,
                    metadata: r.metadata.map(|m| m.to_string()),
                    sent_at: r.sent_at,
                    read_at: r.read_at,
                    created_at: r.created_at,
                    error_message: r.error_message,
                })
            })
            .collect()
    }

    async fn find_failed(&self) -> Result<Vec<Notification>, String> {
        let rows = sqlx::query!(
            r#"
            SELECT id, organization_id, user_id, notification_type, channel, priority, status,
                   title, message, link_url, metadata, sent_at, read_at, created_at, error_message
            FROM notifications
            WHERE status = 'Failed'
            ORDER BY created_at ASC
            "#
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Database error finding failed notifications: {}", e))?;

        rows.into_iter()
            .map(|r| {
                Ok(Notification {
                    id: r.id,
                    organization_id: r.organization_id,
                    user_id: r.user_id,
                    notification_type: Self::type_from_db(&r.notification_type)?,
                    channel: Self::channel_from_db(&r.channel)?,
                    priority: Self::priority_from_db(&r.priority)?,
                    status: Self::status_from_db(&r.status)?,
                    title: r.title,
                    message: r.message,
                    link_url: r.link_url,
                    metadata: r.metadata.map(|m| m.to_string()),
                    sent_at: r.sent_at,
                    read_at: r.read_at,
                    created_at: r.created_at,
                    error_message: r.error_message,
                })
            })
            .collect()
    }

    async fn find_by_organization(
        &self,
        organization_id: Uuid,
    ) -> Result<Vec<Notification>, String> {
        let rows = sqlx::query!(
            r#"
            SELECT id, organization_id, user_id, notification_type, channel, priority, status,
                   title, message, link_url, metadata, sent_at, read_at, created_at, error_message
            FROM notifications
            WHERE organization_id = $1
            ORDER BY created_at DESC
            "#,
            organization_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            format!(
                "Database error finding notifications by organization: {}",
                e
            )
        })?;

        rows.into_iter()
            .map(|r| {
                Ok(Notification {
                    id: r.id,
                    organization_id: r.organization_id,
                    user_id: r.user_id,
                    notification_type: Self::type_from_db(&r.notification_type)?,
                    channel: Self::channel_from_db(&r.channel)?,
                    priority: Self::priority_from_db(&r.priority)?,
                    status: Self::status_from_db(&r.status)?,
                    title: r.title,
                    message: r.message,
                    link_url: r.link_url,
                    metadata: r.metadata.map(|m| m.to_string()),
                    sent_at: r.sent_at,
                    read_at: r.read_at,
                    created_at: r.created_at,
                    error_message: r.error_message,
                })
            })
            .collect()
    }

    async fn update(&self, notification: &Notification) -> Result<Notification, String> {
        let type_str = Self::type_to_db(&notification.notification_type);
        let channel_str = Self::channel_to_db(&notification.channel);
        let priority_str = Self::priority_to_db(&notification.priority);
        let status_str = Self::status_to_db(&notification.status);

        let row = sqlx::query!(
            r#"
            UPDATE notifications
            SET organization_id = $2,
                user_id = $3,
                notification_type = $4,
                channel = $5,
                priority = $6,
                status = $7,
                title = $8,
                message = $9,
                link_url = $10,
                metadata = $11,
                sent_at = $12,
                read_at = $13,
                error_message = $14
            WHERE id = $1
            RETURNING id, organization_id, user_id, notification_type, channel, priority, status,
                      title, message, link_url, metadata, sent_at, read_at, created_at, error_message
            "#,
            notification.id,
            notification.organization_id,
            notification.user_id,
            type_str,
            channel_str,
            priority_str,
            status_str,
            notification.title,
            notification.message,
            notification.link_url,
            notification.metadata.as_ref().map(|m| serde_json::from_str::<serde_json::Value>(m).ok()).flatten(),
            notification.sent_at,
            notification.read_at,
            notification.error_message
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Database error updating notification: {}", e))?;

        Ok(Notification {
            id: row.id,
            organization_id: row.organization_id,
            user_id: row.user_id,
            notification_type: Self::type_from_db(&row.notification_type)?,
            channel: Self::channel_from_db(&row.channel)?,
            priority: Self::priority_from_db(&row.priority)?,
            status: Self::status_from_db(&row.status)?,
            title: row.title,
            message: row.message,
            link_url: row.link_url,
            metadata: row.metadata.map(|m| m.to_string()),
            sent_at: row.sent_at,
            read_at: row.read_at,
            created_at: row.created_at,
            error_message: row.error_message,
        })
    }

    async fn delete(&self, id: Uuid) -> Result<bool, String> {
        let result = sqlx::query!(
            r#"
            DELETE FROM notifications
            WHERE id = $1
            "#,
            id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Database error deleting notification: {}", e))?;

        Ok(result.rows_affected() > 0)
    }

    async fn count_unread_by_user(&self, user_id: Uuid) -> Result<i64, String> {
        let row = sqlx::query!(
            r#"
            SELECT COUNT(*) as count
            FROM notifications
            WHERE user_id = $1 AND channel = 'InApp' AND status = 'Sent' AND read_at IS NULL
            "#,
            user_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Database error counting unread notifications: {}", e))?;

        Ok(row.count.unwrap_or(0))
    }

    async fn count_by_user_and_status(
        &self,
        user_id: Uuid,
        status: NotificationStatus,
    ) -> Result<i64, String> {
        let status_str = Self::status_to_db(&status);

        let row = sqlx::query!(
            r#"
            SELECT COUNT(*) as count
            FROM notifications
            WHERE user_id = $1 AND status = $2
            "#,
            user_id,
            status_str
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Database error counting notifications by status: {}", e))?;

        Ok(row.count.unwrap_or(0))
    }

    async fn mark_all_read_by_user(&self, user_id: Uuid) -> Result<i64, String> {
        let now = chrono::Utc::now();

        let result = sqlx::query!(
            r#"
            UPDATE notifications
            SET status = 'Read',
                read_at = $2
            WHERE user_id = $1
              AND channel = 'InApp'
              AND status = 'Sent'
              AND read_at IS NULL
            "#,
            user_id,
            now
        )
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Database error marking all notifications as read: {}", e))?;

        Ok(result.rows_affected() as i64)
    }

    async fn delete_older_than(&self, days: i64) -> Result<i64, String> {
        let threshold = chrono::Utc::now() - chrono::Duration::days(days);

        let result = sqlx::query!(
            r#"
            DELETE FROM notifications
            WHERE created_at < $1
            "#,
            threshold
        )
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Database error deleting old notifications: {}", e))?;

        Ok(result.rows_affected() as i64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_conversion() {
        assert_eq!(
            PostgresNotificationRepository::type_to_db(&NotificationType::ExpenseCreated),
            "ExpenseCreated"
        );
        assert_eq!(
            PostgresNotificationRepository::type_from_db("MeetingConvocation").unwrap(),
            NotificationType::MeetingConvocation
        );
    }

    #[test]
    fn test_channel_conversion() {
        assert_eq!(
            PostgresNotificationRepository::channel_to_db(&NotificationChannel::Email),
            "Email"
        );
        assert_eq!(
            PostgresNotificationRepository::channel_from_db("InApp").unwrap(),
            NotificationChannel::InApp
        );
    }

    #[test]
    fn test_priority_conversion() {
        assert_eq!(
            PostgresNotificationRepository::priority_to_db(&NotificationPriority::Critical),
            "Critical"
        );
        assert_eq!(
            PostgresNotificationRepository::priority_from_db("Low").unwrap(),
            NotificationPriority::Low
        );
    }

    #[test]
    fn test_status_conversion() {
        assert_eq!(
            PostgresNotificationRepository::status_to_db(&NotificationStatus::Pending),
            "Pending"
        );
        assert_eq!(
            PostgresNotificationRepository::status_from_db("Sent").unwrap(),
            NotificationStatus::Sent
        );
    }
}
