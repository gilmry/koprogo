use crate::application::dto::{
    CreatePaymentMethodRequest, UpdatePaymentMethodRequest,
};
use crate::domain::entities::payment_method::PaymentMethodType;
use crate::infrastructure::audit::{AuditEventType, AuditLogEntry};
use crate::infrastructure::web::{AppState, AuthenticatedUser};
use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use uuid::Uuid;

// ==================== Payment Method CRUD Endpoints ====================

#[post("/payment-methods")]
pub async fn create_payment_method(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    request: web::Json<CreatePaymentMethodRequest>,
) -> impl Responder {
    let organization_id = match user.require_organization() {
        Ok(org_id) => org_id,
        Err(e) => {
            return HttpResponse::Unauthorized()
                .json(serde_json::json!({"error": e.to_string()}))
        }
    };

    match state
        .payment_method_use_cases
        .create_payment_method(organization_id, request.into_inner())
        .await
    {
        Ok(payment_method) => {
            AuditLogEntry::new(
                AuditEventType::PaymentMethodCreated,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_resource("PaymentMethod", payment_method.id)
            .log();

            HttpResponse::Created().json(payment_method)
        }
        Err(err) => {
            AuditLogEntry::new(
                AuditEventType::PaymentMethodCreated,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_error(err.clone())
            .log();

            HttpResponse::BadRequest().json(serde_json::json!({"error": err}))
        }
    }
}

#[get("/payment-methods/{id}")]
pub async fn get_payment_method(
    state: web::Data<AppState>,
    id: web::Path<Uuid>,
) -> impl Responder {
    match state.payment_method_use_cases.get_payment_method(*id).await {
        Ok(Some(method)) => HttpResponse::Ok().json(method),
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Payment method not found"
        })),
        Err(err) => {
            HttpResponse::InternalServerError().json(serde_json::json!({"error": err}))
        }
    }
}

#[get("/payment-methods/stripe/{stripe_payment_method_id}")]
pub async fn get_payment_method_by_stripe_id(
    state: web::Data<AppState>,
    stripe_payment_method_id: web::Path<String>,
) -> impl Responder {
    match state
        .payment_method_use_cases
        .get_payment_method_by_stripe_id(&stripe_payment_method_id)
        .await
    {
        Ok(Some(method)) => HttpResponse::Ok().json(method),
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Payment method not found"
        })),
        Err(err) => {
            HttpResponse::InternalServerError().json(serde_json::json!({"error": err}))
        }
    }
}

#[get("/owners/{owner_id}/payment-methods")]
pub async fn list_owner_payment_methods(
    state: web::Data<AppState>,
    owner_id: web::Path<Uuid>,
) -> impl Responder {
    match state
        .payment_method_use_cases
        .list_owner_payment_methods(*owner_id)
        .await
    {
        Ok(methods) => HttpResponse::Ok().json(methods),
        Err(err) => {
            HttpResponse::InternalServerError().json(serde_json::json!({"error": err}))
        }
    }
}

#[get("/owners/{owner_id}/payment-methods/active")]
pub async fn list_active_owner_payment_methods(
    state: web::Data<AppState>,
    owner_id: web::Path<Uuid>,
) -> impl Responder {
    match state
        .payment_method_use_cases
        .list_active_owner_payment_methods(*owner_id)
        .await
    {
        Ok(methods) => HttpResponse::Ok().json(methods),
        Err(err) => {
            HttpResponse::InternalServerError().json(serde_json::json!({"error": err}))
        }
    }
}

#[get("/owners/{owner_id}/payment-methods/default")]
pub async fn get_default_payment_method(
    state: web::Data<AppState>,
    owner_id: web::Path<Uuid>,
) -> impl Responder {
    match state
        .payment_method_use_cases
        .get_default_payment_method(*owner_id)
        .await
    {
        Ok(Some(method)) => HttpResponse::Ok().json(method),
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "No default payment method found for owner"
        })),
        Err(err) => {
            HttpResponse::InternalServerError().json(serde_json::json!({"error": err}))
        }
    }
}

#[get("/organizations/{organization_id}/payment-methods")]
pub async fn list_organization_payment_methods(
    state: web::Data<AppState>,
    organization_id: web::Path<Uuid>,
) -> impl Responder {
    match state
        .payment_method_use_cases
        .list_organization_payment_methods(*organization_id)
        .await
    {
        Ok(methods) => HttpResponse::Ok().json(methods),
        Err(err) => {
            HttpResponse::InternalServerError().json(serde_json::json!({"error": err}))
        }
    }
}

#[get("/owners/{owner_id}/payment-methods/type/{method_type}")]
pub async fn list_payment_methods_by_type(
    state: web::Data<AppState>,
    path: web::Path<(Uuid, String)>,
) -> impl Responder {
    let (owner_id, method_type_str) = path.into_inner();

    // Parse method type string to enum
    let method_type = match method_type_str.as_str() {
        "card" => PaymentMethodType::Card,
        "sepa_debit" => PaymentMethodType::SepaDebit,
        _ => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid payment method type. Must be one of: card, sepa_debit"
            }))
        }
    };

    match state
        .payment_method_use_cases
        .list_payment_methods_by_type(owner_id, method_type)
        .await
    {
        Ok(methods) => HttpResponse::Ok().json(methods),
        Err(err) => {
            HttpResponse::InternalServerError().json(serde_json::json!({"error": err}))
        }
    }
}

#[put("/payment-methods/{id}")]
pub async fn update_payment_method(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    id: web::Path<Uuid>,
    request: web::Json<UpdatePaymentMethodRequest>,
) -> impl Responder {
    let organization_id = match user.require_organization() {
        Ok(org_id) => org_id,
        Err(e) => {
            return HttpResponse::Unauthorized()
                .json(serde_json::json!({"error": e.to_string()}))
        }
    };

    match state
        .payment_method_use_cases
        .update_payment_method(*id, request.into_inner())
        .await
    {
        Ok(method) => {
            AuditLogEntry::new(
                AuditEventType::PaymentMethodUpdated,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_resource("PaymentMethod", method.id)
            .log();

            HttpResponse::Ok().json(method)
        }
        Err(err) => HttpResponse::BadRequest().json(serde_json::json!({"error": err})),
    }
}

#[put("/payment-methods/{id}/set-default")]
pub async fn set_payment_method_as_default(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    id: web::Path<Uuid>,
    owner_id_json: web::Json<serde_json::Value>,
) -> impl Responder {
    let organization_id = match user.require_organization() {
        Ok(org_id) => org_id,
        Err(e) => {
            return HttpResponse::Unauthorized()
                .json(serde_json::json!({"error": e.to_string()}))
        }
    };

    let owner_id = match owner_id_json.get("owner_id").and_then(|v| v.as_str()) {
        Some(id_str) => match Uuid::parse_str(id_str) {
            Ok(id) => id,
            Err(_) => {
                return HttpResponse::BadRequest().json(serde_json::json!({
                    "error": "Invalid owner_id format"
                }))
            }
        },
        None => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "owner_id is required"
            }))
        }
    };

    match state
        .payment_method_use_cases
        .set_as_default(*id, owner_id)
        .await
    {
        Ok(method) => {
            AuditLogEntry::new(
                AuditEventType::PaymentMethodSetDefault,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_resource("PaymentMethod", method.id)
            .log();

            HttpResponse::Ok().json(method)
        }
        Err(err) => HttpResponse::BadRequest().json(serde_json::json!({"error": err})),
    }
}

#[put("/payment-methods/{id}/deactivate")]
pub async fn deactivate_payment_method(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    id: web::Path<Uuid>,
) -> impl Responder {
    let organization_id = match user.require_organization() {
        Ok(org_id) => org_id,
        Err(e) => {
            return HttpResponse::Unauthorized()
                .json(serde_json::json!({"error": e.to_string()}))
        }
    };

    match state
        .payment_method_use_cases
        .deactivate_payment_method(*id)
        .await
    {
        Ok(method) => {
            AuditLogEntry::new(
                AuditEventType::PaymentMethodDeactivated,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_resource("PaymentMethod", method.id)
            .log();

            HttpResponse::Ok().json(method)
        }
        Err(err) => HttpResponse::BadRequest().json(serde_json::json!({"error": err})),
    }
}

#[put("/payment-methods/{id}/reactivate")]
pub async fn reactivate_payment_method(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    id: web::Path<Uuid>,
) -> impl Responder {
    let organization_id = match user.require_organization() {
        Ok(org_id) => org_id,
        Err(e) => {
            return HttpResponse::Unauthorized()
                .json(serde_json::json!({"error": e.to_string()}))
        }
    };

    match state
        .payment_method_use_cases
        .reactivate_payment_method(*id)
        .await
    {
        Ok(method) => {
            AuditLogEntry::new(
                AuditEventType::PaymentMethodReactivated,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_resource("PaymentMethod", method.id)
            .log();

            HttpResponse::Ok().json(method)
        }
        Err(err) => HttpResponse::BadRequest().json(serde_json::json!({"error": err})),
    }
}

#[delete("/payment-methods/{id}")]
pub async fn delete_payment_method(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    id: web::Path<Uuid>,
) -> impl Responder {
    let organization_id = match user.require_organization() {
        Ok(org_id) => org_id,
        Err(e) => {
            return HttpResponse::Unauthorized()
                .json(serde_json::json!({"error": e.to_string()}))
        }
    };

    match state
        .payment_method_use_cases
        .delete_payment_method(*id)
        .await
    {
        Ok(true) => {
            AuditLogEntry::new(
                AuditEventType::PaymentMethodDeleted,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_resource("PaymentMethod", *id)
            .log();

            HttpResponse::NoContent().finish()
        }
        Ok(false) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Payment method not found"
        })),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({"error": err})),
    }
}

// ==================== Payment Method Statistics Endpoints ====================

#[get("/owners/{owner_id}/payment-methods/count")]
pub async fn count_active_payment_methods(
    state: web::Data<AppState>,
    owner_id: web::Path<Uuid>,
) -> impl Responder {
    match state
        .payment_method_use_cases
        .count_active_payment_methods(*owner_id)
        .await
    {
        Ok(count) => HttpResponse::Ok().json(serde_json::json!({
            "owner_id": *owner_id,
            "active_payment_methods_count": count
        })),
        Err(err) => {
            HttpResponse::InternalServerError().json(serde_json::json!({"error": err}))
        }
    }
}

#[get("/owners/{owner_id}/payment-methods/has-active")]
pub async fn has_active_payment_methods(
    state: web::Data<AppState>,
    owner_id: web::Path<Uuid>,
) -> impl Responder {
    match state
        .payment_method_use_cases
        .has_active_payment_methods(*owner_id)
        .await
    {
        Ok(has_active) => HttpResponse::Ok().json(serde_json::json!({
            "owner_id": *owner_id,
            "has_active_payment_methods": has_active
        })),
        Err(err) => {
            HttpResponse::InternalServerError().json(serde_json::json!({"error": err}))
        }
    }
}
