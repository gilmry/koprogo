use crate::application::ports::BoardDecisionRepository;
use crate::domain::entities::{BoardDecision, DecisionStatus};
use crate::infrastructure::database::pool::DbPool;
use async_trait::async_trait;
use sqlx::Row;
use uuid::Uuid;

pub struct PostgresBoardDecisionRepository {
    pool: DbPool,
}

impl PostgresBoardDecisionRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl BoardDecisionRepository for PostgresBoardDecisionRepository {
    async fn create(&self, decision: &BoardDecision) -> Result<BoardDecision, String> {
        let status_str = decision.status.to_string();

        // Get organization_id from building
        let organization_id: Uuid =
            sqlx::query_scalar("SELECT organization_id FROM buildings WHERE id = $1")
                .bind(decision.building_id)
                .fetch_one(&self.pool)
                .await
                .map_err(|e| format!("Failed to get building organization: {}", e))?;

        sqlx::query(
            r#"
            INSERT INTO board_decisions (id, building_id, meeting_id, organization_id, subject, decision_text, deadline, status, completed_at, notes, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8::decision_status, $9, $10, $11, $12)
            "#,
        )
        .bind(decision.id)
        .bind(decision.building_id)
        .bind(decision.meeting_id)
        .bind(organization_id)
        .bind(&decision.subject)
        .bind(&decision.decision_text)
        .bind(decision.deadline)
        .bind(&status_str)
        .bind(decision.completed_at)
        .bind(&decision.notes)
        .bind(decision.created_at)
        .bind(decision.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(decision.clone())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<BoardDecision>, String> {
        let row = sqlx::query(
            r#"
            SELECT id, building_id, meeting_id, subject, decision_text, deadline, status::TEXT as status, completed_at, notes, created_at, updated_at
            FROM board_decisions
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(row.map(|row| {
            let status_str: String = row.get("status");
            let status = status_str
                .parse::<DecisionStatus>()
                .unwrap_or(DecisionStatus::Pending);

            BoardDecision {
                id: row.get("id"),
                building_id: row.get("building_id"),
                meeting_id: row.get("meeting_id"),
                subject: row.get("subject"),
                decision_text: row.get("decision_text"),
                deadline: row.get("deadline"),
                status,
                completed_at: row.get("completed_at"),
                notes: row.get("notes"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            }
        }))
    }

    async fn find_by_building(&self, building_id: Uuid) -> Result<Vec<BoardDecision>, String> {
        let rows = sqlx::query(
            r#"
            SELECT id, building_id, meeting_id, subject, decision_text, deadline, status::TEXT as status, completed_at, notes, created_at, updated_at
            FROM board_decisions
            WHERE building_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(building_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(rows
            .into_iter()
            .map(|row| {
                let status_str: String = row.get("status");
                let status = status_str
                    .parse::<DecisionStatus>()
                    .unwrap_or(DecisionStatus::Pending);

                BoardDecision {
                    id: row.get("id"),
                    building_id: row.get("building_id"),
                    meeting_id: row.get("meeting_id"),
                    subject: row.get("subject"),
                    decision_text: row.get("decision_text"),
                    deadline: row.get("deadline"),
                    status,
                    completed_at: row.get("completed_at"),
                    notes: row.get("notes"),
                    created_at: row.get("created_at"),
                    updated_at: row.get("updated_at"),
                }
            })
            .collect())
    }

    async fn find_by_meeting(&self, meeting_id: Uuid) -> Result<Vec<BoardDecision>, String> {
        let rows = sqlx::query(
            r#"
            SELECT id, building_id, meeting_id, subject, decision_text, deadline, status::TEXT as status, completed_at, notes, created_at, updated_at
            FROM board_decisions
            WHERE meeting_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(meeting_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(rows
            .into_iter()
            .map(|row| {
                let status_str: String = row.get("status");
                let status = status_str
                    .parse::<DecisionStatus>()
                    .unwrap_or(DecisionStatus::Pending);

                BoardDecision {
                    id: row.get("id"),
                    building_id: row.get("building_id"),
                    meeting_id: row.get("meeting_id"),
                    subject: row.get("subject"),
                    decision_text: row.get("decision_text"),
                    deadline: row.get("deadline"),
                    status,
                    completed_at: row.get("completed_at"),
                    notes: row.get("notes"),
                    created_at: row.get("created_at"),
                    updated_at: row.get("updated_at"),
                }
            })
            .collect())
    }

    async fn find_by_status(
        &self,
        building_id: Uuid,
        status: DecisionStatus,
    ) -> Result<Vec<BoardDecision>, String> {
        let status_str = status.to_string();

        let rows = sqlx::query(
            r#"
            SELECT id, building_id, meeting_id, subject, decision_text, deadline, status::TEXT as status, completed_at, notes, created_at, updated_at
            FROM board_decisions
            WHERE building_id = $1 AND status = $2::decision_status
            ORDER BY created_at DESC
            "#,
        )
        .bind(building_id)
        .bind(&status_str)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(rows
            .into_iter()
            .map(|row| {
                let status_str: String = row.get("status");
                let status = status_str
                    .parse::<DecisionStatus>()
                    .unwrap_or(DecisionStatus::Pending);

                BoardDecision {
                    id: row.get("id"),
                    building_id: row.get("building_id"),
                    meeting_id: row.get("meeting_id"),
                    subject: row.get("subject"),
                    decision_text: row.get("decision_text"),
                    deadline: row.get("deadline"),
                    status,
                    completed_at: row.get("completed_at"),
                    notes: row.get("notes"),
                    created_at: row.get("created_at"),
                    updated_at: row.get("updated_at"),
                }
            })
            .collect())
    }

    async fn find_overdue(&self, building_id: Uuid) -> Result<Vec<BoardDecision>, String> {
        let rows = sqlx::query(
            r#"
            SELECT id, building_id, meeting_id, subject, decision_text, deadline, status::TEXT as status, completed_at, notes, created_at, updated_at
            FROM board_decisions
            WHERE building_id = $1
              AND deadline IS NOT NULL
              AND deadline < CURRENT_TIMESTAMP
              AND status NOT IN ('completed', 'cancelled')
            ORDER BY deadline ASC
            "#,
        )
        .bind(building_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(rows
            .into_iter()
            .map(|row| {
                let status_str: String = row.get("status");
                let status = status_str
                    .parse::<DecisionStatus>()
                    .unwrap_or(DecisionStatus::Pending);

                BoardDecision {
                    id: row.get("id"),
                    building_id: row.get("building_id"),
                    meeting_id: row.get("meeting_id"),
                    subject: row.get("subject"),
                    decision_text: row.get("decision_text"),
                    deadline: row.get("deadline"),
                    status,
                    completed_at: row.get("completed_at"),
                    notes: row.get("notes"),
                    created_at: row.get("created_at"),
                    updated_at: row.get("updated_at"),
                }
            })
            .collect())
    }

    async fn find_deadline_approaching(
        &self,
        building_id: Uuid,
        days_threshold: i32,
    ) -> Result<Vec<BoardDecision>, String> {
        let rows = sqlx::query(
            r#"
            SELECT id, building_id, meeting_id, subject, decision_text, deadline, status::TEXT as status, completed_at, notes, created_at, updated_at
            FROM board_decisions
            WHERE building_id = $1
              AND deadline IS NOT NULL
              AND deadline > CURRENT_TIMESTAMP
              AND deadline <= (CURRENT_TIMESTAMP + INTERVAL '1 day' * $2)
              AND status NOT IN ('completed', 'cancelled')
            ORDER BY deadline ASC
            "#,
        )
        .bind(building_id)
        .bind(days_threshold)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(rows
            .into_iter()
            .map(|row| {
                let status_str: String = row.get("status");
                let status = status_str
                    .parse::<DecisionStatus>()
                    .unwrap_or(DecisionStatus::Pending);

                BoardDecision {
                    id: row.get("id"),
                    building_id: row.get("building_id"),
                    meeting_id: row.get("meeting_id"),
                    subject: row.get("subject"),
                    decision_text: row.get("decision_text"),
                    deadline: row.get("deadline"),
                    status,
                    completed_at: row.get("completed_at"),
                    notes: row.get("notes"),
                    created_at: row.get("created_at"),
                    updated_at: row.get("updated_at"),
                }
            })
            .collect())
    }

    async fn update(&self, decision: &BoardDecision) -> Result<BoardDecision, String> {
        let status_str = decision.status.to_string();

        sqlx::query(
            r#"
            UPDATE board_decisions
            SET subject = $1,
                decision_text = $2,
                deadline = $3,
                status = $4::decision_status,
                completed_at = $5,
                notes = $6,
                updated_at = $7
            WHERE id = $8
            "#,
        )
        .bind(&decision.subject)
        .bind(&decision.decision_text)
        .bind(decision.deadline)
        .bind(&status_str)
        .bind(decision.completed_at)
        .bind(&decision.notes)
        .bind(decision.updated_at)
        .bind(decision.id)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(decision.clone())
    }

    async fn delete(&self, id: Uuid) -> Result<bool, String> {
        let result = sqlx::query(
            r#"
            DELETE FROM board_decisions
            WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(result.rows_affected() > 0)
    }

    async fn count_by_status(
        &self,
        building_id: Uuid,
        status: DecisionStatus,
    ) -> Result<i64, String> {
        let status_str = status.to_string();

        let count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*)
            FROM board_decisions
            WHERE building_id = $1 AND status = $2::decision_status
            "#,
        )
        .bind(building_id)
        .bind(&status_str)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(count)
    }

    async fn count_overdue(&self, building_id: Uuid) -> Result<i64, String> {
        let count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*)
            FROM board_decisions
            WHERE building_id = $1
              AND deadline IS NOT NULL
              AND deadline < CURRENT_TIMESTAMP
              AND status NOT IN ('completed', 'cancelled')
            "#,
        )
        .bind(building_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(count)
    }
}
