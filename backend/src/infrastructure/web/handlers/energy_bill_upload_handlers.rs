use actix_web::{delete, get, post, put, web, HttpResponse};
use uuid::Uuid;

use crate::application::dto::{
    DecryptedConsumptionResponse, EnergyBillUploadResponse, UploadEnergyBillRequest,
    VerifyUploadRequest,
};
use crate::application::use_cases::EnergyBillUploadUseCases;
use crate::domain::entities::EnergyBillUpload;
use crate::infrastructure::web::middleware::AuthenticatedUser;

// Helper: Get encryption key from environment
fn get_encryption_key() -> Result<[u8; 32], String> {
    let key_hex = std::env::var("ENERGY_ENCRYPTION_MASTER_KEY")
        .map_err(|_| "ENERGY_ENCRYPTION_MASTER_KEY not set".to_string())?;

    if key_hex.len() != 64 {
        return Err("Invalid encryption key length (expected 64 hex chars)".to_string());
    }

    let mut key = [0u8; 32];
    hex::decode_to_slice(&key_hex, &mut key).map_err(|e| format!("Invalid hex key: {}", e))?;

    Ok(key)
}

/// POST /api/v1/energy-bills/upload
/// Upload energy bill with GDPR consent
#[post("/energy-bills/upload")]
pub async fn upload_bill(
    uploads: web::Data<EnergyBillUploadUseCases>,
    request: web::Json<UploadEnergyBillRequest>,
    user: AuthenticatedUser,
) -> Result<HttpResponse, actix_web::Error> {
    // Verify GDPR consent
    if !request.consent.accepted {
        return Err(actix_web::error::ErrorBadRequest("GDPR consent required"));
    }

    let encryption_key =
        get_encryption_key().map_err(actix_web::error::ErrorInternalServerError)?;

    let upload = EnergyBillUpload::new(
        request.campaign_id,
        request.unit_id,
        request.building_id,
        user.organization_id
            .ok_or_else(|| actix_web::error::ErrorUnauthorized("Organization required"))?,
        request.bill_period_start,
        request.bill_period_end,
        request.total_kwh,
        request.energy_type.clone(),
        request.postal_code.clone(),
        request.file_hash.clone(),
        request.file_path.clone(), // Will be encrypted
        user.user_id,
        request.consent.ip.clone(),
        request.consent.user_agent.clone(),
        &encryption_key,
    )
    .map_err(actix_web::error::ErrorBadRequest)?;

    let created = uploads
        .upload_bill(upload)
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    // Note: Aggregation should be triggered asynchronously via message queue in production

    Ok(HttpResponse::Created().json(EnergyBillUploadResponse::from(created)))
}

/// GET /api/v1/energy-bills/my-uploads
/// Get my energy bill uploads
#[get("/energy-bills/my-uploads")]
pub async fn get_my_uploads(
    uploads: web::Data<EnergyBillUploadUseCases>,
    user: AuthenticatedUser,
) -> Result<HttpResponse, actix_web::Error> {
    // TODO: Get unit_id from unit_owners table based on user_id
    // Verify user has organization
    let _organization_id = user
        .organization_id
        .ok_or_else(|| actix_web::error::ErrorUnauthorized("Organization required"))?;

    // Get uploads for user's organization (filtered by repository)
    let unit_id = uuid::Uuid::nil(); // Placeholder, should come from unit_owners table

    let list = uploads
        .get_my_uploads(unit_id)
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    let response: Vec<EnergyBillUploadResponse> = list
        .into_iter()
        .map(EnergyBillUploadResponse::from)
        .collect();

    Ok(HttpResponse::Ok().json(response))
}

/// GET /api/v1/energy-bills/{id}
/// Get upload by ID
#[get("/energy-bills/{id}")]
pub async fn get_upload(
    uploads: web::Data<EnergyBillUploadUseCases>,
    path: web::Path<Uuid>,
    user: AuthenticatedUser,
) -> Result<HttpResponse, actix_web::Error> {
    let id = path.into_inner();

    let upload = uploads
        .get_upload(id)
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?
        .ok_or_else(|| actix_web::error::ErrorNotFound("Upload not found"))?;

    // Verify access (owner or same organization)
    if upload.organization_id
        != user
            .organization_id
            .ok_or_else(|| actix_web::error::ErrorForbidden("Organization ID required"))?
    {
        return Err(actix_web::error::ErrorForbidden("Access denied"));
    }

    Ok(HttpResponse::Ok().json(EnergyBillUploadResponse::from(upload)))
}

/// GET /api/v1/energy-bills/{id}/decrypt
/// Decrypt consumption data (owner only)
#[get("/energy-bills/{id}/decrypt")]
pub async fn decrypt_consumption(
    uploads: web::Data<EnergyBillUploadUseCases>,
    path: web::Path<Uuid>,
    user: AuthenticatedUser,
) -> Result<HttpResponse, actix_web::Error> {
    let upload_id = path.into_inner();

    let encryption_key =
        get_encryption_key().map_err(actix_web::error::ErrorInternalServerError)?;

    // Get upload to check ownership
    let upload = uploads
        .get_upload(upload_id)
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?
        .ok_or_else(|| actix_web::error::ErrorNotFound("Upload not found"))?;

    // Verify organization access
    let user_org = user
        .organization_id
        .ok_or_else(|| actix_web::error::ErrorUnauthorized("Organization required"))?;
    if upload.organization_id != user_org {
        return Err(actix_web::error::ErrorForbidden("Access denied"));
    }

    let total_kwh = uploads
        .decrypt_consumption(upload_id, upload.unit_id, &encryption_key)
        .await
        .map_err(actix_web::error::ErrorForbidden)?;

    let response = DecryptedConsumptionResponse {
        upload_id,
        total_kwh,
        energy_type: upload.energy_type,
        bill_period_start: upload.bill_period_start,
        bill_period_end: upload.bill_period_end,
    };

    Ok(HttpResponse::Ok().json(response))
}

/// PUT /api/v1/energy-bills/{id}/verify
/// Verify upload (admin only)
#[put("/energy-bills/{id}/verify")]
pub async fn verify_upload(
    uploads: web::Data<EnergyBillUploadUseCases>,
    path: web::Path<Uuid>,
    _request: web::Json<VerifyUploadRequest>,
    user: AuthenticatedUser,
) -> Result<HttpResponse, actix_web::Error> {
    let upload_id = path.into_inner();

    // TODO: Add admin role check
    // if !user.is_admin() {
    //     return Err(actix_web::error::ErrorForbidden("Admin access required"));
    // }

    let updated = uploads
        .verify_upload(upload_id, user.user_id)
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(EnergyBillUploadResponse::from(updated)))
}

/// DELETE /api/v1/energy-bills/{id}
/// Delete upload (GDPR Art. 17 - Right to erasure)
#[delete("/energy-bills/{id}")]
pub async fn delete_upload(
    uploads: web::Data<EnergyBillUploadUseCases>,
    path: web::Path<Uuid>,
    user: AuthenticatedUser,
) -> Result<HttpResponse, actix_web::Error> {
    let upload_id = path.into_inner();

    // Get upload to verify ownership
    let upload = uploads
        .get_upload(upload_id)
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?
        .ok_or_else(|| actix_web::error::ErrorNotFound("Upload not found"))?;

    // Verify organization access
    let user_org = user
        .organization_id
        .ok_or_else(|| actix_web::error::ErrorUnauthorized("Organization required"))?;
    if upload.organization_id != user_org {
        return Err(actix_web::error::ErrorForbidden("Access denied"));
    }

    uploads
        .delete_upload(upload_id, upload.unit_id)
        .await
        .map_err(actix_web::error::ErrorForbidden)?;

    Ok(HttpResponse::NoContent().finish())
}

/// POST /api/v1/energy-bills/{id}/withdraw-consent
/// Withdraw GDPR consent (Art. 7.3 - Immediate deletion)
#[post("/energy-bills/{id}/withdraw-consent")]
pub async fn withdraw_consent(
    uploads: web::Data<EnergyBillUploadUseCases>,
    path: web::Path<Uuid>,
    user: AuthenticatedUser,
) -> Result<HttpResponse, actix_web::Error> {
    let upload_id = path.into_inner();

    // Get upload to verify ownership
    let upload = uploads
        .get_upload(upload_id)
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?
        .ok_or_else(|| actix_web::error::ErrorNotFound("Upload not found"))?;

    // Verify organization access
    let user_org = user
        .organization_id
        .ok_or_else(|| actix_web::error::ErrorUnauthorized("Organization required"))?;
    if upload.organization_id != user_org {
        return Err(actix_web::error::ErrorForbidden("Access denied"));
    }

    uploads
        .withdraw_consent(upload_id, upload.unit_id)
        .await
        .map_err(actix_web::error::ErrorForbidden)?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Consent withdrawn and data deleted",
        "gdpr_article": "7.3 - Right to withdraw consent"
    })))
}

/// GET /api/v1/energy-campaigns/{campaign_id}/uploads
/// Get all uploads for a campaign (admin)
#[get("/energy-campaigns/{campaign_id}/uploads")]
pub async fn get_campaign_uploads(
    uploads: web::Data<EnergyBillUploadUseCases>,
    path: web::Path<Uuid>,
    user: AuthenticatedUser,
) -> Result<HttpResponse, actix_web::Error> {
    let campaign_id = path.into_inner();

    let list = uploads
        .get_uploads_by_campaign(campaign_id)
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    // Verify organization access
    if !list.is_empty() {
        let user_org = user
            .organization_id
            .ok_or_else(|| actix_web::error::ErrorUnauthorized("Organization required"))?;
        if list[0].organization_id != user_org {
            return Err(actix_web::error::ErrorForbidden("Access denied"));
        }
    }

    let response: Vec<EnergyBillUploadResponse> = list
        .into_iter()
        .map(EnergyBillUploadResponse::from)
        .collect();

    Ok(HttpResponse::Ok().json(response))
}
