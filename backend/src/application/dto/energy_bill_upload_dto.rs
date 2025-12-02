use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::entities::{EnergyBillUpload, EnergyType};

/// DTO for uploading energy bill with GDPR consent
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct UploadEnergyBillRequest {
    pub campaign_id: Uuid,
    pub unit_id: Uuid,

    // Bill details
    pub bill_period_start: DateTime<Utc>,
    pub bill_period_end: DateTime<Utc>,
    pub total_kwh: f64, // Plain text (will be encrypted)
    pub energy_type: EnergyType,
    pub provider: Option<String>,
    pub postal_code: String, // 4-digit Belgian postal code

    // File details
    pub file_hash: String, // SHA-256 of uploaded PDF
    pub file_path: String, // S3 path (will be encrypted)

    // OCR metadata
    pub ocr_confidence: Option<f64>,

    // GDPR Consent (required)
    pub consent: GdprConsentData,
}

/// DTO for GDPR consent data
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct GdprConsentData {
    pub accepted: bool, // Must be true
    pub timestamp: DateTime<Utc>,
    pub ip: String,
    pub user_agent: String,
}

/// DTO for energy bill upload response (anonymized)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct EnergyBillUploadResponse {
    pub id: Uuid,
    pub campaign_id: Uuid,
    pub unit_id: Uuid,
    pub building_id: Uuid,
    pub organization_id: Uuid,

    // Bill period (public)
    pub bill_period_start: DateTime<Utc>,
    pub bill_period_end: DateTime<Utc>,

    // Consumption (encrypted - not exposed)
    // total_kwh_encrypted: NOT INCLUDED (sensitive)
    pub energy_type: EnergyType,
    pub provider: Option<String>,
    pub postal_code: String,

    // File metadata (hashes only)
    pub file_hash: String,
    // file_path_encrypted: NOT INCLUDED (sensitive)
    pub ocr_confidence: f64,
    pub manually_verified: bool,
    pub verified_at: Option<DateTime<Utc>>,

    // GDPR status
    pub consent_timestamp: DateTime<Utc>,
    pub anonymized: bool,
    pub retention_until: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,

    pub uploaded_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<EnergyBillUpload> for EnergyBillUploadResponse {
    fn from(upload: EnergyBillUpload) -> Self {
        Self {
            id: upload.id,
            campaign_id: upload.campaign_id,
            unit_id: upload.unit_id,
            building_id: upload.building_id,
            organization_id: upload.organization_id,
            bill_period_start: upload.bill_period_start,
            bill_period_end: upload.bill_period_end,
            energy_type: upload.energy_type,
            provider: upload.provider,
            postal_code: upload.postal_code,
            file_hash: upload.file_hash,
            ocr_confidence: upload.ocr_confidence,
            manually_verified: upload.manually_verified,
            verified_at: upload.verified_at,
            consent_timestamp: upload.consent_timestamp,
            anonymized: upload.anonymized,
            retention_until: upload.retention_until,
            deleted_at: upload.deleted_at,
            uploaded_at: upload.uploaded_at,
            created_at: upload.created_at,
            updated_at: upload.updated_at,
        }
    }
}

/// DTO for decrypted consumption (owner only)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct DecryptedConsumptionResponse {
    pub upload_id: Uuid,
    pub total_kwh: f64, // Decrypted value
    pub energy_type: EnergyType,
    pub bill_period_start: DateTime<Utc>,
    pub bill_period_end: DateTime<Utc>,
}

/// DTO for manual correction of OCR data
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct CorrectOcrDataRequest {
    pub total_kwh: f64,
    pub energy_type: Option<EnergyType>,
    pub provider: Option<String>,
    pub bill_period_start: Option<DateTime<Utc>>,
    pub bill_period_end: Option<DateTime<Utc>>,
}

/// DTO for verification request (admin)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct VerifyUploadRequest {
    pub verified: bool,
}
