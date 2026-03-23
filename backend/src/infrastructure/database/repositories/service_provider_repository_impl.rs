use crate::application::ports::service_provider_repository::ServiceProviderRepository;
use crate::domain::entities::service_provider::{ServiceProvider, TradeCategory};
use crate::infrastructure::database::pool::DbPool;
use async_trait::async_trait;
use sqlx::Row;
use uuid::Uuid;

pub struct PostgresServiceProviderRepository {
    pool: DbPool,
}

impl PostgresServiceProviderRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ServiceProviderRepository for PostgresServiceProviderRepository {
    async fn create(&self, provider: &ServiceProvider) -> Result<ServiceProvider, String> {
        sqlx::query(
            r#"
            INSERT INTO service_providers (
                id, organization_id, company_name, trade_category,
                specializations, service_zone_postal_codes, certifications,
                ipi_registration, bce_number, rating_avg, reviews_count,
                is_verified, public_profile_slug, created_at, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
            "#,
        )
        .bind(provider.id)
        .bind(provider.organization_id)
        .bind(&provider.company_name)
        .bind(provider.trade_category.to_sql())
        .bind(&provider.specializations)
        .bind(&provider.service_zone_postal_codes)
        .bind(&provider.certifications)
        .bind(&provider.ipi_registration)
        .bind(&provider.bce_number)
        .bind(provider.rating_avg)
        .bind(provider.reviews_count)
        .bind(provider.is_verified)
        .bind(&provider.public_profile_slug)
        .bind(provider.created_at)
        .bind(provider.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Database error creating service provider: {}", e))?;

        Ok(provider.clone())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<ServiceProvider>, String> {
        let row = sqlx::query(
            r#"
            SELECT id, organization_id, company_name, trade_category,
                   specializations, service_zone_postal_codes, certifications,
                   ipi_registration, bce_number, rating_avg, reviews_count,
                   is_verified, public_profile_slug, created_at, updated_at
            FROM service_providers
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(row.map(|row| ServiceProvider {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            company_name: row.get("company_name"),
            trade_category: TradeCategory::from_sql(&row.get::<String, _>("trade_category"))
                .unwrap_or(TradeCategory::Syndic),
            specializations: row.get("specializations"),
            service_zone_postal_codes: row.get("service_zone_postal_codes"),
            certifications: row.get("certifications"),
            ipi_registration: row.get("ipi_registration"),
            bce_number: row.get("bce_number"),
            rating_avg: row.get("rating_avg"),
            reviews_count: row.get("reviews_count"),
            is_verified: row.get("is_verified"),
            public_profile_slug: row.get("public_profile_slug"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }))
    }

    async fn find_by_slug(&self, slug: &str) -> Result<Option<ServiceProvider>, String> {
        let row = sqlx::query(
            r#"
            SELECT id, organization_id, company_name, trade_category,
                   specializations, service_zone_postal_codes, certifications,
                   ipi_registration, bce_number, rating_avg, reviews_count,
                   is_verified, public_profile_slug, created_at, updated_at
            FROM service_providers
            WHERE public_profile_slug = $1
            "#,
        )
        .bind(slug)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(row.map(|row| ServiceProvider {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            company_name: row.get("company_name"),
            trade_category: TradeCategory::from_sql(&row.get::<String, _>("trade_category"))
                .unwrap_or(TradeCategory::Syndic),
            specializations: row.get("specializations"),
            service_zone_postal_codes: row.get("service_zone_postal_codes"),
            certifications: row.get("certifications"),
            ipi_registration: row.get("ipi_registration"),
            bce_number: row.get("bce_number"),
            rating_avg: row.get("rating_avg"),
            reviews_count: row.get("reviews_count"),
            is_verified: row.get("is_verified"),
            public_profile_slug: row.get("public_profile_slug"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }))
    }

    async fn find_all(
        &self,
        organization_id: Option<Uuid>,
        page: i64,
        per_page: i64,
    ) -> Result<Vec<ServiceProvider>, String> {
        if page < 1 || per_page < 1 {
            return Err("Page and per_page must be >= 1".to_string());
        }

        let offset = (page - 1) * per_page;

        let query = if let Some(org_id) = organization_id {
            sqlx::query(
                r#"
                SELECT id, organization_id, company_name, trade_category,
                       specializations, service_zone_postal_codes, certifications,
                       ipi_registration, bce_number, rating_avg, reviews_count,
                       is_verified, public_profile_slug, created_at, updated_at
                FROM service_providers
                WHERE organization_id = $1
                ORDER BY created_at DESC
                LIMIT $2 OFFSET $3
                "#,
            )
            .bind(org_id)
            .bind(per_page)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
        } else {
            sqlx::query(
                r#"
                SELECT id, organization_id, company_name, trade_category,
                       specializations, service_zone_postal_codes, certifications,
                       ipi_registration, bce_number, rating_avg, reviews_count,
                       is_verified, public_profile_slug, created_at, updated_at
                FROM service_providers
                ORDER BY created_at DESC
                LIMIT $1 OFFSET $2
                "#,
            )
            .bind(per_page)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
        };

        let rows = query.map_err(|e| format!("Database error: {}", e))?;

        Ok(rows
            .iter()
            .map(|row| ServiceProvider {
                id: row.get("id"),
                organization_id: row.get("organization_id"),
                company_name: row.get("company_name"),
                trade_category: TradeCategory::from_sql(&row.get::<String, _>("trade_category"))
                    .unwrap_or(TradeCategory::Syndic),
                specializations: row.get("specializations"),
                service_zone_postal_codes: row.get("service_zone_postal_codes"),
                certifications: row.get("certifications"),
                ipi_registration: row.get("ipi_registration"),
                bce_number: row.get("bce_number"),
                rating_avg: row.get("rating_avg"),
                reviews_count: row.get("reviews_count"),
                is_verified: row.get("is_verified"),
                public_profile_slug: row.get("public_profile_slug"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            })
            .collect())
    }

    async fn find_by_trade_category(
        &self,
        category: TradeCategory,
        page: i64,
        per_page: i64,
    ) -> Result<Vec<ServiceProvider>, String> {
        if page < 1 || per_page < 1 {
            return Err("Page and per_page must be >= 1".to_string());
        }

        let offset = (page - 1) * per_page;
        let category_str = category.to_sql();

        let rows = sqlx::query(
            r#"
            SELECT id, organization_id, company_name, trade_category,
                   specializations, service_zone_postal_codes, certifications,
                   ipi_registration, bce_number, rating_avg, reviews_count,
                   is_verified, public_profile_slug, created_at, updated_at
            FROM service_providers
            WHERE trade_category = $1
            ORDER BY rating_avg DESC NULLS LAST, created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(category_str)
        .bind(per_page)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(rows
            .iter()
            .map(|row| ServiceProvider {
                id: row.get("id"),
                organization_id: row.get("organization_id"),
                company_name: row.get("company_name"),
                trade_category: TradeCategory::from_sql(&row.get::<String, _>("trade_category"))
                    .unwrap_or(TradeCategory::Syndic),
                specializations: row.get("specializations"),
                service_zone_postal_codes: row.get("service_zone_postal_codes"),
                certifications: row.get("certifications"),
                ipi_registration: row.get("ipi_registration"),
                bce_number: row.get("bce_number"),
                rating_avg: row.get("rating_avg"),
                reviews_count: row.get("reviews_count"),
                is_verified: row.get("is_verified"),
                public_profile_slug: row.get("public_profile_slug"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            })
            .collect())
    }

    async fn search(
        &self,
        query: &str,
        postal_code: Option<&str>,
        page: i64,
        per_page: i64,
    ) -> Result<Vec<ServiceProvider>, String> {
        if page < 1 || per_page < 1 {
            return Err("Page and per_page must be >= 1".to_string());
        }

        let offset = (page - 1) * per_page;
        let search_query = format!("%{}%", query);

        let rows = if let Some(postal) = postal_code {
            sqlx::query(
                r#"
                SELECT id, organization_id, company_name, trade_category,
                       specializations, service_zone_postal_codes, certifications,
                       ipi_registration, bce_number, rating_avg, reviews_count,
                       is_verified, public_profile_slug, created_at, updated_at
                FROM service_providers
                WHERE (company_name ILIKE $1 OR specializations::text ILIKE $1)
                  AND service_zone_postal_codes @> ARRAY[$2]
                ORDER BY rating_avg DESC NULLS LAST, created_at DESC
                LIMIT $3 OFFSET $4
                "#,
            )
            .bind(&search_query)
            .bind(postal)
            .bind(per_page)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
        } else {
            sqlx::query(
                r#"
                SELECT id, organization_id, company_name, trade_category,
                       specializations, service_zone_postal_codes, certifications,
                       ipi_registration, bce_number, rating_avg, reviews_count,
                       is_verified, public_profile_slug, created_at, updated_at
                FROM service_providers
                WHERE company_name ILIKE $1 OR specializations::text ILIKE $1
                ORDER BY rating_avg DESC NULLS LAST, created_at DESC
                LIMIT $2 OFFSET $3
                "#,
            )
            .bind(&search_query)
            .bind(per_page)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
        };

        let rows = rows.map_err(|e| format!("Database error: {}", e))?;

        Ok(rows
            .iter()
            .map(|row| ServiceProvider {
                id: row.get("id"),
                organization_id: row.get("organization_id"),
                company_name: row.get("company_name"),
                trade_category: TradeCategory::from_sql(&row.get::<String, _>("trade_category"))
                    .unwrap_or(TradeCategory::Syndic),
                specializations: row.get("specializations"),
                service_zone_postal_codes: row.get("service_zone_postal_codes"),
                certifications: row.get("certifications"),
                ipi_registration: row.get("ipi_registration"),
                bce_number: row.get("bce_number"),
                rating_avg: row.get("rating_avg"),
                reviews_count: row.get("reviews_count"),
                is_verified: row.get("is_verified"),
                public_profile_slug: row.get("public_profile_slug"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            })
            .collect())
    }

    async fn update(&self, provider: &ServiceProvider) -> Result<ServiceProvider, String> {
        sqlx::query(
            r#"
            UPDATE service_providers
            SET company_name = $1,
                specializations = $2,
                service_zone_postal_codes = $3,
                certifications = $4,
                ipi_registration = $5,
                bce_number = $6,
                rating_avg = $7,
                reviews_count = $8,
                is_verified = $9,
                updated_at = $10
            WHERE id = $11
            "#,
        )
        .bind(&provider.company_name)
        .bind(&provider.specializations)
        .bind(&provider.service_zone_postal_codes)
        .bind(&provider.certifications)
        .bind(&provider.ipi_registration)
        .bind(&provider.bce_number)
        .bind(provider.rating_avg)
        .bind(provider.reviews_count)
        .bind(provider.is_verified)
        .bind(provider.updated_at)
        .bind(provider.id)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Database error updating service provider: {}", e))?;

        Ok(provider.clone())
    }

    async fn delete(&self, id: Uuid) -> Result<(), String> {
        sqlx::query("DELETE FROM service_providers WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("Database error deleting service provider: {}", e))?;

        Ok(())
    }

    async fn update_rating(
        &self,
        id: Uuid,
        rating_avg: f64,
        reviews_count: i32,
    ) -> Result<(), String> {
        sqlx::query(
            r#"
            UPDATE service_providers
            SET rating_avg = $1, reviews_count = $2, updated_at = NOW()
            WHERE id = $3
            "#,
        )
        .bind(rating_avg)
        .bind(reviews_count)
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Database error updating rating: {}", e))?;

        Ok(())
    }

    async fn count_by_organization(&self, organization_id: Uuid) -> Result<i64, String> {
        let row = sqlx::query("SELECT COUNT(*) as count FROM service_providers WHERE organization_id = $1")
            .bind(organization_id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| format!("Database error: {}", e))?;

        Ok(row.get::<i64, _>("count"))
    }
}
