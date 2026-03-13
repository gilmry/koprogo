use crate::application::ports::contractor_report_repository::ContractorReportRepository;
use crate::domain::entities::contractor_report::{
    ContractorReport, ContractorReportStatus, ReplacedPart,
};
use crate::infrastructure::database::pool::DbPool;
use async_trait::async_trait;
use sqlx::Row;
use uuid::Uuid;

pub struct PostgresContractorReportRepository {
    pool: DbPool,
}

impl PostgresContractorReportRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

fn row_to_report(row: &sqlx::postgres::PgRow) -> ContractorReport {
    let status_str: String = row.get("status");
    let status = ContractorReportStatus::from_db_string(&status_str)
        .unwrap_or(ContractorReportStatus::Draft);

    // Désérialiser les photos (UUID[])
    let photos_before: Vec<Uuid> = row.try_get("photos_before").unwrap_or_default();
    let photos_after: Vec<Uuid> = row.try_get("photos_after").unwrap_or_default();

    // Désérialiser les pièces remplacées (JSONB)
    let parts_json: serde_json::Value = row
        .try_get("parts_replaced")
        .unwrap_or(serde_json::json!([]));
    let parts_replaced: Vec<ReplacedPart> = serde_json::from_value(parts_json).unwrap_or_default();

    ContractorReport {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        building_id: row.get("building_id"),
        ticket_id: row.get("ticket_id"),
        quote_id: row.get("quote_id"),
        contractor_user_id: row.get("contractor_user_id"),
        contractor_name: row.get("contractor_name"),
        work_date: row.get("work_date"),
        compte_rendu: row.get("compte_rendu"),
        photos_before,
        photos_after,
        parts_replaced,
        status,
        magic_token_hash: row.get("magic_token_hash"),
        magic_token_expires_at: row.get("magic_token_expires_at"),
        submitted_at: row.get("submitted_at"),
        validated_at: row.get("validated_at"),
        validated_by: row.get("validated_by"),
        review_comments: row.get("review_comments"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

#[async_trait]
impl ContractorReportRepository for PostgresContractorReportRepository {
    async fn create(&self, report: &ContractorReport) -> Result<ContractorReport, String> {
        let parts_json =
            serde_json::to_value(&report.parts_replaced).unwrap_or(serde_json::json!([]));

        sqlx::query(
            r#"
            INSERT INTO contractor_reports (
                id, organization_id, building_id,
                ticket_id, quote_id, contractor_user_id, contractor_name,
                work_date, compte_rendu,
                photos_before, photos_after, parts_replaced,
                status, magic_token_hash, magic_token_expires_at,
                submitted_at, validated_at, validated_by, review_comments,
                created_at, updated_at
            ) VALUES (
                $1, $2, $3,
                $4, $5, $6, $7,
                $8, $9,
                $10, $11, $12,
                $13::contractor_report_status, $14, $15,
                $16, $17, $18, $19,
                $20, $21
            )
            "#,
        )
        .bind(report.id)
        .bind(report.organization_id)
        .bind(report.building_id)
        .bind(report.ticket_id)
        .bind(report.quote_id)
        .bind(report.contractor_user_id)
        .bind(&report.contractor_name)
        .bind(report.work_date)
        .bind(&report.compte_rendu)
        .bind(&report.photos_before)
        .bind(&report.photos_after)
        .bind(&parts_json)
        .bind(report.status.to_db_str())
        .bind(&report.magic_token_hash)
        .bind(report.magic_token_expires_at)
        .bind(report.submitted_at)
        .bind(report.validated_at)
        .bind(report.validated_by)
        .bind(&report.review_comments)
        .bind(report.created_at)
        .bind(report.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Erreur création rapport: {}", e))?;

        self.find_by_id(report.id)
            .await?
            .ok_or_else(|| "Rapport créé introuvable".to_string())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<ContractorReport>, String> {
        let row = sqlx::query(
            r#"
            SELECT id, organization_id, building_id,
                   ticket_id, quote_id, contractor_user_id, contractor_name,
                   work_date, compte_rendu,
                   photos_before, photos_after, parts_replaced,
                   status::TEXT, magic_token_hash, magic_token_expires_at,
                   submitted_at, validated_at, validated_by, review_comments,
                   created_at, updated_at
            FROM contractor_reports
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Erreur find_by_id: {}", e))?;

        Ok(row.as_ref().map(row_to_report))
    }

    async fn find_by_magic_token(
        &self,
        token_hash: &str,
    ) -> Result<Option<ContractorReport>, String> {
        let row = sqlx::query(
            r#"
            SELECT id, organization_id, building_id,
                   ticket_id, quote_id, contractor_user_id, contractor_name,
                   work_date, compte_rendu,
                   photos_before, photos_after, parts_replaced,
                   status::TEXT, magic_token_hash, magic_token_expires_at,
                   submitted_at, validated_at, validated_by, review_comments,
                   created_at, updated_at
            FROM contractor_reports
            WHERE magic_token_hash = $1
            "#,
        )
        .bind(token_hash)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Erreur find_by_magic_token: {}", e))?;

        Ok(row.as_ref().map(row_to_report))
    }

    async fn find_by_ticket(&self, ticket_id: Uuid) -> Result<Vec<ContractorReport>, String> {
        let rows = sqlx::query(
            r#"
            SELECT id, organization_id, building_id,
                   ticket_id, quote_id, contractor_user_id, contractor_name,
                   work_date, compte_rendu,
                   photos_before, photos_after, parts_replaced,
                   status::TEXT, magic_token_hash, magic_token_expires_at,
                   submitted_at, validated_at, validated_by, review_comments,
                   created_at, updated_at
            FROM contractor_reports
            WHERE ticket_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(ticket_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Erreur find_by_ticket: {}", e))?;

        Ok(rows.iter().map(row_to_report).collect())
    }

    async fn find_by_quote(&self, quote_id: Uuid) -> Result<Vec<ContractorReport>, String> {
        let rows = sqlx::query(
            r#"
            SELECT id, organization_id, building_id,
                   ticket_id, quote_id, contractor_user_id, contractor_name,
                   work_date, compte_rendu,
                   photos_before, photos_after, parts_replaced,
                   status::TEXT, magic_token_hash, magic_token_expires_at,
                   submitted_at, validated_at, validated_by, review_comments,
                   created_at, updated_at
            FROM contractor_reports
            WHERE quote_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(quote_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Erreur find_by_quote: {}", e))?;

        Ok(rows.iter().map(row_to_report).collect())
    }

    async fn find_by_building(&self, building_id: Uuid) -> Result<Vec<ContractorReport>, String> {
        let rows = sqlx::query(
            r#"
            SELECT id, organization_id, building_id,
                   ticket_id, quote_id, contractor_user_id, contractor_name,
                   work_date, compte_rendu,
                   photos_before, photos_after, parts_replaced,
                   status::TEXT, magic_token_hash, magic_token_expires_at,
                   submitted_at, validated_at, validated_by, review_comments,
                   created_at, updated_at
            FROM contractor_reports
            WHERE building_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(building_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Erreur find_by_building: {}", e))?;

        Ok(rows.iter().map(row_to_report).collect())
    }

    async fn find_by_organization(
        &self,
        organization_id: Uuid,
    ) -> Result<Vec<ContractorReport>, String> {
        let rows = sqlx::query(
            r#"
            SELECT id, organization_id, building_id,
                   ticket_id, quote_id, contractor_user_id, contractor_name,
                   work_date, compte_rendu,
                   photos_before, photos_after, parts_replaced,
                   status::TEXT, magic_token_hash, magic_token_expires_at,
                   submitted_at, validated_at, validated_by, review_comments,
                   created_at, updated_at
            FROM contractor_reports
            WHERE organization_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(organization_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Erreur find_by_organization: {}", e))?;

        Ok(rows.iter().map(row_to_report).collect())
    }

    async fn update(&self, report: &ContractorReport) -> Result<ContractorReport, String> {
        let parts_json =
            serde_json::to_value(&report.parts_replaced).unwrap_or(serde_json::json!([]));

        sqlx::query(
            r#"
            UPDATE contractor_reports SET
                work_date = $1,
                compte_rendu = $2,
                photos_before = $3,
                photos_after = $4,
                parts_replaced = $5,
                status = $6::contractor_report_status,
                magic_token_hash = $7,
                magic_token_expires_at = $8,
                submitted_at = $9,
                validated_at = $10,
                validated_by = $11,
                review_comments = $12,
                updated_at = NOW()
            WHERE id = $13
            "#,
        )
        .bind(report.work_date)
        .bind(&report.compte_rendu)
        .bind(&report.photos_before)
        .bind(&report.photos_after)
        .bind(&parts_json)
        .bind(report.status.to_db_str())
        .bind(&report.magic_token_hash)
        .bind(report.magic_token_expires_at)
        .bind(report.submitted_at)
        .bind(report.validated_at)
        .bind(report.validated_by)
        .bind(&report.review_comments)
        .bind(report.id)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Erreur update: {}", e))?;

        self.find_by_id(report.id)
            .await?
            .ok_or_else(|| "Rapport mis à jour introuvable".to_string())
    }

    async fn delete(&self, id: Uuid) -> Result<bool, String> {
        let result =
            sqlx::query(r#"DELETE FROM contractor_reports WHERE id = $1 AND status = 'draft'"#)
                .bind(id)
                .execute(&self.pool)
                .await
                .map_err(|e| format!("Erreur delete: {}", e))?;

        Ok(result.rows_affected() > 0)
    }
}
