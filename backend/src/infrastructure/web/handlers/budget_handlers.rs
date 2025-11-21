use crate::application::dto::{
    CreateBudgetRequest, PageRequest, PageResponse, UpdateBudgetRequest,
};
use crate::domain::entities::BudgetStatus;
use crate::infrastructure::audit::{AuditEventType, AuditLogEntry};
use crate::infrastructure::web::{AppState, AuthenticatedUser};
use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use uuid::Uuid;

/// Create a new budget
#[post("/budgets")]
pub async fn create_budget(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    mut request: web::Json<CreateBudgetRequest>,
) -> impl Responder {
    // Override organization_id from JWT token (security)
    let organization_id = match user.require_organization() {
        Ok(org_id) => org_id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": e.to_string()
            }))
        }
    };
    request.organization_id = organization_id;

    match state
        .budget_use_cases
        .create_budget(request.into_inner())
        .await
    {
        Ok(budget) => {
            AuditLogEntry::new(
                AuditEventType::BudgetCreated,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_resource("Budget", budget.id)
            .log();

            HttpResponse::Created().json(budget)
        }
        Err(err) => {
            AuditLogEntry::new(
                AuditEventType::BudgetCreated,
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

/// Get budget by ID
#[get("/budgets/{id}")]
pub async fn get_budget(state: web::Data<AppState>, id: web::Path<Uuid>) -> impl Responder {
    match state.budget_use_cases.get_budget(*id).await {
        Ok(Some(budget)) => HttpResponse::Ok().json(budget),
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Budget not found"
        })),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": err
        })),
    }
}

/// Get budget by building and fiscal year
#[get("/buildings/{building_id}/budgets/fiscal-year/{fiscal_year}")]
pub async fn get_budget_by_building_and_fiscal_year(
    state: web::Data<AppState>,
    params: web::Path<(Uuid, i32)>,
) -> impl Responder {
    let (building_id, fiscal_year) = params.into_inner();

    match state
        .budget_use_cases
        .get_by_building_and_fiscal_year(building_id, fiscal_year)
        .await
    {
        Ok(Some(budget)) => HttpResponse::Ok().json(budget),
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Budget not found"
        })),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": err
        })),
    }
}

/// Get active budget for a building
#[get("/buildings/{building_id}/budgets/active")]
pub async fn get_active_budget(
    state: web::Data<AppState>,
    building_id: web::Path<Uuid>,
) -> impl Responder {
    match state.budget_use_cases.get_active_budget(*building_id).await {
        Ok(Some(budget)) => HttpResponse::Ok().json(budget),
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "No active budget found for this building"
        })),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": err
        })),
    }
}

/// List budgets for a building
#[get("/buildings/{building_id}/budgets")]
pub async fn list_budgets_by_building(
    state: web::Data<AppState>,
    building_id: web::Path<Uuid>,
) -> impl Responder {
    match state.budget_use_cases.list_by_building(*building_id).await {
        Ok(budgets) => HttpResponse::Ok().json(budgets),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": err
        })),
    }
}

/// List budgets by fiscal year
#[get("/budgets/fiscal-year/{fiscal_year}")]
pub async fn list_budgets_by_fiscal_year(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    fiscal_year: web::Path<i32>,
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
        .budget_use_cases
        .list_by_fiscal_year(organization_id, *fiscal_year)
        .await
    {
        Ok(budgets) => HttpResponse::Ok().json(budgets),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": err
        })),
    }
}

/// List budgets by status
#[get("/budgets/status/{status}")]
pub async fn list_budgets_by_status(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    status: web::Path<String>,
) -> impl Responder {
    let organization_id = match user.require_organization() {
        Ok(org_id) => org_id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": e.to_string()
            }))
        }
    };

    let budget_status = match status.as_str() {
        "draft" => BudgetStatus::Draft,
        "submitted" => BudgetStatus::Submitted,
        "approved" => BudgetStatus::Approved,
        "rejected" => BudgetStatus::Rejected,
        "archived" => BudgetStatus::Archived,
        _ => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid status"
            }))
        }
    };

    match state
        .budget_use_cases
        .list_by_status(organization_id, budget_status)
        .await
    {
        Ok(budgets) => HttpResponse::Ok().json(budgets),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": err
        })),
    }
}

/// List budgets paginated
#[get("/budgets")]
pub async fn list_budgets(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    page_request: web::Query<PageRequest>,
    filters: web::Query<serde_json::Value>,
) -> impl Responder {
    let organization_id = user.organization_id;

    // Parse optional filters
    let building_id = filters
        .get("building_id")
        .and_then(|v| v.as_str())
        .and_then(|s| Uuid::parse_str(s).ok());

    let status = filters
        .get("status")
        .and_then(|v| v.as_str())
        .and_then(|s| match s {
            "draft" => Some(BudgetStatus::Draft),
            "submitted" => Some(BudgetStatus::Submitted),
            "approved" => Some(BudgetStatus::Approved),
            "rejected" => Some(BudgetStatus::Rejected),
            "archived" => Some(BudgetStatus::Archived),
            _ => None,
        });

    match state
        .budget_use_cases
        .list_paginated(&page_request, organization_id, building_id, status)
        .await
    {
        Ok((budgets, total)) => {
            let response =
                PageResponse::new(budgets, page_request.page, page_request.per_page, total);
            HttpResponse::Ok().json(response)
        }
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": err
        })),
    }
}

/// Update budget (Draft only)
#[put("/budgets/{id}")]
pub async fn update_budget(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    id: web::Path<Uuid>,
    request: web::Json<UpdateBudgetRequest>,
) -> impl Responder {
    match state
        .budget_use_cases
        .update_budget(*id, request.into_inner())
        .await
    {
        Ok(budget) => {
            AuditLogEntry::new(
                AuditEventType::BudgetUpdated,
                Some(user.user_id),
                user.organization_id,
            )
            .with_resource("Budget", budget.id)
            .log();

            HttpResponse::Ok().json(budget)
        }
        Err(err) => HttpResponse::BadRequest().json(serde_json::json!({
            "error": err
        })),
    }
}

/// Submit budget for approval
#[put("/budgets/{id}/submit")]
pub async fn submit_budget(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    id: web::Path<Uuid>,
) -> impl Responder {
    match state.budget_use_cases.submit_for_approval(*id).await {
        Ok(budget) => {
            AuditLogEntry::new(
                AuditEventType::BudgetSubmitted,
                Some(user.user_id),
                user.organization_id,
            )
            .with_resource("Budget", budget.id)
            .log();

            HttpResponse::Ok().json(budget)
        }
        Err(err) => HttpResponse::BadRequest().json(serde_json::json!({
            "error": err
        })),
    }
}

/// Approve budget (requires meeting_id)
#[put("/budgets/{id}/approve")]
pub async fn approve_budget(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    id: web::Path<Uuid>,
    payload: web::Json<serde_json::Value>,
) -> impl Responder {
    let meeting_id = match payload.get("meeting_id") {
        Some(serde_json::Value::String(id_str)) => match Uuid::parse_str(id_str) {
            Ok(uuid) => uuid,
            Err(_) => {
                return HttpResponse::BadRequest().json(serde_json::json!({
                    "error": "Invalid meeting_id format"
                }))
            }
        },
        _ => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "meeting_id is required as a UUID string"
            }))
        }
    };

    match state.budget_use_cases.approve_budget(*id, meeting_id).await {
        Ok(budget) => {
            AuditLogEntry::new(
                AuditEventType::BudgetApproved,
                Some(user.user_id),
                user.organization_id,
            )
            .with_resource("Budget", budget.id)
            .with_metadata(serde_json::json!({"meeting_id": meeting_id}))
            .log();

            HttpResponse::Ok().json(budget)
        }
        Err(err) => HttpResponse::BadRequest().json(serde_json::json!({
            "error": err
        })),
    }
}

/// Reject budget (with optional reason)
#[put("/budgets/{id}/reject")]
pub async fn reject_budget(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    id: web::Path<Uuid>,
    payload: web::Json<serde_json::Value>,
) -> impl Responder {
    let reason = payload
        .get("reason")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    match state.budget_use_cases.reject_budget(*id, reason).await {
        Ok(budget) => {
            AuditLogEntry::new(
                AuditEventType::BudgetRejected,
                Some(user.user_id),
                user.organization_id,
            )
            .with_resource("Budget", budget.id)
            .log();

            HttpResponse::Ok().json(budget)
        }
        Err(err) => HttpResponse::BadRequest().json(serde_json::json!({
            "error": err
        })),
    }
}

/// Archive budget
#[put("/budgets/{id}/archive")]
pub async fn archive_budget(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    id: web::Path<Uuid>,
) -> impl Responder {
    match state.budget_use_cases.archive_budget(*id).await {
        Ok(budget) => {
            AuditLogEntry::new(
                AuditEventType::BudgetArchived,
                Some(user.user_id),
                user.organization_id,
            )
            .with_resource("Budget", budget.id)
            .log();

            HttpResponse::Ok().json(budget)
        }
        Err(err) => HttpResponse::BadRequest().json(serde_json::json!({
            "error": err
        })),
    }
}

/// Get budget statistics
#[get("/budgets/stats")]
pub async fn get_budget_stats(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
) -> impl Responder {
    let organization_id = match user.require_organization() {
        Ok(org_id) => org_id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": e.to_string()
            }))
        }
    };

    match state.budget_use_cases.get_stats(organization_id).await {
        Ok(stats) => HttpResponse::Ok().json(stats),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": err
        })),
    }
}

/// Get budget variance analysis (budget vs actual)
#[get("/budgets/{id}/variance")]
pub async fn get_budget_variance(
    state: web::Data<AppState>,
    id: web::Path<Uuid>,
) -> impl Responder {
    match state.budget_use_cases.get_variance(*id).await {
        Ok(Some(variance)) => HttpResponse::Ok().json(variance),
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Budget not found"
        })),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": err
        })),
    }
}

/// Delete budget
#[delete("/budgets/{id}")]
pub async fn delete_budget(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    id: web::Path<Uuid>,
) -> impl Responder {
    match state.budget_use_cases.delete_budget(*id).await {
        Ok(true) => {
            AuditLogEntry::new(
                AuditEventType::BudgetDeleted,
                Some(user.user_id),
                user.organization_id,
            )
            .with_resource("Budget", *id)
            .log();

            HttpResponse::NoContent().finish()
        }
        Ok(false) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Budget not found"
        })),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": err
        })),
    }
}
