// API Handlers for Account Management
//
// CREDITS: Handler patterns inspired by Noalyss API structure (GPL-2.0+)
// https://gitlab.com/noalyss/noalyss

use crate::application::dto::{
    AccountResponseDto, AccountSearchQuery, CreateAccountDto, SeedBelgianPcmnDto,
    SeedPcmnResponseDto, UpdateAccountDto,
};
use crate::domain::entities::AccountType;
use crate::infrastructure::audit::{AuditEventType, AuditLogEntry};
use crate::infrastructure::web::{AppState, AuthenticatedUser};
use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use uuid::Uuid;
use validator::Validate;

/// Convert Domain Account to DTO
fn account_to_dto(account: &crate::domain::entities::Account) -> AccountResponseDto {
    AccountResponseDto {
        id: account.id.to_string(),
        code: account.code.clone(),
        label: account.label.clone(),
        parent_code: account.parent_code.clone(),
        account_type: format!("{:?}", account.account_type).to_uppercase(),
        direct_use: account.direct_use,
        organization_id: account.organization_id.to_string(),
        created_at: account.created_at.to_rfc3339(),
        updated_at: account.updated_at.to_rfc3339(),
    }
}

/// Parse AccountType from string
fn parse_account_type(type_str: &str) -> Result<AccountType, String> {
    match type_str.to_uppercase().as_str() {
        "ASSET" => Ok(AccountType::Asset),
        "LIABILITY" => Ok(AccountType::Liability),
        "EXPENSE" => Ok(AccountType::Expense),
        "REVENUE" => Ok(AccountType::Revenue),
        "OFF_BALANCE" | "OFFBALANCE" => Ok(AccountType::OffBalance),
        _ => Err(format!("Invalid account type: {}", type_str)),
    }
}

// ============================================================================
// POST /api/v1/accounts - Create a new account
// ============================================================================

#[post("/accounts")]
pub async fn create_account(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    dto: web::Json<CreateAccountDto>,
) -> impl Responder {
    // Permission: Accountant or SuperAdmin can create accounts
    if user.role != "accountant" && user.role != "superadmin" {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Only Accountant or SuperAdmin can create accounts"
        }));
    }

    // Validate DTO
    if let Err(errors) = dto.validate() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Validation failed",
            "details": errors.to_string()
        }));
    }

    // Parse organization_id
    let organization_id = match Uuid::parse_str(&dto.organization_id) {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid organization_id format"
            }));
        }
    };

    // Authorization: Check user belongs to organization (unless SuperAdmin)
    if user.role != "superadmin" {
        if let Ok(user_org_id) = user.require_organization() {
            if user_org_id != organization_id {
                return HttpResponse::Forbidden().json(serde_json::json!({
                    "error": "You can only create accounts for your own organization"
                }));
            }
        } else {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "User has no organization"
            }));
        }
    }

    // Parse account_type
    let account_type = match parse_account_type(&dto.account_type) {
        Ok(at) => at,
        Err(e) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": e
            }));
        }
    };

    // Call use case
    match state
        .account_use_cases
        .create_account(
            dto.code.clone(),
            dto.label.clone(),
            dto.parent_code.clone(),
            account_type,
            dto.direct_use,
            organization_id,
        )
        .await
    {
        Ok(account) => {
            // Audit log
            AuditLogEntry::new(
                AuditEventType::AccountCreated,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_resource("Account", account.id)
            .log();

            HttpResponse::Created().json(account_to_dto(&account))
        }
        Err(err) => {
            // Audit log failure
            AuditLogEntry::new(
                AuditEventType::AccountCreated,
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

// ============================================================================
// GET /api/v1/accounts - List accounts (with optional filters)
// ============================================================================

#[get("/accounts")]
pub async fn list_accounts(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    query: web::Query<AccountSearchQuery>,
) -> impl Responder {
    // Get organization_id (SuperAdmin can query any org, others only their own)
    let organization_id = if user.role == "superadmin" {
        // TODO: SuperAdmin could pass org_id as query param
        // For now, require organization
        match user.require_organization() {
            Ok(id) => id,
            Err(e) => {
                return HttpResponse::Unauthorized().json(serde_json::json!({
                    "error": e.to_string()
                }))
            }
        }
    } else {
        match user.require_organization() {
            Ok(id) => id,
            Err(e) => {
                return HttpResponse::Unauthorized().json(serde_json::json!({
                    "error": e.to_string()
                }))
            }
        }
    };

    // Handle different query scenarios
    let accounts_result = if let Some(ref code_pattern) = query.code_pattern {
        // Search by code pattern
        state
            .account_use_cases
            .search_accounts(code_pattern, organization_id)
            .await
    } else if let Some(ref account_type_str) = query.account_type {
        // Filter by account type
        let account_type = match parse_account_type(account_type_str) {
            Ok(at) => at,
            Err(e) => {
                return HttpResponse::BadRequest().json(serde_json::json!({
                    "error": e
                }));
            }
        };
        state
            .account_use_cases
            .list_accounts_by_type(account_type, organization_id)
            .await
    } else if let Some(ref parent_code) = query.parent_code {
        // Filter by parent code
        state
            .account_use_cases
            .list_child_accounts(parent_code, organization_id)
            .await
    } else if query.direct_use_only.unwrap_or(false) {
        // Only direct-use accounts
        state
            .account_use_cases
            .list_direct_use_accounts(organization_id)
            .await
    } else {
        // List all accounts
        state.account_use_cases.list_accounts(organization_id).await
    };

    match accounts_result {
        Ok(accounts) => {
            let dtos: Vec<AccountResponseDto> = accounts.iter().map(account_to_dto).collect();
            HttpResponse::Ok().json(dtos)
        }
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": err
        })),
    }
}

// ============================================================================
// GET /api/v1/accounts/:id - Get account by ID
// ============================================================================

#[get("/accounts/{id}")]
pub async fn get_account(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    id: web::Path<String>,
) -> impl Responder {
    let account_id = match Uuid::parse_str(&id) {
        Ok(uuid) => uuid,
        Err(_) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid account ID format"
            }))
        }
    };

    match state.account_use_cases.get_account(account_id).await {
        Ok(Some(account)) => {
            // Authorization: Check user belongs to organization (unless SuperAdmin)
            if user.role != "superadmin" {
                if let Ok(user_org_id) = user.require_organization() {
                    if user_org_id != account.organization_id {
                        return HttpResponse::Forbidden().json(serde_json::json!({
                            "error": "You can only view accounts from your organization"
                        }));
                    }
                }
            }

            HttpResponse::Ok().json(account_to_dto(&account))
        }
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Account not found"
        })),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": err
        })),
    }
}

// ============================================================================
// GET /api/v1/accounts/code/:code - Get account by code
// ============================================================================

#[get("/accounts/code/{code}")]
pub async fn get_account_by_code(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    code: web::Path<String>,
) -> impl Responder {
    let organization_id = match user.require_organization() {
        Ok(id) => id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": e.to_string()
            }))
        }
    };

    match state
        .account_use_cases
        .get_account_by_code(&code, organization_id)
        .await
    {
        Ok(Some(account)) => HttpResponse::Ok().json(account_to_dto(&account)),
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Account not found"
        })),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": err
        })),
    }
}

// ============================================================================
// PUT /api/v1/accounts/:id - Update account
// ============================================================================

#[put("/accounts/{id}")]
pub async fn update_account(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    id: web::Path<String>,
    dto: web::Json<UpdateAccountDto>,
) -> impl Responder {
    // Permission: Accountant or SuperAdmin
    if user.role != "accountant" && user.role != "superadmin" {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Only Accountant or SuperAdmin can update accounts"
        }));
    }

    // Validate DTO
    if let Err(errors) = dto.validate() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Validation failed",
            "details": errors.to_string()
        }));
    }

    let account_id = match Uuid::parse_str(&id) {
        Ok(uuid) => uuid,
        Err(_) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid account ID format"
            }))
        }
    };

    // Check account exists and user has permission
    let existing_account = match state.account_use_cases.get_account(account_id).await {
        Ok(Some(acc)) => acc,
        Ok(None) => {
            return HttpResponse::NotFound().json(serde_json::json!({
                "error": "Account not found"
            }))
        }
        Err(err) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": err
            }))
        }
    };

    // Authorization check
    if user.role != "superadmin" {
        if let Ok(user_org_id) = user.require_organization() {
            if user_org_id != existing_account.organization_id {
                return HttpResponse::Forbidden().json(serde_json::json!({
                    "error": "You can only update accounts in your organization"
                }));
            }
        }
    }

    // Parse account_type if provided
    let account_type = if let Some(ref type_str) = dto.account_type {
        Some(match parse_account_type(type_str) {
            Ok(at) => at,
            Err(e) => {
                return HttpResponse::BadRequest().json(serde_json::json!({
                    "error": e
                }));
            }
        })
    } else {
        None
    };

    // Call use case
    match state
        .account_use_cases
        .update_account(
            account_id,
            dto.label.clone(),
            dto.parent_code.clone(),
            account_type,
            dto.direct_use,
        )
        .await
    {
        Ok(account) => {
            // Audit log
            AuditLogEntry::new(
                AuditEventType::AccountUpdated,
                Some(user.user_id),
                Some(existing_account.organization_id),
            )
            .with_resource("Account", account.id)
            .log();

            HttpResponse::Ok().json(account_to_dto(&account))
        }
        Err(err) => {
            // Audit log failure
            AuditLogEntry::new(
                AuditEventType::AccountUpdated,
                Some(user.user_id),
                Some(existing_account.organization_id),
            )
            .with_error(err.clone())
            .log();

            HttpResponse::BadRequest().json(serde_json::json!({
                "error": err
            }))
        }
    }
}

// ============================================================================
// DELETE /api/v1/accounts/:id - Delete account
// ============================================================================

#[delete("/accounts/{id}")]
pub async fn delete_account(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    id: web::Path<String>,
) -> impl Responder {
    // Permission: Accountant or SuperAdmin
    if user.role != "accountant" && user.role != "superadmin" {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Only Accountant or SuperAdmin can delete accounts"
        }));
    }

    let account_id = match Uuid::parse_str(&id) {
        Ok(uuid) => uuid,
        Err(_) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid account ID format"
            }))
        }
    };

    // Check account exists and user has permission
    let existing_account = match state.account_use_cases.get_account(account_id).await {
        Ok(Some(acc)) => acc,
        Ok(None) => {
            return HttpResponse::NotFound().json(serde_json::json!({
                "error": "Account not found"
            }))
        }
        Err(err) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": err
            }))
        }
    };

    // Authorization check
    if user.role != "superadmin" {
        if let Ok(user_org_id) = user.require_organization() {
            if user_org_id != existing_account.organization_id {
                return HttpResponse::Forbidden().json(serde_json::json!({
                    "error": "You can only delete accounts in your organization"
                }));
            }
        }
    }

    // Call use case (validates no children, not used in expenses)
    match state.account_use_cases.delete_account(account_id).await {
        Ok(()) => {
            // Audit log
            AuditLogEntry::new(
                AuditEventType::AccountDeleted,
                Some(user.user_id),
                Some(existing_account.organization_id),
            )
            .with_resource("Account", account_id)
            .log();

            HttpResponse::NoContent().finish()
        }
        Err(err) => {
            // Audit log failure
            AuditLogEntry::new(
                AuditEventType::AccountDeleted,
                Some(user.user_id),
                Some(existing_account.organization_id),
            )
            .with_error(err.clone())
            .log();

            HttpResponse::BadRequest().json(serde_json::json!({
                "error": err
            }))
        }
    }
}

// ============================================================================
// POST /api/v1/accounts/seed/belgian-pcmn - Seed Belgian PCMN
// ============================================================================

#[post("/accounts/seed/belgian-pcmn")]
pub async fn seed_belgian_pcmn(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    dto: web::Json<SeedBelgianPcmnDto>,
) -> impl Responder {
    // Permission: SuperAdmin or Accountant
    if user.role != "superadmin" && user.role != "accountant" {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Only SuperAdmin or Accountant can seed PCMN"
        }));
    }

    // Parse organization_id
    let organization_id = match Uuid::parse_str(&dto.organization_id) {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid organization_id format"
            }));
        }
    };

    // Authorization: Check user belongs to organization (unless SuperAdmin)
    if user.role != "superadmin" {
        if let Ok(user_org_id) = user.require_organization() {
            if user_org_id != organization_id {
                return HttpResponse::Forbidden().json(serde_json::json!({
                    "error": "You can only seed PCMN for your own organization"
                }));
            }
        }
    }

    // Call use case
    match state
        .account_use_cases
        .seed_belgian_pcmn(organization_id)
        .await
    {
        Ok(count) => {
            // Audit log
            AuditLogEntry::new(
                AuditEventType::BelgianPCMNSeeded,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_metadata(serde_json::json!({
                "accounts_created": count
            }))
            .log();

            HttpResponse::Ok().json(SeedPcmnResponseDto {
                accounts_created: count,
                message: format!(
                    "Successfully created {} Belgian PCMN accounts for organization",
                    count
                ),
            })
        }
        Err(err) => {
            // Audit log failure
            AuditLogEntry::new(
                AuditEventType::BelgianPCMNSeeded,
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

// ============================================================================
// GET /api/v1/accounts/count - Count accounts in organization
// ============================================================================

#[get("/accounts/count")]
pub async fn count_accounts(state: web::Data<AppState>, user: AuthenticatedUser) -> impl Responder {
    let organization_id = match user.require_organization() {
        Ok(id) => id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": e.to_string()
            }))
        }
    };

    match state
        .account_use_cases
        .count_accounts(organization_id)
        .await
    {
        Ok(count) => HttpResponse::Ok().json(serde_json::json!({
            "count": count,
            "organization_id": organization_id.to_string()
        })),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": err
        })),
    }
}
