use crate::infrastructure::web::{AppState, AuthenticatedUser};
use actix_web::{get, web, HttpResponse, Responder};
use serde::Serialize;

/// DTO for data processing activity (Art. 30 register)
#[derive(Debug, Serialize)]
pub struct DataProcessingActivityDto {
    pub id: String,
    pub activity_name: String,
    pub controller_name: String,
    pub purpose: String,
    pub legal_basis: String,
    pub data_categories: Vec<String>,
    pub data_subjects: Vec<String>,
    pub recipients: Vec<String>,
    pub retention_period: String,
    pub security_measures: String,
    pub created_at: String,
    pub updated_at: String,
}

/// DTO for data processor agreement (DPA with sub-processors)
#[derive(Debug, Serialize)]
pub struct DataProcessorAgreementDto {
    pub id: String,
    pub processor_name: String,
    pub service_description: String,
    pub dpa_signed_at: Option<String>,
    pub dpa_url: Option<String>,
    pub transfer_mechanism: Option<String>,
    pub data_categories: Vec<String>,
    pub certifications: Option<Vec<String>>,
    pub created_at: String,
    pub updated_at: String,
}

/// Response for paginated list of processing activities
#[derive(Debug, Serialize)]
pub struct ProcessingActivitiesResponse {
    pub activities: Vec<DataProcessingActivityDto>,
    pub total: i64,
}

/// Response for paginated list of sub-processors
#[derive(Debug, Serialize)]
pub struct ProcessorsResponse {
    pub processors: Vec<DataProcessorAgreementDto>,
    pub total: i64,
}

/// GET /api/v1/admin/gdpr/processing-register
/// List all GDPR Article 30 data processing activities (SuperAdmin only)
///
/// # Returns
/// * `200 OK` - List of all data processing activities
/// * `401 Unauthorized` - Missing or invalid authentication
/// * `403 Forbidden` - User is not SuperAdmin
/// * `500 Internal Server Error` - Database error
#[get("/admin/gdpr/processing-register")]
pub async fn list_processing_activities(
    data: web::Data<AppState>,
    auth: AuthenticatedUser,
) -> impl Responder {
    // Only SuperAdmin can view processing register
    if auth.role != "superadmin" {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Access denied. SuperAdmin role required."
        }));
    }

    // Fetch processing activities from database
    match data.pool.acquire().await {
        Ok(mut conn) => {
            match sqlx::query_as!(
                ProcessingActivityRow,
                r#"
                SELECT
                    id,
                    activity_name,
                    controller_name,
                    purpose,
                    legal_basis,
                    data_categories,
                    data_subjects,
                    recipients,
                    retention_period,
                    security_measures,
                    created_at,
                    updated_at
                FROM data_processing_activities
                ORDER BY created_at DESC
                "#
            )
            .fetch_all(&mut *conn)
            .await
            {
                Ok(rows) => {
                    let total = rows.len() as i64;
                    let activities: Vec<DataProcessingActivityDto> = rows
                        .into_iter()
                        .map(|row| DataProcessingActivityDto {
                            id: row.id.to_string(),
                            activity_name: row.activity_name,
                            controller_name: row.controller_name,
                            purpose: row.purpose,
                            legal_basis: row.legal_basis,
                            data_categories: row.data_categories.unwrap_or_default(),
                            data_subjects: row.data_subjects.unwrap_or_default(),
                            recipients: row.recipients.unwrap_or_default(),
                            retention_period: row.retention_period,
                            security_measures: row.security_measures,
                            created_at: row.created_at.to_rfc3339(),
                            updated_at: row.updated_at.to_rfc3339(),
                        })
                        .collect();

                    HttpResponse::Ok().json(ProcessingActivitiesResponse { activities, total })
                }
                Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": format!("Failed to fetch processing activities: {}", e)
                })),
            }
        }
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Database connection error: {}", e)
        })),
    }
}

/// GET /api/v1/admin/gdpr/processors
/// List all sub-processor agreements and DPA status (SuperAdmin only)
///
/// # Returns
/// * `200 OK` - List of all sub-processors with DPA status
/// * `401 Unauthorized` - Missing or invalid authentication
/// * `403 Forbidden` - User is not SuperAdmin
/// * `500 Internal Server Error` - Database error
#[get("/admin/gdpr/processors")]
pub async fn list_sub_processors(
    data: web::Data<AppState>,
    auth: AuthenticatedUser,
) -> impl Responder {
    // Only SuperAdmin can view sub-processor register
    if auth.role != "superadmin" {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Access denied. SuperAdmin role required."
        }));
    }

    // Fetch sub-processor agreements from database
    match data.pool.acquire().await {
        Ok(mut conn) => {
            match sqlx::query_as!(
                ProcessorAgreementRow,
                r#"
                SELECT
                    id,
                    processor_name,
                    service_description,
                    dpa_signed_at,
                    dpa_url,
                    transfer_mechanism,
                    data_categories,
                    certifications,
                    created_at,
                    updated_at
                FROM data_processor_agreements
                ORDER BY processor_name ASC
                "#
            )
            .fetch_all(&mut *conn)
            .await
            {
                Ok(rows) => {
                    let total = rows.len() as i64;
                    let processors: Vec<DataProcessorAgreementDto> = rows
                        .into_iter()
                        .map(|row| DataProcessorAgreementDto {
                            id: row.id.to_string(),
                            processor_name: row.processor_name,
                            service_description: row.service_description,
                            dpa_signed_at: row.dpa_signed_at.map(|dt| dt.to_rfc3339()),
                            dpa_url: row.dpa_url,
                            transfer_mechanism: row.transfer_mechanism,
                            data_categories: row.data_categories.unwrap_or_default(),
                            certifications: row.certifications,
                            created_at: row.created_at.to_rfc3339(),
                            updated_at: row.updated_at.to_rfc3339(),
                        })
                        .collect();

                    HttpResponse::Ok().json(ProcessorsResponse { processors, total })
                }
                Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": format!("Failed to fetch sub-processors: {}", e)
                })),
            }
        }
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Database connection error: {}", e)
        })),
    }
}

// Internal structs for sqlx query results
struct ProcessingActivityRow {
    id: uuid::Uuid,
    activity_name: String,
    controller_name: String,
    purpose: String,
    legal_basis: String,
    data_categories: Option<Vec<String>>,
    data_subjects: Option<Vec<String>>,
    recipients: Option<Vec<String>>,
    retention_period: String,
    security_measures: String,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

struct ProcessorAgreementRow {
    id: uuid::Uuid,
    processor_name: String,
    service_description: String,
    dpa_signed_at: Option<chrono::DateTime<chrono::Utc>>,
    dpa_url: Option<String>,
    transfer_mechanism: Option<String>,
    data_categories: Option<Vec<String>>,
    certifications: Option<Vec<String>>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_handler_structure_list_processing_activities() {
        // Structural test - actual testing in E2E
    }

    #[test]
    fn test_handler_structure_list_sub_processors() {
        // Structural test - actual testing in E2E
    }
}
