use crate::application::dto::{PageRequest, UnitFilters};
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
            INSERT INTO units (id, organization_id, building_id, unit_number, unit_type, floor, surface_area, quota, owner_id, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5::unit_type, $6, $7, $8, $9, $10, $11)
            "#,
        )
        .bind(unit.id)
        .bind(unit.organization_id)
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
            SELECT id, organization_id, building_id, unit_number, unit_type::text AS unit_type, floor, surface_area, quota, owner_id, created_at, updated_at
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
                organization_id: row.get("organization_id"),
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
            SELECT id, organization_id, building_id, unit_number, unit_type::text AS unit_type, floor, surface_area, quota, owner_id, created_at, updated_at
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
                    organization_id: row.get("organization_id"),
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
            SELECT id, organization_id, building_id, unit_number, unit_type::text AS unit_type, floor, surface_area, quota, owner_id, created_at, updated_at
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
                    organization_id: row.get("organization_id"),
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

    async fn find_all_paginated(
        &self,
        page_request: &PageRequest,
        filters: &UnitFilters,
    ) -> Result<(Vec<Unit>, i64), String> {
        // Validate page request
        page_request.validate()?;

        // Build WHERE clause dynamically
        let mut where_clauses = Vec::new();
        let mut param_count = 0;

        if filters.building_id.is_some() {
            param_count += 1;
            where_clauses.push(format!("building_id = ${}", param_count));
        }

        if filters.floor.is_some() {
            param_count += 1;
            where_clauses.push(format!("floor = ${}", param_count));
        }

        if let Some(has_owner) = filters.has_owner {
            if has_owner {
                where_clauses.push("owner_id IS NOT NULL".to_string());
            } else {
                where_clauses.push("owner_id IS NULL".to_string());
            }
        }

        if filters.min_area.is_some() {
            param_count += 1;
            where_clauses.push(format!("surface_area >= ${}", param_count));
        }

        if filters.max_area.is_some() {
            param_count += 1;
            where_clauses.push(format!("surface_area <= ${}", param_count));
        }

        let where_clause = if where_clauses.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", where_clauses.join(" AND "))
        };

        // Validate sort column (whitelist)
        let allowed_columns = ["unit_number", "floor", "surface_area", "created_at"];
        let sort_column = page_request.sort_by.as_deref().unwrap_or("unit_number");

        if !allowed_columns.contains(&sort_column) {
            return Err(format!("Invalid sort column: {}", sort_column));
        }

        // Count total items
        let count_query = format!("SELECT COUNT(*) FROM units {}", where_clause);
        let mut count_query = sqlx::query_scalar::<_, i64>(&count_query);

        if let Some(building_id) = filters.building_id {
            count_query = count_query.bind(building_id);
        }
        if let Some(floor) = filters.floor {
            count_query = count_query.bind(floor);
        }
        if let Some(min_area) = filters.min_area {
            count_query = count_query.bind(min_area);
        }
        if let Some(max_area) = filters.max_area {
            count_query = count_query.bind(max_area);
        }

        let total_items = count_query
            .fetch_one(&self.pool)
            .await
            .map_err(|e| format!("Database error: {}", e))?;

        // Fetch paginated data
        param_count += 1;
        let limit_param = param_count;
        param_count += 1;
        let offset_param = param_count;

        let data_query = format!(
            "SELECT id, organization_id, building_id, unit_number, unit_type::text AS unit_type, floor, surface_area, quota, owner_id, created_at, updated_at \
             FROM units {} ORDER BY {} {} LIMIT ${} OFFSET ${}",
            where_clause,
            sort_column,
            page_request.order.to_sql(),
            limit_param,
            offset_param
        );

        let mut data_query = sqlx::query(&data_query);

        if let Some(building_id) = filters.building_id {
            data_query = data_query.bind(building_id);
        }
        if let Some(floor) = filters.floor {
            data_query = data_query.bind(floor);
        }
        if let Some(min_area) = filters.min_area {
            data_query = data_query.bind(min_area);
        }
        if let Some(max_area) = filters.max_area {
            data_query = data_query.bind(max_area);
        }

        data_query = data_query
            .bind(page_request.limit())
            .bind(page_request.offset());

        let rows = data_query
            .fetch_all(&self.pool)
            .await
            .map_err(|e| format!("Database error: {}", e))?;

        let units: Vec<Unit> = rows
            .iter()
            .map(|row| {
                // Try to get as String first (for enum types from PostgreSQL)
                let unit_type_str: String = row
                    .try_get("unit_type")
                    .unwrap_or_else(|_| "apartment".to_string());
                let unit_type = match unit_type_str.as_str() {
                    "apartment" => UnitType::Apartment,
                    "parking" => UnitType::Parking,
                    "cellar" => UnitType::Cellar,
                    "commercial" => UnitType::Commercial,
                    _ => UnitType::Other,
                };

                Unit {
                    id: row.get("id"),
                    organization_id: row.get("organization_id"),
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
            .collect();

        Ok((units, total_items))
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
