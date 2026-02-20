use crate::application::dto::{
    AssignTicketRequest, CancelTicketRequest, CreateTicketRequest, ReopenTicketRequest,
    ResolveTicketRequest,
};
use crate::domain::entities::TicketStatus;
use crate::infrastructure::audit::{AuditEventType, AuditLogEntry};
use crate::infrastructure::web::{AppState, AuthenticatedUser};
use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use uuid::Uuid;

// ==================== Ticket CRUD Endpoints ====================

#[post("/tickets")]
pub async fn create_ticket(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    request: web::Json<CreateTicketRequest>,
) -> impl Responder {
    let organization_id = match user.require_organization() {
        Ok(org_id) => org_id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": e.to_string()}))
        }
    };

    // Use owner_id from AuthenticatedUser as created_by
    let created_by = user.user_id;

    match state
        .ticket_use_cases
        .create_ticket(organization_id, created_by, request.into_inner())
        .await
    {
        Ok(ticket) => {
            AuditLogEntry::new(
                AuditEventType::TicketCreated,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_resource("Ticket", ticket.id)
            .log();

            HttpResponse::Created().json(ticket)
        }
        Err(err) => {
            AuditLogEntry::new(
                AuditEventType::TicketCreated,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_error(err.clone())
            .log();

            HttpResponse::BadRequest().json(serde_json::json!({"error": err}))
        }
    }
}

#[get("/tickets/{id}")]
pub async fn get_ticket(state: web::Data<AppState>, id: web::Path<Uuid>) -> impl Responder {
    match state.ticket_use_cases.get_ticket(*id).await {
        Ok(Some(ticket)) => HttpResponse::Ok().json(ticket),
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Ticket not found"
        })),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({"error": err})),
    }
}

#[get("/buildings/{building_id}/tickets")]
pub async fn list_building_tickets(
    state: web::Data<AppState>,
    building_id: web::Path<Uuid>,
) -> impl Responder {
    match state
        .ticket_use_cases
        .list_tickets_by_building(*building_id)
        .await
    {
        Ok(tickets) => HttpResponse::Ok().json(tickets),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({"error": err})),
    }
}

#[get("/organizations/{organization_id}/tickets")]
pub async fn list_organization_tickets(
    state: web::Data<AppState>,
    organization_id: web::Path<Uuid>,
) -> impl Responder {
    match state
        .ticket_use_cases
        .list_tickets_by_organization(*organization_id)
        .await
    {
        Ok(tickets) => HttpResponse::Ok().json(tickets),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({"error": err})),
    }
}

#[get("/tickets/my-tickets")]
pub async fn list_my_tickets(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
) -> impl Responder {
    let created_by = user.user_id;

    match state.ticket_use_cases.list_my_tickets(created_by).await {
        Ok(tickets) => HttpResponse::Ok().json(tickets),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({"error": err})),
    }
}

#[get("/tickets/assigned-to-me")]
pub async fn list_assigned_tickets(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
) -> impl Responder {
    let assigned_to = user.user_id;

    match state
        .ticket_use_cases
        .list_assigned_tickets(assigned_to)
        .await
    {
        Ok(tickets) => HttpResponse::Ok().json(tickets),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({"error": err})),
    }
}

#[get("/buildings/{building_id}/tickets/status/{status}")]
pub async fn list_tickets_by_status(
    state: web::Data<AppState>,
    path: web::Path<(Uuid, String)>,
) -> impl Responder {
    let (building_id, status_str) = path.into_inner();

    let status = match status_str.as_str() {
        "Open" | "open" => TicketStatus::Open,
        "InProgress" | "in_progress" => TicketStatus::InProgress,
        "Resolved" | "resolved" => TicketStatus::Resolved,
        "Closed" | "closed" => TicketStatus::Closed,
        "Cancelled" | "cancelled" => TicketStatus::Cancelled,
        _ => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": format!("Invalid status: {}", status_str)
            }))
        }
    };

    match state
        .ticket_use_cases
        .list_tickets_by_status(building_id, status)
        .await
    {
        Ok(tickets) => HttpResponse::Ok().json(tickets),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({"error": err})),
    }
}

#[delete("/tickets/{id}")]
pub async fn delete_ticket(
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

    match state.ticket_use_cases.delete_ticket(*id).await {
        Ok(true) => {
            AuditLogEntry::new(
                AuditEventType::TicketDeleted,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_resource("Ticket", *id)
            .log();

            HttpResponse::NoContent().finish()
        }
        Ok(false) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Ticket not found"
        })),
        Err(err) => {
            AuditLogEntry::new(
                AuditEventType::TicketDeleted,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_error(err.clone())
            .log();

            HttpResponse::BadRequest().json(serde_json::json!({"error": err}))
        }
    }
}

// ==================== Ticket Workflow Endpoints ====================

#[put("/tickets/{id}/assign")]
pub async fn assign_ticket(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    id: web::Path<Uuid>,
    request: web::Json<AssignTicketRequest>,
) -> impl Responder {
    let organization_id = match user.require_organization() {
        Ok(org_id) => org_id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": e.to_string()}))
        }
    };

    match state
        .ticket_use_cases
        .assign_ticket(*id, request.into_inner())
        .await
    {
        Ok(ticket) => {
            AuditLogEntry::new(
                AuditEventType::TicketAssigned,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_resource("Ticket", ticket.id)
            .log();

            HttpResponse::Ok().json(ticket)
        }
        Err(err) => {
            AuditLogEntry::new(
                AuditEventType::TicketAssigned,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_error(err.clone())
            .log();

            HttpResponse::BadRequest().json(serde_json::json!({"error": err}))
        }
    }
}

#[put("/tickets/{id}/start-work")]
pub async fn start_work(
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

    match state.ticket_use_cases.start_work(*id).await {
        Ok(ticket) => {
            AuditLogEntry::new(
                AuditEventType::TicketStatusChanged,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_resource("Ticket", ticket.id)
            .log();

            HttpResponse::Ok().json(ticket)
        }
        Err(err) => {
            AuditLogEntry::new(
                AuditEventType::TicketStatusChanged,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_error(err.clone())
            .log();

            HttpResponse::BadRequest().json(serde_json::json!({"error": err}))
        }
    }
}

#[put("/tickets/{id}/resolve")]
pub async fn resolve_ticket(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    id: web::Path<Uuid>,
    request: web::Json<ResolveTicketRequest>,
) -> impl Responder {
    let organization_id = match user.require_organization() {
        Ok(org_id) => org_id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": e.to_string()}))
        }
    };

    match state
        .ticket_use_cases
        .resolve_ticket(*id, request.into_inner())
        .await
    {
        Ok(ticket) => {
            AuditLogEntry::new(
                AuditEventType::TicketResolved,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_resource("Ticket", ticket.id)
            .log();

            HttpResponse::Ok().json(ticket)
        }
        Err(err) => {
            AuditLogEntry::new(
                AuditEventType::TicketResolved,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_error(err.clone())
            .log();

            HttpResponse::BadRequest().json(serde_json::json!({"error": err}))
        }
    }
}

#[put("/tickets/{id}/close")]
pub async fn close_ticket(
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

    match state.ticket_use_cases.close_ticket(*id).await {
        Ok(ticket) => {
            AuditLogEntry::new(
                AuditEventType::TicketClosed,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_resource("Ticket", ticket.id)
            .log();

            HttpResponse::Ok().json(ticket)
        }
        Err(err) => {
            AuditLogEntry::new(
                AuditEventType::TicketClosed,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_error(err.clone())
            .log();

            HttpResponse::BadRequest().json(serde_json::json!({"error": err}))
        }
    }
}

#[put("/tickets/{id}/cancel")]
pub async fn cancel_ticket(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    id: web::Path<Uuid>,
    request: web::Json<CancelTicketRequest>,
) -> impl Responder {
    let organization_id = match user.require_organization() {
        Ok(org_id) => org_id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": e.to_string()}))
        }
    };

    match state
        .ticket_use_cases
        .cancel_ticket(*id, request.into_inner())
        .await
    {
        Ok(ticket) => {
            AuditLogEntry::new(
                AuditEventType::TicketCancelled,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_resource("Ticket", ticket.id)
            .log();

            HttpResponse::Ok().json(ticket)
        }
        Err(err) => {
            AuditLogEntry::new(
                AuditEventType::TicketCancelled,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_error(err.clone())
            .log();

            HttpResponse::BadRequest().json(serde_json::json!({"error": err}))
        }
    }
}

#[put("/tickets/{id}/reopen")]
pub async fn reopen_ticket(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    id: web::Path<Uuid>,
    request: web::Json<ReopenTicketRequest>,
) -> impl Responder {
    let organization_id = match user.require_organization() {
        Ok(org_id) => org_id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": e.to_string()}))
        }
    };

    match state
        .ticket_use_cases
        .reopen_ticket(*id, request.into_inner())
        .await
    {
        Ok(ticket) => {
            AuditLogEntry::new(
                AuditEventType::TicketReopened,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_resource("Ticket", ticket.id)
            .log();

            HttpResponse::Ok().json(ticket)
        }
        Err(err) => {
            AuditLogEntry::new(
                AuditEventType::TicketReopened,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_error(err.clone())
            .log();

            HttpResponse::BadRequest().json(serde_json::json!({"error": err}))
        }
    }
}

// ==================== Ticket Statistics Endpoints ====================

#[get("/tickets/statistics")]
pub async fn get_ticket_statistics_org(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
) -> impl Responder {
    let organization_id = match user.require_organization() {
        Ok(org_id) => org_id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": e.to_string()}))
        }
    };

    match state
        .ticket_use_cases
        .get_ticket_statistics_by_organization(organization_id)
        .await
    {
        Ok(stats) => HttpResponse::Ok().json(stats),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({"error": err})),
    }
}

#[get("/tickets/overdue")]
pub async fn get_overdue_tickets_org(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    query: web::Query<OverdueQuery>,
) -> impl Responder {
    let organization_id = match user.require_organization() {
        Ok(org_id) => org_id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": e.to_string()}))
        }
    };

    let max_days = query.max_days.unwrap_or(7);

    match state
        .ticket_use_cases
        .get_overdue_tickets_by_organization(organization_id, max_days)
        .await
    {
        Ok(tickets) => HttpResponse::Ok().json(tickets),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({"error": err})),
    }
}

#[get("/buildings/{building_id}/tickets/statistics")]
pub async fn get_ticket_statistics(
    state: web::Data<AppState>,
    building_id: web::Path<Uuid>,
) -> impl Responder {
    match state
        .ticket_use_cases
        .get_ticket_statistics(*building_id)
        .await
    {
        Ok(stats) => HttpResponse::Ok().json(stats),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({"error": err})),
    }
}

#[get("/buildings/{building_id}/tickets/overdue")]
pub async fn get_overdue_tickets(
    state: web::Data<AppState>,
    building_id: web::Path<Uuid>,
    query: web::Query<OverdueQuery>,
) -> impl Responder {
    let max_days = query.max_days.unwrap_or(7);

    match state
        .ticket_use_cases
        .get_overdue_tickets(*building_id, max_days)
        .await
    {
        Ok(tickets) => HttpResponse::Ok().json(tickets),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({"error": err})),
    }
}

#[derive(serde::Deserialize)]
pub struct OverdueQuery {
    pub max_days: Option<i64>,
}
