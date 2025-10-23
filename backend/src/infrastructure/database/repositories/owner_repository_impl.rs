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
            INSERT INTO owners (id, first_name, last_name, email, phone, address, city, postal_code, country, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
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
            SELECT id, first_name, last_name, email, phone, address, city, postal_code, country, created_at, updated_at
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
            SELECT id, first_name, last_name, email, phone, address, city, postal_code, country, created_at, updated_at
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
            SELECT id, first_name, last_name, email, phone, address, city, postal_code, country, created_at, updated_at
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
