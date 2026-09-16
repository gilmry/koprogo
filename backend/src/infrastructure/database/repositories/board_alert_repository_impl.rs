use crate::application::error::AppError;
use crate::application::ports::BoardAlertRepository;
use crate::domain::entities::{AlertSeverity, BoardAlert};
use crate::infrastructure::database::pool::DbPool;
use async_trait::async_trait;
use sqlx::Row;
use uuid::Uuid;

pub struct PostgresBoardAlertRepository {
    pool: DbPool,
}

impl PostgresBoardAlertRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

fn row_to_alert(row: sqlx::postgres::PgRow) -> BoardAlert {
    let severity_str: String = row.get("severity");
    let severity = severity_str.parse().unwrap_or(AlertSeverity::Info);

    BoardAlert {
        id: row.get("id"),
        building_id: row.get("building_id"),
        raised_by_board_member_id: row.get("raised_by_board_member_id"),
        text: row.get("text"),
        severity,
        target_meeting_id: row.get("target_meeting_id"),
        created_at: row.get("created_at"),
    }
}

#[async_trait]
impl BoardAlertRepository for PostgresBoardAlertRepository {
    async fn create(&self, alert: &BoardAlert) -> Result<BoardAlert, AppError> {
        sqlx::query(
            r#"
            INSERT INTO board_alerts
                (id, building_id, raised_by_board_member_id, text, severity, target_meeting_id, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
        )
        .bind(alert.id)
        .bind(alert.building_id)
        .bind(alert.raised_by_board_member_id)
        .bind(&alert.text)
        .bind(alert.severity.to_string())
        .bind(alert.target_meeting_id)
        .bind(alert.created_at)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Database(format!("Failed to create board alert: {}", e)))?;

        Ok(alert.clone())
    }

    async fn find_by_target_meeting(&self, meeting_id: Uuid) -> Result<Vec<BoardAlert>, AppError> {
        let rows = sqlx::query(
            r#"
            SELECT id, building_id, raised_by_board_member_id, text, severity, target_meeting_id, created_at
            FROM board_alerts
            WHERE target_meeting_id = $1
            ORDER BY created_at ASC
            "#,
        )
        .bind(meeting_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Database(format!("Failed to list board alerts: {}", e)))?;

        Ok(rows.into_iter().map(row_to_alert).collect())
    }
}
