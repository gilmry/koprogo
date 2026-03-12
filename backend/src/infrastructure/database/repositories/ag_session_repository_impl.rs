use crate::application::ports::ag_session_repository::AgSessionRepository;
use crate::domain::entities::ag_session::{AgSession, AgSessionStatus, VideoPlatform};
use crate::infrastructure::database::pool::DbPool;
use async_trait::async_trait;
use sqlx::Row;
use uuid::Uuid;

pub struct PostgresAgSessionRepository {
    pool: DbPool,
}

impl PostgresAgSessionRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

fn row_to_ag_session(row: &sqlx::postgres::PgRow) -> AgSession {
    let platform_str: String = row.get("platform");
    let platform = VideoPlatform::from_db_string(&platform_str).unwrap_or(VideoPlatform::Other);

    let status_str: String = row.get("status");
    let status = AgSessionStatus::from_db_string(&status_str).unwrap_or(AgSessionStatus::Scheduled);

    AgSession {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        meeting_id: row.get("meeting_id"),
        platform,
        video_url: row.get("video_url"),
        host_url: row.get("host_url"),
        status,
        scheduled_start: row.get("scheduled_start"),
        actual_start: row.get("actual_start"),
        actual_end: row.get("actual_end"),
        remote_attendees_count: row.get("remote_attendees_count"),
        remote_voting_power: row.get("remote_voting_power"),
        quorum_remote_contribution: row.get("quorum_remote_contribution"),
        access_password: row.get("access_password"),
        waiting_room_enabled: row.get("waiting_room_enabled"),
        recording_enabled: row.get("recording_enabled"),
        recording_url: row.get("recording_url"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
        created_by: row.get("created_by"),
    }
}

#[async_trait]
impl AgSessionRepository for PostgresAgSessionRepository {
    async fn create(&self, session: &AgSession) -> Result<AgSession, String> {
        sqlx::query(
            r#"
            INSERT INTO ag_sessions (
                id, organization_id, meeting_id, platform, video_url, host_url,
                status, scheduled_start, actual_start, actual_end,
                remote_attendees_count, remote_voting_power, quorum_remote_contribution,
                access_password, waiting_room_enabled, recording_enabled, recording_url,
                created_at, updated_at, created_by
            ) VALUES (
                $1, $2, $3, $4::video_platform, $5, $6,
                $7::ag_session_status, $8, $9, $10,
                $11, $12, $13,
                $14, $15, $16, $17,
                $18, $19, $20
            )
            "#,
        )
        .bind(session.id)
        .bind(session.organization_id)
        .bind(session.meeting_id)
        .bind(session.platform.to_db_str())
        .bind(&session.video_url)
        .bind(&session.host_url)
        .bind(session.status.to_db_str())
        .bind(session.scheduled_start)
        .bind(session.actual_start)
        .bind(session.actual_end)
        .bind(session.remote_attendees_count)
        .bind(session.remote_voting_power)
        .bind(session.quorum_remote_contribution)
        .bind(&session.access_password)
        .bind(session.waiting_room_enabled)
        .bind(session.recording_enabled)
        .bind(&session.recording_url)
        .bind(session.created_at)
        .bind(session.updated_at)
        .bind(session.created_by)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Database error creating ag_session: {}", e))?;

        Ok(session.clone())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<AgSession>, String> {
        let row = sqlx::query(
            r#"
            SELECT id, organization_id, meeting_id, platform::TEXT, video_url, host_url,
                   status::TEXT, scheduled_start, actual_start, actual_end,
                   remote_attendees_count, remote_voting_power::FLOAT8,
                   quorum_remote_contribution::FLOAT8,
                   access_password, waiting_room_enabled, recording_enabled, recording_url,
                   created_at, updated_at, created_by
            FROM ag_sessions
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Database error finding ag_session: {}", e))?;

        Ok(row.as_ref().map(row_to_ag_session))
    }

    async fn find_by_meeting_id(&self, meeting_id: Uuid) -> Result<Option<AgSession>, String> {
        let row = sqlx::query(
            r#"
            SELECT id, organization_id, meeting_id, platform::TEXT, video_url, host_url,
                   status::TEXT, scheduled_start, actual_start, actual_end,
                   remote_attendees_count, remote_voting_power::FLOAT8,
                   quorum_remote_contribution::FLOAT8,
                   access_password, waiting_room_enabled, recording_enabled, recording_url,
                   created_at, updated_at, created_by
            FROM ag_sessions
            WHERE meeting_id = $1
            "#,
        )
        .bind(meeting_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Database error finding ag_session by meeting: {}", e))?;

        Ok(row.as_ref().map(row_to_ag_session))
    }

    async fn find_by_organization(&self, organization_id: Uuid) -> Result<Vec<AgSession>, String> {
        let rows = sqlx::query(
            r#"
            SELECT id, organization_id, meeting_id, platform::TEXT, video_url, host_url,
                   status::TEXT, scheduled_start, actual_start, actual_end,
                   remote_attendees_count, remote_voting_power::FLOAT8,
                   quorum_remote_contribution::FLOAT8,
                   access_password, waiting_room_enabled, recording_enabled, recording_url,
                   created_at, updated_at, created_by
            FROM ag_sessions
            WHERE organization_id = $1
            ORDER BY scheduled_start DESC
            "#,
        )
        .bind(organization_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Database error listing ag_sessions: {}", e))?;

        Ok(rows.iter().map(row_to_ag_session).collect())
    }

    async fn update(&self, session: &AgSession) -> Result<AgSession, String> {
        sqlx::query(
            r#"
            UPDATE ag_sessions SET
                platform = $2::video_platform,
                video_url = $3,
                host_url = $4,
                status = $5::ag_session_status,
                scheduled_start = $6,
                actual_start = $7,
                actual_end = $8,
                remote_attendees_count = $9,
                remote_voting_power = $10,
                quorum_remote_contribution = $11,
                access_password = $12,
                waiting_room_enabled = $13,
                recording_enabled = $14,
                recording_url = $15,
                updated_at = $16
            WHERE id = $1
            "#,
        )
        .bind(session.id)
        .bind(session.platform.to_db_str())
        .bind(&session.video_url)
        .bind(&session.host_url)
        .bind(session.status.to_db_str())
        .bind(session.scheduled_start)
        .bind(session.actual_start)
        .bind(session.actual_end)
        .bind(session.remote_attendees_count)
        .bind(session.remote_voting_power)
        .bind(session.quorum_remote_contribution)
        .bind(&session.access_password)
        .bind(session.waiting_room_enabled)
        .bind(session.recording_enabled)
        .bind(&session.recording_url)
        .bind(session.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Database error updating ag_session: {}", e))?;

        Ok(session.clone())
    }

    async fn delete(&self, id: Uuid) -> Result<bool, String> {
        let result = sqlx::query("DELETE FROM ag_sessions WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("Database error deleting ag_session: {}", e))?;

        Ok(result.rows_affected() > 0)
    }

    async fn find_pending_start(&self) -> Result<Vec<AgSession>, String> {
        let rows = sqlx::query(
            r#"
            SELECT id, organization_id, meeting_id, platform::TEXT, video_url, host_url,
                   status::TEXT, scheduled_start, actual_start, actual_end,
                   remote_attendees_count, remote_voting_power::FLOAT8,
                   quorum_remote_contribution::FLOAT8,
                   access_password, waiting_room_enabled, recording_enabled, recording_url,
                   created_at, updated_at, created_by
            FROM ag_sessions
            WHERE status = 'scheduled'
              AND scheduled_start <= NOW()
            ORDER BY scheduled_start ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Database error finding pending ag_sessions: {}", e))?;

        Ok(rows.iter().map(row_to_ag_session).collect())
    }
}
