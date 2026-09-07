use crate::application::use_cases::boinc_use_cases::SubmitOptimisationTaskDto;
use crate::infrastructure::web::middleware::scope_guard::verify_owner_org_access;
use crate::infrastructure::web::middleware::AuthenticatedUser;
use crate::infrastructure::web::AppState;
use actix_web::{delete, get, post, web, HttpRequest, HttpResponse, ResponseError, Result};
use serde::Deserialize;
use uuid::Uuid;

// ─────────────────────────────────────────────────────────────────────────────
// MQTT Control Endpoints
// ─────────────────────────────────────────────────────────────────────────────

/// Démarre le listener MQTT Home Assistant.
///
/// POST /api/v1/iot/mqtt/start
/// Requiert rôle: syndic ou superadmin
#[post("/iot/mqtt/start")]
pub async fn start_mqtt_listener(
    state: web::Data<AppState>,
    auth: AuthenticatedUser,
) -> Result<HttpResponse> {
    // Le commentaire de cette route annonce « Requiert rôle: syndic ou
    // superadmin » depuis toujours. Rien ne le vérifiait : la règle n'existait
    // qu'en prose, et l'identité était prise puis ignorée — `_auth` (#772).
    //
    // Démarrer ou arrêter la passerelle MQTT coupe la collecte de mesures pour
    // TOUTES les copropriétés à la fois. Ce n'est pas une action de
    // copropriétaire.
    if !auth.is_superadmin() && auth.role != "syndic" {
        return Ok(HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Réservé au syndic et à l'administration de la plateforme"
        })));
    }

    match state.mqtt_energy_adapter.start_listening().await {
        Ok(()) => Ok(HttpResponse::Ok().json(serde_json::json!({
            "status": "started",
            "message": "MQTT listener started successfully"
        }))),
        Err(e) => Ok(HttpResponse::BadRequest().json(serde_json::json!({
            "error": e.to_string()
        }))),
    }
}

/// Arrête le listener MQTT Home Assistant.
///
/// POST /api/v1/iot/mqtt/stop
#[post("/iot/mqtt/stop")]
pub async fn stop_mqtt_listener(
    state: web::Data<AppState>,
    auth: AuthenticatedUser,
) -> Result<HttpResponse> {
    // Le commentaire de cette route annonce « Requiert rôle: syndic ou
    // superadmin » depuis toujours. Rien ne le vérifiait : la règle n'existait
    // qu'en prose, et l'identité était prise puis ignorée — `_auth` (#772).
    //
    // Démarrer ou arrêter la passerelle MQTT coupe la collecte de mesures pour
    // TOUTES les copropriétés à la fois. Ce n'est pas une action de
    // copropriétaire.
    if !auth.is_superadmin() && auth.role != "syndic" {
        return Ok(HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Réservé au syndic et à l'administration de la plateforme"
        })));
    }

    match state.mqtt_energy_adapter.stop_listening().await {
        Ok(()) => Ok(HttpResponse::Ok().json(serde_json::json!({
            "status": "stopped",
            "message": "MQTT listener stopped"
        }))),
        Err(e) => Ok(HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        }))),
    }
}

/// Statut du listener MQTT.
///
/// GET /api/v1/iot/mqtt/status
#[get("/iot/mqtt/status")]
pub async fn mqtt_status(
    state: web::Data<AppState>,
    auth: AuthenticatedUser,
) -> Result<HttpResponse> {
    // Démarrer, arrêter ou interroger la passerelle MQTT agit sur
    // l'infrastructure de collecte, pas sur les données d'une copropriété :
    // c'est une opération de plateforme. L'identité était prise puis ignorée
    // — `_auth` (#772).
    if !auth.is_superadmin() {
        return Ok(HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Réservé à l'administration de la plateforme"
        })));
    }

    let running = state.mqtt_energy_adapter.is_running().await;
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "running": running,
        "topic": std::env::var("MQTT_TOPIC").unwrap_or_else(|_| "koprogo/+/energy/#".to_string())
    })))
}

// ─────────────────────────────────────────────────────────────────────────────
// BOINC Grid Consent Endpoints (GDPR)
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct ConsentRequest {
    pub owner_id: Uuid,
    pub organization_id: Uuid,
    /// true = accorder, false = révoquer
    pub granted: bool,
}

/// Accorder ou révoquer le consentement BOINC (GDPR Art. 7).
///
/// POST /api/v1/iot/grid/consent
///
/// Body: { "owner_id": "uuid", "organization_id": "uuid", "granted": true|false }
#[post("/iot/grid/consent")]
pub async fn update_grid_consent(
    state: web::Data<AppState>,
    body: web::Json<ConsentRequest>,
    req: HttpRequest,
    auth: AuthenticatedUser,
) -> Result<HttpResponse> {
    let ip = req
        .connection_info()
        .realip_remote_addr()
        .map(|s| s.chars().take(45).collect::<String>());

    if body.granted {
        match state
            .boinc_use_cases
            .grant_consent(body.owner_id, body.organization_id, ip.as_deref())
            .await
        {
            Ok(consent) => Ok(HttpResponse::Ok().json(consent)),
            Err(e) => Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": e
            }))),
        }
    } else {
        match state.boinc_use_cases.revoke_consent(body.owner_id).await {
            Ok(()) => Ok(HttpResponse::Ok().json(serde_json::json!({
                "status": "consent_revoked",
                "owner_id": body.owner_id
            }))),
            Err(e) => Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": e
            }))),
        }
    }
}

/// Récupère le consentement BOINC courant d'un propriétaire.
///
/// GET /api/v1/iot/grid/consent/{owner_id}
#[get("/iot/grid/consent/{owner_id}")]
pub async fn get_grid_consent(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
    auth: AuthenticatedUser,
) -> Result<HttpResponse> {
    // Cloisonnement : le consentement au calcul distribué est une donnée
    // personnelle, rattachée à un copropriétaire NOMMÉ. La lire hors de son
    // organisation, c'est apprendre d'un tiers ce qu'il a accepté ou refusé.
    //
    // L'identité était prise puis ignorée — `_auth` (#772).
    if let Err(err) = verify_owner_org_access(&auth, *path, &state.owner_use_cases).await {
        return Ok(err.error_response());
    }

    match state.boinc_use_cases.get_consent(*path).await {
        Ok(Some(consent)) => Ok(HttpResponse::Ok().json(consent)),
        Ok(None) => Ok(HttpResponse::Ok().json(serde_json::json!({
            "owner_id": *path,
            "granted": false,
            "message": "No consent record found"
        }))),
        Err(e) => Ok(HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e
        }))),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// BOINC Grid Task Endpoints
// ─────────────────────────────────────────────────────────────────────────────

/// Soumet une tâche d'optimisation énergétique groupée à BOINC.
///
/// POST /api/v1/iot/grid/tasks
///
/// Body: { "building_id": "uuid", "owner_id": "uuid", "organization_id": "uuid", "simulation_months": 12 }
/// Requiert consentement BOINC préalable (GDPR).
#[post("/iot/grid/tasks")]
pub async fn submit_grid_task(
    state: web::Data<AppState>,
    body: web::Json<SubmitOptimisationTaskDto>,
    auth: AuthenticatedUser,
) -> Result<HttpResponse> {
    match state
        .boinc_use_cases
        .submit_optimisation_task(body.into_inner())
        .await
    {
        Ok(resp) => Ok(HttpResponse::Created().json(resp)),
        Err(e) if e.contains("not consented") => {
            Ok(HttpResponse::Forbidden().json(serde_json::json!({
                "error": e,
                "hint": "Grant BOINC consent first via POST /iot/grid/consent"
            })))
        }
        Err(e) => Ok(HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e
        }))),
    }
}

/// Récupère le statut d'une tâche BOINC.
///
/// GET /api/v1/iot/grid/tasks/{task_id}
#[get("/iot/grid/tasks/{task_id}")]
pub async fn get_task_status(
    state: web::Data<AppState>,
    path: web::Path<String>,
    auth: AuthenticatedUser,
) -> Result<HttpResponse> {
    match state.boinc_use_cases.poll_task(&path).await {
        Ok(status) => Ok(HttpResponse::Ok().json(status)),
        Err(e) if e.contains("not found") => Ok(HttpResponse::NotFound().json(serde_json::json!({
            "error": e
        }))),
        Err(e) => Ok(HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e
        }))),
    }
}

/// Annule une tâche BOINC en cours.
///
/// DELETE /api/v1/iot/grid/tasks/{task_id}
#[delete("/iot/grid/tasks/{task_id}")]
pub async fn cancel_grid_task(
    state: web::Data<AppState>,
    path: web::Path<String>,
    auth: AuthenticatedUser,
) -> Result<HttpResponse> {
    match state.boinc_use_cases.cancel_task(&path).await {
        Ok(()) => Ok(HttpResponse::Ok().json(serde_json::json!({
            "status": "cancelled",
            "task_id": *path
        }))),
        Err(e) => Ok(HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e
        }))),
    }
}
