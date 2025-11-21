use crate::application::ports::ResolutionRepository;
use crate::domain::entities::{MajorityType, Resolution, ResolutionStatus, ResolutionType};
use crate::infrastructure::database::pool::DbPool;
use async_trait::async_trait;
use sqlx::Row;
use uuid::Uuid;

pub struct PostgresResolutionRepository {
    pool: DbPool,
}

impl PostgresResolutionRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    /// Parse MajorityType from database string format
    fn parse_majority_type(s: &str) -> MajorityType {
        if s.starts_with("Qualified:") {
            let threshold = s
                .strip_prefix("Qualified:")
                .and_then(|t| t.parse::<f64>().ok())
                .unwrap_or(0.67);
            MajorityType::Qualified(threshold)
        } else if s == "Absolute" {
            MajorityType::Absolute
        } else {
            MajorityType::Simple
        }
    }

    /// Convert MajorityType to database string format
    fn majority_type_to_string(majority: &MajorityType) -> String {
        match majority {
            MajorityType::Simple => "Simple".to_string(),
            MajorityType::Absolute => "Absolute".to_string(),
            MajorityType::Qualified(threshold) => format!("Qualified:{}", threshold),
        }
    }
}

#[async_trait]
impl ResolutionRepository for PostgresResolutionRepository {
    async fn create(&self, resolution: &Resolution) -> Result<Resolution, String> {
        let resolution_type_str = match resolution.resolution_type {
            ResolutionType::Ordinary => "Ordinary",
            ResolutionType::Extraordinary => "Extraordinary",
        };

        let status_str = match resolution.status {
            ResolutionStatus::Pending => "Pending",
            ResolutionStatus::Adopted => "Adopted",
            ResolutionStatus::Rejected => "Rejected",
        };

        let majority_str = Self::majority_type_to_string(&resolution.majority_required);

        sqlx::query(
            r#"
            INSERT INTO resolutions (
                id, meeting_id, title, description, resolution_type, majority_required,
                vote_count_pour, vote_count_contre, vote_count_abstention,
                total_voting_power_pour, total_voting_power_contre, total_voting_power_abstention,
                status, created_at, voted_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
            "#,
        )
        .bind(resolution.id)
        .bind(resolution.meeting_id)
        .bind(&resolution.title)
        .bind(&resolution.description)
        .bind(resolution_type_str)
        .bind(majority_str)
        .bind(resolution.vote_count_pour)
        .bind(resolution.vote_count_contre)
        .bind(resolution.vote_count_abstention)
        .bind(resolution.total_voting_power_pour)
        .bind(resolution.total_voting_power_contre)
        .bind(resolution.total_voting_power_abstention)
        .bind(status_str)
        .bind(resolution.created_at)
        .bind(resolution.voted_at)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Database error creating resolution: {}", e))?;

        Ok(resolution.clone())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Resolution>, String> {
        let row = sqlx::query(
            r#"
            SELECT id, meeting_id, title, description, resolution_type, majority_required,
                   vote_count_pour, vote_count_contre, vote_count_abstention,
                   total_voting_power_pour, total_voting_power_contre, total_voting_power_abstention,
                   status, created_at, voted_at
            FROM resolutions
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Database error finding resolution: {}", e))?;

        Ok(row.map(|row| {
            let resolution_type_str: String = row.get("resolution_type");
            let resolution_type = match resolution_type_str.as_str() {
                "Extraordinary" => ResolutionType::Extraordinary,
                _ => ResolutionType::Ordinary,
            };

            let status_str: String = row.get("status");
            let status = match status_str.as_str() {
                "Adopted" => ResolutionStatus::Adopted,
                "Rejected" => ResolutionStatus::Rejected,
                _ => ResolutionStatus::Pending,
            };

            let majority_str: String = row.get("majority_required");
            let majority_required = Self::parse_majority_type(&majority_str);

            Resolution {
                id: row.get("id"),
                meeting_id: row.get("meeting_id"),
                title: row.get("title"),
                description: row.get("description"),
                resolution_type,
                majority_required,
                vote_count_pour: row.get("vote_count_pour"),
                vote_count_contre: row.get("vote_count_contre"),
                vote_count_abstention: row.get("vote_count_abstention"),
                total_voting_power_pour: row.get("total_voting_power_pour"),
                total_voting_power_contre: row.get("total_voting_power_contre"),
                total_voting_power_abstention: row.get("total_voting_power_abstention"),
                status,
                created_at: row.get("created_at"),
                voted_at: row.get("voted_at"),
            }
        }))
    }

    async fn find_by_meeting_id(&self, meeting_id: Uuid) -> Result<Vec<Resolution>, String> {
        let rows = sqlx::query(
            r#"
            SELECT id, meeting_id, title, description, resolution_type, majority_required,
                   vote_count_pour, vote_count_contre, vote_count_abstention,
                   total_voting_power_pour, total_voting_power_contre, total_voting_power_abstention,
                   status, created_at, voted_at
            FROM resolutions
            WHERE meeting_id = $1
            ORDER BY created_at ASC
            "#,
        )
        .bind(meeting_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Database error finding resolutions by meeting: {}", e))?;

        Ok(rows
            .into_iter()
            .map(|row| {
                let resolution_type_str: String = row.get("resolution_type");
                let resolution_type = match resolution_type_str.as_str() {
                    "Extraordinary" => ResolutionType::Extraordinary,
                    _ => ResolutionType::Ordinary,
                };

                let status_str: String = row.get("status");
                let status = match status_str.as_str() {
                    "Adopted" => ResolutionStatus::Adopted,
                    "Rejected" => ResolutionStatus::Rejected,
                    _ => ResolutionStatus::Pending,
                };

                let majority_str: String = row.get("majority_required");
                let majority_required = Self::parse_majority_type(&majority_str);

                Resolution {
                    id: row.get("id"),
                    meeting_id: row.get("meeting_id"),
                    title: row.get("title"),
                    description: row.get("description"),
                    resolution_type,
                    majority_required,
                    vote_count_pour: row.get("vote_count_pour"),
                    vote_count_contre: row.get("vote_count_contre"),
                    vote_count_abstention: row.get("vote_count_abstention"),
                    total_voting_power_pour: row.get("total_voting_power_pour"),
                    total_voting_power_contre: row.get("total_voting_power_contre"),
                    total_voting_power_abstention: row.get("total_voting_power_abstention"),
                    status,
                    created_at: row.get("created_at"),
                    voted_at: row.get("voted_at"),
                }
            })
            .collect())
    }

    async fn find_by_status(&self, status: ResolutionStatus) -> Result<Vec<Resolution>, String> {
        let status_str = match status {
            ResolutionStatus::Pending => "Pending",
            ResolutionStatus::Adopted => "Adopted",
            ResolutionStatus::Rejected => "Rejected",
        };

        let rows = sqlx::query(
            r#"
            SELECT id, meeting_id, title, description, resolution_type, majority_required,
                   vote_count_pour, vote_count_contre, vote_count_abstention,
                   total_voting_power_pour, total_voting_power_contre, total_voting_power_abstention,
                   status, created_at, voted_at
            FROM resolutions
            WHERE status = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(status_str)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Database error finding resolutions by status: {}", e))?;

        Ok(rows
            .into_iter()
            .map(|row| {
                let resolution_type_str: String = row.get("resolution_type");
                let resolution_type = match resolution_type_str.as_str() {
                    "Extraordinary" => ResolutionType::Extraordinary,
                    _ => ResolutionType::Ordinary,
                };

                let majority_str: String = row.get("majority_required");
                let majority_required = Self::parse_majority_type(&majority_str);

                Resolution {
                    id: row.get("id"),
                    meeting_id: row.get("meeting_id"),
                    title: row.get("title"),
                    description: row.get("description"),
                    resolution_type,
                    majority_required,
                    vote_count_pour: row.get("vote_count_pour"),
                    vote_count_contre: row.get("vote_count_contre"),
                    vote_count_abstention: row.get("vote_count_abstention"),
                    total_voting_power_pour: row.get("total_voting_power_pour"),
                    total_voting_power_contre: row.get("total_voting_power_contre"),
                    total_voting_power_abstention: row.get("total_voting_power_abstention"),
                    status: status.clone(),
                    created_at: row.get("created_at"),
                    voted_at: row.get("voted_at"),
                }
            })
            .collect())
    }

    async fn update(&self, resolution: &Resolution) -> Result<Resolution, String> {
        let resolution_type_str = match resolution.resolution_type {
            ResolutionType::Ordinary => "Ordinary",
            ResolutionType::Extraordinary => "Extraordinary",
        };

        let status_str = match resolution.status {
            ResolutionStatus::Pending => "Pending",
            ResolutionStatus::Adopted => "Adopted",
            ResolutionStatus::Rejected => "Rejected",
        };

        let majority_str = Self::majority_type_to_string(&resolution.majority_required);

        sqlx::query(
            r#"
            UPDATE resolutions
            SET meeting_id = $2, title = $3, description = $4, resolution_type = $5,
                majority_required = $6, vote_count_pour = $7, vote_count_contre = $8,
                vote_count_abstention = $9, total_voting_power_pour = $10,
                total_voting_power_contre = $11, total_voting_power_abstention = $12,
                status = $13, voted_at = $14
            WHERE id = $1
            "#,
        )
        .bind(resolution.id)
        .bind(resolution.meeting_id)
        .bind(&resolution.title)
        .bind(&resolution.description)
        .bind(resolution_type_str)
        .bind(majority_str)
        .bind(resolution.vote_count_pour)
        .bind(resolution.vote_count_contre)
        .bind(resolution.vote_count_abstention)
        .bind(resolution.total_voting_power_pour)
        .bind(resolution.total_voting_power_contre)
        .bind(resolution.total_voting_power_abstention)
        .bind(status_str)
        .bind(resolution.voted_at)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Database error updating resolution: {}", e))?;

        Ok(resolution.clone())
    }

    async fn delete(&self, id: Uuid) -> Result<bool, String> {
        let result = sqlx::query(
            r#"
            DELETE FROM resolutions WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Database error deleting resolution: {}", e))?;

        Ok(result.rows_affected() > 0)
    }

    async fn update_vote_counts(
        &self,
        resolution_id: Uuid,
        vote_count_pour: i32,
        vote_count_contre: i32,
        vote_count_abstention: i32,
        total_voting_power_pour: f64,
        total_voting_power_contre: f64,
        total_voting_power_abstention: f64,
    ) -> Result<(), String> {
        sqlx::query(
            r#"
            UPDATE resolutions
            SET vote_count_pour = $2, vote_count_contre = $3, vote_count_abstention = $4,
                total_voting_power_pour = $5, total_voting_power_contre = $6,
                total_voting_power_abstention = $7
            WHERE id = $1
            "#,
        )
        .bind(resolution_id)
        .bind(vote_count_pour)
        .bind(vote_count_contre)
        .bind(vote_count_abstention)
        .bind(total_voting_power_pour)
        .bind(total_voting_power_contre)
        .bind(total_voting_power_abstention)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Database error updating vote counts: {}", e))?;

        Ok(())
    }

    async fn close_voting(
        &self,
        resolution_id: Uuid,
        final_status: ResolutionStatus,
    ) -> Result<(), String> {
        let status_str = match final_status {
            ResolutionStatus::Pending => "Pending",
            ResolutionStatus::Adopted => "Adopted",
            ResolutionStatus::Rejected => "Rejected",
        };

        sqlx::query(
            r#"
            UPDATE resolutions
            SET status = $2, voted_at = CURRENT_TIMESTAMP
            WHERE id = $1
            "#,
        )
        .bind(resolution_id)
        .bind(status_str)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Database error closing voting: {}", e))?;

        Ok(())
    }

    async fn get_meeting_vote_summary(
        &self,
        meeting_id: Uuid,
    ) -> Result<Vec<Resolution>, String> {
        // Same as find_by_meeting_id, but could be enhanced with additional stats
        self.find_by_meeting_id(meeting_id).await
    }
}
