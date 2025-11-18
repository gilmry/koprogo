use crate::application::dto::{
    ApproveInvoiceDto, CreateExpenseDto, CreateInvoiceDraftDto, PageRequest, PageResponse,
    RejectInvoiceDto, SubmitForApprovalDto, UpdateInvoiceDraftDto,
};
use crate::infrastructure::audit::{AuditEventType, AuditLogEntry};
use crate::infrastructure::web::{AppState, AuthenticatedUser};
use actix_web::{get, post, put, web, HttpResponse, Responder};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use uuid::Uuid;
use validator::Validate;

/// Helper function to check if owner role is trying to modify data
/// Note: Accountant CAN create expenses and mark them as paid
fn check_owner_readonly(user: &AuthenticatedUser) -> Option<HttpResponse> {
    if user.role == "owner" {
        Some(HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Owner role has read-only access"
        })))
    } else {
        None
    }
}

/// Helper function to check if user has syndic role (for approval workflow)
fn check_syndic_role(user: &AuthenticatedUser) -> Option<HttpResponse> {
    if user.role != "syndic" && user.role != "superadmin" {
        Some(HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Only syndic or superadmin can approve/reject invoices"
        })))
    } else {
        None
    }
}

/// Helper function to check if user has accountant role (for creating/editing invoices)
fn check_accountant_role(user: &AuthenticatedUser) -> Option<HttpResponse> {
    if user.role != "accountant" && user.role != "syndic" && user.role != "superadmin" {
        Some(HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Only accountant, syndic, or superadmin can create/edit invoices"
        })))
    } else {
        None
    }
}

#[post("/expenses")]
pub async fn create_expense(
    state: web::Data<AppState>,
    user: AuthenticatedUser, // JWT-extracted user info (SECURE!)
    mut dto: web::Json<CreateExpenseDto>,
) -> impl Responder {
    if let Some(response) = check_owner_readonly(&user) {
        return response;
    }

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
    if let Some(response) = check_owner_readonly(&user) {
        return response;
    }

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

#[post("/expenses/{id}/mark-overdue")]
pub async fn mark_expense_overdue(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    id: web::Path<Uuid>,
) -> impl Responder {
    match state.expense_use_cases.mark_as_overdue(*id).await {
        Ok(expense) => {
            AuditLogEntry::new(
                AuditEventType::ExpenseMarkedPaid,
                Some(user.user_id),
                user.organization_id,
            )
            .with_resource("Expense", *id)
            .log();

            HttpResponse::Ok().json(expense)
        }
        Err(err) => HttpResponse::BadRequest().json(serde_json::json!({
            "error": err
        })),
    }
}

#[post("/expenses/{id}/cancel")]
pub async fn cancel_expense(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    id: web::Path<Uuid>,
) -> impl Responder {
    match state.expense_use_cases.cancel_expense(*id).await {
        Ok(expense) => {
            AuditLogEntry::new(
                AuditEventType::ExpenseMarkedPaid,
                Some(user.user_id),
                user.organization_id,
            )
            .with_resource("Expense", *id)
            .log();

            HttpResponse::Ok().json(expense)
        }
        Err(err) => HttpResponse::BadRequest().json(serde_json::json!({
            "error": err
        })),
    }
}

#[post("/expenses/{id}/reactivate")]
pub async fn reactivate_expense(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    id: web::Path<Uuid>,
) -> impl Responder {
    match state.expense_use_cases.reactivate_expense(*id).await {
        Ok(expense) => {
            AuditLogEntry::new(
                AuditEventType::ExpenseMarkedPaid,
                Some(user.user_id),
                user.organization_id,
            )
            .with_resource("Expense", *id)
            .log();

            HttpResponse::Ok().json(expense)
        }
        Err(err) => HttpResponse::BadRequest().json(serde_json::json!({
            "error": err
        })),
    }
}

#[post("/expenses/{id}/unpay")]
pub async fn unpay_expense(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    id: web::Path<Uuid>,
) -> impl Responder {
    match state.expense_use_cases.unpay_expense(*id).await {
        Ok(expense) => {
            AuditLogEntry::new(
                AuditEventType::ExpenseMarkedPaid,
                Some(user.user_id),
                user.organization_id,
            )
            .with_resource("Expense", *id)
            .log();

            HttpResponse::Ok().json(expense)
        }
        Err(err) => HttpResponse::BadRequest().json(serde_json::json!({
            "error": err
        })),
    }
}

// ========== Invoice Workflow Endpoints (Issue #73) ==========

/// POST /invoices/draft - Create a new invoice draft with VAT
#[post("/invoices/draft")]
pub async fn create_invoice_draft(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    mut dto: web::Json<CreateInvoiceDraftDto>,
) -> impl Responder {
    if let Some(response) = check_accountant_role(&user) {
        return response;
    }

    // Override organization_id from JWT token
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
        .create_invoice_draft(dto.into_inner())
        .await
    {
        Ok(invoice) => {
            AuditLogEntry::new(
                AuditEventType::ExpenseCreated,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_resource("Invoice", Uuid::parse_str(&invoice.id).unwrap())
            .log();

            HttpResponse::Created().json(invoice)
        }
        Err(err) => {
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

/// PUT /invoices/{id} - Update invoice draft (only if Draft or Rejected)
#[put("/invoices/{id}")]
pub async fn update_invoice_draft(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    id: web::Path<Uuid>,
    dto: web::Json<UpdateInvoiceDraftDto>,
) -> impl Responder {
    if let Some(response) = check_accountant_role(&user) {
        return response;
    }

    if let Err(errors) = dto.validate() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Validation failed",
            "details": errors.to_string()
        }));
    }

    match state
        .expense_use_cases
        .update_invoice_draft(*id, dto.into_inner())
        .await
    {
        Ok(invoice) => {
            AuditLogEntry::new(
                AuditEventType::ExpenseMarkedPaid, // TODO: Add InvoiceUpdated event type
                Some(user.user_id),
                user.organization_id,
            )
            .with_resource("Invoice", *id)
            .log();

            HttpResponse::Ok().json(invoice)
        }
        Err(err) => HttpResponse::BadRequest().json(serde_json::json!({
            "error": err
        })),
    }
}

/// PUT /invoices/{id}/submit - Submit invoice for approval (Draft → PendingApproval)
#[put("/invoices/{id}/submit")]
pub async fn submit_invoice_for_approval(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    id: web::Path<Uuid>,
) -> impl Responder {
    if let Some(response) = check_accountant_role(&user) {
        return response;
    }

    match state
        .expense_use_cases
        .submit_for_approval(*id, SubmitForApprovalDto {})
        .await
    {
        Ok(invoice) => {
            AuditLogEntry::new(
                AuditEventType::ExpenseMarkedPaid, // TODO: Add InvoiceSubmitted event type
                Some(user.user_id),
                user.organization_id,
            )
            .with_resource("Invoice", *id)
            .log();

            HttpResponse::Ok().json(invoice)
        }
        Err(err) => HttpResponse::BadRequest().json(serde_json::json!({
            "error": err
        })),
    }
}

/// PUT /invoices/{id}/approve - Approve invoice (PendingApproval → Approved)
/// Only syndic or superadmin can approve
#[put("/invoices/{id}/approve")]
pub async fn approve_invoice(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    id: web::Path<Uuid>,
) -> impl Responder {
    if let Some(response) = check_syndic_role(&user) {
        return response;
    }

    let dto = ApproveInvoiceDto {
        approved_by_user_id: user.user_id.to_string(),
    };

    match state.expense_use_cases.approve_invoice(*id, dto).await {
        Ok(invoice) => {
            AuditLogEntry::new(
                AuditEventType::ExpenseMarkedPaid, // TODO: Add InvoiceApproved event type
                Some(user.user_id),
                user.organization_id,
            )
            .with_resource("Invoice", *id)
            .log();

            HttpResponse::Ok().json(invoice)
        }
        Err(err) => HttpResponse::BadRequest().json(serde_json::json!({
            "error": err
        })),
    }
}

/// PUT /invoices/{id}/reject - Reject invoice with reason (PendingApproval → Rejected)
/// Only syndic or superadmin can reject
#[put("/invoices/{id}/reject")]
pub async fn reject_invoice(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    id: web::Path<Uuid>,
    dto: web::Json<RejectInvoiceDto>,
) -> impl Responder {
    if let Some(response) = check_syndic_role(&user) {
        return response;
    }

    if let Err(errors) = dto.validate() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Validation failed",
            "details": errors.to_string()
        }));
    }

    let mut reject_dto = dto.into_inner();
    reject_dto.rejected_by_user_id = user.user_id.to_string();

    match state
        .expense_use_cases
        .reject_invoice(*id, reject_dto)
        .await
    {
        Ok(invoice) => {
            AuditLogEntry::new(
                AuditEventType::ExpenseMarkedPaid, // TODO: Add InvoiceRejected event type
                Some(user.user_id),
                user.organization_id,
            )
            .with_resource("Invoice", *id)
            .log();

            HttpResponse::Ok().json(invoice)
        }
        Err(err) => HttpResponse::BadRequest().json(serde_json::json!({
            "error": err
        })),
    }
}

/// GET /invoices/pending - Get all pending invoices (for syndic dashboard)
/// Only syndic or superadmin can view pending invoices
#[get("/invoices/pending")]
pub async fn get_pending_invoices(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
) -> impl Responder {
    if let Some(response) = check_syndic_role(&user) {
        return response;
    }

    let organization_id = match user.require_organization() {
        Ok(org_id) => org_id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": e.to_string()
            }))
        }
    };

    match state
        .expense_use_cases
        .get_pending_invoices(organization_id)
        .await
    {
        Ok(pending_list) => HttpResponse::Ok().json(pending_list),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": err
        })),
    }
}

/// GET /invoices/{id} - Get full invoice details (enriched with all fields)
#[get("/invoices/{id}")]
pub async fn get_invoice(state: web::Data<AppState>, id: web::Path<Uuid>) -> impl Responder {
    match state.expense_use_cases.get_invoice(*id).await {
        Ok(Some(invoice)) => HttpResponse::Ok().json(invoice),
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Invoice not found"
        })),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": err
        })),
    }
}

/// Export Work Quote to PDF
///
/// GET /expenses/{expense_id}/export-quote-pdf?contractor_name={name}&contractor_contact={email}&timeline={description}
///
/// Generates a "Devis de Travaux" PDF for work-related expenses.
#[derive(Debug, Deserialize)]
pub struct ExportWorkQuoteQuery {
    pub contractor_name: String,
    pub contractor_contact: String,
    pub timeline: String, // e.g., "2-3 weeks" or "Délai: 15 jours ouvrables"
}

#[get("/expenses/{id}/export-quote-pdf")]
pub async fn export_work_quote_pdf(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    id: web::Path<Uuid>,
    query: web::Query<ExportWorkQuoteQuery>,
) -> impl Responder {
    use crate::domain::entities::{Building, Expense};
    use crate::domain::services::{QuoteLineItem, WorkQuoteExporter};

    let organization_id = match user.require_organization() {
        Ok(org_id) => org_id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": e.to_string()
            }))
        }
    };

    let expense_id = *id;

    // 1. Get expense
    let expense_dto = match state.expense_use_cases.get_expense(expense_id).await {
        Ok(Some(dto)) => dto,
        Ok(None) => {
            return HttpResponse::NotFound().json(serde_json::json!({
                "error": "Expense not found"
            }))
        }
        Err(err) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": err
            }))
        }
    };

    // 2. Parse expense DTO fields
    let expense_building_id = match Uuid::parse_str(&expense_dto.building_id) {
        Ok(id) => id,
        Err(e) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": format!("Invalid building_id: {}", e)
            }))
        }
    };
    let expense_id_uuid = match Uuid::parse_str(&expense_dto.id) {
        Ok(id) => id,
        Err(e) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": format!("Invalid expense_id: {}", e)
            }))
        }
    };

    // 3. Get building
    let building_dto = match state
        .building_use_cases
        .get_building(expense_building_id)
        .await
    {
        Ok(Some(dto)) => dto,
        Ok(None) => {
            return HttpResponse::NotFound().json(serde_json::json!({
                "error": "Building not found"
            }))
        }
        Err(err) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": err
            }))
        }
    };

    // Convert DTOs to domain entities
    let building_org_id = Uuid::parse_str(&building_dto.organization_id)
        .unwrap_or(organization_id);

    let building_created_at = DateTime::parse_from_rfc3339(&building_dto.created_at)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now());

    let building_updated_at = DateTime::parse_from_rfc3339(&building_dto.updated_at)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now());

    let building_entity = Building {
        id: Uuid::parse_str(&building_dto.id).unwrap_or(expense_building_id),
        name: building_dto.name.clone(),
        address: building_dto.address,
        city: building_dto.city,
        postal_code: building_dto.postal_code,
        country: building_dto.country,
        total_units: building_dto.total_units,
        total_tantiemes: building_dto.total_tantiemes,
        construction_year: building_dto.construction_year,
        syndic_name: None,
        syndic_email: None,
        syndic_phone: None,
        syndic_address: None,
        syndic_office_hours: None,
        syndic_emergency_contact: None,
        slug: None,
        organization_id: building_org_id,
        created_at: building_created_at,
        updated_at: building_updated_at,
    };

    let expense_date = DateTime::parse_from_rfc3339(&expense_dto.expense_date)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now());

    use crate::domain::entities::ApprovalStatus;
    let expense_entity = Expense {
        id: expense_id_uuid,
        organization_id,
        building_id: expense_building_id,
        category: expense_dto.category.clone(),
        description: expense_dto.description.clone(),
        amount: expense_dto.amount,
        amount_excl_vat: None,
        vat_rate: None,
        vat_amount: None,
        amount_incl_vat: None,
        expense_date,
        invoice_date: None,
        due_date: None,
        paid_date: None,
        approval_status: expense_dto.approval_status.clone(),
        submitted_at: None,
        approved_by: None,
        approved_at: None,
        rejection_reason: None,
        payment_status: expense_dto.payment_status.clone(),
        supplier: expense_dto.supplier.clone(),
        invoice_number: expense_dto.invoice_number.clone(),
        account_code: expense_dto.account_code.clone(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    // Create a single line item from expense amount (simplified for now)
    // Note: For full invoice support with line items, we would need to:
    // 1. Get the invoice via get_invoice()
    // 2. Add a repository method to fetch line items
    // For now, we use simplified single line item approach
    let quote_line_items: Vec<QuoteLineItem> = vec![QuoteLineItem {
        description: expense_dto.description.clone(),
        quantity: 1.0,
        unit_price: expense_dto.amount,
        total: expense_dto.amount,
    }];

    // 4. Generate PDF
    match WorkQuoteExporter::export_to_pdf(
        &building_entity,
        &expense_entity,
        &quote_line_items,
        &query.contractor_name,
        &query.contractor_contact,
        &query.timeline,
    ) {
        Ok(pdf_bytes) => {
            // Audit log
            AuditLogEntry::new(
                AuditEventType::ReportGenerated,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_resource("Expense", expense_id)
            .with_metadata(serde_json::json!({
                "report_type": "work_quote_pdf",
                "building_name": building_entity.name,
                "contractor_name": query.contractor_name,
                "amount": expense_dto.amount
            }))
            .log();

            HttpResponse::Ok()
                .content_type("application/pdf")
                .insert_header((
                    "Content-Disposition",
                    format!(
                        "attachment; filename=\"Devis_Travaux_{}_{}.pdf\"",
                        building_entity.name.replace(' ', "_"),
                        expense_entity.id
                    ),
                ))
                .body(pdf_bytes)
        }
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to generate PDF: {}", err)
        })),
    }
}
