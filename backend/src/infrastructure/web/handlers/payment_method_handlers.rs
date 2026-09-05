use crate::application::dto::{
    CreatePaymentMethodRequest, PaymentMethodResponse, UpdatePaymentMethodRequest,
};
use crate::domain::entities::payment_method::PaymentMethodType;
use crate::infrastructure::audit::{AuditEventType, AuditLogEntry};
use crate::infrastructure::web::{AppState, AuthenticatedUser};
use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use uuid::Uuid;

// ==================== Payment Method CRUD Endpoints ====================

#[utoipa::path(
    post,
    path = "/payment-methods",
    tag = "PaymentMethods",
    summary = "Enregistrer un moyen de paiement pour un copropriétaire",
    request_body = CreatePaymentMethodRequest,
    responses(
        (status = 201, description = "Moyen de paiement créé", body = PaymentMethodResponse),
        (status = 400, description = "Requête invalide — champs manquants ou moyen déjà enregistré"),
        (status = 401, description = "L'utilisateur n'appartient à aucune organisation"),
    ),
    security(("bearer_auth" = []))
)]
#[post("/payment-methods")]
pub async fn create_payment_method(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    request: web::Json<CreatePaymentMethodRequest>,
) -> impl Responder {
    let organization_id = match user.require_organization() {
        Ok(org_id) => org_id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": e.to_string()}))
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

#[utoipa::path(
    get,
    path = "/payment-methods/{id}",
    tag = "PaymentMethods",
    summary = "Récupérer un moyen de paiement",
    params(("id" = Uuid, Path, description = "Identifiant du moyen de paiement")),
    responses(
        (status = 200, description = "Moyen de paiement", body = PaymentMethodResponse),
        (status = 404, description = "Introuvable"),
    ),
    security(("bearer_auth" = []))
)]
#[get("/payment-methods/{id}")]
pub async fn get_payment_method(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    id: web::Path<Uuid>,
) -> impl Responder {
    match state.payment_method_use_cases.get_payment_method(*id).await {
        Ok(Some(method)) => {
            // Verify organization access
            if let Err(err) = user.verify_org_access(method.organization_id) {
                return HttpResponse::Forbidden().json(serde_json::json!({"error": err}));
            }
            HttpResponse::Ok().json(method)
        }
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Payment method not found"
        })),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({"error": err})),
    }
}

#[utoipa::path(
    get,
    path = "/payment-methods/stripe/{stripe_payment_method_id}",
    tag = "PaymentMethods",
    summary = "Récupérer un moyen de paiement par son identifiant Stripe",
    params(("stripe_payment_method_id" = String, Path, description = "Identifiant Stripe")),
    responses(
        (status = 200, description = "Moyen de paiement", body = PaymentMethodResponse),
        (status = 404, description = "Introuvable"),
    ),
    security(("bearer_auth" = []))
)]
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
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({"error": err})),
    }
}

#[utoipa::path(
    get,
    path = "/owners/{owner_id}/payment-methods",
    tag = "PaymentMethods",
    summary = "Lister les moyens de paiement d'un copropriétaire",
    params(("owner_id" = Uuid, Path, description = "Identifiant du copropriétaire")),
    responses(
        (status = 200, description = "Liste des moyens de paiement", body = Vec<PaymentMethodResponse>),
    ),
    security(("bearer_auth" = []))
)]
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
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({"error": err})),
    }
}

#[utoipa::path(
    get,
    path = "/owners/{owner_id}/payment-methods/active",
    tag = "PaymentMethods",
    summary = "Lister les moyens de paiement actifs d'un copropriétaire",
    params(("owner_id" = Uuid, Path, description = "Identifiant du copropriétaire")),
    responses(
        (status = 200, description = "Liste des moyens actifs", body = Vec<PaymentMethodResponse>),
    ),
    security(("bearer_auth" = []))
)]
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
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({"error": err})),
    }
}

#[utoipa::path(
    get,
    path = "/owners/{owner_id}/payment-methods/default",
    tag = "PaymentMethods",
    summary = "Récupérer le moyen de paiement par défaut d'un copropriétaire",
    params(("owner_id" = Uuid, Path, description = "Identifiant du copropriétaire")),
    responses(
        (status = 200, description = "Moyen par défaut", body = PaymentMethodResponse),
        (status = 404, description = "Aucun moyen par défaut"),
    ),
    security(("bearer_auth" = []))
)]
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
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({"error": err})),
    }
}

#[utoipa::path(
    get,
    path = "/organizations/{organization_id}/payment-methods",
    tag = "PaymentMethods",
    summary = "Lister les moyens de paiement d'une organisation",
    params(("organization_id" = Uuid, Path, description = "Identifiant de l'organisation")),
    responses(
        (status = 200, description = "Liste des moyens de paiement", body = Vec<PaymentMethodResponse>),
    ),
    security(("bearer_auth" = []))
)]
#[get("/organizations/{organization_id}/payment-methods")]
pub async fn list_organization_payment_methods(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    organization_id: web::Path<Uuid>,
) -> impl Responder {
    if let Err(e) = user.verify_org_access(*organization_id) {
        return HttpResponse::Forbidden().json(serde_json::json!({"error": e}));
    }
    match state
        .payment_method_use_cases
        .list_organization_payment_methods(*organization_id)
        .await
    {
        Ok(methods) => HttpResponse::Ok().json(methods),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({"error": err})),
    }
}

#[utoipa::path(
    get,
    path = "/owners/{owner_id}/payment-methods/type/{method_type}",
    tag = "PaymentMethods",
    summary = "Lister les moyens de paiement d'un copropriétaire par type",
    params(
        ("owner_id" = Uuid, Path, description = "Identifiant du copropriétaire"),
        ("method_type" = String, Path, description = "Type de moyen : card, sepa_debit, bancontact"),
    ),
    responses(
        (status = 200, description = "Liste filtrée", body = Vec<PaymentMethodResponse>),
        (status = 400, description = "Type de moyen inconnu"),
    ),
    security(("bearer_auth" = []))
)]
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
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({"error": err})),
    }
}

#[utoipa::path(
    put,
    path = "/payment-methods/{id}",
    tag = "PaymentMethods",
    summary = "Mettre à jour un moyen de paiement",
    params(("id" = Uuid, Path, description = "Identifiant du moyen de paiement")),
    request_body = UpdatePaymentMethodRequest,
    responses(
        (status = 200, description = "Moyen mis à jour", body = PaymentMethodResponse),
        (status = 400, description = "Requête invalide"),
        (status = 404, description = "Introuvable"),
    ),
    security(("bearer_auth" = []))
)]
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
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": e.to_string()}))
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

#[utoipa::path(
    put,
    path = "/payment-methods/{id}/set-default",
    tag = "PaymentMethods",
    summary = "Désigner un moyen de paiement comme moyen par défaut",
    params(("id" = Uuid, Path, description = "Identifiant du moyen de paiement")),
    responses(
        (status = 200, description = "Moyen désigné par défaut", body = PaymentMethodResponse),
        (status = 404, description = "Introuvable"),
    ),
    security(("bearer_auth" = []))
)]
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
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": e.to_string()}))
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

#[utoipa::path(
    put,
    path = "/payment-methods/{id}/deactivate",
    tag = "PaymentMethods",
    summary = "Désactiver un moyen de paiement",
    params(("id" = Uuid, Path, description = "Identifiant du moyen de paiement")),
    responses(
        (status = 200, description = "Moyen désactivé", body = PaymentMethodResponse),
        (status = 404, description = "Introuvable"),
    ),
    security(("bearer_auth" = []))
)]
#[put("/payment-methods/{id}/deactivate")]
pub async fn deactivate_payment_method(
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

#[utoipa::path(
    put,
    path = "/payment-methods/{id}/reactivate",
    tag = "PaymentMethods",
    summary = "Réactiver un moyen de paiement",
    params(("id" = Uuid, Path, description = "Identifiant du moyen de paiement")),
    responses(
        (status = 200, description = "Moyen réactivé", body = PaymentMethodResponse),
        (status = 404, description = "Introuvable"),
    ),
    security(("bearer_auth" = []))
)]
#[put("/payment-methods/{id}/reactivate")]
pub async fn reactivate_payment_method(
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

#[utoipa::path(
    delete,
    path = "/payment-methods/{id}",
    tag = "PaymentMethods",
    summary = "Supprimer un moyen de paiement",
    params(("id" = Uuid, Path, description = "Identifiant du moyen de paiement")),
    responses(
        (status = 204, description = "Moyen supprimé"),
        (status = 404, description = "Introuvable"),
    ),
    security(("bearer_auth" = []))
)]
#[delete("/payment-methods/{id}")]
pub async fn delete_payment_method(
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

#[utoipa::path(
    get,
    path = "/owners/{owner_id}/payment-methods/count",
    tag = "PaymentMethods",
    summary = "Compter les moyens de paiement actifs d'un copropriétaire",
    params(("owner_id" = Uuid, Path, description = "Identifiant du copropriétaire")),
    responses(
        (status = 200, description = "Nombre de moyens actifs"),
    ),
    security(("bearer_auth" = []))
)]
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
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({"error": err})),
    }
}

#[utoipa::path(
    get,
    path = "/owners/{owner_id}/payment-methods/has-active",
    tag = "PaymentMethods",
    summary = "Indiquer si un copropriétaire a au moins un moyen de paiement actif",
    params(("owner_id" = Uuid, Path, description = "Identifiant du copropriétaire")),
    responses(
        (status = 200, description = "Présence d'un moyen actif"),
    ),
    security(("bearer_auth" = []))
)]
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
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({"error": err})),
    }
}
