use crate::application::ports::QuoteRepository;
use crate::domain::entities::{Quote, QuoteStatus};
use crate::infrastructure::database::pool::DbPool;
use async_trait::async_trait;
use sqlx::Row;
use uuid::Uuid;

pub struct PostgresQuoteRepository {
    pool: DbPool,
}

/// Quote SELECT columns with cast for status enum
const QUOTE_COLUMNS: &str = r#"
    id, building_id, contractor_id, project_title, project_description,
    amount_excl_vat, vat_rate, amount_incl_vat, validity_date,
    estimated_start_date, estimated_duration_days, warranty_years,
    contractor_rating, status::text as status_text, requested_at, submitted_at,
    reviewed_at, decision_at, decision_by, decision_notes,
    created_at, updated_at
"#;

impl PostgresQuoteRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl QuoteRepository for PostgresQuoteRepository {
    async fn create(&self, quote: &Quote) -> Result<Quote, String> {
        sqlx::query(
            r#"
            INSERT INTO quotes (
                id, building_id, contractor_id, project_title, project_description,
                amount_excl_vat, vat_rate, amount_incl_vat, validity_date,
                estimated_start_date, estimated_duration_days, warranty_years,
                contractor_rating, status, requested_at, submitted_at,
                reviewed_at, decision_at, decision_by, decision_notes,
                created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14::quote_status, $15, $16, $17, $18, $19, $20, $21, $22)
            "#,
        )
        .bind(quote.id)
        .bind(quote.building_id)
        .bind(quote.contractor_id)
        .bind(&quote.project_title)
        .bind(&quote.project_description)
        .bind(quote.amount_excl_vat)
        .bind(quote.vat_rate)
        .bind(quote.amount_incl_vat)
        .bind(quote.validity_date)
        .bind(quote.estimated_start_date)
        .bind(quote.estimated_duration_days)
        .bind(quote.warranty_years)
        .bind(quote.contractor_rating)
        .bind(quote.status.to_sql())
        .bind(quote.requested_at)
        .bind(quote.submitted_at)
        .bind(quote.reviewed_at)
        .bind(quote.decision_at)
        .bind(quote.decision_by)
        .bind(&quote.decision_notes)
        .bind(quote.created_at)
        .bind(quote.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(quote.clone())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Quote>, String> {
        let sql = format!("SELECT {} FROM quotes WHERE id = $1", QUOTE_COLUMNS);
        let row = sqlx::query(&sql)
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| format!("Database error: {}", e))?;

        Ok(row.map(|row| map_row_to_quote(&row)))
    }

    async fn find_by_building(&self, building_id: Uuid) -> Result<Vec<Quote>, String> {
        let sql = format!(
            "SELECT {} FROM quotes WHERE building_id = $1 ORDER BY requested_at DESC",
            QUOTE_COLUMNS
        );
        let rows = sqlx::query(&sql)
            .bind(building_id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| format!("Database error: {}", e))?;

        Ok(rows.iter().map(map_row_to_quote).collect())
    }

    async fn find_by_contractor(&self, contractor_id: Uuid) -> Result<Vec<Quote>, String> {
        let sql = format!(
            "SELECT {} FROM quotes WHERE contractor_id = $1 ORDER BY requested_at DESC",
            QUOTE_COLUMNS
        );
        let rows = sqlx::query(&sql)
            .bind(contractor_id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| format!("Database error: {}", e))?;

        Ok(rows.iter().map(map_row_to_quote).collect())
    }

    async fn find_by_status(&self, building_id: Uuid, status: &str) -> Result<Vec<Quote>, String> {
        let sql = format!(
            "SELECT {} FROM quotes WHERE building_id = $1 AND status = $2::quote_status ORDER BY requested_at DESC",
            QUOTE_COLUMNS
        );
        let rows = sqlx::query(&sql)
            .bind(building_id)
            .bind(status)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| format!("Database error: {}", e))?;

        Ok(rows.iter().map(map_row_to_quote).collect())
    }

    async fn find_by_ids(&self, ids: Vec<Uuid>) -> Result<Vec<Quote>, String> {
        if ids.is_empty() {
            return Ok(vec![]);
        }

        let sql = format!(
            "SELECT {} FROM quotes WHERE id = ANY($1) ORDER BY amount_incl_vat ASC",
            QUOTE_COLUMNS
        );
        let rows = sqlx::query(&sql)
            .bind(&ids)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| format!("Database error: {}", e))?;

        Ok(rows.iter().map(map_row_to_quote).collect())
    }

    async fn find_by_project_title(
        &self,
        building_id: Uuid,
        project_title: &str,
    ) -> Result<Vec<Quote>, String> {
        let sql = format!(
            "SELECT {} FROM quotes WHERE building_id = $1 AND project_title ILIKE $2 ORDER BY requested_at DESC",
            QUOTE_COLUMNS
        );
        let rows = sqlx::query(&sql)
            .bind(building_id)
            .bind(format!("%{}%", project_title))
            .fetch_all(&self.pool)
            .await
            .map_err(|e| format!("Database error: {}", e))?;

        Ok(rows.iter().map(map_row_to_quote).collect())
    }

    async fn find_expired(&self) -> Result<Vec<Quote>, String> {
        let sql = format!(
            "SELECT {} FROM quotes WHERE validity_date < NOW() AND status::text NOT IN ('Accepted', 'Rejected', 'Expired', 'Withdrawn') ORDER BY validity_date ASC",
            QUOTE_COLUMNS
        );
        let rows = sqlx::query(&sql)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| format!("Database error: {}", e))?;

        Ok(rows.iter().map(map_row_to_quote).collect())
    }

    async fn update(&self, quote: &Quote) -> Result<Quote, String> {
        sqlx::query(
            r#"
            UPDATE quotes
            SET
                building_id = $2,
                contractor_id = $3,
                project_title = $4,
                project_description = $5,
                amount_excl_vat = $6,
                vat_rate = $7,
                amount_incl_vat = $8,
                validity_date = $9,
                estimated_start_date = $10,
                estimated_duration_days = $11,
                warranty_years = $12,
                contractor_rating = $13,
                status = $14::quote_status,
                requested_at = $15,
                submitted_at = $16,
                reviewed_at = $17,
                decision_at = $18,
                decision_by = $19,
                decision_notes = $20,
                updated_at = $21
            WHERE id = $1
            "#,
        )
        .bind(quote.id)
        .bind(quote.building_id)
        .bind(quote.contractor_id)
        .bind(&quote.project_title)
        .bind(&quote.project_description)
        .bind(quote.amount_excl_vat)
        .bind(quote.vat_rate)
        .bind(quote.amount_incl_vat)
        .bind(quote.validity_date)
        .bind(quote.estimated_start_date)
        .bind(quote.estimated_duration_days)
        .bind(quote.warranty_years)
        .bind(quote.contractor_rating)
        .bind(quote.status.to_sql())
        .bind(quote.requested_at)
        .bind(quote.submitted_at)
        .bind(quote.reviewed_at)
        .bind(quote.decision_at)
        .bind(quote.decision_by)
        .bind(&quote.decision_notes)
        .bind(quote.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(quote.clone())
    }

    async fn delete(&self, id: Uuid) -> Result<bool, String> {
        let result = sqlx::query("DELETE FROM quotes WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("Database error: {}", e))?;

        Ok(result.rows_affected() > 0)
    }

    async fn count_by_building(&self, building_id: Uuid) -> Result<i64, String> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM quotes WHERE building_id = $1")
            .bind(building_id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| format!("Database error: {}", e))?;

        Ok(count)
    }

    async fn count_by_status(&self, building_id: Uuid, status: &str) -> Result<i64, String> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM quotes WHERE building_id = $1 AND status = $2::quote_status",
        )
        .bind(building_id)
        .bind(status)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(count)
    }
}

/// Helper function to map PostgreSQL row to Quote entity
fn map_row_to_quote(row: &sqlx::postgres::PgRow) -> Quote {
    let status_str: String = row.get("status_text");
    Quote {
        id: row.get("id"),
        building_id: row.get("building_id"),
        contractor_id: row.get("contractor_id"),
        project_title: row.get("project_title"),
        project_description: row.get("project_description"),
        amount_excl_vat: row.get("amount_excl_vat"),
        vat_rate: row.get("vat_rate"),
        amount_incl_vat: row.get("amount_incl_vat"),
        validity_date: row.get("validity_date"),
        estimated_start_date: row.get("estimated_start_date"),
        estimated_duration_days: row.get("estimated_duration_days"),
        warranty_years: row.get("warranty_years"),
        contractor_rating: row.get("contractor_rating"),
        status: QuoteStatus::from_sql(&status_str).unwrap_or(QuoteStatus::Requested),
        requested_at: row.get("requested_at"),
        submitted_at: row.get("submitted_at"),
        reviewed_at: row.get("reviewed_at"),
        decision_at: row.get("decision_at"),
        decision_by: row.get("decision_by"),
        decision_notes: row.get("decision_notes"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}
