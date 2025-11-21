use crate::application::dto::{
    CreateNotificationRequest, NotificationPreferenceResponse, NotificationResponse,
    NotificationStats, UpdatePreferenceRequest,
};
use crate::application::ports::{NotificationPreferenceRepository, NotificationRepository};
use crate::domain::entities::{
    Notification, NotificationChannel, NotificationPreference, NotificationStatus,
    NotificationType,
};
use std::sync::Arc;
use uuid::Uuid;

pub struct NotificationUseCases {
    notification_repository: Arc<dyn NotificationRepository>,
    preference_repository: Arc<dyn NotificationPreferenceRepository>,
}

impl NotificationUseCases {
    pub fn new(
        notification_repository: Arc<dyn NotificationRepository>,
        preference_repository: Arc<dyn NotificationPreferenceRepository>,
    ) -> Self {
        Self {
            notification_repository,
            preference_repository,
        }
    }

    /// Create a new notification
    pub async fn create_notification(
        &self,
        organization_id: Uuid,
        request: CreateNotificationRequest,
    ) -> Result<NotificationResponse, String> {
        // Check if user has enabled this channel for this notification type
        let is_enabled = self
            .preference_repository
            .is_channel_enabled(request.user_id, request.notification_type.clone(), request.channel.clone())
            .await?;

        if !is_enabled {
            return Err("User has disabled this notification channel".to_string());
        }

        let mut notification = Notification::new(
            organization_id,
            request.user_id,
            request.notification_type,
            request.channel,
            request.priority,
            request.title,
            request.message,
        )?;

        if let Some(link_url) = request.link_url {
            notification = notification.with_link(link_url);
        }

        if let Some(metadata) = request.metadata {
            notification = notification.with_metadata(metadata);
        }

        let created = self.notification_repository.create(&notification).await?;
        Ok(NotificationResponse::from(created))
    }

    /// Get a notification by ID
    pub async fn get_notification(&self, id: Uuid) -> Result<Option<NotificationResponse>, String> {
        match self.notification_repository.find_by_id(id).await? {
            Some(notification) => Ok(Some(NotificationResponse::from(notification))),
            None => Ok(None),
        }
    }

    /// List all notifications for a user
    pub async fn list_user_notifications(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<NotificationResponse>, String> {
        let notifications = self.notification_repository.find_by_user(user_id).await?;
        Ok(notifications
            .into_iter()
            .map(NotificationResponse::from)
            .collect())
    }

    /// List unread in-app notifications for a user
    pub async fn list_unread_notifications(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<NotificationResponse>, String> {
        let notifications = self
            .notification_repository
            .find_unread_by_user(user_id)
            .await?;
        Ok(notifications
            .into_iter()
            .map(NotificationResponse::from)
            .collect())
    }

    /// Mark an in-app notification as read
    pub async fn mark_as_read(&self, id: Uuid) -> Result<NotificationResponse, String> {
        let mut notification = self
            .notification_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| "Notification not found".to_string())?;

        notification.mark_read()?;

        let updated = self.notification_repository.update(&notification).await?;
        Ok(NotificationResponse::from(updated))
    }

    /// Mark all in-app notifications as read for a user
    pub async fn mark_all_read(&self, user_id: Uuid) -> Result<i64, String> {
        self.notification_repository
            .mark_all_read_by_user(user_id)
            .await
    }

    /// Delete a notification
    pub async fn delete_notification(&self, id: Uuid) -> Result<bool, String> {
        self.notification_repository.delete(id).await
    }

    /// Get notification statistics for a user
    pub async fn get_user_stats(&self, user_id: Uuid) -> Result<NotificationStats, String> {
        let total = self
            .notification_repository
            .find_by_user(user_id)
            .await?
            .len() as i64;

        let unread = self
            .notification_repository
            .count_unread_by_user(user_id)
            .await?;

        let pending = self
            .notification_repository
            .count_by_user_and_status(user_id, NotificationStatus::Pending)
            .await?;

        let sent = self
            .notification_repository
            .count_by_user_and_status(user_id, NotificationStatus::Sent)
            .await?;

        let failed = self
            .notification_repository
            .count_by_user_and_status(user_id, NotificationStatus::Failed)
            .await?;

        Ok(NotificationStats {
            total,
            unread,
            pending,
            sent,
            failed,
        })
    }

    // ==================== Notification Preferences ====================

    /// Get user's notification preferences
    pub async fn get_user_preferences(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<NotificationPreferenceResponse>, String> {
        let preferences = self.preference_repository.find_by_user(user_id).await?;

        // If user has no preferences yet, create defaults
        if preferences.is_empty() {
            let defaults = self
                .preference_repository
                .create_defaults_for_user(user_id)
                .await?;
            return Ok(defaults
                .into_iter()
                .map(NotificationPreferenceResponse::from)
                .collect());
        }

        Ok(preferences
            .into_iter()
            .map(NotificationPreferenceResponse::from)
            .collect())
    }

    /// Get user's preference for a specific notification type
    pub async fn get_preference(
        &self,
        user_id: Uuid,
        notification_type: NotificationType,
    ) -> Result<Option<NotificationPreferenceResponse>, String> {
        match self
            .preference_repository
            .find_by_user_and_type(user_id, notification_type.clone())
            .await?
        {
            Some(pref) => Ok(Some(NotificationPreferenceResponse::from(pref))),
            None => {
                // Create default preference for this type
                let default_pref = NotificationPreference::new(user_id, notification_type);
                let created = self.preference_repository.create(&default_pref).await?;
                Ok(Some(NotificationPreferenceResponse::from(created)))
            }
        }
    }

    /// Update user's notification preference for a specific notification type
    pub async fn update_preference(
        &self,
        user_id: Uuid,
        notification_type: NotificationType,
        request: UpdatePreferenceRequest,
    ) -> Result<NotificationPreferenceResponse, String> {
        let mut preference = match self
            .preference_repository
            .find_by_user_and_type(user_id, notification_type.clone())
            .await?
        {
            Some(pref) => pref,
            None => {
                // Create default if doesn't exist
                let default_pref = NotificationPreference::new(user_id, notification_type);
                self.preference_repository.create(&default_pref).await?
            }
        };

        // Update channels as requested
        if let Some(email_enabled) = request.email_enabled {
            preference.set_channel_enabled(NotificationChannel::Email, email_enabled);
        }

        if let Some(in_app_enabled) = request.in_app_enabled {
            preference.set_channel_enabled(NotificationChannel::InApp, in_app_enabled);
        }

        if let Some(push_enabled) = request.push_enabled {
            preference.set_channel_enabled(NotificationChannel::Push, push_enabled);
        }

        let updated = self.preference_repository.update(&preference).await?;
        Ok(NotificationPreferenceResponse::from(updated))
    }

    // ==================== Admin/System Methods ====================

    /// Get all pending notifications (for background processing)
    pub async fn get_pending_notifications(&self) -> Result<Vec<NotificationResponse>, String> {
        let notifications = self.notification_repository.find_pending().await?;
        Ok(notifications
            .into_iter()
            .map(NotificationResponse::from)
            .collect())
    }

    /// Get all failed notifications (for retry)
    pub async fn get_failed_notifications(&self) -> Result<Vec<NotificationResponse>, String> {
        let notifications = self.notification_repository.find_failed().await?;
        Ok(notifications
            .into_iter()
            .map(NotificationResponse::from)
            .collect())
    }

    /// Mark notification as sent
    pub async fn mark_as_sent(&self, id: Uuid) -> Result<NotificationResponse, String> {
        let mut notification = self
            .notification_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| "Notification not found".to_string())?;

        notification.mark_sent();

        let updated = self.notification_repository.update(&notification).await?;
        Ok(NotificationResponse::from(updated))
    }

    /// Mark notification as failed
    pub async fn mark_as_failed(
        &self,
        id: Uuid,
        error_message: String,
    ) -> Result<NotificationResponse, String> {
        let mut notification = self
            .notification_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| "Notification not found".to_string())?;

        notification.mark_failed(error_message);

        let updated = self.notification_repository.update(&notification).await?;
        Ok(NotificationResponse::from(updated))
    }

    /// Retry a failed notification
    pub async fn retry_notification(&self, id: Uuid) -> Result<NotificationResponse, String> {
        let mut notification = self
            .notification_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| "Notification not found".to_string())?;

        notification.retry()?;

        let updated = self.notification_repository.update(&notification).await?;
        Ok(NotificationResponse::from(updated))
    }

    /// Delete old notifications (cleanup)
    pub async fn cleanup_old_notifications(&self, days: i64) -> Result<i64, String> {
        self.notification_repository.delete_older_than(days).await
    }
}
