use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Notification Type - Different categories of notifications
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NotificationType {
    ExpenseCreated,      // Nouvel appel de fonds
    MeetingConvocation,  // Convocation AG
    PaymentReceived,     // Paiement reçu
    TicketResolved,      // Ticket résolu
    DocumentAdded,       // Document ajouté
    BoardMessage,        // Message conseil copropriété
    PaymentReminder,     // Relance paiement
    BudgetApproved,      // Budget approuvé
    ResolutionVote,      // Vote sur résolution
    System,              // Notification système
}

/// Notification Channel - Delivery method
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NotificationChannel {
    Email,   // Email notification
    InApp,   // In-app notification (dashboard)
    Push,    // Web Push notification (service worker)
}

/// Notification Status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NotificationStatus {
    Pending,   // Waiting to be sent
    Sent,      // Successfully sent
    Failed,    // Failed to send
    Read,      // Read by user (for in-app only)
}

/// Notification Priority
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub enum NotificationPriority {
    Low,       // Basse
    Medium,    // Moyenne
    High,      // Haute
    Critical,  // Critique/Urgente
}

/// Notification Entity
///
/// Represents a notification sent to a user via one or more channels.
/// Supports email, in-app, and web push notifications.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub user_id: Uuid,                  // Recipient
    pub notification_type: NotificationType,
    pub channel: NotificationChannel,
    pub priority: NotificationPriority,
    pub status: NotificationStatus,
    pub title: String,
    pub message: String,
    pub link_url: Option<String>,       // Optional link to resource
    pub metadata: Option<String>,       // JSON metadata for rich notifications
    pub sent_at: Option<DateTime<Utc>>,
    pub read_at: Option<DateTime<Utc>>, // For in-app notifications
    pub created_at: DateTime<Utc>,
    pub error_message: Option<String>,  // Error details if failed
}

impl Notification {
    /// Create a new notification
    pub fn new(
        organization_id: Uuid,
        user_id: Uuid,
        notification_type: NotificationType,
        channel: NotificationChannel,
        priority: NotificationPriority,
        title: String,
        message: String,
    ) -> Result<Self, String> {
        // Validation
        if title.trim().is_empty() {
            return Err("Title cannot be empty".to_string());
        }

        if title.len() > 200 {
            return Err("Title cannot exceed 200 characters".to_string());
        }

        if message.trim().is_empty() {
            return Err("Message cannot be empty".to_string());
        }

        if message.len() > 5000 {
            return Err("Message cannot exceed 5000 characters".to_string());
        }

        let now = Utc::now();

        Ok(Self {
            id: Uuid::new_v4(),
            organization_id,
            user_id,
            notification_type,
            channel,
            priority,
            status: NotificationStatus::Pending,
            title,
            message,
            link_url: None,
            metadata: None,
            sent_at: None,
            read_at: None,
            created_at: now,
            error_message: None,
        })
    }

    /// Set link URL for the notification
    pub fn with_link(mut self, url: String) -> Self {
        self.link_url = Some(url);
        self
    }

    /// Set metadata for the notification
    pub fn with_metadata(mut self, metadata: String) -> Self {
        self.metadata = Some(metadata);
        self
    }

    /// Mark notification as sent
    pub fn mark_sent(&mut self) {
        self.status = NotificationStatus::Sent;
        self.sent_at = Some(Utc::now());
        self.error_message = None;
    }

    /// Mark notification as failed
    pub fn mark_failed(&mut self, error: String) {
        self.status = NotificationStatus::Failed;
        self.error_message = Some(error);
    }

    /// Mark notification as read (in-app only)
    pub fn mark_read(&mut self) -> Result<(), String> {
        if self.channel != NotificationChannel::InApp {
            return Err("Only in-app notifications can be marked as read".to_string());
        }

        if self.status != NotificationStatus::Sent {
            return Err("Can only mark sent notifications as read".to_string());
        }

        self.status = NotificationStatus::Read;
        self.read_at = Some(Utc::now());
        Ok(())
    }

    /// Check if notification is unread
    pub fn is_unread(&self) -> bool {
        self.channel == NotificationChannel::InApp
            && self.status == NotificationStatus::Sent
            && self.read_at.is_none()
    }

    /// Check if notification is pending
    pub fn is_pending(&self) -> bool {
        self.status == NotificationStatus::Pending
    }

    /// Retry failed notification
    pub fn retry(&mut self) -> Result<(), String> {
        if self.status != NotificationStatus::Failed {
            return Err("Can only retry failed notifications".to_string());
        }

        self.status = NotificationStatus::Pending;
        self.error_message = None;
        Ok(())
    }
}

/// User Notification Preferences
///
/// Allows users to opt-in/opt-out of specific notification types per channel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationPreference {
    pub id: Uuid,
    pub user_id: Uuid,
    pub notification_type: NotificationType,
    pub email_enabled: bool,
    pub in_app_enabled: bool,
    pub push_enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl NotificationPreference {
    /// Create new notification preference with default settings
    pub fn new(user_id: Uuid, notification_type: NotificationType) -> Self {
        let now = Utc::now();

        Self {
            id: Uuid::new_v4(),
            user_id,
            notification_type,
            // Default: all channels enabled
            email_enabled: true,
            in_app_enabled: true,
            push_enabled: true,
            created_at: now,
            updated_at: now,
        }
    }

    /// Update preference for a specific channel
    pub fn set_channel_enabled(&mut self, channel: NotificationChannel, enabled: bool) {
        match channel {
            NotificationChannel::Email => self.email_enabled = enabled,
            NotificationChannel::InApp => self.in_app_enabled = enabled,
            NotificationChannel::Push => self.push_enabled = enabled,
        }
        self.updated_at = Utc::now();
    }

    /// Check if a channel is enabled
    pub fn is_channel_enabled(&self, channel: &NotificationChannel) -> bool {
        match channel {
            NotificationChannel::Email => self.email_enabled,
            NotificationChannel::InApp => self.in_app_enabled,
            NotificationChannel::Push => self.push_enabled,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_notification_success() {
        let notification = Notification::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            NotificationType::ExpenseCreated,
            NotificationChannel::Email,
            NotificationPriority::High,
            "Nouvel appel de fonds".to_string(),
            "Un nouvel appel de fonds de 500€ a été créé.".to_string(),
        );

        assert!(notification.is_ok());
        let notification = notification.unwrap();
        assert_eq!(notification.status, NotificationStatus::Pending);
        assert!(notification.sent_at.is_none());
    }

    #[test]
    fn test_create_notification_empty_title() {
        let result = Notification::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            NotificationType::System,
            NotificationChannel::InApp,
            NotificationPriority::Low,
            "   ".to_string(),
            "Message".to_string(),
        );

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Title cannot be empty");
    }

    #[test]
    fn test_mark_sent() {
        let mut notification = Notification::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            NotificationType::PaymentReceived,
            NotificationChannel::Email,
            NotificationPriority::Medium,
            "Paiement reçu".to_string(),
            "Votre paiement a été reçu.".to_string(),
        )
        .unwrap();

        notification.mark_sent();

        assert_eq!(notification.status, NotificationStatus::Sent);
        assert!(notification.sent_at.is_some());
        assert!(notification.error_message.is_none());
    }

    #[test]
    fn test_mark_failed() {
        let mut notification = Notification::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            NotificationType::TicketResolved,
            NotificationChannel::Email,
            NotificationPriority::Low,
            "Ticket résolu".to_string(),
            "Votre ticket a été résolu.".to_string(),
        )
        .unwrap();

        notification.mark_failed("SMTP error".to_string());

        assert_eq!(notification.status, NotificationStatus::Failed);
        assert_eq!(notification.error_message, Some("SMTP error".to_string()));
    }

    #[test]
    fn test_mark_read_in_app() {
        let mut notification = Notification::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            NotificationType::DocumentAdded,
            NotificationChannel::InApp,
            NotificationPriority::Low,
            "Nouveau document".to_string(),
            "Un nouveau document a été ajouté.".to_string(),
        )
        .unwrap();

        notification.mark_sent();
        let result = notification.mark_read();

        assert!(result.is_ok());
        assert_eq!(notification.status, NotificationStatus::Read);
        assert!(notification.read_at.is_some());
    }

    #[test]
    fn test_cannot_mark_read_email() {
        let mut notification = Notification::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            NotificationType::BoardMessage,
            NotificationChannel::Email,
            NotificationPriority::Medium,
            "Message du conseil".to_string(),
            "Le conseil a envoyé un message.".to_string(),
        )
        .unwrap();

        notification.mark_sent();
        let result = notification.mark_read();

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Only in-app"));
    }

    #[test]
    fn test_is_unread() {
        let mut notification = Notification::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            NotificationType::MeetingConvocation,
            NotificationChannel::InApp,
            NotificationPriority::High,
            "Convocation AG".to_string(),
            "Vous êtes convoqué à l'AG du 15/12.".to_string(),
        )
        .unwrap();

        assert!(!notification.is_unread()); // Pending

        notification.mark_sent();
        assert!(notification.is_unread()); // Sent but not read

        notification.mark_read().unwrap();
        assert!(!notification.is_unread()); // Read
    }

    #[test]
    fn test_retry_failed_notification() {
        let mut notification = Notification::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            NotificationType::PaymentReminder,
            NotificationChannel::Email,
            NotificationPriority::High,
            "Relance paiement".to_string(),
            "Votre paiement est en retard.".to_string(),
        )
        .unwrap();

        notification.mark_failed("Network error".to_string());

        let result = notification.retry();
        assert!(result.is_ok());
        assert_eq!(notification.status, NotificationStatus::Pending);
        assert!(notification.error_message.is_none());
    }

    #[test]
    fn test_notification_preference() {
        let pref = NotificationPreference::new(
            Uuid::new_v4(),
            NotificationType::ExpenseCreated,
        );

        assert!(pref.email_enabled);
        assert!(pref.in_app_enabled);
        assert!(pref.push_enabled);
    }

    #[test]
    fn test_set_channel_enabled() {
        let mut pref = NotificationPreference::new(
            Uuid::new_v4(),
            NotificationType::MeetingConvocation,
        );

        pref.set_channel_enabled(NotificationChannel::Email, false);

        assert!(!pref.email_enabled);
        assert!(pref.in_app_enabled);
        assert!(pref.push_enabled);
    }

    #[test]
    fn test_is_channel_enabled() {
        let pref = NotificationPreference::new(
            Uuid::new_v4(),
            NotificationType::TicketResolved,
        );

        assert!(pref.is_channel_enabled(&NotificationChannel::Email));
        assert!(pref.is_channel_enabled(&NotificationChannel::InApp));
        assert!(pref.is_channel_enabled(&NotificationChannel::Push));
    }
}
