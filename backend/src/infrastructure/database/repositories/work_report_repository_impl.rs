use crate::application::dto::{PageRequest, WorkReportFilters};
use crate::application::ports::WorkReportRepository;
use crate::domain::entities::{WarrantyType, WorkReport, WorkType};
use crate::infrastructure::database::pool::DbPool;
use async_trait::async_trait;
use sqlx::Row;
use uuid::Uuid;

pub struct PostgresWorkReportRepository {
    pool: DbPool,
}

impl PostgresWorkReportRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl WorkReportRepository for PostgresWorkReportRepository {
    async fn create(&self, work_report: &WorkReport) -> Result<WorkReport, String> {
        let (warranty_type_str, warranty_custom_years) = warranty_type_to_sql(&work_report.warranty_type);

        sqlx::query(
            r#"
            INSERT INTO work_reports (
                id, organization_id, building_id, title, description, work_type,
                contractor_name, contractor_contact, work_date, completion_date,
                cost, invoice_number, photos, documents, notes, warranty_type,
                warranty_custom_years, warranty_expiry, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20)
            "#,
        )
        .bind(work_report.id)
        .bind(work_report.organization_id)
        .bind(work_report.building_id)
        .bind(&work_report.title)
        .bind(&work_report.description)
        .bind(work_type_to_sql(&work_report.work_type))
        .bind(&work_report.contractor_name)
        .bind(&work_report.contractor_contact)
        .bind(work_report.work_date)
        .bind(work_report.completion_date)
        .bind(work_report.cost)
        .bind(&work_report.invoice_number)
        .bind(serde_json::to_value(&work_report.photos).unwrap_or(serde_json::json!([])))
        .bind(serde_json::to_value(&work_report.documents).unwrap_or(serde_json::json!([])))
        .bind(&work_report.notes)
        .bind(warranty_type_str)
        .bind(warranty_custom_years)
        .bind(work_report.warranty_expiry)
        .bind(work_report.created_at)
        .bind(work_report.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Database error creating work report: {}", e))?;

        Ok(work_report.clone())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<WorkReport>, String> {
        let row = sqlx::query(
            r#"
            SELECT
                id, organization_id, building_id, title, description, work_type,
                contractor_name, contractor_contact, work_date, completion_date,
                cost, invoice_number, photos, documents, notes, warranty_type,
                warranty_custom_years, warranty_expiry, created_at, updated_at
            FROM work_reports
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Database error finding work report: {}", e))?;

        Ok(row.map(|r| map_row_to_work_report(&r)))
    }

    async fn find_by_building(&self, building_id: Uuid) -> Result<Vec<WorkReport>, String> {
        let rows = sqlx::query(
            r#"
            SELECT
                id, organization_id, building_id, title, description, work_type,
                contractor_name, contractor_contact, work_date, completion_date,
                cost, invoice_number, photos, documents, notes, warranty_type,
                warranty_custom_years, warranty_expiry, created_at, updated_at
            FROM work_reports
            WHERE building_id = $1
            ORDER BY work_date DESC
            "#,
        )
        .bind(building_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Database error finding work reports by building: {}", e))?;

        Ok(rows.iter().map(map_row_to_work_report).collect())
    }

    async fn find_by_organization(&self, organization_id: Uuid) -> Result<Vec<WorkReport>, String> {
        let rows = sqlx::query(
            r#"
            SELECT
                id, organization_id, building_id, title, description, work_type,
                contractor_name, contractor_contact, work_date, completion_date,
                cost, invoice_number, photos, documents, notes, warranty_type,
                warranty_custom_years, warranty_expiry, created_at, updated_at
            FROM work_reports
            WHERE organization_id = $1
            ORDER BY work_date DESC
            "#,
        )
        .bind(organization_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Database error finding work reports by organization: {}", e))?;

        Ok(rows.iter().map(map_row_to_work_report).collect())
    }

    async fn find_all_paginated(
        &self,
        page_request: &PageRequest,
        filters: &WorkReportFilters,
    ) -> Result<(Vec<WorkReport>, i64), String> {
        let offset = page_request.offset();
        let limit = page_request.limit();

        // Build WHERE clause based on filters
        let mut where_clauses = vec![];
        let mut bind_count = 0;

        if let Some(building_id) = filters.building_id {
            bind_count += 1;
            where_clauses.push(format!("building_id = ${}", bind_count));
        }

        if let Some(ref work_type) = filters.work_type {
            bind_count += 1;
            where_clauses.push(format!("work_type = ${}", bind_count));
        }

        let where_clause = if where_clauses.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", where_clauses.join(" AND "))
        };

        // Count total
        let count_query = format!("SELECT COUNT(*) FROM work_reports {}", where_clause);
        let mut count_query = sqlx::query_scalar::<_, i64>(&count_query);

        if let Some(building_id) = filters.building_id {
            count_query = count_query.bind(building_id);
        }
        if let Some(ref work_type) = filters.work_type {
            count_query = count_query.bind(work_type);
        }

        let total = count_query
            .fetch_one(&self.pool)
            .await
            .map_err(|e| format!("Database error counting work reports: {}", e))?;

        // Fetch paginated results
        let select_query = format!(
            r#"
            SELECT
                id, organization_id, building_id, title, description, work_type,
                contractor_name, contractor_contact, work_date, completion_date,
                cost, invoice_number, photos, documents, notes, warranty_type,
                warranty_custom_years, warranty_expiry, created_at, updated_at
            FROM work_reports
            {}
            ORDER BY work_date DESC
            LIMIT ${}
            OFFSET ${}
            "#,
            where_clause,
            bind_count + 1,
            bind_count + 2
        );

        let mut select_query = sqlx::query(&select_query);

        if let Some(building_id) = filters.building_id {
            select_query = select_query.bind(building_id);
        }
        if let Some(ref work_type) = filters.work_type {
            select_query = select_query.bind(work_type);
        }

        let rows = select_query
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| format!("Database error fetching work reports: {}", e))?;

        let work_reports = rows.iter().map(map_row_to_work_report).collect();

        Ok((work_reports, total))
    }

    async fn find_with_active_warranty(
        &self,
        building_id: Uuid,
    ) -> Result<Vec<WorkReport>, String> {
        let rows = sqlx::query(
            r#"
            SELECT
                id, organization_id, building_id, title, description, work_type,
                contractor_name, contractor_contact, work_date, completion_date,
                cost, invoice_number, photos, documents, notes, warranty_type,
                warranty_custom_years, warranty_expiry, created_at, updated_at
            FROM work_reports
            WHERE building_id = $1
              AND warranty_type != 'none'
              AND warranty_expiry > NOW()
            ORDER BY warranty_expiry ASC
            "#,
        )
        .bind(building_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Database error finding active warranties: {}", e))?;

        Ok(rows.iter().map(map_row_to_work_report).collect())
    }

    async fn find_with_expiring_warranty(
        &self,
        building_id: Uuid,
        days: i32,
    ) -> Result<Vec<WorkReport>, String> {
        let rows = sqlx::query(
            r#"
            SELECT
                id, organization_id, building_id, title, description, work_type,
                contractor_name, contractor_contact, work_date, completion_date,
                cost, invoice_number, photos, documents, notes, warranty_type,
                warranty_custom_years, warranty_expiry, created_at, updated_at
            FROM work_reports
            WHERE building_id = $1
              AND warranty_type != 'none'
              AND warranty_expiry > NOW()
              AND warranty_expiry <= NOW() + INTERVAL '1 day' * $2
            ORDER BY warranty_expiry ASC
            "#,
        )
        .bind(building_id)
        .bind(days)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Database error finding expiring warranties: {}", e))?;

        Ok(rows.iter().map(map_row_to_work_report).collect())
    }

    async fn update(&self, work_report: &WorkReport) -> Result<WorkReport, String> {
        let (warranty_type_str, warranty_custom_years) = warranty_type_to_sql(&work_report.warranty_type);

        sqlx::query(
            r#"
            UPDATE work_reports
            SET
                building_id = $2,
                title = $3,
                description = $4,
                work_type = $5,
                contractor_name = $6,
                contractor_contact = $7,
                work_date = $8,
                completion_date = $9,
                cost = $10,
                invoice_number = $11,
                photos = $12,
                documents = $13,
                notes = $14,
                warranty_type = $15,
                warranty_custom_years = $16,
                warranty_expiry = $17,
                updated_at = $18
            WHERE id = $1
            "#,
        )
        .bind(work_report.id)
        .bind(work_report.building_id)
        .bind(&work_report.title)
        .bind(&work_report.description)
        .bind(work_type_to_sql(&work_report.work_type))
        .bind(&work_report.contractor_name)
        .bind(&work_report.contractor_contact)
        .bind(work_report.work_date)
        .bind(work_report.completion_date)
        .bind(work_report.cost)
        .bind(&work_report.invoice_number)
        .bind(serde_json::to_value(&work_report.photos).unwrap_or(serde_json::json!([])))
        .bind(serde_json::to_value(&work_report.documents).unwrap_or(serde_json::json!([])))
        .bind(&work_report.notes)
        .bind(warranty_type_str)
        .bind(warranty_custom_years)
        .bind(work_report.warranty_expiry)
        .bind(work_report.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Database error updating work report: {}", e))?;

        Ok(work_report.clone())
    }

    async fn delete(&self, id: Uuid) -> Result<bool, String> {
        let result = sqlx::query("DELETE FROM work_reports WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("Database error deleting work report: {}", e))?;

        Ok(result.rows_affected() > 0)
    }
}

/// Helper function to map PostgreSQL row to WorkReport entity
fn map_row_to_work_report(row: &sqlx::postgres::PgRow) -> WorkReport {
    let warranty_type_str: String = row.get("warranty_type");
    let warranty_custom_years: Option<i32> = row.get("warranty_custom_years");
    let warranty_type = warranty_type_from_sql(&warranty_type_str, warranty_custom_years);

    let photos: Vec<String> = row
        .get::<serde_json::Value, _>("photos")
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    let documents: Vec<String> = row
        .get::<serde_json::Value, _>("documents")
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    WorkReport {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        building_id: row.get("building_id"),
        title: row.get("title"),
        description: row.get("description"),
        work_type: work_type_from_sql(row.get("work_type")),
        contractor_name: row.get("contractor_name"),
        contractor_contact: row.get("contractor_contact"),
        work_date: row.get("work_date"),
        completion_date: row.get("completion_date"),
        cost: row.get("cost"),
        invoice_number: row.get("invoice_number"),
        photos,
        documents,
        notes: row.get("notes"),
        warranty_type,
        warranty_expiry: row.get("warranty_expiry"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

/// Convert WorkType to SQL string
fn work_type_to_sql(work_type: &WorkType) -> &'static str {
    match work_type {
        WorkType::Maintenance => "maintenance",
        WorkType::Repair => "repair",
        WorkType::Renovation => "renovation",
        WorkType::Emergency => "emergency",
        WorkType::Inspection => "inspection",
        WorkType::Installation => "installation",
        WorkType::Other => "other",
    }
}

/// Convert SQL string to WorkType
fn work_type_from_sql(s: &str) -> WorkType {
    match s {
        "maintenance" => WorkType::Maintenance,
        "repair" => WorkType::Repair,
        "renovation" => WorkType::Renovation,
        "emergency" => WorkType::Emergency,
        "inspection" => WorkType::Inspection,
        "installation" => WorkType::Installation,
        _ => WorkType::Other,
    }
}

/// Convert WarrantyType to SQL (type string, custom years option)
fn warranty_type_to_sql(warranty_type: &WarrantyType) -> (&'static str, Option<i32>) {
    match warranty_type {
        WarrantyType::None => ("none", None),
        WarrantyType::Standard => ("standard", None),
        WarrantyType::Decennial => ("decennial", None),
        WarrantyType::Extended => ("extended", None),
        WarrantyType::Custom { years } => ("custom", Some(*years)),
    }
}

/// Convert SQL to WarrantyType
fn warranty_type_from_sql(type_str: &str, custom_years: Option<i32>) -> WarrantyType {
    match type_str {
        "none" => WarrantyType::None,
        "standard" => WarrantyType::Standard,
        "decennial" => WarrantyType::Decennial,
        "extended" => WarrantyType::Extended,
        "custom" => WarrantyType::Custom {
            years: custom_years.unwrap_or(1),
        },
        _ => WarrantyType::None,
    }
}
