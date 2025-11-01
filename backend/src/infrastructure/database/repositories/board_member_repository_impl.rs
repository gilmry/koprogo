use crate::application::ports::BoardMemberRepository;
use crate::domain::entities::{BoardMember, BoardPosition};
use crate::infrastructure::database::pool::DbPool;
use async_trait::async_trait;
use sqlx::Row;
use uuid::Uuid;

pub struct PostgresBoardMemberRepository {
    pool: DbPool,
}

impl PostgresBoardMemberRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl BoardMemberRepository for PostgresBoardMemberRepository {
    async fn create(&self, board_member: &BoardMember) -> Result<BoardMember, String> {
        let position_str = match board_member.position {
            BoardPosition::President => "president",
            BoardPosition::Treasurer => "treasurer",
            BoardPosition::Member => "member",
        };

        // Get organization_id from building
        let organization_id: Uuid =
            sqlx::query_scalar("SELECT organization_id FROM buildings WHERE id = $1")
                .bind(board_member.building_id)
                .fetch_one(&self.pool)
                .await
                .map_err(|e| format!("Failed to get building organization: {}", e))?;

        sqlx::query(
            r#"
            INSERT INTO board_members (id, owner_id, building_id, organization_id, position, mandate_start, mandate_end, elected_by_meeting_id, is_active, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5::board_position, $6, $7, $8, $9, $10, $11)
            "#,
        )
        .bind(board_member.id)
        .bind(board_member.owner_id)
        .bind(board_member.building_id)
        .bind(organization_id)
        .bind(position_str)
        .bind(board_member.mandate_start)
        .bind(board_member.mandate_end)
        .bind(board_member.elected_by_meeting_id)
        .bind(true) // is_active - new mandate is always active
        .bind(board_member.created_at)
        .bind(board_member.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(board_member.clone())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<BoardMember>, String> {
        let row = sqlx::query(
            r#"
            SELECT id, owner_id, building_id, position::TEXT as position, mandate_start, mandate_end, elected_by_meeting_id, created_at, updated_at
            FROM board_members
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(row.map(|row| {
            let position_str: String = row.get("position");
            let position = match position_str.as_str() {
                "president" => BoardPosition::President,
                "treasurer" => BoardPosition::Treasurer,
                _ => BoardPosition::Member,
            };

            BoardMember {
                id: row.get("id"),
                owner_id: row.get("owner_id"),
                building_id: row.get("building_id"),
                position,
                mandate_start: row.get("mandate_start"),
                mandate_end: row.get("mandate_end"),
                elected_by_meeting_id: row.get("elected_by_meeting_id"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            }
        }))
    }

    async fn find_by_building(&self, building_id: Uuid) -> Result<Vec<BoardMember>, String> {
        let rows = sqlx::query(
            r#"
            SELECT id, owner_id, building_id, position::TEXT as position, mandate_start, mandate_end, elected_by_meeting_id, created_at, updated_at
            FROM board_members
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
                let position_str: String = row.get("position");
                let position = match position_str.as_str() {
                    "president" => BoardPosition::President,
                    "treasurer" => BoardPosition::Treasurer,
                    _ => BoardPosition::Member,
                };

                BoardMember {
                    id: row.get("id"),
                    owner_id: row.get("owner_id"),
                    building_id: row.get("building_id"),
                    position,
                    mandate_start: row.get("mandate_start"),
                    mandate_end: row.get("mandate_end"),
                    elected_by_meeting_id: row.get("elected_by_meeting_id"),
                    created_at: row.get("created_at"),
                    updated_at: row.get("updated_at"),
                }
            })
            .collect())
    }

    async fn find_active_by_building(&self, building_id: Uuid) -> Result<Vec<BoardMember>, String> {
        let rows = sqlx::query(
            r#"
            SELECT id, owner_id, building_id, position::TEXT as position, mandate_start, mandate_end, elected_by_meeting_id, created_at, updated_at
            FROM board_members
            WHERE building_id = $1 AND is_active = true AND mandate_end > CURRENT_TIMESTAMP
            ORDER BY mandate_start DESC
            "#,
        )
        .bind(building_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(rows
            .into_iter()
            .map(|row| {
                let position_str: String = row.get("position");
                let position = match position_str.as_str() {
                    "president" => BoardPosition::President,
                    "treasurer" => BoardPosition::Treasurer,
                    _ => BoardPosition::Member,
                };

                BoardMember {
                    id: row.get("id"),
                    owner_id: row.get("owner_id"),
                    building_id: row.get("building_id"),
                    position,
                    mandate_start: row.get("mandate_start"),
                    mandate_end: row.get("mandate_end"),
                    elected_by_meeting_id: row.get("elected_by_meeting_id"),
                    created_at: row.get("created_at"),
                    updated_at: row.get("updated_at"),
                }
            })
            .collect())
    }

    async fn find_expiring_soon(
        &self,
        building_id: Uuid,
        days_threshold: i32,
    ) -> Result<Vec<BoardMember>, String> {
        let rows = sqlx::query(
            r#"
            SELECT id, owner_id, building_id, position::TEXT as position, mandate_start, mandate_end, elected_by_meeting_id, created_at, updated_at
            FROM board_members
            WHERE building_id = $1
              AND mandate_end > CURRENT_TIMESTAMP
              AND mandate_end <= (CURRENT_TIMESTAMP + INTERVAL '1 day' * $2)
            ORDER BY mandate_end ASC
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
                let position_str: String = row.get("position");
                let position = match position_str.as_str() {
                    "president" => BoardPosition::President,
                    "treasurer" => BoardPosition::Treasurer,
                    _ => BoardPosition::Member,
                };

                BoardMember {
                    id: row.get("id"),
                    owner_id: row.get("owner_id"),
                    building_id: row.get("building_id"),
                    position,
                    mandate_start: row.get("mandate_start"),
                    mandate_end: row.get("mandate_end"),
                    elected_by_meeting_id: row.get("elected_by_meeting_id"),
                    created_at: row.get("created_at"),
                    updated_at: row.get("updated_at"),
                }
            })
            .collect())
    }

    async fn find_by_owner(&self, owner_id: Uuid) -> Result<Vec<BoardMember>, String> {
        let rows = sqlx::query(
            r#"
            SELECT id, owner_id, building_id, position::TEXT as position, mandate_start, mandate_end, elected_by_meeting_id, created_at, updated_at
            FROM board_members
            WHERE owner_id = $1
            ORDER BY mandate_start DESC
            "#,
        )
        .bind(owner_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(rows
            .into_iter()
            .map(|row| {
                let position_str: String = row.get("position");
                let position = match position_str.as_str() {
                    "president" => BoardPosition::President,
                    "treasurer" => BoardPosition::Treasurer,
                    _ => BoardPosition::Member,
                };

                BoardMember {
                    id: row.get("id"),
                    owner_id: row.get("owner_id"),
                    building_id: row.get("building_id"),
                    position,
                    mandate_start: row.get("mandate_start"),
                    mandate_end: row.get("mandate_end"),
                    elected_by_meeting_id: row.get("elected_by_meeting_id"),
                    created_at: row.get("created_at"),
                    updated_at: row.get("updated_at"),
                }
            })
            .collect())
    }

    async fn find_by_owner_and_building(
        &self,
        owner_id: Uuid,
        building_id: Uuid,
    ) -> Result<Option<BoardMember>, String> {
        let row = sqlx::query(
            r#"
            SELECT id, owner_id, building_id, position::TEXT as position, mandate_start, mandate_end, elected_by_meeting_id, created_at, updated_at
            FROM board_members
            WHERE owner_id = $1
              AND building_id = $2
            ORDER BY mandate_end DESC
            LIMIT 1
            "#,
        )
        .bind(owner_id)
        .bind(building_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(row.map(|row| {
            let position_str: String = row.get("position");
            let position = match position_str.as_str() {
                "president" => BoardPosition::President,
                "treasurer" => BoardPosition::Treasurer,
                _ => BoardPosition::Member,
            };

            BoardMember {
                id: row.get("id"),
                owner_id: row.get("owner_id"),
                building_id: row.get("building_id"),
                position,
                mandate_start: row.get("mandate_start"),
                mandate_end: row.get("mandate_end"),
                elected_by_meeting_id: row.get("elected_by_meeting_id"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            }
        }))
    }

    async fn has_active_mandate(&self, owner_id: Uuid, building_id: Uuid) -> Result<bool, String> {
        let count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*)
            FROM board_members
            WHERE owner_id = $1
              AND building_id = $2
              AND mandate_end > CURRENT_TIMESTAMP
            "#,
        )
        .bind(owner_id)
        .bind(building_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(count > 0)
    }

    async fn update(&self, board_member: &BoardMember) -> Result<BoardMember, String> {
        let position_str = match board_member.position {
            BoardPosition::President => "president",
            BoardPosition::Treasurer => "treasurer",
            BoardPosition::Member => "member",
        };

        sqlx::query(
            r#"
            UPDATE board_members
            SET position = $1,
                mandate_start = $2,
                mandate_end = $3,
                elected_by_meeting_id = $4,
                updated_at = $5
            WHERE id = $6
            "#,
        )
        .bind(position_str)
        .bind(board_member.mandate_start)
        .bind(board_member.mandate_end)
        .bind(board_member.elected_by_meeting_id)
        .bind(board_member.updated_at)
        .bind(board_member.id)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(board_member.clone())
    }

    async fn delete(&self, id: Uuid) -> Result<bool, String> {
        let result = sqlx::query(
            r#"
            DELETE FROM board_members
            WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(result.rows_affected() > 0)
    }

    async fn count_active_by_building(&self, building_id: Uuid) -> Result<i64, String> {
        let count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*)
            FROM board_members
            WHERE building_id = $1 AND mandate_end > CURRENT_TIMESTAMP
            "#,
        )
        .bind(building_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(count)
    }
}
