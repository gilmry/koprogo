use crate::application::dto::{
    CastVoteRequest, ChangeVoteRequest, CloseVotingRequest, CreateResolutionRequest,
    ResolutionResponse, VoteResponse,
};
use crate::infrastructure::audit::{AuditEventType, AuditLogEntry};
use crate::infrastructure::web::{AppState, AuthenticatedUser};
use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use uuid::Uuid;

// ==================== Resolution Endpoints ====================

#[utoipa::path(
    post,
    path = "/meetings/{meeting_id}/resolutions",
    tag = "Resolutions",
    summary = "Create a resolution for a meeting",
    params(
        ("meeting_id" = Uuid, Path, description = "Meeting UUID")
    ),
    request_body = CreateResolutionRequest,
    responses(
        (status = 201, description = "Resolution created"),
        (status = 400, description = "Bad Request"),
        (status = 401, description = "Unauthorized"),
    ),
    security(("bearer_auth" = []))
)]
#[post("/meetings/{meeting_id}/resolutions")]
pub async fn create_resolution(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    meeting_id: web::Path<Uuid>,
    request: web::Json<CreateResolutionRequest>,
) -> impl Responder {
    let organization_id = match user.require_organization() {
        Ok(org_id) => org_id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": e.to_string()}))
        }
    };

    let meeting_id = *meeting_id;

    match state
        .resolution_use_cases
        .create_resolution(
            meeting_id,
            request.title.clone(),
            request.description.clone(),
            request.resolution_type.clone(),
            request.majority_required.clone(),
        )
        .await
    {
        Ok(resolution) => {
            AuditLogEntry::new(
                AuditEventType::ResolutionCreated,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_resource("Resolution", resolution.id)
            .log();

            HttpResponse::Created().json(ResolutionResponse::from(resolution))
        }
        Err(err) => {
            AuditLogEntry::new(
                AuditEventType::ResolutionCreated,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_error(err.clone())
            .log();

            HttpResponse::BadRequest().json(serde_json::json!({"error": err}))
        }
    }
}

#[utoipa::path(
    get,
    path = "/resolutions/{id}",
    tag = "Resolutions",
    summary = "Get resolution by ID",
    params(
        ("id" = Uuid, Path, description = "Resolution UUID")
    ),
    responses(
        (status = 200, description = "Resolution found"),
        (status = 404, description = "Resolution not found"),
        (status = 500, description = "Internal Server Error"),
    ),
    security(("bearer_auth" = []))
)]
#[get("/resolutions/{id}")]
pub async fn get_resolution(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    id: web::Path<Uuid>,
) -> impl Responder {
    match state.resolution_use_cases.get_resolution(*id).await {
        Ok(Some(resolution)) => {
            // Multi-tenant isolation: verify resolution's meeting belongs to user's organization
            // Resolution → Meeting → Building → Organization
            if let Ok(Some(meeting)) = state.meeting_use_cases.get_meeting(resolution.meeting_id).await {
                if let Ok(Some(building)) = state.building_use_cases.get_building(meeting.building_id).await {
                    if let Ok(building_org) = Uuid::parse_str(&building.organization_id) {
                        if let Err(e) = user.verify_org_access(building_org) {
                            return HttpResponse::Forbidden().json(serde_json::json!({ "error": e }));
                        }
                    }
                }
            }
            HttpResponse::Ok().json(ResolutionResponse::from(resolution))
        }
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Resolution not found"
        })),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": err
        })),
    }
}

#[utoipa::path(
    get,
    path = "/meetings/{meeting_id}/resolutions",
    tag = "Resolutions",
    summary = "List all resolutions for a meeting",
    params(
        ("meeting_id" = Uuid, Path, description = "Meeting UUID")
    ),
    responses(
        (status = 200, description = "List of resolutions"),
        (status = 500, description = "Internal Server Error"),
    ),
    security(("bearer_auth" = []))
)]
#[get("/meetings/{meeting_id}/resolutions")]
pub async fn list_meeting_resolutions(
    state: web::Data<AppState>,
    meeting_id: web::Path<Uuid>,
) -> impl Responder {
    match state
        .resolution_use_cases
        .get_meeting_resolutions(*meeting_id)
        .await
    {
        Ok(resolutions) => {
            let responses: Vec<ResolutionResponse> = resolutions
                .into_iter()
                .map(ResolutionResponse::from)
                .collect();
            HttpResponse::Ok().json(responses)
        }
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": err
        })),
    }
}

#[utoipa::path(
    delete,
    path = "/resolutions/{id}",
    tag = "Resolutions",
    summary = "Delete a resolution",
    params(
        ("id" = Uuid, Path, description = "Resolution UUID")
    ),
    responses(
        (status = 204, description = "Resolution deleted"),
        (status = 400, description = "Bad Request"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Resolution not found"),
    ),
    security(("bearer_auth" = []))
)]
#[delete("/resolutions/{id}")]
pub async fn delete_resolution(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    id: web::Path<Uuid>,
) -> impl Responder {
    let organization_id = match user.require_organization() {
        Ok(org_id) => org_id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": e.to_string()}))
        }
    };

    match state.resolution_use_cases.delete_resolution(*id).await {
        Ok(true) => {
            AuditLogEntry::new(
                AuditEventType::ResolutionDeleted,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_resource("Resolution", *id)
            .log();

            HttpResponse::NoContent().finish()
        }
        Ok(false) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Resolution not found"
        })),
        Err(err) => {
            AuditLogEntry::new(
                AuditEventType::ResolutionDeleted,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_error(err.clone())
            .log();

            HttpResponse::BadRequest().json(serde_json::json!({"error": err}))
        }
    }
}

// ==================== Vote Endpoints ====================

#[utoipa::path(
    post,
    path = "/resolutions/{resolution_id}/vote",
    tag = "Resolutions",
    summary = "Cast a vote on a resolution",
    params(
        ("resolution_id" = Uuid, Path, description = "Resolution UUID")
    ),
    request_body = CastVoteRequest,
    responses(
        (status = 201, description = "Vote cast"),
        (status = 400, description = "Bad Request"),
        (status = 401, description = "Unauthorized"),
    ),
    security(("bearer_auth" = []))
)]
#[post("/resolutions/{resolution_id}/vote")]
pub async fn cast_vote(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    resolution_id: web::Path<Uuid>,
    request: web::Json<CastVoteRequest>,
) -> impl Responder {
    let organization_id = match user.require_organization() {
        Ok(org_id) => org_id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": e.to_string()}))
        }
    };

    match state
        .resolution_use_cases
        .cast_vote(
            *resolution_id,
            request.owner_id,
            request.unit_id,
            request.vote_choice.clone(),
            request.voting_power,
            request.proxy_owner_id,
        )
        .await
    {
        Ok(vote) => {
            AuditLogEntry::new(
                AuditEventType::VoteCast,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_resource("Vote", vote.id)
            .with_metadata(serde_json::json!({
                "resolution_id": *resolution_id,
                "vote_choice": format!("{:?}", vote.vote_choice)
            }))
            .log();

            HttpResponse::Created().json(VoteResponse::from(vote))
        }
        Err(err) => {
            AuditLogEntry::new(
                AuditEventType::VoteCast,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_error(err.clone())
            .log();

            HttpResponse::BadRequest().json(serde_json::json!({"error": err}))
        }
    }
}

#[utoipa::path(
    get,
    path = "/resolutions/{resolution_id}/votes",
    tag = "Resolutions",
    summary = "List all votes for a resolution",
    params(
        ("resolution_id" = Uuid, Path, description = "Resolution UUID")
    ),
    responses(
        (status = 200, description = "List of votes"),
        (status = 500, description = "Internal Server Error"),
    ),
    security(("bearer_auth" = []))
)]
#[get("/resolutions/{resolution_id}/votes")]
pub async fn list_resolution_votes(
    state: web::Data<AppState>,
    resolution_id: web::Path<Uuid>,
) -> impl Responder {
    match state
        .resolution_use_cases
        .get_resolution_votes(*resolution_id)
        .await
    {
        Ok(votes) => {
            let responses: Vec<VoteResponse> = votes.into_iter().map(VoteResponse::from).collect();
            HttpResponse::Ok().json(responses)
        }
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": err
        })),
    }
}

#[utoipa::path(
    put,
    path = "/votes/{vote_id}",
    tag = "Resolutions",
    summary = "Change an existing vote",
    params(
        ("vote_id" = Uuid, Path, description = "Vote UUID")
    ),
    request_body = ChangeVoteRequest,
    responses(
        (status = 200, description = "Vote changed"),
        (status = 400, description = "Bad Request"),
        (status = 401, description = "Unauthorized"),
    ),
    security(("bearer_auth" = []))
)]
#[put("/votes/{vote_id}")]
pub async fn change_vote(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    vote_id: web::Path<Uuid>,
    request: web::Json<ChangeVoteRequest>,
) -> impl Responder {
    let organization_id = match user.require_organization() {
        Ok(org_id) => org_id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": e.to_string()}))
        }
    };

    match state
        .resolution_use_cases
        .change_vote(*vote_id, request.vote_choice.clone())
        .await
    {
        Ok(vote) => {
            AuditLogEntry::new(
                AuditEventType::VoteChanged,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_resource("Vote", vote.id)
            .with_metadata(serde_json::json!({
                "new_choice": format!("{:?}", vote.vote_choice)
            }))
            .log();

            HttpResponse::Ok().json(VoteResponse::from(vote))
        }
        Err(err) => {
            AuditLogEntry::new(
                AuditEventType::VoteChanged,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_error(err.clone())
            .log();

            HttpResponse::BadRequest().json(serde_json::json!({"error": err}))
        }
    }
}

#[utoipa::path(
    put,
    path = "/resolutions/{resolution_id}/close",
    tag = "Resolutions",
    summary = "Close voting on a resolution and calculate result",
    params(
        ("resolution_id" = Uuid, Path, description = "Resolution UUID")
    ),
    request_body = CloseVotingRequest,
    responses(
        (status = 200, description = "Voting closed and result calculated"),
        (status = 400, description = "Bad Request"),
        (status = 401, description = "Unauthorized"),
    ),
    security(("bearer_auth" = []))
)]
#[put("/resolutions/{resolution_id}/close")]
pub async fn close_voting(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    resolution_id: web::Path<Uuid>,
    request: web::Json<CloseVotingRequest>,
) -> impl Responder {
    let organization_id = match user.require_organization() {
        Ok(org_id) => org_id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": e.to_string()}))
        }
    };

    match state
        .resolution_use_cases
        .close_voting(*resolution_id, request.total_voting_power)
        .await
    {
        Ok(resolution) => {
            AuditLogEntry::new(
                AuditEventType::VotingClosed,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_resource("Resolution", resolution.id)
            .with_metadata(serde_json::json!({
                "final_status": format!("{:?}", resolution.status)
            }))
            .log();

            HttpResponse::Ok().json(ResolutionResponse::from(resolution))
        }
        Err(err) => {
            AuditLogEntry::new(
                AuditEventType::VotingClosed,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_error(err.clone())
            .log();

            HttpResponse::BadRequest().json(serde_json::json!({"error": err}))
        }
    }
}

#[utoipa::path(
    get,
    path = "/meetings/{meeting_id}/vote-summary",
    tag = "Resolutions",
    summary = "Get vote summary for a meeting",
    params(
        ("meeting_id" = Uuid, Path, description = "Meeting UUID")
    ),
    responses(
        (status = 200, description = "Vote summary for all meeting resolutions"),
        (status = 500, description = "Internal Server Error"),
    ),
    security(("bearer_auth" = []))
)]
#[get("/meetings/{meeting_id}/vote-summary")]
pub async fn get_meeting_vote_summary(
    state: web::Data<AppState>,
    meeting_id: web::Path<Uuid>,
) -> impl Responder {
    match state
        .resolution_use_cases
        .get_meeting_vote_summary(*meeting_id)
        .await
    {
        Ok(resolutions) => {
            let responses: Vec<ResolutionResponse> = resolutions
                .into_iter()
                .map(ResolutionResponse::from)
                .collect();
            HttpResponse::Ok().json(responses)
        }
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": err
        })),
    }
}
