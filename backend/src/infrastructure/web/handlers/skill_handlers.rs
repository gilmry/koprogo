use crate::application::dto::{CreateSkillDto, UpdateSkillDto};
use crate::domain::entities::{ExpertiseLevel, SkillCategory};
use crate::infrastructure::web::app_state::AppState;
use crate::infrastructure::web::classification_erreurs;
use crate::infrastructure::web::middleware::AuthenticatedUser;
use actix_web::{delete, get, post, put, web, HttpResponse, Responder, ResponseError};
use uuid::Uuid;

/// Create a new skill
///
/// POST /skills
#[post("/skills")]
pub async fn create_skill(
    data: web::Data<AppState>,
    auth: AuthenticatedUser,
    request: web::Json<CreateSkillDto>,
) -> impl Responder {
    let org_id = match auth.require_organization() {
        Ok(id) => id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": e.to_string()}))
        }
    };
    match data
        .skill_use_cases
        .create_skill(auth.user_id, org_id, request.into_inner())
        .await
    {
        Ok(skill) => HttpResponse::Created().json(skill),
        Err(e) => HttpResponse::BadRequest().json(serde_json::json!({"error": e})),
    }
}

/// Get skill by ID with owner name enrichment
///
/// GET /skills/:id
#[get("/skills/{id}")]
pub async fn get_skill(data: web::Data<AppState>, id: web::Path<Uuid>) -> impl Responder {
    match data.skill_use_cases.get_skill(id.into_inner()).await {
        Ok(skill) => HttpResponse::Ok().json(skill),
        Err(e) => {
            if classification_erreurs::est_introuvable(&e) {
                HttpResponse::NotFound().json(serde_json::json!({"error": e}))
            } else {
                HttpResponse::InternalServerError().json(serde_json::json!({"error": e}))
            }
        }
    }
}

/// List all skills for a building
///
/// GET /buildings/:building_id/skills
#[get("/buildings/{building_id}/skills")]
pub async fn list_building_skills(
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
        .skill_use_cases
        .list_building_skills(building_id.into_inner())
        .await
    {
        Ok(skills) => HttpResponse::Ok().json(skills),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e})),
    }
}

/// List available skills for a building (marketplace view)
///
/// GET /buildings/:building_id/skills/available
#[get("/buildings/{building_id}/skills/available")]
pub async fn list_available_skills(
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
        .skill_use_cases
        .list_available_skills(building_id.into_inner())
        .await
    {
        Ok(skills) => HttpResponse::Ok().json(skills),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e})),
    }
}

/// List free/volunteer skills for a building
///
/// GET /buildings/:building_id/skills/free
#[get("/buildings/{building_id}/skills/free")]
pub async fn list_free_skills(
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
        .skill_use_cases
        .list_free_skills(building_id.into_inner())
        .await
    {
        Ok(skills) => HttpResponse::Ok().json(skills),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e})),
    }
}

/// List professional skills for a building
///
/// GET /buildings/:building_id/skills/professional
#[get("/buildings/{building_id}/skills/professional")]
pub async fn list_professional_skills(
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
        .skill_use_cases
        .list_professional_skills(building_id.into_inner())
        .await
    {
        Ok(skills) => HttpResponse::Ok().json(skills),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e})),
    }
}

/// List skills by category (HomeRepair, Languages, Technology, etc.)
///
/// GET /buildings/:building_id/skills/category/:category
#[get("/buildings/{building_id}/skills/category/{category}")]
pub async fn list_skills_by_category(
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

    // Parse skill category
    let category = match serde_json::from_str::<SkillCategory>(&format!("\"{}\"", category_str)) {
        Ok(c) => c,
        Err(_) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": format!("Invalid skill category: {}. Valid categories: HomeRepair, Languages, Technology, Education, Arts, Sports, Cooking, Gardening, Health, Legal, Financial, PetCare, Other", category_str)
            }))
        }
    };

    match data
        .skill_use_cases
        .list_skills_by_category(building_id, category)
        .await
    {
        Ok(skills) => HttpResponse::Ok().json(skills),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e})),
    }
}

/// List skills by expertise level (Beginner, Intermediate, Advanced, Expert)
///
/// GET /buildings/:building_id/skills/expertise/:level
#[get("/buildings/{building_id}/skills/expertise/{level}")]
pub async fn list_skills_by_expertise(
    data: web::Data<AppState>,
    path: web::Path<(Uuid, String)>,
    user: AuthenticatedUser,
) -> impl Responder {
    let (building_id, level_str) = path.into_inner();
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

    // Parse expertise level
    let level = match serde_json::from_str::<ExpertiseLevel>(&format!("\"{}\"", level_str)) {
        Ok(l) => l,
        Err(_) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": format!("Invalid expertise level: {}. Valid levels: Beginner, Intermediate, Advanced, Expert", level_str)
            }))
        }
    };

    match data
        .skill_use_cases
        .list_skills_by_expertise(building_id, level)
        .await
    {
        Ok(skills) => HttpResponse::Ok().json(skills),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e})),
    }
}

/// List all skills created by an owner
///
/// GET /owners/:owner_id/skills
#[get("/owners/{owner_id}/skills")]
pub async fn list_owner_skills(
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
        .skill_use_cases
        .list_owner_skills(owner_id.into_inner())
        .await
    {
        Ok(skills) => HttpResponse::Ok().json(skills),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e})),
    }
}

/// Update a skill
///
/// PUT /skills/:id
#[put("/skills/{id}")]
pub async fn update_skill(
    data: web::Data<AppState>,
    auth: AuthenticatedUser,
    id: web::Path<Uuid>,
    request: web::Json<UpdateSkillDto>,
) -> impl Responder {
    let org_id = match auth.require_organization() {
        Ok(id) => id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": e.to_string()}))
        }
    };
    match data
        .skill_use_cases
        .update_skill(id.into_inner(), auth.user_id, org_id, request.into_inner())
        .await
    {
        Ok(skill) => HttpResponse::Ok().json(skill),
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

/// Mark skill as available for help
///
/// POST /skills/:id/mark-available
#[post("/skills/{id}/mark-available")]
pub async fn mark_skill_available(
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
        .skill_use_cases
        .mark_skill_available(id.into_inner(), auth.user_id, org_id)
        .await
    {
        Ok(skill) => HttpResponse::Ok().json(skill),
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

/// Mark skill as unavailable for help
///
/// POST /skills/:id/mark-unavailable
#[post("/skills/{id}/mark-unavailable")]
pub async fn mark_skill_unavailable(
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
        .skill_use_cases
        .mark_skill_unavailable(id.into_inner(), auth.user_id, org_id)
        .await
    {
        Ok(skill) => HttpResponse::Ok().json(skill),
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

/// Delete a skill
///
/// DELETE /skills/:id
#[delete("/skills/{id}")]
pub async fn delete_skill(
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
        .skill_use_cases
        .delete_skill(id.into_inner(), auth.user_id, org_id)
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

/// Get skill statistics for a building
///
/// GET /buildings/:building_id/skills/statistics
#[get("/buildings/{building_id}/skills/statistics")]
pub async fn get_skill_statistics(
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
        .skill_use_cases
        .get_skill_statistics(building_id.into_inner())
        .await
    {
        Ok(stats) => HttpResponse::Ok().json(stats),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e})),
    }
}
