use crate::domain::entities::{InspectionStatus, InspectionType};
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Deserialize, Validate, Clone)]
pub struct CreateTechnicalInspectionDto {
    pub organization_id: String,
    pub building_id: String,

    #[validate(length(min = 1, max = 255))]
    pub title: String,

    pub description: Option<String>,
    pub inspection_type: InspectionType,

    #[validate(length(min = 1, max = 255))]
    pub inspector_name: String,

    #[validate(length(max = 255))]
    pub inspector_company: Option<String>,

    #[validate(length(max = 100))]
    pub inspector_certification: Option<String>,

    pub inspection_date: String, // ISO 8601 format

    pub result_summary: Option<String>,
    pub defects_found: Option<String>,
    pub recommendations: Option<String>,

    pub compliant: Option<bool>,

    #[validate(length(max = 100))]
    pub compliance_certificate_number: Option<String>,

    pub compliance_valid_until: Option<String>, // ISO 8601 format

    #[validate(range(min = 0.0))]
    pub cost: Option<f64>,

    #[validate(length(max = 100))]
    pub invoice_number: Option<String>,

    pub notes: Option<String>,
}

#[derive(Debug, Deserialize, Validate, Clone)]
pub struct UpdateTechnicalInspectionDto {
    #[validate(length(min = 1, max = 255))]
    pub title: Option<String>,

    pub description: Option<String>,
    pub inspection_type: Option<InspectionType>,

    #[validate(length(min = 1, max = 255))]
    pub inspector_name: Option<String>,

    #[validate(length(max = 255))]
    pub inspector_company: Option<String>,

    #[validate(length(max = 100))]
    pub inspector_certification: Option<String>,

    pub inspection_date: Option<String>,
    pub status: Option<InspectionStatus>,

    pub result_summary: Option<String>,
    pub defects_found: Option<String>,
    pub recommendations: Option<String>,

    pub compliant: Option<bool>,

    #[validate(length(max = 100))]
    pub compliance_certificate_number: Option<String>,

    pub compliance_valid_until: Option<String>,

    #[validate(range(min = 0.0))]
    pub cost: Option<f64>,

    #[validate(length(max = 100))]
    pub invoice_number: Option<String>,

    pub notes: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct TechnicalInspectionResponseDto {
    pub id: String,
    pub organization_id: String,
    pub building_id: String,
    pub title: String,
    pub description: Option<String>,
    pub inspection_type: InspectionType,
    pub inspector_name: String,
    pub inspector_company: Option<String>,
    pub inspector_certification: Option<String>,
    pub inspection_date: String,
    pub next_due_date: String,
    pub status: InspectionStatus,
    pub result_summary: Option<String>,
    pub defects_found: Option<String>,
    pub recommendations: Option<String>,
    pub compliant: Option<bool>,
    pub compliance_certificate_number: Option<String>,
    pub compliance_valid_until: Option<String>,
    pub cost: Option<f64>,
    pub invoice_number: Option<String>,
    pub reports: Vec<String>,
    pub photos: Vec<String>,
    pub certificates: Vec<String>,
    pub notes: Option<String>,
    pub is_overdue: bool,
    pub days_until_due: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct AddReportDto {
    #[validate(length(min = 1))]
    pub report_path: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct AddInspectionPhotoDto {
    #[validate(length(min = 1))]
    pub photo_path: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct AddCertificateDto {
    #[validate(length(min = 1))]
    pub certificate_path: String,
}

#[derive(Debug, Serialize)]
pub struct TechnicalInspectionListResponseDto {
    pub inspections: Vec<TechnicalInspectionResponseDto>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
}

#[derive(Debug, Serialize)]
pub struct InspectionStatusDto {
    pub inspection_id: String,
    pub title: String,
    pub inspection_type: InspectionType,
    pub next_due_date: String,
    pub status: InspectionStatus,
    pub is_overdue: bool,
    pub days_until_due: i64,
}
