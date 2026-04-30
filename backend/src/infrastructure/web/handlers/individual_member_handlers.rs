use actix_web::{delete, post, put, web, HttpResponse};
use uuid::Uuid;

use crate::application::dto::{
    GrantConsentDto, JoinCampaignAsIndividualDto, UnsubscribeConfirmationDto, UpdateConsumptionDto,
};
use crate::infrastructure::web::AppState;

#[post("/energy-campaigns/{campaign_id}/join-as-individual")]
pub async fn join_campaign_as_individual(
    state: web::Data<AppState>,
    campaign_id: web::Path<Uuid>,
    request: web::Json<JoinCampaignAsIndividualDto>,
) -> Result<HttpResponse, actix_web::Error> {
    let campaign_id = campaign_id.into_inner();
    let response = state
        .individual_member_use_cases
        .join_campaign(
            campaign_id,
            request.email.clone(),
            request.postal_code.clone(),
        )
        .await
        .map_err(actix_web::error::ErrorBadRequest)?;
    Ok(HttpResponse::Created().json(response))
}

#[post("/energy-campaigns/{campaign_id}/members/{member_id}/consent")]
pub async fn grant_consent(
    state: web::Data<AppState>,
    path: web::Path<(Uuid, Uuid)>,
    _request: web::Json<GrantConsentDto>,
) -> Result<HttpResponse, actix_web::Error> {
    let (campaign_id, member_id) = path.into_inner();
    let response = state
        .individual_member_use_cases
        .grant_consent(campaign_id, member_id)
        .await
        .map_err(actix_web::error::ErrorBadRequest)?;
    Ok(HttpResponse::Ok().json(response))
}

#[put("/energy-campaigns/{campaign_id}/members/{member_id}/consumption")]
pub async fn update_consumption(
    state: web::Data<AppState>,
    path: web::Path<(Uuid, Uuid)>,
    request: web::Json<UpdateConsumptionDto>,
) -> Result<HttpResponse, actix_web::Error> {
    let (campaign_id, member_id) = path.into_inner();
    let response = state
        .individual_member_use_cases
        .update_consumption(
            campaign_id,
            member_id,
            Some(request.annual_consumption_kwh),
            request.current_provider.clone(),
            request.ean_code.clone(),
        )
        .await
        .map_err(actix_web::error::ErrorBadRequest)?;
    Ok(HttpResponse::Ok().json(response))
}

#[delete("/energy-campaigns/{campaign_id}/members/{member_id}/withdraw")]
pub async fn withdraw_from_campaign(
    state: web::Data<AppState>,
    path: web::Path<(Uuid, Uuid)>,
) -> Result<HttpResponse, actix_web::Error> {
    let (campaign_id, member_id) = path.into_inner();
    let email = state
        .individual_member_use_cases
        .withdraw(campaign_id, member_id)
        .await
        .map_err(actix_web::error::ErrorBadRequest)?;
    let response = UnsubscribeConfirmationDto {
        success: true,
        message: "Successfully withdrawn from campaign".to_string(),
        email,
    };
    Ok(HttpResponse::Ok().json(response))
}
