use crate::application::dto::{BorrowObjectDto, CreateSharedObjectDto, UpdateSharedObjectDto};
use crate::domain::entities::SharedObjectCategory;
use crate::infrastructure::web::app_state::AppState;
use crate::infrastructure::web::classification_erreurs;
use crate::infrastructure::web::middleware::AuthenticatedUser;
use actix_web::{delete, get, post, put, web, HttpResponse, Responder, ResponseError};
use uuid::Uuid;

/// Create a new shared object
///
/// POST /shared-objects
#[post("/shared-objects")]
pub async fn create_shared_object(
    data: web::Data<AppState>,
    auth: AuthenticatedUser,
    request: web::Json<CreateSharedObjectDto>,
) -> impl Responder {
    let org_id = match auth.require_organization() {
        Ok(id) => id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": e.to_string()}))
        }
    };
    match data
        .shared_object_use_cases
        .create_shared_object(auth.user_id, org_id, request.into_inner())
        .await
    {
        Ok(object) => HttpResponse::Created().json(object),
        Err(e) => HttpResponse::BadRequest().json(serde_json::json!({"error": e})),
    }
}

/// Get shared object by ID with owner/borrower name enrichment
///
/// GET /shared-objects/:id
#[get("/shared-objects/{id}")]
pub async fn get_shared_object(
    data: web::Data<AppState>,
    user: AuthenticatedUser,
    id: web::Path<Uuid>,
) -> impl Responder {
    let identifiant = id.into_inner();

    // Cette route ne prenait AUCUNE identité : n'importe qui pouvait la lire
    // sur simple connaissance de l'identifiant. Le cliquet de #772 ne la
    // voyait pas — il ne compte que les routes PRENANT une identité sans
    // s'en servir. Cf. #845.
    if let Err(err) =
        crate::infrastructure::web::middleware::scope_guard::verify_shared_object_org_access(
            &user,
            identifiant,
            &data.shared_object_use_cases,
            &data.building_use_cases,
            &data.acp_use_cases,
        )
        .await
    {
        return err.error_response();
    }

    match data
        .shared_object_use_cases
        .get_shared_object(identifiant)
        .await
    {
        Ok(object) => HttpResponse::Ok().json(object),
        Err(e) => {
            if classification_erreurs::est_introuvable(&e) {
                HttpResponse::NotFound().json(serde_json::json!({"error": e}))
            } else {
                HttpResponse::InternalServerError().json(serde_json::json!({"error": e}))
            }
        }
    }
}

/// List all shared objects for a building
///
/// GET /buildings/:building_id/shared-objects
#[get("/buildings/{building_id}/shared-objects")]
pub async fn list_building_objects(
    data: web::Data<AppState>,
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
            &data.building_use_cases,
            &data.acp_use_cases,
        )
        .await
    {
        return err.error_response();
    }

    match data
        .shared_object_use_cases
        .list_building_objects(building_id.into_inner())
        .await
    {
        Ok(objects) => HttpResponse::Ok().json(objects),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e})),
    }
}

/// List available shared objects for a building (marketplace view)
///
/// GET /buildings/:building_id/shared-objects/available
#[get("/buildings/{building_id}/shared-objects/available")]
pub async fn list_available_objects(
    data: web::Data<AppState>,
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
            &data.building_use_cases,
            &data.acp_use_cases,
        )
        .await
    {
        return err.error_response();
    }

    match data
        .shared_object_use_cases
        .list_available_objects(building_id.into_inner())
        .await
    {
        Ok(objects) => HttpResponse::Ok().json(objects),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e})),
    }
}

/// List borrowed shared objects for a building
///
/// GET /buildings/:building_id/shared-objects/borrowed
#[get("/buildings/{building_id}/shared-objects/borrowed")]
pub async fn list_borrowed_objects(
    data: web::Data<AppState>,
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
            &data.building_use_cases,
            &data.acp_use_cases,
        )
        .await
    {
        return err.error_response();
    }

    match data
        .shared_object_use_cases
        .list_borrowed_objects(building_id.into_inner())
        .await
    {
        Ok(objects) => HttpResponse::Ok().json(objects),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e})),
    }
}

/// List overdue shared objects for a building
///
/// GET /buildings/:building_id/shared-objects/overdue
#[get("/buildings/{building_id}/shared-objects/overdue")]
pub async fn list_overdue_objects(
    data: web::Data<AppState>,
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
            &data.building_use_cases,
            &data.acp_use_cases,
        )
        .await
    {
        return err.error_response();
    }

    match data
        .shared_object_use_cases
        .list_overdue_objects(building_id.into_inner())
        .await
    {
        Ok(objects) => HttpResponse::Ok().json(objects),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e})),
    }
}

/// List free/volunteer shared objects for a building
///
/// GET /buildings/:building_id/shared-objects/free
#[get("/buildings/{building_id}/shared-objects/free")]
pub async fn list_free_objects(
    data: web::Data<AppState>,
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
            &data.building_use_cases,
            &data.acp_use_cases,
        )
        .await
    {
        return err.error_response();
    }

    match data
        .shared_object_use_cases
        .list_free_objects(building_id.into_inner())
        .await
    {
        Ok(objects) => HttpResponse::Ok().json(objects),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e})),
    }
}

/// List shared objects by category (Tools, Books, Electronics, etc.)
///
/// GET /buildings/:building_id/shared-objects/category/:category
#[get("/buildings/{building_id}/shared-objects/category/{category}")]
pub async fn list_objects_by_category(
    data: web::Data<AppState>,
    path: web::Path<(Uuid, String)>,
    user: AuthenticatedUser,
) -> impl Responder {
    let (building_id, category_str) = path.into_inner();
    // Route imbriquee non gardee au releve du 2026-09-06 (issue #772) : elle
    // servait une sous-collection d'un dossier d'ACP a quiconque connaissait
    // un identifiant, sans demander d'identite.
    if let Err(err) =
        crate::infrastructure::web::middleware::scope_guard::verify_building_org_access(
            &user,
            building_id,
            &data.building_use_cases,
            &data.acp_use_cases,
        )
        .await
    {
        return err.error_response();
    }

    // Parse object category
    let category = match serde_json::from_str::<SharedObjectCategory>(&format!(
        "\"{}\"",
        category_str
    )) {
        Ok(c) => c,
        Err(_) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": format!("Invalid shared object category: {}. Valid categories: Tools, Books, Electronics, Sports, Gardening, Kitchen, Baby, Other", category_str)
            }))
        }
    };

    match data
        .shared_object_use_cases
        .list_objects_by_category(building_id, category)
        .await
    {
        Ok(objects) => HttpResponse::Ok().json(objects),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e})),
    }
}

/// List all shared objects created by an owner
///
/// GET /owners/:owner_id/shared-objects
#[get("/owners/{owner_id}/shared-objects")]
pub async fn list_owner_objects(
    data: web::Data<AppState>,
    owner_id: web::Path<Uuid>,
    user: AuthenticatedUser,
) -> impl Responder {
    // Route imbriquee non gardee au releve du 2026-09-06 (issue #772) : elle
    // servait la situation financiere NOMINATIVE d'une personne a quiconque
    // connaissait son identifiant.
    if let Err(err) = crate::infrastructure::web::middleware::scope_guard::verify_owner_org_access(
        &user,
        *owner_id,
        &data.owner_use_cases,
    )
    .await
    {
        return err.error_response();
    }

    match data
        .shared_object_use_cases
        .list_owner_objects(owner_id.into_inner())
        .await
    {
        Ok(objects) => HttpResponse::Ok().json(objects),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e})),
    }
}

/// List all shared objects currently borrowed by a user
///
/// GET /shared-objects/my-borrowed
#[get("/shared-objects/my-borrowed")]
pub async fn list_my_borrowed_objects(
    data: web::Data<AppState>,
    auth: AuthenticatedUser,
) -> impl Responder {
    let org_id = match auth.require_organization() {
        Ok(id) => id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": e.to_string()}))
        }
    };
    match data
        .shared_object_use_cases
        .list_user_borrowed_objects(auth.user_id, org_id)
        .await
    {
        Ok(objects) => HttpResponse::Ok().json(objects),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e})),
    }
}

/// Update a shared object
///
/// PUT /shared-objects/:id
#[put("/shared-objects/{id}")]
pub async fn update_shared_object(
    data: web::Data<AppState>,
    auth: AuthenticatedUser,
    id: web::Path<Uuid>,
    request: web::Json<UpdateSharedObjectDto>,
) -> impl Responder {
    let org_id = match auth.require_organization() {
        Ok(id) => id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": e.to_string()}))
        }
    };
    match data
        .shared_object_use_cases
        .update_shared_object(id.into_inner(), auth.user_id, org_id, request.into_inner())
        .await
    {
        Ok(object) => HttpResponse::Ok().json(object),
        Err(e) => {
            if classification_erreurs::est_interdit(&e) {
                HttpResponse::Forbidden().json(serde_json::json!({"error": e}))
            } else if classification_erreurs::est_introuvable(&e) {
                HttpResponse::NotFound().json(serde_json::json!({"error": e}))
            } else {
                HttpResponse::BadRequest().json(serde_json::json!({"error": e}))
            }
        }
    }
}

/// Mark shared object as available
///
/// POST /shared-objects/:id/mark-available
#[post("/shared-objects/{id}/mark-available")]
pub async fn mark_object_available(
    data: web::Data<AppState>,
    auth: AuthenticatedUser,
    id: web::Path<Uuid>,
) -> impl Responder {
    let org_id = match auth.require_organization() {
        Ok(id) => id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": e.to_string()}))
        }
    };
    match data
        .shared_object_use_cases
        .mark_object_available(id.into_inner(), auth.user_id, org_id)
        .await
    {
        Ok(object) => HttpResponse::Ok().json(object),
        Err(e) => {
            if classification_erreurs::est_interdit(&e) {
                HttpResponse::Forbidden().json(serde_json::json!({"error": e}))
            } else if classification_erreurs::est_introuvable(&e) {
                HttpResponse::NotFound().json(serde_json::json!({"error": e}))
            } else {
                HttpResponse::BadRequest().json(serde_json::json!({"error": e}))
            }
        }
    }
}

/// Mark shared object as unavailable
///
/// POST /shared-objects/:id/mark-unavailable
#[post("/shared-objects/{id}/mark-unavailable")]
pub async fn mark_object_unavailable(
    data: web::Data<AppState>,
    auth: AuthenticatedUser,
    id: web::Path<Uuid>,
) -> impl Responder {
    let org_id = match auth.require_organization() {
        Ok(id) => id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": e.to_string()}))
        }
    };
    match data
        .shared_object_use_cases
        .mark_object_unavailable(id.into_inner(), auth.user_id, org_id)
        .await
    {
        Ok(object) => HttpResponse::Ok().json(object),
        Err(e) => {
            if classification_erreurs::est_interdit(&e) {
                HttpResponse::Forbidden().json(serde_json::json!({"error": e}))
            } else if classification_erreurs::est_introuvable(&e) {
                HttpResponse::NotFound().json(serde_json::json!({"error": e}))
            } else {
                HttpResponse::BadRequest().json(serde_json::json!({"error": e}))
            }
        }
    }
}

/// Borrow a shared object
///
/// POST /shared-objects/:id/borrow
#[post("/shared-objects/{id}/borrow")]
pub async fn borrow_object(
    data: web::Data<AppState>,
    auth: AuthenticatedUser,
    id: web::Path<Uuid>,
    request: web::Json<BorrowObjectDto>,
) -> impl Responder {
    let org_id = match auth.require_organization() {
        Ok(id) => id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": e.to_string()}))
        }
    };
    match data
        .shared_object_use_cases
        .borrow_object(id.into_inner(), auth.user_id, org_id, request.into_inner())
        .await
    {
        Ok(object) => HttpResponse::Ok().json(object),
        Err(e) => {
            if e.contains("Owner cannot borrow") {
                HttpResponse::Forbidden().json(serde_json::json!({"error": e}))
            } else if classification_erreurs::est_introuvable(&e) {
                HttpResponse::NotFound().json(serde_json::json!({"error": e}))
            } else {
                HttpResponse::BadRequest().json(serde_json::json!({"error": e}))
            }
        }
    }
}

/// Return a borrowed object
///
/// POST /shared-objects/:id/return
#[post("/shared-objects/{id}/return")]
pub async fn return_object(
    data: web::Data<AppState>,
    auth: AuthenticatedUser,
    id: web::Path<Uuid>,
) -> impl Responder {
    let org_id = match auth.require_organization() {
        Ok(id) => id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": e.to_string()}))
        }
    };
    match data
        .shared_object_use_cases
        .return_object(id.into_inner(), auth.user_id, org_id)
        .await
    {
        Ok(object) => HttpResponse::Ok().json(object),
        Err(e) => {
            if e.contains("Only borrower can return") {
                HttpResponse::Forbidden().json(serde_json::json!({"error": e}))
            } else if classification_erreurs::est_introuvable(&e) {
                HttpResponse::NotFound().json(serde_json::json!({"error": e}))
            } else {
                HttpResponse::BadRequest().json(serde_json::json!({"error": e}))
            }
        }
    }
}

/// Delete a shared object
///
/// DELETE /shared-objects/:id
#[delete("/shared-objects/{id}")]
pub async fn delete_shared_object(
    data: web::Data<AppState>,
    auth: AuthenticatedUser,
    id: web::Path<Uuid>,
) -> impl Responder {
    let org_id = match auth.require_organization() {
        Ok(id) => id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": e.to_string()}))
        }
    };
    match data
        .shared_object_use_cases
        .delete_shared_object(id.into_inner(), auth.user_id, org_id)
        .await
    {
        Ok(_) => HttpResponse::NoContent().finish(),
        Err(e) => {
            if classification_erreurs::est_interdit(&e) {
                HttpResponse::Forbidden().json(serde_json::json!({"error": e}))
            } else if classification_erreurs::est_introuvable(&e) {
                HttpResponse::NotFound().json(serde_json::json!({"error": e}))
            } else {
                HttpResponse::BadRequest().json(serde_json::json!({"error": e}))
            }
        }
    }
}

/// Get shared object statistics for a building
///
/// GET /buildings/:building_id/shared-objects/statistics
#[get("/buildings/{building_id}/shared-objects/statistics")]
pub async fn get_object_statistics(
    data: web::Data<AppState>,
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
            &data.building_use_cases,
            &data.acp_use_cases,
        )
        .await
    {
        return err.error_response();
    }

    match data
        .shared_object_use_cases
        .get_object_statistics(building_id.into_inner())
        .await
    {
        Ok(stats) => HttpResponse::Ok().json(stats),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e})),
    }
}
