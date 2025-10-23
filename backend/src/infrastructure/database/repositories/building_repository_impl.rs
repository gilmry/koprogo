use crate::application::ports::BuildingRepository;
use crate::domain::entities::Building;
use crate::infrastructure::database::pool::DbPool;
use async_trait::async_trait;
use sqlx::Row;
use uuid::Uuid;

pub struct PostgresBuildingRepository {
    pool: DbPool,
}

impl PostgresBuildingRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl BuildingRepository for PostgresBuildingRepository {
    async fn create(&self, building: &Building) -> Result<Building, String> {
        sqlx::query(
            r#"
            INSERT INTO buildings (id, name, address, city, postal_code, country, total_units, construction_year, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
        )
        .bind(building.id)
        .bind(&building.name)
        .bind(&building.address)
        .bind(&building.city)
        .bind(&building.postal_code)
        .bind(&building.country)
        .bind(building.total_units)
        .bind(building.construction_year)
        .bind(building.created_at)
        .bind(building.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(building.clone())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Building>, String> {
        let row = sqlx::query(
            r#"
            SELECT id, name, address, city, postal_code, country, total_units, construction_year, created_at, updated_at
            FROM buildings
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(row.map(|row| Building {
            id: row.get("id"),
            name: row.get("name"),
            address: row.get("address"),
            city: row.get("city"),
            postal_code: row.get("postal_code"),
            country: row.get("country"),
            total_units: row.get("total_units"),
            construction_year: row.get("construction_year"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }))
    }

    async fn find_all(&self) -> Result<Vec<Building>, String> {
        let rows = sqlx::query(
            r#"
            SELECT id, name, address, city, postal_code, country, total_units, construction_year, created_at, updated_at
            FROM buildings
            ORDER BY created_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(rows
            .iter()
            .map(|row| Building {
                id: row.get("id"),
                name: row.get("name"),
                address: row.get("address"),
                city: row.get("city"),
                postal_code: row.get("postal_code"),
                country: row.get("country"),
                total_units: row.get("total_units"),
                construction_year: row.get("construction_year"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            })
            .collect())
    }

    async fn update(&self, building: &Building) -> Result<Building, String> {
        sqlx::query(
            r#"
            UPDATE buildings
            SET name = $2, address = $3, city = $4, postal_code = $5, updated_at = $6
            WHERE id = $1
            "#,
        )
        .bind(building.id)
        .bind(&building.name)
        .bind(&building.address)
        .bind(&building.city)
        .bind(&building.postal_code)
        .bind(building.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(building.clone())
    }

    async fn delete(&self, id: Uuid) -> Result<bool, String> {
        let result = sqlx::query("DELETE FROM buildings WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("Database error: {}", e))?;

        Ok(result.rows_affected() > 0)
    }
}
