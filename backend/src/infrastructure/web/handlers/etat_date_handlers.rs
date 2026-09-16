use crate::application::dto::{
    CreateEtatDateRequest, PageRequest, PageResponse, UpdateEtatDateAdditionalDataRequest,
    UpdateEtatDateFinancialRequest,
};
use crate::domain::entities::EtatDateStatus;
use crate::infrastructure::audit::{AuditEventType, AuditLogEntry};
use crate::infrastructure::web::handlers::conformity_response::try_build_conformity_response;
use crate::infrastructure::web::middleware::scope_guard::verify_building_org_access;
use crate::infrastructure::web::{AppState, AuthenticatedUser};
use actix_web::{delete, get, post, put, web, HttpResponse, Responder, ResponseError};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct EtatDateListQuery {
    #[serde(default = "default_page")]
    pub page: i64,
    #[serde(default = "default_per_page")]
    pub per_page: i64,
    pub status: Option<String>,
}

fn default_page() -> i64 {
    1
}
fn default_per_page() -> i64 {
    10
}

/// Cloisonne un état daté AVANT de le muter (#864).
///
/// ── Pourquoi `verify_` et pas un nom français ─────────────────────────────
///
/// Ce helper s'appelait `cloisonner_*` à sa première écriture. Le cliquet de
/// #864 est resté à 95 : son détecteur cherche les idiomes par lesquels CE
/// dépôt refuse un accès — `verify_`, `scope_guard`, `Forbidden`,
/// `require_organization` — et `cloisonner_` n'en est pas un. Dix trous
/// venaient d'être bouchés, et l'instrument ne le voyait pas.
///
/// Deux sorties possibles : allonger la liste du détecteur, ou porter le nom
/// que le dépôt emploie déjà (`verify_org_access`, `verify_acp_org_access`,
/// `verify_building_org_access`). La première aurait fait tomber le compteur
/// de dix par une modification de sa DÉFINITION, ce qui est précisément le
/// geste que la méthode interdit. La seconde corrige une incohérence de
/// nommage que je venais d'introduire, et la dette tombe à 85 parce que le
/// travail a été fait.
///
/// ── Ce que ces cinq routes laissaient passer ──────────────────────────────
///
/// `mark_in_progress`, `mark_generated`, `mark_delivered`,
/// `update_financial_data` et `update_additional_data` prenaient
/// `AuthenticatedUser` et ne s'en servaient que pour journaliser après coup.
/// `get_etat_date`, dans ce même fichier, cloisonne correctement : le
/// contrôle existait, il manquait sur les écritures.
///
/// ── Pourquoi c'est plus lourd qu'un budget ────────────────────────────────
///
/// L'état daté est une pièce **légale** : Art. 3.89 § 5, l'information que le
/// syndic doit au notaire lors d'une mutation. Marquer « délivré » un état
/// daté d'un autre cabinet fait courir les délais sur un dossier qu'on ne
/// gère pas ; en altérer les données financières fausse le décompte des
/// arriérés que l'acquéreur reprend.
///
/// Rend `Some(réponse)` quand l'appel doit être refusé, `None` sinon.
async fn verify_etat_date_org_access(
    state: &web::Data<AppState>,
    user: &AuthenticatedUser,
    id: Uuid,
) -> Option<HttpResponse> {
    match state.etat_date_use_cases.get_etat_date(id).await {
        Ok(Some(etat_date)) => match user.verify_org_access(etat_date.organization_id) {
            Ok(()) => None,
            Err(err) => Some(HttpResponse::Forbidden().json(serde_json::json!({ "error": err }))),
        },
        Ok(None) => Some(HttpResponse::NotFound().json(serde_json::json!({
            "error": "État daté not found"
        }))),
        Err(err) => Some(HttpResponse::InternalServerError().json(serde_json::json!({
            "error": err
        }))),
    }
}

/// Create a new état daté request
#[post("/etats-dates")]
pub async fn create_etat_date(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    mut request: web::Json<CreateEtatDateRequest>,
) -> impl Responder {
    // Override organization_id from JWT token (security)
    let organization_id = match user.require_organization() {
        Ok(org_id) => org_id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": e.to_string()
            }))
        }
    };
    request.organization_id = organization_id;

    // Isolation multi-tenant à l'ÉCRITURE : l'immeuble visé doit relever d'une
    // ACP dont ce syndic a la gestion. L'affectation de `organization_id`
    // ci-dessus protège le mauvais champ — elle empêche d'estampiller
    // l'enregistrement au nom d'autrui, pas de le rattacher au patrimoine
    // d'autrui (audit du 2026-09-02).
    if let Err(err) = verify_building_org_access(
        &user,
        request.building_id,
        &state.building_use_cases,
        &state.acp_use_cases,
    )
    .await
    {
        return err.error_response();
    }

    match state
        .etat_date_use_cases
        .create_etat_date(request.into_inner())
        .await
    {
        Ok(etat_date) => {
            AuditLogEntry::new(
                AuditEventType::EtatDateCreated,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_resource("EtatDate", etat_date.id)
            .log();

            HttpResponse::Created().json(etat_date)
        }
        Err(err) => {
            AuditLogEntry::new(
                AuditEventType::EtatDateCreated,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_error(err.clone())
            .log();

            // Track H Story H2 — pre-check validate-before-compute → 422 narratif
            if let Some(resp) = try_build_conformity_response(&err) {
                return resp;
            }
            HttpResponse::BadRequest().json(serde_json::json!({
                "error": err
            }))
        }
    }
}

/// Get état daté by ID
#[get("/etats-dates/{id}")]
pub async fn get_etat_date(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    id: web::Path<Uuid>,
) -> impl Responder {
    match state.etat_date_use_cases.get_etat_date(*id).await {
        Ok(Some(etat_date)) => {
            // Verify organization access
            if let Err(err) = user.verify_org_access(etat_date.organization_id) {
                return HttpResponse::Forbidden().json(serde_json::json!({"error": err}));
            }
            HttpResponse::Ok().json(etat_date)
        }
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "État daté not found"
        })),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": err
        })),
    }
}

#[derive(Debug, Deserialize)]
pub struct NotaryLinkTokenQuery {
    pub token: Option<String>,
}

/// Get état daté by reference number — **derrière un lien notaire** (#845,
/// ADR 0048, ADR 0051).
///
/// ── Ce que cette route servait avant ──────────────────────────────────────
///
/// Aucune identité : ni `AuthenticatedUser`, ni jeton lu à la main. Un état
/// daté porte les dettes d'un copropriétaire nommé, et sa référence n'est pas
/// un secret — elle circule dans des courriels, des dossiers de vente. Le
/// seul obstacle pour lire n'importe quel état daté était de connaître une
/// référence devinable. Dernière des 30 routes nues relevées par
/// `garde_identite_absente` (#845).
///
/// ── Ce qu'elle sert maintenant ────────────────────────────────────────────
///
/// Le notaire présente `?token=<jeton>`, émis par le syndic via
/// `POST /etats-dates/{id}/notary-link`. `verify_token` (ci-dessous) vérifie
/// que ce jeton précis ouvre CET état daté, n'est ni expiré ni révoqué. Le
/// jeton EST l'identité — même idiome que `/c/{token}` (liens magiques),
/// mais multi-lecture et révocable/renouvelable (ADR 0051), donc gardé
/// nommément plutôt que rejoint la liste `PUBLIQUES` de la garde.
#[get("/etats-dates/reference/{reference_number}")]
pub async fn get_by_reference_number(
    state: web::Data<AppState>,
    reference_number: web::Path<String>,
    query: web::Query<NotaryLinkTokenQuery>,
) -> impl Responder {
    let etat_date = match state
        .etat_date_use_cases
        .get_by_reference_number(&reference_number)
        .await
    {
        Ok(Some(etat_date)) => etat_date,
        Ok(None) => {
            return HttpResponse::NotFound().json(serde_json::json!({
                "error": "État daté not found"
            }))
        }
        Err(err) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": err
            }))
        }
    };

    let token = query.token.clone().unwrap_or_default();
    let lien = match state
        .lien_notaire_use_cases
        .verify_token(etat_date.id, &token)
        .await
    {
        Ok(lien) => lien,
        Err(err) => return err.error_response(),
    };

    // Consultation anonyme : le lecteur est un notaire sans compte
    // KoproGo, donc `user_id: None` — la traçabilité passe par
    // `lien_notaire_id` (cf. ADR 0051, question laissée ouverte sur le
    // `subject_user_id` d'un notaire sans compte).
    AuditLogEntry::new(
        AuditEventType::NotaryLinkConsulted,
        None,
        Some(etat_date.organization_id),
    )
    .with_resource("EtatDate", etat_date.id)
    .with_metadata(serde_json::json!({ "lien_notaire_id": lien.id }))
    .log();

    HttpResponse::Ok().json(etat_date)
}

/// Émettre un lien notaire pour un état daté (#845 — ADR 0051).
///
/// Cloisonné comme les autres écritures de ce fichier (#864) : le syndic doit
/// avoir la gestion de l'ACP dont relève l'état daté.
#[post("/etats-dates/{id}/notary-link")]
pub async fn issue_notary_link(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    id: web::Path<Uuid>,
) -> impl Responder {
    if let Some(refus) = verify_etat_date_org_access(&state, &user, *id).await {
        return refus;
    }

    match state.lien_notaire_use_cases.issue(*id, user.user_id).await {
        Ok(issued) => {
            AuditLogEntry::new(
                AuditEventType::NotaryLinkIssued,
                Some(user.user_id),
                user.organization_id,
            )
            .with_resource("EtatDate", *id)
            .log();

            HttpResponse::Created().json(issued)
        }
        Err(err) => err.error_response(),
    }
}

/// Renouveler le lien notaire actif d'un état daté — sept jours de plus à
/// partir de maintenant, même jeton (#845 — ADR 0051).
#[put("/etats-dates/{id}/notary-link/renew")]
pub async fn renew_notary_link(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    id: web::Path<Uuid>,
) -> impl Responder {
    if let Some(refus) = verify_etat_date_org_access(&state, &user, *id).await {
        return refus;
    }

    match state.lien_notaire_use_cases.renew(*id).await {
        Ok(status) => {
            AuditLogEntry::new(
                AuditEventType::NotaryLinkRenewed,
                Some(user.user_id),
                user.organization_id,
            )
            .with_resource("EtatDate", *id)
            .log();

            HttpResponse::Ok().json(status)
        }
        Err(err) => err.error_response(),
    }
}

/// Révoquer le lien notaire actif d'un état daté avant terme (#845 — ADR 0051).
#[delete("/etats-dates/{id}/notary-link")]
pub async fn revoke_notary_link(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    id: web::Path<Uuid>,
) -> impl Responder {
    if let Some(refus) = verify_etat_date_org_access(&state, &user, *id).await {
        return refus;
    }

    match state.lien_notaire_use_cases.revoke(*id, user.user_id).await {
        Ok(()) => {
            AuditLogEntry::new(
                AuditEventType::NotaryLinkRevoked,
                Some(user.user_id),
                user.organization_id,
            )
            .with_resource("EtatDate", *id)
            .log();

            HttpResponse::NoContent().finish()
        }
        Err(err) => err.error_response(),
    }
}

/// List états datés paginated
#[get("/etats-dates")]
pub async fn list_etats_dates(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    query: web::Query<EtatDateListQuery>,
) -> impl Responder {
    let organization_id = user.organization_id;

    // Parse status filter
    let status = query.status.as_ref().and_then(|s| match s.as_str() {
        "requested" => Some(EtatDateStatus::Requested),
        "in_progress" => Some(EtatDateStatus::InProgress),
        "generated" => Some(EtatDateStatus::Generated),
        "delivered" => Some(EtatDateStatus::Delivered),
        "expired" => Some(EtatDateStatus::Expired),
        _ => None,
    });

    let page_request = PageRequest {
        page: query.page,
        per_page: query.per_page,
        sort_by: None,
        order: Default::default(),
    };

    match state
        .etat_date_use_cases
        .list_paginated(&page_request, organization_id, status)
        .await
    {
        Ok((etats, total)) => {
            let response =
                PageResponse::new(etats, page_request.page, page_request.per_page, total);
            HttpResponse::Ok().json(response)
        }
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": err
        })),
    }
}

/// List états datés by unit
#[get("/units/{unit_id}/etats-dates")]
pub async fn list_etats_dates_by_unit(
    state: web::Data<AppState>,
    unit_id: web::Path<Uuid>,
    user: AuthenticatedUser,
) -> impl Responder {
    // Route imbriquee non gardee au releve du 2026-09-06 (issue #772).
    if let Err(err) = crate::infrastructure::web::middleware::scope_guard::verify_unit_org_access(
        &user,
        *unit_id,
        &state.unit_use_cases,
        &state.building_use_cases,
        &state.acp_use_cases,
    )
    .await
    {
        return err.error_response();
    }

    match state.etat_date_use_cases.list_by_unit(*unit_id).await {
        Ok(etats) => HttpResponse::Ok().json(etats),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": err
        })),
    }
}

/// List états datés by building
#[get("/buildings/{building_id}/etats-dates")]
pub async fn list_etats_dates_by_building(
    state: web::Data<AppState>,
    building_id: web::Path<Uuid>,
    user: AuthenticatedUser,
) -> impl Responder {
    // Route imbriquee non gardee au releve du 2026-09-06 (issue #772) : elle
    // servait une sous-collection d'un dossier d'ACP a quiconque connaissait
    // un identifiant, sans demander d'identite.
    if let Err(err) =
        crate::infrastructure::web::middleware::scope_guard::verify_building_org_access(
            &user,
            *building_id,
            &state.building_use_cases,
            &state.acp_use_cases,
        )
        .await
    {
        return err.error_response();
    }

    match state
        .etat_date_use_cases
        .list_by_building(*building_id)
        .await
    {
        Ok(etats) => HttpResponse::Ok().json(etats),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": err
        })),
    }
}

/// Mark état daté as in progress
#[put("/etats-dates/{id}/mark-in-progress")]
pub async fn mark_in_progress(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    id: web::Path<Uuid>,
) -> impl Responder {
    // Cloisonnement AVANT la mutation (#864).
    if let Some(refus) = verify_etat_date_org_access(&state, &user, *id).await {
        return refus;
    }

    match state.etat_date_use_cases.mark_in_progress(*id).await {
        Ok(etat_date) => {
            AuditLogEntry::new(
                AuditEventType::EtatDateInProgress,
                Some(user.user_id),
                user.organization_id,
            )
            .with_resource("EtatDate", etat_date.id)
            .log();

            HttpResponse::Ok().json(etat_date)
        }
        Err(err) => HttpResponse::BadRequest().json(serde_json::json!({
            "error": err
        })),
    }
}

/// Mark état daté as generated (with PDF file path)
#[put("/etats-dates/{id}/mark-generated")]
pub async fn mark_generated(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    id: web::Path<Uuid>,
    pdf_path: web::Json<serde_json::Value>,
) -> impl Responder {
    // Cloisonnement AVANT la mutation (#864).
    if let Some(refus) = verify_etat_date_org_access(&state, &user, *id).await {
        return refus;
    }

    let pdf_file_path = match pdf_path.get("pdf_file_path") {
        Some(serde_json::Value::String(path)) => path.clone(),
        _ => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "pdf_file_path is required as a string"
            }))
        }
    };

    match state
        .etat_date_use_cases
        .mark_generated(*id, pdf_file_path)
        .await
    {
        Ok(etat_date) => {
            AuditLogEntry::new(
                AuditEventType::EtatDateGenerated,
                Some(user.user_id),
                user.organization_id,
            )
            .with_resource("EtatDate", etat_date.id)
            .log();

            HttpResponse::Ok().json(etat_date)
        }
        Err(err) => HttpResponse::BadRequest().json(serde_json::json!({
            "error": err
        })),
    }
}

/// Mark état daté as delivered to notary
#[put("/etats-dates/{id}/mark-delivered")]
pub async fn mark_delivered(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    id: web::Path<Uuid>,
) -> impl Responder {
    // Cloisonnement AVANT la mutation (#864).
    if let Some(refus) = verify_etat_date_org_access(&state, &user, *id).await {
        return refus;
    }

    match state.etat_date_use_cases.mark_delivered(*id).await {
        Ok(etat_date) => {
            AuditLogEntry::new(
                AuditEventType::EtatDateDelivered,
                Some(user.user_id),
                user.organization_id,
            )
            .with_resource("EtatDate", etat_date.id)
            .log();

            HttpResponse::Ok().json(etat_date)
        }
        Err(err) => HttpResponse::BadRequest().json(serde_json::json!({
            "error": err
        })),
    }
}

/// Update financial data
#[put("/etats-dates/{id}/financial")]
pub async fn update_financial_data(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    id: web::Path<Uuid>,
    request: web::Json<UpdateEtatDateFinancialRequest>,
) -> impl Responder {
    // Cloisonnement AVANT la mutation (#864).
    if let Some(refus) = verify_etat_date_org_access(&state, &user, *id).await {
        return refus;
    }

    match state
        .etat_date_use_cases
        .update_financial_data(*id, request.into_inner())
        .await
    {
        Ok(etat_date) => {
            AuditLogEntry::new(
                AuditEventType::EtatDateFinancialUpdate,
                Some(user.user_id),
                user.organization_id,
            )
            .with_resource("EtatDate", etat_date.id)
            .log();

            HttpResponse::Ok().json(etat_date)
        }
        Err(err) => HttpResponse::BadRequest().json(serde_json::json!({
            "error": err
        })),
    }
}

/// Update additional data (sections 7-16)
#[put("/etats-dates/{id}/additional-data")]
pub async fn update_additional_data(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    id: web::Path<Uuid>,
    request: web::Json<UpdateEtatDateAdditionalDataRequest>,
) -> impl Responder {
    // Cloisonnement AVANT la mutation (#864).
    if let Some(refus) = verify_etat_date_org_access(&state, &user, *id).await {
        return refus;
    }

    match state
        .etat_date_use_cases
        .update_additional_data(*id, request.into_inner())
        .await
    {
        Ok(etat_date) => {
            AuditLogEntry::new(
                AuditEventType::EtatDateAdditionalDataUpdate,
                Some(user.user_id),
                user.organization_id,
            )
            .with_resource("EtatDate", etat_date.id)
            .log();

            HttpResponse::Ok().json(etat_date)
        }
        Err(err) => HttpResponse::BadRequest().json(serde_json::json!({
            "error": err
        })),
    }
}

/// List overdue états datés (>10 days, not generated yet)
#[get("/etats-dates/overdue")]
pub async fn list_overdue(state: web::Data<AppState>, user: AuthenticatedUser) -> impl Responder {
    let organization_id = match user.require_organization() {
        Ok(org_id) => org_id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": e.to_string()
            }))
        }
    };

    match state
        .etat_date_use_cases
        .list_overdue(organization_id)
        .await
    {
        Ok(etats) => HttpResponse::Ok().json(etats),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": err
        })),
    }
}

/// List expired états datés (>3 months from reference date)
#[get("/etats-dates/expired")]
pub async fn list_expired(state: web::Data<AppState>, user: AuthenticatedUser) -> impl Responder {
    let organization_id = match user.require_organization() {
        Ok(org_id) => org_id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": e.to_string()
            }))
        }
    };

    match state
        .etat_date_use_cases
        .list_expired(organization_id)
        .await
    {
        Ok(etats) => HttpResponse::Ok().json(etats),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": err
        })),
    }
}

/// Get statistics for dashboard
#[get("/etats-dates/stats")]
pub async fn get_stats(state: web::Data<AppState>, user: AuthenticatedUser) -> impl Responder {
    let organization_id = match user.require_organization() {
        Ok(org_id) => org_id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": e.to_string()
            }))
        }
    };

    match state.etat_date_use_cases.get_stats(organization_id).await {
        Ok(stats) => HttpResponse::Ok().json(stats),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": err
        })),
    }
}

/// Delete état daté
#[delete("/etats-dates/{id}")]
pub async fn delete_etat_date(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    id: web::Path<Uuid>,
) -> impl Responder {
    // Cloisonnement AVANT la suppression (#864).
    //
    // Un état daté est la pièce qu'un notaire réclame à la vente. L'effacer
    // n'était soumis à aucun contrôle d'organisation, alors que le LIRE l'était
    // déjà (`get_etat_date`, même fichier).
    match state.etat_date_use_cases.get_etat_date(*id).await {
        Ok(Some(etat_date)) => {
            if let Err(err) = user.verify_org_access(etat_date.organization_id) {
                return HttpResponse::Forbidden().json(serde_json::json!({ "error": err }));
            }
        }
        Ok(None) => {
            return HttpResponse::NotFound().json(serde_json::json!({
                "error": "État daté not found"
            }))
        }
        Err(err) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": err.to_string()
            }))
        }
    }

    match state.etat_date_use_cases.delete_etat_date(*id).await {
        Ok(true) => {
            AuditLogEntry::new(
                AuditEventType::EtatDateDeleted,
                Some(user.user_id),
                user.organization_id,
            )
            .with_resource("EtatDate", *id)
            .log();

            HttpResponse::NoContent().finish()
        }
        Ok(false) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "État daté not found"
        })),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": err
        })),
    }
}
