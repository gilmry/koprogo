use crate::application::dto::{
    CreateOwnerContributionRequest, OwnerContributionResponse, RecordPaymentRequest,
};
use crate::infrastructure::web::{AppState, AuthenticatedUser};
use actix_web::{get, post, put, web, HttpResponse};
use uuid::Uuid;

/// POST /api/v1/owner-contributions
/// Create a new owner contribution
#[post("/owner-contributions")]
pub async fn create_contribution(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    req: web::Json<CreateOwnerContributionRequest>,
) -> HttpResponse {
    // Get organization_id from user (required for creating contributions)
    let organization_id = match user.organization_id {
        Some(org_id) => org_id,
        None => return HttpResponse::BadRequest().body("Organization ID required"),
    };

    match state
        .owner_contribution_use_cases
        .create_contribution(
            organization_id,
            req.owner_id,
            req.unit_id,
            req.description.clone(),
            req.amount,
            req.contribution_type.clone(),
            req.contribution_date,
            req.account_code.clone(),
        )
        .await
    {
        Ok(contribution) => {
            let response = OwnerContributionResponse::from(contribution);
            HttpResponse::Created().json(response)
        }
        Err(e) => HttpResponse::BadRequest().body(e),
    }
}

/// GET /api/v1/owner-contributions/{id}
/// Get contribution by ID
#[get("/owner-contributions/{id}")]
pub async fn get_contribution(
    state: web::Data<AppState>,
    _user: AuthenticatedUser,
    id: web::Path<Uuid>,
) -> HttpResponse {
    match state
        .owner_contribution_use_cases
        .get_contribution(*id)
        .await
    {
        Ok(Some(contribution)) => {
            let response = OwnerContributionResponse::from(contribution);
            HttpResponse::Ok().json(response)
        }
        Ok(None) => HttpResponse::NotFound().body("Contribution not found"),
        Err(e) => HttpResponse::InternalServerError().body(e),
    }
}

/// GET /api/v1/owner-contributions?owner_id={uuid}
/// Get contributions by owner, or all contributions for organization if owner_id not provided
#[get("/owner-contributions")]
pub async fn get_contributions_by_owner(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> HttpResponse {
    // If owner_id provided, filter by owner
    if let Some(id_str) = query.get("owner_id") {
        let owner_id = match Uuid::parse_str(id_str) {
            Ok(id) => id,
            Err(_) => return HttpResponse::BadRequest().body("Invalid owner_id format"),
        };

        match state
            .owner_contribution_use_cases
            .get_contributions_by_owner(owner_id)
            .await
        {
            Ok(contributions) => {
                let responses: Vec<OwnerContributionResponse> =
                    contributions.into_iter().map(Into::into).collect();
                return HttpResponse::Ok().json(responses);
            }
            Err(e) => return HttpResponse::InternalServerError().body(e),
        }
    }

    // Otherwise, return all contributions for user's organization
    let organization_id = match user.organization_id {
        Some(org_id) => org_id,
        None => return HttpResponse::BadRequest().body("Organization ID required"),
    };

    match state
        .owner_contribution_use_cases
        .get_contributions_by_organization(organization_id)
        .await
    {
        Ok(contributions) => {
            let responses: Vec<OwnerContributionResponse> =
                contributions.into_iter().map(Into::into).collect();
            HttpResponse::Ok().json(responses)
        }
        Err(e) => HttpResponse::InternalServerError().body(e),
    }
}

/// GET /api/v1/owner-contributions/outstanding?owner_id={uuid}
/// Get outstanding (unpaid) contributions for an owner
#[get("/owner-contributions/outstanding")]
pub async fn get_outstanding_contributions(
    state: web::Data<AppState>,
    _user: AuthenticatedUser,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> HttpResponse {
    let owner_id = match query.get("owner_id") {
        Some(id_str) => match Uuid::parse_str(id_str) {
            Ok(id) => id,
            Err(_) => return HttpResponse::BadRequest().body("Invalid owner_id format"),
        },
        None => return HttpResponse::BadRequest().body("owner_id is required"),
    };

    match state
        .owner_contribution_use_cases
        .get_outstanding_contributions(owner_id)
        .await
    {
        Ok(contributions) => {
            let responses: Vec<OwnerContributionResponse> =
                contributions.into_iter().map(Into::into).collect();
            HttpResponse::Ok().json(responses)
        }
        Err(e) => HttpResponse::InternalServerError().body(e),
    }
}

/// PUT /api/v1/owner-contributions/{id}/mark-paid
/// Record payment for a contribution
#[put("/owner-contributions/{id}/mark-paid")]
pub async fn record_payment(
    state: web::Data<AppState>,
    _user: AuthenticatedUser,
    id: web::Path<Uuid>,
    req: web::Json<RecordPaymentRequest>,
) -> HttpResponse {
    match state
        .owner_contribution_use_cases
        .record_payment(
            *id,
            req.payment_date,
            req.payment_method.clone(),
            req.payment_reference.clone(),
        )
        .await
    {
        Ok(contribution) => {
            let response = OwnerContributionResponse::from(contribution);
            HttpResponse::Ok().json(response)
        }
        Err(e) => HttpResponse::BadRequest().body(e),
    }
}
