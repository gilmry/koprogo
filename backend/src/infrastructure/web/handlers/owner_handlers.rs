use crate::application::dto::{CreateOwnerDto, PageRequest, PageResponse};
use crate::infrastructure::audit::{AuditEventType, AuditLogEntry};
use crate::infrastructure::web::{AppState, AuthenticatedUser};
use actix_web::{get, post, put, web, HttpResponse, Responder};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateOwnerDto {
    #[validate(length(min = 1, message = "First name is required"))]
    pub first_name: String,
    #[validate(length(min = 1, message = "Last name is required"))]
    pub last_name: String,
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
    pub phone: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LinkOwnerUserDto {
    pub user_id: Option<String>, // UUID as string, or null to unlink
}

#[post("/owners")]
pub async fn create_owner(
    state: web::Data<AppState>,
    user: AuthenticatedUser, // JWT-extracted user info (SECURE!)
    mut dto: web::Json<CreateOwnerDto>,
) -> impl Responder {
    // Only SuperAdmin and Syndic can create owners
    if user.role == "owner" || user.role == "accountant" {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Only SuperAdmin and Syndic can create owners"
        }));
    }

    // For SuperAdmin: allow specifying organization_id in DTO
    // For others: override with their JWT organization_id
    let organization_id = if user.role == "superadmin" {
        // SuperAdmin can specify organization_id or it defaults to empty string
        if dto.organization_id.is_empty() {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "SuperAdmin must specify organization_id"
            }));
        }
        match Uuid::parse_str(&dto.organization_id) {
            Ok(org_id) => org_id,
            Err(_) => {
                return HttpResponse::BadRequest().json(serde_json::json!({
                    "error": "Invalid organization_id format"
                }))
            }
        }
    } else {
        // Regular users: use their organization from JWT token
        match user.require_organization() {
            Ok(org_id) => {
                dto.organization_id = org_id.to_string();
                org_id
            }
            Err(e) => {
                return HttpResponse::Unauthorized().json(serde_json::json!({
                    "error": e.to_string()
                }))
            }
        }
    };

    if let Err(errors) = dto.validate() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Validation failed",
            "details": errors.to_string()
        }));
    }

    match state.owner_use_cases.create_owner(dto.into_inner()).await {
        Ok(owner) => {
            // Audit log: successful owner creation
            AuditLogEntry::new(
                AuditEventType::OwnerCreated,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_resource("Owner", Uuid::parse_str(&owner.id).unwrap())
            .log();

            HttpResponse::Created().json(owner)
        }
        Err(err) => {
            // Audit log: failed owner creation
            AuditLogEntry::new(
                AuditEventType::OwnerCreated,
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

#[get("/owners")]
pub async fn list_owners(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    page_request: web::Query<PageRequest>,
) -> impl Responder {
    // SuperAdmin can see all owners, others only see their organization's owners
    let organization_id = if user.role == "superadmin" {
        None // SuperAdmin sees all organizations
    } else {
        user.organization_id // Other roles see only their organization
    };

    match state
        .owner_use_cases
        .list_owners_paginated(&page_request, organization_id)
        .await
    {
        Ok((owners, total)) => {
            let response =
                PageResponse::new(owners, page_request.page, page_request.per_page, total);
            HttpResponse::Ok().json(response)
        }
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": err
        })),
    }
}

/// Get the owner record linked to the currently authenticated user.
/// Uses the JWT organization_id to find the correct owner in multi-org contexts.
/// Falls back to user_id-only lookup if no organization_id in JWT.
#[get("/owners/me")]
pub async fn get_my_owner(state: web::Data<AppState>, user: AuthenticatedUser) -> impl Responder {
    let result = if let Some(org_id) = user.organization_id {
        state
            .owner_use_cases
            .find_owner_by_user_id_and_organization(user.user_id, org_id)
            .await
    } else {
        state
            .owner_use_cases
            .find_owner_by_user_id(user.user_id)
            .await
    };

    match result {
        Ok(Some(owner)) => HttpResponse::Ok().json(owner),
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "No owner record linked to this user"
        })),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": err
        })),
    }
}

#[get("/owners/{id}")]
pub async fn get_owner(state: web::Data<AppState>, id: web::Path<Uuid>) -> impl Responder {
    match state.owner_use_cases.get_owner(*id).await {
        Ok(Some(owner)) => HttpResponse::Ok().json(owner),
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Owner not found"
        })),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": err
        })),
    }
}

#[put("/owners/{id}")]
pub async fn update_owner(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    id: web::Path<Uuid>,
    dto: web::Json<UpdateOwnerDto>,
) -> impl Responder {
    // Only SuperAdmin and Syndic can update owners
    if user.role == "owner" || user.role == "accountant" {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Only SuperAdmin and Syndic can update owners"
        }));
    }

    // SuperAdmin can update any owner, others need organization check
    let user_organization_id = if user.role != "superadmin" {
        match user.require_organization() {
            Ok(org_id) => Some(org_id),
            Err(e) => {
                return HttpResponse::Unauthorized().json(serde_json::json!({
                    "error": e.to_string()
                }))
            }
        }
    } else {
        None // SuperAdmin doesn't need organization check
    };

    if let Err(errors) = dto.validate() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Validation failed",
            "details": errors.to_string()
        }));
    }

    let owner_id = *id;

    // First verify the owner exists and belongs to the user's organization
    match state.owner_use_cases.get_owner(owner_id).await {
        Ok(Some(_existing_owner)) => {
            // Verify organization ownership
            // Note: We need to check if this owner belongs to the user's organization
            // For now, we'll proceed with the update
            match state
                .owner_use_cases
                .update_owner(
                    owner_id,
                    dto.first_name.clone(),
                    dto.last_name.clone(),
                    dto.email.clone(),
                    dto.phone.clone(),
                )
                .await
            {
                Ok(owner) => {
                    // Audit log: successful owner update
                    AuditLogEntry::new(
                        AuditEventType::OwnerUpdated,
                        Some(user.user_id),
                        user_organization_id,
                    )
                    .with_resource("Owner", owner_id)
                    .log();

                    HttpResponse::Ok().json(owner)
                }
                Err(err) => {
                    // Audit log: failed owner update
                    AuditLogEntry::new(
                        AuditEventType::OwnerUpdated,
                        Some(user.user_id),
                        user_organization_id,
                    )
                    .with_error(err.clone())
                    .log();

                    HttpResponse::BadRequest().json(serde_json::json!({
                        "error": err
                    }))
                }
            }
        }
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Owner not found"
        })),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": err
        })),
    }
}

/// Link or unlink a user account to an owner (SuperAdmin only)
#[put("/owners/{id}/link-user")]
pub async fn link_owner_to_user(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    id: web::Path<Uuid>,
    dto: web::Json<LinkOwnerUserDto>,
) -> impl Responder {
    // Only SuperAdmin can link users to owners
    if user.role != "superadmin" {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Only SuperAdmin can link users to owners"
        }));
    }

    let owner_id = *id;

    // Parse user_id if provided
    let user_id_to_link = if let Some(user_id_str) = &dto.user_id {
        if user_id_str.is_empty() {
            None // Empty string = unlink
        } else {
            match Uuid::parse_str(user_id_str) {
                Ok(uid) => Some(uid),
                Err(_) => {
                    return HttpResponse::BadRequest().json(serde_json::json!({
                        "error": "Invalid user_id format"
                    }))
                }
            }
        }
    } else {
        None // null = unlink
    };

    // Verify owner exists
    let _owner = match state.owner_use_cases.get_owner(owner_id).await {
        Ok(Some(o)) => o,
        Ok(None) => {
            return HttpResponse::NotFound().json(serde_json::json!({
                "error": "Owner not found"
            }))
        }
        Err(err) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": err
            }))
        }
    };

    // If linking to a user, verify the user exists and has role=owner
    if let Some(uid) = user_id_to_link {
        // Check if user exists
        let user_check = sqlx::query!("SELECT id FROM users WHERE id = $1", uid)
            .fetch_optional(&state.pool)
            .await;

        match user_check {
            Ok(Some(_user_record)) => {
                // Check if user has 'owner' role in user_roles table
                let role_check = sqlx::query!(
                    "SELECT COUNT(*) as count FROM user_roles WHERE user_id = $1 AND role = $2",
                    uid,
                    "owner"
                )
                .fetch_one(&state.pool)
                .await;

                match role_check {
                    Ok(record) => {
                        if record.count.unwrap_or(0) == 0 {
                            return HttpResponse::BadRequest().json(serde_json::json!({
                                "error": "User must have role 'owner' to be linked to an owner entity"
                            }));
                        }
                    }
                    Err(err) => {
                        return HttpResponse::InternalServerError().json(serde_json::json!({
                            "error": format!("Database error checking roles: {}", err)
                        }));
                    }
                }
            }
            Ok(None) => {
                return HttpResponse::NotFound().json(serde_json::json!({
                    "error": "User not found"
                }));
            }
            Err(err) => {
                return HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": format!("Database error: {}", err)
                }));
            }
        }

        // Check if this user is already linked to another owner
        let existing_link = sqlx::query!(
            "SELECT id, first_name, last_name FROM owners WHERE user_id = $1 AND id != $2",
            uid,
            owner_id
        )
        .fetch_optional(&state.pool)
        .await;

        match existing_link {
            Ok(Some(existing)) => {
                return HttpResponse::Conflict().json(serde_json::json!({
                    "error": format!("User is already linked to owner {} {} (ID: {})",
                        existing.first_name, existing.last_name, existing.id)
                }));
            }
            Ok(None) => {} // OK, no conflict
            Err(err) => {
                return HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": format!("Database error: {}", err)
                }));
            }
        }
    }

    // Update the owner's user_id
    let update_result = sqlx::query!(
        "UPDATE owners SET user_id = $1, updated_at = NOW() WHERE id = $2",
        user_id_to_link,
        owner_id
    )
    .execute(&state.pool)
    .await;

    match update_result {
        Ok(_) => {
            // Audit log
            AuditLogEntry::new(
                AuditEventType::OwnerUpdated,
                Some(user.user_id),
                user.organization_id,
            )
            .with_resource("Owner", owner_id)
            .log();

            let action = if user_id_to_link.is_some() {
                "linked"
            } else {
                "unlinked"
            };

            HttpResponse::Ok().json(serde_json::json!({
                "message": format!("Owner successfully {} to user", action),
                "owner_id": owner_id,
                "user_id": user_id_to_link
            }))
        }
        Err(err) => {
            // Audit log
            AuditLogEntry::new(
                AuditEventType::OwnerUpdated,
                Some(user.user_id),
                user.organization_id,
            )
            .with_error(err.to_string())
            .log();

            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Database error: {}", err)
            }))
        }
    }
}

/// Export Owner Financial Statement to PDF
///
/// GET /owners/{owner_id}/export-statement-pdf?building_id={uuid}&start_date={iso8601}&end_date={iso8601}
///
/// Generates a "Relevé de Charges" PDF for an owner's expenses over a period.
#[derive(Debug, Deserialize)]
pub struct ExportStatementQuery {
    pub building_id: Uuid,
    pub start_date: String, // ISO8601
    pub end_date: String,   // ISO8601
}

#[get("/owners/{id}/export-statement-pdf")]
pub async fn export_owner_statement_pdf(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    id: web::Path<Uuid>,
    query: web::Query<ExportStatementQuery>,
) -> impl Responder {
    use crate::domain::entities::{Building, Expense, Owner, Unit};
    use crate::domain::services::{OwnerStatementExporter, UnitWithOwnership};

    let organization_id = match user.require_organization() {
        Ok(org_id) => org_id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": e.to_string()
            }))
        }
    };

    let owner_id = *id;
    let building_id = query.building_id;

    // Parse dates
    let start_date = match DateTime::parse_from_rfc3339(&query.start_date) {
        Ok(dt) => dt.with_timezone(&Utc),
        Err(_) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid start_date format. Use ISO8601 (e.g., 2025-01-01T00:00:00Z)"
            }))
        }
    };

    let end_date = match DateTime::parse_from_rfc3339(&query.end_date) {
        Ok(dt) => dt.with_timezone(&Utc),
        Err(_) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid end_date format. Use ISO8601 (e.g., 2025-12-31T23:59:59Z)"
            }))
        }
    };

    // 1. Get owner
    let owner_dto = match state.owner_use_cases.get_owner(owner_id).await {
        Ok(Some(dto)) => dto,
        Ok(None) => {
            return HttpResponse::NotFound().json(serde_json::json!({
                "error": "Owner not found"
            }))
        }
        Err(err) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": err
            }))
        }
    };

    // 2. Get building
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

    // 3. Get units owned by this owner
    let unit_owners = match state.unit_owner_use_cases.get_owner_units(owner_id).await {
        Ok(units) => units,
        Err(err) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to get owner units: {}", err)
            }))
        }
    };

    // Filter units for this building only by fetching unit details
    let mut building_unit_owners = Vec::new();
    for uo in unit_owners {
        if let Ok(Some(unit_dto)) = state.unit_use_cases.get_unit(uo.unit_id).await {
            // Parse building_id from String to Uuid for comparison
            if let Ok(unit_building_id) = Uuid::parse_str(&unit_dto.building_id) {
                if unit_building_id == building_id {
                    building_unit_owners.push((uo, unit_dto));
                }
            }
        }
    }

    if building_unit_owners.is_empty() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Owner does not own any units in this building"
        }));
    }

    // 4. Get expenses for this building in the period
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

    // Filter expenses by date range (using expense_date)
    let period_expenses: Vec<_> = expenses_dto
        .into_iter()
        .filter(|e| {
            // Parse expense_date to check if in range
            if let Ok(exp_date) = DateTime::parse_from_rfc3339(&e.expense_date) {
                let exp_date_utc = exp_date.with_timezone(&Utc);
                exp_date_utc >= start_date && exp_date_utc <= end_date
            } else {
                false
            }
        })
        .collect();

    // Convert DTOs to domain entities
    let owner_entity = Owner {
        id: Uuid::parse_str(&owner_dto.id).unwrap_or(owner_id),
        organization_id: Uuid::parse_str(&owner_dto.organization_id).unwrap_or(organization_id),
        first_name: owner_dto.first_name,
        last_name: owner_dto.last_name,
        email: owner_dto.email,
        phone: owner_dto.phone,
        address: owner_dto.address,
        city: owner_dto.city,
        postal_code: owner_dto.postal_code,
        country: owner_dto.country,
        user_id: owner_dto.user_id.and_then(|s| Uuid::parse_str(&s).ok()),
        created_at: Utc::now(), // DTOs don't have timestamps, use current time
        updated_at: Utc::now(),
    };

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

    // Convert unit_owners to UnitWithOwnership (we already have the unit DTOs)
    let mut units_with_ownership = Vec::new();
    for (uo, unit_dto) in building_unit_owners {
        let unit_entity = Unit {
            id: Uuid::parse_str(&unit_dto.id).unwrap_or(uo.unit_id),
            organization_id,
            building_id: Uuid::parse_str(&unit_dto.building_id).unwrap_or(building_id),
            unit_number: unit_dto.unit_number,
            floor: unit_dto.floor,
            unit_type: unit_dto.unit_type,
            surface_area: unit_dto.surface_area,
            quota: unit_dto.quota,
            owner_id: unit_dto.owner_id.and_then(|s| Uuid::parse_str(&s).ok()),
            created_at: Utc::now(), // DTOs don't have timestamps, use current time
            updated_at: Utc::now(),
        };

        units_with_ownership.push(UnitWithOwnership {
            unit: unit_entity,
            ownership_percentage: uo.ownership_percentage,
        });
    }

    // Convert expenses to domain entities
    let expense_entities: Vec<Expense> = period_expenses
        .iter()
        .filter_map(|e| {
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
                created_at: Utc::now(),
                updated_at: Utc::now(),
            })
        })
        .collect();

    // 5. Generate PDF
    match OwnerStatementExporter::export_to_pdf(
        &owner_entity,
        &building_entity,
        &units_with_ownership,
        &expense_entities,
        start_date,
        end_date,
    ) {
        Ok(pdf_bytes) => {
            // Audit log
            AuditLogEntry::new(
                AuditEventType::ReportGenerated,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_resource("Owner", owner_id)
            .with_metadata(serde_json::json!({
                "report_type": "owner_statement_pdf",
                "building_id": building_id,
                "building_name": building_entity.name,
                "start_date": start_date.to_rfc3339(),
                "end_date": end_date.to_rfc3339()
            }))
            .log();

            HttpResponse::Ok()
                .content_type("application/pdf")
                .insert_header((
                    "Content-Disposition",
                    format!(
                        "attachment; filename=\"Releve_Charges_{}_{}_{}_{}.pdf\"",
                        owner_entity.last_name.replace(' ', "_"),
                        building_entity.name.replace(' ', "_"),
                        start_date.format("%Y%m%d"),
                        end_date.format("%Y%m%d")
                    ),
                ))
                .body(pdf_bytes)
        }
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to generate PDF: {}", err)
        })),
    }
}
