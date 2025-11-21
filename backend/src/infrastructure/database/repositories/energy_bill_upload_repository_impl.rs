use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::application::ports::EnergyBillUploadRepository;
use crate::domain::entities::{EnergyBillUpload, EnergyType};

pub struct PostgresEnergyBillUploadRepository {
    pub pool: PgPool,
}

impl PostgresEnergyBillUploadRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl EnergyBillUploadRepository for PostgresEnergyBillUploadRepository {
    async fn create(&self, upload: &EnergyBillUpload) -> Result<EnergyBillUpload, String> {
        sqlx::query!(
            r#"
            INSERT INTO energy_bill_uploads (
                id, campaign_id, unit_id, building_id, organization_id,
                bill_period_start, bill_period_end, total_kwh_encrypted, energy_type, provider, postal_code,
                file_hash, file_path_encrypted, ocr_confidence, manually_verified,
                uploaded_by, uploaded_at, verified_at, verified_by,
                consent_timestamp, consent_ip, consent_user_agent, consent_signature_hash,
                anonymized, retention_until, deleted_at, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23, $24, $25, $26, $27, $28)
            "#,
            upload.id,
            upload.campaign_id,
            upload.unit_id,
            upload.building_id,
            upload.organization_id,
            upload.bill_period_start,
            upload.bill_period_end,
            &upload.total_kwh_encrypted,
            upload.energy_type.to_string(),
            upload.provider.as_ref(),
            upload.postal_code,
            upload.file_hash,
            upload.file_path_encrypted,
            upload.ocr_confidence,
            upload.manually_verified,
            upload.uploaded_by,
            upload.uploaded_at,
            upload.verified_at,
            upload.verified_by,
            upload.consent_timestamp,
            upload.consent_ip,
            upload.consent_user_agent,
            upload.consent_signature_hash,
            upload.anonymized,
            upload.retention_until,
            upload.deleted_at,
            upload.created_at,
            upload.updated_at,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to create energy bill upload: {}", e))?;

        Ok(upload.clone())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<EnergyBillUpload>, String> {
        let row = sqlx::query!(
            r#"
            SELECT
                id, campaign_id, unit_id, building_id, organization_id,
                bill_period_start, bill_period_end, total_kwh_encrypted, energy_type, provider, postal_code,
                file_hash, file_path_encrypted, ocr_confidence, manually_verified,
                uploaded_by, uploaded_at, verified_at, verified_by,
                consent_timestamp, consent_ip, consent_user_agent, consent_signature_hash,
                anonymized, retention_until, deleted_at, created_at, updated_at
            FROM energy_bill_uploads
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Failed to find energy bill upload: {}", e))?;

        Ok(row.map(|r| EnergyBillUpload {
            id: r.id,
            campaign_id: r.campaign_id,
            unit_id: r.unit_id,
            building_id: r.building_id,
            organization_id: r.organization_id,
            bill_period_start: r.bill_period_start,
            bill_period_end: r.bill_period_end,
            total_kwh_encrypted: r.total_kwh_encrypted,
            energy_type: r.energy_type.parse().unwrap_or(EnergyType::Electricity),
            provider: r.provider,
            postal_code: r.postal_code,
            file_hash: r.file_hash,
            file_path_encrypted: r.file_path_encrypted,
            ocr_confidence: r.ocr_confidence,
            manually_verified: r.manually_verified,
            uploaded_by: r.uploaded_by,
            uploaded_at: r.uploaded_at,
            verified_at: r.verified_at,
            verified_by: r.verified_by,
            consent_timestamp: r.consent_timestamp,
            consent_ip: r.consent_ip,
            consent_user_agent: r.consent_user_agent,
            consent_signature_hash: r.consent_signature_hash,
            anonymized: r.anonymized,
            retention_until: r.retention_until,
            deleted_at: r.deleted_at,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }))
    }

    async fn find_by_campaign(&self, campaign_id: Uuid) -> Result<Vec<EnergyBillUpload>, String> {
        let rows = sqlx::query!(
            r#"
            SELECT
                id, campaign_id, unit_id, building_id, organization_id,
                bill_period_start, bill_period_end, total_kwh_encrypted, energy_type, provider, postal_code,
                file_hash, file_path_encrypted, ocr_confidence, manually_verified,
                uploaded_by, uploaded_at, verified_at, verified_by,
                consent_timestamp, consent_ip, consent_user_agent, consent_signature_hash,
                anonymized, retention_until, deleted_at, created_at, updated_at
            FROM energy_bill_uploads
            WHERE campaign_id = $1
            ORDER BY uploaded_at DESC
            "#,
            campaign_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to find uploads by campaign: {}", e))?;

        let uploads = rows
            .into_iter()
            .map(|r| EnergyBillUpload {
                id: r.id,
                campaign_id: r.campaign_id,
                unit_id: r.unit_id,
                building_id: r.building_id,
                organization_id: r.organization_id,
                bill_period_start: r.bill_period_start,
                bill_period_end: r.bill_period_end,
                total_kwh_encrypted: r.total_kwh_encrypted,
                energy_type: r.energy_type.parse().unwrap_or(EnergyType::Electricity),
                provider: r.provider,
                postal_code: r.postal_code,
                file_hash: r.file_hash,
                file_path_encrypted: r.file_path_encrypted,
                ocr_confidence: r.ocr_confidence,
                manually_verified: r.manually_verified,
                uploaded_by: r.uploaded_by,
                uploaded_at: r.uploaded_at,
                verified_at: r.verified_at,
                verified_by: r.verified_by,
                consent_timestamp: r.consent_timestamp,
                consent_ip: r.consent_ip,
                consent_user_agent: r.consent_user_agent,
                consent_signature_hash: r.consent_signature_hash,
                anonymized: r.anonymized,
                retention_until: r.retention_until,
                deleted_at: r.deleted_at,
                created_at: r.created_at,
                updated_at: r.updated_at,
            })
            .collect();

        Ok(uploads)
    }

    async fn find_by_unit(&self, unit_id: Uuid) -> Result<Vec<EnergyBillUpload>, String> {
        let rows = sqlx::query!(
            r#"
            SELECT
                id, campaign_id, unit_id, building_id, organization_id,
                bill_period_start, bill_period_end, total_kwh_encrypted, energy_type, provider, postal_code,
                file_hash, file_path_encrypted, ocr_confidence, manually_verified,
                uploaded_by, uploaded_at, verified_at, verified_by,
                consent_timestamp, consent_ip, consent_user_agent, consent_signature_hash,
                anonymized, retention_until, deleted_at, created_at, updated_at
            FROM energy_bill_uploads
            WHERE unit_id = $1
            ORDER BY uploaded_at DESC
            "#,
            unit_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to find uploads by unit: {}", e))?;

        let uploads = rows
            .into_iter()
            .map(|r| EnergyBillUpload {
                id: r.id,
                campaign_id: r.campaign_id,
                unit_id: r.unit_id,
                building_id: r.building_id,
                organization_id: r.organization_id,
                bill_period_start: r.bill_period_start,
                bill_period_end: r.bill_period_end,
                total_kwh_encrypted: r.total_kwh_encrypted,
                energy_type: r.energy_type.parse().unwrap_or(EnergyType::Electricity),
                provider: r.provider,
                postal_code: r.postal_code,
                file_hash: r.file_hash,
                file_path_encrypted: r.file_path_encrypted,
                ocr_confidence: r.ocr_confidence,
                manually_verified: r.manually_verified,
                uploaded_by: r.uploaded_by,
                uploaded_at: r.uploaded_at,
                verified_at: r.verified_at,
                verified_by: r.verified_by,
                consent_timestamp: r.consent_timestamp,
                consent_ip: r.consent_ip,
                consent_user_agent: r.consent_user_agent,
                consent_signature_hash: r.consent_signature_hash,
                anonymized: r.anonymized,
                retention_until: r.retention_until,
                deleted_at: r.deleted_at,
                created_at: r.created_at,
                updated_at: r.updated_at,
            })
            .collect();

        Ok(uploads)
    }

    async fn find_by_campaign_and_unit(
        &self,
        campaign_id: Uuid,
        unit_id: Uuid,
    ) -> Result<Option<EnergyBillUpload>, String> {
        let row = sqlx::query!(
            r#"
            SELECT
                id, campaign_id, unit_id, building_id, organization_id,
                bill_period_start, bill_period_end, total_kwh_encrypted, energy_type, provider, postal_code,
                file_hash, file_path_encrypted, ocr_confidence, manually_verified,
                uploaded_by, uploaded_at, verified_at, verified_by,
                consent_timestamp, consent_ip, consent_user_agent, consent_signature_hash,
                anonymized, retention_until, deleted_at, created_at, updated_at
            FROM energy_bill_uploads
            WHERE campaign_id = $1 AND unit_id = $2
            "#,
            campaign_id,
            unit_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Failed to find upload by campaign and unit: {}", e))?;

        Ok(row.map(|r| EnergyBillUpload {
            id: r.id,
            campaign_id: r.campaign_id,
            unit_id: r.unit_id,
            building_id: r.building_id,
            organization_id: r.organization_id,
            bill_period_start: r.bill_period_start,
            bill_period_end: r.bill_period_end,
            total_kwh_encrypted: r.total_kwh_encrypted,
            energy_type: r.energy_type.parse().unwrap_or(EnergyType::Electricity),
            provider: r.provider,
            postal_code: r.postal_code,
            file_hash: r.file_hash,
            file_path_encrypted: r.file_path_encrypted,
            ocr_confidence: r.ocr_confidence,
            manually_verified: r.manually_verified,
            uploaded_by: r.uploaded_by,
            uploaded_at: r.uploaded_at,
            verified_at: r.verified_at,
            verified_by: r.verified_by,
            consent_timestamp: r.consent_timestamp,
            consent_ip: r.consent_ip,
            consent_user_agent: r.consent_user_agent,
            consent_signature_hash: r.consent_signature_hash,
            anonymized: r.anonymized,
            retention_until: r.retention_until,
            deleted_at: r.deleted_at,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }))
    }

    async fn find_by_building(
        &self,
        building_id: Uuid,
    ) -> Result<Vec<EnergyBillUpload>, String> {
        let rows = sqlx::query!(
            r#"
            SELECT
                id, campaign_id, unit_id, building_id, organization_id,
                bill_period_start, bill_period_end, total_kwh_encrypted, energy_type, provider, postal_code,
                file_hash, file_path_encrypted, ocr_confidence, manually_verified,
                uploaded_by, uploaded_at, verified_at, verified_by,
                consent_timestamp, consent_ip, consent_user_agent, consent_signature_hash,
                anonymized, retention_until, deleted_at, created_at, updated_at
            FROM energy_bill_uploads
            WHERE building_id = $1
            ORDER BY uploaded_at DESC
            "#,
            building_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to find uploads by building: {}", e))?;

        let uploads = rows
            .into_iter()
            .map(|r| EnergyBillUpload {
                id: r.id,
                campaign_id: r.campaign_id,
                unit_id: r.unit_id,
                building_id: r.building_id,
                organization_id: r.organization_id,
                bill_period_start: r.bill_period_start,
                bill_period_end: r.bill_period_end,
                total_kwh_encrypted: r.total_kwh_encrypted,
                energy_type: r.energy_type.parse().unwrap_or(EnergyType::Electricity),
                provider: r.provider,
                postal_code: r.postal_code,
                file_hash: r.file_hash,
                file_path_encrypted: r.file_path_encrypted,
                ocr_confidence: r.ocr_confidence,
                manually_verified: r.manually_verified,
                uploaded_by: r.uploaded_by,
                uploaded_at: r.uploaded_at,
                verified_at: r.verified_at,
                verified_by: r.verified_by,
                consent_timestamp: r.consent_timestamp,
                consent_ip: r.consent_ip,
                consent_user_agent: r.consent_user_agent,
                consent_signature_hash: r.consent_signature_hash,
                anonymized: r.anonymized,
                retention_until: r.retention_until,
                deleted_at: r.deleted_at,
                created_at: r.created_at,
                updated_at: r.updated_at,
            })
            .collect();

        Ok(uploads)
    }

    async fn update(&self, upload: &EnergyBillUpload) -> Result<EnergyBillUpload, String> {
        sqlx::query!(
            r#"
            UPDATE energy_bill_uploads
            SET
                bill_period_start = $2,
                bill_period_end = $3,
                total_kwh_encrypted = $4,
                energy_type = $5,
                provider = $6,
                postal_code = $7,
                file_hash = $8,
                file_path_encrypted = $9,
                ocr_confidence = $10,
                manually_verified = $11,
                verified_at = $12,
                verified_by = $13,
                consent_timestamp = $14,
                consent_ip = $15,
                consent_user_agent = $16,
                consent_signature_hash = $17,
                anonymized = $18,
                retention_until = $19,
                deleted_at = $20,
                updated_at = $21
            WHERE id = $1
            "#,
            upload.id,
            upload.bill_period_start,
            upload.bill_period_end,
            &upload.total_kwh_encrypted,
            upload.energy_type.to_string(),
            upload.provider.as_ref(),
            upload.postal_code,
            upload.file_hash,
            upload.file_path_encrypted,
            upload.ocr_confidence,
            upload.manually_verified,
            upload.verified_at,
            upload.verified_by,
            upload.consent_timestamp,
            upload.consent_ip,
            upload.consent_user_agent,
            upload.consent_signature_hash,
            upload.anonymized,
            upload.retention_until,
            upload.deleted_at,
            upload.updated_at,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to update energy bill upload: {}", e))?;

        Ok(upload.clone())
    }

    async fn delete(&self, id: Uuid) -> Result<(), String> {
        // Soft delete
        sqlx::query!(
            r#"
            UPDATE energy_bill_uploads
            SET deleted_at = NOW(), updated_at = NOW()
            WHERE id = $1
            "#,
            id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to delete energy bill upload: {}", e))?;

        Ok(())
    }

    async fn find_expired(&self) -> Result<Vec<EnergyBillUpload>, String> {
        let rows = sqlx::query!(
            r#"
            SELECT
                id, campaign_id, unit_id, building_id, organization_id,
                bill_period_start, bill_period_end, total_kwh_encrypted, energy_type, provider, postal_code,
                file_hash, file_path_encrypted, ocr_confidence, manually_verified,
                uploaded_by, uploaded_at, verified_at, verified_by,
                consent_timestamp, consent_ip, consent_user_agent, consent_signature_hash,
                anonymized, retention_until, deleted_at, created_at, updated_at
            FROM energy_bill_uploads
            WHERE retention_until < NOW() AND deleted_at IS NULL
            "#
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to find expired uploads: {}", e))?;

        let uploads = rows
            .into_iter()
            .map(|r| EnergyBillUpload {
                id: r.id,
                campaign_id: r.campaign_id,
                unit_id: r.unit_id,
                building_id: r.building_id,
                organization_id: r.organization_id,
                bill_period_start: r.bill_period_start,
                bill_period_end: r.bill_period_end,
                total_kwh_encrypted: r.total_kwh_encrypted,
                energy_type: r.energy_type.parse().unwrap_or(EnergyType::Electricity),
                provider: r.provider,
                postal_code: r.postal_code,
                file_hash: r.file_hash,
                file_path_encrypted: r.file_path_encrypted,
                ocr_confidence: r.ocr_confidence,
                manually_verified: r.manually_verified,
                uploaded_by: r.uploaded_by,
                uploaded_at: r.uploaded_at,
                verified_at: r.verified_at,
                verified_by: r.verified_by,
                consent_timestamp: r.consent_timestamp,
                consent_ip: r.consent_ip,
                consent_user_agent: r.consent_user_agent,
                consent_signature_hash: r.consent_signature_hash,
                anonymized: r.anonymized,
                retention_until: r.retention_until,
                deleted_at: r.deleted_at,
                created_at: r.created_at,
                updated_at: r.updated_at,
            })
            .collect();

        Ok(uploads)
    }

    async fn count_verified_by_campaign(&self, campaign_id: Uuid) -> Result<i32, String> {
        let result = sqlx::query!(
            r#"
            SELECT COUNT(*) as count
            FROM energy_bill_uploads
            WHERE campaign_id = $1
            AND manually_verified = TRUE
            AND deleted_at IS NULL
            "#,
            campaign_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to count verified uploads: {}", e))?;

        Ok(result.count.unwrap_or(0) as i32)
    }

    async fn find_verified_by_campaign(
        &self,
        campaign_id: Uuid,
    ) -> Result<Vec<EnergyBillUpload>, String> {
        let rows = sqlx::query!(
            r#"
            SELECT
                id, campaign_id, unit_id, building_id, organization_id,
                bill_period_start, bill_period_end, total_kwh_encrypted, energy_type, provider, postal_code,
                file_hash, file_path_encrypted, ocr_confidence, manually_verified,
                uploaded_by, uploaded_at, verified_at, verified_by,
                consent_timestamp, consent_ip, consent_user_agent, consent_signature_hash,
                anonymized, retention_until, deleted_at, created_at, updated_at
            FROM energy_bill_uploads
            WHERE campaign_id = $1
            AND manually_verified = TRUE
            AND deleted_at IS NULL
            ORDER BY uploaded_at DESC
            "#,
            campaign_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to find verified uploads: {}", e))?;

        let uploads = rows
            .into_iter()
            .map(|r| EnergyBillUpload {
                id: r.id,
                campaign_id: r.campaign_id,
                unit_id: r.unit_id,
                building_id: r.building_id,
                organization_id: r.organization_id,
                bill_period_start: r.bill_period_start,
                bill_period_end: r.bill_period_end,
                total_kwh_encrypted: r.total_kwh_encrypted,
                energy_type: r.energy_type.parse().unwrap_or(EnergyType::Electricity),
                provider: r.provider,
                postal_code: r.postal_code,
                file_hash: r.file_hash,
                file_path_encrypted: r.file_path_encrypted,
                ocr_confidence: r.ocr_confidence,
                manually_verified: r.manually_verified,
                uploaded_by: r.uploaded_by,
                uploaded_at: r.uploaded_at,
                verified_at: r.verified_at,
                verified_by: r.verified_by,
                consent_timestamp: r.consent_timestamp,
                consent_ip: r.consent_ip,
                consent_user_agent: r.consent_user_agent,
                consent_signature_hash: r.consent_signature_hash,
                anonymized: r.anonymized,
                retention_until: r.retention_until,
                deleted_at: r.deleted_at,
                created_at: r.created_at,
                updated_at: r.updated_at,
            })
            .collect();

        Ok(uploads)
    }

    async fn delete_expired(&self) -> Result<i32, String> {
        let result = sqlx::query!(
            r#"
            UPDATE energy_bill_uploads
            SET deleted_at = NOW(), updated_at = NOW()
            WHERE retention_until < NOW()
            AND deleted_at IS NULL
            "#
        )
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to delete expired uploads: {}", e))?;

        Ok(result.rows_affected() as i32)
    }
}
