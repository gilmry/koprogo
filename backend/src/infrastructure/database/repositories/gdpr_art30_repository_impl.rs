use crate::application::ports::gdpr_art30_repository::GdprArt30Repository;
use crate::domain::entities::gdpr_art30::{ProcessingActivity, ProcessorAgreement};
use crate::infrastructure::database::pool::DbPool;
use async_trait::async_trait;
use sqlx::Row;

pub struct PostgresGdprArt30Repository {
    pool: DbPool,
}

impl PostgresGdprArt30Repository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl GdprArt30Repository for PostgresGdprArt30Repository {
    async fn list_processing_activities(&self) -> Result<Vec<ProcessingActivity>, String> {
        let rows = sqlx::query(
            r#"
            SELECT id, activity_name, controller_name, purpose, legal_basis,
                   data_categories, data_subjects, recipients,
                   retention_period, security_measures, created_at, updated_at
            FROM data_processing_activities
            ORDER BY created_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to fetch processing activities: {}", e))?;

        Ok(rows
            .iter()
            .map(|row| ProcessingActivity {
                id: row.get("id"),
                activity_name: row.get("activity_name"),
                controller_name: row.get("controller_name"),
                purpose: row.get("purpose"),
                legal_basis: row.get("legal_basis"),
                data_categories: row
                    .try_get::<Option<Vec<String>>, _>("data_categories")
                    .ok()
                    .flatten()
                    .unwrap_or_default(),
                data_subjects: row
                    .try_get::<Option<Vec<String>>, _>("data_subjects")
                    .ok()
                    .flatten()
                    .unwrap_or_default(),
                recipients: row
                    .try_get::<Option<Vec<String>>, _>("recipients")
                    .ok()
                    .flatten()
                    .unwrap_or_default(),
                retention_period: row.get("retention_period"),
                security_measures: row.get("security_measures"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            })
            .collect())
    }

    async fn list_processor_agreements(&self) -> Result<Vec<ProcessorAgreement>, String> {
        let rows = sqlx::query(
            r#"
            SELECT id, processor_name, service_description,
                   dpa_signed_at, dpa_url, transfer_mechanism,
                   data_categories, certifications, created_at, updated_at
            FROM data_processor_agreements
            ORDER BY processor_name ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to fetch processor agreements: {}", e))?;

        Ok(rows
            .iter()
            .map(|row| ProcessorAgreement {
                id: row.get("id"),
                processor_name: row.get("processor_name"),
                service_description: row.get("service_description"),
                dpa_signed_at: row.try_get("dpa_signed_at").ok().flatten(),
                dpa_url: row.try_get("dpa_url").ok().flatten(),
                transfer_mechanism: row.try_get("transfer_mechanism").ok().flatten(),
                data_categories: row
                    .try_get::<Option<Vec<String>>, _>("data_categories")
                    .ok()
                    .flatten()
                    .unwrap_or_default(),
                certifications: row.try_get("certifications").ok().flatten(),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            })
            .collect())
    }
}
