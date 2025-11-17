use crate::application::ports::{ConvocationRecipientRepository, RecipientTrackingSummary};
use crate::domain::entities::{AttendanceStatus, ConvocationRecipient};
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

/// PostgreSQL implementation of ConvocationRecipientRepository
pub struct PostgresConvocationRecipientRepository {
    pool: PgPool,
}

impl PostgresConvocationRecipientRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Convert AttendanceStatus enum to database string
    fn attendance_status_to_db(status: &AttendanceStatus) -> &'static str {
        status.to_db_string()
    }

    /// Convert database string to AttendanceStatus enum
    fn attendance_status_from_db(s: &str) -> Result<AttendanceStatus, String> {
        AttendanceStatus::from_db_string(s)
    }
}

#[async_trait]
impl ConvocationRecipientRepository for PostgresConvocationRecipientRepository {
    async fn create(
        &self,
        recipient: &ConvocationRecipient,
    ) -> Result<ConvocationRecipient, String> {
        let attendance_status_str = Self::attendance_status_to_db(&recipient.attendance_status);

        let row = sqlx::query!(
            r#"
            INSERT INTO convocation_recipients (
                id, convocation_id, owner_id, email, email_sent_at, email_opened_at,
                email_failed, email_failure_reason, reminder_sent_at, reminder_opened_at,
                attendance_status, attendance_updated_at, proxy_owner_id,
                created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
            RETURNING id, convocation_id, owner_id, email, email_sent_at, email_opened_at,
                      email_failed, email_failure_reason, reminder_sent_at, reminder_opened_at,
                      attendance_status::text AS "attendance_status!", attendance_updated_at, proxy_owner_id,
                      created_at, updated_at
            "#,
            recipient.id,
            recipient.convocation_id,
            recipient.owner_id,
            recipient.email,
            recipient.email_sent_at,
            recipient.email_opened_at,
            recipient.email_failed,
            recipient.email_failure_reason,
            recipient.reminder_sent_at,
            recipient.reminder_opened_at,
            attendance_status_str,
            recipient.attendance_updated_at,
            recipient.proxy_owner_id,
            recipient.created_at,
            recipient.updated_at,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to create convocation recipient: {}", e))?;

        Ok(ConvocationRecipient {
            id: row.id,
            convocation_id: row.convocation_id,
            owner_id: row.owner_id,
            email: row.email,
            email_sent_at: row.email_sent_at,
            email_opened_at: row.email_opened_at,
            email_failed: row.email_failed,
            email_failure_reason: row.email_failure_reason,
            reminder_sent_at: row.reminder_sent_at,
            reminder_opened_at: row.reminder_opened_at,
            attendance_status: Self::attendance_status_from_db(&row.attendance_status)?,
            attendance_updated_at: row.attendance_updated_at,
            proxy_owner_id: row.proxy_owner_id,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }

    async fn create_many(
        &self,
        recipients: &[ConvocationRecipient],
    ) -> Result<Vec<ConvocationRecipient>, String> {
        if recipients.is_empty() {
            return Ok(Vec::new());
        }

        // Use a transaction for bulk insert
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| format!("Failed to begin transaction: {}", e))?;

        let mut created = Vec::new();

        for recipient in recipients {
            let attendance_status_str = Self::attendance_status_to_db(&recipient.attendance_status);

            let row = sqlx::query!(
                r#"
                INSERT INTO convocation_recipients (
                    id, convocation_id, owner_id, email, email_sent_at, email_opened_at,
                    email_failed, email_failure_reason, reminder_sent_at, reminder_opened_at,
                    attendance_status, attendance_updated_at, proxy_owner_id,
                    created_at, updated_at
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
                RETURNING id, convocation_id, owner_id, email, email_sent_at, email_opened_at,
                          email_failed, email_failure_reason, reminder_sent_at, reminder_opened_at,
                          attendance_status AS "attendance_status!", attendance_updated_at, proxy_owner_id,
                          created_at, updated_at
                "#,
                recipient.id,
                recipient.convocation_id,
                recipient.owner_id,
                recipient.email,
                recipient.email_sent_at,
                recipient.email_opened_at,
                recipient.email_failed,
                recipient.email_failure_reason,
                recipient.reminder_sent_at,
                recipient.reminder_opened_at,
                attendance_status_str,
                recipient.attendance_updated_at,
                recipient.proxy_owner_id,
                recipient.created_at,
                recipient.updated_at,
            )
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| format!("Failed to create recipient in batch: {}", e))?;

            created.push(ConvocationRecipient {
                id: row.id,
                convocation_id: row.convocation_id,
                owner_id: row.owner_id,
                email: row.email,
                email_sent_at: row.email_sent_at,
                email_opened_at: row.email_opened_at,
                email_failed: row.email_failed,
                email_failure_reason: row.email_failure_reason,
                reminder_sent_at: row.reminder_sent_at,
                reminder_opened_at: row.reminder_opened_at,
                attendance_status: Self::attendance_status_from_db(&row.attendance_status)?,
                attendance_updated_at: row.attendance_updated_at,
                proxy_owner_id: row.proxy_owner_id,
                created_at: row.created_at,
                updated_at: row.updated_at,
            });
        }

        tx.commit()
            .await
            .map_err(|e| format!("Failed to commit transaction: {}", e))?;

        Ok(created)
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<ConvocationRecipient>, String> {
        let row = sqlx::query!(
            r#"
            SELECT id, convocation_id, owner_id, email, email_sent_at, email_opened_at,
                   email_failed, email_failure_reason, reminder_sent_at, reminder_opened_at,
                   attendance_status::text AS "attendance_status!", attendance_updated_at, proxy_owner_id,
                   created_at, updated_at
            FROM convocation_recipients
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Failed to find convocation recipient by id: {}", e))?;

        match row {
            Some(row) => Ok(Some(ConvocationRecipient {
                id: row.id,
                convocation_id: row.convocation_id,
                owner_id: row.owner_id,
                email: row.email,
                email_sent_at: row.email_sent_at,
                email_opened_at: row.email_opened_at,
                email_failed: row.email_failed,
                email_failure_reason: row.email_failure_reason,
                reminder_sent_at: row.reminder_sent_at,
                reminder_opened_at: row.reminder_opened_at,
                attendance_status: Self::attendance_status_from_db(&row.attendance_status)?,
                attendance_updated_at: row.attendance_updated_at,
                proxy_owner_id: row.proxy_owner_id,
                created_at: row.created_at,
                updated_at: row.updated_at,
            })),
            None => Ok(None),
        }
    }

    async fn find_by_convocation(
        &self,
        convocation_id: Uuid,
    ) -> Result<Vec<ConvocationRecipient>, String> {
        let rows = sqlx::query!(
            r#"
            SELECT id, convocation_id, owner_id, email, email_sent_at, email_opened_at,
                   email_failed, email_failure_reason, reminder_sent_at, reminder_opened_at,
                   attendance_status::text AS "attendance_status!", attendance_updated_at, proxy_owner_id,
                   created_at, updated_at
            FROM convocation_recipients
            WHERE convocation_id = $1
            ORDER BY created_at ASC
            "#,
            convocation_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to find recipients by convocation: {}", e))?;

        rows.into_iter()
            .map(|row| {
                Ok(ConvocationRecipient {
                    id: row.id,
                    convocation_id: row.convocation_id,
                    owner_id: row.owner_id,
                    email: row.email,
                    email_sent_at: row.email_sent_at,
                    email_opened_at: row.email_opened_at,
                    email_failed: row.email_failed,
                    email_failure_reason: row.email_failure_reason,
                    reminder_sent_at: row.reminder_sent_at,
                    reminder_opened_at: row.reminder_opened_at,
                    attendance_status: Self::attendance_status_from_db(&row.attendance_status)?,
                    attendance_updated_at: row.attendance_updated_at,
                    proxy_owner_id: row.proxy_owner_id,
                    created_at: row.created_at,
                    updated_at: row.updated_at,
                })
            })
            .collect()
    }

    async fn find_by_convocation_and_owner(
        &self,
        convocation_id: Uuid,
        owner_id: Uuid,
    ) -> Result<Option<ConvocationRecipient>, String> {
        let row = sqlx::query!(
            r#"
            SELECT id, convocation_id, owner_id, email, email_sent_at, email_opened_at,
                   email_failed, email_failure_reason, reminder_sent_at, reminder_opened_at,
                   attendance_status::text AS "attendance_status!", attendance_updated_at, proxy_owner_id,
                   created_at, updated_at
            FROM convocation_recipients
            WHERE convocation_id = $1 AND owner_id = $2
            "#,
            convocation_id,
            owner_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            format!(
                "Failed to find recipient by convocation and owner: {}",
                e
            )
        })?;

        match row {
            Some(row) => Ok(Some(ConvocationRecipient {
                id: row.id,
                convocation_id: row.convocation_id,
                owner_id: row.owner_id,
                email: row.email,
                email_sent_at: row.email_sent_at,
                email_opened_at: row.email_opened_at,
                email_failed: row.email_failed,
                email_failure_reason: row.email_failure_reason,
                reminder_sent_at: row.reminder_sent_at,
                reminder_opened_at: row.reminder_opened_at,
                attendance_status: Self::attendance_status_from_db(&row.attendance_status)?,
                attendance_updated_at: row.attendance_updated_at,
                proxy_owner_id: row.proxy_owner_id,
                created_at: row.created_at,
                updated_at: row.updated_at,
            })),
            None => Ok(None),
        }
    }

    async fn find_by_owner(&self, owner_id: Uuid) -> Result<Vec<ConvocationRecipient>, String> {
        let rows = sqlx::query!(
            r#"
            SELECT id, convocation_id, owner_id, email, email_sent_at, email_opened_at,
                   email_failed, email_failure_reason, reminder_sent_at, reminder_opened_at,
                   attendance_status::text AS "attendance_status!", attendance_updated_at, proxy_owner_id,
                   created_at, updated_at
            FROM convocation_recipients
            WHERE owner_id = $1
            ORDER BY created_at DESC
            "#,
            owner_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to find recipients by owner: {}", e))?;

        rows.into_iter()
            .map(|row| {
                Ok(ConvocationRecipient {
                    id: row.id,
                    convocation_id: row.convocation_id,
                    owner_id: row.owner_id,
                    email: row.email,
                    email_sent_at: row.email_sent_at,
                    email_opened_at: row.email_opened_at,
                    email_failed: row.email_failed,
                    email_failure_reason: row.email_failure_reason,
                    reminder_sent_at: row.reminder_sent_at,
                    reminder_opened_at: row.reminder_opened_at,
                    attendance_status: Self::attendance_status_from_db(&row.attendance_status)?,
                    attendance_updated_at: row.attendance_updated_at,
                    proxy_owner_id: row.proxy_owner_id,
                    created_at: row.created_at,
                    updated_at: row.updated_at,
                })
            })
            .collect()
    }

    async fn find_by_attendance_status(
        &self,
        convocation_id: Uuid,
        status: AttendanceStatus,
    ) -> Result<Vec<ConvocationRecipient>, String> {
        let status_str = Self::attendance_status_to_db(&status);

        let rows = sqlx::query!(
            r#"
            SELECT id, convocation_id, owner_id, email, email_sent_at, email_opened_at,
                   email_failed, email_failure_reason, reminder_sent_at, reminder_opened_at,
                   attendance_status::text AS "attendance_status!", attendance_updated_at, proxy_owner_id,
                   created_at, updated_at
            FROM convocation_recipients
            WHERE convocation_id = $1 AND attendance_status = $2
            ORDER BY created_at ASC
            "#,
            convocation_id,
            status_str
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to find recipients by attendance status: {}", e))?;

        rows.into_iter()
            .map(|row| {
                Ok(ConvocationRecipient {
                    id: row.id,
                    convocation_id: row.convocation_id,
                    owner_id: row.owner_id,
                    email: row.email,
                    email_sent_at: row.email_sent_at,
                    email_opened_at: row.email_opened_at,
                    email_failed: row.email_failed,
                    email_failure_reason: row.email_failure_reason,
                    reminder_sent_at: row.reminder_sent_at,
                    reminder_opened_at: row.reminder_opened_at,
                    attendance_status: Self::attendance_status_from_db(&row.attendance_status)?,
                    attendance_updated_at: row.attendance_updated_at,
                    proxy_owner_id: row.proxy_owner_id,
                    created_at: row.created_at,
                    updated_at: row.updated_at,
                })
            })
            .collect()
    }

    async fn find_needing_reminder(
        &self,
        convocation_id: Uuid,
    ) -> Result<Vec<ConvocationRecipient>, String> {
        let rows = sqlx::query!(
            r#"
            SELECT id, convocation_id, owner_id, email, email_sent_at, email_opened_at,
                   email_failed, email_failure_reason, reminder_sent_at, reminder_opened_at,
                   attendance_status::text AS "attendance_status!", attendance_updated_at, proxy_owner_id,
                   created_at, updated_at
            FROM convocation_recipients
            WHERE convocation_id = $1
              AND email_sent_at IS NOT NULL
              AND email_opened_at IS NULL
              AND reminder_sent_at IS NULL
              AND email_failed = FALSE
            ORDER BY created_at ASC
            "#,
            convocation_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to find recipients needing reminder: {}", e))?;

        rows.into_iter()
            .map(|row| {
                Ok(ConvocationRecipient {
                    id: row.id,
                    convocation_id: row.convocation_id,
                    owner_id: row.owner_id,
                    email: row.email,
                    email_sent_at: row.email_sent_at,
                    email_opened_at: row.email_opened_at,
                    email_failed: row.email_failed,
                    email_failure_reason: row.email_failure_reason,
                    reminder_sent_at: row.reminder_sent_at,
                    reminder_opened_at: row.reminder_opened_at,
                    attendance_status: Self::attendance_status_from_db(&row.attendance_status)?,
                    attendance_updated_at: row.attendance_updated_at,
                    proxy_owner_id: row.proxy_owner_id,
                    created_at: row.created_at,
                    updated_at: row.updated_at,
                })
            })
            .collect()
    }

    async fn find_failed_emails(
        &self,
        convocation_id: Uuid,
    ) -> Result<Vec<ConvocationRecipient>, String> {
        let rows = sqlx::query!(
            r#"
            SELECT id, convocation_id, owner_id, email, email_sent_at, email_opened_at,
                   email_failed, email_failure_reason, reminder_sent_at, reminder_opened_at,
                   attendance_status::text AS "attendance_status!", attendance_updated_at, proxy_owner_id,
                   created_at, updated_at
            FROM convocation_recipients
            WHERE convocation_id = $1 AND email_failed = TRUE
            ORDER BY created_at ASC
            "#,
            convocation_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to find failed emails: {}", e))?;

        rows.into_iter()
            .map(|row| {
                Ok(ConvocationRecipient {
                    id: row.id,
                    convocation_id: row.convocation_id,
                    owner_id: row.owner_id,
                    email: row.email,
                    email_sent_at: row.email_sent_at,
                    email_opened_at: row.email_opened_at,
                    email_failed: row.email_failed,
                    email_failure_reason: row.email_failure_reason,
                    reminder_sent_at: row.reminder_sent_at,
                    reminder_opened_at: row.reminder_opened_at,
                    attendance_status: Self::attendance_status_from_db(&row.attendance_status)?,
                    attendance_updated_at: row.attendance_updated_at,
                    proxy_owner_id: row.proxy_owner_id,
                    created_at: row.created_at,
                    updated_at: row.updated_at,
                })
            })
            .collect()
    }

    async fn update(
        &self,
        recipient: &ConvocationRecipient,
    ) -> Result<ConvocationRecipient, String> {
        let attendance_status_str = Self::attendance_status_to_db(&recipient.attendance_status);

        let row = sqlx::query!(
            r#"
            UPDATE convocation_recipients
            SET convocation_id = $2, owner_id = $3, email = $4, email_sent_at = $5, email_opened_at = $6,
                email_failed = $7, email_failure_reason = $8, reminder_sent_at = $9, reminder_opened_at = $10,
                attendance_status = $11, attendance_updated_at = $12, proxy_owner_id = $13,
                updated_at = $14
            WHERE id = $1
            RETURNING id, convocation_id, owner_id, email, email_sent_at, email_opened_at,
                      email_failed, email_failure_reason, reminder_sent_at, reminder_opened_at,
                      attendance_status::text AS "attendance_status!", attendance_updated_at, proxy_owner_id,
                      created_at, updated_at
            "#,
            recipient.id,
            recipient.convocation_id,
            recipient.owner_id,
            recipient.email,
            recipient.email_sent_at,
            recipient.email_opened_at,
            recipient.email_failed,
            recipient.email_failure_reason,
            recipient.reminder_sent_at,
            recipient.reminder_opened_at,
            attendance_status_str,
            recipient.attendance_updated_at,
            recipient.proxy_owner_id,
            recipient.updated_at,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to update convocation recipient: {}", e))?;

        Ok(ConvocationRecipient {
            id: row.id,
            convocation_id: row.convocation_id,
            owner_id: row.owner_id,
            email: row.email,
            email_sent_at: row.email_sent_at,
            email_opened_at: row.email_opened_at,
            email_failed: row.email_failed,
            email_failure_reason: row.email_failure_reason,
            reminder_sent_at: row.reminder_sent_at,
            reminder_opened_at: row.reminder_opened_at,
            attendance_status: Self::attendance_status_from_db(&row.attendance_status)?,
            attendance_updated_at: row.attendance_updated_at,
            proxy_owner_id: row.proxy_owner_id,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }

    async fn delete(&self, id: Uuid) -> Result<bool, String> {
        let result = sqlx::query!(
            r#"
            DELETE FROM convocation_recipients
            WHERE id = $1
            "#,
            id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to delete convocation recipient: {}", e))?;

        Ok(result.rows_affected() > 0)
    }

    async fn count_by_convocation(&self, convocation_id: Uuid) -> Result<i64, String> {
        let row = sqlx::query!(
            r#"
            SELECT COUNT(*) AS "count!"
            FROM convocation_recipients
            WHERE convocation_id = $1
            "#,
            convocation_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to count recipients by convocation: {}", e))?;

        Ok(row.count)
    }

    async fn count_opened(&self, convocation_id: Uuid) -> Result<i64, String> {
        let row = sqlx::query!(
            r#"
            SELECT COUNT(*) AS "count!"
            FROM convocation_recipients
            WHERE convocation_id = $1 AND email_opened_at IS NOT NULL
            "#,
            convocation_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to count opened emails: {}", e))?;

        Ok(row.count)
    }

    async fn count_by_attendance_status(
        &self,
        convocation_id: Uuid,
        status: AttendanceStatus,
    ) -> Result<i64, String> {
        let status_str = Self::attendance_status_to_db(&status);

        let row = sqlx::query!(
            r#"
            SELECT COUNT(*) AS "count!"
            FROM convocation_recipients
            WHERE convocation_id = $1 AND attendance_status = $2
            "#,
            convocation_id,
            status_str
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to count by attendance status: {}", e))?;

        Ok(row.count)
    }

    async fn get_tracking_summary(
        &self,
        convocation_id: Uuid,
    ) -> Result<RecipientTrackingSummary, String> {
        let row = sqlx::query!(
            r#"
            SELECT
                COUNT(*) AS "total_count!",
                COUNT(*) FILTER (WHERE email_opened_at IS NOT NULL) AS "opened_count!",
                COUNT(*) FILTER (WHERE attendance_status = 'will_attend') AS "will_attend_count!",
                COUNT(*) FILTER (WHERE attendance_status = 'will_not_attend') AS "will_not_attend_count!",
                COUNT(*) FILTER (WHERE attendance_status = 'attended') AS "attended_count!",
                COUNT(*) FILTER (WHERE attendance_status = 'did_not_attend') AS "did_not_attend_count!",
                COUNT(*) FILTER (WHERE attendance_status = 'pending') AS "pending_count!",
                COUNT(*) FILTER (WHERE email_failed = TRUE) AS "failed_email_count!"
            FROM convocation_recipients
            WHERE convocation_id = $1
            "#,
            convocation_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to get tracking summary: {}", e))?;

        Ok(RecipientTrackingSummary {
            total_count: row.total_count,
            opened_count: row.opened_count,
            will_attend_count: row.will_attend_count,
            will_not_attend_count: row.will_not_attend_count,
            attended_count: row.attended_count,
            did_not_attend_count: row.did_not_attend_count,
            pending_count: row.pending_count,
            failed_email_count: row.failed_email_count,
        })
    }
}
