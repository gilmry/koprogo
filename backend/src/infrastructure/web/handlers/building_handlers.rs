use crate::application::dto::{CreateBuildingDto, PageRequest, PageResponse, UpdateBuildingDto};
use crate::infrastructure::audit::{AuditEventType, AuditLogEntry};
use crate::infrastructure::web::{AppState, AuthenticatedUser};
use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use chrono::{DateTime, Datelike, Utc};
use serde::Deserialize;
use uuid::Uuid;
use validator::Validate;

#[post("/buildings")]
pub async fn create_building(
    state: web::Data<AppState>,
    user: AuthenticatedUser, // JWT-extracted user info (SECURE!)
    mut dto: web::Json<CreateBuildingDto>,
) -> impl Responder {
    // Only SuperAdmin can create buildings (structural data)
    if user.role != "superadmin" {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Only SuperAdmin can create buildings (structural data cannot be modified after creation)"
        }));
    }

    // SuperAdmin can create buildings for any organization
    // Regular users can only create for their own organization
    let organization_id: Uuid;

    if user.role == "superadmin" {
        // SuperAdmin: organization_id must be provided in DTO
        if dto.organization_id.is_empty() {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "SuperAdmin must specify organization_id"
            }));
        }
        // Parse the organization_id from DTO
        organization_id = match Uuid::parse_str(&dto.organization_id) {
            Ok(id) => id,
            Err(_) => {
                return HttpResponse::BadRequest().json(serde_json::json!({
                    "error": "Invalid organization_id format"
                }));
            }
        };
    } else {
        // Regular user: override organization_id from JWT token
        // This prevents users from creating buildings in other organizations
        organization_id = match user.require_organization() {
            Ok(org_id) => org_id,
            Err(e) => {
                return HttpResponse::Unauthorized().json(serde_json::json!({
                    "error": e.to_string()
                }))
            }
        };
        dto.organization_id = organization_id.to_string();
    }

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
                Some(organization_id),
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

#[get("/buildings")]
pub async fn list_buildings(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    page_request: web::Query<PageRequest>,
) -> impl Responder {
    // Extract organization_id from authenticated user (secure!)
    let organization_id = user.organization_id;

    match state
        .building_use_cases
        .list_buildings_paginated(&page_request, organization_id)
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

#[get("/buildings/{id}")]
pub async fn get_building(state: web::Data<AppState>, id: web::Path<Uuid>) -> impl Responder {
    match state.building_use_cases.get_building(*id).await {
        Ok(Some(building)) => HttpResponse::Ok().json(building),
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Building not found"
        })),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": err
        })),
    }
}

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

    // Only SuperAdmin can change organization_id
    if dto.organization_id.is_some() && user.role != "superadmin" {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Only SuperAdmins can change building organization"
        }));
    }

    // For non-SuperAdmins, verify they own the building
    if user.role != "superadmin" {
        match state.building_use_cases.get_building(*id).await {
            Ok(Some(building)) => {
                let building_org_id = match Uuid::parse_str(&building.organization_id) {
                    Ok(id) => id,
                    Err(_) => {
                        return HttpResponse::InternalServerError().json(serde_json::json!({
                            "error": "Invalid building organization_id"
                        }));
                    }
                };

                let user_org_id = match user.require_organization() {
                    Ok(id) => id,
                    Err(e) => {
                        return HttpResponse::Unauthorized().json(serde_json::json!({
                            "error": e.to_string()
                        }));
                    }
                };

                if building_org_id != user_org_id {
                    return HttpResponse::Forbidden().json(serde_json::json!({
                        "error": "You can only update buildings in your own organization"
                    }));
                }
            }
            Ok(None) => {
                return HttpResponse::NotFound().json(serde_json::json!({
                    "error": "Building not found"
                }));
            }
            Err(err) => {
                return HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": err
                }));
            }
        }
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
#[derive(Debug, Deserialize)]
pub struct ExportAnnualReportQuery {
    pub year: i32,
    #[serde(default)]
    pub reserve_fund: Option<f64>, // Optional reserve fund balance
    #[serde(default)]
    pub total_income: Option<f64>, // Optional total income (calculated if not provided)
}

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
    let reserve_fund = query.reserve_fund.unwrap_or(0.0);

    // Convert DTOs to domain entities
    let building_org_id = Uuid::parse_str(&building_dto.organization_id).unwrap_or(organization_id);

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
        organization_id: building_org_id,
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
