use crate::application::dto::PaymentReminderResponseDto;
use crate::application::dto::{
    AddTrackingNumberDto, BulkCreateRemindersDto, CancelReminderDto, CreatePaymentReminderDto,
    EscalateReminderDto, MarkReminderSentDto,
};
use crate::application::error::AppError;
use crate::infrastructure::audit::{AuditEventType, AuditLogEntry};
use crate::infrastructure::web::middleware::scope_guard::own_owner_ids;
use crate::infrastructure::web::{AppState, AuthenticatedUser};
use actix_web::{delete, get, post, put, web, HttpResponse, Responder, ResponseError};
use std::collections::HashMap;
use uuid::Uuid;
use validator::Validate;

/// Helper function to check if owner role is trying to modify data
/// Owners can view their own reminders but cannot create/modify them
fn verify_write_permission(user: &AuthenticatedUser) -> Option<HttpResponse> {
    if user.role == "owner" {
        Some(HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Owner role cannot create or modify payment reminders"
        })))
    } else {
        None
    }
}

/// Create a new payment reminder
#[post("/payment-reminders")]
pub async fn create_reminder(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    mut dto: web::Json<CreatePaymentReminderDto>,
) -> impl Responder {
    if let Some(response) = verify_write_permission(&user) {
        return response;
    }

    // Override the organization_id from DTO with the one from JWT token
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
        .payment_reminder_use_cases
        .create_reminder(dto.into_inner())
        .await
    {
        Ok(reminder) => {
            // Audit log: successful reminder creation
            AuditLogEntry::new(
                AuditEventType::PaymentReminderCreated,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_resource("PaymentReminder", Uuid::parse_str(&reminder.id).unwrap())
            .log();

            HttpResponse::Created().json(reminder)
        }
        Err(err) => {
            // Audit log: failed reminder creation
            AuditLogEntry::new(
                AuditEventType::PaymentReminderCreated,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_error(err.to_string())
            .log();

            HttpResponse::BadRequest().json(serde_json::json!({
                "error": err.to_string()
            }))
        }
    }
}

/// Ne garde que les rappels que l'appelant a le droit de lire (#1027).
///
/// Un rappel dit qui est en retard de paiement, et de combien. Les lectures
/// de ce fichier ne vérifiaient que l'existence d'une organisation dans le
/// jeton, pas celle du rappel : un identifiant de rappel, d'une dépense ou
/// d'un copropriétaire suffisait à lire les retards d'une autre copropriété,
/// et un copropriétaire lisait ceux de tous ses voisins.
///
/// - superadmin : tout ;
/// - copropriétaire : ses propres rappels seulement ;
/// - autres rôles : les rappels des copropriétaires de leur organisation.
async fn rappels_visibles(
    state: &AppState,
    user: &AuthenticatedUser,
    rappels: Vec<PaymentReminderResponseDto>,
) -> Result<Vec<PaymentReminderResponseDto>, AppError> {
    if user.is_superadmin() {
        return Ok(rappels);
    }
    if user.role == "owner" {
        let miens: Vec<String> = own_owner_ids(user, &state.owner_use_cases)
            .await?
            .iter()
            .map(Uuid::to_string)
            .collect();
        return Ok(rappels
            .into_iter()
            .filter(|r| miens.contains(&r.owner_id))
            .collect());
    }
    let Some(org_id) = user.organization_id else {
        return Ok(Vec::new());
    };
    let org = org_id.to_string();
    let mut organisation_de: HashMap<String, Option<String>> = HashMap::new();
    let mut visibles = Vec::new();
    for rappel in rappels {
        if !organisation_de.contains_key(&rappel.owner_id) {
            let fiche = match Uuid::parse_str(&rappel.owner_id) {
                Ok(id) => state
                    .owner_use_cases
                    .get_owner(id)
                    .await
                    .map_err(AppError::from)?,
                Err(_) => None,
            };
            organisation_de.insert(rappel.owner_id.clone(), fiche.map(|f| f.organization_id));
        }
        if organisation_de[&rappel.owner_id].as_deref() == Some(org.as_str()) {
            visibles.push(rappel);
        }
    }
    Ok(visibles)
}

/// Get a payment reminder by ID
#[get("/payment-reminders/{id}")]
pub async fn get_reminder(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    id: web::Path<Uuid>,
) -> impl Responder {
    let _ = match user.require_organization() {
        Ok(org_id) => org_id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": e.to_string()
            }))
        }
    };

    match state.payment_reminder_use_cases.get_reminder(*id).await {
        Ok(Some(reminder)) => match rappels_visibles(&state, &user, vec![reminder]).await {
            Ok(mut visibles) => match visibles.pop() {
                Some(reminder) => HttpResponse::Ok().json(reminder),
                None => HttpResponse::NotFound().json(serde_json::json!({
                    "error": "Payment reminder not found"
                })),
            },
            Err(err) => err.error_response(),
        },
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Payment reminder not found"
        })),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": err.to_string()
        })),
    }
}

/// List all payment reminders for an expense
#[get("/expenses/{expense_id}/payment-reminders")]
pub async fn list_by_expense(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    expense_id: web::Path<Uuid>,
) -> impl Responder {
    let _ = match user.require_organization() {
        Ok(org_id) => org_id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": e.to_string()
            }))
        }
    };

    match state
        .payment_reminder_use_cases
        .list_by_expense(*expense_id)
        .await
    {
        Ok(reminders) => match rappels_visibles(&state, &user, reminders).await {
            Ok(visibles) => HttpResponse::Ok().json(visibles),
            Err(err) => err.error_response(),
        },
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": err.to_string()
        })),
    }
}

/// List all payment reminders for an owner
#[get("/owners/{owner_id}/payment-reminders")]
pub async fn list_by_owner(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    owner_id: web::Path<Uuid>,
) -> impl Responder {
    let _ = match user.require_organization() {
        Ok(org_id) => org_id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": e.to_string()
            }))
        }
    };

    match state
        .payment_reminder_use_cases
        .list_by_owner(*owner_id)
        .await
    {
        Ok(reminders) => match rappels_visibles(&state, &user, reminders).await {
            Ok(visibles) => HttpResponse::Ok().json(visibles),
            Err(err) => err.error_response(),
        },
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": err.to_string()
        })),
    }
}

/// List all active payment reminders for an owner (non-paid, non-cancelled)
#[get("/owners/{owner_id}/payment-reminders/active")]
pub async fn list_active_by_owner(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    owner_id: web::Path<Uuid>,
) -> impl Responder {
    let _ = match user.require_organization() {
        Ok(org_id) => org_id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": e.to_string()
            }))
        }
    };

    match state
        .payment_reminder_use_cases
        .list_active_by_owner(*owner_id)
        .await
    {
        Ok(reminders) => match rappels_visibles(&state, &user, reminders).await {
            Ok(visibles) => HttpResponse::Ok().json(visibles),
            Err(err) => err.error_response(),
        },
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": err.to_string()
        })),
    }
}

/// List all payment reminders for an organization
#[get("/payment-reminders")]
pub async fn list_by_organization(
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

    match state
        .payment_reminder_use_cases
        .list_by_organization(organization_id)
        .await
    {
        Ok(reminders) => match rappels_visibles(&state, &user, reminders).await {
            Ok(visibles) => HttpResponse::Ok().json(visibles),
            Err(err) => err.error_response(),
        },
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": err.to_string()
        })),
    }
}

/// Mark a payment reminder as sent
#[put("/payment-reminders/{id}/mark-sent")]
pub async fn mark_as_sent(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    id: web::Path<Uuid>,
    dto: web::Json<MarkReminderSentDto>,
) -> impl Responder {
    if let Some(response) = verify_write_permission(&user) {
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

    if let Err(errors) = dto.validate() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Validation failed",
            "details": errors.to_string()
        }));
    }

    match state
        .payment_reminder_use_cases
        .mark_as_sent(*id, dto.into_inner())
        .await
    {
        Ok(reminder) => {
            AuditLogEntry::new(
                AuditEventType::PaymentReminderSent,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_resource("PaymentReminder", *id)
            .log();

            HttpResponse::Ok().json(reminder)
        }
        Err(err) => HttpResponse::BadRequest().json(serde_json::json!({
            "error": err.to_string()
        })),
    }
}

/// Mark a payment reminder as opened (email opened)
#[put("/payment-reminders/{id}/mark-opened")]
pub async fn mark_as_opened(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    id: web::Path<Uuid>,
) -> impl Responder {
    if let Some(response) = verify_write_permission(&user) {
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

    match state.payment_reminder_use_cases.mark_as_opened(*id).await {
        Ok(reminder) => {
            AuditLogEntry::new(
                AuditEventType::PaymentReminderOpened,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_resource("PaymentReminder", *id)
            .log();

            HttpResponse::Ok().json(reminder)
        }
        Err(err) => HttpResponse::BadRequest().json(serde_json::json!({
            "error": err.to_string()
        })),
    }
}

/// Mark a payment reminder as paid
#[put("/payment-reminders/{id}/mark-paid")]
pub async fn mark_as_paid(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    id: web::Path<Uuid>,
) -> impl Responder {
    if let Some(response) = verify_write_permission(&user) {
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

    match state.payment_reminder_use_cases.mark_as_paid(*id).await {
        Ok(reminder) => {
            AuditLogEntry::new(
                AuditEventType::PaymentReminderPaid,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_resource("PaymentReminder", *id)
            .log();

            HttpResponse::Ok().json(reminder)
        }
        Err(err) => HttpResponse::BadRequest().json(serde_json::json!({
            "error": err.to_string()
        })),
    }
}

/// Cancel a payment reminder
#[put("/payment-reminders/{id}/cancel")]
pub async fn cancel_reminder(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    id: web::Path<Uuid>,
    dto: web::Json<CancelReminderDto>,
) -> impl Responder {
    if let Some(response) = verify_write_permission(&user) {
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

    if let Err(errors) = dto.validate() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Validation failed",
            "details": errors.to_string()
        }));
    }

    match state
        .payment_reminder_use_cases
        .cancel_reminder(*id, dto.into_inner())
        .await
    {
        Ok(reminder) => {
            AuditLogEntry::new(
                AuditEventType::PaymentReminderCancelled,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_resource("PaymentReminder", *id)
            .log();

            HttpResponse::Ok().json(reminder)
        }
        Err(err) => HttpResponse::BadRequest().json(serde_json::json!({
            "error": err.to_string()
        })),
    }
}

/// Escalate a payment reminder to next level
#[post("/payment-reminders/{id}/escalate")]
pub async fn escalate_reminder(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    id: web::Path<Uuid>,
    dto: web::Json<EscalateReminderDto>,
) -> impl Responder {
    if let Some(response) = verify_write_permission(&user) {
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
        .payment_reminder_use_cases
        .escalate_reminder(*id, dto.into_inner())
        .await
    {
        Ok(Some(reminder)) => {
            AuditLogEntry::new(
                AuditEventType::PaymentReminderEscalated,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_resource("PaymentReminder", *id)
            .log();

            HttpResponse::Ok().json(reminder)
        }
        Ok(None) => HttpResponse::Ok().json(serde_json::json!({
            "message": "Reminder escalated (no next level created)"
        })),
        Err(err) => HttpResponse::BadRequest().json(serde_json::json!({
            "error": err.to_string()
        })),
    }
}

/// Add tracking number to a payment reminder (for registered letters)
#[put("/payment-reminders/{id}/tracking-number")]
pub async fn add_tracking_number(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    id: web::Path<Uuid>,
    dto: web::Json<AddTrackingNumberDto>,
) -> impl Responder {
    if let Some(response) = verify_write_permission(&user) {
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

    if let Err(errors) = dto.validate() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Validation failed",
            "details": errors.to_string()
        }));
    }

    match state
        .payment_reminder_use_cases
        .add_tracking_number(*id, dto.into_inner())
        .await
    {
        Ok(reminder) => {
            AuditLogEntry::new(
                AuditEventType::PaymentReminderTrackingAdded,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_resource("PaymentReminder", *id)
            .log();

            HttpResponse::Ok().json(reminder)
        }
        Err(err) => HttpResponse::BadRequest().json(serde_json::json!({
            "error": err.to_string()
        })),
    }
}

/// Get payment recovery statistics for organization
#[get("/payment-reminders/stats")]
pub async fn get_recovery_stats(
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

    match state
        .payment_reminder_use_cases
        .get_recovery_stats(organization_id)
        .await
    {
        Ok(stats) => HttpResponse::Ok().json(stats),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": err.to_string()
        })),
    }
}

/// Find overdue expenses without reminders (for manual review)
#[get("/payment-reminders/overdue-without-reminders")]
pub async fn find_overdue_without_reminders(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    query: web::Query<serde_json::Value>,
) -> impl Responder {
    if let Some(response) = verify_write_permission(&user) {
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

    let min_days_overdue = query
        .get("min_days_overdue")
        .and_then(|v| v.as_i64())
        .unwrap_or(15);

    match state
        .payment_reminder_use_cases
        .find_overdue_expenses_without_reminders(organization_id, min_days_overdue)
        .await
    {
        Ok(overdue) => HttpResponse::Ok().json(overdue),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": err.to_string()
        })),
    }
}

/// Bulk create reminders for all overdue expenses
#[post("/payment-reminders/bulk-create")]
pub async fn bulk_create_reminders(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    mut dto: web::Json<BulkCreateRemindersDto>,
) -> impl Responder {
    if let Some(response) = verify_write_permission(&user) {
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
    dto.organization_id = organization_id.to_string();

    if let Err(errors) = dto.validate() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Validation failed",
            "details": errors.to_string()
        }));
    }

    match state
        .payment_reminder_use_cases
        .bulk_create_reminders(dto.into_inner())
        .await
    {
        Ok(response) => {
            AuditLogEntry::new(
                AuditEventType::PaymentRemindersBulkCreated,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_metadata(serde_json::json!({
                "created_count": response.created_count,
                "skipped_count": response.skipped_count
            }))
            .log();

            HttpResponse::Ok().json(response)
        }
        Err(err) => HttpResponse::BadRequest().json(serde_json::json!({
            "error": err.to_string()
        })),
    }
}

/// Delete a payment reminder
#[delete("/payment-reminders/{id}")]
pub async fn delete_reminder(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    id: web::Path<Uuid>,
) -> impl Responder {
    if let Some(response) = verify_write_permission(&user) {
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

    match state.payment_reminder_use_cases.delete_reminder(*id).await {
        Ok(true) => {
            AuditLogEntry::new(
                AuditEventType::PaymentReminderDeleted,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_resource("PaymentReminder", *id)
            .log();

            HttpResponse::NoContent().finish()
        }
        Ok(false) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Payment reminder not found"
        })),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": err.to_string()
        })),
    }
}
