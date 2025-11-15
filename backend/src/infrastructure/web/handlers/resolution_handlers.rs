use crate::application::dto::{
    CastVoteRequest, ChangeVoteRequest, CloseVotingRequest, CreateResolutionRequest,
    ResolutionResponse, VoteResponse,
};
use crate::infrastructure::audit::{AuditEventType, AuditLogEntry};
use crate::infrastructure::web::{AppState, AuthenticatedUser};
use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use uuid::Uuid;

// ==================== Resolution Endpoints ====================

#[post("/meetings/{meeting_id}/resolutions")]
pub async fn create_resolution(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    meeting_id: web::Path<Uuid>,
    request: web::Json<CreateResolutionRequest>,
) -> impl Responder {
    let organization_id = match user.require_organization() {
        Ok(org_id) => org_id,
        Err(e) => return HttpResponse::Unauthorized().json(serde_json::json!({"error": e.to_string()})),
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

#[get("/resolutions/{id}")]
pub async fn get_resolution(
    state: web::Data<AppState>,
    id: web::Path<Uuid>,
) -> impl Responder {
    match state.resolution_use_cases.get_resolution(*id).await {
        Ok(Some(resolution)) => HttpResponse::Ok().json(ResolutionResponse::from(resolution)),
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Resolution not found"
        })),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": err
        })),
    }
}

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

#[delete("/resolutions/{id}")]
pub async fn delete_resolution(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    id: web::Path<Uuid>,
) -> impl Responder {
    let organization_id = match user.require_organization() {
        Ok(org_id) => org_id,
        Err(e) => return HttpResponse::Unauthorized().json(serde_json::json!({"error": e.to_string()})),
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

#[post("/resolutions/{resolution_id}/vote")]
pub async fn cast_vote(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    resolution_id: web::Path<Uuid>,
    request: web::Json<CastVoteRequest>,
) -> impl Responder {
    let organization_id = match user.require_organization() {
        Ok(org_id) => org_id,
        Err(e) => return HttpResponse::Unauthorized().json(serde_json::json!({"error": e.to_string()})),
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
            let responses: Vec<VoteResponse> = votes
                .into_iter()
                .map(VoteResponse::from)
                .collect();
            HttpResponse::Ok().json(responses)
        }
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": err
        })),
    }
}

#[put("/votes/{vote_id}")]
pub async fn change_vote(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    vote_id: web::Path<Uuid>,
    request: web::Json<ChangeVoteRequest>,
) -> impl Responder {
    let organization_id = match user.require_organization() {
        Ok(org_id) => org_id,
        Err(e) => return HttpResponse::Unauthorized().json(serde_json::json!({"error": e.to_string()})),
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

#[put("/resolutions/{resolution_id}/close")]
pub async fn close_voting(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    resolution_id: web::Path<Uuid>,
    request: web::Json<CloseVotingRequest>,
) -> impl Responder {
    let organization_id = match user.require_organization() {
        Ok(org_id) => org_id,
        Err(e) => return HttpResponse::Unauthorized().json(serde_json::json!({"error": e.to_string()})),
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
