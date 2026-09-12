use crate::application::dto::{
    AddCertificateDto, AddInspectionPhotoDto, AddReportDto, CreateTechnicalInspectionDto,
    PageRequest, TechnicalInspectionFilters, UpdateTechnicalInspectionDto,
};
use crate::infrastructure::audit::{AuditEventType, AuditLogEntry};
use crate::infrastructure::web::{AppState, AuthenticatedUser};
use actix_web::{delete, get, post, put, web, HttpResponse, Responder, ResponseError};
use uuid::Uuid;

// ==================== Technical Inspection CRUD Endpoints ====================

/// Create a new technical inspection
#[post("/technical-inspections")]
pub async fn create_technical_inspection(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    request: web::Json<CreateTechnicalInspectionDto>,
) -> impl Responder {
    let organization_id = match user.require_organization() {
        Ok(org_id) => org_id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": e.to_string()}))
        }
    };

    match state
        .technical_inspection_use_cases
        // L'organisation vient du JETON, jamais du corps de la requête.
        //
        // Elle y était pourtant attendue, et le client ne l'avait pas toujours :
        // sur `/building-detail`, `organizationId` est résolu via
        // `getAcp(building.acp_id)`, qui **dégrade silencieusement en 403** pour
        // un syndic ou un copropriétaire. Le formulaire postait alors une chaîne
        // vide, `Uuid::parse_str("")` échouait, et le serveur répondait 400 sans
        // que rien n'indique quel champ posait problème (#552).
        //
        // Le handler calculait déjà cette organisation — `require_organization()`
        // ci-dessus — puis la jetait. La lire du jeton corrige le 400 **et**
        // ferme une porte : un client ne peut plus estampiller un inspection technique
        // au nom d'une autre organisation (même famille que l'ADR-0045).
        .create_technical_inspection({
            let mut dto = request.into_inner();
            dto.organization_id = organization_id.to_string();
            dto
        })
        .await
    {
        Ok(inspection) => {
            AuditLogEntry::new(
                AuditEventType::TechnicalInspectionCreated,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_resource("TechnicalInspection", inspection.id.parse().unwrap())
            .log();

            HttpResponse::Created().json(inspection)
        }
        Err(err) => HttpResponse::BadRequest().json(serde_json::json!({"error": err})),
    }
}

/// Get technical inspection by ID
#[get("/technical-inspections/{id}")]
pub async fn get_technical_inspection(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    id: web::Path<Uuid>,
) -> impl Responder {
    // Cette route ne prenait AUCUNE identité : ni `AuthenticatedUser`, ni
    // jeton lu à la main. Le cliquet de #772 ne la voyait pas — il ne
    // compte que les routes PRENANT une identité sans s'en servir.
    // Cf. #845.

    match state
        .technical_inspection_use_cases
        .get_technical_inspection(*id)
        .await
    {
        Ok(Some(inspection)) => {
            // Un contrôle technique porte l'organisation de son immeuble : on
            // refuse celui d'une autre copropriété plutôt que de le servir.
            match Uuid::parse_str(&inspection.organization_id) {
                Ok(org) => match user.verify_org_access(org) {
                    Ok(()) => HttpResponse::Ok().json(inspection),
                    Err(e) => HttpResponse::Forbidden().json(serde_json::json!({ "error": e })),
                },
                Err(_) => HttpResponse::InternalServerError()
                    .json(serde_json::json!({"error": "Invalid organization_id"})),
            }
        }
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Technical inspection not found"
        })),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({"error": err})),
    }
}

/// List technical inspections by building
#[get("/buildings/{building_id}/technical-inspections")]
pub async fn list_building_technical_inspections(
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
        .technical_inspection_use_cases
        .list_technical_inspections_by_building(*building_id)
        .await
    {
        Ok(inspections) => HttpResponse::Ok().json(inspections),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({"error": err})),
    }
}

/// List technical inspections by organization
#[get("/organizations/{organization_id}/technical-inspections")]
pub async fn list_organization_technical_inspections(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    organization_id: web::Path<Uuid>,
) -> impl Responder {
    if let Err(e) = user.verify_org_access(*organization_id) {
        return HttpResponse::Forbidden().json(serde_json::json!({"error": e}));
    }
    match state
        .technical_inspection_use_cases
        .list_technical_inspections_by_organization(*organization_id)
        .await
    {
        Ok(inspections) => HttpResponse::Ok().json(inspections),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({"error": err})),
    }
}

/// List technical inspections with pagination and filters
#[get("/technical-inspections")]
pub async fn list_technical_inspections_paginated(
    state: web::Data<AppState>,
    _user: AuthenticatedUser,
    page_request: web::Query<PageRequest>,
    filters: web::Query<TechnicalInspectionFilters>,
) -> impl Responder {
    // Cette route ne prenait AUCUNE identité : ni `AuthenticatedUser`, ni
    // jeton lu à la main. Le cliquet de #772 ne la voyait pas — il ne
    // compte que les routes PRENANT une identité sans s'en servir.
    // Cf. #845.
    //
    // L'identité est EXIGÉE mais pas encore employée à filtrer : la liste
    // paginée ne porte pas de périmètre, et l'y ajouter demande de savoir si
    // elle doit être bornée par organisation ou par immeuble. Exiger un jeton
    // ferme la porte anonyme sans préjuger de ce filtrage — la dette de
    // lecture imbriquée reste suivie par `garde_lecture`.

    match state
        .technical_inspection_use_cases
        .list_technical_inspections_paginated(&page_request.into_inner(), &filters.into_inner())
        .await
    {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({"error": err})),
    }
}

/// Update technical inspection
#[put("/technical-inspections/{id}")]
pub async fn update_technical_inspection(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    id: web::Path<Uuid>,
    request: web::Json<UpdateTechnicalInspectionDto>,
) -> impl Responder {
    let organization_id = match user.require_organization() {
        Ok(org_id) => org_id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": e.to_string()}))
        }
    };

    match state
        .technical_inspection_use_cases
        .update_technical_inspection(*id, request.into_inner())
        .await
    {
        Ok(inspection) => {
            AuditLogEntry::new(
                AuditEventType::TechnicalInspectionUpdated,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_resource("TechnicalInspection", *id)
            .log();

            HttpResponse::Ok().json(inspection)
        }
        Err(err) => HttpResponse::BadRequest().json(serde_json::json!({"error": err})),
    }
}

/// Delete technical inspection
#[delete("/technical-inspections/{id}")]
pub async fn delete_technical_inspection(
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
        .technical_inspection_use_cases
        .delete_technical_inspection(*id)
        .await
    {
        Ok(deleted) => {
            if deleted {
                AuditLogEntry::new(
                    AuditEventType::TechnicalInspectionDeleted,
                    Some(user.user_id),
                    Some(organization_id),
                )
                .with_resource("TechnicalInspection", *id)
                .log();

                HttpResponse::NoContent().finish()
            } else {
                HttpResponse::NotFound().json(serde_json::json!({
                    "error": "Technical inspection not found"
                }))
            }
        }
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({"error": err})),
    }
}

// ==================== Inspection Tracking Endpoints ====================

/// Get overdue inspections for a building
#[get("/buildings/{building_id}/technical-inspections/overdue")]
pub async fn get_overdue_inspections(
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
        .technical_inspection_use_cases
        .get_overdue_inspections(*building_id)
        .await
    {
        Ok(inspections) => HttpResponse::Ok().json(inspections),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({"error": err})),
    }
}

/// Get upcoming inspections for a building (within X days)
#[get("/buildings/{building_id}/technical-inspections/upcoming")]
pub async fn get_upcoming_inspections(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
    query: web::Query<serde_json::Value>,
    user: AuthenticatedUser,
) -> impl Responder {
    let building_id = path.into_inner();
    let days = query.get("days").and_then(|v| v.as_i64()).unwrap_or(90) as i32;

    // Route imbriquee non gardee au releve du 2026-09-06 (issue #772).
    //
    // Elle avait recu le parametre `user` sans la garde qui l'emploie : le
    // cliquet, qui comptait la presence d'`AuthenticatedUser`, l'aurait donc
    // declaree protegee alors qu'elle ne l'etait pas. Seul l'avertissement
    // `unused variable` du compilateur l'a signalee. D'ou le second cliquet.
    if let Err(err) =
        crate::infrastructure::web::middleware::scope_guard::verify_building_org_access(
            &user,
            building_id,
            &state.building_use_cases,
            &state.acp_use_cases,
        )
        .await
    {
        return err.error_response();
    }

    match state
        .technical_inspection_use_cases
        .get_upcoming_inspections(building_id, days)
        .await
    {
        Ok(inspections) => HttpResponse::Ok().json(inspections),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({"error": err})),
    }
}

/// Get inspections by type for a building
#[get("/buildings/{building_id}/technical-inspections/type/{inspection_type}")]
pub async fn get_inspections_by_type(
    state: web::Data<AppState>,
    path: web::Path<(Uuid, String)>,
    user: AuthenticatedUser,
) -> impl Responder {
    let (building_id, inspection_type) = path.into_inner();
    // Route imbriquee non gardee au releve du 2026-09-06 (issue #772) : elle
    // servait une sous-collection d'un dossier d'ACP a quiconque connaissait
    // un identifiant, sans demander d'identite.
    if let Err(err) =
        crate::infrastructure::web::middleware::scope_guard::verify_building_org_access(
            &user,
            building_id,
            &state.building_use_cases,
            &state.acp_use_cases,
        )
        .await
    {
        return err.error_response();
    }

    // Route imbriquee non gardee au releve du 2026-09-06 (issue #772) : elle
    // servait une sous-collection d'un dossier d'ACP a quiconque connaissait
    // un identifiant, sans demander d'identite.
    if let Err(err) =
        crate::infrastructure::web::middleware::scope_guard::verify_building_org_access(
            &user,
            building_id,
            &state.building_use_cases,
            &state.acp_use_cases,
        )
        .await
    {
        return err.error_response();
    }

    match state
        .technical_inspection_use_cases
        .get_inspections_by_type(building_id, &inspection_type)
        .await
    {
        Ok(inspections) => HttpResponse::Ok().json(inspections),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({"error": err})),
    }
}

// ==================== Document Management ====================

/// Add report to technical inspection
#[post("/technical-inspections/{id}/reports")]
pub async fn add_report(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    id: web::Path<Uuid>,
    request: web::Json<AddReportDto>,
) -> impl Responder {
    let organization_id = match user.require_organization() {
        Ok(org_id) => org_id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": e.to_string()}))
        }
    };

    match state
        .technical_inspection_use_cases
        .add_report(*id, request.into_inner())
        .await
    {
        Ok(inspection) => {
            AuditLogEntry::new(
                AuditEventType::TechnicalInspectionReportAdded,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_resource("TechnicalInspection", *id)
            .log();

            HttpResponse::Ok().json(inspection)
        }
        Err(err) => HttpResponse::BadRequest().json(serde_json::json!({"error": err})),
    }
}

/// Add photo to technical inspection
#[post("/technical-inspections/{id}/photos")]
pub async fn add_inspection_photo(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    id: web::Path<Uuid>,
    request: web::Json<AddInspectionPhotoDto>,
) -> impl Responder {
    let organization_id = match user.require_organization() {
        Ok(org_id) => org_id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": e.to_string()}))
        }
    };

    match state
        .technical_inspection_use_cases
        .add_photo(*id, request.into_inner())
        .await
    {
        Ok(inspection) => {
            AuditLogEntry::new(
                AuditEventType::TechnicalInspectionPhotoAdded,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_resource("TechnicalInspection", *id)
            .log();

            HttpResponse::Ok().json(inspection)
        }
        Err(err) => HttpResponse::BadRequest().json(serde_json::json!({"error": err})),
    }
}

/// Add certificate to technical inspection
#[post("/technical-inspections/{id}/certificates")]
pub async fn add_certificate(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    id: web::Path<Uuid>,
    request: web::Json<AddCertificateDto>,
) -> impl Responder {
    let organization_id = match user.require_organization() {
        Ok(org_id) => org_id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": e.to_string()}))
        }
    };

    match state
        .technical_inspection_use_cases
        .add_certificate(*id, request.into_inner())
        .await
    {
        Ok(inspection) => {
            AuditLogEntry::new(
                AuditEventType::TechnicalInspectionCertificateAdded,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_resource("TechnicalInspection", *id)
            .log();

            HttpResponse::Ok().json(inspection)
        }
        Err(err) => HttpResponse::BadRequest().json(serde_json::json!({"error": err})),
    }
}
