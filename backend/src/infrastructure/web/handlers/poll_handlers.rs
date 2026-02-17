use crate::application::dto::{
    CastVoteDto, CreatePollDto, PageRequest, PollFilters, SortOrder, UpdatePollDto,
};
use crate::infrastructure::web::middleware::AuthenticatedUser;
use crate::infrastructure::web::AppState;
use actix_web::{delete, get, post, put, web, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ============================================================================
// Poll Management Endpoints
// ============================================================================

/// Create a new poll (draft status)
/// POST /api/v1/polls
#[post("/polls")]
pub async fn create_poll(
    state: web::Data<AppState>,
    auth_user: AuthenticatedUser,
    dto: web::Json<CreatePollDto>,
) -> HttpResponse {
    match state
        .poll_use_cases
        .create_poll(dto.into_inner(), auth_user.user_id)
        .await
    {
        Ok(poll) => HttpResponse::Created().json(poll),
        Err(e) => HttpResponse::BadRequest().json(serde_json::json!({
            "error": e
        })),
    }
}

/// Get poll by ID
/// GET /api/v1/polls/:id
#[get("/polls/{id}")]
pub async fn get_poll(
    state: web::Data<AppState>,
    _auth_user: AuthenticatedUser,
    path: web::Path<String>,
) -> HttpResponse {
    let poll_id = match Uuid::parse_str(&path.into_inner()) {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid poll ID format"
            }))
        }
    };

    match state.poll_use_cases.get_poll(poll_id).await {
        Ok(poll) => HttpResponse::Ok().json(poll),
        Err(e) => {
            if e.contains("not found") {
                HttpResponse::NotFound().json(serde_json::json!({
                    "error": e
                }))
            } else {
                HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": e
                }))
            }
        }
    }
}

/// Update poll (only draft polls can be updated)
/// PUT /api/v1/polls/:id
#[put("/polls/{id}")]
pub async fn update_poll(
    state: web::Data<AppState>,
    auth_user: AuthenticatedUser,
    path: web::Path<String>,
    dto: web::Json<UpdatePollDto>,
) -> HttpResponse {
    let poll_id = match Uuid::parse_str(&path.into_inner()) {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid poll ID format"
            }))
        }
    };

    match state
        .poll_use_cases
        .update_poll(poll_id, dto.into_inner(), auth_user.user_id)
        .await
    {
        Ok(poll) => HttpResponse::Ok().json(poll),
        Err(e) => {
            if e.contains("not found") {
                HttpResponse::NotFound().json(serde_json::json!({
                    "error": e
                }))
            } else if e.contains("Only the poll creator") {
                HttpResponse::Forbidden().json(serde_json::json!({
                    "error": e
                }))
            } else {
                HttpResponse::BadRequest().json(serde_json::json!({
                    "error": e
                }))
            }
        }
    }
}

/// List polls with pagination and filters
/// GET /api/v1/polls?page=1&per_page=10&building_id=xxx&status=active
#[get("/polls")]
pub async fn list_polls(
    state: web::Data<AppState>,
    _auth_user: AuthenticatedUser,
    query: web::Query<ListPollsQuery>,
) -> HttpResponse {
    let page_request = PageRequest {
        page: query.page.unwrap_or(1),
        per_page: query.per_page.unwrap_or(10),
        sort_by: None,
        order: SortOrder::Desc,
    };

    let filters = PollFilters {
        building_id: query.building_id.clone(),
        created_by: query.created_by.clone(),
        status: None, // Parse from string if needed
        poll_type: None,
        ends_before: query.ends_before.clone(),
        ends_after: query.ends_after.clone(),
    };

    match state
        .poll_use_cases
        .list_polls_paginated(&page_request, &filters)
        .await
    {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e
        })),
    }
}

#[derive(Debug, Deserialize)]
pub struct ListPollsQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub building_id: Option<String>,
    pub created_by: Option<String>,
    pub ends_before: Option<String>,
    pub ends_after: Option<String>,
}

/// Find active polls for a building
/// GET /api/v1/buildings/:building_id/polls/active
#[get("/buildings/{building_id}/polls/active")]
pub async fn find_active_polls(
    state: web::Data<AppState>,
    _auth_user: AuthenticatedUser,
    path: web::Path<String>,
) -> HttpResponse {
    let building_id = match Uuid::parse_str(&path.into_inner()) {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid building ID format"
            }))
        }
    };

    match state.poll_use_cases.find_active_polls(building_id).await {
        Ok(polls) => HttpResponse::Ok().json(polls),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e
        })),
    }
}

/// Publish a poll (change from draft to active)
/// POST /api/v1/polls/:id/publish
#[post("/polls/{id}/publish")]
pub async fn publish_poll(
    state: web::Data<AppState>,
    auth_user: AuthenticatedUser,
    path: web::Path<String>,
) -> HttpResponse {
    let poll_id = match Uuid::parse_str(&path.into_inner()) {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid poll ID format"
            }))
        }
    };

    match state
        .poll_use_cases
        .publish_poll(poll_id, auth_user.user_id)
        .await
    {
        Ok(poll) => HttpResponse::Ok().json(poll),
        Err(e) => {
            if e.contains("not found") {
                HttpResponse::NotFound().json(serde_json::json!({
                    "error": e
                }))
            } else if e.contains("Only the poll creator") {
                HttpResponse::Forbidden().json(serde_json::json!({
                    "error": e
                }))
            } else {
                HttpResponse::BadRequest().json(serde_json::json!({
                    "error": e
                }))
            }
        }
    }
}

/// Close a poll manually
/// POST /api/v1/polls/:id/close
#[post("/polls/{id}/close")]
pub async fn close_poll(
    state: web::Data<AppState>,
    auth_user: AuthenticatedUser,
    path: web::Path<String>,
) -> HttpResponse {
    let poll_id = match Uuid::parse_str(&path.into_inner()) {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid poll ID format"
            }))
        }
    };

    match state
        .poll_use_cases
        .close_poll(poll_id, auth_user.user_id)
        .await
    {
        Ok(poll) => HttpResponse::Ok().json(poll),
        Err(e) => {
            if e.contains("not found") {
                HttpResponse::NotFound().json(serde_json::json!({
                    "error": e
                }))
            } else if e.contains("Only the poll creator") {
                HttpResponse::Forbidden().json(serde_json::json!({
                    "error": e
                }))
            } else {
                HttpResponse::BadRequest().json(serde_json::json!({
                    "error": e
                }))
            }
        }
    }
}

/// Cancel a poll
/// POST /api/v1/polls/:id/cancel
#[post("/polls/{id}/cancel")]
pub async fn cancel_poll(
    state: web::Data<AppState>,
    auth_user: AuthenticatedUser,
    path: web::Path<String>,
) -> HttpResponse {
    let poll_id = match Uuid::parse_str(&path.into_inner()) {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid poll ID format"
            }))
        }
    };

    match state
        .poll_use_cases
        .cancel_poll(poll_id, auth_user.user_id)
        .await
    {
        Ok(poll) => HttpResponse::Ok().json(poll),
        Err(e) => {
            if e.contains("not found") {
                HttpResponse::NotFound().json(serde_json::json!({
                    "error": e
                }))
            } else if e.contains("Only the poll creator") {
                HttpResponse::Forbidden().json(serde_json::json!({
                    "error": e
                }))
            } else {
                HttpResponse::BadRequest().json(serde_json::json!({
                    "error": e
                }))
            }
        }
    }
}

/// Delete a poll (only draft or cancelled)
/// DELETE /api/v1/polls/:id
#[delete("/polls/{id}")]
pub async fn delete_poll(
    state: web::Data<AppState>,
    auth_user: AuthenticatedUser,
    path: web::Path<String>,
) -> HttpResponse {
    let poll_id = match Uuid::parse_str(&path.into_inner()) {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid poll ID format"
            }))
        }
    };

    match state
        .poll_use_cases
        .delete_poll(poll_id, auth_user.user_id)
        .await
    {
        Ok(true) => HttpResponse::NoContent().finish(),
        Ok(false) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Poll not found"
        })),
        Err(e) => {
            if e.contains("Only the poll creator") {
                HttpResponse::Forbidden().json(serde_json::json!({
                    "error": e
                }))
            } else {
                HttpResponse::BadRequest().json(serde_json::json!({
                    "error": e
                }))
            }
        }
    }
}

// ============================================================================
// Voting Endpoints
// ============================================================================

/// Cast a vote on a poll
/// POST /api/v1/polls/vote
#[post("/polls/vote")]
pub async fn cast_poll_vote(
    state: web::Data<AppState>,
    auth_user: AuthenticatedUser,
    dto: web::Json<CastVoteDto>,
    _req: HttpRequest,
) -> HttpResponse {
    // Owner ID is optional (anonymous votes)
    // For now, we use the authenticated user's ID
    // In production, you'd have logic to determine if vote is anonymous
    let owner_id = Some(auth_user.user_id);

    match state
        .poll_use_cases
        .cast_vote(dto.into_inner(), owner_id)
        .await
    {
        Ok(message) => HttpResponse::Created().json(serde_json::json!({
            "message": message
        })),
        Err(e) => {
            if e.contains("not active") || e.contains("expired") {
                HttpResponse::BadRequest().json(serde_json::json!({
                    "error": e
                }))
            } else if e.contains("already voted") {
                HttpResponse::Conflict().json(serde_json::json!({
                    "error": e
                }))
            } else if e.contains("not found") {
                HttpResponse::NotFound().json(serde_json::json!({
                    "error": e
                }))
            } else {
                HttpResponse::BadRequest().json(serde_json::json!({
                    "error": e
                }))
            }
        }
    }
}

/// Get poll results
/// GET /api/v1/polls/:id/results
#[get("/polls/{id}/results")]
pub async fn get_poll_results(
    state: web::Data<AppState>,
    _auth_user: AuthenticatedUser,
    path: web::Path<String>,
) -> HttpResponse {
    let poll_id = match Uuid::parse_str(&path.into_inner()) {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid poll ID format"
            }))
        }
    };

    match state.poll_use_cases.get_poll_results(poll_id).await {
        Ok(results) => HttpResponse::Ok().json(results),
        Err(e) => {
            if e.contains("not found") {
                HttpResponse::NotFound().json(serde_json::json!({
                    "error": e
                }))
            } else {
                HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": e
                }))
            }
        }
    }
}

// ============================================================================
// Statistics Endpoints
// ============================================================================

/// Get poll statistics for a building
/// GET /api/v1/buildings/:building_id/polls/statistics
#[get("/buildings/{building_id}/polls/statistics")]
pub async fn get_poll_building_statistics(
    state: web::Data<AppState>,
    _auth_user: AuthenticatedUser,
    path: web::Path<String>,
) -> HttpResponse {
    let building_id = match Uuid::parse_str(&path.into_inner()) {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid building ID format"
            }))
        }
    };

    match state
        .poll_use_cases
        .get_building_statistics(building_id)
        .await
    {
        Ok(stats) => HttpResponse::Ok().json(stats),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e
        })),
    }
}

// ============================================================================
// Statistics Response DTO
// ============================================================================

#[derive(Debug, Serialize)]
pub struct PollStatisticsResponse {
    pub total_polls: i64,
    pub active_polls: i64,
    pub closed_polls: i64,
    pub average_participation_rate: f64,
}
