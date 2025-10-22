use crate::application::ports::UnitRepository;
use crate::domain::entities::{Unit, UnitType};
use crate::infrastructure::database::pool::DbPool;
use async_trait::async_trait;
use sqlx::Row;
use uuid::Uuid;

pub struct PostgresUnitRepository {
    pool: DbPool,
}

impl PostgresUnitRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UnitRepository for PostgresUnitRepository {
    async fn create(&self, unit: &Unit) -> Result<Unit, String> {
        let unit_type_str = match unit.unit_type {
            UnitType::Apartment => "apartment",
            UnitType::Parking => "parking",
            UnitType::Cellar => "cellar",
            UnitType::Commercial => "commercial",
            UnitType::Other => "other",
        };

        sqlx::query(
            r#"
            INSERT INTO units (id, building_id, unit_number, unit_type, floor, surface_area, quota, owner_id, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
        )
        .bind(unit.id)
        .bind(unit.building_id)
        .bind(&unit.unit_number)
        .bind(unit_type_str)
        .bind(unit.floor)
        .bind(unit.surface_area)
        .bind(unit.quota)
        .bind(unit.owner_id)
        .bind(unit.created_at)
        .bind(unit.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(unit.clone())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Unit>, String> {
        let row = sqlx::query(
            r#"
            SELECT id, building_id, unit_number, unit_type, floor, surface_area, quota, owner_id, created_at, updated_at
            FROM units
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(row.map(|row| {
            let unit_type_str: String = row.get("unit_type");
            let unit_type = match unit_type_str.as_str() {
                "apartment" => UnitType::Apartment,
                "parking" => UnitType::Parking,
                "cellar" => UnitType::Cellar,
                "commercial" => UnitType::Commercial,
                _ => UnitType::Other,
            };

            Unit {
                id: row.get("id"),
                building_id: row.get("building_id"),
                unit_number: row.get("unit_number"),
                unit_type,
                floor: row.get("floor"),
                surface_area: row.get("surface_area"),
                quota: row.get("quota"),
                owner_id: row.get("owner_id"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            }
        }))
    }

    async fn find_by_building(&self, building_id: Uuid) -> Result<Vec<Unit>, String> {
        let rows = sqlx::query(
            r#"
            SELECT id, building_id, unit_number, unit_type, floor, surface_area, quota, owner_id, created_at, updated_at
            FROM units
            WHERE building_id = $1
            ORDER BY unit_number
            "#,
        )
        .bind(building_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(rows
            .iter()
            .map(|row| {
                let unit_type_str: String = row.get("unit_type");
                let unit_type = match unit_type_str.as_str() {
                    "apartment" => UnitType::Apartment,
                    "parking" => UnitType::Parking,
                    "cellar" => UnitType::Cellar,
                    "commercial" => UnitType::Commercial,
                    _ => UnitType::Other,
                };

                Unit {
                    id: row.get("id"),
                    building_id: row.get("building_id"),
                    unit_number: row.get("unit_number"),
                    unit_type,
                    floor: row.get("floor"),
                    surface_area: row.get("surface_area"),
                    quota: row.get("quota"),
                    owner_id: row.get("owner_id"),
                    created_at: row.get("created_at"),
                    updated_at: row.get("updated_at"),
                }
            })
            .collect())
    }

    async fn find_by_owner(&self, owner_id: Uuid) -> Result<Vec<Unit>, String> {
        let rows = sqlx::query(
            r#"
            SELECT id, building_id, unit_number, unit_type, floor, surface_area, quota, owner_id, created_at, updated_at
            FROM units
            WHERE owner_id = $1
            ORDER BY unit_number
            "#,
        )
        .bind(owner_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(rows
            .iter()
            .map(|row| {
                let unit_type_str: String = row.get("unit_type");
                let unit_type = match unit_type_str.as_str() {
                    "apartment" => UnitType::Apartment,
                    "parking" => UnitType::Parking,
                    "cellar" => UnitType::Cellar,
                    "commercial" => UnitType::Commercial,
                    _ => UnitType::Other,
                };

                Unit {
                    id: row.get("id"),
                    building_id: row.get("building_id"),
                    unit_number: row.get("unit_number"),
                    unit_type,
                    floor: row.get("floor"),
                    surface_area: row.get("surface_area"),
                    quota: row.get("quota"),
                    owner_id: row.get("owner_id"),
                    created_at: row.get("created_at"),
                    updated_at: row.get("updated_at"),
                }
            })
            .collect())
    }

    async fn update(&self, unit: &Unit) -> Result<Unit, String> {
        sqlx::query(
            r#"
            UPDATE units
            SET owner_id = $2, updated_at = $3
            WHERE id = $1
            "#,
        )
        .bind(unit.id)
        .bind(unit.owner_id)
        .bind(unit.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(unit.clone())
    }

    async fn delete(&self, id: Uuid) -> Result<bool, String> {
        let result = sqlx::query("DELETE FROM units WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("Database error: {}", e))?;

        Ok(result.rows_affected() > 0)
    }
}
