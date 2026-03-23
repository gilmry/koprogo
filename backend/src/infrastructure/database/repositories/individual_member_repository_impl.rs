use crate::application::ports::individual_member_repository::IndividualMemberRepository;
use crate::domain::entities::individual_member::IndividualMember;
use crate::infrastructure::database::pool::DbPool;
use async_trait::async_trait;
use sqlx::Row;
use uuid::Uuid;

pub struct PostgresIndividualMemberRepository {
    pool: DbPool,
}

impl PostgresIndividualMemberRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl IndividualMemberRepository for PostgresIndividualMemberRepository {
    async fn create(&self, member: &IndividualMember) -> Result<IndividualMember, String> {
        sqlx::query(
            r#"
            INSERT INTO individual_members (
                id, campaign_id, email, postal_code, has_gdpr_consent,
                consent_at, annual_consumption_kwh, current_provider,
                ean_code, unsubscribed_at, created_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            "#,
        )
        .bind(member.id)
        .bind(member.campaign_id)
        .bind(&member.email)
        .bind(&member.postal_code)
        .bind(member.has_gdpr_consent)
        .bind(member.consent_at)
        .bind(member.annual_consumption_kwh)
        .bind(&member.current_provider)
        .bind(&member.ean_code)
        .bind(member.unsubscribed_at)
        .bind(member.created_at)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Database error creating individual member: {}", e))?;

        Ok(member.clone())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<IndividualMember>, String> {
        let row = sqlx::query(
            r#"
            SELECT id, campaign_id, email, postal_code, has_gdpr_consent,
                   consent_at, annual_consumption_kwh, current_provider,
                   ean_code, unsubscribed_at, created_at
            FROM individual_members
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(row.map(|row| IndividualMember {
            id: row.get("id"),
            campaign_id: row.get("campaign_id"),
            email: row.get("email"),
            postal_code: row.get("postal_code"),
            has_gdpr_consent: row.get("has_gdpr_consent"),
            consent_at: row.get("consent_at"),
            annual_consumption_kwh: row.get("annual_consumption_kwh"),
            current_provider: row.get("current_provider"),
            ean_code: row.get("ean_code"),
            unsubscribed_at: row.get("unsubscribed_at"),
            created_at: row.get("created_at"),
        }))
    }

    async fn find_by_campaign(
        &self,
        campaign_id: Uuid,
        page: i64,
        per_page: i64,
    ) -> Result<Vec<IndividualMember>, String> {
        if page < 1 || per_page < 1 {
            return Err("Page and per_page must be >= 1".to_string());
        }

        let offset = (page - 1) * per_page;

        let rows = sqlx::query(
            r#"
            SELECT id, campaign_id, email, postal_code, has_gdpr_consent,
                   consent_at, annual_consumption_kwh, current_provider,
                   ean_code, unsubscribed_at, created_at
            FROM individual_members
            WHERE campaign_id = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(campaign_id)
        .bind(per_page)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(rows
            .iter()
            .map(|row| IndividualMember {
                id: row.get("id"),
                campaign_id: row.get("campaign_id"),
                email: row.get("email"),
                postal_code: row.get("postal_code"),
                has_gdpr_consent: row.get("has_gdpr_consent"),
                consent_at: row.get("consent_at"),
                annual_consumption_kwh: row.get("annual_consumption_kwh"),
                current_provider: row.get("current_provider"),
                ean_code: row.get("ean_code"),
                unsubscribed_at: row.get("unsubscribed_at"),
                created_at: row.get("created_at"),
            })
            .collect())
    }

    async fn find_by_email(&self, email: &str) -> Result<Option<IndividualMember>, String> {
        let row = sqlx::query(
            r#"
            SELECT id, campaign_id, email, postal_code, has_gdpr_consent,
                   consent_at, annual_consumption_kwh, current_provider,
                   ean_code, unsubscribed_at, created_at
            FROM individual_members
            WHERE email = $1
            "#,
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(row.map(|row| IndividualMember {
            id: row.get("id"),
            campaign_id: row.get("campaign_id"),
            email: row.get("email"),
            postal_code: row.get("postal_code"),
            has_gdpr_consent: row.get("has_gdpr_consent"),
            consent_at: row.get("consent_at"),
            annual_consumption_kwh: row.get("annual_consumption_kwh"),
            current_provider: row.get("current_provider"),
            ean_code: row.get("ean_code"),
            unsubscribed_at: row.get("unsubscribed_at"),
            created_at: row.get("created_at"),
        }))
    }

    async fn find_by_email_and_campaign(
        &self,
        email: &str,
        campaign_id: Uuid,
    ) -> Result<Option<IndividualMember>, String> {
        let row = sqlx::query(
            r#"
            SELECT id, campaign_id, email, postal_code, has_gdpr_consent,
                   consent_at, annual_consumption_kwh, current_provider,
                   ean_code, unsubscribed_at, created_at
            FROM individual_members
            WHERE email = $1 AND campaign_id = $2
            "#,
        )
        .bind(email)
        .bind(campaign_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(row.map(|row| IndividualMember {
            id: row.get("id"),
            campaign_id: row.get("campaign_id"),
            email: row.get("email"),
            postal_code: row.get("postal_code"),
            has_gdpr_consent: row.get("has_gdpr_consent"),
            consent_at: row.get("consent_at"),
            annual_consumption_kwh: row.get("annual_consumption_kwh"),
            current_provider: row.get("current_provider"),
            ean_code: row.get("ean_code"),
            unsubscribed_at: row.get("unsubscribed_at"),
            created_at: row.get("created_at"),
        }))
    }

    async fn find_active_by_campaign(
        &self,
        campaign_id: Uuid,
        page: i64,
        per_page: i64,
    ) -> Result<Vec<IndividualMember>, String> {
        if page < 1 || per_page < 1 {
            return Err("Page and per_page must be >= 1".to_string());
        }

        let offset = (page - 1) * per_page;

        let rows = sqlx::query(
            r#"
            SELECT id, campaign_id, email, postal_code, has_gdpr_consent,
                   consent_at, annual_consumption_kwh, current_provider,
                   ean_code, unsubscribed_at, created_at
            FROM individual_members
            WHERE campaign_id = $1 AND unsubscribed_at IS NULL
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(campaign_id)
        .bind(per_page)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(rows
            .iter()
            .map(|row| IndividualMember {
                id: row.get("id"),
                campaign_id: row.get("campaign_id"),
                email: row.get("email"),
                postal_code: row.get("postal_code"),
                has_gdpr_consent: row.get("has_gdpr_consent"),
                consent_at: row.get("consent_at"),
                annual_consumption_kwh: row.get("annual_consumption_kwh"),
                current_provider: row.get("current_provider"),
                ean_code: row.get("ean_code"),
                unsubscribed_at: row.get("unsubscribed_at"),
                created_at: row.get("created_at"),
            })
            .collect())
    }

    async fn find_with_consent_by_campaign(
        &self,
        campaign_id: Uuid,
        page: i64,
        per_page: i64,
    ) -> Result<Vec<IndividualMember>, String> {
        if page < 1 || per_page < 1 {
            return Err("Page and per_page must be >= 1".to_string());
        }

        let offset = (page - 1) * per_page;

        let rows = sqlx::query(
            r#"
            SELECT id, campaign_id, email, postal_code, has_gdpr_consent,
                   consent_at, annual_consumption_kwh, current_provider,
                   ean_code, unsubscribed_at, created_at
            FROM individual_members
            WHERE campaign_id = $1 AND has_gdpr_consent = TRUE AND unsubscribed_at IS NULL
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(campaign_id)
        .bind(per_page)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(rows
            .iter()
            .map(|row| IndividualMember {
                id: row.get("id"),
                campaign_id: row.get("campaign_id"),
                email: row.get("email"),
                postal_code: row.get("postal_code"),
                has_gdpr_consent: row.get("has_gdpr_consent"),
                consent_at: row.get("consent_at"),
                annual_consumption_kwh: row.get("annual_consumption_kwh"),
                current_provider: row.get("current_provider"),
                ean_code: row.get("ean_code"),
                unsubscribed_at: row.get("unsubscribed_at"),
                created_at: row.get("created_at"),
            })
            .collect())
    }

    async fn update(&self, member: &IndividualMember) -> Result<IndividualMember, String> {
        sqlx::query(
            r#"
            UPDATE individual_members
            SET email = $1,
                postal_code = $2,
                has_gdpr_consent = $3,
                consent_at = $4,
                annual_consumption_kwh = $5,
                current_provider = $6,
                ean_code = $7,
                unsubscribed_at = $8
            WHERE id = $9
            "#,
        )
        .bind(&member.email)
        .bind(&member.postal_code)
        .bind(member.has_gdpr_consent)
        .bind(member.consent_at)
        .bind(member.annual_consumption_kwh)
        .bind(&member.current_provider)
        .bind(&member.ean_code)
        .bind(member.unsubscribed_at)
        .bind(member.id)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Database error updating individual member: {}", e))?;

        Ok(member.clone())
    }

    async fn delete(&self, id: Uuid) -> Result<(), String> {
        sqlx::query("DELETE FROM individual_members WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("Database error deleting individual member: {}", e))?;

        Ok(())
    }

    async fn withdraw_consent(&self, id: Uuid) -> Result<(), String> {
        sqlx::query(
            r#"
            UPDATE individual_members
            SET email = CONCAT('withdrawn_', id::text, '@anonymized.invalid'),
                postal_code = 'ANONYMIZED',
                current_provider = NULL,
                ean_code = NULL,
                has_gdpr_consent = FALSE,
                unsubscribed_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Database error withdrawing consent: {}", e))?;

        Ok(())
    }

    async fn count_by_campaign(&self, campaign_id: Uuid) -> Result<i64, String> {
        let row = sqlx::query("SELECT COUNT(*) as count FROM individual_members WHERE campaign_id = $1")
            .bind(campaign_id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| format!("Database error: {}", e))?;

        Ok(row.get::<i64, _>("count"))
    }

    async fn count_active_by_campaign(&self, campaign_id: Uuid) -> Result<i64, String> {
        let row = sqlx::query(
            "SELECT COUNT(*) as count FROM individual_members WHERE campaign_id = $1 AND unsubscribed_at IS NULL",
        )
        .bind(campaign_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(row.get::<i64, _>("count"))
    }

    async fn count_with_consent_by_campaign(&self, campaign_id: Uuid) -> Result<i64, String> {
        let row = sqlx::query(
            "SELECT COUNT(*) as count FROM individual_members WHERE campaign_id = $1 AND has_gdpr_consent = TRUE AND unsubscribed_at IS NULL",
        )
        .bind(campaign_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(row.get::<i64, _>("count"))
    }
}
