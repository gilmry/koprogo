use crate::infrastructure::web::{AppState, AuthenticatedUser};
use actix_web::{get, web, HttpResponse, Responder};
use serde::Serialize;

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

#[derive(Debug, Serialize)]
pub struct ProcessingActivitiesResponse {
    pub activities: Vec<DataProcessingActivityDto>,
    pub total: i64,
}

#[derive(Debug, Serialize)]
pub struct ProcessorsResponse {
    pub processors: Vec<DataProcessorAgreementDto>,
    pub total: i64,
}

/// GET /api/v1/admin/gdpr/processing-register — SuperAdmin only
#[get("/admin/gdpr/processing-register")]
pub async fn list_processing_activities(
    data: web::Data<AppState>,
    auth: AuthenticatedUser,
) -> impl Responder {
    if auth.role != "superadmin" {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Access denied. SuperAdmin role required."
        }));
    }

    match data.gdpr_art30_use_cases.list_processing_activities().await {
        Ok(activities) => {
            let dtos: Vec<DataProcessingActivityDto> = activities
                .into_iter()
                .map(|a| DataProcessingActivityDto {
                    id: a.id.to_string(),
                    activity_name: a.activity_name,
                    controller_name: a.controller_name,
                    purpose: a.purpose,
                    legal_basis: a.legal_basis,
                    data_categories: a.data_categories,
                    data_subjects: a.data_subjects,
                    recipients: a.recipients,
                    retention_period: a.retention_period,
                    security_measures: a.security_measures,
                    created_at: a.created_at.to_rfc3339(),
                    updated_at: a.updated_at.to_rfc3339(),
                })
                .collect();
            let total = dtos.len() as i64;
            HttpResponse::Ok().json(ProcessingActivitiesResponse { activities: dtos, total })
        }
        Err(e) => HttpResponse::InternalServerError()
            .json(serde_json::json!({ "error": format!("Failed to fetch processing activities: {}", e) })),
    }
}

/// GET /api/v1/admin/gdpr/processors — SuperAdmin only
#[get("/admin/gdpr/processors")]
pub async fn list_sub_processors(
    data: web::Data<AppState>,
    auth: AuthenticatedUser,
) -> impl Responder {
    if auth.role != "superadmin" {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Access denied. SuperAdmin role required."
        }));
    }

    match data.gdpr_art30_use_cases.list_processor_agreements().await {
        Ok(processors) => {
            let dtos: Vec<DataProcessorAgreementDto> = processors
                .into_iter()
                .map(|p| DataProcessorAgreementDto {
                    id: p.id.to_string(),
                    processor_name: p.processor_name,
                    service_description: p.service_description,
                    dpa_signed_at: p.dpa_signed_at.map(|dt| dt.to_rfc3339()),
                    dpa_url: p.dpa_url,
                    transfer_mechanism: p.transfer_mechanism,
                    data_categories: p.data_categories,
                    certifications: p.certifications,
                    created_at: p.created_at.to_rfc3339(),
                    updated_at: p.updated_at.to_rfc3339(),
                })
                .collect();
            let total = dtos.len() as i64;
            HttpResponse::Ok().json(ProcessorsResponse { processors: dtos, total })
        }
        Err(e) => HttpResponse::InternalServerError()
            .json(serde_json::json!({ "error": format!("Failed to fetch sub-processors: {}", e) })),
    }
}
