use crate::application::ports::consent_repository::ConsentRepository;
use crate::domain::entities::consent::{ConsentRecord, ConsentStatus};
use crate::infrastructure::database::pool::DbPool;
use async_trait::async_trait;
use sqlx::Row;
use uuid::Uuid;

pub struct PostgresConsentRepository {
    pool: DbPool,
}

impl PostgresConsentRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

fn row_to_consent_record(row: &sqlx::postgres::PgRow) -> ConsentRecord {
    ConsentRecord {
        id: row.get("id"),
        user_id: row.get("user_id"),
        organization_id: row.get("organization_id"),
        consent_type: row.get("consent_type"),
        accepted_at: row.get("accepted_at"),
        ip_address: row.get("ip_address"),
        user_agent: row.get("user_agent"),
        policy_version: row.get("policy_version"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

#[async_trait]
impl ConsentRepository for PostgresConsentRepository {
    async fn create(&self, record: &ConsentRecord) -> Result<ConsentRecord, String> {
        let row = sqlx::query(
            r#"
            INSERT INTO consent_records (id, user_id, organization_id, consent_type, accepted_at, ip_address, user_agent, policy_version, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING id, user_id, organization_id, consent_type, accepted_at, ip_address, user_agent, policy_version, created_at, updated_at
            "#,
        )
        .bind(record.id)
        .bind(record.user_id)
        .bind(record.organization_id)
        .bind(&record.consent_type)
        .bind(record.accepted_at)
        .bind(&record.ip_address)
        .bind(&record.user_agent)
        .bind(&record.policy_version)
        .bind(record.created_at)
        .bind(record.updated_at)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to create consent record: {}", e))?;

        Ok(row_to_consent_record(&row))
    }

    async fn find_latest_by_user_and_type(
        &self,
        user_id: Uuid,
        consent_type: &str,
    ) -> Result<Option<ConsentRecord>, String> {
        let row = sqlx::query(
            r#"
            SELECT id, user_id, organization_id, consent_type, accepted_at, ip_address, user_agent, policy_version, created_at, updated_at
            FROM consent_records
            WHERE user_id = $1 AND consent_type = $2
            ORDER BY accepted_at DESC
            LIMIT 1
            "#,
        )
        .bind(user_id)
        .bind(consent_type)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Failed to find consent record: {}", e))?;

        Ok(row.as_ref().map(row_to_consent_record))
    }

    async fn find_all_by_user(&self, user_id: Uuid) -> Result<Vec<ConsentRecord>, String> {
        let rows = sqlx::query(
            r#"
            SELECT id, user_id, organization_id, consent_type, accepted_at, ip_address, user_agent, policy_version, created_at, updated_at
            FROM consent_records
            WHERE user_id = $1
            ORDER BY accepted_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to find consent records: {}", e))?;

        Ok(rows.iter().map(row_to_consent_record).collect())
    }

    async fn has_accepted(&self, user_id: Uuid, consent_type: &str) -> Result<bool, String> {
        let row: (bool,) = sqlx::query_as(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM consent_records
                WHERE user_id = $1 AND consent_type = $2
            )
            "#,
        )
        .bind(user_id)
        .bind(consent_type)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to check consent: {}", e))?;

        Ok(row.0)
    }

    async fn get_consent_status(&self, user_id: Uuid) -> Result<ConsentStatus, String> {
        let privacy = self
            .find_latest_by_user_and_type(user_id, "privacy_policy")
            .await?;
        let terms = self
            .find_latest_by_user_and_type(user_id, "terms")
            .await?;

        Ok(ConsentStatus {
            privacy_policy_accepted: privacy.is_some(),
            terms_accepted: terms.is_some(),
            privacy_policy_accepted_at: privacy.as_ref().map(|r| r.accepted_at),
            terms_accepted_at: terms.as_ref().map(|r| r.accepted_at),
            privacy_policy_version: privacy.map(|r| r.policy_version),
            terms_version: terms.map(|r| r.policy_version),
        })
    }
}
