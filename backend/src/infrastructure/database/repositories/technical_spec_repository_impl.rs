//! PostgreSQL implementation of [`TechnicalSpecRepository`] (Story 3.8 —
//! FR33).
//!
//! All `sqlx::Error` paths are wrapped in `AppError::Database(_)` — no
//! `Result<_, String>` debt (CRITICAL.md #4 / #555).
//!
//! Signatures are append-only: only [`save_signature`] writes new rows.
//! The DB trigger `tech_spec_sig_no_mutation` (cf. migration
//! `20260605060000`) blocks any UPDATE / DELETE at the SQL boundary.

use crate::application::error::AppError;
use crate::application::ports::TechnicalSpecRepository;
use crate::domain::entities::{
    SemVer, SignatoryRole, TechnicalSpec, TechnicalSpecSignature, TechnicalSpecStatus,
};
use crate::infrastructure::database::pool::DbPool;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::Row;
use std::str::FromStr;
use uuid::Uuid;

pub struct PostgresTechnicalSpecRepository {
    pool: DbPool,
}

impl PostgresTechnicalSpecRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    fn row_to_spec(row: &sqlx::postgres::PgRow) -> Result<TechnicalSpec, AppError> {
        let status_str: String = row.get("status");
        let status = TechnicalSpecStatus::from_str(&status_str)?;
        let required_str: Vec<String> = row.get("required_signatures");
        let required_signatures: Result<Vec<SignatoryRole>, AppError> = required_str
            .iter()
            .map(|r| SignatoryRole::from_str(r))
            .collect();

        let major: i32 = row.get("version_major");
        let minor: i32 = row.get("version_minor");
        let patch: i32 = row.get("version_patch");
        let version = SemVer::new(major as u32, minor as u32, patch as u32);

        Ok(TechnicalSpec {
            id: row.get("id"),
            acp_id: row.get("acp_id"),
            building_id: row.get("building_id"),
            title: row.get("title"),
            description: row.get("description"),
            version,
            status,
            deliverables: row.get("deliverables"),
            required_signatures: required_signatures?,
            attachments: row.get("attachments"),
            previous_version_id: row.get("previous_version_id"),
            created_by: row.get("created_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }

    fn row_to_signature(row: &sqlx::postgres::PgRow) -> Result<TechnicalSpecSignature, AppError> {
        let role_str: String = row.get("role");
        let role = SignatoryRole::from_str(&role_str)?;
        Ok(TechnicalSpecSignature {
            id: row.get("id"),
            technical_spec_id: row.get("technical_spec_id"),
            signatory_user_id: row.get("signatory_user_id"),
            role,
            mandate_id: row.get("mandate_id"),
            signed_at: row.get("signed_at"),
        })
    }
}

#[async_trait]
impl TechnicalSpecRepository for PostgresTechnicalSpecRepository {
    async fn save(&self, spec: &TechnicalSpec) -> Result<(), AppError> {
        let required_strs: Vec<String> = spec
            .required_signatures
            .iter()
            .map(|r| r.to_string())
            .collect();
        sqlx::query(
            r#"
            INSERT INTO technical_specs (
                id, acp_id, building_id, title, description,
                version_major, version_minor, version_patch,
                status, deliverables, required_signatures, attachments,
                previous_version_id, created_by, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
            "#,
        )
        .bind(spec.id)
        .bind(spec.acp_id)
        .bind(spec.building_id)
        .bind(&spec.title)
        .bind(&spec.description)
        .bind(spec.version.major as i32)
        .bind(spec.version.minor as i32)
        .bind(spec.version.patch as i32)
        .bind(spec.status.to_string())
        .bind(&spec.deliverables)
        .bind(&required_strs)
        .bind(&spec.attachments)
        .bind(spec.previous_version_id)
        .bind(spec.created_by)
        .bind(spec.created_at)
        .bind(spec.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(())
    }

    async fn update_status(
        &self,
        spec_id: Uuid,
        status: &str,
        updated_at: DateTime<Utc>,
    ) -> Result<(), AppError> {
        sqlx::query(
            r#"
            UPDATE technical_specs
            SET status = $2, updated_at = $3
            WHERE id = $1
            "#,
        )
        .bind(spec_id)
        .bind(status)
        .bind(updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<TechnicalSpec>, AppError> {
        let row = sqlx::query(
            r#"
            SELECT id, acp_id, building_id, title, description,
                   version_major, version_minor, version_patch,
                   status, deliverables, required_signatures, attachments,
                   previous_version_id, created_by, created_at, updated_at
            FROM technical_specs
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        match row {
            None => Ok(None),
            Some(row) => Ok(Some(Self::row_to_spec(&row)?)),
        }
    }

    async fn list_for_acp(&self, acp_id: Uuid) -> Result<Vec<TechnicalSpec>, AppError> {
        let rows = sqlx::query(
            r#"
            SELECT id, acp_id, building_id, title, description,
                   version_major, version_minor, version_patch,
                   status, deliverables, required_signatures, attachments,
                   previous_version_id, created_by, created_at, updated_at
            FROM technical_specs
            WHERE acp_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(acp_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        rows.iter().map(Self::row_to_spec).collect()
    }

    async fn save_signature(&self, sig: &TechnicalSpecSignature) -> Result<(), AppError> {
        let res = sqlx::query(
            r#"
            INSERT INTO technical_spec_signatures (
                id, technical_spec_id, signatory_user_id, role, mandate_id, signed_at
            )
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(sig.id)
        .bind(sig.technical_spec_id)
        .bind(sig.signatory_user_id)
        .bind(sig.role.to_string())
        .bind(sig.mandate_id)
        .bind(sig.signed_at)
        .execute(&self.pool)
        .await;

        match res {
            Ok(_) => Ok(()),
            Err(sqlx::Error::Database(db_err)) => {
                // PG unique_violation = 23505.
                if db_err.code().as_deref() == Some("23505") {
                    Err(AppError::SignatureAlreadyExists)
                } else {
                    Err(AppError::Database(db_err.to_string()))
                }
            }
            Err(e) => Err(AppError::Database(e.to_string())),
        }
    }

    async fn list_signatures_for_spec(
        &self,
        spec_id: Uuid,
    ) -> Result<Vec<TechnicalSpecSignature>, AppError> {
        let rows = sqlx::query(
            r#"
            SELECT id, technical_spec_id, signatory_user_id, role, mandate_id, signed_at
            FROM technical_spec_signatures
            WHERE technical_spec_id = $1
            ORDER BY signed_at ASC
            "#,
        )
        .bind(spec_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        rows.iter().map(Self::row_to_signature).collect()
    }
}
