use crate::application::dto::{
    CreateOwnerContributionRequest, OwnerContributionResponse, RecordPaymentRequest,
};
use crate::infrastructure::web::{AppState, AuthenticatedUser};
use actix_web::{get, post, put, web, HttpResponse};
use uuid::Uuid;

/// POST /api/v1/owner-contributions
/// Create a new owner contribution
#[utoipa::path(
    post,
    path = "/owner-contributions",
    tag = "OwnerContributions",
    summary = "Create an owner contribution (quote-part)",
    request_body = CreateOwnerContributionRequest,
    responses(
        (status = 201, description = "Contribution created", body = OwnerContributionResponse),
        (status = 400, description = "Validation error, or unknown field in the body"),
        (status = 401, description = "User does not belong to an organization"),
    ),
    security(("bearer_auth" = []))
)]
#[post("/owner-contributions")]
pub async fn create_contribution(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    req: web::Json<CreateOwnerContributionRequest>,
) -> HttpResponse {
    // Get organization_id from user (required for creating contributions)
    let organization_id = match user.organization_id {
        Some(org_id) => org_id,
        None => return HttpResponse::BadRequest().json(serde_json::json!({ "error": "Organization ID required" })),
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
        Err(e) => HttpResponse::BadRequest().json(serde_json::json!({ "error": e })),
    }
}

/// GET /api/v1/owner-contributions/{id}
/// Get contribution by ID
#[utoipa::path(
    get,
    path = "/owner-contributions/{id}",
    tag = "OwnerContributions",
    summary = "Get a single owner contribution",
    params(("id" = Uuid, Path, description = "Contribution identifier")),
    responses(
        (status = 200, description = "Contribution", body = OwnerContributionResponse),
        (status = 404, description = "Contribution not found"),
    ),
    security(("bearer_auth" = []))
)]
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
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({ "error": "Contribution not found" })),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({ "error": e })),
    }
}

/// GET /api/v1/owner-contributions?owner_id={uuid}
/// Get contributions by owner, or all contributions for organization if owner_id not provided
#[utoipa::path(
    get,
    path = "/owner-contributions",
    tag = "OwnerContributions",
    summary = "List contributions of the organization, or of a single owner",
    params(("owner_id" = Option<Uuid>, Query, description = "Restrict to one owner")),
    responses(
        (status = 200, description = "Contributions", body = Vec<OwnerContributionResponse>),
        (status = 401, description = "User does not belong to an organization"),
    ),
    security(("bearer_auth" = []))
)]
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
            Err(_) => return HttpResponse::BadRequest().json(serde_json::json!({ "error": "Invalid owner_id format" })),
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
            Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({ "error": e })),
        }
    }

    // Otherwise, return all contributions for user's organization
    let organization_id = match user.organization_id {
        Some(org_id) => org_id,
        None => return HttpResponse::BadRequest().json(serde_json::json!({ "error": "Organization ID required" })),
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
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({ "error": e })),
    }
}

/// GET /api/v1/owner-contributions/outstanding?owner_id={uuid}
/// Get outstanding (unpaid) contributions for an owner
#[utoipa::path(
    get,
    path = "/owner-contributions/outstanding",
    tag = "OwnerContributions",
    summary = "List unpaid contributions",
    responses(
        (status = 200, description = "Outstanding contributions", body = Vec<OwnerContributionResponse>),
        (status = 401, description = "User does not belong to an organization"),
    ),
    security(("bearer_auth" = []))
)]
#[get("/owner-contributions/outstanding")]
pub async fn get_outstanding_contributions(
    state: web::Data<AppState>,
    _user: AuthenticatedUser,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> HttpResponse {
    let owner_id = match query.get("owner_id") {
        Some(id_str) => match Uuid::parse_str(id_str) {
            Ok(id) => id,
            Err(_) => return HttpResponse::BadRequest().json(serde_json::json!({ "error": "Invalid owner_id format" })),
        },
        None => return HttpResponse::BadRequest().json(serde_json::json!({ "error": "owner_id is required" })),
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
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({ "error": e })),
    }
}

/// PUT /api/v1/owner-contributions/{id}/mark-paid
/// Record payment for a contribution
#[utoipa::path(
    put,
    path = "/owner-contributions/{id}/mark-paid",
    tag = "OwnerContributions",
    summary = "Record a payment against a contribution",
    description = "Voie SUPPORTEE pour solder une quote-part depuis l'interface. \
Un paiement du module `/payments` peut aussi la solder automatiquement : \
il suffit de lui passer `contribution_id`, et la quote-part bascule quand \
le paiement atteint `succeeded`.",
    params(("id" = Uuid, Path, description = "Contribution identifier")),
    request_body = RecordPaymentRequest,
    responses(
        (status = 200, description = "Payment recorded", body = OwnerContributionResponse),
        (status = 400, description = "Already paid, or unknown field in the body"),
        (status = 404, description = "Contribution not found"),
    ),
    security(("bearer_auth" = []))
)]
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
        Err(e) => HttpResponse::BadRequest().json(serde_json::json!({ "error": e })),
    }
}
