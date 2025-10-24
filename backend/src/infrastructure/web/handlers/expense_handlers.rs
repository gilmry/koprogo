use crate::application::dto::{CreateExpenseDto, PageRequest, PageResponse};
use crate::infrastructure::audit::{AuditEventType, AuditLogEntry};
use crate::infrastructure::web::{AppState, AuthenticatedUser};
use actix_web::{get, post, put, web, HttpResponse, Responder};
use uuid::Uuid;
use validator::Validate;

#[post("/expenses")]
pub async fn create_expense(
    state: web::Data<AppState>,
    user: AuthenticatedUser, // JWT-extracted user info (SECURE!)
    mut dto: web::Json<CreateExpenseDto>,
) -> impl Responder {
    // Override the organization_id from DTO with the one from JWT token
    // This prevents users from creating expenses in other organizations
    let organization_id = match user.require_organization() {
        Ok(org_id) => org_id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": e.to_string()
            }))
        }
    };
    dto.organization_id = organization_id.to_string();

    if let Err(errors) = dto.validate() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Validation failed",
            "details": errors.to_string()
        }));
    }

    match state
        .expense_use_cases
        .create_expense(dto.into_inner())
        .await
    {
        Ok(expense) => {
            // Audit log: successful expense creation
            AuditLogEntry::new(
                AuditEventType::ExpenseCreated,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_resource("Expense", Uuid::parse_str(&expense.id).unwrap())
            .log();

            HttpResponse::Created().json(expense)
        }
        Err(err) => {
            // Audit log: failed expense creation
            AuditLogEntry::new(
                AuditEventType::ExpenseCreated,
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

#[get("/expenses/{id}")]
pub async fn get_expense(state: web::Data<AppState>, id: web::Path<Uuid>) -> impl Responder {
    match state.expense_use_cases.get_expense(*id).await {
        Ok(Some(expense)) => HttpResponse::Ok().json(expense),
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Expense not found"
        })),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": err
        })),
    }
}

#[get("/expenses")]
pub async fn list_expenses(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    page_request: web::Query<PageRequest>,
) -> impl Responder {
    let organization_id = user.organization_id;

    match state
        .expense_use_cases
        .list_expenses_paginated(&page_request, organization_id)
        .await
    {
        Ok((expenses, total)) => {
            let response =
                PageResponse::new(expenses, page_request.page, page_request.per_page, total);
            HttpResponse::Ok().json(response)
        }
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": err
        })),
    }
}

#[get("/buildings/{building_id}/expenses")]
pub async fn list_expenses_by_building(
    state: web::Data<AppState>,
    building_id: web::Path<Uuid>,
) -> impl Responder {
    match state
        .expense_use_cases
        .list_expenses_by_building(*building_id)
        .await
    {
        Ok(expenses) => HttpResponse::Ok().json(expenses),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": err
        })),
    }
}

#[put("/expenses/{id}/mark-paid")]
pub async fn mark_expense_paid(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    id: web::Path<Uuid>,
) -> impl Responder {
    match state.expense_use_cases.mark_as_paid(*id).await {
        Ok(expense) => {
            // Audit log: successful expense marked paid
            AuditLogEntry::new(
                AuditEventType::ExpenseMarkedPaid,
                Some(user.user_id),
                user.organization_id,
            )
            .with_resource("Expense", *id)
            .log();

            HttpResponse::Ok().json(expense)
        }
        Err(err) => {
            // Audit log: failed expense marked paid
            AuditLogEntry::new(
                AuditEventType::ExpenseMarkedPaid,
                Some(user.user_id),
                user.organization_id,
            )
            .with_resource("Expense", *id)
            .with_error(err.clone())
            .log();

            HttpResponse::BadRequest().json(serde_json::json!({
                "error": err
            }))
        }
    }
}
