use crate::application::ports::SharedObjectRepository;
use crate::domain::entities::{ObjectCondition, SharedObject, SharedObjectCategory};
use crate::infrastructure::pool::DbPool;
use async_trait::async_trait;
use sqlx::Row;
use uuid::Uuid;

pub struct PostgresSharedObjectRepository {
    pool: DbPool,
}

impl PostgresSharedObjectRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

/// Helper function to map database row to SharedObject entity
fn map_row_to_shared_object(row: &sqlx::postgres::PgRow) -> SharedObject {
    let category_str: String = row.get("object_category");
    let condition_str: String = row.get("condition");

    SharedObject {
        id: row.get("id"),
        owner_id: row.get("owner_id"),
        building_id: row.get("building_id"),
        object_category: serde_json::from_str(&format!("\"{}\"", category_str))
            .unwrap_or(SharedObjectCategory::Other),
        object_name: row.get("object_name"),
        description: row.get("description"),
        condition: serde_json::from_str(&format!("\"{}\"", condition_str))
            .unwrap_or(ObjectCondition::Good),
        is_available: row.get("is_available"),
        rental_credits_per_day: row.get("rental_credits_per_day"),
        deposit_credits: row.get("deposit_credits"),
        borrowing_duration_days: row.get("borrowing_duration_days"),
        current_borrower_id: row.get("current_borrower_id"),
        borrowed_at: row.get("borrowed_at"),
        due_back_at: row.get("due_back_at"),
        photos: row.get("photos"),
        location_details: row.get("location_details"),
        usage_instructions: row.get("usage_instructions"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

#[async_trait]
impl SharedObjectRepository for PostgresSharedObjectRepository {
    async fn create(&self, object: &SharedObject) -> Result<SharedObject, String> {
        let category_str = serde_json::to_string(&object.object_category)
            .map_err(|e| format!("Failed to serialize object_category: {}", e))?
            .trim_matches('"')
            .to_string();

        let condition_str = serde_json::to_string(&object.condition)
            .map_err(|e| format!("Failed to serialize condition: {}", e))?
            .trim_matches('"')
            .to_string();

        sqlx::query(
            r#"
            INSERT INTO shared_objects (
                id, owner_id, building_id, object_category, object_name, description,
                condition, is_available, rental_credits_per_day, deposit_credits,
                borrowing_duration_days, current_borrower_id, borrowed_at, due_back_at,
                photos, location_details, usage_instructions, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4::shared_object_category, $5, $6, $7::object_condition,
                    $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19)
            "#,
        )
        .bind(object.id)
        .bind(object.owner_id)
        .bind(object.building_id)
        .bind(&category_str)
        .bind(&object.object_name)
        .bind(&object.description)
        .bind(&condition_str)
        .bind(object.is_available)
        .bind(object.rental_credits_per_day)
        .bind(object.deposit_credits)
        .bind(object.borrowing_duration_days)
        .bind(object.current_borrower_id)
        .bind(object.borrowed_at)
        .bind(object.due_back_at)
        .bind(&object.photos)
        .bind(&object.location_details)
        .bind(&object.usage_instructions)
        .bind(object.created_at)
        .bind(object.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to create shared object: {}", e))?;

        Ok(object.clone())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<SharedObject>, String> {
        let row = sqlx::query(
            r#"
            SELECT id, owner_id, building_id, object_category::text AS object_category,
                   object_name, description, condition::text AS condition, is_available,
                   rental_credits_per_day, deposit_credits, borrowing_duration_days,
                   current_borrower_id, borrowed_at, due_back_at, photos, location_details,
                   usage_instructions, created_at, updated_at
            FROM shared_objects
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Failed to find shared object by ID: {}", e))?;

        Ok(row.as_ref().map(map_row_to_shared_object))
    }

    async fn find_by_building(&self, building_id: Uuid) -> Result<Vec<SharedObject>, String> {
        let rows = sqlx::query(
            r#"
            SELECT id, owner_id, building_id, object_category::text AS object_category,
                   object_name, description, condition::text AS condition, is_available,
                   rental_credits_per_day, deposit_credits, borrowing_duration_days,
                   current_borrower_id, borrowed_at, due_back_at, photos, location_details,
                   usage_instructions, created_at, updated_at
            FROM shared_objects
            WHERE building_id = $1
            ORDER BY
                is_available DESC,
                object_name ASC
            "#,
        )
        .bind(building_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to find shared objects by building: {}", e))?;

        Ok(rows.iter().map(map_row_to_shared_object).collect())
    }

    async fn find_available_by_building(
        &self,
        building_id: Uuid,
    ) -> Result<Vec<SharedObject>, String> {
        let rows = sqlx::query(
            r#"
            SELECT id, owner_id, building_id, object_category::text AS object_category,
                   object_name, description, condition::text AS condition, is_available,
                   rental_credits_per_day, deposit_credits, borrowing_duration_days,
                   current_borrower_id, borrowed_at, due_back_at, photos, location_details,
                   usage_instructions, created_at, updated_at
            FROM shared_objects
            WHERE building_id = $1 AND is_available = TRUE
            ORDER BY object_name ASC
            "#,
        )
        .bind(building_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to find available shared objects by building: {}", e))?;

        Ok(rows.iter().map(map_row_to_shared_object).collect())
    }

    async fn find_borrowed_by_building(
        &self,
        building_id: Uuid,
    ) -> Result<Vec<SharedObject>, String> {
        let rows = sqlx::query(
            r#"
            SELECT id, owner_id, building_id, object_category::text AS object_category,
                   object_name, description, condition::text AS condition, is_available,
                   rental_credits_per_day, deposit_credits, borrowing_duration_days,
                   current_borrower_id, borrowed_at, due_back_at, photos, location_details,
                   usage_instructions, created_at, updated_at
            FROM shared_objects
            WHERE building_id = $1 AND current_borrower_id IS NOT NULL
            ORDER BY due_back_at ASC
            "#,
        )
        .bind(building_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to find borrowed shared objects by building: {}", e))?;

        Ok(rows.iter().map(map_row_to_shared_object).collect())
    }

    async fn find_overdue_by_building(
        &self,
        building_id: Uuid,
    ) -> Result<Vec<SharedObject>, String> {
        let rows = sqlx::query(
            r#"
            SELECT id, owner_id, building_id, object_category::text AS object_category,
                   object_name, description, condition::text AS condition, is_available,
                   rental_credits_per_day, deposit_credits, borrowing_duration_days,
                   current_borrower_id, borrowed_at, due_back_at, photos, location_details,
                   usage_instructions, created_at, updated_at
            FROM shared_objects
            WHERE building_id = $1
              AND current_borrower_id IS NOT NULL
              AND due_back_at < NOW()
            ORDER BY due_back_at ASC
            "#,
        )
        .bind(building_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to find overdue shared objects by building: {}", e))?;

        Ok(rows.iter().map(map_row_to_shared_object).collect())
    }

    async fn find_by_owner(&self, owner_id: Uuid) -> Result<Vec<SharedObject>, String> {
        let rows = sqlx::query(
            r#"
            SELECT id, owner_id, building_id, object_category::text AS object_category,
                   object_name, description, condition::text AS condition, is_available,
                   rental_credits_per_day, deposit_credits, borrowing_duration_days,
                   current_borrower_id, borrowed_at, due_back_at, photos, location_details,
                   usage_instructions, created_at, updated_at
            FROM shared_objects
            WHERE owner_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(owner_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to find shared objects by owner: {}", e))?;

        Ok(rows.iter().map(map_row_to_shared_object).collect())
    }

    async fn find_borrowed_by_user(
        &self,
        borrower_id: Uuid,
    ) -> Result<Vec<SharedObject>, String> {
        let rows = sqlx::query(
            r#"
            SELECT id, owner_id, building_id, object_category::text AS object_category,
                   object_name, description, condition::text AS condition, is_available,
                   rental_credits_per_day, deposit_credits, borrowing_duration_days,
                   current_borrower_id, borrowed_at, due_back_at, photos, location_details,
                   usage_instructions, created_at, updated_at
            FROM shared_objects
            WHERE current_borrower_id = $1
            ORDER BY due_back_at ASC
            "#,
        )
        .bind(borrower_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to find shared objects borrowed by user: {}", e))?;

        Ok(rows.iter().map(map_row_to_shared_object).collect())
    }

    async fn find_by_category(
        &self,
        building_id: Uuid,
        category: SharedObjectCategory,
    ) -> Result<Vec<SharedObject>, String> {
        let category_str = serde_json::to_string(&category)
            .map_err(|e| format!("Failed to serialize category: {}", e))?
            .trim_matches('"')
            .to_string();

        let rows = sqlx::query(
            r#"
            SELECT id, owner_id, building_id, object_category::text AS object_category,
                   object_name, description, condition::text AS condition, is_available,
                   rental_credits_per_day, deposit_credits, borrowing_duration_days,
                   current_borrower_id, borrowed_at, due_back_at, photos, location_details,
                   usage_instructions, created_at, updated_at
            FROM shared_objects
            WHERE building_id = $1 AND object_category = $2::shared_object_category
            ORDER BY
                is_available DESC,
                object_name ASC
            "#,
        )
        .bind(building_id)
        .bind(&category_str)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to find shared objects by category: {}", e))?;

        Ok(rows.iter().map(map_row_to_shared_object).collect())
    }

    async fn find_free_by_building(
        &self,
        building_id: Uuid,
    ) -> Result<Vec<SharedObject>, String> {
        let rows = sqlx::query(
            r#"
            SELECT id, owner_id, building_id, object_category::text AS object_category,
                   object_name, description, condition::text AS condition, is_available,
                   rental_credits_per_day, deposit_credits, borrowing_duration_days,
                   current_borrower_id, borrowed_at, due_back_at, photos, location_details,
                   usage_instructions, created_at, updated_at
            FROM shared_objects
            WHERE building_id = $1
              AND (rental_credits_per_day IS NULL OR rental_credits_per_day = 0)
            ORDER BY
                is_available DESC,
                object_name ASC
            "#,
        )
        .bind(building_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to find free shared objects by building: {}", e))?;

        Ok(rows.iter().map(map_row_to_shared_object).collect())
    }

    async fn update(&self, object: &SharedObject) -> Result<SharedObject, String> {
        let category_str = serde_json::to_string(&object.object_category)
            .map_err(|e| format!("Failed to serialize object_category: {}", e))?
            .trim_matches('"')
            .to_string();

        let condition_str = serde_json::to_string(&object.condition)
            .map_err(|e| format!("Failed to serialize condition: {}", e))?
            .trim_matches('"')
            .to_string();

        sqlx::query(
            r#"
            UPDATE shared_objects
            SET object_category = $2::shared_object_category,
                object_name = $3,
                description = $4,
                condition = $5::object_condition,
                is_available = $6,
                rental_credits_per_day = $7,
                deposit_credits = $8,
                borrowing_duration_days = $9,
                current_borrower_id = $10,
                borrowed_at = $11,
                due_back_at = $12,
                photos = $13,
                location_details = $14,
                usage_instructions = $15,
                updated_at = $16
            WHERE id = $1
            "#,
        )
        .bind(object.id)
        .bind(&category_str)
        .bind(&object.object_name)
        .bind(&object.description)
        .bind(&condition_str)
        .bind(object.is_available)
        .bind(object.rental_credits_per_day)
        .bind(object.deposit_credits)
        .bind(object.borrowing_duration_days)
        .bind(object.current_borrower_id)
        .bind(object.borrowed_at)
        .bind(object.due_back_at)
        .bind(&object.photos)
        .bind(&object.location_details)
        .bind(&object.usage_instructions)
        .bind(object.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to update shared object: {}", e))?;

        Ok(object.clone())
    }

    async fn delete(&self, id: Uuid) -> Result<(), String> {
        sqlx::query(
            r#"
            DELETE FROM shared_objects WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to delete shared object: {}", e))?;

        Ok(())
    }

    async fn count_by_building(&self, building_id: Uuid) -> Result<i64, String> {
        let row = sqlx::query(
            r#"
            SELECT COUNT(*) as count FROM shared_objects WHERE building_id = $1
            "#,
        )
        .bind(building_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to count shared objects by building: {}", e))?;

        Ok(row.get("count"))
    }

    async fn count_available_by_building(&self, building_id: Uuid) -> Result<i64, String> {
        let row = sqlx::query(
            r#"
            SELECT COUNT(*) as count
            FROM shared_objects
            WHERE building_id = $1 AND is_available = TRUE
            "#,
        )
        .bind(building_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to count available shared objects by building: {}", e))?;

        Ok(row.get("count"))
    }

    async fn count_borrowed_by_building(&self, building_id: Uuid) -> Result<i64, String> {
        let row = sqlx::query(
            r#"
            SELECT COUNT(*) as count
            FROM shared_objects
            WHERE building_id = $1 AND current_borrower_id IS NOT NULL
            "#,
        )
        .bind(building_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to count borrowed shared objects by building: {}", e))?;

        Ok(row.get("count"))
    }

    async fn count_overdue_by_building(&self, building_id: Uuid) -> Result<i64, String> {
        let row = sqlx::query(
            r#"
            SELECT COUNT(*) as count
            FROM shared_objects
            WHERE building_id = $1
              AND current_borrower_id IS NOT NULL
              AND due_back_at < NOW()
            "#,
        )
        .bind(building_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to count overdue shared objects by building: {}", e))?;

        Ok(row.get("count"))
    }

    async fn count_by_category(
        &self,
        building_id: Uuid,
        category: SharedObjectCategory,
    ) -> Result<i64, String> {
        let category_str = serde_json::to_string(&category)
            .map_err(|e| format!("Failed to serialize category: {}", e))?
            .trim_matches('"')
            .to_string();

        let row = sqlx::query(
            r#"
            SELECT COUNT(*) as count
            FROM shared_objects
            WHERE building_id = $1 AND object_category = $2::shared_object_category
            "#,
        )
        .bind(building_id)
        .bind(&category_str)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to count shared objects by category: {}", e))?;

        Ok(row.get("count"))
    }
}
