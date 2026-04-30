use crate::application::dto::{
    CreateNotificationRequest, NotificationPreferenceResponse, NotificationResponse,
    NotificationStats, UpdatePreferenceRequest,
};
use crate::application::ports::{NotificationPreferenceRepository, NotificationRepository};
use crate::domain::entities::{
    Notification, NotificationChannel, NotificationPreference, NotificationStatus, NotificationType,
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
            .is_channel_enabled(
                request.user_id,
                request.notification_type.clone(),
                request.channel.clone(),
            )
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

        // InApp notifications are immediately "sent" (delivered to user's inbox)
        if notification.channel == NotificationChannel::InApp {
            notification.mark_sent();
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::ports::{NotificationPreferenceRepository, NotificationRepository};
    use crate::domain::entities::{
        Notification, NotificationChannel, NotificationPreference, NotificationPriority,
        NotificationStatus, NotificationType,
    };
    use async_trait::async_trait;
    use chrono::Utc;
    use std::collections::HashMap;
    use std::sync::Mutex;

    // ==================== Mock Repositories ====================

    struct MockNotificationRepository {
        notifications: Mutex<HashMap<Uuid, Notification>>,
    }

    impl MockNotificationRepository {
        fn new() -> Self {
            Self {
                notifications: Mutex::new(HashMap::new()),
            }
        }

        fn with_notification(self, notification: Notification) -> Self {
            self.notifications
                .lock()
                .unwrap()
                .insert(notification.id, notification);
            self
        }
    }

    #[async_trait]
    impl NotificationRepository for MockNotificationRepository {
        async fn create(&self, notification: &Notification) -> Result<Notification, String> {
            let n = notification.clone();
            self.notifications.lock().unwrap().insert(n.id, n.clone());
            Ok(n)
        }

        async fn find_by_id(&self, id: Uuid) -> Result<Option<Notification>, String> {
            Ok(self.notifications.lock().unwrap().get(&id).cloned())
        }

        async fn find_by_user(&self, user_id: Uuid) -> Result<Vec<Notification>, String> {
            Ok(self
                .notifications
                .lock()
                .unwrap()
                .values()
                .filter(|n| n.user_id == user_id)
                .cloned()
                .collect())
        }

        async fn find_by_user_and_status(
            &self,
            user_id: Uuid,
            status: NotificationStatus,
        ) -> Result<Vec<Notification>, String> {
            Ok(self
                .notifications
                .lock()
                .unwrap()
                .values()
                .filter(|n| n.user_id == user_id && n.status == status)
                .cloned()
                .collect())
        }

        async fn find_by_user_and_channel(
            &self,
            user_id: Uuid,
            channel: NotificationChannel,
        ) -> Result<Vec<Notification>, String> {
            Ok(self
                .notifications
                .lock()
                .unwrap()
                .values()
                .filter(|n| n.user_id == user_id && n.channel == channel)
                .cloned()
                .collect())
        }

        async fn find_unread_by_user(&self, user_id: Uuid) -> Result<Vec<Notification>, String> {
            Ok(self
                .notifications
                .lock()
                .unwrap()
                .values()
                .filter(|n| {
                    n.user_id == user_id
                        && n.channel == NotificationChannel::InApp
                        && n.status == NotificationStatus::Sent
                        && n.read_at.is_none()
                })
                .cloned()
                .collect())
        }

        async fn find_pending(&self) -> Result<Vec<Notification>, String> {
            Ok(self
                .notifications
                .lock()
                .unwrap()
                .values()
                .filter(|n| n.status == NotificationStatus::Pending)
                .cloned()
                .collect())
        }

        async fn find_failed(&self) -> Result<Vec<Notification>, String> {
            Ok(self
                .notifications
                .lock()
                .unwrap()
                .values()
                .filter(|n| n.status == NotificationStatus::Failed)
                .cloned()
                .collect())
        }

        async fn find_by_organization(
            &self,
            organization_id: Uuid,
        ) -> Result<Vec<Notification>, String> {
            Ok(self
                .notifications
                .lock()
                .unwrap()
                .values()
                .filter(|n| n.organization_id == organization_id)
                .cloned()
                .collect())
        }

        async fn update(&self, notification: &Notification) -> Result<Notification, String> {
            self.notifications
                .lock()
                .unwrap()
                .insert(notification.id, notification.clone());
            Ok(notification.clone())
        }

        async fn delete(&self, id: Uuid) -> Result<bool, String> {
            Ok(self.notifications.lock().unwrap().remove(&id).is_some())
        }

        async fn count_unread_by_user(&self, user_id: Uuid) -> Result<i64, String> {
            let count = self
                .notifications
                .lock()
                .unwrap()
                .values()
                .filter(|n| {
                    n.user_id == user_id
                        && n.channel == NotificationChannel::InApp
                        && n.status == NotificationStatus::Sent
                        && n.read_at.is_none()
                })
                .count();
            Ok(count as i64)
        }

        async fn count_by_user_and_status(
            &self,
            user_id: Uuid,
            status: NotificationStatus,
        ) -> Result<i64, String> {
            let count = self
                .notifications
                .lock()
                .unwrap()
                .values()
                .filter(|n| n.user_id == user_id && n.status == status)
                .count();
            Ok(count as i64)
        }

        async fn mark_all_read_by_user(&self, user_id: Uuid) -> Result<i64, String> {
            let mut store = self.notifications.lock().unwrap();
            let mut count = 0i64;
            for n in store.values_mut() {
                if n.user_id == user_id
                    && n.channel == NotificationChannel::InApp
                    && n.status == NotificationStatus::Sent
                {
                    n.status = NotificationStatus::Read;
                    n.read_at = Some(Utc::now());
                    count += 1;
                }
            }
            Ok(count)
        }

        async fn delete_older_than(&self, _days: i64) -> Result<i64, String> {
            Ok(0)
        }
    }

    struct MockNotificationPreferenceRepository {
        preferences: Mutex<HashMap<Uuid, NotificationPreference>>,
        /// When true, is_channel_enabled always returns true (default behaviour).
        channel_enabled: Mutex<bool>,
    }

    impl MockNotificationPreferenceRepository {
        fn new() -> Self {
            Self {
                preferences: Mutex::new(HashMap::new()),
                channel_enabled: Mutex::new(true),
            }
        }

        fn with_channel_disabled(self) -> Self {
            *self.channel_enabled.lock().unwrap() = false;
            self
        }

        fn with_preference(self, pref: NotificationPreference) -> Self {
            self.preferences.lock().unwrap().insert(pref.id, pref);
            self
        }
    }

    #[async_trait]
    impl NotificationPreferenceRepository for MockNotificationPreferenceRepository {
        async fn create(
            &self,
            preference: &NotificationPreference,
        ) -> Result<NotificationPreference, String> {
            self.preferences
                .lock()
                .unwrap()
                .insert(preference.id, preference.clone());
            Ok(preference.clone())
        }

        async fn find_by_id(&self, id: Uuid) -> Result<Option<NotificationPreference>, String> {
            Ok(self.preferences.lock().unwrap().get(&id).cloned())
        }

        async fn find_by_user_and_type(
            &self,
            user_id: Uuid,
            notification_type: NotificationType,
        ) -> Result<Option<NotificationPreference>, String> {
            Ok(self
                .preferences
                .lock()
                .unwrap()
                .values()
                .find(|p| p.user_id == user_id && p.notification_type == notification_type)
                .cloned())
        }

        async fn find_by_user(&self, user_id: Uuid) -> Result<Vec<NotificationPreference>, String> {
            Ok(self
                .preferences
                .lock()
                .unwrap()
                .values()
                .filter(|p| p.user_id == user_id)
                .cloned()
                .collect())
        }

        async fn update(
            &self,
            preference: &NotificationPreference,
        ) -> Result<NotificationPreference, String> {
            self.preferences
                .lock()
                .unwrap()
                .insert(preference.id, preference.clone());
            Ok(preference.clone())
        }

        async fn delete(&self, id: Uuid) -> Result<bool, String> {
            Ok(self.preferences.lock().unwrap().remove(&id).is_some())
        }

        async fn is_channel_enabled(
            &self,
            _user_id: Uuid,
            _notification_type: NotificationType,
            _channel: NotificationChannel,
        ) -> Result<bool, String> {
            Ok(*self.channel_enabled.lock().unwrap())
        }

        async fn create_defaults_for_user(
            &self,
            user_id: Uuid,
        ) -> Result<Vec<NotificationPreference>, String> {
            let pref = NotificationPreference::new(user_id, NotificationType::System);
            self.preferences
                .lock()
                .unwrap()
                .insert(pref.id, pref.clone());
            Ok(vec![pref])
        }
    }

    // ==================== Helper ====================

    fn make_use_cases(
        notif_repo: MockNotificationRepository,
        pref_repo: MockNotificationPreferenceRepository,
    ) -> NotificationUseCases {
        NotificationUseCases::new(Arc::new(notif_repo), Arc::new(pref_repo))
    }

    fn make_in_app_sent_notification(user_id: Uuid, org_id: Uuid) -> Notification {
        let mut n = Notification::new(
            org_id,
            user_id,
            NotificationType::TicketResolved,
            NotificationChannel::InApp,
            NotificationPriority::Medium,
            "Ticket resolved".to_string(),
            "Your ticket has been resolved.".to_string(),
        )
        .unwrap();
        n.mark_sent();
        n
    }

    // ==================== Tests ====================

    #[tokio::test]
    async fn test_create_notification_success() {
        let org_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        let uc = make_use_cases(
            MockNotificationRepository::new(),
            MockNotificationPreferenceRepository::new(), // channel_enabled = true
        );

        let request = CreateNotificationRequest {
            user_id,
            notification_type: NotificationType::ExpenseCreated,
            channel: NotificationChannel::InApp,
            priority: NotificationPriority::High,
            title: "Nouvel appel de fonds".to_string(),
            message: "Un appel de 500 EUR a ete cree.".to_string(),
            link_url: Some("https://app.koprogo.be/expenses/123".to_string()),
            metadata: None,
        };

        let result = uc.create_notification(org_id, request).await;
        assert!(result.is_ok(), "Expected Ok, got: {:?}", result.err());

        let resp = result.unwrap();
        assert_eq!(resp.organization_id, org_id);
        assert_eq!(resp.user_id, user_id);
        assert_eq!(resp.title, "Nouvel appel de fonds");
        // InApp notifications should be automatically marked as Sent
        assert_eq!(resp.status, NotificationStatus::Sent);
        assert!(resp.sent_at.is_some());
        assert!(resp.link_url.is_some());
    }

    #[tokio::test]
    async fn test_create_notification_channel_disabled() {
        let org_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        let uc = make_use_cases(
            MockNotificationRepository::new(),
            MockNotificationPreferenceRepository::new().with_channel_disabled(),
        );

        let request = CreateNotificationRequest {
            user_id,
            notification_type: NotificationType::PaymentReminder,
            channel: NotificationChannel::Email,
            priority: NotificationPriority::Medium,
            title: "Relance paiement".to_string(),
            message: "Votre paiement est en retard.".to_string(),
            link_url: None,
            metadata: None,
        };

        let result = uc.create_notification(org_id, request).await;
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "User has disabled this notification channel"
        );
    }

    #[tokio::test]
    async fn test_mark_as_read() {
        let org_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let notification = make_in_app_sent_notification(user_id, org_id);
        let notif_id = notification.id;

        let uc = make_use_cases(
            MockNotificationRepository::new().with_notification(notification),
            MockNotificationPreferenceRepository::new(),
        );

        let result = uc.mark_as_read(notif_id).await;
        assert!(result.is_ok(), "Expected Ok, got: {:?}", result.err());

        let resp = result.unwrap();
        assert_eq!(resp.status, NotificationStatus::Read);
        assert!(resp.read_at.is_some());
    }

    #[tokio::test]
    async fn test_mark_all_read() {
        let org_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        let n1 = make_in_app_sent_notification(user_id, org_id);
        let n2 = make_in_app_sent_notification(user_id, org_id);

        let repo = MockNotificationRepository::new()
            .with_notification(n1)
            .with_notification(n2);

        let uc = make_use_cases(repo, MockNotificationPreferenceRepository::new());

        let result = uc.mark_all_read(user_id).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 2);
    }

    #[tokio::test]
    async fn test_list_unread_notifications() {
        let org_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        // One unread (Sent InApp)
        let n_unread = make_in_app_sent_notification(user_id, org_id);

        // One read (Sent InApp then marked read)
        let mut n_read = make_in_app_sent_notification(user_id, org_id);
        n_read.mark_read().unwrap();

        // One email notification (should not appear in unread in-app)
        let n_email = Notification::new(
            org_id,
            user_id,
            NotificationType::PaymentReceived,
            NotificationChannel::Email,
            NotificationPriority::Low,
            "Paiement recu".to_string(),
            "Votre paiement a ete recu.".to_string(),
        )
        .unwrap();

        let repo = MockNotificationRepository::new()
            .with_notification(n_unread)
            .with_notification(n_read)
            .with_notification(n_email);

        let uc = make_use_cases(repo, MockNotificationPreferenceRepository::new());

        let result = uc.list_unread_notifications(user_id).await;
        assert!(result.is_ok());
        let unread = result.unwrap();
        assert_eq!(unread.len(), 1);
        assert_eq!(unread[0].status, NotificationStatus::Sent);
        assert_eq!(unread[0].channel, NotificationChannel::InApp);
    }

    #[tokio::test]
    async fn test_delete_notification() {
        let org_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        let notification = Notification::new(
            org_id,
            user_id,
            NotificationType::System,
            NotificationChannel::InApp,
            NotificationPriority::Low,
            "System update".to_string(),
            "The system will be under maintenance.".to_string(),
        )
        .unwrap();
        let notif_id = notification.id;

        let uc = make_use_cases(
            MockNotificationRepository::new().with_notification(notification),
            MockNotificationPreferenceRepository::new(),
        );

        let result = uc.delete_notification(notif_id).await;
        assert!(result.is_ok());
        assert!(result.unwrap()); // true = was deleted

        // Verify it no longer exists
        let get_result = uc.get_notification(notif_id).await;
        assert!(get_result.is_ok());
        assert!(get_result.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_get_user_stats() {
        let org_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        // 1 sent in-app (unread)
        let n_sent = make_in_app_sent_notification(user_id, org_id);

        // 1 pending email
        let n_pending = Notification::new(
            org_id,
            user_id,
            NotificationType::MeetingConvocation,
            NotificationChannel::Email,
            NotificationPriority::High,
            "Convocation AG".to_string(),
            "Vous etes convoque a l'AG.".to_string(),
        )
        .unwrap(); // status = Pending

        // 1 failed email
        let mut n_failed = Notification::new(
            org_id,
            user_id,
            NotificationType::PaymentReminder,
            NotificationChannel::Email,
            NotificationPriority::Medium,
            "Relance".to_string(),
            "Paiement en retard.".to_string(),
        )
        .unwrap();
        n_failed.mark_failed("SMTP error".to_string());

        let repo = MockNotificationRepository::new()
            .with_notification(n_sent)
            .with_notification(n_pending)
            .with_notification(n_failed);

        let uc = make_use_cases(repo, MockNotificationPreferenceRepository::new());

        let result = uc.get_user_stats(user_id).await;
        assert!(result.is_ok());

        let stats = result.unwrap();
        assert_eq!(stats.total, 3);
        assert_eq!(stats.unread, 1); // only the in-app sent notification
        assert_eq!(stats.pending, 1);
        assert_eq!(stats.sent, 1);
        assert_eq!(stats.failed, 1);
    }

    #[tokio::test]
    async fn test_update_preference() {
        let user_id = Uuid::new_v4();

        // Seed an existing preference
        let pref = NotificationPreference::new(user_id, NotificationType::ExpenseCreated);

        let pref_repo = MockNotificationPreferenceRepository::new().with_preference(pref);

        let uc = make_use_cases(MockNotificationRepository::new(), pref_repo);

        let request = UpdatePreferenceRequest {
            email_enabled: Some(false),
            in_app_enabled: None,     // unchanged
            push_enabled: Some(true), // unchanged (already true by default)
        };

        let result = uc
            .update_preference(user_id, NotificationType::ExpenseCreated, request)
            .await;
        assert!(result.is_ok(), "Expected Ok, got: {:?}", result.err());

        let resp = result.unwrap();
        assert_eq!(resp.user_id, user_id);
        assert!(!resp.email_enabled); // disabled
        assert!(resp.in_app_enabled); // unchanged default
        assert!(resp.push_enabled); // unchanged
    }
}
