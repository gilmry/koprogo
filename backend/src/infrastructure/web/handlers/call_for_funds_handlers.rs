use crate::application::dto::{
    CallForFundsResponse, CreateCallForFundsRequest, SendCallForFundsRequest,
    SendCallForFundsResponse,
};
use crate::domain::entities::ContributionType;
use crate::infrastructure::web::{AppState, AuthenticatedUser};
use actix_web::{delete, get, post, put, web, HttpResponse};
use uuid::Uuid;

/// POST /api/v1/call-for-funds
/// Create a new call for funds
#[post("/call-for-funds")]
pub async fn create_call_for_funds(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    req: web::Json<CreateCallForFundsRequest>,
) -> HttpResponse {
    let organization_id = match user.organization_id {
        Some(org_id) => org_id,
        None => return HttpResponse::BadRequest().body("Organization ID required"),
    };

    // Parse contribution type
    let contribution_type = match req.contribution_type.as_str() {
        "regular" => ContributionType::Regular,
        "extraordinary" => ContributionType::Extraordinary,
        "advance" => ContributionType::Advance,
        "adjustment" => ContributionType::Adjustment,
        _ => return HttpResponse::BadRequest().body("Invalid contribution type"),
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
        )
        .await
    {
        Ok(call) => {
            let response = CallForFundsResponse::from(call);
            HttpResponse::Created().json(response)
        }
        Err(e) => HttpResponse::BadRequest().body(e),
    }
}

/// GET /api/v1/call-for-funds/{id}
/// Get a call for funds by ID
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
        Ok(None) => HttpResponse::NotFound().body("Call for funds not found"),
        Err(e) => HttpResponse::InternalServerError().body(e),
    }
}

/// GET /api/v1/call-for-funds?building_id={uuid}
/// List all calls for funds for a building or organization
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
            Err(_) => return HttpResponse::BadRequest().body("Invalid building_id format"),
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
            Err(e) => return HttpResponse::InternalServerError().body(e),
        }
    }

    // Otherwise, return all calls for user's organization
    let organization_id = match user.organization_id {
        Some(org_id) => org_id,
        None => return HttpResponse::BadRequest().body("Organization ID required"),
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
        Err(e) => HttpResponse::InternalServerError().body(e),
    }
}

/// GET /api/v1/call-for-funds/overdue
/// Get all overdue calls for funds
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
        Err(e) => HttpResponse::InternalServerError().body(e),
    }
}

/// POST /api/v1/call-for-funds/{id}/send
/// Send a call for funds (marks as sent and generates individual contributions)
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
        Err(e) => HttpResponse::BadRequest().body(e),
    }
}

/// PUT /api/v1/call-for-funds/{id}/cancel
/// Cancel a call for funds
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
        Err(e) => HttpResponse::BadRequest().body(e),
    }
}

/// DELETE /api/v1/call-for-funds/{id}
/// Delete a call for funds (only if in draft status)
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
        Ok(false) => HttpResponse::NotFound().body("Call for funds not found"),
        Err(e) => HttpResponse::BadRequest().body(e),
    }
}
