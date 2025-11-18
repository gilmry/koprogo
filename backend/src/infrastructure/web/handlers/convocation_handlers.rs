use crate::application::dto::{
    CreateConvocationRequest, ScheduleConvocationRequest, SendConvocationRequest, SetProxyRequest,
    UpdateAttendanceRequest,
};
use crate::infrastructure::audit::{AuditEventType, AuditLogEntry};
use crate::infrastructure::web::{AppState, AuthenticatedUser};
use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use uuid::Uuid;

// ==================== Convocation CRUD Endpoints ====================

#[post("/convocations")]
pub async fn create_convocation(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    request: web::Json<CreateConvocationRequest>,
) -> impl Responder {
    let organization_id = match user.require_organization() {
        Ok(org_id) => org_id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": e.to_string()}))
        }
    };

    let created_by = user.user_id;

    match state
        .convocation_use_cases
        .create_convocation(organization_id, request.into_inner(), created_by)
        .await
    {
        Ok(convocation) => {
            AuditLogEntry::new(
                AuditEventType::ConvocationCreated,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_resource("Convocation", convocation.id)
            .log();

            HttpResponse::Created().json(convocation)
        }
        Err(err) => {
            AuditLogEntry::new(
                AuditEventType::ConvocationCreated,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_error(err.clone())
            .log();

            HttpResponse::BadRequest().json(serde_json::json!({"error": err}))
        }
    }
}

#[get("/convocations/{id}")]
pub async fn get_convocation(state: web::Data<AppState>, id: web::Path<Uuid>) -> impl Responder {
    match state.convocation_use_cases.get_convocation(*id).await {
        Ok(convocation) => HttpResponse::Ok().json(convocation),
        Err(err) => HttpResponse::NotFound().json(serde_json::json!({"error": err})),
    }
}

#[get("/meetings/{meeting_id}/convocation")]
pub async fn get_convocation_by_meeting(
    state: web::Data<AppState>,
    meeting_id: web::Path<Uuid>,
) -> impl Responder {
    match state
        .convocation_use_cases
        .get_convocation_by_meeting(*meeting_id)
        .await
    {
        Ok(Some(convocation)) => HttpResponse::Ok().json(convocation),
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Convocation not found for this meeting"
        })),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({"error": err})),
    }
}

#[get("/buildings/{building_id}/convocations")]
pub async fn list_building_convocations(
    state: web::Data<AppState>,
    building_id: web::Path<Uuid>,
) -> impl Responder {
    match state
        .convocation_use_cases
        .list_building_convocations(*building_id)
        .await
    {
        Ok(convocations) => HttpResponse::Ok().json(convocations),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({"error": err})),
    }
}

#[get("/organizations/{organization_id}/convocations")]
pub async fn list_organization_convocations(
    state: web::Data<AppState>,
    organization_id: web::Path<Uuid>,
) -> impl Responder {
    match state
        .convocation_use_cases
        .list_organization_convocations(*organization_id)
        .await
    {
        Ok(convocations) => HttpResponse::Ok().json(convocations),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({"error": err})),
    }
}

#[delete("/convocations/{id}")]
pub async fn delete_convocation(
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

    match state.convocation_use_cases.delete_convocation(*id).await {
        Ok(true) => {
            AuditLogEntry::new(
                AuditEventType::ConvocationDeleted,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_resource("Convocation", *id)
            .log();

            HttpResponse::NoContent().finish()
        }
        Ok(false) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Convocation not found"
        })),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({"error": err})),
    }
}

// ==================== Convocation Actions ====================

#[put("/convocations/{id}/schedule")]
pub async fn schedule_convocation(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    id: web::Path<Uuid>,
    request: web::Json<ScheduleConvocationRequest>,
) -> impl Responder {
    let organization_id = match user.require_organization() {
        Ok(org_id) => org_id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": e.to_string()}))
        }
    };

    match state
        .convocation_use_cases
        .schedule_convocation(*id, request.into_inner())
        .await
    {
        Ok(convocation) => {
            AuditLogEntry::new(
                AuditEventType::ConvocationScheduled,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_resource("Convocation", convocation.id)
            .log();

            HttpResponse::Ok().json(convocation)
        }
        Err(err) => HttpResponse::BadRequest().json(serde_json::json!({"error": err})),
    }
}

#[post("/convocations/{id}/send")]
pub async fn send_convocation(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    id: web::Path<Uuid>,
    request: web::Json<SendConvocationRequest>,
) -> impl Responder {
    let organization_id = match user.require_organization() {
        Ok(org_id) => org_id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": e.to_string()}))
        }
    };

    // TODO: Generate PDF file path (placeholder for now)
    let pdf_file_path = format!("/uploads/convocations/conv-{}.pdf", id);

    match state
        .convocation_use_cases
        .send_convocation(*id, request.into_inner(), pdf_file_path)
        .await
    {
        Ok(convocation) => {
            AuditLogEntry::new(
                AuditEventType::ConvocationSent,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_resource("Convocation", convocation.id)
            .with_details(format!("recipients: {}", convocation.total_recipients))
            .log();

            HttpResponse::Ok().json(convocation)
        }
        Err(err) => HttpResponse::BadRequest().json(serde_json::json!({"error": err})),
    }
}

#[put("/convocations/{id}/cancel")]
pub async fn cancel_convocation(
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

    match state.convocation_use_cases.cancel_convocation(*id).await {
        Ok(convocation) => {
            AuditLogEntry::new(
                AuditEventType::ConvocationCancelled,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_resource("Convocation", convocation.id)
            .log();

            HttpResponse::Ok().json(convocation)
        }
        Err(err) => HttpResponse::BadRequest().json(serde_json::json!({"error": err})),
    }
}

// ==================== Recipient Endpoints ====================

#[get("/convocations/{id}/recipients")]
pub async fn list_convocation_recipients(
    state: web::Data<AppState>,
    id: web::Path<Uuid>,
) -> impl Responder {
    match state
        .convocation_use_cases
        .list_convocation_recipients(*id)
        .await
    {
        Ok(recipients) => HttpResponse::Ok().json(recipients),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({"error": err})),
    }
}

#[get("/convocations/{id}/tracking-summary")]
pub async fn get_convocation_tracking_summary(
    state: web::Data<AppState>,
    id: web::Path<Uuid>,
) -> impl Responder {
    match state.convocation_use_cases.get_tracking_summary(*id).await {
        Ok(summary) => HttpResponse::Ok().json(summary),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({"error": err})),
    }
}

#[put("/convocation-recipients/{id}/email-opened")]
pub async fn mark_recipient_email_opened(
    state: web::Data<AppState>,
    id: web::Path<Uuid>,
) -> impl Responder {
    match state
        .convocation_use_cases
        .mark_recipient_email_opened(*id)
        .await
    {
        Ok(recipient) => HttpResponse::Ok().json(recipient),
        Err(err) => HttpResponse::BadRequest().json(serde_json::json!({"error": err})),
    }
}

#[put("/convocation-recipients/{id}/attendance")]
pub async fn update_recipient_attendance(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    id: web::Path<Uuid>,
    request: web::Json<UpdateAttendanceRequest>,
) -> impl Responder {
    let organization_id = match user.require_organization() {
        Ok(org_id) => org_id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": e.to_string()}))
        }
    };

    match state
        .convocation_use_cases
        .update_recipient_attendance(*id, request.attendance_status.clone())
        .await
    {
        Ok(recipient) => {
            AuditLogEntry::new(
                AuditEventType::ConvocationAttendanceUpdated,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_resource("ConvocationRecipient", recipient.id)
            .with_details(format!("status: {:?}", recipient.attendance_status))
            .log();

            HttpResponse::Ok().json(recipient)
        }
        Err(err) => HttpResponse::BadRequest().json(serde_json::json!({"error": err})),
    }
}

#[put("/convocation-recipients/{id}/proxy")]
pub async fn set_recipient_proxy(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    id: web::Path<Uuid>,
    request: web::Json<SetProxyRequest>,
) -> impl Responder {
    let organization_id = match user.require_organization() {
        Ok(org_id) => org_id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": e.to_string()}))
        }
    };

    match state
        .convocation_use_cases
        .set_recipient_proxy(*id, request.proxy_owner_id)
        .await
    {
        Ok(recipient) => {
            AuditLogEntry::new(
                AuditEventType::ConvocationProxySet,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_resource("ConvocationRecipient", recipient.id)
            .with_details(format!("proxy_owner_id: {:?}", recipient.proxy_owner_id))
            .log();

            HttpResponse::Ok().json(recipient)
        }
        Err(err) => HttpResponse::BadRequest().json(serde_json::json!({"error": err})),
    }
}

#[post("/convocations/{id}/send-reminders")]
pub async fn send_convocation_reminders(
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

    match state.convocation_use_cases.send_reminders(*id).await {
        Ok(recipients) => {
            AuditLogEntry::new(
                AuditEventType::ConvocationReminderSent,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_resource("Convocation", *id)
            .with_details(format!("recipients: {}", recipients.len()))
            .log();

            HttpResponse::Ok().json(recipients)
        }
        Err(err) => HttpResponse::BadRequest().json(serde_json::json!({"error": err})),
    }
}
