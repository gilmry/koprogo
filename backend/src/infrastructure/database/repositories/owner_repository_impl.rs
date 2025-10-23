use crate::application::dto::{OwnerFilters, PageRequest};
use crate::application::ports::OwnerRepository;
use crate::domain::entities::Owner;
use crate::infrastructure::database::pool::DbPool;
use async_trait::async_trait;
use sqlx::Row;
use uuid::Uuid;

pub struct PostgresOwnerRepository {
    pool: DbPool,
}

impl PostgresOwnerRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl OwnerRepository for PostgresOwnerRepository {
    async fn create(&self, owner: &Owner) -> Result<Owner, String> {
        sqlx::query(
            r#"
            INSERT INTO owners (id, organization_id, first_name, last_name, email, phone, address, city, postal_code, country, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            "#,
        )
        .bind(owner.id)
        .bind(owner.organization_id)
        .bind(&owner.first_name)
        .bind(&owner.last_name)
        .bind(&owner.email)
        .bind(&owner.phone)
        .bind(&owner.address)
        .bind(&owner.city)
        .bind(&owner.postal_code)
        .bind(&owner.country)
        .bind(owner.created_at)
        .bind(owner.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(owner.clone())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Owner>, String> {
        let row = sqlx::query(
            r#"
            SELECT id, organization_id, first_name, last_name, email, phone, address, city, postal_code, country, created_at, updated_at
            FROM owners
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(row.map(|row| Owner {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            first_name: row.get("first_name"),
            last_name: row.get("last_name"),
            email: row.get("email"),
            phone: row.get("phone"),
            address: row.get("address"),
            city: row.get("city"),
            postal_code: row.get("postal_code"),
            country: row.get("country"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }))
    }

    async fn find_by_email(&self, email: &str) -> Result<Option<Owner>, String> {
        let row = sqlx::query(
            r#"
            SELECT id, organization_id, first_name, last_name, email, phone, address, city, postal_code, country, created_at, updated_at
            FROM owners
            WHERE email = $1
            "#,
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(row.map(|row| Owner {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            first_name: row.get("first_name"),
            last_name: row.get("last_name"),
            email: row.get("email"),
            phone: row.get("phone"),
            address: row.get("address"),
            city: row.get("city"),
            postal_code: row.get("postal_code"),
            country: row.get("country"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }))
    }

    async fn find_all(&self) -> Result<Vec<Owner>, String> {
        let rows = sqlx::query(
            r#"
            SELECT id, organization_id, first_name, last_name, email, phone, address, city, postal_code, country, created_at, updated_at
            FROM owners
            ORDER BY last_name, first_name
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(rows
            .iter()
            .map(|row| Owner {
                id: row.get("id"),
                organization_id: row.get("organization_id"),
                first_name: row.get("first_name"),
                last_name: row.get("last_name"),
                email: row.get("email"),
                phone: row.get("phone"),
                address: row.get("address"),
                city: row.get("city"),
                postal_code: row.get("postal_code"),
                country: row.get("country"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            })
            .collect())
    }

    async fn find_all_paginated(
        &self,
        page_request: &PageRequest,
        filters: &OwnerFilters,
    ) -> Result<(Vec<Owner>, i64), String> {
        // Validate page request
        page_request.validate()?;

        // Build WHERE clause dynamically
        let mut where_clauses = Vec::new();
        let mut param_count = 0;

        if filters.email.is_some() {
            param_count += 1;
            where_clauses.push(format!("email ILIKE ${}", param_count));
        }

        if filters.phone.is_some() {
            param_count += 1;
            where_clauses.push(format!("phone ILIKE ${}", param_count));
        }

        if filters.last_name.is_some() {
            param_count += 1;
            where_clauses.push(format!("last_name ILIKE ${}", param_count));
        }

        if filters.first_name.is_some() {
            param_count += 1;
            where_clauses.push(format!("first_name ILIKE ${}", param_count));
        }

        let where_clause = if where_clauses.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", where_clauses.join(" AND "))
        };

        // Validate sort column (whitelist)
        let allowed_columns = ["last_name", "first_name", "email", "created_at"];
        let sort_column = page_request.sort_by.as_deref().unwrap_or("last_name");

        if !allowed_columns.contains(&sort_column) {
            return Err(format!("Invalid sort column: {}", sort_column));
        }

        // Count total items
        let count_query = format!("SELECT COUNT(*) FROM owners {}", where_clause);
        let mut count_query = sqlx::query_scalar::<_, i64>(&count_query);

        if let Some(email) = &filters.email {
            count_query = count_query.bind(format!("%{}%", email));
        }
        if let Some(phone) = &filters.phone {
            count_query = count_query.bind(format!("%{}%", phone));
        }
        if let Some(last_name) = &filters.last_name {
            count_query = count_query.bind(format!("%{}%", last_name));
        }
        if let Some(first_name) = &filters.first_name {
            count_query = count_query.bind(format!("%{}%", first_name));
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
            "SELECT id, first_name, last_name, email, phone, address, city, postal_code, country, created_at, updated_at \
             FROM owners {} ORDER BY {} {} LIMIT ${} OFFSET ${}",
            where_clause,
            sort_column,
            page_request.order.to_sql(),
            limit_param,
            offset_param
        );

        let mut data_query = sqlx::query(&data_query);

        if let Some(email) = &filters.email {
            data_query = data_query.bind(format!("%{}%", email));
        }
        if let Some(phone) = &filters.phone {
            data_query = data_query.bind(format!("%{}%", phone));
        }
        if let Some(last_name) = &filters.last_name {
            data_query = data_query.bind(format!("%{}%", last_name));
        }
        if let Some(first_name) = &filters.first_name {
            data_query = data_query.bind(format!("%{}%", first_name));
        }

        data_query = data_query
            .bind(page_request.limit())
            .bind(page_request.offset());

        let rows = data_query
            .fetch_all(&self.pool)
            .await
            .map_err(|e| format!("Database error: {}", e))?;

        let owners: Vec<Owner> = rows
            .iter()
            .map(|row| Owner {
                id: row.get("id"),
                organization_id: row.get("organization_id"),
                first_name: row.get("first_name"),
                last_name: row.get("last_name"),
                email: row.get("email"),
                phone: row.get("phone"),
                address: row.get("address"),
                city: row.get("city"),
                postal_code: row.get("postal_code"),
                country: row.get("country"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            })
            .collect();

        Ok((owners, total_items))
    }

    async fn update(&self, owner: &Owner) -> Result<Owner, String> {
        sqlx::query(
            r#"
            UPDATE owners
            SET first_name = $2, last_name = $3, email = $4, phone = $5, address = $6, city = $7, postal_code = $8, country = $9, updated_at = $10
            WHERE id = $1
            "#,
        )
        .bind(owner.id)
        .bind(&owner.first_name)
        .bind(&owner.last_name)
        .bind(&owner.email)
        .bind(&owner.phone)
        .bind(&owner.address)
        .bind(&owner.city)
        .bind(&owner.postal_code)
        .bind(&owner.country)
        .bind(owner.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(owner.clone())
    }

    async fn delete(&self, id: Uuid) -> Result<bool, String> {
        let result = sqlx::query("DELETE FROM owners WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("Database error: {}", e))?;

        Ok(result.rows_affected() > 0)
    }
}
