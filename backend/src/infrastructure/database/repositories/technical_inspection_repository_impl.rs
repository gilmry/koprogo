use crate::application::dto::{PageRequest, TechnicalInspectionFilters};
use crate::application::ports::TechnicalInspectionRepository;
use crate::domain::entities::{InspectionStatus, InspectionType, TechnicalInspection};
use crate::infrastructure::database::pool::DbPool;
use async_trait::async_trait;
use sqlx::Row;
use uuid::Uuid;

pub struct PostgresTechnicalInspectionRepository {
    pool: DbPool,
}

impl PostgresTechnicalInspectionRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TechnicalInspectionRepository for PostgresTechnicalInspectionRepository {
    async fn create(
        &self,
        inspection: &TechnicalInspection,
    ) -> Result<TechnicalInspection, String> {
        sqlx::query(
            r#"
            INSERT INTO technical_inspections (
                id, organization_id, building_id, title, description, inspection_type,
                inspector_name, inspector_company, inspector_certification,
                inspection_date, next_due_date, status, result_summary, defects_found,
                recommendations, compliant, compliance_certificate_number,
                compliance_valid_until, cost, invoice_number, reports, photos,
                certificates, notes, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23, $24, $25, $26)
            "#,
        )
        .bind(inspection.id)
        .bind(inspection.organization_id)
        .bind(inspection.building_id)
        .bind(&inspection.title)
        .bind(&inspection.description)
        .bind(inspection_type_to_sql(&inspection.inspection_type))
        .bind(&inspection.inspector_name)
        .bind(&inspection.inspector_company)
        .bind(&inspection.inspector_certification)
        .bind(inspection.inspection_date)
        .bind(inspection.next_due_date)
        .bind(inspection_status_to_sql(&inspection.status))
        .bind(&inspection.result_summary)
        .bind(&inspection.defects_found)
        .bind(&inspection.recommendations)
        .bind(inspection.compliant)
        .bind(&inspection.compliance_certificate_number)
        .bind(inspection.compliance_valid_until)
        .bind(inspection.cost)
        .bind(&inspection.invoice_number)
        .bind(serde_json::to_value(&inspection.reports).unwrap_or(serde_json::json!([])))
        .bind(serde_json::to_value(&inspection.photos).unwrap_or(serde_json::json!([])))
        .bind(serde_json::to_value(&inspection.certificates).unwrap_or(serde_json::json!([])))
        .bind(&inspection.notes)
        .bind(inspection.created_at)
        .bind(inspection.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Database error creating technical inspection: {}", e))?;

        Ok(inspection.clone())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<TechnicalInspection>, String> {
        let row = sqlx::query(
            r#"
            SELECT
                id, organization_id, building_id, title, description, inspection_type,
                inspector_name, inspector_company, inspector_certification,
                inspection_date, next_due_date, status, result_summary, defects_found,
                recommendations, compliant, compliance_certificate_number,
                compliance_valid_until, cost, invoice_number, reports, photos,
                certificates, notes, created_at, updated_at
            FROM technical_inspections
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Database error finding technical inspection: {}", e))?;

        Ok(row.map(|r| map_row_to_technical_inspection(&r)))
    }

    async fn find_by_building(
        &self,
        building_id: Uuid,
    ) -> Result<Vec<TechnicalInspection>, String> {
        let rows = sqlx::query(
            r#"
            SELECT
                id, organization_id, building_id, title, description, inspection_type,
                inspector_name, inspector_company, inspector_certification,
                inspection_date, next_due_date, status, result_summary, defects_found,
                recommendations, compliant, compliance_certificate_number,
                compliance_valid_until, cost, invoice_number, reports, photos,
                certificates, notes, created_at, updated_at
            FROM technical_inspections
            WHERE building_id = $1
            ORDER BY inspection_date DESC
            "#,
        )
        .bind(building_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Database error finding inspections by building: {}", e))?;

        Ok(rows.iter().map(map_row_to_technical_inspection).collect())
    }

    async fn find_by_organization(
        &self,
        organization_id: Uuid,
    ) -> Result<Vec<TechnicalInspection>, String> {
        let rows = sqlx::query(
            r#"
            SELECT
                id, organization_id, building_id, title, description, inspection_type,
                inspector_name, inspector_company, inspector_certification,
                inspection_date, next_due_date, status, result_summary, defects_found,
                recommendations, compliant, compliance_certificate_number,
                compliance_valid_until, cost, invoice_number, reports, photos,
                certificates, notes, created_at, updated_at
            FROM technical_inspections
            WHERE organization_id = $1
            ORDER BY inspection_date DESC
            "#,
        )
        .bind(organization_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Database error finding inspections by organization: {}", e))?;

        Ok(rows.iter().map(map_row_to_technical_inspection).collect())
    }

    async fn find_all_paginated(
        &self,
        page_request: &PageRequest,
        filters: &TechnicalInspectionFilters,
    ) -> Result<(Vec<TechnicalInspection>, i64), String> {
        let offset = page_request.offset();
        let limit = page_request.limit();

        // Build WHERE clause based on filters
        let mut where_clauses = vec![];
        let mut bind_count = 0;

        #[allow(unused_variables)]
        if let Some(building_id) = filters.building_id {
            bind_count += 1;
            where_clauses.push(format!("building_id = ${}", bind_count));
        }

        #[allow(unused_variables)]
        if let Some(ref inspection_type) = filters.inspection_type {
            bind_count += 1;
            where_clauses.push(format!("inspection_type = ${}", bind_count));
        }

        #[allow(unused_variables)]
        if let Some(ref status) = filters.status {
            bind_count += 1;
            where_clauses.push(format!("status = ${}", bind_count));
        }

        let where_clause = if where_clauses.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", where_clauses.join(" AND "))
        };

        // Count total
        let count_query = format!(
            "SELECT COUNT(*) FROM technical_inspections {}",
            where_clause
        );
        let mut count_query = sqlx::query_scalar::<_, i64>(&count_query);

        if let Some(building_id) = filters.building_id {
            count_query = count_query.bind(building_id);
        }
        if let Some(ref inspection_type) = filters.inspection_type {
            count_query = count_query.bind(inspection_type);
        }
        if let Some(ref status) = filters.status {
            count_query = count_query.bind(status);
        }

        let total = count_query
            .fetch_one(&self.pool)
            .await
            .map_err(|e| format!("Database error counting inspections: {}", e))?;

        // Fetch paginated results
        let select_query = format!(
            r#"
            SELECT
                id, organization_id, building_id, title, description, inspection_type,
                inspector_name, inspector_company, inspector_certification,
                inspection_date, next_due_date, status, result_summary, defects_found,
                recommendations, compliant, compliance_certificate_number,
                compliance_valid_until, cost, invoice_number, reports, photos,
                certificates, notes, created_at, updated_at
            FROM technical_inspections
            {}
            ORDER BY inspection_date DESC
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
        if let Some(ref inspection_type) = filters.inspection_type {
            select_query = select_query.bind(inspection_type);
        }
        if let Some(ref status) = filters.status {
            select_query = select_query.bind(status);
        }

        let rows = select_query
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| format!("Database error fetching inspections: {}", e))?;

        let inspections = rows.iter().map(map_row_to_technical_inspection).collect();

        Ok((inspections, total))
    }

    async fn find_overdue(&self, building_id: Uuid) -> Result<Vec<TechnicalInspection>, String> {
        let rows = sqlx::query(
            r#"
            SELECT
                id, organization_id, building_id, title, description, inspection_type,
                inspector_name, inspector_company, inspector_certification,
                inspection_date, next_due_date, status, result_summary, defects_found,
                recommendations, compliant, compliance_certificate_number,
                compliance_valid_until, cost, invoice_number, reports, photos,
                certificates, notes, created_at, updated_at
            FROM technical_inspections
            WHERE building_id = $1
              AND next_due_date < NOW()
              AND status = 'pending'
            ORDER BY next_due_date ASC
            "#,
        )
        .bind(building_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Database error finding overdue inspections: {}", e))?;

        Ok(rows.iter().map(map_row_to_technical_inspection).collect())
    }

    async fn find_upcoming(
        &self,
        building_id: Uuid,
        days: i32,
    ) -> Result<Vec<TechnicalInspection>, String> {
        let rows = sqlx::query(
            r#"
            SELECT
                id, organization_id, building_id, title, description, inspection_type,
                inspector_name, inspector_company, inspector_certification,
                inspection_date, next_due_date, status, result_summary, defects_found,
                recommendations, compliant, compliance_certificate_number,
                compliance_valid_until, cost, invoice_number, reports, photos,
                certificates, notes, created_at, updated_at
            FROM technical_inspections
            WHERE building_id = $1
              AND next_due_date > NOW()
              AND next_due_date <= NOW() + INTERVAL '1 day' * $2
              AND status = 'pending'
            ORDER BY next_due_date ASC
            "#,
        )
        .bind(building_id)
        .bind(days)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Database error finding upcoming inspections: {}", e))?;

        Ok(rows.iter().map(map_row_to_technical_inspection).collect())
    }

    async fn find_by_type(
        &self,
        building_id: Uuid,
        inspection_type: &str,
    ) -> Result<Vec<TechnicalInspection>, String> {
        let rows = sqlx::query(
            r#"
            SELECT
                id, organization_id, building_id, title, description, inspection_type,
                inspector_name, inspector_company, inspector_certification,
                inspection_date, next_due_date, status, result_summary, defects_found,
                recommendations, compliant, compliance_certificate_number,
                compliance_valid_until, cost, invoice_number, reports, photos,
                certificates, notes, created_at, updated_at
            FROM technical_inspections
            WHERE building_id = $1
              AND inspection_type = $2
            ORDER BY inspection_date DESC
            "#,
        )
        .bind(building_id)
        .bind(inspection_type)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Database error finding inspections by type: {}", e))?;

        Ok(rows.iter().map(map_row_to_technical_inspection).collect())
    }

    async fn update(
        &self,
        inspection: &TechnicalInspection,
    ) -> Result<TechnicalInspection, String> {
        sqlx::query(
            r#"
            UPDATE technical_inspections
            SET
                building_id = $2,
                title = $3,
                description = $4,
                inspection_type = $5,
                inspector_name = $6,
                inspector_company = $7,
                inspector_certification = $8,
                inspection_date = $9,
                next_due_date = $10,
                status = $11,
                result_summary = $12,
                defects_found = $13,
                recommendations = $14,
                compliant = $15,
                compliance_certificate_number = $16,
                compliance_valid_until = $17,
                cost = $18,
                invoice_number = $19,
                reports = $20,
                photos = $21,
                certificates = $22,
                notes = $23,
                updated_at = $24
            WHERE id = $1
            "#,
        )
        .bind(inspection.id)
        .bind(inspection.building_id)
        .bind(&inspection.title)
        .bind(&inspection.description)
        .bind(inspection_type_to_sql(&inspection.inspection_type))
        .bind(&inspection.inspector_name)
        .bind(&inspection.inspector_company)
        .bind(&inspection.inspector_certification)
        .bind(inspection.inspection_date)
        .bind(inspection.next_due_date)
        .bind(inspection_status_to_sql(&inspection.status))
        .bind(&inspection.result_summary)
        .bind(&inspection.defects_found)
        .bind(&inspection.recommendations)
        .bind(inspection.compliant)
        .bind(&inspection.compliance_certificate_number)
        .bind(inspection.compliance_valid_until)
        .bind(inspection.cost)
        .bind(&inspection.invoice_number)
        .bind(serde_json::to_value(&inspection.reports).unwrap_or(serde_json::json!([])))
        .bind(serde_json::to_value(&inspection.photos).unwrap_or(serde_json::json!([])))
        .bind(serde_json::to_value(&inspection.certificates).unwrap_or(serde_json::json!([])))
        .bind(&inspection.notes)
        .bind(inspection.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Database error updating technical inspection: {}", e))?;

        Ok(inspection.clone())
    }

    async fn delete(&self, id: Uuid) -> Result<bool, String> {
        let result = sqlx::query("DELETE FROM technical_inspections WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("Database error deleting technical inspection: {}", e))?;

        Ok(result.rows_affected() > 0)
    }
}

/// Helper function to map PostgreSQL row to TechnicalInspection entity
fn map_row_to_technical_inspection(row: &sqlx::postgres::PgRow) -> TechnicalInspection {
    let inspection_type_str: String = row.get("inspection_type");
    let inspection_type = inspection_type_from_sql(&inspection_type_str);

    let status_str: String = row.get("status");
    let status = inspection_status_from_sql(&status_str);

    let reports: Vec<String> = row
        .get::<serde_json::Value, _>("reports")
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    let photos: Vec<String> = row
        .get::<serde_json::Value, _>("photos")
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    let certificates: Vec<String> = row
        .get::<serde_json::Value, _>("certificates")
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    TechnicalInspection {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        building_id: row.get("building_id"),
        inspection_type,
        title: row.get("title"),
        description: row.get("description"),
        inspector_name: row.get("inspector_name"),
        inspector_company: row.get("inspector_company"),
        inspector_certification: row.get("inspector_certification"),
        inspection_date: row.get("inspection_date"),
        next_due_date: row.get("next_due_date"),
        status,
        result_summary: row.get("result_summary"),
        defects_found: row.get("defects_found"),
        recommendations: row.get("recommendations"),
        compliant: row.get("compliant"),
        compliance_certificate_number: row.get("compliance_certificate_number"),
        compliance_valid_until: row.get("compliance_valid_until"),
        cost: row.get("cost"),
        invoice_number: row.get("invoice_number"),
        reports,
        photos,
        certificates,
        notes: row.get("notes"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

/// Convert InspectionType to SQL string
fn inspection_type_to_sql(inspection_type: &InspectionType) -> String {
    match inspection_type {
        InspectionType::Elevator => "elevator".to_string(),
        InspectionType::Boiler => "boiler".to_string(),
        InspectionType::Electrical => "electrical".to_string(),
        InspectionType::FireExtinguisher => "fire_extinguisher".to_string(),
        InspectionType::FireAlarm => "fire_alarm".to_string(),
        InspectionType::GasInstallation => "gas_installation".to_string(),
        InspectionType::RoofStructure => "roof".to_string(),
        InspectionType::Facade => "facade".to_string(),
        InspectionType::WaterQuality => "water_tank".to_string(),
        InspectionType::Other { name: _ } => {
            // Store custom inspection types as "other" in the DB
            // The name is stored in the title field
            "other".to_string()
        }
    }
}

/// Convert SQL string to InspectionType
fn inspection_type_from_sql(s: &str) -> InspectionType {
    match s {
        "elevator" => InspectionType::Elevator,
        "boiler" => InspectionType::Boiler,
        "electrical" => InspectionType::Electrical,
        "fire_extinguisher" => InspectionType::FireExtinguisher,
        "fire_alarm" => InspectionType::FireAlarm,
        "gas_installation" => InspectionType::GasInstallation,
        "roof" => InspectionType::RoofStructure,
        "facade" => InspectionType::Facade,
        "water_tank" => InspectionType::WaterQuality,
        "drainage" => InspectionType::Other {
            name: "Drainage".to_string(),
        },
        "emergency_lighting" => InspectionType::Other {
            name: "Emergency Lighting".to_string(),
        },
        "other" => InspectionType::Other {
            name: "Other".to_string(),
        },
        _ => InspectionType::Other {
            name: s.to_string(),
        },
    }
}

/// Convert InspectionStatus to SQL string
fn inspection_status_to_sql(status: &InspectionStatus) -> &'static str {
    match status {
        InspectionStatus::Scheduled => "pending",
        InspectionStatus::InProgress => "pending", // Map to pending in DB
        InspectionStatus::Completed => "completed",
        InspectionStatus::Failed => "failed",
        InspectionStatus::Overdue => "pending", // Map to pending in DB
        InspectionStatus::Cancelled => "failed", // Map to failed in DB
    }
}

/// Convert SQL string to InspectionStatus
fn inspection_status_from_sql(s: &str) -> InspectionStatus {
    match s {
        "pending" => InspectionStatus::Scheduled,
        "completed" => InspectionStatus::Completed,
        "failed" => InspectionStatus::Failed,
        "passed_with_remarks" => InspectionStatus::Completed, // Map to completed
        _ => InspectionStatus::Scheduled,
    }
}
