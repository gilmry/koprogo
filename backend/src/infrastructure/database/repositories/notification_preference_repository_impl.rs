use crate::application::ports::NotificationPreferenceRepository;
use crate::domain::entities::{NotificationChannel, NotificationPreference, NotificationType};
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

/// PostgreSQL implementation of NotificationPreferenceRepository
pub struct PostgresNotificationPreferenceRepository {
    pool: PgPool,
}

impl PostgresNotificationPreferenceRepository {
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
}

#[async_trait]
impl NotificationPreferenceRepository for PostgresNotificationPreferenceRepository {
    async fn create(
        &self,
        preference: &NotificationPreference,
    ) -> Result<NotificationPreference, String> {
        let type_str = Self::type_to_db(&preference.notification_type);

        let row = sqlx::query!(
            r#"
            INSERT INTO notification_preferences (
                id, user_id, notification_type, email_enabled, in_app_enabled, push_enabled,
                created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id, user_id, notification_type, email_enabled, in_app_enabled, push_enabled,
                      created_at, updated_at
            "#,
            preference.id,
            preference.user_id,
            type_str,
            preference.email_enabled,
            preference.in_app_enabled,
            preference.push_enabled,
            preference.created_at,
            preference.updated_at
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Database error creating notification preference: {}", e))?;

        Ok(NotificationPreference {
            id: row.id,
            user_id: row.user_id,
            notification_type: Self::type_from_db(&row.notification_type)?,
            email_enabled: row.email_enabled,
            in_app_enabled: row.in_app_enabled,
            push_enabled: row.push_enabled,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<NotificationPreference>, String> {
        let row = sqlx::query!(
            r#"
            SELECT id, user_id, notification_type, email_enabled, in_app_enabled, push_enabled,
                   created_at, updated_at
            FROM notification_preferences
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Database error finding notification preference: {}", e))?;

        match row {
            Some(r) => Ok(Some(NotificationPreference {
                id: r.id,
                user_id: r.user_id,
                notification_type: Self::type_from_db(&r.notification_type)?,
                email_enabled: r.email_enabled,
                in_app_enabled: r.in_app_enabled,
                push_enabled: r.push_enabled,
                created_at: r.created_at,
                updated_at: r.updated_at,
            })),
            None => Ok(None),
        }
    }

    async fn find_by_user_and_type(
        &self,
        user_id: Uuid,
        notification_type: NotificationType,
    ) -> Result<Option<NotificationPreference>, String> {
        let type_str = Self::type_to_db(&notification_type);

        let row = sqlx::query!(
            r#"
            SELECT id, user_id, notification_type, email_enabled, in_app_enabled, push_enabled,
                   created_at, updated_at
            FROM notification_preferences
            WHERE user_id = $1 AND notification_type = $2
            "#,
            user_id,
            type_str
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            format!(
                "Database error finding notification preference by user and type: {}",
                e
            )
        })?;

        match row {
            Some(r) => Ok(Some(NotificationPreference {
                id: r.id,
                user_id: r.user_id,
                notification_type: Self::type_from_db(&r.notification_type)?,
                email_enabled: r.email_enabled,
                in_app_enabled: r.in_app_enabled,
                push_enabled: r.push_enabled,
                created_at: r.created_at,
                updated_at: r.updated_at,
            })),
            None => Ok(None),
        }
    }

    async fn find_by_user(&self, user_id: Uuid) -> Result<Vec<NotificationPreference>, String> {
        let rows = sqlx::query!(
            r#"
            SELECT id, user_id, notification_type, email_enabled, in_app_enabled, push_enabled,
                   created_at, updated_at
            FROM notification_preferences
            WHERE user_id = $1
            ORDER BY notification_type
            "#,
            user_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Database error finding notification preferences by user: {}", e))?;

        rows.into_iter()
            .map(|r| {
                Ok(NotificationPreference {
                    id: r.id,
                    user_id: r.user_id,
                    notification_type: Self::type_from_db(&r.notification_type)?,
                    email_enabled: r.email_enabled,
                    in_app_enabled: r.in_app_enabled,
                    push_enabled: r.push_enabled,
                    created_at: r.created_at,
                    updated_at: r.updated_at,
                })
            })
            .collect()
    }

    async fn update(
        &self,
        preference: &NotificationPreference,
    ) -> Result<NotificationPreference, String> {
        let type_str = Self::type_to_db(&preference.notification_type);

        let row = sqlx::query!(
            r#"
            UPDATE notification_preferences
            SET user_id = $2,
                notification_type = $3,
                email_enabled = $4,
                in_app_enabled = $5,
                push_enabled = $6,
                updated_at = $7
            WHERE id = $1
            RETURNING id, user_id, notification_type, email_enabled, in_app_enabled, push_enabled,
                      created_at, updated_at
            "#,
            preference.id,
            preference.user_id,
            type_str,
            preference.email_enabled,
            preference.in_app_enabled,
            preference.push_enabled,
            preference.updated_at
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Database error updating notification preference: {}", e))?;

        Ok(NotificationPreference {
            id: row.id,
            user_id: row.user_id,
            notification_type: Self::type_from_db(&row.notification_type)?,
            email_enabled: row.email_enabled,
            in_app_enabled: row.in_app_enabled,
            push_enabled: row.push_enabled,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }

    async fn delete(&self, id: Uuid) -> Result<bool, String> {
        let result = sqlx::query!(
            r#"
            DELETE FROM notification_preferences
            WHERE id = $1
            "#,
            id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Database error deleting notification preference: {}", e))?;

        Ok(result.rows_affected() > 0)
    }

    async fn is_channel_enabled(
        &self,
        user_id: Uuid,
        notification_type: NotificationType,
        channel: NotificationChannel,
    ) -> Result<bool, String> {
        let type_str = Self::type_to_db(&notification_type);

        let column = match channel {
            NotificationChannel::Email => "email_enabled",
            NotificationChannel::InApp => "in_app_enabled",
            NotificationChannel::Push => "push_enabled",
        };

        let query = format!(
            r#"
            SELECT {}
            FROM notification_preferences
            WHERE user_id = $1 AND notification_type = $2
            "#,
            column
        );

        let row = sqlx::query_scalar::<_, bool>(&query)
            .bind(user_id)
            .bind(type_str)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| format!("Database error checking channel enabled: {}", e))?;

        // If no preference exists, default to true (all channels enabled)
        Ok(row.unwrap_or(true))
    }

    async fn create_defaults_for_user(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<NotificationPreference>, String> {
        let notification_types = vec![
            NotificationType::ExpenseCreated,
            NotificationType::MeetingConvocation,
            NotificationType::PaymentReceived,
            NotificationType::TicketResolved,
            NotificationType::DocumentAdded,
            NotificationType::BoardMessage,
            NotificationType::PaymentReminder,
            NotificationType::BudgetApproved,
            NotificationType::ResolutionVote,
            NotificationType::System,
        ];

        let mut created_prefs = Vec::new();

        for notification_type in notification_types {
            let pref = NotificationPreference::new(user_id, notification_type);
            let created = self.create(&pref).await?;
            created_prefs.push(created);
        }

        Ok(created_prefs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_conversion() {
        assert_eq!(
            PostgresNotificationPreferenceRepository::type_to_db(
                &NotificationType::ExpenseCreated
            ),
            "ExpenseCreated"
        );
        assert_eq!(
            PostgresNotificationPreferenceRepository::type_from_db("MeetingConvocation").unwrap(),
            NotificationType::MeetingConvocation
        );
    }

    #[test]
    fn test_invalid_type() {
        assert!(PostgresNotificationPreferenceRepository::type_from_db("Invalid").is_err());
    }
}
