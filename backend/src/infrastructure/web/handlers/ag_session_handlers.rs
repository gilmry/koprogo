use crate::application::dto::ag_session_dto::{
    CreateAgSessionDto, EndAgSessionDto, RecordRemoteJoinDto,
};
use crate::infrastructure::web::{AppState, AuthenticatedUser};
use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use uuid::Uuid;

/// POST /meetings/:id/ag-session — Crée une session visio pour une réunion
#[post("/meetings/{meeting_id}/ag-session")]
pub async fn create_ag_session(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    path: web::Path<Uuid>,
    body: web::Json<CreateAgSessionDto>,
) -> impl Responder {
    let organization_id = match user.require_organization() {
        Ok(id) => id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": e.to_string()}))
        }
    };

    let mut dto = body.into_inner();
    dto.meeting_id = path.into_inner();

    match state
        .ag_session_use_cases
        .create_session(organization_id, dto, user.user_id)
        .await
    {
        Ok(session) => HttpResponse::Created().json(session),
        Err(e) => HttpResponse::BadRequest().json(serde_json::json!({"error": e})),
    }
}

/// GET /meetings/:id/ag-session — Récupère la session visio d'une réunion
#[get("/meetings/{meeting_id}/ag-session")]
pub async fn get_ag_session_for_meeting(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    path: web::Path<Uuid>,
) -> impl Responder {
    let organization_id = match user.require_organization() {
        Ok(id) => id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": e.to_string()}))
        }
    };

    match state
        .ag_session_use_cases
        .get_session_for_meeting(path.into_inner(), organization_id)
        .await
    {
        Ok(Some(session)) => HttpResponse::Ok().json(session),
        Ok(None) => HttpResponse::NotFound()
            .json(serde_json::json!({"error": "Aucune session visio pour cette réunion"})),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e})),
    }
}

/// GET /ag-sessions — Liste les sessions de l'organisation
#[get("/ag-sessions")]
pub async fn list_ag_sessions(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
) -> impl Responder {
    let organization_id = match user.require_organization() {
        Ok(id) => id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": e.to_string()}))
        }
    };

    match state
        .ag_session_use_cases
        .list_sessions(organization_id)
        .await
    {
        Ok(sessions) => HttpResponse::Ok().json(sessions),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e})),
    }
}

/// GET /ag-sessions/:id — Récupère une session par son ID
#[get("/ag-sessions/{id}")]
pub async fn get_ag_session(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    path: web::Path<Uuid>,
) -> impl Responder {
    let organization_id = match user.require_organization() {
        Ok(id) => id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": e.to_string()}))
        }
    };

    match state
        .ag_session_use_cases
        .get_session(path.into_inner(), organization_id)
        .await
    {
        Ok(session) => HttpResponse::Ok().json(session),
        Err(e) => {
            if e.contains("introuvable") {
                HttpResponse::NotFound().json(serde_json::json!({"error": e}))
            } else if e.contains("refusé") {
                HttpResponse::Forbidden().json(serde_json::json!({"error": e}))
            } else {
                HttpResponse::InternalServerError().json(serde_json::json!({"error": e}))
            }
        }
    }
}

/// PUT /ag-sessions/:id/start — Démarre la session (Scheduled → Live)
#[put("/ag-sessions/{id}/start")]
pub async fn start_ag_session(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    path: web::Path<Uuid>,
) -> impl Responder {
    let organization_id = match user.require_organization() {
        Ok(id) => id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": e.to_string()}))
        }
    };

    match state
        .ag_session_use_cases
        .start_session(path.into_inner(), organization_id)
        .await
    {
        Ok(session) => HttpResponse::Ok().json(session),
        Err(e) => HttpResponse::BadRequest().json(serde_json::json!({"error": e})),
    }
}

/// PUT /ag-sessions/:id/end — Termine la session (Live → Ended)
#[put("/ag-sessions/{id}/end")]
pub async fn end_ag_session(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    path: web::Path<Uuid>,
    body: web::Json<EndAgSessionDto>,
) -> impl Responder {
    let organization_id = match user.require_organization() {
        Ok(id) => id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": e.to_string()}))
        }
    };

    match state
        .ag_session_use_cases
        .end_session(path.into_inner(), organization_id, body.into_inner())
        .await
    {
        Ok(session) => HttpResponse::Ok().json(session),
        Err(e) => HttpResponse::BadRequest().json(serde_json::json!({"error": e})),
    }
}

/// PUT /ag-sessions/:id/cancel — Annule la session (Scheduled → Cancelled)
#[put("/ag-sessions/{id}/cancel")]
pub async fn cancel_ag_session(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    path: web::Path<Uuid>,
) -> impl Responder {
    let organization_id = match user.require_organization() {
        Ok(id) => id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": e.to_string()}))
        }
    };

    match state
        .ag_session_use_cases
        .cancel_session(path.into_inner(), organization_id)
        .await
    {
        Ok(session) => HttpResponse::Ok().json(session),
        Err(e) => HttpResponse::BadRequest().json(serde_json::json!({"error": e})),
    }
}

/// POST /ag-sessions/:id/join — Enregistre un participant distant
#[post("/ag-sessions/{id}/join")]
pub async fn record_remote_join(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    path: web::Path<Uuid>,
    body: web::Json<RecordRemoteJoinDto>,
) -> impl Responder {
    let organization_id = match user.require_organization() {
        Ok(id) => id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": e.to_string()}))
        }
    };

    match state
        .ag_session_use_cases
        .record_remote_join(path.into_inner(), organization_id, body.into_inner())
        .await
    {
        Ok(session) => HttpResponse::Ok().json(session),
        Err(e) => HttpResponse::BadRequest().json(serde_json::json!({"error": e})),
    }
}

/// GET /ag-sessions/:id/quorum — Calcule le quorum combiné (présentiel + distanciel)
#[get("/ag-sessions/{id}/quorum")]
pub async fn get_combined_quorum(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    path: web::Path<Uuid>,
    query: web::Query<CombinedQuorumQuery>,
) -> impl Responder {
    let organization_id = match user.require_organization() {
        Ok(id) => id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": e.to_string()}))
        }
    };

    match state
        .ag_session_use_cases
        .calculate_combined_quorum(
            path.into_inner(),
            organization_id,
            query.physical_quotas,
            query.total_building_quotas,
        )
        .await
    {
        Ok(result) => HttpResponse::Ok().json(result),
        Err(e) => HttpResponse::BadRequest().json(serde_json::json!({"error": e})),
    }
}

/// DELETE /ag-sessions/:id — Supprime une session annulée ou planifiée
#[delete("/ag-sessions/{id}")]
pub async fn delete_ag_session(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    path: web::Path<Uuid>,
) -> impl Responder {
    let organization_id = match user.require_organization() {
        Ok(id) => id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": e.to_string()}))
        }
    };

    match state
        .ag_session_use_cases
        .delete_session(path.into_inner(), organization_id)
        .await
    {
        Ok(()) => HttpResponse::NoContent().finish(),
        Err(e) => {
            if e.contains("introuvable") {
                HttpResponse::NotFound().json(serde_json::json!({"error": e}))
            } else if e.contains("refusé") {
                HttpResponse::Forbidden().json(serde_json::json!({"error": e}))
            } else {
                HttpResponse::BadRequest().json(serde_json::json!({"error": e}))
            }
        }
    }
}

#[derive(serde::Deserialize)]
pub struct CombinedQuorumQuery {
    pub physical_quotas: f64,
    pub total_building_quotas: f64,
}
