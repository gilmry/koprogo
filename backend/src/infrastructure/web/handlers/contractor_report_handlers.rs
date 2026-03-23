use crate::application::dto::contractor_report_dto::{
    CreateContractorReportDto, GenerateMagicLinkDto, RejectReportDto, RequestCorrectionsDto,
    UpdateContractorReportDto,
};
use crate::infrastructure::web::{AppState, AuthenticatedUser};
use actix_web::{delete, get, post, put, web, HttpRequest, HttpResponse, Responder};
use serde::Deserialize;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Endpoints authentifiés (syndic / CdC)
// ---------------------------------------------------------------------------

/// POST /contractor-reports — Créer un rapport (syndic ou système)
#[post("/contractor-reports")]
pub async fn create_contractor_report(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    body: web::Json<CreateContractorReportDto>,
) -> impl Responder {
    let organization_id = match user.require_organization() {
        Ok(id) => id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": e.to_string()}))
        }
    };

    match state
        .contractor_report_use_cases
        .create(organization_id, body.into_inner())
        .await
    {
        Ok(r) => HttpResponse::Created().json(r),
        Err(e) => HttpResponse::BadRequest().json(serde_json::json!({"error": e})),
    }
}

/// GET /contractor-reports/:id — Détail d'un rapport (authentifié)
#[get("/contractor-reports/{id}")]
pub async fn get_contractor_report(
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
        .contractor_report_use_cases
        .get(path.into_inner(), organization_id)
        .await
    {
        Ok(r) => HttpResponse::Ok().json(r),
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

/// GET /buildings/:building_id/contractor-reports — Liste des rapports d'un bâtiment
#[get("/buildings/{building_id}/contractor-reports")]
pub async fn list_contractor_reports_by_building(
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
        .contractor_report_use_cases
        .list_by_building(path.into_inner(), organization_id)
        .await
    {
        Ok(r) => HttpResponse::Ok().json(r),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e})),
    }
}

/// GET /tickets/:ticket_id/contractor-reports — Rapports liés à un ticket
#[get("/tickets/{ticket_id}/contractor-reports")]
pub async fn list_contractor_reports_by_ticket(
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
        .contractor_report_use_cases
        .list_by_ticket(path.into_inner(), organization_id)
        .await
    {
        Ok(r) => HttpResponse::Ok().json(r),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e})),
    }
}

/// PUT /contractor-reports/:id — Modifier le brouillon
#[put("/contractor-reports/{id}")]
pub async fn update_contractor_report(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    path: web::Path<Uuid>,
    body: web::Json<UpdateContractorReportDto>,
) -> impl Responder {
    let organization_id = match user.require_organization() {
        Ok(id) => id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": e.to_string()}))
        }
    };

    match state
        .contractor_report_use_cases
        .update(path.into_inner(), organization_id, body.into_inner())
        .await
    {
        Ok(r) => HttpResponse::Ok().json(r),
        Err(e) => HttpResponse::BadRequest().json(serde_json::json!({"error": e})),
    }
}

/// POST /contractor-reports/:id/submit — Soumettre pour validation CdC (auth)
#[post("/contractor-reports/{id}/submit")]
pub async fn submit_contractor_report(
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
        .contractor_report_use_cases
        .submit(path.into_inner(), organization_id)
        .await
    {
        Ok(r) => HttpResponse::Ok().json(r),
        Err(e) => HttpResponse::BadRequest().json(serde_json::json!({"error": e})),
    }
}

/// PUT /contractor-reports/:id/validate — CdC valide le rapport → paiement auto
#[put("/contractor-reports/{id}/validate")]
pub async fn validate_contractor_report(
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
        .contractor_report_use_cases
        .validate(path.into_inner(), organization_id, user.user_id)
        .await
    {
        Ok(r) => HttpResponse::Ok().json(r),
        Err(e) => HttpResponse::BadRequest().json(serde_json::json!({"error": e})),
    }
}

/// PUT /contractor-reports/:id/request-corrections — CdC demande des corrections
#[put("/contractor-reports/{id}/request-corrections")]
pub async fn request_corrections(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    path: web::Path<Uuid>,
    body: web::Json<RequestCorrectionsDto>,
) -> impl Responder {
    let organization_id = match user.require_organization() {
        Ok(id) => id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": e.to_string()}))
        }
    };

    match state
        .contractor_report_use_cases
        .request_corrections(path.into_inner(), organization_id, body.into_inner())
        .await
    {
        Ok(r) => HttpResponse::Ok().json(r),
        Err(e) => HttpResponse::BadRequest().json(serde_json::json!({"error": e})),
    }
}

/// PUT /contractor-reports/:id/reject — CdC rejette le rapport
#[put("/contractor-reports/{id}/reject")]
pub async fn reject_contractor_report(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    path: web::Path<Uuid>,
    body: web::Json<RejectReportDto>,
) -> impl Responder {
    let organization_id = match user.require_organization() {
        Ok(id) => id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": e.to_string()}))
        }
    };

    match state
        .contractor_report_use_cases
        .reject(
            path.into_inner(),
            organization_id,
            body.into_inner(),
            user.user_id,
        )
        .await
    {
        Ok(r) => HttpResponse::Ok().json(r),
        Err(e) => HttpResponse::BadRequest().json(serde_json::json!({"error": e})),
    }
}

/// POST /contractor-reports/magic-link — Génère un magic link pour le corps de métier
#[post("/contractor-reports/magic-link")]
pub async fn generate_magic_link(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    req: HttpRequest,
    body: web::Json<GenerateMagicLinkDto>,
) -> impl Responder {
    let organization_id = match user.require_organization() {
        Ok(id) => id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({"error": e.to_string()}))
        }
    };

    // Déduire la base URL depuis la requête (drop connection_info avant l'await)
    let base_url = {
        let connection_info = req.connection_info();
        format!("{}://{}", connection_info.scheme(), connection_info.host())
    };

    match state
        .contractor_report_use_cases
        .generate_magic_link(body.report_id, organization_id, &base_url)
        .await
    {
        Ok(r) => HttpResponse::Ok().json(r),
        Err(e) => HttpResponse::BadRequest().json(serde_json::json!({"error": e})),
    }
}

/// DELETE /contractor-reports/:id — Supprimer un rapport (Draft seulement)
#[delete("/contractor-reports/{id}")]
pub async fn delete_contractor_report(
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
        .contractor_report_use_cases
        .delete(path.into_inner(), organization_id)
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

// ---------------------------------------------------------------------------
// Endpoints PWA sans authentification (magic link)
// ---------------------------------------------------------------------------

/// GET /contractor/token/:token — PWA corps de métier : voir son rapport via magic link
#[get("/contractor/token/{token}")]
pub async fn get_report_by_token(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> impl Responder {
    match state
        .contractor_report_use_cases
        .get_by_token(&path.into_inner())
        .await
    {
        Ok(r) => HttpResponse::Ok().json(r),
        Err(e) => HttpResponse::Unauthorized().json(serde_json::json!({"error": e})),
    }
}

/// POST /contractor/token/:token/submit — PWA corps de métier : soumettre via magic link
#[post("/contractor/token/{token}/submit")]
pub async fn submit_report_by_token(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> impl Responder {
    match state
        .contractor_report_use_cases
        .submit_by_token(&path.into_inner())
        .await
    {
        Ok(r) => HttpResponse::Ok().json(r),
        Err(e) => HttpResponse::BadRequest().json(serde_json::json!({"error": e})),
    }
}

// ---------------------------------------------------------------------------
// New PWA endpoints with improved magic link UX (Issue #275)
// ---------------------------------------------------------------------------

/// GET /contractor-reports/magic/:token — PWA contractor: view report via magic link (no auth)
/// Issue #275: Contractor PWA Backoffice Refinements
#[get("/contractor-reports/magic/{token}")]
pub async fn get_report_by_magic_token(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> impl Responder {
    match state
        .contractor_report_use_cases
        .get_by_token(&path.into_inner())
        .await
    {
        Ok(r) => HttpResponse::Ok().json(r),
        Err(e) => HttpResponse::Unauthorized().json(serde_json::json!({"error": e})),
    }
}

/// POST /contractor-reports/magic/:token/submit — PWA contractor: submit report via magic link (no auth)
/// Issue #275: Contractor PWA Backoffice Refinements
/// Accepts updated report data in body
#[derive(Deserialize)]
pub struct MagicLinkSubmitDto {
    pub work_date: Option<String>,
    pub contractor_name: Option<String>,
    pub compte_rendu: Option<String>,
    pub parts_replaced: Option<Vec<serde_json::Value>>,
    pub photos_before: Option<Vec<String>>,
    pub photos_after: Option<Vec<String>>,
}

#[post("/contractor-reports/magic/{token}/submit")]
pub async fn submit_report_by_magic_token(
    state: web::Data<AppState>,
    path: web::Path<String>,
    _body: web::Json<MagicLinkSubmitDto>,
) -> impl Responder {
    let token = path.into_inner();

    // First, get the report by token to validate it exists and is in Draft state
    match state.contractor_report_use_cases.get_by_token(&token).await {
        Ok(_report) => {
            // Report exists, now submit it
            match state
                .contractor_report_use_cases
                .submit_by_token(&token)
                .await
            {
                Ok(r) => HttpResponse::Ok().json(r),
                Err(e) => HttpResponse::BadRequest().json(serde_json::json!({"error": e})),
            }
        }
        Err(e) => HttpResponse::Unauthorized().json(serde_json::json!({"error": e})),
    }
}
