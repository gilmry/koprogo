use crate::application::dto::{BuildingFilters, PageRequest};
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
            INSERT INTO buildings (id, organization_id, name, address, city, postal_code, country, total_units, construction_year, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            "#,
        )
        .bind(building.id)
        .bind(building.organization_id)
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
            SELECT id, organization_id, name, address, city, postal_code, country, total_units, construction_year, created_at, updated_at
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
            organization_id: row.get("organization_id"),
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
            SELECT id, organization_id, name, address, city, postal_code, country, total_units, construction_year, created_at, updated_at
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
                organization_id: row.get("organization_id"),
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

    async fn find_all_paginated(
        &self,
        page_request: &PageRequest,
        filters: &BuildingFilters,
    ) -> Result<(Vec<Building>, i64), String> {
        // Validate page request
        page_request.validate()?;

        // Build WHERE clause dynamically
        let mut where_clauses = Vec::new();
        let mut param_count = 0;

        if filters.organization_id.is_some() {
            param_count += 1;
            where_clauses.push(format!("organization_id = ${}", param_count));
        }

        if filters.city.is_some() {
            param_count += 1;
            where_clauses.push(format!("city ILIKE ${}", param_count));
        }

        if filters.construction_year.is_some() {
            param_count += 1;
            where_clauses.push(format!("construction_year = ${}", param_count));
        }

        if filters.min_units.is_some() {
            param_count += 1;
            where_clauses.push(format!("total_units >= ${}", param_count));
        }

        if filters.max_units.is_some() {
            param_count += 1;
            where_clauses.push(format!("total_units <= ${}", param_count));
        }

        let where_clause = if where_clauses.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", where_clauses.join(" AND "))
        };

        // Validate sort column (whitelist)
        let allowed_columns = [
            "name",
            "created_at",
            "total_units",
            "city",
            "construction_year",
        ];
        let sort_column = page_request.sort_by.as_deref().unwrap_or("created_at");

        if !allowed_columns.contains(&sort_column) {
            return Err(format!("Invalid sort column: {}", sort_column));
        }

        // Count total items
        let count_query = format!("SELECT COUNT(*) FROM buildings {}", where_clause);
        let mut count_query = sqlx::query_scalar::<_, i64>(&count_query);

        if let Some(org_id) = filters.organization_id {
            count_query = count_query.bind(org_id);
        }
        if let Some(city) = &filters.city {
            count_query = count_query.bind(format!("%{}%", city));
        }
        if let Some(year) = filters.construction_year {
            count_query = count_query.bind(year);
        }
        if let Some(min) = filters.min_units {
            count_query = count_query.bind(min);
        }
        if let Some(max) = filters.max_units {
            count_query = count_query.bind(max);
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
            "SELECT id, organization_id, name, address, city, postal_code, country, total_units, construction_year, created_at, updated_at \
             FROM buildings {} ORDER BY {} {} LIMIT ${} OFFSET ${}",
            where_clause,
            sort_column,
            page_request.order.to_sql(),
            limit_param,
            offset_param
        );

        let mut data_query = sqlx::query(&data_query);

        if let Some(org_id) = filters.organization_id {
            data_query = data_query.bind(org_id);
        }
        if let Some(city) = &filters.city {
            data_query = data_query.bind(format!("%{}%", city));
        }
        if let Some(year) = filters.construction_year {
            data_query = data_query.bind(year);
        }
        if let Some(min) = filters.min_units {
            data_query = data_query.bind(min);
        }
        if let Some(max) = filters.max_units {
            data_query = data_query.bind(max);
        }

        data_query = data_query
            .bind(page_request.limit())
            .bind(page_request.offset());

        let rows = data_query
            .fetch_all(&self.pool)
            .await
            .map_err(|e| format!("Database error: {}", e))?;

        let buildings: Vec<Building> = rows
            .iter()
            .map(|row| Building {
                id: row.get("id"),
                organization_id: row.get("organization_id"),
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
            .collect();

        Ok((buildings, total_items))
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
