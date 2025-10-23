use crate::application::ports::OrganizationRepository;
use crate::domain::entities::{Organization, SubscriptionPlan};
use crate::infrastructure::pool::DbPool;
use async_trait::async_trait;
use uuid::Uuid;

pub struct PostgresOrganizationRepository {
    pool: DbPool,
}

impl PostgresOrganizationRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl OrganizationRepository for PostgresOrganizationRepository {
    async fn create(&self, org: &Organization) -> Result<Organization, String> {
        sqlx::query!(
            r#"
            INSERT INTO organizations (id, name, slug, contact_email, contact_phone, subscription_plan, max_buildings, max_users, is_active, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            "#,
            org.id,
            org.name,
            org.slug,
            org.contact_email,
            org.contact_phone,
            org.subscription_plan.to_string(),
            org.max_buildings,
            org.max_users,
            org.is_active,
            org.created_at,
            org.updated_at,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to create organization: {}", e))?;

        Ok(org.clone())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Organization>, String> {
        let result = sqlx::query!(
            r#"
            SELECT id, name, slug, contact_email, contact_phone, subscription_plan, max_buildings, max_users, is_active, created_at, updated_at
            FROM organizations
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Failed to find organization: {}", e))?;

        match result {
            Some(row) => {
                let subscription_plan = row
                    .subscription_plan
                    .parse::<SubscriptionPlan>()
                    .map_err(|e| format!("Invalid subscription plan: {}", e))?;

                Ok(Some(Organization {
                    id: row.id,
                    name: row.name,
                    slug: row.slug,
                    contact_email: row.contact_email,
                    contact_phone: row.contact_phone,
                    subscription_plan,
                    max_buildings: row.max_buildings,
                    max_users: row.max_users,
                    is_active: row.is_active,
                    created_at: row.created_at,
                    updated_at: row.updated_at,
                }))
            }
            None => Ok(None),
        }
    }

    async fn find_by_slug(&self, slug: &str) -> Result<Option<Organization>, String> {
        let result = sqlx::query!(
            r#"
            SELECT id, name, slug, contact_email, contact_phone, subscription_plan, max_buildings, max_users, is_active, created_at, updated_at
            FROM organizations
            WHERE slug = $1
            "#,
            slug
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Failed to find organization by slug: {}", e))?;

        match result {
            Some(row) => {
                let subscription_plan = row
                    .subscription_plan
                    .parse::<SubscriptionPlan>()
                    .map_err(|e| format!("Invalid subscription plan: {}", e))?;

                Ok(Some(Organization {
                    id: row.id,
                    name: row.name,
                    slug: row.slug,
                    contact_email: row.contact_email,
                    contact_phone: row.contact_phone,
                    subscription_plan,
                    max_buildings: row.max_buildings,
                    max_users: row.max_users,
                    is_active: row.is_active,
                    created_at: row.created_at,
                    updated_at: row.updated_at,
                }))
            }
            None => Ok(None),
        }
    }

    async fn find_all(&self) -> Result<Vec<Organization>, String> {
        let rows = sqlx::query!(
            r#"
            SELECT id, name, slug, contact_email, contact_phone, subscription_plan, max_buildings, max_users, is_active, created_at, updated_at
            FROM organizations
            ORDER BY created_at DESC
            "#
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to fetch organizations: {}", e))?;

        let orgs = rows
            .into_iter()
            .filter_map(|row| {
                let subscription_plan = row.subscription_plan.parse::<SubscriptionPlan>().ok()?;
                Some(Organization {
                    id: row.id,
                    name: row.name,
                    slug: row.slug,
                    contact_email: row.contact_email,
                    contact_phone: row.contact_phone,
                    subscription_plan,
                    max_buildings: row.max_buildings,
                    max_users: row.max_users,
                    is_active: row.is_active,
                    created_at: row.created_at,
                    updated_at: row.updated_at,
                })
            })
            .collect();

        Ok(orgs)
    }

    async fn update(&self, org: &Organization) -> Result<Organization, String> {
        sqlx::query!(
            r#"
            UPDATE organizations
            SET name = $2, slug = $3, contact_email = $4, contact_phone = $5,
                subscription_plan = $6, max_buildings = $7, max_users = $8,
                is_active = $9, updated_at = $10
            WHERE id = $1
            "#,
            org.id,
            org.name,
            org.slug,
            org.contact_email,
            org.contact_phone,
            org.subscription_plan.to_string(),
            org.max_buildings,
            org.max_users,
            org.is_active,
            org.updated_at,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to update organization: {}", e))?;

        Ok(org.clone())
    }

    async fn delete(&self, id: Uuid) -> Result<bool, String> {
        let result = sqlx::query!(
            r#"
            DELETE FROM organizations
            WHERE id = $1
            "#,
            id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to delete organization: {}", e))?;

        Ok(result.rows_affected() > 0)
    }

    async fn count_buildings(&self, org_id: Uuid) -> Result<i64, String> {
        let result = sqlx::query!(
            r#"
            SELECT COUNT(*) as count
            FROM buildings
            WHERE organization_id = $1
            "#,
            org_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to count buildings: {}", e))?;

        Ok(result.count.unwrap_or(0))
    }
}
