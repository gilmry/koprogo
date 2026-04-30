use crate::application::dto::{CreatePaymentRequest, RefundPaymentRequest};
use crate::domain::entities::TransactionStatus;
use crate::infrastructure::audit::{AuditEventType, AuditLogEntry};
use crate::infrastructure::web::{AppState, AuthenticatedUser};
use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use uuid::Uuid;

// ==================== Payment CRUD Endpoints ====================

#[utoipa::path(
    post,
    path = "/payments",
    tag = "Payments",
    summary = "Create a payment",
    request_body = CreatePaymentRequest,
    responses(
        (status = 201, description = "Payment created"),
        (status = 400, description = "Bad request"),
        (status = 401, description = "Unauthorized"),
    ),
    security(("bearer_auth" = []))
)]
#[post("/payments")]
pub async fn create_payment(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    request: web::Json<CreatePaymentRequest>,
) -> impl Responder {
    let organization_id = match user.require_organization() {
        Ok(org_id) => org_id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": e.to_string()}))
        }
    };

    match state
        .payment_use_cases
        .create_payment(organization_id, request.into_inner())
        .await
    {
        Ok(payment) => {
            AuditLogEntry::new(
                AuditEventType::PaymentCreated,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_resource("Payment", payment.id)
            .log();

            HttpResponse::Created().json(payment)
        }
        Err(err) => {
            AuditLogEntry::new(
                AuditEventType::PaymentCreated,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_error(err.clone())
            .log();

            HttpResponse::BadRequest().json(serde_json::json!({"error": err}))
        }
    }
}

#[utoipa::path(
    get,
    path = "/payments/{id}",
    tag = "Payments",
    summary = "Get a payment by ID",
    params(("id" = Uuid, Path, description = "Payment ID")),
    responses(
        (status = 200, description = "Payment found"),
        (status = 404, description = "Payment not found"),
        (status = 500, description = "Internal server error"),
    ),
    security(("bearer_auth" = []))
)]
#[get("/payments/{id}")]
pub async fn get_payment(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    id: web::Path<Uuid>,
) -> impl Responder {
    match state.payment_use_cases.get_payment(*id).await {
        Ok(Some(payment)) => {
            // Multi-tenant isolation: verify payment belongs to user's organization
            if let Err(e) = user.verify_org_access(payment.organization_id) {
                return HttpResponse::Forbidden().json(serde_json::json!({ "error": e }));
            }
            HttpResponse::Ok().json(payment)
        }
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Payment not found"
        })),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({"error": err})),
    }
}

#[utoipa::path(
    get,
    path = "/payments/stripe/{stripe_payment_intent_id}",
    tag = "Payments",
    summary = "Get a payment by Stripe payment intent ID",
    params(("stripe_payment_intent_id" = String, Path, description = "Stripe payment intent ID")),
    responses(
        (status = 200, description = "Payment found"),
        (status = 404, description = "Payment not found"),
        (status = 500, description = "Internal server error"),
    ),
    security(("bearer_auth" = []))
)]
#[get("/payments/stripe/{stripe_payment_intent_id}")]
pub async fn get_payment_by_stripe_intent(
    state: web::Data<AppState>,
    stripe_payment_intent_id: web::Path<String>,
) -> impl Responder {
    match state
        .payment_use_cases
        .get_payment_by_stripe_intent(&stripe_payment_intent_id)
        .await
    {
        Ok(Some(payment)) => HttpResponse::Ok().json(payment),
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Payment not found"
        })),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({"error": err})),
    }
}

#[utoipa::path(
    get,
    path = "/owners/{owner_id}/payments",
    tag = "Payments",
    summary = "List all payments for an owner",
    params(("owner_id" = Uuid, Path, description = "Owner ID")),
    responses(
        (status = 200, description = "List of owner payments"),
        (status = 500, description = "Internal server error"),
    ),
    security(("bearer_auth" = []))
)]
#[get("/owners/{owner_id}/payments")]
pub async fn list_owner_payments(
    state: web::Data<AppState>,
    owner_id: web::Path<Uuid>,
) -> impl Responder {
    match state.payment_use_cases.list_owner_payments(*owner_id).await {
        Ok(payments) => HttpResponse::Ok().json(payments),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({"error": err})),
    }
}

#[utoipa::path(
    get,
    path = "/buildings/{building_id}/payments",
    tag = "Payments",
    summary = "List all payments for a building",
    params(("building_id" = Uuid, Path, description = "Building ID")),
    responses(
        (status = 200, description = "List of building payments"),
        (status = 500, description = "Internal server error"),
    ),
    security(("bearer_auth" = []))
)]
#[get("/buildings/{building_id}/payments")]
pub async fn list_building_payments(
    state: web::Data<AppState>,
    building_id: web::Path<Uuid>,
) -> impl Responder {
    match state
        .payment_use_cases
        .list_building_payments(*building_id)
        .await
    {
        Ok(payments) => HttpResponse::Ok().json(payments),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({"error": err})),
    }
}

#[utoipa::path(
    get,
    path = "/expenses/{expense_id}/payments",
    tag = "Payments",
    summary = "List all payments for an expense",
    params(("expense_id" = Uuid, Path, description = "Expense ID")),
    responses(
        (status = 200, description = "List of expense payments"),
        (status = 500, description = "Internal server error"),
    ),
    security(("bearer_auth" = []))
)]
#[get("/expenses/{expense_id}/payments")]
pub async fn list_expense_payments(
    state: web::Data<AppState>,
    expense_id: web::Path<Uuid>,
) -> impl Responder {
    match state
        .payment_use_cases
        .list_expense_payments(*expense_id)
        .await
    {
        Ok(payments) => HttpResponse::Ok().json(payments),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({"error": err})),
    }
}

#[utoipa::path(
    get,
    path = "/organizations/{organization_id}/payments",
    tag = "Payments",
    summary = "List all payments for an organization",
    params(("organization_id" = Uuid, Path, description = "Organization ID")),
    responses(
        (status = 200, description = "List of organization payments"),
        (status = 500, description = "Internal server error"),
    ),
    security(("bearer_auth" = []))
)]
#[get("/organizations/{organization_id}/payments")]
pub async fn list_organization_payments(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    organization_id: web::Path<Uuid>,
) -> impl Responder {
    if let Err(e) = user.verify_org_access(*organization_id) {
        return HttpResponse::Forbidden().json(serde_json::json!({"error": e}));
    }
    match state
        .payment_use_cases
        .list_organization_payments(*organization_id)
        .await
    {
        Ok(payments) => HttpResponse::Ok().json(payments),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({"error": err})),
    }
}

#[utoipa::path(
    get,
    path = "/payments/status/{status}",
    tag = "Payments",
    summary = "List payments by transaction status",
    params(("status" = String, Path, description = "Transaction status (pending, processing, requires_action, succeeded, failed, cancelled, refunded)")),
    responses(
        (status = 200, description = "List of payments with given status"),
        (status = 400, description = "Invalid status value"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error"),
    ),
    security(("bearer_auth" = []))
)]
#[get("/payments/status/{status}")]
pub async fn list_payments_by_status(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    status_str: web::Path<String>,
) -> impl Responder {
    let organization_id = match user.require_organization() {
        Ok(org_id) => org_id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": e.to_string()}))
        }
    };

    // Parse status string to TransactionStatus enum
    let status = match status_str.as_str() {
        "pending" => TransactionStatus::Pending,
        "processing" => TransactionStatus::Processing,
        "requires_action" => TransactionStatus::RequiresAction,
        "succeeded" => TransactionStatus::Succeeded,
        "failed" => TransactionStatus::Failed,
        "cancelled" => TransactionStatus::Cancelled,
        "refunded" => TransactionStatus::Refunded,
        _ => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid status. Must be one of: pending, processing, requires_action, succeeded, failed, cancelled, refunded"
            }))
        }
    };

    match state
        .payment_use_cases
        .list_payments_by_status(organization_id, status)
        .await
    {
        Ok(payments) => HttpResponse::Ok().json(payments),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({"error": err})),
    }
}

#[utoipa::path(
    get,
    path = "/payments/pending",
    tag = "Payments",
    summary = "List all pending payments for the organization",
    responses(
        (status = 200, description = "List of pending payments"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error"),
    ),
    security(("bearer_auth" = []))
)]
#[get("/payments/pending")]
pub async fn list_pending_payments(
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
        .payment_use_cases
        .list_pending_payments(organization_id)
        .await
    {
        Ok(payments) => HttpResponse::Ok().json(payments),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({"error": err})),
    }
}

#[utoipa::path(
    get,
    path = "/payments/failed",
    tag = "Payments",
    summary = "List all failed payments for the organization",
    responses(
        (status = 200, description = "List of failed payments"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error"),
    ),
    security(("bearer_auth" = []))
)]
#[get("/payments/failed")]
pub async fn list_failed_payments(
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
        .payment_use_cases
        .list_failed_payments(organization_id)
        .await
    {
        Ok(payments) => HttpResponse::Ok().json(payments),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({"error": err})),
    }
}

// ==================== Payment Status Update Endpoints ====================

#[utoipa::path(
    put,
    path = "/payments/{id}/processing",
    tag = "Payments",
    summary = "Mark a payment as processing",
    params(("id" = Uuid, Path, description = "Payment ID")),
    responses(
        (status = 200, description = "Payment marked as processing"),
        (status = 400, description = "Invalid state transition"),
        (status = 401, description = "Unauthorized"),
    ),
    security(("bearer_auth" = []))
)]
#[put("/payments/{id}/processing")]
pub async fn mark_payment_processing(
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

    match state.payment_use_cases.mark_processing(*id).await {
        Ok(payment) => {
            AuditLogEntry::new(
                AuditEventType::PaymentProcessing,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_resource("Payment", payment.id)
            .log();

            HttpResponse::Ok().json(payment)
        }
        Err(err) => HttpResponse::BadRequest().json(serde_json::json!({"error": err})),
    }
}

#[utoipa::path(
    put,
    path = "/payments/{id}/requires-action",
    tag = "Payments",
    summary = "Mark a payment as requiring action",
    params(("id" = Uuid, Path, description = "Payment ID")),
    responses(
        (status = 200, description = "Payment marked as requires action"),
        (status = 400, description = "Invalid state transition"),
        (status = 401, description = "Unauthorized"),
    ),
    security(("bearer_auth" = []))
)]
#[put("/payments/{id}/requires-action")]
pub async fn mark_payment_requires_action(
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

    match state.payment_use_cases.mark_requires_action(*id).await {
        Ok(payment) => {
            AuditLogEntry::new(
                AuditEventType::PaymentRequiresAction,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_resource("Payment", payment.id)
            .log();

            HttpResponse::Ok().json(payment)
        }
        Err(err) => HttpResponse::BadRequest().json(serde_json::json!({"error": err})),
    }
}

#[utoipa::path(
    put,
    path = "/payments/{id}/succeeded",
    tag = "Payments",
    summary = "Mark a payment as succeeded",
    params(("id" = Uuid, Path, description = "Payment ID")),
    responses(
        (status = 200, description = "Payment marked as succeeded"),
        (status = 400, description = "Invalid state transition"),
        (status = 401, description = "Unauthorized"),
    ),
    security(("bearer_auth" = []))
)]
#[put("/payments/{id}/succeeded")]
pub async fn mark_payment_succeeded(
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

    match state.payment_use_cases.mark_succeeded(*id).await {
        Ok(payment) => {
            AuditLogEntry::new(
                AuditEventType::PaymentSucceeded,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_resource("Payment", payment.id)
            .log();

            HttpResponse::Ok().json(payment)
        }
        Err(err) => HttpResponse::BadRequest().json(serde_json::json!({"error": err})),
    }
}

#[utoipa::path(
    put,
    path = "/payments/{id}/failed",
    tag = "Payments",
    summary = "Mark a payment as failed",
    params(("id" = Uuid, Path, description = "Payment ID")),
    request_body = inline(serde_json::Value),
    responses(
        (status = 200, description = "Payment marked as failed"),
        (status = 400, description = "Invalid state transition"),
        (status = 401, description = "Unauthorized"),
    ),
    security(("bearer_auth" = []))
)]
#[put("/payments/{id}/failed")]
pub async fn mark_payment_failed(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    id: web::Path<Uuid>,
    request: web::Json<serde_json::Value>,
) -> impl Responder {
    let organization_id = match user.require_organization() {
        Ok(org_id) => org_id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": e.to_string()}))
        }
    };

    let reason = request
        .get("reason")
        .and_then(|v| v.as_str())
        .unwrap_or("Unknown failure reason")
        .to_string();

    match state.payment_use_cases.mark_failed(*id, reason).await {
        Ok(payment) => {
            AuditLogEntry::new(
                AuditEventType::PaymentFailed,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_resource("Payment", payment.id)
            .log();

            HttpResponse::Ok().json(payment)
        }
        Err(err) => HttpResponse::BadRequest().json(serde_json::json!({"error": err})),
    }
}

#[utoipa::path(
    put,
    path = "/payments/{id}/cancelled",
    tag = "Payments",
    summary = "Mark a payment as cancelled",
    params(("id" = Uuid, Path, description = "Payment ID")),
    responses(
        (status = 200, description = "Payment marked as cancelled"),
        (status = 400, description = "Invalid state transition"),
        (status = 401, description = "Unauthorized"),
    ),
    security(("bearer_auth" = []))
)]
#[put("/payments/{id}/cancelled")]
pub async fn mark_payment_cancelled(
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

    match state.payment_use_cases.mark_cancelled(*id).await {
        Ok(payment) => {
            AuditLogEntry::new(
                AuditEventType::PaymentCancelled,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_resource("Payment", payment.id)
            .log();

            HttpResponse::Ok().json(payment)
        }
        Err(err) => HttpResponse::BadRequest().json(serde_json::json!({"error": err})),
    }
}

#[utoipa::path(
    post,
    path = "/payments/{id}/refund",
    tag = "Payments",
    summary = "Refund a payment (partial or full)",
    params(("id" = Uuid, Path, description = "Payment ID")),
    request_body = RefundPaymentRequest,
    responses(
        (status = 200, description = "Payment refunded"),
        (status = 400, description = "Refund not allowed or exceeds payment amount"),
        (status = 401, description = "Unauthorized"),
    ),
    security(("bearer_auth" = []))
)]
#[post("/payments/{id}/refund")]
pub async fn refund_payment(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    id: web::Path<Uuid>,
    request: web::Json<RefundPaymentRequest>,
) -> impl Responder {
    let organization_id = match user.require_organization() {
        Ok(org_id) => org_id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": e.to_string()}))
        }
    };

    match state
        .payment_use_cases
        .refund_payment(*id, request.into_inner())
        .await
    {
        Ok(payment) => {
            AuditLogEntry::new(
                AuditEventType::PaymentRefunded,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_resource("Payment", payment.id)
            .log();

            HttpResponse::Ok().json(payment)
        }
        Err(err) => {
            AuditLogEntry::new(
                AuditEventType::PaymentRefunded,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_error(err.clone())
            .log();

            HttpResponse::BadRequest().json(serde_json::json!({"error": err}))
        }
    }
}

#[utoipa::path(
    delete,
    path = "/payments/{id}",
    tag = "Payments",
    summary = "Delete a payment",
    params(("id" = Uuid, Path, description = "Payment ID")),
    responses(
        (status = 204, description = "Payment deleted"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Payment not found"),
        (status = 500, description = "Internal server error"),
    ),
    security(("bearer_auth" = []))
)]
#[delete("/payments/{id}")]
pub async fn delete_payment(
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

    match state.payment_use_cases.delete_payment(*id).await {
        Ok(true) => {
            AuditLogEntry::new(
                AuditEventType::PaymentDeleted,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_resource("Payment", *id)
            .log();

            HttpResponse::NoContent().finish()
        }
        Ok(false) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Payment not found"
        })),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({"error": err})),
    }
}

// ==================== Payment Statistics Endpoints ====================

#[utoipa::path(
    get,
    path = "/owners/{owner_id}/payments/stats",
    tag = "Payments",
    summary = "Get payment statistics for an owner",
    params(("owner_id" = Uuid, Path, description = "Owner ID")),
    responses(
        (status = 200, description = "Owner payment statistics"),
        (status = 500, description = "Internal server error"),
    ),
    security(("bearer_auth" = []))
)]
#[get("/owners/{owner_id}/payments/stats")]
pub async fn get_owner_payment_stats(
    state: web::Data<AppState>,
    owner_id: web::Path<Uuid>,
) -> impl Responder {
    match state
        .payment_use_cases
        .get_owner_payment_stats(*owner_id)
        .await
    {
        Ok(stats) => HttpResponse::Ok().json(stats),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({"error": err})),
    }
}

#[utoipa::path(
    get,
    path = "/buildings/{building_id}/payments/stats",
    tag = "Payments",
    summary = "Get payment statistics for a building",
    params(("building_id" = Uuid, Path, description = "Building ID")),
    responses(
        (status = 200, description = "Building payment statistics"),
        (status = 500, description = "Internal server error"),
    ),
    security(("bearer_auth" = []))
)]
#[get("/buildings/{building_id}/payments/stats")]
pub async fn get_building_payment_stats(
    state: web::Data<AppState>,
    building_id: web::Path<Uuid>,
) -> impl Responder {
    match state
        .payment_use_cases
        .get_building_payment_stats(*building_id)
        .await
    {
        Ok(stats) => HttpResponse::Ok().json(stats),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({"error": err})),
    }
}

#[utoipa::path(
    get,
    path = "/expenses/{expense_id}/payments/total",
    tag = "Payments",
    summary = "Get total amount paid for an expense",
    params(("expense_id" = Uuid, Path, description = "Expense ID")),
    responses(
        (status = 200, description = "Total paid amount in cents"),
        (status = 500, description = "Internal server error"),
    ),
    security(("bearer_auth" = []))
)]
#[get("/expenses/{expense_id}/payments/total")]
pub async fn get_expense_total_paid(
    state: web::Data<AppState>,
    expense_id: web::Path<Uuid>,
) -> impl Responder {
    match state
        .payment_use_cases
        .get_total_paid_for_expense(*expense_id)
        .await
    {
        Ok(total) => HttpResponse::Ok().json(serde_json::json!({
            "expense_id": *expense_id,
            "total_paid_cents": total
        })),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({"error": err})),
    }
}

#[utoipa::path(
    get,
    path = "/owners/{owner_id}/payments/total",
    tag = "Payments",
    summary = "Get total amount paid by an owner",
    params(("owner_id" = Uuid, Path, description = "Owner ID")),
    responses(
        (status = 200, description = "Total paid amount in cents"),
        (status = 500, description = "Internal server error"),
    ),
    security(("bearer_auth" = []))
)]
#[get("/owners/{owner_id}/payments/total")]
pub async fn get_owner_total_paid(
    state: web::Data<AppState>,
    owner_id: web::Path<Uuid>,
) -> impl Responder {
    match state
        .payment_use_cases
        .get_total_paid_by_owner(*owner_id)
        .await
    {
        Ok(total) => HttpResponse::Ok().json(serde_json::json!({
            "owner_id": *owner_id,
            "total_paid_cents": total
        })),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({"error": err})),
    }
}

#[utoipa::path(
    get,
    path = "/buildings/{building_id}/payments/total",
    tag = "Payments",
    summary = "Get total amount paid for a building",
    params(("building_id" = Uuid, Path, description = "Building ID")),
    responses(
        (status = 200, description = "Total paid amount in cents"),
        (status = 500, description = "Internal server error"),
    ),
    security(("bearer_auth" = []))
)]
#[get("/buildings/{building_id}/payments/total")]
pub async fn get_building_total_paid(
    state: web::Data<AppState>,
    building_id: web::Path<Uuid>,
) -> impl Responder {
    match state
        .payment_use_cases
        .get_total_paid_for_building(*building_id)
        .await
    {
        Ok(total) => HttpResponse::Ok().json(serde_json::json!({
            "building_id": *building_id,
            "total_paid_cents": total
        })),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({"error": err})),
    }
}
