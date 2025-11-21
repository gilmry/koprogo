use crate::domain::entities::{EtatDate, EtatDateLanguage, EtatDateStatus};
use chrono::{DateTime, Utc};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Request pour créer un nouvel état daté
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateEtatDateRequest {
    pub organization_id: Uuid, // Will be overridden by JWT token
    pub building_id: Uuid,
    pub unit_id: Uuid,
    pub reference_date: DateTime<Utc>,
    pub language: EtatDateLanguage,
    pub notary_name: String,
    pub notary_email: String,
    pub notary_phone: Option<String>,
}

/// Request pour mettre à jour les données financières d'un état daté
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateEtatDateFinancialRequest {
    pub owner_balance: f64,
    pub arrears_amount: f64,
    pub monthly_provision_amount: f64,
    pub total_balance: f64,
    pub approved_works_unpaid: f64,
}

/// Request pour mettre à jour les données additionnelles (sections 7-16)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateEtatDateAdditionalDataRequest {
    pub additional_data: serde_json::Value,
}

/// Response DTO pour un état daté
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EtatDateResponse {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub building_id: Uuid,
    pub unit_id: Uuid,
    pub reference_date: DateTime<Utc>,
    pub requested_date: DateTime<Utc>,
    pub generated_date: Option<DateTime<Utc>>,
    pub delivered_date: Option<DateTime<Utc>>,
    pub status: EtatDateStatus,
    pub language: EtatDateLanguage,
    pub reference_number: String,
    pub notary_name: String,
    pub notary_email: String,
    pub notary_phone: Option<String>,
    pub building_name: String,
    pub building_address: String,
    pub unit_number: String,
    pub unit_floor: Option<String>,
    pub unit_area: Option<f64>,
    pub ordinary_charges_quota: f64,
    pub extraordinary_charges_quota: f64,
    pub owner_balance: f64,
    pub arrears_amount: f64,
    pub monthly_provision_amount: f64,
    pub total_balance: f64,
    pub approved_works_unpaid: f64,
    pub additional_data: serde_json::Value,
    pub pdf_file_path: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    // Computed fields
    pub is_overdue: bool,
    pub is_expired: bool,
    pub days_since_request: i64,
}

impl From<EtatDate> for EtatDateResponse {
    fn from(etat_date: EtatDate) -> Self {
        let is_overdue = etat_date.is_overdue();
        let is_expired = etat_date.is_expired();
        let days_since_request = etat_date.days_since_request();

        Self {
            id: etat_date.id,
            organization_id: etat_date.organization_id,
            building_id: etat_date.building_id,
            unit_id: etat_date.unit_id,
            reference_date: etat_date.reference_date,
            requested_date: etat_date.requested_date,
            generated_date: etat_date.generated_date,
            delivered_date: etat_date.delivered_date,
            status: etat_date.status,
            language: etat_date.language,
            reference_number: etat_date.reference_number,
            notary_name: etat_date.notary_name,
            notary_email: etat_date.notary_email,
            notary_phone: etat_date.notary_phone,
            building_name: etat_date.building_name,
            building_address: etat_date.building_address,
            unit_number: etat_date.unit_number,
            unit_floor: etat_date.unit_floor,
            unit_area: etat_date.unit_area,
            ordinary_charges_quota: etat_date.ordinary_charges_quota,
            extraordinary_charges_quota: etat_date.extraordinary_charges_quota,
            owner_balance: etat_date.owner_balance,
            arrears_amount: etat_date.arrears_amount,
            monthly_provision_amount: etat_date.monthly_provision_amount,
            total_balance: etat_date.total_balance,
            approved_works_unpaid: etat_date.approved_works_unpaid,
            additional_data: etat_date.additional_data,
            pdf_file_path: etat_date.pdf_file_path,
            created_at: etat_date.created_at,
            updated_at: etat_date.updated_at,
            is_overdue,
            is_expired,
            days_since_request,
        }
    }
}

/// Response avec statistiques pour dashboard syndic
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EtatDateStatsResponse {
    pub total_requests: i64,
    pub requested_count: i64,
    pub in_progress_count: i64,
    pub generated_count: i64,
    pub delivered_count: i64,
    pub expired_count: i64,
    pub overdue_count: i64, // En retard (>10 jours)
    pub average_processing_days: f64,
}
