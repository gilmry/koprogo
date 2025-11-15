use crate::application::ports::VoteRepository;
use crate::domain::entities::{Vote, VoteChoice};
use crate::infrastructure::database::pool::DbPool;
use async_trait::async_trait;
use sqlx::Row;
use uuid::Uuid;

pub struct PostgresVoteRepository {
    pool: DbPool,
}

impl PostgresVoteRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl VoteRepository for PostgresVoteRepository {
    async fn create(&self, vote: &Vote) -> Result<Vote, String> {
        let vote_choice_str = match vote.vote_choice {
            VoteChoice::Pour => "Pour",
            VoteChoice::Contre => "Contre",
            VoteChoice::Abstention => "Abstention",
        };

        sqlx::query(
            r#"
            INSERT INTO votes (
                id, resolution_id, owner_id, unit_id, vote_choice,
                voting_power, proxy_owner_id, voted_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
        )
        .bind(vote.id)
        .bind(vote.resolution_id)
        .bind(vote.owner_id)
        .bind(vote.unit_id)
        .bind(vote_choice_str)
        .bind(vote.voting_power)
        .bind(vote.proxy_owner_id)
        .bind(vote.voted_at)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Database error creating vote: {}", e))?;

        Ok(vote.clone())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Vote>, String> {
        let row = sqlx::query(
            r#"
            SELECT id, resolution_id, owner_id, unit_id, vote_choice,
                   voting_power, proxy_owner_id, voted_at
            FROM votes
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Database error finding vote: {}", e))?;

        Ok(row.map(|row| {
            let vote_choice_str: String = row.get("vote_choice");
            let vote_choice = match vote_choice_str.as_str() {
                "Contre" => VoteChoice::Contre,
                "Abstention" => VoteChoice::Abstention,
                _ => VoteChoice::Pour,
            };

            Vote {
                id: row.get("id"),
                resolution_id: row.get("resolution_id"),
                owner_id: row.get("owner_id"),
                unit_id: row.get("unit_id"),
                vote_choice,
                voting_power: row.get("voting_power"),
                proxy_owner_id: row.get("proxy_owner_id"),
                voted_at: row.get("voted_at"),
            }
        }))
    }

    async fn find_by_resolution_id(&self, resolution_id: Uuid) -> Result<Vec<Vote>, String> {
        let rows = sqlx::query(
            r#"
            SELECT id, resolution_id, owner_id, unit_id, vote_choice,
                   voting_power, proxy_owner_id, voted_at
            FROM votes
            WHERE resolution_id = $1
            ORDER BY voted_at ASC
            "#,
        )
        .bind(resolution_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Database error finding votes by resolution: {}", e))?;

        Ok(rows
            .into_iter()
            .map(|row| {
                let vote_choice_str: String = row.get("vote_choice");
                let vote_choice = match vote_choice_str.as_str() {
                    "Contre" => VoteChoice::Contre,
                    "Abstention" => VoteChoice::Abstention,
                    _ => VoteChoice::Pour,
                };

                Vote {
                    id: row.get("id"),
                    resolution_id: row.get("resolution_id"),
                    owner_id: row.get("owner_id"),
                    unit_id: row.get("unit_id"),
                    vote_choice,
                    voting_power: row.get("voting_power"),
                    proxy_owner_id: row.get("proxy_owner_id"),
                    voted_at: row.get("voted_at"),
                }
            })
            .collect())
    }

    async fn find_by_owner_id(&self, owner_id: Uuid) -> Result<Vec<Vote>, String> {
        let rows = sqlx::query(
            r#"
            SELECT id, resolution_id, owner_id, unit_id, vote_choice,
                   voting_power, proxy_owner_id, voted_at
            FROM votes
            WHERE owner_id = $1
            ORDER BY voted_at DESC
            "#,
        )
        .bind(owner_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Database error finding votes by owner: {}", e))?;

        Ok(rows
            .into_iter()
            .map(|row| {
                let vote_choice_str: String = row.get("vote_choice");
                let vote_choice = match vote_choice_str.as_str() {
                    "Contre" => VoteChoice::Contre,
                    "Abstention" => VoteChoice::Abstention,
                    _ => VoteChoice::Pour,
                };

                Vote {
                    id: row.get("id"),
                    resolution_id: row.get("resolution_id"),
                    owner_id: row.get("owner_id"),
                    unit_id: row.get("unit_id"),
                    vote_choice,
                    voting_power: row.get("voting_power"),
                    proxy_owner_id: row.get("proxy_owner_id"),
                    voted_at: row.get("voted_at"),
                }
            })
            .collect())
    }

    async fn find_by_resolution_and_unit(
        &self,
        resolution_id: Uuid,
        unit_id: Uuid,
    ) -> Result<Option<Vote>, String> {
        let row = sqlx::query(
            r#"
            SELECT id, resolution_id, owner_id, unit_id, vote_choice,
                   voting_power, proxy_owner_id, voted_at
            FROM votes
            WHERE resolution_id = $1 AND unit_id = $2
            "#,
        )
        .bind(resolution_id)
        .bind(unit_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Database error finding vote by resolution and unit: {}", e))?;

        Ok(row.map(|row| {
            let vote_choice_str: String = row.get("vote_choice");
            let vote_choice = match vote_choice_str.as_str() {
                "Contre" => VoteChoice::Contre,
                "Abstention" => VoteChoice::Abstention,
                _ => VoteChoice::Pour,
            };

            Vote {
                id: row.get("id"),
                resolution_id: row.get("resolution_id"),
                owner_id: row.get("owner_id"),
                unit_id: row.get("unit_id"),
                vote_choice,
                voting_power: row.get("voting_power"),
                proxy_owner_id: row.get("proxy_owner_id"),
                voted_at: row.get("voted_at"),
            }
        }))
    }

    async fn has_voted(&self, resolution_id: Uuid, unit_id: Uuid) -> Result<bool, String> {
        let row = sqlx::query(
            r#"
            SELECT EXISTS(SELECT 1 FROM votes WHERE resolution_id = $1 AND unit_id = $2) AS has_voted
            "#,
        )
        .bind(resolution_id)
        .bind(unit_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Database error checking if voted: {}", e))?;

        Ok(row.get("has_voted"))
    }

    async fn update(&self, vote: &Vote) -> Result<Vote, String> {
        let vote_choice_str = match vote.vote_choice {
            VoteChoice::Pour => "Pour",
            VoteChoice::Contre => "Contre",
            VoteChoice::Abstention => "Abstention",
        };

        sqlx::query(
            r#"
            UPDATE votes
            SET resolution_id = $2, owner_id = $3, unit_id = $4, vote_choice = $5,
                voting_power = $6, proxy_owner_id = $7, voted_at = $8
            WHERE id = $1
            "#,
        )
        .bind(vote.id)
        .bind(vote.resolution_id)
        .bind(vote.owner_id)
        .bind(vote.unit_id)
        .bind(vote_choice_str)
        .bind(vote.voting_power)
        .bind(vote.proxy_owner_id)
        .bind(vote.voted_at)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Database error updating vote: {}", e))?;

        Ok(vote.clone())
    }

    async fn delete(&self, id: Uuid) -> Result<bool, String> {
        let result = sqlx::query(
            r#"
            DELETE FROM votes WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Database error deleting vote: {}", e))?;

        Ok(result.rows_affected() > 0)
    }

    async fn count_by_resolution_and_choice(
        &self,
        resolution_id: Uuid,
    ) -> Result<(i32, i32, i32), String> {
        let row = sqlx::query(
            r#"
            SELECT
                COUNT(*) FILTER (WHERE vote_choice = 'Pour') AS pour_count,
                COUNT(*) FILTER (WHERE vote_choice = 'Contre') AS contre_count,
                COUNT(*) FILTER (WHERE vote_choice = 'Abstention') AS abstention_count
            FROM votes
            WHERE resolution_id = $1
            "#,
        )
        .bind(resolution_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Database error counting votes: {}", e))?;

        let pour_count: Option<i64> = row.get("pour_count");
        let contre_count: Option<i64> = row.get("contre_count");
        let abstention_count: Option<i64> = row.get("abstention_count");

        Ok((
            pour_count.unwrap_or(0) as i32,
            contre_count.unwrap_or(0) as i32,
            abstention_count.unwrap_or(0) as i32,
        ))
    }

    async fn sum_voting_power_by_resolution(
        &self,
        resolution_id: Uuid,
    ) -> Result<(f64, f64, f64), String> {
        let row = sqlx::query(
            r#"
            SELECT
                COALESCE(SUM(voting_power) FILTER (WHERE vote_choice = 'Pour'), 0) AS pour_power,
                COALESCE(SUM(voting_power) FILTER (WHERE vote_choice = 'Contre'), 0) AS contre_power,
                COALESCE(SUM(voting_power) FILTER (WHERE vote_choice = 'Abstention'), 0) AS abstention_power
            FROM votes
            WHERE resolution_id = $1
            "#,
        )
        .bind(resolution_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Database error summing voting power: {}", e))?;

        // Get voting power sums as f64 (sqlx handles DECIMAL conversion)
        Ok((
            row.get::<f64, _>("pour_power"),
            row.get::<f64, _>("contre_power"),
            row.get::<f64, _>("abstention_power"),
        ))
    }
}
