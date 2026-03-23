use actix_web::{delete, post, web, HttpResponse};
use uuid::Uuid;

use crate::application::dto::{
    JoinCampaignAsIndividualDto, IndividualMemberResponseDto, GrantConsentDto,
    UpdateConsumptionDto, UnsubscribeRequestDto, UnsubscribeConfirmationDto,
};
use crate::domain::entities::IndividualMember;
use crate::infrastructure::web::AppState;

/// POST /api/v1/energy-campaigns/{campaign_id}/join-as-individual
/// Join energy campaign as individual member (no authentication required)
/// Issue #280: Energy group buying extensions
#[post("/energy-campaigns/{campaign_id}/join-as-individual")]
pub async fn join_campaign_as_individual(
    state: web::Data<AppState>,
    campaign_id: web::Path<Uuid>,
    request: web::Json<JoinCampaignAsIndividualDto>,
) -> Result<HttpResponse, actix_web::Error> {
    let campaign_id = campaign_id.into_inner();

    // TODO: Verify campaign exists and is open to individuals (audience_type != 'CoProprietiesOnly')

    // Create individual member
    let member = IndividualMember::new(
        campaign_id,
        request.email.clone(),
        request.postal_code.clone(),
    )
    .map_err(actix_web::error::ErrorBadRequest)?;

    // TODO: Save to database using repository
    // TODO: Check for duplicate email in campaign (UNIQUE constraint will handle)
    // TODO: Send confirmation email with consent link

    let response = IndividualMemberResponseDto::from(member);
    Ok(HttpResponse::Created().json(response))
}

/// POST /api/v1/energy-campaigns/{campaign_id}/members/{member_id}/consent
/// Grant GDPR consent for campaign participation
#[post("/energy-campaigns/{campaign_id}/members/{member_id}/consent")]
pub async fn grant_consent(
    state: web::Data<AppState>,
    path: web::Path<(Uuid, Uuid)>,
    request: web::Json<GrantConsentDto>,
) -> Result<HttpResponse, actix_web::Error> {
    let (campaign_id, member_id) = path.into_inner();

    // TODO: Fetch member from database
    // TODO: Verify campaign_id matches
    // TODO: Update has_gdpr_consent = true
    // TODO: Log consent in gdpr_art30_register for audit trail

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "message": "Consent granted successfully"
    })))
}

/// PUT /api/v1/energy-campaigns/{campaign_id}/members/{member_id}/consumption
/// Update consumption data for member
#[put("/energy-campaigns/{campaign_id}/members/{member_id}/consumption")]
pub async fn update_consumption(
    state: web::Data<AppState>,
    path: web::Path<(Uuid, Uuid)>,
    request: web::Json<UpdateConsumptionDto>,
) -> Result<HttpResponse, actix_web::Error> {
    let (campaign_id, member_id) = path.into_inner();

    // TODO: Fetch member from database
    // TODO: Verify campaign_id matches
    // TODO: Update consumption data
    // TODO: Validate kwh >= 0

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "message": "Consumption data updated"
    })))
}

/// DELETE /api/v1/energy-campaigns/{campaign_id}/members/{member_id}/withdraw
/// Withdraw from campaign (GDPR right to erasure preparation)
/// TODO: Requires email token or authenticated user
#[delete("/energy-campaigns/{campaign_id}/members/{member_id}/withdraw")]
pub async fn withdraw_from_campaign(
    state: web::Data<AppState>,
    path: web::Path<(Uuid, Uuid)>,
) -> Result<HttpResponse, actix_web::Error> {
    let (campaign_id, member_id) = path.into_inner();

    // TODO: Fetch member from database
    // TODO: Verify campaign_id matches
    // TODO: Mark unsubscribed_at = NOW()
    // TODO: Schedule data anonymization/deletion (GDPR Article 17)

    let response = UnsubscribeConfirmationDto {
        success: true,
        message: "Successfully withdrawn from campaign".to_string(),
        email: "user@example.com".to_string(), // TODO: Get from member
    };

    Ok(HttpResponse::Ok().json(response))
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_individual_member_handlers_compile() {
        // Placeholder test to verify handler structure
    }
}
