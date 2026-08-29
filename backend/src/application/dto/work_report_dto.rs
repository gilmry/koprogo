//! Work report DTOs — les montants utilisent `rust_decimal::Decimal` (ADR-0007/0008).
//!
//! Deux notes qui expliquent les attributs serde ci-dessous :
//!
//! 1. `validator` ne sait pas borner un `Decimal` (`#[validate(range(...))]`
//!    n'accepte que des littéraux flottants). L'invariant « coût ≥ 0 » descend
//!    donc dans le domaine — `WorkReport::new` / `WorkReport::set_cost` — où il
//!    couvre tous les appelants et plus seulement la route HTTP.
//!
//! 2. Un `Decimal` nu sérialise en **chaîne** JSON (`"1500.00"`), là où le type
//!    d'avant produisait un nombre (`1500.0`). `serde::float` /
//!    `serde::float_option` préservent la représentation numérique : la
//!    conversion ne provoque donc AUCUN drift de contrat API. Le `default` sur
//!    les champs optionnels est indispensable : `#[serde(with = ...)]` fait
//!    perdre le défaut implicite d'un `Option` absent, et un `cost` omis dans
//!    une mise à jour partielle deviendrait un 400 « missing field ».

use crate::domain::entities::{WarrantyType, WorkType};
use rust_decimal::Decimal;
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

    #[serde(with = "rust_decimal::serde::float")]
    pub cost: Decimal,

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

    #[serde(default, with = "rust_decimal::serde::float_option")]
    pub cost: Option<Decimal>,

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
    #[serde(with = "rust_decimal::serde::float")]
    pub cost: Decimal,
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
