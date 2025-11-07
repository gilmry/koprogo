use crate::application::dto::{
    AddDecisionNotesDto, CreateBoardDecisionDto, UpdateBoardDecisionDto,
};
use crate::infrastructure::audit::{AuditEventType, AuditLogEntry};
use crate::infrastructure::web::{AppState, AuthenticatedUser};
use actix_web::{get, post, put, web, HttpResponse, Responder};
use uuid::Uuid;

/// Créer une nouvelle décision à suivre suite à une AG
#[post("/board-decisions")]
pub async fn create_decision(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    request: web::Json<CreateBoardDecisionDto>,
) -> impl Responder {
    let organization_id = match user.require_organization() {
        Ok(org_id) => org_id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": e.to_string()
            }))
        }
    };

    match state
        .board_decision_use_cases
        .create_decision(request.into_inner())
        .await
    {
        Ok(decision) => {
            // Audit log: successful decision creation
            if let Ok(decision_uuid) = Uuid::parse_str(&decision.id) {
                AuditLogEntry::new(
                    AuditEventType::BoardDecisionCreated,
                    Some(user.user_id),
                    Some(organization_id),
                )
                .with_resource("BoardDecision", decision_uuid)
                .log();
            }

            HttpResponse::Created().json(decision)
        }
        Err(err) => {
            // Audit log: failed decision creation
            AuditLogEntry::new(
                AuditEventType::BoardDecisionCreated,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_error(err.clone())
            .log();

            HttpResponse::BadRequest().json(serde_json::json!({
                "error": err
            }))
        }
    }
}

/// Récupérer une décision par ID
#[get("/board-decisions/{id}")]
pub async fn get_decision(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    id: web::Path<Uuid>,
) -> impl Responder {
    let _organization_id = match user.require_organization() {
        Ok(org_id) => org_id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": e.to_string()
            }))
        }
    };

    match state.board_decision_use_cases.get_decision(*id).await {
        Ok(decision) => HttpResponse::Ok().json(decision),
        Err(err) => HttpResponse::NotFound().json(serde_json::json!({
            "error": err
        })),
    }
}

/// Lister toutes les décisions d'un immeuble
#[get("/buildings/{building_id}/board-decisions")]
pub async fn list_decisions_by_building(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    building_id: web::Path<Uuid>,
) -> impl Responder {
    let _organization_id = match user.require_organization() {
        Ok(org_id) => org_id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": e.to_string()
            }))
        }
    };

    match state
        .board_decision_use_cases
        .list_decisions_by_building(*building_id)
        .await
    {
        Ok(decisions) => HttpResponse::Ok().json(decisions),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": err
        })),
    }
}

/// Lister les décisions par statut pour un immeuble
#[get("/buildings/{building_id}/board-decisions/status/{status}")]
pub async fn list_decisions_by_status(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    path: web::Path<(Uuid, String)>,
) -> impl Responder {
    let (building_id, status) = path.into_inner();

    let _organization_id = match user.require_organization() {
        Ok(org_id) => org_id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": e.to_string()
            }))
        }
    };

    match state
        .board_decision_use_cases
        .list_decisions_by_status(building_id, &status)
        .await
    {
        Ok(decisions) => HttpResponse::Ok().json(decisions),
        Err(err) => HttpResponse::BadRequest().json(serde_json::json!({
            "error": err
        })),
    }
}

/// Lister les décisions en retard pour un immeuble
#[get("/buildings/{building_id}/board-decisions/overdue")]
pub async fn list_overdue_decisions(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    building_id: web::Path<Uuid>,
) -> impl Responder {
    let _organization_id = match user.require_organization() {
        Ok(org_id) => org_id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": e.to_string()
            }))
        }
    };

    match state
        .board_decision_use_cases
        .list_overdue_decisions(*building_id)
        .await
    {
        Ok(decisions) => HttpResponse::Ok().json(decisions),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": err
        })),
    }
}

/// Mettre à jour le statut d'une décision
#[put("/board-decisions/{id}")]
pub async fn update_decision_status(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    id: web::Path<Uuid>,
    request: web::Json<UpdateBoardDecisionDto>,
) -> impl Responder {
    let organization_id = match user.require_organization() {
        Ok(org_id) => org_id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": e.to_string()
            }))
        }
    };

    match state
        .board_decision_use_cases
        .update_decision_status(*id, request.into_inner())
        .await
    {
        Ok(decision) => {
            // Audit log: successful decision update
            if let Ok(decision_uuid) = Uuid::parse_str(&decision.id) {
                AuditLogEntry::new(
                    AuditEventType::BoardDecisionUpdated,
                    Some(user.user_id),
                    Some(organization_id),
                )
                .with_resource("BoardDecision", decision_uuid)
                .log();
            }

            HttpResponse::Ok().json(decision)
        }
        Err(err) => {
            // Audit log: failed decision update
            AuditLogEntry::new(
                AuditEventType::BoardDecisionUpdated,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_error(err.clone())
            .log();

            HttpResponse::BadRequest().json(serde_json::json!({
                "error": err
            }))
        }
    }
}

/// Ajouter des notes à une décision
#[post("/board-decisions/{id}/notes")]
pub async fn add_notes(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    id: web::Path<Uuid>,
    request: web::Json<AddDecisionNotesDto>,
) -> impl Responder {
    let organization_id = match user.require_organization() {
        Ok(org_id) => org_id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": e.to_string()
            }))
        }
    };

    match state
        .board_decision_use_cases
        .add_notes(*id, request.into_inner())
        .await
    {
        Ok(decision) => {
            // Audit log: successful notes addition
            if let Ok(decision_uuid) = Uuid::parse_str(&decision.id) {
                AuditLogEntry::new(
                    AuditEventType::BoardDecisionNotesAdded,
                    Some(user.user_id),
                    Some(organization_id),
                )
                .with_resource("BoardDecision", decision_uuid)
                .log();
            }

            HttpResponse::Ok().json(decision)
        }
        Err(err) => {
            // Audit log: failed notes addition
            AuditLogEntry::new(
                AuditEventType::BoardDecisionNotesAdded,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_error(err.clone())
            .log();

            HttpResponse::BadRequest().json(serde_json::json!({
                "error": err
            }))
        }
    }
}

/// Marquer une décision comme complétée
#[put("/board-decisions/{id}/complete")]
pub async fn complete_decision(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    id: web::Path<Uuid>,
) -> impl Responder {
    let organization_id = match user.require_organization() {
        Ok(org_id) => org_id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": e.to_string()
            }))
        }
    };

    match state.board_decision_use_cases.complete_decision(*id).await {
        Ok(decision) => {
            // Audit log: successful decision completion
            if let Ok(decision_uuid) = Uuid::parse_str(&decision.id) {
                AuditLogEntry::new(
                    AuditEventType::BoardDecisionCompleted,
                    Some(user.user_id),
                    Some(organization_id),
                )
                .with_resource("BoardDecision", decision_uuid)
                .log();
            }

            HttpResponse::Ok().json(decision)
        }
        Err(err) => {
            // Audit log: failed decision completion
            AuditLogEntry::new(
                AuditEventType::BoardDecisionCompleted,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_error(err.clone())
            .log();

            HttpResponse::BadRequest().json(serde_json::json!({
                "error": err
            }))
        }
    }
}

/// Obtenir des statistiques sur les décisions d'un immeuble
#[get("/buildings/{building_id}/board-decisions/stats")]
pub async fn get_decision_stats(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    building_id: web::Path<Uuid>,
) -> impl Responder {
    let _organization_id = match user.require_organization() {
        Ok(org_id) => org_id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": e.to_string()
            }))
        }
    };

    match state
        .board_decision_use_cases
        .get_decision_stats(*building_id)
        .await
    {
        Ok(stats) => HttpResponse::Ok().json(stats),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": err
        })),
    }
}
