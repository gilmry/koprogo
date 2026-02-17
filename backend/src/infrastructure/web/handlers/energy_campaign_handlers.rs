use actix_web::{delete, get, post, put, web, HttpResponse};
use uuid::Uuid;

use crate::application::dto::{
    CampaignStatsResponse, CreateEnergyCampaignRequest, CreateProviderOfferRequest,
    EnergyCampaignResponse, ProviderOfferResponse, SelectOfferRequest, UpdateCampaignStatusRequest,
};
use crate::domain::entities::EnergyCampaign;
use crate::infrastructure::web::middleware::AuthenticatedUser;
use crate::infrastructure::web::AppState;

/// POST /api/v1/energy-campaigns
/// Create a new energy campaign
#[post("/energy-campaigns")]
pub async fn create_campaign(
    state: web::Data<AppState>,
    request: web::Json<CreateEnergyCampaignRequest>,
    user: AuthenticatedUser,
) -> Result<HttpResponse, actix_web::Error> {
    let org_id = user
        .organization_id
        .ok_or_else(|| actix_web::error::ErrorBadRequest("Organization ID required"))?;

    let campaign = EnergyCampaign::new(
        org_id,
        request.building_id,
        request.campaign_name.clone(),
        request.deadline_participation,
        request.energy_types.clone(),
        user.user_id,
    )
    .map_err(actix_web::error::ErrorBadRequest)?;

    let created = state
        .energy_campaign_use_cases
        .create_campaign(campaign)
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Created().json(EnergyCampaignResponse::from(created)))
}

/// GET /api/v1/energy-campaigns
/// List all campaigns for current organization
#[get("/energy-campaigns")]
pub async fn list_campaigns(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
) -> Result<HttpResponse, actix_web::Error> {
    let org_id = user
        .organization_id
        .ok_or_else(|| actix_web::error::ErrorBadRequest("Organization ID required"))?;

    let list = state
        .energy_campaign_use_cases
        .get_campaigns_by_organization(org_id)
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    let response: Vec<EnergyCampaignResponse> =
        list.into_iter().map(EnergyCampaignResponse::from).collect();

    Ok(HttpResponse::Ok().json(response))
}

/// GET /api/v1/energy-campaigns/{id}
/// Get campaign by ID
#[get("/energy-campaigns/{id}")]
pub async fn get_campaign(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
    user: AuthenticatedUser,
) -> Result<HttpResponse, actix_web::Error> {
    let id = path.into_inner();

    let campaign = state
        .energy_campaign_use_cases
        .get_campaign(id)
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?
        .ok_or_else(|| actix_web::error::ErrorNotFound("Campaign not found"))?;

    // Verify organization access
    if campaign.organization_id
        != user
            .organization_id
            .ok_or_else(|| actix_web::error::ErrorForbidden("Organization ID required"))?
    {
        return Err(actix_web::error::ErrorForbidden("Access denied"));
    }

    Ok(HttpResponse::Ok().json(EnergyCampaignResponse::from(campaign)))
}

/// PUT /api/v1/energy-campaigns/{id}/status
/// Update campaign status
#[put("/energy-campaigns/{id}/status")]
pub async fn update_campaign_status(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
    request: web::Json<UpdateCampaignStatusRequest>,
    user: AuthenticatedUser,
) -> Result<HttpResponse, actix_web::Error> {
    let id = path.into_inner();

    // Verify ownership
    let campaign = state
        .energy_campaign_use_cases
        .get_campaign(id)
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?
        .ok_or_else(|| actix_web::error::ErrorNotFound("Campaign not found"))?;

    if campaign.organization_id
        != user
            .organization_id
            .ok_or_else(|| actix_web::error::ErrorForbidden("Organization ID required"))?
    {
        return Err(actix_web::error::ErrorForbidden("Access denied"));
    }

    let updated = state
        .energy_campaign_use_cases
        .update_campaign_status(id, request.status.clone())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(EnergyCampaignResponse::from(updated)))
}

/// GET /api/v1/energy-campaigns/{id}/stats
/// Get campaign statistics (anonymized)
#[get("/energy-campaigns/{id}/stats")]
pub async fn get_campaign_stats(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
    user: AuthenticatedUser,
) -> Result<HttpResponse, actix_web::Error> {
    let id = path.into_inner();

    // Verify ownership
    let campaign = state
        .energy_campaign_use_cases
        .get_campaign(id)
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?
        .ok_or_else(|| actix_web::error::ErrorNotFound("Campaign not found"))?;

    if campaign.organization_id
        != user
            .organization_id
            .ok_or_else(|| actix_web::error::ErrorForbidden("Organization ID required"))?
    {
        return Err(actix_web::error::ErrorForbidden("Access denied"));
    }

    let stats = state
        .energy_campaign_use_cases
        .get_campaign_stats(id)
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    let response = CampaignStatsResponse {
        total_participants: stats.total_participants,
        participation_rate: stats.participation_rate,
        total_kwh_electricity: stats.total_kwh_electricity,
        total_kwh_gas: stats.total_kwh_gas,
        avg_kwh_per_unit: stats.avg_kwh_per_unit,
        can_negotiate: stats.can_negotiate,
        estimated_savings_pct: stats.estimated_savings_pct,
        k_anonymity_met: stats.total_participants >= 5,
    };

    Ok(HttpResponse::Ok().json(response))
}

/// POST /api/v1/energy-campaigns/{id}/offers
/// Add provider offer (broker/admin only)
#[post("/energy-campaigns/{id}/offers")]
pub async fn add_offer(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
    request: web::Json<CreateProviderOfferRequest>,
    user: AuthenticatedUser,
) -> Result<HttpResponse, actix_web::Error> {
    let campaign_id = path.into_inner();

    // Verify ownership
    let campaign = state
        .energy_campaign_use_cases
        .get_campaign(campaign_id)
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?
        .ok_or_else(|| actix_web::error::ErrorNotFound("Campaign not found"))?;

    if campaign.organization_id
        != user
            .organization_id
            .ok_or_else(|| actix_web::error::ErrorForbidden("Organization ID required"))?
    {
        return Err(actix_web::error::ErrorForbidden("Access denied"));
    }

    use crate::domain::entities::ProviderOffer;

    let offer = ProviderOffer::new(
        campaign_id,
        request.provider_name.clone(),
        request.price_kwh_electricity,
        request.price_kwh_gas,
        request.fixed_monthly_fee,
        request.green_energy_pct,
        request.contract_duration_months,
        request.estimated_savings_pct,
        request.offer_valid_until,
    )
    .map_err(actix_web::error::ErrorBadRequest)?;

    let created = state
        .energy_campaign_use_cases
        .add_offer(campaign_id, offer)
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Created().json(ProviderOfferResponse::from(created)))
}

/// GET /api/v1/energy-campaigns/{id}/offers
/// List all offers for a campaign
#[get("/energy-campaigns/{id}/offers")]
pub async fn list_offers(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
    user: AuthenticatedUser,
) -> Result<HttpResponse, actix_web::Error> {
    let campaign_id = path.into_inner();

    // Verify ownership
    let campaign = state
        .energy_campaign_use_cases
        .get_campaign(campaign_id)
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?
        .ok_or_else(|| actix_web::error::ErrorNotFound("Campaign not found"))?;

    if campaign.organization_id
        != user
            .organization_id
            .ok_or_else(|| actix_web::error::ErrorForbidden("Organization ID required"))?
    {
        return Err(actix_web::error::ErrorForbidden("Access denied"));
    }

    let offers = state
        .energy_campaign_use_cases
        .get_campaign_offers(campaign_id)
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    let response: Vec<ProviderOfferResponse> = offers
        .into_iter()
        .map(ProviderOfferResponse::from)
        .collect();

    Ok(HttpResponse::Ok().json(response))
}

/// POST /api/v1/energy-campaigns/{id}/select-offer
/// Select winning offer (after vote)
#[post("/energy-campaigns/{id}/select-offer")]
pub async fn select_offer(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
    request: web::Json<SelectOfferRequest>,
    user: AuthenticatedUser,
) -> Result<HttpResponse, actix_web::Error> {
    let campaign_id = path.into_inner();

    // Verify ownership
    let campaign = state
        .energy_campaign_use_cases
        .get_campaign(campaign_id)
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?
        .ok_or_else(|| actix_web::error::ErrorNotFound("Campaign not found"))?;

    if campaign.organization_id
        != user
            .organization_id
            .ok_or_else(|| actix_web::error::ErrorForbidden("Organization ID required"))?
    {
        return Err(actix_web::error::ErrorForbidden("Access denied"));
    }

    let updated = state
        .energy_campaign_use_cases
        .select_offer(campaign_id, request.offer_id)
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(EnergyCampaignResponse::from(updated)))
}

/// POST /api/v1/energy-campaigns/{id}/finalize
/// Finalize campaign (after final vote)
#[post("/energy-campaigns/{id}/finalize")]
pub async fn finalize_campaign(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
    user: AuthenticatedUser,
) -> Result<HttpResponse, actix_web::Error> {
    let campaign_id = path.into_inner();

    // Verify ownership
    let campaign = state
        .energy_campaign_use_cases
        .get_campaign(campaign_id)
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?
        .ok_or_else(|| actix_web::error::ErrorNotFound("Campaign not found"))?;

    if campaign.organization_id
        != user
            .organization_id
            .ok_or_else(|| actix_web::error::ErrorForbidden("Organization ID required"))?
    {
        return Err(actix_web::error::ErrorForbidden("Access denied"));
    }

    let updated = state
        .energy_campaign_use_cases
        .finalize_campaign(campaign_id)
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(EnergyCampaignResponse::from(updated)))
}

/// DELETE /api/v1/energy-campaigns/{id}
/// Delete campaign
#[delete("/energy-campaigns/{id}")]
pub async fn delete_campaign(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
    user: AuthenticatedUser,
) -> Result<HttpResponse, actix_web::Error> {
    let campaign_id = path.into_inner();

    // Verify ownership
    let campaign = state
        .energy_campaign_use_cases
        .get_campaign(campaign_id)
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?
        .ok_or_else(|| actix_web::error::ErrorNotFound("Campaign not found"))?;

    if campaign.organization_id
        != user
            .organization_id
            .ok_or_else(|| actix_web::error::ErrorForbidden("Organization ID required"))?
    {
        return Err(actix_web::error::ErrorForbidden("Access denied"));
    }

    state
        .energy_campaign_use_cases
        .delete_campaign(campaign_id)
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::NoContent().finish())
}