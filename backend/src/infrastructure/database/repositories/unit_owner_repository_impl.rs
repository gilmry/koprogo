use crate::application::ports::UnitOwnerRepository;
use crate::domain::entities::UnitOwner;
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

pub struct PostgresUnitOwnerRepository {
    pool: PgPool,
}

impl PostgresUnitOwnerRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UnitOwnerRepository for PostgresUnitOwnerRepository {
    async fn create(&self, unit_owner: &UnitOwner) -> Result<UnitOwner, String> {
        let result = sqlx::query!(
            r#"
            INSERT INTO unit_owners (
                id, unit_id, owner_id, ownership_percentage,
                start_date, end_date, is_primary_contact,
                created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING *
            "#,
            unit_owner.id,
            unit_owner.unit_id,
            unit_owner.owner_id,
            unit_owner.ownership_percentage,
            unit_owner.start_date,
            unit_owner.end_date,
            unit_owner.is_primary_contact,
            unit_owner.created_at,
            unit_owner.updated_at
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to create unit_owner: {}", e))?;

        Ok(UnitOwner {
            id: result.id,
            unit_id: result.unit_id,
            owner_id: result.owner_id,
            ownership_percentage: result.ownership_percentage,
            start_date: result.start_date,
            end_date: result.end_date,
            is_primary_contact: result.is_primary_contact,
            created_at: result.created_at,
            updated_at: result.updated_at,
        })
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<UnitOwner>, String> {
        let result = sqlx::query!(
            r#"
            SELECT * FROM unit_owners WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Failed to find unit_owner: {}", e))?;

        Ok(result.map(|row| UnitOwner {
            id: row.id,
            unit_id: row.unit_id,
            owner_id: row.owner_id,
            ownership_percentage: row.ownership_percentage,
            start_date: row.start_date,
            end_date: row.end_date,
            is_primary_contact: row.is_primary_contact,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }))
    }

    async fn find_current_owners_by_unit(&self, unit_id: Uuid) -> Result<Vec<UnitOwner>, String> {
        let results = sqlx::query!(
            r#"
            SELECT * FROM unit_owners
            WHERE unit_id = $1 AND end_date IS NULL
            ORDER BY is_primary_contact DESC, created_at ASC
            "#,
            unit_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to find owners by unit: {}", e))?;

        Ok(results
            .into_iter()
            .map(|row| UnitOwner {
                id: row.id,
                unit_id: row.unit_id,
                owner_id: row.owner_id,
                ownership_percentage: row.ownership_percentage,
                start_date: row.start_date,
                end_date: row.end_date,
                is_primary_contact: row.is_primary_contact,
                created_at: row.created_at,
                updated_at: row.updated_at,
            })
            .collect())
    }

    async fn find_current_units_by_owner(&self, owner_id: Uuid) -> Result<Vec<UnitOwner>, String> {
        let results = sqlx::query!(
            r#"
            SELECT * FROM unit_owners
            WHERE owner_id = $1 AND end_date IS NULL
            ORDER BY created_at ASC
            "#,
            owner_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to find units by owner: {}", e))?;

        Ok(results
            .into_iter()
            .map(|row| UnitOwner {
                id: row.id,
                unit_id: row.unit_id,
                owner_id: row.owner_id,
                ownership_percentage: row.ownership_percentage,
                start_date: row.start_date,
                end_date: row.end_date,
                is_primary_contact: row.is_primary_contact,
                created_at: row.created_at,
                updated_at: row.updated_at,
            })
            .collect())
    }

    async fn find_all_owners_by_unit(&self, unit_id: Uuid) -> Result<Vec<UnitOwner>, String> {
        let results = sqlx::query!(
            r#"
            SELECT * FROM unit_owners
            WHERE unit_id = $1
            ORDER BY start_date DESC
            "#,
            unit_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to find all owners by unit: {}", e))?;

        Ok(results
            .into_iter()
            .map(|row| UnitOwner {
                id: row.id,
                unit_id: row.unit_id,
                owner_id: row.owner_id,
                ownership_percentage: row.ownership_percentage,
                start_date: row.start_date,
                end_date: row.end_date,
                is_primary_contact: row.is_primary_contact,
                created_at: row.created_at,
                updated_at: row.updated_at,
            })
            .collect())
    }

    async fn find_all_units_by_owner(&self, owner_id: Uuid) -> Result<Vec<UnitOwner>, String> {
        let results = sqlx::query!(
            r#"
            SELECT * FROM unit_owners
            WHERE owner_id = $1
            ORDER BY start_date DESC
            "#,
            owner_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to find all units by owner: {}", e))?;

        Ok(results
            .into_iter()
            .map(|row| UnitOwner {
                id: row.id,
                unit_id: row.unit_id,
                owner_id: row.owner_id,
                ownership_percentage: row.ownership_percentage,
                start_date: row.start_date,
                end_date: row.end_date,
                is_primary_contact: row.is_primary_contact,
                created_at: row.created_at,
                updated_at: row.updated_at,
            })
            .collect())
    }

    async fn update(&self, unit_owner: &UnitOwner) -> Result<UnitOwner, String> {
        let result = sqlx::query!(
            r#"
            UPDATE unit_owners
            SET ownership_percentage = $2,
                end_date = $3,
                is_primary_contact = $4,
                updated_at = $5
            WHERE id = $1
            RETURNING *
            "#,
            unit_owner.id,
            unit_owner.ownership_percentage,
            unit_owner.end_date,
            unit_owner.is_primary_contact,
            unit_owner.updated_at
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to update unit_owner: {}", e))?;

        Ok(UnitOwner {
            id: result.id,
            unit_id: result.unit_id,
            owner_id: result.owner_id,
            ownership_percentage: result.ownership_percentage,
            start_date: result.start_date,
            end_date: result.end_date,
            is_primary_contact: result.is_primary_contact,
            created_at: result.created_at,
            updated_at: result.updated_at,
        })
    }

    async fn delete(&self, id: Uuid) -> Result<(), String> {
        sqlx::query!(
            r#"
            DELETE FROM unit_owners WHERE id = $1
            "#,
            id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to delete unit_owner: {}", e))?;

        Ok(())
    }

    async fn has_active_owners(&self, unit_id: Uuid) -> Result<bool, String> {
        let result = sqlx::query!(
            r#"
            SELECT EXISTS(SELECT 1 FROM unit_owners WHERE unit_id = $1 AND end_date IS NULL) as "exists!"
            "#,
            unit_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to check active owners: {}", e))?;

        Ok(result.exists)
    }

    async fn get_total_ownership_percentage(&self, unit_id: Uuid) -> Result<f64, String> {
        let result = sqlx::query!(
            r#"
            SELECT COALESCE(SUM(ownership_percentage), 0) as "total!"
            FROM unit_owners
            WHERE unit_id = $1 AND end_date IS NULL
            "#,
            unit_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to get total ownership percentage: {}", e))?;

        Ok(result.total)
    }

    async fn find_active_by_unit_and_owner(
        &self,
        unit_id: Uuid,
        owner_id: Uuid,
    ) -> Result<Option<UnitOwner>, String> {
        let result = sqlx::query!(
            r#"
            SELECT * FROM unit_owners
            WHERE unit_id = $1 AND owner_id = $2 AND end_date IS NULL
            "#,
            unit_id,
            owner_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Failed to find active unit_owner: {}", e))?;

        Ok(result.map(|row| UnitOwner {
            id: row.id,
            unit_id: row.unit_id,
            owner_id: row.owner_id,
            ownership_percentage: row.ownership_percentage,
            start_date: row.start_date,
            end_date: row.end_date,
            is_primary_contact: row.is_primary_contact,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }))
    }

    async fn find_active_by_building(
        &self,
        building_id: Uuid,
    ) -> Result<Vec<(Uuid, Uuid, f64)>, String> {
        let results = sqlx::query!(
            r#"
            SELECT uo.unit_id, uo.owner_id, uo.ownership_percentage
            FROM unit_owners uo
            JOIN units u ON uo.unit_id = u.id
            WHERE u.building_id = $1 AND uo.end_date IS NULL
            "#,
            building_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to find active unit_owners by building: {}", e))?;

        Ok(results
            .into_iter()
            .map(|row| (row.unit_id, row.owner_id, row.ownership_percentage))
            .collect())
    }
}
