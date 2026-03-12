use crate::application::dto::age_request_dto::{
    AddCosignatoryDto, CreateAgeRequestDto, SyndicResponseDto,
};
use crate::infrastructure::web::{AppState, AuthenticatedUser};
use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use uuid::Uuid;

/// POST /buildings/:id/age-requests — Créer une demande d'AGE
#[post("/buildings/{building_id}/age-requests")]
pub async fn create_age_request(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    path: web::Path<Uuid>,
    body: web::Json<CreateAgeRequestDtoWithoutBuildingId>,
) -> impl Responder {
    let organization_id = match user.require_organization() {
        Ok(id) => id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": e.to_string()}))
        }
    };

    let building_id = path.into_inner();
    let dto = CreateAgeRequestDto {
        building_id,
        title: body.title.clone(),
        description: body.description.clone(),
    };

    match state
        .age_request_use_cases
        .create(organization_id, user.user_id, dto)
        .await
    {
        Ok(req) => HttpResponse::Created().json(req),
        Err(e) => HttpResponse::BadRequest().json(serde_json::json!({"error": e})),
    }
}

/// GET /buildings/:id/age-requests — Lister les demandes d'AGE d'un bâtiment
#[get("/buildings/{building_id}/age-requests")]
pub async fn list_age_requests(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    path: web::Path<Uuid>,
) -> impl Responder {
    let organization_id = match user.require_organization() {
        Ok(id) => id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": e.to_string()}))
        }
    };

    match state
        .age_request_use_cases
        .list_by_building(path.into_inner(), organization_id)
        .await
    {
        Ok(reqs) => HttpResponse::Ok().json(reqs),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e})),
    }
}

/// GET /age-requests/:id — Détail d'une demande d'AGE
#[get("/age-requests/{id}")]
pub async fn get_age_request(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    path: web::Path<Uuid>,
) -> impl Responder {
    let organization_id = match user.require_organization() {
        Ok(id) => id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": e.to_string()}))
        }
    };

    match state
        .age_request_use_cases
        .get(path.into_inner(), organization_id)
        .await
    {
        Ok(req) => HttpResponse::Ok().json(req),
        Err(e) => {
            if e.contains("introuvable") {
                HttpResponse::NotFound().json(serde_json::json!({"error": e}))
            } else if e.contains("refusé") {
                HttpResponse::Forbidden().json(serde_json::json!({"error": e}))
            } else {
                HttpResponse::InternalServerError().json(serde_json::json!({"error": e}))
            }
        }
    }
}

/// PUT /age-requests/:id/open — Ouvrir la demande pour signatures
#[put("/age-requests/{id}/open")]
pub async fn open_age_request(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    path: web::Path<Uuid>,
) -> impl Responder {
    let organization_id = match user.require_organization() {
        Ok(id) => id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": e.to_string()}))
        }
    };

    match state
        .age_request_use_cases
        .open(path.into_inner(), organization_id, user.user_id)
        .await
    {
        Ok(req) => HttpResponse::Ok().json(req),
        Err(e) => HttpResponse::BadRequest().json(serde_json::json!({"error": e})),
    }
}

/// POST /age-requests/:id/cosignatories — Ajouter un cosignataire
#[post("/age-requests/{id}/cosignatories")]
pub async fn add_cosignatory(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    path: web::Path<Uuid>,
    body: web::Json<AddCosignatoryDto>,
) -> impl Responder {
    let organization_id = match user.require_organization() {
        Ok(id) => id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": e.to_string()}))
        }
    };

    match state
        .age_request_use_cases
        .add_cosignatory(path.into_inner(), organization_id, body.into_inner())
        .await
    {
        Ok(req) => HttpResponse::Ok().json(req),
        Err(e) => HttpResponse::BadRequest().json(serde_json::json!({"error": e})),
    }
}

/// DELETE /age-requests/:id/cosignatories/:owner_id — Retirer un cosignataire
#[delete("/age-requests/{id}/cosignatories/{owner_id}")]
pub async fn remove_cosignatory(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    path: web::Path<(Uuid, Uuid)>,
) -> impl Responder {
    let organization_id = match user.require_organization() {
        Ok(id) => id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": e.to_string()}))
        }
    };

    let (id, owner_id) = path.into_inner();
    match state
        .age_request_use_cases
        .remove_cosignatory(id, owner_id, organization_id)
        .await
    {
        Ok(req) => HttpResponse::Ok().json(req),
        Err(e) => HttpResponse::BadRequest().json(serde_json::json!({"error": e})),
    }
}

/// POST /age-requests/:id/submit — Soumettre la demande au syndic
#[post("/age-requests/{id}/submit")]
pub async fn submit_age_request(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    path: web::Path<Uuid>,
) -> impl Responder {
    let organization_id = match user.require_organization() {
        Ok(id) => id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": e.to_string()}))
        }
    };

    match state
        .age_request_use_cases
        .submit_to_syndic(path.into_inner(), organization_id, user.user_id)
        .await
    {
        Ok(req) => HttpResponse::Ok().json(req),
        Err(e) => HttpResponse::BadRequest().json(serde_json::json!({"error": e})),
    }
}

/// PUT /age-requests/:id/syndic-response — Réponse du syndic (accept/reject)
#[put("/age-requests/{id}/syndic-response")]
pub async fn syndic_response(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    path: web::Path<Uuid>,
    body: web::Json<SyndicResponseDto>,
) -> impl Responder {
    let organization_id = match user.require_organization() {
        Ok(id) => id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": e.to_string()}))
        }
    };

    match state
        .age_request_use_cases
        .syndic_response(path.into_inner(), organization_id, body.into_inner())
        .await
    {
        Ok(req) => HttpResponse::Ok().json(req),
        Err(e) => HttpResponse::BadRequest().json(serde_json::json!({"error": e})),
    }
}

/// POST /age-requests/:id/auto-convocation — Déclencher l'auto-convocation
/// (si délai syndic dépassé, Art. 3.87 §2 CC)
#[post("/age-requests/{id}/auto-convocation")]
pub async fn trigger_auto_convocation(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    path: web::Path<Uuid>,
) -> impl Responder {
    let organization_id = match user.require_organization() {
        Ok(id) => id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": e.to_string()}))
        }
    };

    match state
        .age_request_use_cases
        .trigger_auto_convocation(path.into_inner(), organization_id)
        .await
    {
        Ok(req) => HttpResponse::Ok().json(req),
        Err(e) => HttpResponse::BadRequest().json(serde_json::json!({"error": e})),
    }
}

/// POST /age-requests/:id/withdraw — Retirer la demande
#[post("/age-requests/{id}/withdraw")]
pub async fn withdraw_age_request(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    path: web::Path<Uuid>,
) -> impl Responder {
    let organization_id = match user.require_organization() {
        Ok(id) => id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": e.to_string()}))
        }
    };

    match state
        .age_request_use_cases
        .withdraw(path.into_inner(), organization_id, user.user_id)
        .await
    {
        Ok(req) => HttpResponse::Ok().json(req),
        Err(e) => HttpResponse::BadRequest().json(serde_json::json!({"error": e})),
    }
}

/// DELETE /age-requests/:id — Supprimer une demande (Draft/Withdrawn seulement)
#[delete("/age-requests/{id}")]
pub async fn delete_age_request(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    path: web::Path<Uuid>,
) -> impl Responder {
    let organization_id = match user.require_organization() {
        Ok(id) => id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": e.to_string()}))
        }
    };

    match state
        .age_request_use_cases
        .delete(path.into_inner(), organization_id, user.user_id)
        .await
    {
        Ok(()) => HttpResponse::NoContent().finish(),
        Err(e) => {
            if e.contains("introuvable") {
                HttpResponse::NotFound().json(serde_json::json!({"error": e}))
            } else if e.contains("refusé") {
                HttpResponse::Forbidden().json(serde_json::json!({"error": e}))
            } else {
                HttpResponse::BadRequest().json(serde_json::json!({"error": e}))
            }
        }
    }
}

/// DTO de création sans le building_id (qui vient du path)
#[derive(serde::Deserialize)]
pub struct CreateAgeRequestDtoWithoutBuildingId {
    pub title: String,
    pub description: Option<String>,
}
