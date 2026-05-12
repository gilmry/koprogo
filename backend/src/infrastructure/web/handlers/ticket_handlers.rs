use crate::application::dto::{
    AssignTicketRequest, CancelTicketRequest, CreateTicketRequest, ReopenTicketRequest,
    ResolveTicketRequest, TicketResponse,
};
use crate::domain::entities::TicketStatus;
use crate::infrastructure::audit::{AuditEventType, AuditLogEntry};
use crate::infrastructure::web::{AppState, AuthenticatedUser};
use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use std::collections::HashMap;
use uuid::Uuid;

/// Resolve display names for the requester and assignee of a single ticket.
async fn enrich_ticket(state: &AppState, mut ticket: TicketResponse) -> TicketResponse {
    if let Ok(name) = state
        .user_use_cases
        .find_display_name(ticket.created_by)
        .await
    {
        ticket.requester_name = name;
    }
    if let Some(assignee_id) = ticket.assigned_to {
        if let Ok(name) = state.user_use_cases.find_display_name(assignee_id).await {
            ticket.assigned_to_name = name;
        }
    }
    ticket
}

/// Enrich a list of tickets in a single pass, deduplicating user lookups.
async fn enrich_tickets(state: &AppState, tickets: Vec<TicketResponse>) -> Vec<TicketResponse> {
    let mut user_ids: Vec<Uuid> = Vec::new();
    for t in &tickets {
        user_ids.push(t.created_by);
        if let Some(a) = t.assigned_to {
            user_ids.push(a);
        }
    }
    user_ids.sort();
    user_ids.dedup();

    let mut names: HashMap<Uuid, String> = HashMap::new();
    for id in user_ids {
        if let Ok(Some(name)) = state.user_use_cases.find_display_name(id).await {
            names.insert(id, name);
        }
    }

    tickets
        .into_iter()
        .map(|mut t| {
            t.requester_name = names.get(&t.created_by).cloned();
            t.assigned_to_name = t.assigned_to.and_then(|a| names.get(&a).cloned());
            t
        })
        .collect()
}

// ==================== Ticket CRUD Endpoints ====================

#[utoipa::path(
    post,
    path = "/tickets",
    tag = "Tickets",
    summary = "Create a new maintenance ticket",
    request_body = CreateTicketRequest,
    responses(
        (status = 201, description = "Ticket created"),
        (status = 400, description = "Bad request"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error"),
    ),
    security(("bearer_auth" = []))
)]
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

            let enriched = enrich_ticket(&state, ticket).await;
            HttpResponse::Created().json(enriched)
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

#[utoipa::path(
    get,
    path = "/tickets/{id}",
    tag = "Tickets",
    summary = "Get a ticket by ID",
    params(
        ("id" = Uuid, Path, description = "Ticket ID")
    ),
    responses(
        (status = 200, description = "Ticket found"),
        (status = 404, description = "Ticket not found"),
        (status = 500, description = "Internal server error"),
    ),
    security(("bearer_auth" = []))
)]
#[get("/tickets/{id}")]
pub async fn get_ticket(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    id: web::Path<Uuid>,
) -> impl Responder {
    match state.ticket_use_cases.get_ticket(*id).await {
        Ok(Some(ticket)) => {
            // Multi-tenant isolation: verify ticket belongs to user's organization
            if let Err(e) = user.verify_org_access(ticket.organization_id) {
                return HttpResponse::Forbidden().json(serde_json::json!({ "error": e }));
            }
            let enriched = enrich_ticket(&state, ticket).await;
            HttpResponse::Ok().json(enriched)
        }
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Ticket not found"
        })),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({"error": err})),
    }
}

#[utoipa::path(
    get,
    path = "/buildings/{building_id}/tickets",
    tag = "Tickets",
    summary = "List all tickets for a building",
    params(
        ("building_id" = Uuid, Path, description = "Building ID")
    ),
    responses(
        (status = 200, description = "List of tickets"),
        (status = 500, description = "Internal server error"),
    ),
    security(("bearer_auth" = []))
)]
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
        Ok(tickets) => HttpResponse::Ok().json(enrich_tickets(&state, tickets).await),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({"error": err})),
    }
}

#[utoipa::path(
    get,
    path = "/organizations/{organization_id}/tickets",
    tag = "Tickets",
    summary = "List all tickets for an organization",
    params(
        ("organization_id" = Uuid, Path, description = "Organization ID")
    ),
    responses(
        (status = 200, description = "List of tickets"),
        (status = 500, description = "Internal server error"),
    ),
    security(("bearer_auth" = []))
)]
#[get("/organizations/{organization_id}/tickets")]
pub async fn list_organization_tickets(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    organization_id: web::Path<Uuid>,
) -> impl Responder {
    if let Err(e) = user.verify_org_access(*organization_id) {
        return HttpResponse::Forbidden().json(serde_json::json!({"error": e}));
    }
    match state
        .ticket_use_cases
        .list_tickets_by_organization(*organization_id)
        .await
    {
        Ok(tickets) => HttpResponse::Ok().json(enrich_tickets(&state, tickets).await),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({"error": err})),
    }
}

#[utoipa::path(
    get,
    path = "/tickets/my",
    tag = "Tickets",
    summary = "List tickets created by the authenticated user",
    responses(
        (status = 200, description = "List of tickets"),
        (status = 500, description = "Internal server error"),
    ),
    security(("bearer_auth" = []))
)]
#[get("/tickets/my")]
pub async fn list_my_tickets(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
) -> impl Responder {
    let created_by = user.user_id;

    match state.ticket_use_cases.list_my_tickets(created_by).await {
        Ok(tickets) => HttpResponse::Ok().json(enrich_tickets(&state, tickets).await),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({"error": err})),
    }
}

#[utoipa::path(
    get,
    path = "/tickets/assigned-to-me",
    tag = "Tickets",
    summary = "List tickets assigned to the authenticated user",
    responses(
        (status = 200, description = "List of assigned tickets"),
        (status = 500, description = "Internal server error"),
    ),
    security(("bearer_auth" = []))
)]
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
        Ok(tickets) => HttpResponse::Ok().json(enrich_tickets(&state, tickets).await),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({"error": err})),
    }
}

#[utoipa::path(
    get,
    path = "/buildings/{building_id}/tickets/status/{status}",
    tag = "Tickets",
    summary = "List tickets by status for a building",
    params(
        ("building_id" = Uuid, Path, description = "Building ID"),
        ("status" = String, Path, description = "Ticket status (Open, InProgress, Resolved, Closed, Cancelled)")
    ),
    responses(
        (status = 200, description = "List of tickets"),
        (status = 400, description = "Invalid status"),
        (status = 500, description = "Internal server error"),
    ),
    security(("bearer_auth" = []))
)]
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
        Ok(tickets) => HttpResponse::Ok().json(enrich_tickets(&state, tickets).await),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({"error": err})),
    }
}

#[utoipa::path(
    delete,
    path = "/tickets/{id}",
    tag = "Tickets",
    summary = "Delete a ticket",
    params(
        ("id" = Uuid, Path, description = "Ticket ID")
    ),
    responses(
        (status = 204, description = "Ticket deleted"),
        (status = 400, description = "Bad request"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Ticket not found"),
    ),
    security(("bearer_auth" = []))
)]
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

#[utoipa::path(
    put,
    path = "/tickets/{id}/assign",
    tag = "Tickets",
    summary = "Assign a ticket to a contractor",
    params(
        ("id" = Uuid, Path, description = "Ticket ID")
    ),
    request_body = AssignTicketRequest,
    responses(
        (status = 200, description = "Ticket assigned"),
        (status = 400, description = "Bad request"),
        (status = 401, description = "Unauthorized"),
    ),
    security(("bearer_auth" = []))
)]
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

            let enriched = enrich_ticket(&state, ticket).await;
            HttpResponse::Ok().json(enriched)
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

#[utoipa::path(
    put,
    path = "/tickets/{id}/start-work",
    tag = "Tickets",
    summary = "Start work on an assigned ticket",
    params(
        ("id" = Uuid, Path, description = "Ticket ID")
    ),
    responses(
        (status = 200, description = "Work started"),
        (status = 400, description = "Bad request"),
        (status = 401, description = "Unauthorized"),
    ),
    security(("bearer_auth" = []))
)]
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

            let enriched = enrich_ticket(&state, ticket).await;
            HttpResponse::Ok().json(enriched)
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

#[utoipa::path(
    put,
    path = "/tickets/{id}/resolve",
    tag = "Tickets",
    summary = "Mark a ticket as resolved",
    params(
        ("id" = Uuid, Path, description = "Ticket ID")
    ),
    request_body = ResolveTicketRequest,
    responses(
        (status = 200, description = "Ticket resolved"),
        (status = 400, description = "Bad request"),
        (status = 401, description = "Unauthorized"),
    ),
    security(("bearer_auth" = []))
)]
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

            let enriched = enrich_ticket(&state, ticket).await;
            HttpResponse::Ok().json(enriched)
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

#[utoipa::path(
    put,
    path = "/tickets/{id}/close",
    tag = "Tickets",
    summary = "Close a resolved ticket",
    params(
        ("id" = Uuid, Path, description = "Ticket ID")
    ),
    responses(
        (status = 200, description = "Ticket closed"),
        (status = 400, description = "Bad request"),
        (status = 401, description = "Unauthorized"),
    ),
    security(("bearer_auth" = []))
)]
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

            let enriched = enrich_ticket(&state, ticket).await;
            HttpResponse::Ok().json(enriched)
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

#[utoipa::path(
    put,
    path = "/tickets/{id}/cancel",
    tag = "Tickets",
    summary = "Cancel a ticket",
    params(
        ("id" = Uuid, Path, description = "Ticket ID")
    ),
    request_body = CancelTicketRequest,
    responses(
        (status = 200, description = "Ticket cancelled"),
        (status = 400, description = "Bad request"),
        (status = 401, description = "Unauthorized"),
    ),
    security(("bearer_auth" = []))
)]
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

            let enriched = enrich_ticket(&state, ticket).await;
            HttpResponse::Ok().json(enriched)
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

#[utoipa::path(
    put,
    path = "/tickets/{id}/reopen",
    tag = "Tickets",
    summary = "Reopen a closed or cancelled ticket",
    params(
        ("id" = Uuid, Path, description = "Ticket ID")
    ),
    request_body = ReopenTicketRequest,
    responses(
        (status = 200, description = "Ticket reopened"),
        (status = 400, description = "Bad request"),
        (status = 401, description = "Unauthorized"),
    ),
    security(("bearer_auth" = []))
)]
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

            let enriched = enrich_ticket(&state, ticket).await;
            HttpResponse::Ok().json(enriched)
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

// ==================== Ticket Work Order Endpoint ====================

#[utoipa::path(
    put,
    path = "/tickets/{id}/send-work-order",
    tag = "Tickets",
    summary = "Send work order to contractor (magic link PWA)",
    params(
        ("id" = Uuid, Path, description = "Ticket ID")
    ),
    responses(
        (status = 200, description = "Work order sent"),
        (status = 400, description = "Bad request"),
        (status = 401, description = "Unauthorized"),
    ),
    security(("bearer_auth" = []))
)]
#[put("/tickets/{id}/send-work-order")]
pub async fn send_work_order(
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

    match state.ticket_use_cases.send_work_order(*id).await {
        Ok(ticket) => {
            AuditLogEntry::new(
                AuditEventType::TicketWorkOrderSent,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_resource("Ticket", ticket.id)
            .log();

            let enriched = enrich_ticket(&state, ticket).await;
            HttpResponse::Ok().json(enriched)
        }
        Err(err) => {
            AuditLogEntry::new(
                AuditEventType::TicketWorkOrderSent,
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

#[utoipa::path(
    get,
    path = "/tickets/statistics",
    tag = "Tickets",
    summary = "Get ticket statistics for the organization",
    responses(
        (status = 200, description = "Ticket statistics"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error"),
    ),
    security(("bearer_auth" = []))
)]
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

#[utoipa::path(
    get,
    path = "/tickets/overdue",
    tag = "Tickets",
    summary = "List overdue tickets for the organization",
    params(
        ("max_days" = Option<i64>, Query, description = "Maximum overdue days filter (default: 7)")
    ),
    responses(
        (status = 200, description = "List of overdue tickets"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error"),
    ),
    security(("bearer_auth" = []))
)]
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
        Ok(tickets) => HttpResponse::Ok().json(enrich_tickets(&state, tickets).await),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({"error": err})),
    }
}

#[utoipa::path(
    get,
    path = "/buildings/{building_id}/tickets/statistics",
    tag = "Tickets",
    summary = "Get ticket statistics for a building",
    params(
        ("building_id" = Uuid, Path, description = "Building ID")
    ),
    responses(
        (status = 200, description = "Ticket statistics"),
        (status = 500, description = "Internal server error"),
    ),
    security(("bearer_auth" = []))
)]
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

#[utoipa::path(
    get,
    path = "/buildings/{building_id}/tickets/overdue",
    tag = "Tickets",
    summary = "List overdue tickets for a building",
    params(
        ("building_id" = Uuid, Path, description = "Building ID"),
        ("max_days" = Option<i64>, Query, description = "Maximum overdue days filter (default: 7)")
    ),
    responses(
        (status = 200, description = "List of overdue tickets"),
        (status = 500, description = "Internal server error"),
    ),
    security(("bearer_auth" = []))
)]
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
        Ok(tickets) => HttpResponse::Ok().json(enrich_tickets(&state, tickets).await),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({"error": err})),
    }
}

#[derive(serde::Deserialize)]
pub struct OverdueQuery {
    pub max_days: Option<i64>,
}

// ==================== Assignable Users ====================

/// DTO for the assignable-users dropdown (minimal fields).
#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct AssignableUserDto {
    pub id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub role: String,
    pub profession: Option<String>,
}

/// List users that can be assigned to a ticket (syndic, board_member, contractor).
/// Sorted by role (contractors first) then last_name.
#[utoipa::path(
    get,
    path = "/tickets/assignable-users",
    tag = "Tickets",
    summary = "List users assignable to tickets",
    responses(
        (status = 200, description = "List of assignable users"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden — only syndic/superadmin"),
    ),
    security(("bearer_auth" = []))
)]
#[get("/tickets/assignable-users")]
pub async fn list_assignable_users(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
) -> impl Responder {
    let organization_id = match user.require_organization() {
        Ok(org_id) => org_id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": e.to_string()}))
        }
    };

    // Fetch all org users + filter roles eligible for ticket assignment
    match state.user_use_cases.list_all().await {
        Ok(users) => {
            let assignable: Vec<AssignableUserDto> = users
                .into_iter()
                .filter(|u| {
                    // Same org check
                    u.organization_id.as_deref() == Some(&organization_id.to_string())
                })
                .filter(|u| matches!(u.role.as_str(), "syndic" | "board_member" | "contractor"))
                .map(|u| AssignableUserDto {
                    id: u.id.parse().unwrap_or_default(),
                    first_name: u.first_name.clone(),
                    last_name: u.last_name.clone(),
                    role: u.role.clone(),
                    profession: None, // TODO: join contractor_profiles when needed
                })
                .collect();
            HttpResponse::Ok().json(assignable)
        }
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({"error": err})),
    }
}
