use crate::application::ports::NoticeRepository;
use crate::domain::entities::{Notice, NoticeCategory, NoticeStatus, NoticeType};
use crate::infrastructure::database::DbPool;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::Row;
use uuid::Uuid;

pub struct PostgresNoticeRepository {
    pool: DbPool,
}

impl PostgresNoticeRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

/// Helper function to map database row to Notice entity
fn map_row_to_notice(row: &sqlx::postgres::PgRow) -> Notice {
    let notice_type_str: String = row.get("notice_type");
    let category_str: String = row.get("category");
    let status_str: String = row.get("status");

    Notice {
        id: row.get("id"),
        building_id: row.get("building_id"),
        author_id: row.get("author_id"),
        notice_type: serde_json::from_str(&format!("\"{}\"", notice_type_str))
            .unwrap_or(NoticeType::Announcement),
        category: serde_json::from_str(&format!("\"{}\"", category_str))
            .unwrap_or(NoticeCategory::General),
        title: row.get("title"),
        content: row.get("content"),
        status: serde_json::from_str(&format!("\"{}\"", status_str))
            .unwrap_or(NoticeStatus::Draft),
        is_pinned: row.get("is_pinned"),
        published_at: row.get("published_at"),
        expires_at: row.get("expires_at"),
        archived_at: row.get("archived_at"),
        event_date: row.get("event_date"),
        event_location: row.get("event_location"),
        contact_info: row.get("contact_info"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

#[async_trait]
impl NoticeRepository for PostgresNoticeRepository {
    async fn create(&self, notice: &Notice) -> Result<Notice, String> {
        let notice_type_str = serde_json::to_string(&notice.notice_type)
            .map_err(|e| format!("Failed to serialize notice_type: {}", e))?
            .trim_matches('"')
            .to_string();

        let category_str = serde_json::to_string(&notice.category)
            .map_err(|e| format!("Failed to serialize category: {}", e))?
            .trim_matches('"')
            .to_string();

        let status_str = serde_json::to_string(&notice.status)
            .map_err(|e| format!("Failed to serialize status: {}", e))?
            .trim_matches('"')
            .to_string();

        sqlx::query(
            r#"
            INSERT INTO notices (
                id, building_id, author_id, notice_type, category, title, content,
                status, is_pinned, published_at, expires_at, archived_at,
                event_date, event_location, contact_info, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4::notice_type, $5::notice_category, $6, $7, $8::notice_status,
                    $9, $10, $11, $12, $13, $14, $15, $16, $17)
            "#,
        )
        .bind(notice.id)
        .bind(notice.building_id)
        .bind(notice.author_id)
        .bind(&notice_type_str)
        .bind(&category_str)
        .bind(&notice.title)
        .bind(&notice.content)
        .bind(&status_str)
        .bind(notice.is_pinned)
        .bind(notice.published_at)
        .bind(notice.expires_at)
        .bind(notice.archived_at)
        .bind(notice.event_date)
        .bind(&notice.event_location)
        .bind(&notice.contact_info)
        .bind(notice.created_at)
        .bind(notice.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to create notice: {}", e))?;

        Ok(notice.clone())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Notice>, String> {
        let row = sqlx::query(
            r#"
            SELECT id, building_id, author_id, notice_type::text AS notice_type,
                   category::text AS category, title, content, status::text AS status,
                   is_pinned, published_at, expires_at, archived_at,
                   event_date, event_location, contact_info, created_at, updated_at
            FROM notices
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Failed to find notice by ID: {}", e))?;

        Ok(row.as_ref().map(map_row_to_notice))
    }

    async fn find_by_building(&self, building_id: Uuid) -> Result<Vec<Notice>, String> {
        let rows = sqlx::query(
            r#"
            SELECT id, building_id, author_id, notice_type::text AS notice_type,
                   category::text AS category, title, content, status::text AS status,
                   is_pinned, published_at, expires_at, archived_at,
                   event_date, event_location, contact_info, created_at, updated_at
            FROM notices
            WHERE building_id = $1
            ORDER BY
                is_pinned DESC,
                created_at DESC
            "#,
        )
        .bind(building_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to find notices by building: {}", e))?;

        Ok(rows.iter().map(map_row_to_notice).collect())
    }

    async fn find_published_by_building(
        &self,
        building_id: Uuid,
    ) -> Result<Vec<Notice>, String> {
        let rows = sqlx::query(
            r#"
            SELECT id, building_id, author_id, notice_type::text AS notice_type,
                   category::text AS category, title, content, status::text AS status,
                   is_pinned, published_at, expires_at, archived_at,
                   event_date, event_location, contact_info, created_at, updated_at
            FROM notices
            WHERE building_id = $1 AND status = 'Published'
            ORDER BY
                is_pinned DESC,
                published_at DESC
            "#,
        )
        .bind(building_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to find published notices: {}", e))?;

        Ok(rows.iter().map(map_row_to_notice).collect())
    }

    async fn find_pinned_by_building(&self, building_id: Uuid) -> Result<Vec<Notice>, String> {
        let rows = sqlx::query(
            r#"
            SELECT id, building_id, author_id, notice_type::text AS notice_type,
                   category::text AS category, title, content, status::text AS status,
                   is_pinned, published_at, expires_at, archived_at,
                   event_date, event_location, contact_info, created_at, updated_at
            FROM notices
            WHERE building_id = $1 AND is_pinned = true
            ORDER BY published_at DESC
            "#,
        )
        .bind(building_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to find pinned notices: {}", e))?;

        Ok(rows.iter().map(map_row_to_notice).collect())
    }

    async fn find_by_type(
        &self,
        building_id: Uuid,
        notice_type: NoticeType,
    ) -> Result<Vec<Notice>, String> {
        let notice_type_str = serde_json::to_string(&notice_type)
            .map_err(|e| format!("Failed to serialize notice_type: {}", e))?
            .trim_matches('"')
            .to_string();

        let rows = sqlx::query(
            r#"
            SELECT id, building_id, author_id, notice_type::text AS notice_type,
                   category::text AS category, title, content, status::text AS status,
                   is_pinned, published_at, expires_at, archived_at,
                   event_date, event_location, contact_info, created_at, updated_at
            FROM notices
            WHERE building_id = $1 AND notice_type = $2::notice_type
            ORDER BY
                is_pinned DESC,
                created_at DESC
            "#,
        )
        .bind(building_id)
        .bind(&notice_type_str)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to find notices by type: {}", e))?;

        Ok(rows.iter().map(map_row_to_notice).collect())
    }

    async fn find_by_category(
        &self,
        building_id: Uuid,
        category: NoticeCategory,
    ) -> Result<Vec<Notice>, String> {
        let category_str = serde_json::to_string(&category)
            .map_err(|e| format!("Failed to serialize category: {}", e))?
            .trim_matches('"')
            .to_string();

        let rows = sqlx::query(
            r#"
            SELECT id, building_id, author_id, notice_type::text AS notice_type,
                   category::text AS category, title, content, status::text AS status,
                   is_pinned, published_at, expires_at, archived_at,
                   event_date, event_location, contact_info, created_at, updated_at
            FROM notices
            WHERE building_id = $1 AND category = $2::notice_category
            ORDER BY
                is_pinned DESC,
                created_at DESC
            "#,
        )
        .bind(building_id)
        .bind(&category_str)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to find notices by category: {}", e))?;

        Ok(rows.iter().map(map_row_to_notice).collect())
    }

    async fn find_by_status(
        &self,
        building_id: Uuid,
        status: NoticeStatus,
    ) -> Result<Vec<Notice>, String> {
        let status_str = serde_json::to_string(&status)
            .map_err(|e| format!("Failed to serialize status: {}", e))?
            .trim_matches('"')
            .to_string();

        let rows = sqlx::query(
            r#"
            SELECT id, building_id, author_id, notice_type::text AS notice_type,
                   category::text AS category, title, content, status::text AS status,
                   is_pinned, published_at, expires_at, archived_at,
                   event_date, event_location, contact_info, created_at, updated_at
            FROM notices
            WHERE building_id = $1 AND status = $2::notice_status
            ORDER BY
                is_pinned DESC,
                created_at DESC
            "#,
        )
        .bind(building_id)
        .bind(&status_str)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to find notices by status: {}", e))?;

        Ok(rows.iter().map(map_row_to_notice).collect())
    }

    async fn find_by_author(&self, author_id: Uuid) -> Result<Vec<Notice>, String> {
        let rows = sqlx::query(
            r#"
            SELECT id, building_id, author_id, notice_type::text AS notice_type,
                   category::text AS category, title, content, status::text AS status,
                   is_pinned, published_at, expires_at, archived_at,
                   event_date, event_location, contact_info, created_at, updated_at
            FROM notices
            WHERE author_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(author_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to find notices by author: {}", e))?;

        Ok(rows.iter().map(map_row_to_notice).collect())
    }

    async fn find_expired(&self, building_id: Uuid) -> Result<Vec<Notice>, String> {
        let now = Utc::now();

        let rows = sqlx::query(
            r#"
            SELECT id, building_id, author_id, notice_type::text AS notice_type,
                   category::text AS category, title, content, status::text AS status,
                   is_pinned, published_at, expires_at, archived_at,
                   event_date, event_location, contact_info, created_at, updated_at
            FROM notices
            WHERE building_id = $1
              AND status = 'Published'
              AND expires_at IS NOT NULL
              AND expires_at < $2
            ORDER BY expires_at ASC
            "#,
        )
        .bind(building_id)
        .bind(now)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to find expired notices: {}", e))?;

        Ok(rows.iter().map(map_row_to_notice).collect())
    }

    async fn update(&self, notice: &Notice) -> Result<Notice, String> {
        let notice_type_str = serde_json::to_string(&notice.notice_type)
            .map_err(|e| format!("Failed to serialize notice_type: {}", e))?
            .trim_matches('"')
            .to_string();

        let category_str = serde_json::to_string(&notice.category)
            .map_err(|e| format!("Failed to serialize category: {}", e))?
            .trim_matches('"')
            .to_string();

        let status_str = serde_json::to_string(&notice.status)
            .map_err(|e| format!("Failed to serialize status: {}", e))?
            .trim_matches('"')
            .to_string();

        let result = sqlx::query(
            r#"
            UPDATE notices
            SET building_id = $2,
                author_id = $3,
                notice_type = $4::notice_type,
                category = $5::notice_category,
                title = $6,
                content = $7,
                status = $8::notice_status,
                is_pinned = $9,
                published_at = $10,
                expires_at = $11,
                archived_at = $12,
                event_date = $13,
                event_location = $14,
                contact_info = $15,
                updated_at = $16
            WHERE id = $1
            "#,
        )
        .bind(notice.id)
        .bind(notice.building_id)
        .bind(notice.author_id)
        .bind(&notice_type_str)
        .bind(&category_str)
        .bind(&notice.title)
        .bind(&notice.content)
        .bind(&status_str)
        .bind(notice.is_pinned)
        .bind(notice.published_at)
        .bind(notice.expires_at)
        .bind(notice.archived_at)
        .bind(notice.event_date)
        .bind(&notice.event_location)
        .bind(&notice.contact_info)
        .bind(notice.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to update notice: {}", e))?;

        if result.rows_affected() == 0 {
            return Err("Notice not found".to_string());
        }

        Ok(notice.clone())
    }

    async fn delete(&self, id: Uuid) -> Result<(), String> {
        let result = sqlx::query(
            r#"
            DELETE FROM notices
            WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to delete notice: {}", e))?;

        if result.rows_affected() == 0 {
            return Err("Notice not found".to_string());
        }

        Ok(())
    }

    async fn count_by_building(&self, building_id: Uuid) -> Result<i64, String> {
        let row = sqlx::query(
            r#"
            SELECT COUNT(*) as count
            FROM notices
            WHERE building_id = $1
            "#,
        )
        .bind(building_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to count notices by building: {}", e))?;

        let count: i64 = row.get("count");
        Ok(count)
    }

    async fn count_published_by_building(&self, building_id: Uuid) -> Result<i64, String> {
        let row = sqlx::query(
            r#"
            SELECT COUNT(*) as count
            FROM notices
            WHERE building_id = $1 AND status = 'Published'
            "#,
        )
        .bind(building_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to count published notices: {}", e))?;

        let count: i64 = row.get("count");
        Ok(count)
    }

    async fn count_pinned_by_building(&self, building_id: Uuid) -> Result<i64, String> {
        let row = sqlx::query(
            r#"
            SELECT COUNT(*) as count
            FROM notices
            WHERE building_id = $1 AND is_pinned = true
            "#,
        )
        .bind(building_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to count pinned notices: {}", e))?;

        let count: i64 = row.get("count");
        Ok(count)
    }
}
