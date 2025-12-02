use crate::domain::entities::{WarrantyType, WorkType};
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Deserialize, Validate, Clone)]
pub struct CreateWorkReportDto {
    pub organization_id: String,
    pub building_id: String,

    #[validate(length(min = 1, max = 255))]
    pub title: String,

    #[validate(length(min = 1))]
    pub description: String,

    pub work_type: WorkType,

    #[validate(length(min = 1, max = 255))]
    pub contractor_name: String,

    #[validate(length(max = 255))]
    pub contractor_contact: Option<String>,

    pub work_date: String,               // ISO 8601 format
    pub completion_date: Option<String>, // ISO 8601 format

    #[validate(range(min = 0.0))]
    pub cost: f64,

    #[validate(length(max = 100))]
    pub invoice_number: Option<String>,

    pub notes: Option<String>,
    pub warranty_type: WarrantyType,
}

#[derive(Debug, Deserialize, Validate, Clone)]
pub struct UpdateWorkReportDto {
    #[validate(length(min = 1, max = 255))]
    pub title: Option<String>,

    #[validate(length(min = 1))]
    pub description: Option<String>,

    pub work_type: Option<WorkType>,

    #[validate(length(min = 1, max = 255))]
    pub contractor_name: Option<String>,

    #[validate(length(max = 255))]
    pub contractor_contact: Option<String>,

    pub work_date: Option<String>,
    pub completion_date: Option<String>,

    #[validate(range(min = 0.0))]
    pub cost: Option<f64>,

    #[validate(length(max = 100))]
    pub invoice_number: Option<String>,

    pub notes: Option<String>,
    pub warranty_type: Option<WarrantyType>,
}

#[derive(Debug, Serialize)]
pub struct WorkReportResponseDto {
    pub id: String,
    pub organization_id: String,
    pub building_id: String,
    pub title: String,
    pub description: String,
    pub work_type: WorkType,
    pub contractor_name: String,
    pub contractor_contact: Option<String>,
    pub work_date: String,
    pub completion_date: Option<String>,
    pub cost: f64,
    pub invoice_number: Option<String>,
    pub photos: Vec<String>,
    pub documents: Vec<String>,
    pub notes: Option<String>,
    pub warranty_type: WarrantyType,
    pub warranty_expiry: String,
    pub is_warranty_valid: bool,
    pub warranty_days_remaining: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct AddPhotoDto {
    #[validate(length(min = 1))]
    pub photo_path: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct AddDocumentDto {
    #[validate(length(min = 1))]
    pub document_path: String,
}

#[derive(Debug, Serialize)]
pub struct WorkReportListResponseDto {
    pub work_reports: Vec<WorkReportResponseDto>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
}

#[derive(Debug, Serialize)]
pub struct WarrantyStatusDto {
    pub work_report_id: String,
    pub title: String,
    pub warranty_type: WarrantyType,
    pub warranty_expiry: String,
    pub is_valid: bool,
    pub days_remaining: i64,
}
