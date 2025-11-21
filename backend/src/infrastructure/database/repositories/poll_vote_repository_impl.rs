use crate::application::ports::PollVoteRepository;
use crate::domain::entities::PollVote;
use crate::infrastructure::database::pool::DbPool;
use async_trait::async_trait;
use serde_json;
use sqlx::Row;
use uuid::Uuid;

pub struct PostgresPollVoteRepository {
    pool: DbPool,
}

impl PostgresPollVoteRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    /// Map database row to PollVote entity
    fn row_to_poll_vote(&self, row: &sqlx::postgres::PgRow) -> Result<PollVote, String> {
        // Parse selected_option_ids from JSONB
        let selected_ids_json: serde_json::Value = row
            .try_get("selected_option_ids")
            .map_err(|e| format!("Failed to get selected_option_ids: {}", e))?;

        let selected_ids: Vec<Uuid> = serde_json::from_value(selected_ids_json)
            .map_err(|e| format!("Failed to deserialize selected_option_ids: {}", e))?;

        Ok(PollVote {
            id: row
                .try_get("id")
                .map_err(|e| format!("Failed to get id: {}", e))?,
            poll_id: row
                .try_get("poll_id")
                .map_err(|e| format!("Failed to get poll_id: {}", e))?,
            owner_id: row
                .try_get("owner_id")
                .map_err(|e| format!("Failed to get owner_id: {}", e))?,
            building_id: row
                .try_get("building_id")
                .map_err(|e| format!("Failed to get building_id: {}", e))?,
            selected_option_ids: selected_ids,
            rating_value: row
                .try_get("rating_value")
                .map_err(|e| format!("Failed to get rating_value: {}", e))?,
            open_text: row
                .try_get("open_text")
                .map_err(|e| format!("Failed to get open_text: {}", e))?,
            voted_at: row
                .try_get("voted_at")
                .map_err(|e| format!("Failed to get voted_at: {}", e))?,
            ip_address: row
                .try_get("ip_address")
                .map_err(|e| format!("Failed to get ip_address: {}", e))?,
        })
    }
}

#[async_trait]
impl PollVoteRepository for PostgresPollVoteRepository {
    async fn create(&self, vote: &PollVote) -> Result<PollVote, String> {
        // Serialize selected_option_ids to JSONB
        let selected_ids_json = serde_json::to_value(&vote.selected_option_ids)
            .map_err(|e| format!("Failed to serialize selected_option_ids: {}", e))?;

        sqlx::query(
            r#"
            INSERT INTO poll_votes (
                id, poll_id, owner_id, building_id, selected_option_ids,
                rating_value, open_text, voted_at, ip_address
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
        )
        .bind(vote.id)
        .bind(vote.poll_id)
        .bind(vote.owner_id)
        .bind(vote.building_id)
        .bind(selected_ids_json)
        .bind(vote.rating_value)
        .bind(&vote.open_text)
        .bind(vote.voted_at)
        .bind(&vote.ip_address)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to insert poll vote: {}", e))?;

        Ok(vote.clone())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<PollVote>, String> {
        let row = sqlx::query(
            r#"
            SELECT * FROM poll_votes WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Failed to fetch poll vote: {}", e))?;

        match row {
            Some(r) => Ok(Some(self.row_to_poll_vote(&r)?)),
            None => Ok(None),
        }
    }

    async fn find_by_poll(&self, poll_id: Uuid) -> Result<Vec<PollVote>, String> {
        let rows = sqlx::query(
            r#"
            SELECT * FROM poll_votes 
            WHERE poll_id = $1
            ORDER BY voted_at DESC
            "#,
        )
        .bind(poll_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to fetch poll votes: {}", e))?;

        rows.iter()
            .map(|row| self.row_to_poll_vote(row))
            .collect::<Result<Vec<PollVote>, String>>()
    }

    async fn find_by_poll_and_owner(
        &self,
        poll_id: Uuid,
        owner_id: Uuid,
    ) -> Result<Option<PollVote>, String> {
        let row = sqlx::query(
            r#"
            SELECT * FROM poll_votes 
            WHERE poll_id = $1 AND owner_id = $2
            LIMIT 1
            "#,
        )
        .bind(poll_id)
        .bind(owner_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Failed to fetch poll vote: {}", e))?;

        match row {
            Some(r) => Ok(Some(self.row_to_poll_vote(&r)?)),
            None => Ok(None),
        }
    }

    async fn find_by_owner(&self, owner_id: Uuid) -> Result<Vec<PollVote>, String> {
        let rows = sqlx::query(
            r#"
            SELECT * FROM poll_votes 
            WHERE owner_id = $1
            ORDER BY voted_at DESC
            "#,
        )
        .bind(owner_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to fetch poll votes: {}", e))?;

        rows.iter()
            .map(|row| self.row_to_poll_vote(row))
            .collect::<Result<Vec<PollVote>, String>>()
    }

    async fn delete(&self, id: Uuid) -> Result<bool, String> {
        let result = sqlx::query(
            r#"
            DELETE FROM poll_votes WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to delete poll vote: {}", e))?;

        Ok(result.rows_affected() > 0)
    }
}
