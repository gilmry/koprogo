use crate::application::ports::MeetingRepository;
use crate::domain::entities::{Meeting, MeetingStatus, MeetingType};
use crate::infrastructure::database::pool::DbPool;
use async_trait::async_trait;
use sqlx::Row;
use uuid::Uuid;

pub struct PostgresMeetingRepository {
    pool: DbPool,
}

impl PostgresMeetingRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl MeetingRepository for PostgresMeetingRepository {
    async fn create(&self, meeting: &Meeting) -> Result<Meeting, String> {
        let meeting_type_str = match meeting.meeting_type {
            MeetingType::Ordinary => "ordinary",
            MeetingType::Extraordinary => "extraordinary",
        };

        let status_str = match meeting.status {
            MeetingStatus::Scheduled => "scheduled",
            MeetingStatus::Completed => "completed",
            MeetingStatus::Cancelled => "cancelled",
        };

        let agenda_json = serde_json::to_value(&meeting.agenda)
            .map_err(|e| format!("JSON serialization error: {}", e))?;

        sqlx::query(
            r#"
            INSERT INTO meetings (id, building_id, meeting_type, title, description, scheduled_date, location, status, agenda, attendees_count, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            "#,
        )
        .bind(meeting.id)
        .bind(meeting.building_id)
        .bind(meeting_type_str)
        .bind(&meeting.title)
        .bind(&meeting.description)
        .bind(meeting.scheduled_date)
        .bind(&meeting.location)
        .bind(status_str)
        .bind(agenda_json)
        .bind(meeting.attendees_count)
        .bind(meeting.created_at)
        .bind(meeting.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(meeting.clone())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Meeting>, String> {
        let row = sqlx::query(
            r#"
            SELECT id, building_id, meeting_type, title, description, scheduled_date, location, status, agenda, attendees_count, created_at, updated_at
            FROM meetings
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(row.map(|row| {
            let meeting_type_str: String = row.get("meeting_type");
            let meeting_type = match meeting_type_str.as_str() {
                "extraordinary" => MeetingType::Extraordinary,
                _ => MeetingType::Ordinary,
            };

            let status_str: String = row.get("status");
            let status = match status_str.as_str() {
                "completed" => MeetingStatus::Completed,
                "cancelled" => MeetingStatus::Cancelled,
                _ => MeetingStatus::Scheduled,
            };

            let agenda_json: serde_json::Value = row.get("agenda");
            let agenda: Vec<String> = serde_json::from_value(agenda_json).unwrap_or_default();

            Meeting {
                id: row.get("id"),
                building_id: row.get("building_id"),
                meeting_type,
                title: row.get("title"),
                description: row.get("description"),
                scheduled_date: row.get("scheduled_date"),
                location: row.get("location"),
                status,
                agenda,
                attendees_count: row.get("attendees_count"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            }
        }))
    }

    async fn find_by_building(&self, building_id: Uuid) -> Result<Vec<Meeting>, String> {
        let rows = sqlx::query(
            r#"
            SELECT id, building_id, meeting_type, title, description, scheduled_date, location, status, agenda, attendees_count, created_at, updated_at
            FROM meetings
            WHERE building_id = $1
            ORDER BY scheduled_date DESC
            "#,
        )
        .bind(building_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(rows
            .iter()
            .map(|row| {
                let meeting_type_str: String = row.get("meeting_type");
                let meeting_type = match meeting_type_str.as_str() {
                    "extraordinary" => MeetingType::Extraordinary,
                    _ => MeetingType::Ordinary,
                };

                let status_str: String = row.get("status");
                let status = match status_str.as_str() {
                    "completed" => MeetingStatus::Completed,
                    "cancelled" => MeetingStatus::Cancelled,
                    _ => MeetingStatus::Scheduled,
                };

                let agenda_json: serde_json::Value = row.get("agenda");
                let agenda: Vec<String> = serde_json::from_value(agenda_json).unwrap_or_default();

                Meeting {
                    id: row.get("id"),
                    building_id: row.get("building_id"),
                    meeting_type,
                    title: row.get("title"),
                    description: row.get("description"),
                    scheduled_date: row.get("scheduled_date"),
                    location: row.get("location"),
                    status,
                    agenda,
                    attendees_count: row.get("attendees_count"),
                    created_at: row.get("created_at"),
                    updated_at: row.get("updated_at"),
                }
            })
            .collect())
    }

    async fn update(&self, meeting: &Meeting) -> Result<Meeting, String> {
        let status_str = match meeting.status {
            MeetingStatus::Scheduled => "scheduled",
            MeetingStatus::Completed => "completed",
            MeetingStatus::Cancelled => "cancelled",
        };

        let agenda_json = serde_json::to_value(&meeting.agenda)
            .map_err(|e| format!("JSON serialization error: {}", e))?;

        sqlx::query(
            r#"
            UPDATE meetings
            SET status = $2, agenda = $3, attendees_count = $4, updated_at = $5
            WHERE id = $1
            "#,
        )
        .bind(meeting.id)
        .bind(status_str)
        .bind(agenda_json)
        .bind(meeting.attendees_count)
        .bind(meeting.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(meeting.clone())
    }

    async fn delete(&self, id: Uuid) -> Result<bool, String> {
        let result = sqlx::query("DELETE FROM meetings WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("Database error: {}", e))?;

        Ok(result.rows_affected() > 0)
    }
}
