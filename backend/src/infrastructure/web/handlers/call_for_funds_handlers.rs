use crate::application::dto::{
    CallForFundsResponse, CreateCallForFundsRequest, SendCallForFundsRequest,
    SendCallForFundsResponse,
};
use crate::domain::entities::{ContributionType, UserRole};
use crate::infrastructure::web::handlers::conformity_response::try_build_conformity_response;
use crate::infrastructure::web::middleware::scope_guard::verify_building_org_access;
use crate::infrastructure::web::{AppState, AuthenticatedUser};
use actix_web::{delete, get, post, put, web, HttpResponse, ResponseError};
use std::str::FromStr;
use uuid::Uuid;

/// Story 3.1 — Vérifie que l'utilisateur peut créer un appel de fonds.
///
/// INV-10 : sortie financière → réservé aux émetteurs (syndic, superadmin,
/// accountant générique, accountant.emetteur). Un `accountant.encodeur` seul
/// est rejeté (403 `invalid_role`).
fn check_can_create_call_for_funds(user: &AuthenticatedUser) -> Option<HttpResponse> {
    match UserRole::from_str(&user.role).ok() {
        Some(role) if role.can_create_call_for_funds() => None,
        _ => Some(HttpResponse::Forbidden().json(serde_json::json!({
            "error":
                "Only syndic, superadmin, or accountant émetteur can create call-for-funds",
            "code": "invalid_role",
        }))),
    }
}

/// POST /api/v1/call-for-funds
/// Create a new call for funds
#[utoipa::path(
    post,
    path = "/call-for-funds",
    tag = "CallForFunds",
    summary = "Create a collective call for funds (draft)",
    request_body = CreateCallForFundsRequest,
    responses(
        (status = 201, description = "Call for funds created", body = CallForFundsResponse),
        (status = 400, description = "Validation error, or unknown field in the body"),
        (status = 401, description = "User does not belong to an organization"),
    ),
    security(("bearer_auth" = []))
)]
#[post("/call-for-funds")]
pub async fn create_call_for_funds(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    req: web::Json<CreateCallForFundsRequest>,
) -> HttpResponse {
    // Story 3.1 INV-10 : seuls les émetteurs peuvent créer un appel de fonds.
    if let Some(response) = check_can_create_call_for_funds(&user) {
        return response;
    }

    let organization_id = match user.organization_id {
        Some(org_id) => org_id,
        None => {
            return HttpResponse::BadRequest()
                .json(serde_json::json!({ "error": "Organization ID required" }))
        }
    };

    // Isolation multi-tenant à l'ÉCRITURE : l'immeuble visé doit relever d'une
    // ACP dont ce syndic a la gestion. Mesuré le 2026-09-02 : un cabinet tiers
    // pouvait créer PUIS ENVOYER un appel de fonds sur l'immeuble d'un autre,
    // générant des quotes-parts réclamées à des copropriétaires qui ne sont
    // pas les siens.
    if let Err(err) = verify_building_org_access(
        &user,
        req.building_id,
        &state.building_use_cases,
        &state.acp_use_cases,
    )
    .await
    {
        return err.error_response();
    }

    // Parse contribution type
    let contribution_type = match req.contribution_type.as_str() {
        "regular" => ContributionType::Regular,
        "extraordinary" => ContributionType::Extraordinary,
        "advance" => ContributionType::Advance,
        "adjustment" => ContributionType::Adjustment,
        _ => {
            return HttpResponse::BadRequest()
                .json(serde_json::json!({ "error": "Invalid contribution type" }))
        }
    };

    match state
        .call_for_funds_use_cases
        .create_call_for_funds(
            organization_id,
            req.building_id,
            req.title.clone(),
            req.description.clone(),
            req.total_amount,
            contribution_type,
            req.call_date,
            req.due_date,
            req.account_code.clone(),
            Some(user.user_id),
            req.reserve_fund_share,
        )
        .await
    {
        Ok(call) => {
            let response = CallForFundsResponse::from(call);
            HttpResponse::Created().json(response)
        }
        Err(e) => {
            // Track H Story H2 — pre-check validate-before-compute → 422 narratif
            if let Some(resp) = try_build_conformity_response(&e) {
                return resp;
            }
            HttpResponse::BadRequest().json(serde_json::json!({ "error": e }))
        }
    }
}

/// GET /api/v1/call-for-funds/{id}
/// Get a call for funds by ID
#[utoipa::path(
    get,
    path = "/call-for-funds/{id}",
    tag = "CallForFunds",
    summary = "Get a single call for funds",
    params(("id" = Uuid, Path, description = "Call for funds identifier")),
    responses(
        (status = 200, description = "Call for funds", body = CallForFundsResponse),
        (status = 404, description = "Not found"),
    ),
    security(("bearer_auth" = []))
)]
#[get("/call-for-funds/{id}")]
pub async fn get_call_for_funds(
    state: web::Data<AppState>,
    _user: AuthenticatedUser,
    id: web::Path<Uuid>,
) -> HttpResponse {
    match state.call_for_funds_use_cases.get_call_for_funds(*id).await {
        Ok(Some(call)) => {
            let response = CallForFundsResponse::from(call);
            HttpResponse::Ok().json(response)
        }
        Ok(None) => HttpResponse::NotFound()
            .json(serde_json::json!({ "error": "Call for funds not found" })),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({ "error": e })),
    }
}

/// GET /api/v1/call-for-funds?building_id={uuid}
/// List all calls for funds for a building or organization
#[utoipa::path(
    get,
    path = "/call-for-funds",
    tag = "CallForFunds",
    summary = "List calls for funds, optionally filtered by building or status",
    params(
        ("building_id" = Option<Uuid>, Query, description = "Restrict to one building"),
        ("status" = Option<String>, Query, description = "draft | sent | overdue | cancelled"),
    ),
    responses(
        (status = 200, description = "Calls for funds", body = Vec<CallForFundsResponse>),
        (status = 401, description = "User does not belong to an organization"),
    ),
    security(("bearer_auth" = []))
)]
#[get("/call-for-funds")]
pub async fn list_call_for_funds(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> HttpResponse {
    // If building_id provided, filter by building
    if let Some(id_str) = query.get("building_id") {
        let building_id = match Uuid::parse_str(id_str) {
            Ok(id) => id,
            Err(_) => {
                return HttpResponse::BadRequest()
                    .json(serde_json::json!({ "error": "Invalid building_id format" }))
            }
        };

        match state
            .call_for_funds_use_cases
            .list_by_building(building_id)
            .await
        {
            Ok(calls) => {
                let responses: Vec<CallForFundsResponse> =
                    calls.into_iter().map(Into::into).collect();
                return HttpResponse::Ok().json(responses);
            }
            Err(e) => {
                return HttpResponse::InternalServerError().json(serde_json::json!({ "error": e }))
            }
        }
    }

    // Otherwise, return all calls for user's organization
    let organization_id = match user.organization_id {
        Some(org_id) => org_id,
        None => {
            return HttpResponse::BadRequest()
                .json(serde_json::json!({ "error": "Organization ID required" }))
        }
    };

    match state
        .call_for_funds_use_cases
        .list_by_organization(organization_id)
        .await
    {
        Ok(calls) => {
            let responses: Vec<CallForFundsResponse> = calls.into_iter().map(Into::into).collect();
            HttpResponse::Ok().json(responses)
        }
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({ "error": e })),
    }
}

/// GET /api/v1/call-for-funds/overdue
/// Get all overdue calls for funds
#[utoipa::path(
    get,
    path = "/call-for-funds/overdue",
    tag = "CallForFunds",
    summary = "List overdue calls for funds",
    responses(
        (status = 200, description = "Overdue calls", body = Vec<CallForFundsResponse>),
        (status = 401, description = "User does not belong to an organization"),
    ),
    security(("bearer_auth" = []))
)]
#[get("/call-for-funds/overdue")]
pub async fn get_overdue_calls(
    state: web::Data<AppState>,
    _user: AuthenticatedUser,
) -> HttpResponse {
    match state.call_for_funds_use_cases.get_overdue_calls().await {
        Ok(calls) => {
            let responses: Vec<CallForFundsResponse> = calls.into_iter().map(Into::into).collect();
            HttpResponse::Ok().json(responses)
        }
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({ "error": e })),
    }
}

/// POST /api/v1/call-for-funds/{id}/send
/// Send a call for funds (marks as sent and generates individual contributions)
#[utoipa::path(
    post,
    path = "/call-for-funds/{id}/send",
    tag = "CallForFunds",
    summary = "Send a call for funds and generate individual contributions",
    description = "Ventile le montant total entre les coproprietaires ACTIFS du \
batiment, au prorata de leurs quotites. Les detentions sont lues dans \
`unit_owners` (routes `/unit-owners`), PAS dans le champ deprecie \
`units.owner_id` : un batiment dont les lots n'ont pas de detenteur actif \
enregistre la echoue avec « No active owners found for this building ».",
    params(("id" = Uuid, Path, description = "Call for funds identifier")),
    responses(
        (status = 200, description = "Sent, contributions generated", body = SendCallForFundsResponse),
        (status = 400, description = "No active owners, or building not conformant"),
        (status = 404, description = "Not found"),
    ),
    security(("bearer_auth" = []))
)]
#[post("/call-for-funds/{id}/send")]
pub async fn send_call_for_funds(
    state: web::Data<AppState>,
    _user: AuthenticatedUser,
    id: web::Path<Uuid>,
    _req: web::Json<SendCallForFundsRequest>,
) -> HttpResponse {
    match state
        .call_for_funds_use_cases
        .send_call_for_funds(*id)
        .await
    {
        Ok(call) => {
            // Get the number of contributions generated
            // (In a real implementation, we'd return this from send_call_for_funds)
            let contributions_generated = match state
                .owner_contribution_use_cases
                .get_contributions_by_organization(call.organization_id)
                .await
            {
                Ok(contribs) => contribs
                    .iter()
                    .filter(|c| c.call_for_funds_id == Some(call.id))
                    .count(),
                Err(_) => 0,
            };

            let response = SendCallForFundsResponse {
                call_for_funds: CallForFundsResponse::from(call),
                contributions_generated,
            };
            HttpResponse::Ok().json(response)
        }
        Err(e) => {
            // Track H Story H2 — pre-check validate-before-compute → 422 narratif
            if let Some(resp) = try_build_conformity_response(&e) {
                return resp;
            }
            HttpResponse::BadRequest().json(serde_json::json!({ "error": e }))
        }
    }
}

/// PUT /api/v1/call-for-funds/{id}/cancel
/// Cancel a call for funds
#[utoipa::path(
    put,
    path = "/call-for-funds/{id}/cancel",
    tag = "CallForFunds",
    summary = "Cancel a call for funds",
    params(("id" = Uuid, Path, description = "Call for funds identifier")),
    responses(
        (status = 200, description = "Cancelled", body = CallForFundsResponse),
        (status = 404, description = "Not found"),
    ),
    security(("bearer_auth" = []))
)]
#[put("/call-for-funds/{id}/cancel")]
pub async fn cancel_call_for_funds(
    state: web::Data<AppState>,
    _user: AuthenticatedUser,
    id: web::Path<Uuid>,
) -> HttpResponse {
    match state
        .call_for_funds_use_cases
        .cancel_call_for_funds(*id)
        .await
    {
        Ok(call) => {
            let response = CallForFundsResponse::from(call);
            HttpResponse::Ok().json(response)
        }
        Err(e) => HttpResponse::BadRequest().json(serde_json::json!({ "error": e })),
    }
}

/// DELETE /api/v1/call-for-funds/{id}
/// Delete a call for funds (only if in draft status)
#[utoipa::path(
    delete,
    path = "/call-for-funds/{id}",
    tag = "CallForFunds",
    summary = "Delete a draft call for funds",
    params(("id" = Uuid, Path, description = "Call for funds identifier")),
    responses(
        (status = 204, description = "Deleted"),
        (status = 400, description = "Only a draft can be deleted"),
        (status = 404, description = "Not found"),
    ),
    security(("bearer_auth" = []))
)]
#[delete("/call-for-funds/{id}")]
pub async fn delete_call_for_funds(
    state: web::Data<AppState>,
    _user: AuthenticatedUser,
    id: web::Path<Uuid>,
) -> HttpResponse {
    match state
        .call_for_funds_use_cases
        .delete_call_for_funds(*id)
        .await
    {
        Ok(true) => HttpResponse::NoContent().finish(),
        Ok(false) => HttpResponse::NotFound()
            .json(serde_json::json!({ "error": "Call for funds not found" })),
        Err(e) => HttpResponse::BadRequest().json(serde_json::json!({ "error": e })),
    }
}
