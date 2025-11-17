use crate::application::ports::ConvocationRepository;
use crate::domain::entities::{Convocation, ConvocationStatus, ConvocationType};
use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use sqlx::PgPool;
use uuid::Uuid;

/// PostgreSQL implementation of ConvocationRepository
pub struct PostgresConvocationRepository {
    pool: PgPool,
}

impl PostgresConvocationRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Convert ConvocationType enum to database string
    fn convocation_type_to_db(meeting_type: &ConvocationType) -> &'static str {
        meeting_type.to_db_string()
    }

    /// Convert database string to ConvocationType enum
    fn convocation_type_from_db(s: &str) -> Result<ConvocationType, String> {
        ConvocationType::from_db_string(s)
    }

    /// Convert ConvocationStatus enum to database string
    fn status_to_db(status: &ConvocationStatus) -> &'static str {
        status.to_db_string()
    }

    /// Convert database string to ConvocationStatus enum
    fn status_from_db(s: &str) -> Result<ConvocationStatus, String> {
        ConvocationStatus::from_db_string(s)
    }
}

#[async_trait]
impl ConvocationRepository for PostgresConvocationRepository {
    async fn create(&self, convocation: &Convocation) -> Result<Convocation, String> {
        let meeting_type_str = Self::convocation_type_to_db(&convocation.meeting_type);
        let status_str = Self::status_to_db(&convocation.status);

        let row = sqlx::query!(
            r#"
            INSERT INTO convocations (
                id, organization_id, building_id, meeting_id, meeting_type, meeting_date,
                status, minimum_send_date, actual_send_date, scheduled_send_date,
                pdf_file_path, language, total_recipients, opened_count,
                will_attend_count, will_not_attend_count, reminder_sent_at,
                created_at, updated_at, created_by
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20)
            RETURNING id, organization_id, building_id, meeting_id, meeting_type::text AS "meeting_type!", meeting_date,
                      status::text AS "status!", minimum_send_date, actual_send_date, scheduled_send_date,
                      pdf_file_path, language, total_recipients, opened_count,
                      will_attend_count, will_not_attend_count, reminder_sent_at,
                      created_at, updated_at, created_by
            "#,
            convocation.id,
            convocation.organization_id,
            convocation.building_id,
            convocation.meeting_id,
            meeting_type_str,
            convocation.meeting_date,
            status_str,
            convocation.minimum_send_date,
            convocation.actual_send_date,
            convocation.scheduled_send_date,
            convocation.pdf_file_path,
            convocation.language,
            convocation.total_recipients,
            convocation.opened_count,
            convocation.will_attend_count,
            convocation.will_not_attend_count,
            convocation.reminder_sent_at,
            convocation.created_at,
            convocation.updated_at,
            convocation.created_by,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to create convocation: {}", e))?;

        Ok(Convocation {
            id: row.id,
            organization_id: row.organization_id,
            building_id: row.building_id,
            meeting_id: row.meeting_id,
            meeting_type: Self::convocation_type_from_db(&row.meeting_type)?,
            meeting_date: row.meeting_date,
            status: Self::status_from_db(&row.status)?,
            minimum_send_date: row.minimum_send_date,
            actual_send_date: row.actual_send_date,
            scheduled_send_date: row.scheduled_send_date,
            pdf_file_path: row.pdf_file_path,
            language: row.language,
            total_recipients: row.total_recipients,
            opened_count: row.opened_count,
            will_attend_count: row.will_attend_count,
            will_not_attend_count: row.will_not_attend_count,
            reminder_sent_at: row.reminder_sent_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
            created_by: row.created_by,
        })
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Convocation>, String> {
        let row = sqlx::query!(
            r#"
            SELECT id, organization_id, building_id, meeting_id, meeting_type::text AS "meeting_type!", meeting_date,
                   status::text AS "status!", minimum_send_date, actual_send_date, scheduled_send_date,
                   pdf_file_path, language, total_recipients, opened_count,
                   will_attend_count, will_not_attend_count, reminder_sent_at,
                   created_at, updated_at, created_by
            FROM convocations
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Failed to find convocation by id: {}", e))?;

        match row {
            Some(row) => Ok(Some(Convocation {
                id: row.id,
                organization_id: row.organization_id,
                building_id: row.building_id,
                meeting_id: row.meeting_id,
                meeting_type: Self::convocation_type_from_db(&row.meeting_type)?,
                meeting_date: row.meeting_date,
                status: Self::status_from_db(&row.status)?,
                minimum_send_date: row.minimum_send_date,
                actual_send_date: row.actual_send_date,
                scheduled_send_date: row.scheduled_send_date,
                pdf_file_path: row.pdf_file_path,
                language: row.language,
                total_recipients: row.total_recipients,
                opened_count: row.opened_count,
                will_attend_count: row.will_attend_count,
                will_not_attend_count: row.will_not_attend_count,
                reminder_sent_at: row.reminder_sent_at,
                created_at: row.created_at,
                updated_at: row.updated_at,
                created_by: row.created_by,
            })),
            None => Ok(None),
        }
    }

    async fn find_by_meeting_id(&self, meeting_id: Uuid) -> Result<Option<Convocation>, String> {
        let row = sqlx::query!(
            r#"
            SELECT id, organization_id, building_id, meeting_id, meeting_type::text AS "meeting_type!", meeting_date,
                   status::text AS "status!", minimum_send_date, actual_send_date, scheduled_send_date,
                   pdf_file_path, language, total_recipients, opened_count,
                   will_attend_count, will_not_attend_count, reminder_sent_at,
                   created_at, updated_at, created_by
            FROM convocations
            WHERE meeting_id = $1
            "#,
            meeting_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Failed to find convocation by meeting_id: {}", e))?;

        match row {
            Some(row) => Ok(Some(Convocation {
                id: row.id,
                organization_id: row.organization_id,
                building_id: row.building_id,
                meeting_id: row.meeting_id,
                meeting_type: Self::convocation_type_from_db(&row.meeting_type)?,
                meeting_date: row.meeting_date,
                status: Self::status_from_db(&row.status)?,
                minimum_send_date: row.minimum_send_date,
                actual_send_date: row.actual_send_date,
                scheduled_send_date: row.scheduled_send_date,
                pdf_file_path: row.pdf_file_path,
                language: row.language,
                total_recipients: row.total_recipients,
                opened_count: row.opened_count,
                will_attend_count: row.will_attend_count,
                will_not_attend_count: row.will_not_attend_count,
                reminder_sent_at: row.reminder_sent_at,
                created_at: row.created_at,
                updated_at: row.updated_at,
                created_by: row.created_by,
            })),
            None => Ok(None),
        }
    }

    async fn find_by_building(&self, building_id: Uuid) -> Result<Vec<Convocation>, String> {
        let rows = sqlx::query!(
            r#"
            SELECT id, organization_id, building_id, meeting_id, meeting_type::text AS "meeting_type!", meeting_date,
                   status::text AS "status!", minimum_send_date, actual_send_date, scheduled_send_date,
                   pdf_file_path, language, total_recipients, opened_count,
                   will_attend_count, will_not_attend_count, reminder_sent_at,
                   created_at, updated_at, created_by
            FROM convocations
            WHERE building_id = $1
            ORDER BY meeting_date DESC
            "#,
            building_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to find convocations by building: {}", e))?;

        rows.into_iter()
            .map(|row| {
                Ok(Convocation {
                    id: row.id,
                    organization_id: row.organization_id,
                    building_id: row.building_id,
                    meeting_id: row.meeting_id,
                    meeting_type: Self::convocation_type_from_db(&row.meeting_type)?,
                    meeting_date: row.meeting_date,
                    status: Self::status_from_db(&row.status)?,
                    minimum_send_date: row.minimum_send_date,
                    actual_send_date: row.actual_send_date,
                    scheduled_send_date: row.scheduled_send_date,
                    pdf_file_path: row.pdf_file_path,
                    language: row.language,
                    total_recipients: row.total_recipients,
                    opened_count: row.opened_count,
                    will_attend_count: row.will_attend_count,
                    will_not_attend_count: row.will_not_attend_count,
                    reminder_sent_at: row.reminder_sent_at,
                    created_at: row.created_at,
                    updated_at: row.updated_at,
                    created_by: row.created_by,
                })
            })
            .collect()
    }

    async fn find_by_organization(
        &self,
        organization_id: Uuid,
    ) -> Result<Vec<Convocation>, String> {
        let rows = sqlx::query!(
            r#"
            SELECT id, organization_id, building_id, meeting_id, meeting_type::text AS "meeting_type!", meeting_date,
                   status::text AS "status!", minimum_send_date, actual_send_date, scheduled_send_date,
                   pdf_file_path, language, total_recipients, opened_count,
                   will_attend_count, will_not_attend_count, reminder_sent_at,
                   created_at, updated_at, created_by
            FROM convocations
            WHERE organization_id = $1
            ORDER BY meeting_date DESC
            "#,
            organization_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to find convocations by organization: {}", e))?;

        rows.into_iter()
            .map(|row| {
                Ok(Convocation {
                    id: row.id,
                    organization_id: row.organization_id,
                    building_id: row.building_id,
                    meeting_id: row.meeting_id,
                    meeting_type: Self::convocation_type_from_db(&row.meeting_type)?,
                    meeting_date: row.meeting_date,
                    status: Self::status_from_db(&row.status)?,
                    minimum_send_date: row.minimum_send_date,
                    actual_send_date: row.actual_send_date,
                    scheduled_send_date: row.scheduled_send_date,
                    pdf_file_path: row.pdf_file_path,
                    language: row.language,
                    total_recipients: row.total_recipients,
                    opened_count: row.opened_count,
                    will_attend_count: row.will_attend_count,
                    will_not_attend_count: row.will_not_attend_count,
                    reminder_sent_at: row.reminder_sent_at,
                    created_at: row.created_at,
                    updated_at: row.updated_at,
                    created_by: row.created_by,
                })
            })
            .collect()
    }

    async fn find_by_status(
        &self,
        organization_id: Uuid,
        status: ConvocationStatus,
    ) -> Result<Vec<Convocation>, String> {
        let status_str = Self::status_to_db(&status);

        let rows = sqlx::query!(
            r#"
            SELECT id, organization_id, building_id, meeting_id, meeting_type::text AS "meeting_type!", meeting_date,
                   status::text AS "status!", minimum_send_date, actual_send_date, scheduled_send_date,
                   pdf_file_path, language, total_recipients, opened_count,
                   will_attend_count, will_not_attend_count, reminder_sent_at,
                   created_at, updated_at, created_by
            FROM convocations
            WHERE organization_id = $1 AND status = $2
            ORDER BY meeting_date DESC
            "#,
            organization_id,
            status_str
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to find convocations by status: {}", e))?;

        rows.into_iter()
            .map(|row| {
                Ok(Convocation {
                    id: row.id,
                    organization_id: row.organization_id,
                    building_id: row.building_id,
                    meeting_id: row.meeting_id,
                    meeting_type: Self::convocation_type_from_db(&row.meeting_type)?,
                    meeting_date: row.meeting_date,
                    status: Self::status_from_db(&row.status)?,
                    minimum_send_date: row.minimum_send_date,
                    actual_send_date: row.actual_send_date,
                    scheduled_send_date: row.scheduled_send_date,
                    pdf_file_path: row.pdf_file_path,
                    language: row.language,
                    total_recipients: row.total_recipients,
                    opened_count: row.opened_count,
                    will_attend_count: row.will_attend_count,
                    will_not_attend_count: row.will_not_attend_count,
                    reminder_sent_at: row.reminder_sent_at,
                    created_at: row.created_at,
                    updated_at: row.updated_at,
                    created_by: row.created_by,
                })
            })
            .collect()
    }

    async fn find_pending_scheduled(&self, now: DateTime<Utc>) -> Result<Vec<Convocation>, String> {
        let rows = sqlx::query!(
            r#"
            SELECT id, organization_id, building_id, meeting_id, meeting_type::text AS "meeting_type!", meeting_date,
                   status::text AS "status!", minimum_send_date, actual_send_date, scheduled_send_date,
                   pdf_file_path, language, total_recipients, opened_count,
                   will_attend_count, will_not_attend_count, reminder_sent_at,
                   created_at, updated_at, created_by
            FROM convocations
            WHERE status = 'scheduled'
              AND scheduled_send_date IS NOT NULL
              AND scheduled_send_date <= $1
            ORDER BY scheduled_send_date ASC
            "#,
            now
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to find pending scheduled convocations: {}", e))?;

        rows.into_iter()
            .map(|row| {
                Ok(Convocation {
                    id: row.id,
                    organization_id: row.organization_id,
                    building_id: row.building_id,
                    meeting_id: row.meeting_id,
                    meeting_type: Self::convocation_type_from_db(&row.meeting_type)?,
                    meeting_date: row.meeting_date,
                    status: Self::status_from_db(&row.status)?,
                    minimum_send_date: row.minimum_send_date,
                    actual_send_date: row.actual_send_date,
                    scheduled_send_date: row.scheduled_send_date,
                    pdf_file_path: row.pdf_file_path,
                    language: row.language,
                    total_recipients: row.total_recipients,
                    opened_count: row.opened_count,
                    will_attend_count: row.will_attend_count,
                    will_not_attend_count: row.will_not_attend_count,
                    reminder_sent_at: row.reminder_sent_at,
                    created_at: row.created_at,
                    updated_at: row.updated_at,
                    created_by: row.created_by,
                })
            })
            .collect()
    }

    async fn find_needing_reminder(&self, now: DateTime<Utc>) -> Result<Vec<Convocation>, String> {
        // Find convocations that:
        // - Status = sent
        // - Meeting date is in 0-3 days
        // - Reminder not sent yet (reminder_sent_at IS NULL)
        let three_days_from_now = now + Duration::days(3);

        let rows = sqlx::query!(
            r#"
            SELECT id, organization_id, building_id, meeting_id, meeting_type::text AS "meeting_type!", meeting_date,
                   status::text AS "status!", minimum_send_date, actual_send_date, scheduled_send_date,
                   pdf_file_path, language, total_recipients, opened_count,
                   will_attend_count, will_not_attend_count, reminder_sent_at,
                   created_at, updated_at, created_by
            FROM convocations
            WHERE status = 'sent'
              AND meeting_date >= $1
              AND meeting_date <= $2
              AND reminder_sent_at IS NULL
            ORDER BY meeting_date ASC
            "#,
            now,
            three_days_from_now
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to find convocations needing reminder: {}", e))?;

        rows.into_iter()
            .map(|row| {
                Ok(Convocation {
                    id: row.id,
                    organization_id: row.organization_id,
                    building_id: row.building_id,
                    meeting_id: row.meeting_id,
                    meeting_type: Self::convocation_type_from_db(&row.meeting_type)?,
                    meeting_date: row.meeting_date,
                    status: Self::status_from_db(&row.status)?,
                    minimum_send_date: row.minimum_send_date,
                    actual_send_date: row.actual_send_date,
                    scheduled_send_date: row.scheduled_send_date,
                    pdf_file_path: row.pdf_file_path,
                    language: row.language,
                    total_recipients: row.total_recipients,
                    opened_count: row.opened_count,
                    will_attend_count: row.will_attend_count,
                    will_not_attend_count: row.will_not_attend_count,
                    reminder_sent_at: row.reminder_sent_at,
                    created_at: row.created_at,
                    updated_at: row.updated_at,
                    created_by: row.created_by,
                })
            })
            .collect()
    }

    async fn update(&self, convocation: &Convocation) -> Result<Convocation, String> {
        let meeting_type_str = Self::convocation_type_to_db(&convocation.meeting_type);
        let status_str = Self::status_to_db(&convocation.status);

        let row = sqlx::query!(
            r#"
            UPDATE convocations
            SET organization_id = $2, building_id = $3, meeting_id = $4, meeting_type = $5, meeting_date = $6,
                status = $7, minimum_send_date = $8, actual_send_date = $9, scheduled_send_date = $10,
                pdf_file_path = $11, language = $12, total_recipients = $13, opened_count = $14,
                will_attend_count = $15, will_not_attend_count = $16, reminder_sent_at = $17,
                updated_at = $18
            WHERE id = $1
            RETURNING id, organization_id, building_id, meeting_id, meeting_type::text AS "meeting_type!", meeting_date,
                      status::text AS "status!", minimum_send_date, actual_send_date, scheduled_send_date,
                      pdf_file_path, language, total_recipients, opened_count,
                      will_attend_count, will_not_attend_count, reminder_sent_at,
                      created_at, updated_at, created_by
            "#,
            convocation.id,
            convocation.organization_id,
            convocation.building_id,
            convocation.meeting_id,
            meeting_type_str,
            convocation.meeting_date,
            status_str,
            convocation.minimum_send_date,
            convocation.actual_send_date,
            convocation.scheduled_send_date,
            convocation.pdf_file_path,
            convocation.language,
            convocation.total_recipients,
            convocation.opened_count,
            convocation.will_attend_count,
            convocation.will_not_attend_count,
            convocation.reminder_sent_at,
            convocation.updated_at,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to update convocation: {}", e))?;

        Ok(Convocation {
            id: row.id,
            organization_id: row.organization_id,
            building_id: row.building_id,
            meeting_id: row.meeting_id,
            meeting_type: Self::convocation_type_from_db(&row.meeting_type)?,
            meeting_date: row.meeting_date,
            status: Self::status_from_db(&row.status)?,
            minimum_send_date: row.minimum_send_date,
            actual_send_date: row.actual_send_date,
            scheduled_send_date: row.scheduled_send_date,
            pdf_file_path: row.pdf_file_path,
            language: row.language,
            total_recipients: row.total_recipients,
            opened_count: row.opened_count,
            will_attend_count: row.will_attend_count,
            will_not_attend_count: row.will_not_attend_count,
            reminder_sent_at: row.reminder_sent_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
            created_by: row.created_by,
        })
    }

    async fn delete(&self, id: Uuid) -> Result<bool, String> {
        let result = sqlx::query!(
            r#"
            DELETE FROM convocations
            WHERE id = $1
            "#,
            id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to delete convocation: {}", e))?;

        Ok(result.rows_affected() > 0)
    }

    async fn count_by_building(&self, building_id: Uuid) -> Result<i64, String> {
        let row = sqlx::query!(
            r#"
            SELECT COUNT(*) AS "count!"
            FROM convocations
            WHERE building_id = $1
            "#,
            building_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to count convocations by building: {}", e))?;

        Ok(row.count)
    }

    async fn count_by_status(
        &self,
        organization_id: Uuid,
        status: ConvocationStatus,
    ) -> Result<i64, String> {
        let status_str = Self::status_to_db(&status);

        let row = sqlx::query!(
            r#"
            SELECT COUNT(*) AS "count!"
            FROM convocations
            WHERE organization_id = $1 AND status = $2
            "#,
            organization_id,
            status_str
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to count convocations by status: {}", e))?;

        Ok(row.count)
    }
}
