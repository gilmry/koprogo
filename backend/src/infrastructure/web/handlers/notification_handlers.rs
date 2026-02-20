use crate::application::dto::{
    CreateNotificationRequest, MarkReadRequest, UpdatePreferenceRequest,
};
use crate::domain::entities::NotificationType;
use crate::infrastructure::audit::{AuditEventType, AuditLogEntry};
use crate::infrastructure::web::{AppState, AuthenticatedUser};
use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use uuid::Uuid;

/// Parse notification type from both snake_case and PascalCase
fn parse_notification_type(s: &str) -> Option<NotificationType> {
    match s {
        "expense_created" | "ExpenseCreated" => Some(NotificationType::ExpenseCreated),
        "meeting_convocation" | "MeetingConvocation" => Some(NotificationType::MeetingConvocation),
        "payment_received" | "PaymentReceived" => Some(NotificationType::PaymentReceived),
        "ticket_resolved" | "TicketResolved" => Some(NotificationType::TicketResolved),
        "document_added" | "DocumentAdded" => Some(NotificationType::DocumentAdded),
        "board_message" | "BoardMessage" => Some(NotificationType::BoardMessage),
        "payment_reminder" | "PaymentReminder" => Some(NotificationType::PaymentReminder),
        "budget_approved" | "BudgetApproved" => Some(NotificationType::BudgetApproved),
        "resolution_vote" | "ResolutionVote" => Some(NotificationType::ResolutionVote),
        "system" | "System" => Some(NotificationType::System),
        _ => None,
    }
}

// ==================== Notification Endpoints ====================

#[post("/notifications")]
pub async fn create_notification(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    request: web::Json<CreateNotificationRequest>,
) -> impl Responder {
    let organization_id = match user.require_organization() {
        Ok(org_id) => org_id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": e.to_string()}))
        }
    };

    match state
        .notification_use_cases
        .create_notification(organization_id, request.into_inner())
        .await
    {
        Ok(notification) => {
            AuditLogEntry::new(
                AuditEventType::NotificationCreated,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_resource("Notification", notification.id)
            .log();

            HttpResponse::Created().json(notification)
        }
        Err(err) => {
            AuditLogEntry::new(
                AuditEventType::NotificationCreated,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_error(err.clone())
            .log();

            HttpResponse::BadRequest().json(serde_json::json!({"error": err}))
        }
    }
}

#[get("/notifications/{id}")]
pub async fn get_notification(
    state: web::Data<AppState>,
    _user: AuthenticatedUser,
    id: web::Path<Uuid>,
) -> impl Responder {
    match state.notification_use_cases.get_notification(*id).await {
        Ok(Some(notification)) => HttpResponse::Ok().json(notification),
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Notification not found"
        })),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({"error": err})),
    }
}

#[get("/notifications/my")]
pub async fn list_my_notifications(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
) -> impl Responder {
    match state
        .notification_use_cases
        .list_user_notifications(user.user_id)
        .await
    {
        Ok(notifications) => HttpResponse::Ok().json(notifications),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({"error": err})),
    }
}

#[get("/notifications/unread")]
pub async fn list_unread_notifications(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
) -> impl Responder {
    match state
        .notification_use_cases
        .list_unread_notifications(user.user_id)
        .await
    {
        Ok(notifications) => HttpResponse::Ok().json(notifications),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({"error": err})),
    }
}

#[put("/notifications/{id}/read")]
pub async fn mark_notification_read(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    id: web::Path<Uuid>,
    _request: web::Json<MarkReadRequest>,
) -> impl Responder {
    let organization_id = match user.require_organization() {
        Ok(org_id) => org_id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": e.to_string()}))
        }
    };

    match state.notification_use_cases.mark_as_read(*id).await {
        Ok(notification) => {
            AuditLogEntry::new(
                AuditEventType::NotificationRead,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_resource("Notification", notification.id)
            .log();

            HttpResponse::Ok().json(notification)
        }
        Err(err) => {
            AuditLogEntry::new(
                AuditEventType::NotificationRead,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_error(err.clone())
            .log();

            HttpResponse::BadRequest().json(serde_json::json!({"error": err}))
        }
    }
}

#[put("/notifications/read-all")]
pub async fn mark_all_notifications_read(
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
        .notification_use_cases
        .mark_all_read(user.user_id)
        .await
    {
        Ok(count) => {
            AuditLogEntry::new(
                AuditEventType::NotificationRead,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_details(format!("Marked {} notifications as read", count))
            .log();

            HttpResponse::Ok().json(serde_json::json!({"marked_read": count}))
        }
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({"error": err})),
    }
}

#[delete("/notifications/{id}")]
pub async fn delete_notification(
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

    match state.notification_use_cases.delete_notification(*id).await {
        Ok(true) => {
            AuditLogEntry::new(
                AuditEventType::NotificationDeleted,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_resource("Notification", *id)
            .log();

            HttpResponse::NoContent().finish()
        }
        Ok(false) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Notification not found"
        })),
        Err(err) => {
            AuditLogEntry::new(
                AuditEventType::NotificationDeleted,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_error(err.clone())
            .log();

            HttpResponse::BadRequest().json(serde_json::json!({"error": err}))
        }
    }
}

#[get("/notifications/stats")]
pub async fn get_notification_stats(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
) -> impl Responder {
    match state
        .notification_use_cases
        .get_user_stats(user.user_id)
        .await
    {
        Ok(stats) => HttpResponse::Ok().json(stats),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({"error": err})),
    }
}

// ==================== Notification Preference Endpoints ====================

#[get("/notification-preferences")]
pub async fn get_user_preferences(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
) -> impl Responder {
    match state
        .notification_use_cases
        .get_user_preferences(user.user_id)
        .await
    {
        Ok(preferences) => HttpResponse::Ok().json(preferences),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({"error": err})),
    }
}

#[get("/notification-preferences/{notification_type}")]
pub async fn get_preference(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    notification_type: web::Path<String>,
) -> impl Responder {
    let notification_type = match parse_notification_type(notification_type.as_str()) {
        Some(nt) => nt,
        None => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": format!("Invalid notification type: {}", notification_type)
            }))
        }
    };

    match state
        .notification_use_cases
        .get_preference(user.user_id, notification_type)
        .await
    {
        Ok(Some(preference)) => HttpResponse::Ok().json(preference),
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Preference not found"
        })),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({"error": err})),
    }
}

#[put("/notification-preferences/{notification_type}")]
pub async fn update_preference(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    notification_type: web::Path<String>,
    request: web::Json<UpdatePreferenceRequest>,
) -> impl Responder {
    let organization_id = match user.require_organization() {
        Ok(org_id) => org_id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": e.to_string()}))
        }
    };

    let notification_type = match parse_notification_type(notification_type.as_str()) {
        Some(nt) => nt,
        None => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": format!("Invalid notification type: {}", notification_type)
            }))
        }
    };

    match state
        .notification_use_cases
        .update_preference(user.user_id, notification_type, request.into_inner())
        .await
    {
        Ok(preference) => {
            AuditLogEntry::new(
                AuditEventType::NotificationPreferenceUpdated,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_resource("NotificationPreference", preference.id)
            .log();

            HttpResponse::Ok().json(preference)
        }
        Err(err) => {
            AuditLogEntry::new(
                AuditEventType::NotificationPreferenceUpdated,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_error(err.clone())
            .log();

            HttpResponse::BadRequest().json(serde_json::json!({"error": err}))
        }
    }
}
