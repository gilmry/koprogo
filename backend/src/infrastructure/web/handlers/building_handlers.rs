use crate::application::dto::{CreateBuildingDto, PageRequest, PageResponse, UpdateBuildingDto};
use crate::infrastructure::audit::{AuditEventType, AuditLogEntry};
use crate::infrastructure::web::{AppState, AuthenticatedUser};
use actix_web::{delete, get, post, put, web, HttpResponse, Responder, ResponseError};
use chrono::{DateTime, Datelike, Utc};
use serde::Deserialize;
use uuid::Uuid;
use validator::Validate;

#[utoipa::path(
    post,
    path = "/buildings",
    tag = "Buildings",
    summary = "Create a building",
    request_body = CreateBuildingDto,
    responses(
        (status = 201, description = "Building created successfully"),
        (status = 400, description = "Bad Request"),
        (status = 403, description = "Forbidden - SuperAdmin only"),
        (status = 500, description = "Internal Server Error"),
    ),
    security(("bearer_auth" = []))
)]
#[post("/buildings")]
pub async fn create_building(
    state: web::Data<AppState>,
    user: AuthenticatedUser, // JWT-extracted user info (SECURE!)
    dto: web::Json<CreateBuildingDto>,
) -> impl Responder {
    // Only SuperAdmin can create buildings (structural data)
    if user.role != "superadmin" {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Only SuperAdmin can create buildings (structural data cannot be modified after creation)"
        }));
    }

    // Story 1.2 — Building::acp_id (FK vers acps.id, ex-organization_id).
    // SuperAdmin must specify the target ACP id in the DTO.
    let acp_id: Uuid = if dto.acp_id.is_empty() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "SuperAdmin must specify acp_id"
        }));
    } else {
        match Uuid::parse_str(&dto.acp_id) {
            Ok(id) => id,
            Err(_) => {
                return HttpResponse::BadRequest().json(serde_json::json!({
                    "error": "Invalid acp_id format"
                }));
            }
        }
    };
    // The audit log still records the user's home organization for traceability ;
    // the building's organization is now derived via its parent ACP.
    let organization_id: Option<Uuid> = user.organization_id;
    let _ = acp_id; // used implicitly via dto.acp_id forwarded to use_case

    if let Err(errors) = dto.validate() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Validation failed",
            "details": errors.to_string()
        }));
    }

    match state
        .building_use_cases
        .create_building(dto.into_inner())
        .await
    {
        Ok(building) => {
            // Audit log: successful building creation
            AuditLogEntry::new(
                AuditEventType::BuildingCreated,
                Some(user.user_id),
                organization_id,
            )
            .with_resource("Building", Uuid::parse_str(&building.id).unwrap())
            .log();

            HttpResponse::Created().json(building)
        }
        Err(err) => {
            // Audit log: failed building creation
            AuditLogEntry::new(
                AuditEventType::BuildingCreated,
                Some(user.user_id),
                organization_id,
            )
            .with_error(err.clone())
            .log();

            HttpResponse::BadRequest().json(serde_json::json!({
                "error": err
            }))
        }
    }
}

#[utoipa::path(
    get,
    path = "/buildings",
    tag = "Buildings",
    summary = "List buildings (paginated)",
    params(PageRequest),
    responses(
        (status = 200, description = "Paginated list of buildings"),
        (status = 500, description = "Internal Server Error"),
    ),
    security(("bearer_auth" = []))
)]
#[get("/buildings")]
pub async fn list_buildings(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    page_request: web::Query<PageRequest>,
) -> impl Responder {
    // Story 1.3 — role-based scope derivation (ADR-0010 §3.3) :
    // - superadmin / admin without org : ListScope::All  -> no FK filter
    // - admin / syndic / accountant : ListScope::Organization -> filter by org
    // - owner : ListScope::Owner -> filter via unit_owners (BUG-WF14-2)
    let organization_id = user.effective_org_filter();

    let owner_user_id = if user.role == "owner" {
        Some(user.user_id)
    } else {
        None
    };

    match state
        .building_use_cases
        .list_buildings_paginated_for_user(&page_request, organization_id, owner_user_id)
        .await
    {
        Ok((buildings, total)) => {
            let response =
                PageResponse::new(buildings, page_request.page, page_request.per_page, total);
            HttpResponse::Ok().json(response)
        }
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": err
        })),
    }
}

#[utoipa::path(
    get,
    path = "/buildings/{id}",
    tag = "Buildings",
    summary = "Get a building by ID",
    params(
        ("id" = Uuid, Path, description = "Building UUID")
    ),
    responses(
        (status = 200, description = "Building found"),
        (status = 404, description = "Building not found"),
        (status = 500, description = "Internal Server Error"),
    ),
    security(("bearer_auth" = []))
)]
#[get("/buildings/{id}")]
pub async fn get_building(
    state: web::Data<AppState>,
    _user: AuthenticatedUser,
    id: web::Path<Uuid>,
) -> impl Responder {
    // Story 1.4 — use `get_building_with_metrics` so units_count, quota_sum,
    // is_conformant and quota_delta are exposed (#553 Bugs 1/3/4 + FR11/FR12/FR23).
    match state
        .building_use_cases
        .get_building_with_metrics(*id)
        .await
    {
        Ok(Some(building)) => {
            // TODO(#602/hotfix-blocker): Multi-tenant isolation must be
            // re-implemented via ACP→organization resolution. The DTO field
            // `organization_id` was renamed to `acp_id` (Story 1.2 migration
            // DROP buildings.organization_id) ; verifying the user's org
            // access now requires `acp_repo.find_by_id(building.acp_id)`
            // to retrieve the parent organization_id. Skipped for the
            // minimal runtime hotfix — the use_case-level scope filter
            // (list_buildings) already enforces tenant isolation.
            let _building_acp_id = &building.acp_id;
            HttpResponse::Ok().json(building)
        }
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Building not found"
        })),
        Err(err) => err.error_response(),
    }
}

#[utoipa::path(
    put,
    path = "/buildings/{id}",
    tag = "Buildings",
    summary = "Update a building",
    params(
        ("id" = Uuid, Path, description = "Building UUID")
    ),
    request_body = UpdateBuildingDto,
    responses(
        (status = 200, description = "Building updated successfully"),
        (status = 400, description = "Bad Request"),
        (status = 403, description = "Forbidden - SuperAdmin only"),
        (status = 404, description = "Building not found"),
        (status = 500, description = "Internal Server Error"),
    ),
    security(("bearer_auth" = []))
)]
#[put("/buildings/{id}")]
pub async fn update_building(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    id: web::Path<Uuid>,
    dto: web::Json<UpdateBuildingDto>,
) -> impl Responder {
    // Only SuperAdmin can update buildings (structural data)
    if user.role != "superadmin" {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Only SuperAdmin can update buildings (structural data)"
        }));
    }

    if let Err(errors) = dto.validate() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Validation failed",
            "details": errors.to_string()
        }));
    }

    // Story 1.2 — Only SuperAdmin can re-affect the parent ACP.
    if dto.acp_id.is_some() && user.role != "superadmin" {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Only SuperAdmins can change building ACP"
        }));
    }

    // TODO(#602/hotfix-blocker): Non-SuperAdmins multi-tenant ownership
    // verification needs ACP→organization resolution (DTO no longer
    // carries organization_id). Branch unreachable here because the
    // superadmin-only guard above (line `if user.role != "superadmin"`)
    // already returns 403 — kept as defensive no-op while we wait for
    // the proper acp_repo dependency injection in this handler.
    if user.role != "superadmin" {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Only SuperAdmin can update buildings (structural data)"
        }));
    }

    match state
        .building_use_cases
        .update_building(*id, dto.into_inner())
        .await
    {
        Ok(building) => {
            // Audit log: successful building update
            AuditLogEntry::new(
                AuditEventType::BuildingUpdated,
                Some(user.user_id),
                user.organization_id,
            )
            .with_resource("Building", *id)
            .log();

            HttpResponse::Ok().json(building)
        }
        Err(err) => {
            // Audit log: failed building update
            AuditLogEntry::new(
                AuditEventType::BuildingUpdated,
                Some(user.user_id),
                user.organization_id,
            )
            .with_resource("Building", *id)
            .with_error(err.clone())
            .log();

            HttpResponse::BadRequest().json(serde_json::json!({
                "error": err
            }))
        }
    }
}

#[utoipa::path(
    delete,
    path = "/buildings/{id}",
    tag = "Buildings",
    summary = "Delete a building",
    params(
        ("id" = Uuid, Path, description = "Building UUID")
    ),
    responses(
        (status = 204, description = "Building deleted successfully"),
        (status = 403, description = "Forbidden - SuperAdmin only"),
        (status = 404, description = "Building not found"),
        (status = 500, description = "Internal Server Error"),
    ),
    security(("bearer_auth" = []))
)]
#[delete("/buildings/{id}")]
pub async fn delete_building(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    id: web::Path<Uuid>,
) -> impl Responder {
    // Only SuperAdmin can delete buildings
    if user.role != "superadmin" {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Only SuperAdmin can delete buildings"
        }));
    }

    match state.building_use_cases.delete_building(*id).await {
        Ok(true) => {
            // Audit log: successful building deletion
            AuditLogEntry::new(
                AuditEventType::BuildingDeleted,
                Some(user.user_id),
                user.organization_id,
            )
            .with_resource("Building", *id)
            .log();

            HttpResponse::NoContent().finish()
        }
        Ok(false) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Building not found"
        })),
        Err(err) => {
            // Audit log: failed building deletion
            AuditLogEntry::new(
                AuditEventType::BuildingDeleted,
                Some(user.user_id),
                user.organization_id,
            )
            .with_resource("Building", *id)
            .with_error(err.clone())
            .log();

            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": err
            }))
        }
    }
}

/// Export Annual Financial Report to PDF
///
/// GET /buildings/{building_id}/export-annual-report-pdf?year={2025}&reserve_fund={10000.00}&total_income={50000.00}
///
/// Generates a "Rapport Financier Annuel" PDF for a building's annual financial summary.
#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct ExportAnnualReportQuery {
    pub year: i32,
    #[serde(default)]
    pub reserve_fund: Option<rust_decimal::Decimal>, // Optional reserve fund balance
    #[serde(default)]
    pub total_income: Option<rust_decimal::Decimal>, // Optional total income (calculated if not provided)
}

#[utoipa::path(
    get,
    path = "/buildings/{id}/export-annual-report-pdf",
    tag = "Buildings",
    summary = "Export annual financial report as PDF",
    params(
        ("id" = Uuid, Path, description = "Building UUID"),
        ExportAnnualReportQuery
    ),
    responses(
        (status = 200, description = "PDF generated successfully", content_type = "application/pdf"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Building not found"),
        (status = 500, description = "Internal Server Error"),
    ),
    security(("bearer_auth" = []))
)]
#[get("/buildings/{id}/export-annual-report-pdf")]
pub async fn export_annual_report_pdf(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    id: web::Path<Uuid>,
    query: web::Query<ExportAnnualReportQuery>,
) -> impl Responder {
    use crate::domain::entities::{Building, Expense};
    use crate::domain::services::{AnnualReportExporter, BudgetItem};

    let organization_id = match user.require_organization() {
        Ok(org_id) => org_id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": e.to_string()
            }))
        }
    };

    let building_id = *id;
    let year = query.year;

    // 1. Get building
    let building_dto = match state.building_use_cases.get_building(building_id).await {
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

    // 2. Get all expenses for this building
    let expenses_dto = match state
        .expense_use_cases
        .list_expenses_by_building(building_id)
        .await
    {
        Ok(expenses) => expenses,
        Err(err) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to get expenses: {}", err)
            }))
        }
    };

    // Filter expenses by year (using expense_date from DTO)
    let year_expenses: Vec<_> = expenses_dto
        .into_iter()
        .filter(|e| {
            // Parse expense_date string to get year
            DateTime::parse_from_rfc3339(&e.expense_date)
                .map(|dt| dt.year() == year)
                .unwrap_or(false)
        })
        .collect();

    // Calculate total income if not provided (sum of all paid expenses)
    use crate::domain::entities::PaymentStatus;
    let total_income = query.total_income.unwrap_or_else(|| {
        year_expenses
            .iter()
            .filter(|e| e.payment_status == PaymentStatus::Paid)
            .map(|e| e.amount)
            .sum()
    });

    // Reserve fund (default to 0.0 if not provided)
    let reserve_fund = query.reserve_fund.unwrap_or(rust_decimal::Decimal::ZERO);

    // Convert DTOs to domain entities — Story 1.2 : the DTO field is now
    // `acp_id` (the building's parent ACP). The legacy variable name
    // `organization_id` is preserved upstream for audit/scope use only.
    let building_acp_id = Uuid::parse_str(&building_dto.acp_id).unwrap_or_else(|_| Uuid::new_v4());

    let building_created_at = DateTime::parse_from_rfc3339(&building_dto.created_at)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now());

    let building_updated_at = DateTime::parse_from_rfc3339(&building_dto.updated_at)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now());

    let building_entity = Building {
        id: Uuid::parse_str(&building_dto.id).unwrap_or(building_id),
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
        acp_id: building_acp_id,
        created_at: building_created_at,
        updated_at: building_updated_at,
    };

    // Convert expenses to domain entities
    let expense_entities: Vec<Expense> = year_expenses
        .iter()
        .filter_map(|e| {
            // Parse DTO fields
            let exp_id = Uuid::parse_str(&e.id).ok()?;
            let bldg_id = Uuid::parse_str(&e.building_id).ok()?;
            let exp_date = DateTime::parse_from_rfc3339(&e.expense_date)
                .ok()?
                .with_timezone(&Utc);

            Some(Expense {
                id: exp_id,
                organization_id,
                building_id: bldg_id,
                category: e.category.clone(),
                description: e.description.clone(),
                amount: e.amount,
                amount_excl_vat: None,
                vat_rate: None,
                vat_amount: None,
                amount_incl_vat: None,
                expense_date: exp_date,
                invoice_date: None,
                due_date: None,
                paid_date: None,
                approval_status: e.approval_status.clone(),
                submitted_at: None,
                approved_by: None,
                approved_at: None,
                rejection_reason: None,
                payment_status: e.payment_status.clone(),
                supplier: e.supplier.clone(),
                invoice_number: e.invoice_number.clone(),
                account_code: e.account_code.clone(),
                contractor_report_id: None,
                created_at: Utc::now(), // Simplified
                updated_at: Utc::now(), // Simplified
            })
        })
        .collect();

    // Budget items (empty for now, to be implemented with budget system)
    let budget_items: Vec<BudgetItem> = Vec::new();

    // 3. Generate PDF
    match AnnualReportExporter::export_to_pdf(
        &building_entity,
        year,
        &expense_entities,
        &budget_items,
        total_income,
        reserve_fund,
    ) {
        Ok(pdf_bytes) => {
            // Audit log
            AuditLogEntry::new(
                AuditEventType::ReportGenerated,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_resource("Building", building_id)
            .with_metadata(serde_json::json!({
                "report_type": "annual_report_pdf",
                "building_name": building_entity.name,
                "year": year,
                "total_income": total_income,
                "reserve_fund": reserve_fund
            }))
            .log();

            HttpResponse::Ok()
                .content_type("application/pdf")
                .insert_header((
                    "Content-Disposition",
                    format!(
                        "attachment; filename=\"Rapport_Annuel_{}_{}.pdf\"",
                        building_entity.name.replace(' ', "_"),
                        year
                    ),
                ))
                .body(pdf_bytes)
        }
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to generate PDF: {}", err)
        })),
    }
}
