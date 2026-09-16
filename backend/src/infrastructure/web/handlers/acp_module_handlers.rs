//! Handlers Actix du registre de modules — Story 5.1 (#585), ADR-0015.
//!
//! Endpoints :
//! - GET `/acps/{id}/modules`                   : modules actifs (portée ACP)
//! - PUT `/acps/{id}/modules/{module}/enable`   : allumer (admin + portée)
//! - PUT `/acps/{id}/modules/{module}/disable`  : éteindre (admin + portée)
//!
//! Les chemins ne sont pas choisis ici : ils sont **imposés par la moitié
//! frontend déjà livrée** (`modules.ts`, `OnboardingWizard.svelte`), écrite
//! en attendant cette story. Les changer casserait un client en production.
//!
//! Le mapping `AuthenticatedUser → AcpCaller` est **réutilisé** depuis
//! `acp_handlers` plutôt que recopié : deux mappings de rôles qui dérivent
//! l'un de l'autre est une faille, pas une duplication anodine.

use actix_web::{get, put, web, HttpResponse, Responder, ResponseError};
use uuid::Uuid;

use crate::application::dto::EnabledModulesResponseDto;
use crate::application::use_cases::module_registry_use_cases::ModuleRegistryUseCases;
use crate::infrastructure::audit::{AuditEventType, AuditLogEntry};
use crate::infrastructure::web::handlers::acp_handlers::caller_from_user;
use crate::infrastructure::web::{AppState, AuthenticatedUser};

#[utoipa::path(
    get,
    path = "/acps/{id}/modules",
    tag = "Acps",
    summary = "Modules actifs d'une ACP (portée ACP)",
    params(("id" = Uuid, Path, description = "ACP UUID")),
    responses(
        (status = 200, description = "Modules actifs", body = crate::application::dto::EnabledModulesResponseDto),
        (status = 403, description = "Hors portée"),
    ),
    security(("bearer_auth" = []))
)]
#[get("/acps/{id}/modules")]
pub async fn list_acp_modules(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    id: web::Path<Uuid>,
) -> impl Responder {
    let acp_id = *id;
    let caller = caller_from_user(&user);
    match state
        .module_registry_use_cases
        .list_enabled(&caller, acp_id)
        .await
    {
        Ok(modules) => HttpResponse::Ok().json(EnabledModulesResponseDto {
            acp_id: acp_id.to_string(),
            modules,
        }),
        Err(err) => err.error_response(),
    }
}

#[utoipa::path(
    put,
    path = "/acps/{id}/modules/{module}/enable",
    tag = "Acps",
    summary = "Allumer un module pour une ACP (admin + portée)",
    params(
        ("id" = Uuid, Path, description = "ACP UUID"),
        ("module" = String, Path, description = "Nom du module (ADR-0015)"),
    ),
    responses(
        (status = 204, description = "Module actif"),
        (status = 403, description = "Hors portée ou droits insuffisants"),
        (status = 422, description = "Nom de module inconnu"),
    ),
    security(("bearer_auth" = []))
)]
#[put("/acps/{id}/modules/{module}/enable")]
pub async fn enable_acp_module(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    chemin: web::Path<(Uuid, String)>,
) -> impl Responder {
    let (acp_id, nom) = chemin.into_inner();
    let module = match ModuleRegistryUseCases::reconnaitre(&nom) {
        Ok(m) => m,
        Err(err) => return err.error_response(),
    };
    let caller = caller_from_user(&user);
    let issue = state
        .module_registry_use_cases
        .enable(&caller, acp_id, module)
        .await;
    journaliser(AuditEventType::AcpModuleEnabled, &user, acp_id, &issue);
    match issue {
        // 204 et non 200 : le client n'a pas besoin du corps, et le rendre
        // obligerait à relire l'état pour l'afficher.
        Ok(()) => HttpResponse::NoContent().finish(),
        Err(err) => err.error_response(),
    }
}

#[utoipa::path(
    put,
    path = "/acps/{id}/modules/{module}/disable",
    tag = "Acps",
    summary = "Éteindre un module pour une ACP (admin + portée)",
    params(
        ("id" = Uuid, Path, description = "ACP UUID"),
        ("module" = String, Path, description = "Nom du module (ADR-0015)"),
    ),
    responses(
        (status = 204, description = "Module éteint"),
        (status = 403, description = "Hors portée, droits insuffisants, ou module toujours actif"),
        (status = 422, description = "Nom de module inconnu"),
    ),
    security(("bearer_auth" = []))
)]
#[put("/acps/{id}/modules/{module}/disable")]
pub async fn disable_acp_module(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    chemin: web::Path<(Uuid, String)>,
) -> impl Responder {
    let (acp_id, nom) = chemin.into_inner();
    let module = match ModuleRegistryUseCases::reconnaitre(&nom) {
        Ok(m) => m,
        Err(err) => return err.error_response(),
    };
    let caller = caller_from_user(&user);
    let issue = state
        .module_registry_use_cases
        .disable(&caller, acp_id, module)
        .await;
    journaliser(AuditEventType::AcpModuleDisabled, &user, acp_id, &issue);
    match issue {
        Ok(()) => HttpResponse::NoContent().finish(),
        Err(err) => err.error_response(),
    }
}

/// Consigne la tentative, réussie **ou non**.
///
/// Éteindre un module fait disparaître des écrans pour tous les utilisateurs
/// de l'ACP : la tentative refusée est aussi intéressante que celle qui
/// aboutit (pattern des handlers ACP, traçabilité INV-24).
fn journaliser(
    evenement: AuditEventType,
    user: &AuthenticatedUser,
    acp_id: Uuid,
    issue: &Result<(), crate::application::error::AppError>,
) {
    let entree = AuditLogEntry::new(evenement, Some(user.user_id), user.organization_id)
        .with_resource("Acp", acp_id);
    match issue {
        Ok(()) => entree.log(),
        Err(err) => entree.with_error(err.to_string()).log(),
    }
}
