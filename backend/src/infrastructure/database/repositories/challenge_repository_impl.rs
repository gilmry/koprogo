use crate::application::ports::{ChallengeProgressRepository, ChallengeRepository};
use crate::domain::entities::{Challenge, ChallengeProgress, ChallengeStatus, ChallengeType};
use crate::infrastructure::pool::DbPool;
use async_trait::async_trait;
use sqlx::Row;
use uuid::Uuid;

pub struct PostgresChallengeRepository {
    pool: DbPool,
}

impl PostgresChallengeRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    /// Helper to convert database row to Challenge entity
    fn row_to_challenge(row: &sqlx::postgres::PgRow) -> Result<Challenge, String> {
        // Parse ENUMs from database strings
        let challenge_type_str: String = row
            .try_get("challenge_type")
            .map_err(|e| format!("Failed to get challenge_type: {}", e))?;
        let challenge_type: ChallengeType =
            serde_json::from_str(&format!("\"{}\"", challenge_type_str))
                .map_err(|e| format!("Failed to parse challenge_type: {}", e))?;

        let status_str: String = row
            .try_get("status")
            .map_err(|e| format!("Failed to get status: {}", e))?;
        let status: ChallengeStatus = serde_json::from_str(&format!("\"{}\"", status_str))
            .map_err(|e| format!("Failed to parse status: {}", e))?;

        Ok(Challenge {
            id: row
                .try_get("id")
                .map_err(|e| format!("Failed to get id: {}", e))?,
            organization_id: row
                .try_get("organization_id")
                .map_err(|e| format!("Failed to get organization_id: {}", e))?,
            building_id: row
                .try_get("building_id")
                .map_err(|e| format!("Failed to get building_id: {}", e))?,
            challenge_type,
            status,
            title: row
                .try_get("title")
                .map_err(|e| format!("Failed to get title: {}", e))?,
            description: row
                .try_get("description")
                .map_err(|e| format!("Failed to get description: {}", e))?,
            icon: row
                .try_get("icon")
                .map_err(|e| format!("Failed to get icon: {}", e))?,
            start_date: row
                .try_get("start_date")
                .map_err(|e| format!("Failed to get start_date: {}", e))?,
            end_date: row
                .try_get("end_date")
                .map_err(|e| format!("Failed to get end_date: {}", e))?,
            target_metric: row
                .try_get("target_metric")
                .map_err(|e| format!("Failed to get target_metric: {}", e))?,
            target_value: row
                .try_get("target_value")
                .map_err(|e| format!("Failed to get target_value: {}", e))?,
            reward_points: row
                .try_get("reward_points")
                .map_err(|e| format!("Failed to get reward_points: {}", e))?,
            created_at: row
                .try_get("created_at")
                .map_err(|e| format!("Failed to get created_at: {}", e))?,
            updated_at: row
                .try_get("updated_at")
                .map_err(|e| format!("Failed to get updated_at: {}", e))?,
        })
    }
}

#[async_trait]
impl ChallengeRepository for PostgresChallengeRepository {
    async fn create(&self, challenge: &Challenge) -> Result<Challenge, String> {
        // Serialize ENUMs to strings for database
        let challenge_type_str = serde_json::to_string(&challenge.challenge_type)
            .map_err(|e| format!("Failed to serialize challenge_type: {}", e))?
            .trim_matches('"')
            .to_string();

        let status_str = serde_json::to_string(&challenge.status)
            .map_err(|e| format!("Failed to serialize status: {}", e))?
            .trim_matches('"')
            .to_string();

        sqlx::query(
            r#"
            INSERT INTO challenges (
                id, organization_id, building_id, challenge_type, status, title,
                description, icon, start_date, end_date, target_metric,
                target_value, reward_points, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4::challenge_type, $5::challenge_status, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
            "#,
        )
        .bind(&challenge.id)
        .bind(&challenge.organization_id)
        .bind(&challenge.building_id)
        .bind(&challenge_type_str)
        .bind(&status_str)
        .bind(&challenge.title)
        .bind(&challenge.description)
        .bind(&challenge.icon)
        .bind(&challenge.start_date)
        .bind(&challenge.end_date)
        .bind(&challenge.target_metric)
        .bind(&challenge.target_value)
        .bind(&challenge.reward_points)
        .bind(&challenge.created_at)
        .bind(&challenge.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to create challenge: {}", e))?;

        Ok(challenge.clone())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Challenge>, String> {
        let row = sqlx::query(
            r#"
            SELECT id, organization_id, building_id, challenge_type, status, title,
                   description, icon, start_date, end_date, target_metric,
                   target_value, reward_points, created_at, updated_at
            FROM challenges
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Failed to find challenge by id: {}", e))?;

        row.as_ref().map(Self::row_to_challenge).transpose()
    }

    async fn find_by_organization(&self, organization_id: Uuid) -> Result<Vec<Challenge>, String> {
        let rows = sqlx::query(
            r#"
            SELECT id, organization_id, building_id, challenge_type, status, title,
                   description, icon, start_date, end_date, target_metric,
                   target_value, reward_points, created_at, updated_at
            FROM challenges
            WHERE organization_id = $1
            ORDER BY start_date DESC
            "#,
        )
        .bind(organization_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to find challenges by organization: {}", e))?;

        rows.iter().map(Self::row_to_challenge).collect()
    }

    async fn find_by_organization_and_status(
        &self,
        organization_id: Uuid,
        status: ChallengeStatus,
    ) -> Result<Vec<Challenge>, String> {
        let status_str = serde_json::to_string(&status)
            .map_err(|e| format!("Failed to serialize status: {}", e))?
            .trim_matches('"')
            .to_string();

        let rows = sqlx::query(
            r#"
            SELECT id, organization_id, building_id, challenge_type, status, title,
                   description, icon, start_date, end_date, target_metric,
                   target_value, reward_points, created_at, updated_at
            FROM challenges
            WHERE organization_id = $1
              AND status = $2::challenge_status
            ORDER BY start_date DESC
            "#,
        )
        .bind(organization_id)
        .bind(&status_str)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to find challenges by status: {}", e))?;

        rows.iter().map(Self::row_to_challenge).collect()
    }

    async fn find_by_building(&self, building_id: Uuid) -> Result<Vec<Challenge>, String> {
        let rows = sqlx::query(
            r#"
            SELECT id, organization_id, building_id, challenge_type, status, title,
                   description, icon, start_date, end_date, target_metric,
                   target_value, reward_points, created_at, updated_at
            FROM challenges
            WHERE building_id = $1
            ORDER BY start_date DESC
            "#,
        )
        .bind(building_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to find challenges by building: {}", e))?;

        rows.iter().map(Self::row_to_challenge).collect()
    }

    async fn find_active(&self, organization_id: Uuid) -> Result<Vec<Challenge>, String> {
        let rows = sqlx::query(
            r#"
            SELECT id, organization_id, building_id, challenge_type, status, title,
                   description, icon, start_date, end_date, target_metric,
                   target_value, reward_points, created_at, updated_at
            FROM challenges
            WHERE organization_id = $1
              AND status = 'Active'
              AND start_date <= NOW()
              AND end_date > NOW()
            ORDER BY start_date DESC
            "#,
        )
        .bind(organization_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to find active challenges: {}", e))?;

        rows.iter().map(Self::row_to_challenge).collect()
    }

    async fn find_ended_not_completed(&self) -> Result<Vec<Challenge>, String> {
        let rows = sqlx::query(
            r#"
            SELECT id, organization_id, building_id, challenge_type, status, title,
                   description, icon, start_date, end_date, target_metric,
                   target_value, reward_points, created_at, updated_at
            FROM challenges
            WHERE status = 'Active'
              AND end_date <= NOW()
            ORDER BY end_date ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to find ended challenges: {}", e))?;

        rows.iter().map(Self::row_to_challenge).collect()
    }

    async fn update(&self, challenge: &Challenge) -> Result<Challenge, String> {
        let challenge_type_str = serde_json::to_string(&challenge.challenge_type)
            .map_err(|e| format!("Failed to serialize challenge_type: {}", e))?
            .trim_matches('"')
            .to_string();

        let status_str = serde_json::to_string(&challenge.status)
            .map_err(|e| format!("Failed to serialize status: {}", e))?
            .trim_matches('"')
            .to_string();

        let result = sqlx::query(
            r#"
            UPDATE challenges
            SET challenge_type = $2::challenge_type,
                status = $3::challenge_status,
                title = $4,
                description = $5,
                icon = $6,
                start_date = $7,
                end_date = $8,
                target_metric = $9,
                target_value = $10,
                reward_points = $11,
                updated_at = $12
            WHERE id = $1
            "#,
        )
        .bind(&challenge.id)
        .bind(&challenge_type_str)
        .bind(&status_str)
        .bind(&challenge.title)
        .bind(&challenge.description)
        .bind(&challenge.icon)
        .bind(&challenge.start_date)
        .bind(&challenge.end_date)
        .bind(&challenge.target_metric)
        .bind(&challenge.target_value)
        .bind(&challenge.reward_points)
        .bind(&challenge.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to update challenge: {}", e))?;

        if result.rows_affected() == 0 {
            return Err("Challenge not found".to_string());
        }

        Ok(challenge.clone())
    }

    async fn delete(&self, id: Uuid) -> Result<(), String> {
        let result = sqlx::query(
            r#"
            DELETE FROM challenges
            WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to delete challenge: {}", e))?;

        if result.rows_affected() == 0 {
            return Err("Challenge not found".to_string());
        }

        Ok(())
    }

    async fn count_by_organization(&self, organization_id: Uuid) -> Result<i64, String> {
        let row = sqlx::query(
            r#"
            SELECT COUNT(*) as count
            FROM challenges
            WHERE organization_id = $1
            "#,
        )
        .bind(organization_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to count challenges: {}", e))?;

        let count: i64 = row
            .try_get("count")
            .map_err(|e| format!("Failed to get count: {}", e))?;

        Ok(count)
    }
}

// ============================================================================
// ChallengeProgressRepository Implementation
// ============================================================================

pub struct PostgresChallengeProgressRepository {
    pool: DbPool,
}

impl PostgresChallengeProgressRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    /// Helper to convert database row to ChallengeProgress entity
    fn row_to_progress(row: &sqlx::postgres::PgRow) -> Result<ChallengeProgress, String> {
        Ok(ChallengeProgress {
            id: row
                .try_get("id")
                .map_err(|e| format!("Failed to get id: {}", e))?,
            challenge_id: row
                .try_get("challenge_id")
                .map_err(|e| format!("Failed to get challenge_id: {}", e))?,
            user_id: row
                .try_get("user_id")
                .map_err(|e| format!("Failed to get user_id: {}", e))?,
            current_value: row
                .try_get("current_value")
                .map_err(|e| format!("Failed to get current_value: {}", e))?,
            completed: row
                .try_get("completed")
                .map_err(|e| format!("Failed to get completed: {}", e))?,
            completed_at: row
                .try_get("completed_at")
                .map_err(|e| format!("Failed to get completed_at: {}", e))?,
            created_at: row
                .try_get("created_at")
                .map_err(|e| format!("Failed to get created_at: {}", e))?,
            updated_at: row
                .try_get("updated_at")
                .map_err(|e| format!("Failed to get updated_at: {}", e))?,
        })
    }
}

#[async_trait]
impl ChallengeProgressRepository for PostgresChallengeProgressRepository {
    async fn create(&self, progress: &ChallengeProgress) -> Result<ChallengeProgress, String> {
        sqlx::query(
            r#"
            INSERT INTO challenge_progress (
                id, challenge_id, user_id, current_value, completed,
                completed_at, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
        )
        .bind(&progress.id)
        .bind(&progress.challenge_id)
        .bind(&progress.user_id)
        .bind(&progress.current_value)
        .bind(&progress.completed)
        .bind(&progress.completed_at)
        .bind(&progress.created_at)
        .bind(&progress.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to create challenge progress: {}", e))?;

        Ok(progress.clone())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<ChallengeProgress>, String> {
        let row = sqlx::query(
            r#"
            SELECT id, challenge_id, user_id, current_value, completed,
                   completed_at, created_at, updated_at
            FROM challenge_progress
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Failed to find challenge progress by id: {}", e))?;

        row.as_ref().map(Self::row_to_progress).transpose()
    }

    async fn find_by_user_and_challenge(
        &self,
        user_id: Uuid,
        challenge_id: Uuid,
    ) -> Result<Option<ChallengeProgress>, String> {
        let row = sqlx::query(
            r#"
            SELECT id, challenge_id, user_id, current_value, completed,
                   completed_at, created_at, updated_at
            FROM challenge_progress
            WHERE user_id = $1 AND challenge_id = $2
            "#,
        )
        .bind(user_id)
        .bind(challenge_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Failed to find challenge progress: {}", e))?;

        row.as_ref().map(Self::row_to_progress).transpose()
    }

    async fn find_by_user(&self, user_id: Uuid) -> Result<Vec<ChallengeProgress>, String> {
        let rows = sqlx::query(
            r#"
            SELECT id, challenge_id, user_id, current_value, completed,
                   completed_at, created_at, updated_at
            FROM challenge_progress
            WHERE user_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to find progress by user: {}", e))?;

        rows.iter().map(Self::row_to_progress).collect()
    }

    async fn find_by_challenge(
        &self,
        challenge_id: Uuid,
    ) -> Result<Vec<ChallengeProgress>, String> {
        let rows = sqlx::query(
            r#"
            SELECT id, challenge_id, user_id, current_value, completed,
                   completed_at, created_at, updated_at
            FROM challenge_progress
            WHERE challenge_id = $1
            ORDER BY current_value DESC
            "#,
        )
        .bind(challenge_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to find progress by challenge: {}", e))?;

        rows.iter().map(Self::row_to_progress).collect()
    }

    async fn find_active_by_user(&self, user_id: Uuid) -> Result<Vec<ChallengeProgress>, String> {
        let rows = sqlx::query(
            r#"
            SELECT cp.id, cp.challenge_id, cp.user_id, cp.current_value,
                   cp.completed, cp.completed_at, cp.created_at, cp.updated_at
            FROM challenge_progress cp
            JOIN challenges c ON c.id = cp.challenge_id
            WHERE cp.user_id = $1
              AND c.status = 'Active'
              AND cp.completed = FALSE
            ORDER BY c.end_date ASC
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to find active progress: {}", e))?;

        rows.iter().map(Self::row_to_progress).collect()
    }

    async fn update(&self, progress: &ChallengeProgress) -> Result<ChallengeProgress, String> {
        let result = sqlx::query(
            r#"
            UPDATE challenge_progress
            SET current_value = $2,
                completed = $3,
                completed_at = $4,
                updated_at = $5
            WHERE id = $1
            "#,
        )
        .bind(&progress.id)
        .bind(&progress.current_value)
        .bind(&progress.completed)
        .bind(&progress.completed_at)
        .bind(&progress.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to update challenge progress: {}", e))?;

        if result.rows_affected() == 0 {
            return Err("Challenge progress not found".to_string());
        }

        Ok(progress.clone())
    }

    async fn count_completed_by_user(&self, user_id: Uuid) -> Result<i64, String> {
        let row = sqlx::query(
            r#"
            SELECT COUNT(*) as count
            FROM challenge_progress
            WHERE user_id = $1 AND completed = TRUE
            "#,
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to count completed challenges: {}", e))?;

        let count: i64 = row
            .try_get("count")
            .map_err(|e| format!("Failed to get count: {}", e))?;

        Ok(count)
    }

    async fn get_leaderboard(
        &self,
        organization_id: Uuid,
        building_id: Option<Uuid>,
        limit: i64,
    ) -> Result<Vec<(Uuid, i32)>, String> {
        // Calculate total points from completed challenges
        // Filter by organization and optionally by building
        let rows = if let Some(bldg_id) = building_id {
            sqlx::query(
                r#"
                SELECT cp.user_id, COALESCE(SUM(c.reward_points), 0)::INTEGER as total_points
                FROM challenge_progress cp
                JOIN challenges c ON c.id = cp.challenge_id
                WHERE c.organization_id = $1
                  AND c.building_id = $2
                  AND cp.completed = TRUE
                GROUP BY cp.user_id
                ORDER BY total_points DESC
                LIMIT $3
                "#,
            )
            .bind(organization_id)
            .bind(bldg_id)
            .bind(limit)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| format!("Failed to get leaderboard: {}", e))?
        } else {
            sqlx::query(
                r#"
                SELECT cp.user_id, COALESCE(SUM(c.reward_points), 0)::INTEGER as total_points
                FROM challenge_progress cp
                JOIN challenges c ON c.id = cp.challenge_id
                WHERE c.organization_id = $1
                  AND cp.completed = TRUE
                GROUP BY cp.user_id
                ORDER BY total_points DESC
                LIMIT $2
                "#,
            )
            .bind(organization_id)
            .bind(limit)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| format!("Failed to get leaderboard: {}", e))?
        };

        let leaderboard = rows
            .iter()
            .map(|row| {
                let user_id: Uuid = row
                    .try_get("user_id")
                    .map_err(|e| format!("Failed to get user_id: {}", e))?;
                let total_points: i32 = row
                    .try_get("total_points")
                    .map_err(|e| format!("Failed to get total_points: {}", e))?;
                Ok((user_id, total_points))
            })
            .collect::<Result<Vec<_>, String>>()?;

        Ok(leaderboard)
    }
}
