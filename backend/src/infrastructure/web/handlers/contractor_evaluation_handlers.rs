//! HTTP handlers for ContractorEvaluation (Story 3.9 — FR34 FR35 INV-21
//! INV-24).
//!
//! Routes:
//! - `POST /contractor-evaluations`                            — create (syndic/superadmin)
//! - `GET  /contractor-evaluations/{id}`                       — fetch details
//! - `GET  /contractors/{contractor_user_id}/evaluations`      — list for a contractor
//!
//! All routes are JWT-protected. `GET /contractor-evaluations/{id}` took
//! `AuthenticatedUser` without using it (`_user`) and is now cloisonnée via
//! `verify_technical_spec_org_access` (#882). `GET
//! /contractors/{contractor_user_id}/evaluations` remains classified but
//! unfixed — see the comment above `list_contractor_evaluations`: a product
//! decision is needed first.

use crate::application::error::AppError;
use crate::domain::entities::{ContractorEvaluation, EvaluationScores};
use crate::infrastructure::web::middleware::scope_guard::verify_technical_spec_org_access;
use crate::infrastructure::web::{AppState, AuthenticatedUser};
use actix_web::{get, post, web, HttpResponse};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// DTOs
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct EvaluationScoresDto {
    pub quality: u8,
    pub timeliness: u8,
    pub communication: u8,
    pub cost_compliance: u8,
    pub overall: u8,
}

impl From<EvaluationScoresDto> for EvaluationScores {
    fn from(d: EvaluationScoresDto) -> Self {
        Self {
            quality: d.quality,
            timeliness: d.timeliness,
            communication: d.communication,
            cost_compliance: d.cost_compliance,
            overall: d.overall,
        }
    }
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateContractorEvaluationRequest {
    pub contractor_user_id: Uuid,
    pub technical_spec_id: Uuid,
    #[serde(default)]
    pub linked_ticket_ids: Vec<Uuid>,
    pub scores: EvaluationScoresDto,
    pub comment: String,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct EvaluationScoresOutDto {
    pub quality: u8,
    pub timeliness: u8,
    pub communication: u8,
    pub cost_compliance: u8,
    pub overall: u8,
}

impl From<EvaluationScores> for EvaluationScoresOutDto {
    fn from(s: EvaluationScores) -> Self {
        Self {
            quality: s.quality,
            timeliness: s.timeliness,
            communication: s.communication,
            cost_compliance: s.cost_compliance,
            overall: s.overall,
        }
    }
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ContractorEvaluationDto {
    pub id: Uuid,
    pub contractor_user_id: Uuid,
    pub technical_spec_id: Uuid,
    pub linked_ticket_ids: Vec<Uuid>,
    pub evaluator_user_id: Uuid,
    pub scores: EvaluationScoresOutDto,
    pub average_score: f64,
    pub comment: String,
    pub created_at: DateTime<Utc>,
}

impl From<ContractorEvaluation> for ContractorEvaluationDto {
    fn from(e: ContractorEvaluation) -> Self {
        let average = e.average_score();
        Self {
            id: e.id,
            contractor_user_id: e.contractor_user_id,
            technical_spec_id: e.technical_spec_id,
            linked_ticket_ids: e.linked_ticket_ids,
            evaluator_user_id: e.evaluator_user_id,
            scores: EvaluationScoresOutDto::from(e.scores),
            average_score: average,
            comment: e.comment,
            created_at: e.created_at,
        }
    }
}

// ---------------------------------------------------------------------------
// Guards
// ---------------------------------------------------------------------------

fn require_syndic_or_superadmin(user: &AuthenticatedUser) -> Result<(), AppError> {
    match user.role.as_str() {
        "syndic" | "superadmin" => Ok(()),
        _ => Err(AppError::Forbidden(
            "Only syndic or superadmin can record a ContractorEvaluation".to_string(),
        )),
    }
}

// ---------------------------------------------------------------------------
// POST /contractor-evaluations
// ---------------------------------------------------------------------------

#[utoipa::path(
    post,
    path = "/contractor-evaluations",
    tag = "ContractorEvaluation",
    request_body = CreateContractorEvaluationRequest,
    responses(
        (status = 201, description = "Evaluation created", body = ContractorEvaluationDto),
        (status = 400, description = "Validation error"),
        (status = 403, description = "Forbidden — only syndic / superadmin"),
        (status = 404, description = "TechnicalSpec not found"),
        (status = 422, description = "TechnicalSpec not Approved, or self-evaluation"),
    ),
)]
#[post("/contractor-evaluations")]
pub async fn create_contractor_evaluation(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    body: web::Json<CreateContractorEvaluationRequest>,
) -> Result<HttpResponse, AppError> {
    require_syndic_or_superadmin(&user)?;
    let payload = body.into_inner();
    let evaluation = state
        .contractor_evaluation_use_cases
        .create_evaluation(
            payload.contractor_user_id,
            payload.technical_spec_id,
            payload.linked_ticket_ids,
            user.user_id,
            payload.scores.into(),
            payload.comment,
        )
        .await?;
    Ok(HttpResponse::Created().json(ContractorEvaluationDto::from(evaluation)))
}

// ---------------------------------------------------------------------------
// GET /contractor-evaluations/{id}
// ---------------------------------------------------------------------------

#[utoipa::path(
    get,
    path = "/contractor-evaluations/{id}",
    tag = "ContractorEvaluation",
    responses(
        (status = 200, description = "Evaluation details", body = ContractorEvaluationDto),
        (status = 404, description = "Evaluation not found"),
    ),
)]
#[get("/contractor-evaluations/{id}")]
pub async fn get_contractor_evaluation(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let id = path.into_inner();
    let evaluation = state
        .contractor_evaluation_use_cases
        .get_evaluation(id)
        .await?;

    // Cloisonnement (#882) : l'évaluation ne porte pas d'organisation
    // directement, mais remonte à une fiche technique (`technical_spec_id`)
    // qui, elle, relève d'une ACP précise. L'identité était prise et jetée
    // (`_user`), et `comment`/`scores` sont le jugement porté sur un
    // prestataire pour un chantier qui n'a pas à être lu hors de son ACP.
    verify_technical_spec_org_access(
        &user,
        evaluation.technical_spec_id,
        &state.technical_spec_use_cases,
        &state.acp_use_cases,
    )
    .await?;

    Ok(HttpResponse::Ok().json(ContractorEvaluationDto::from(evaluation)))
}

// ---------------------------------------------------------------------------
// GET /contractors/{contractor_user_id}/evaluations
// ---------------------------------------------------------------------------

#[utoipa::path(
    get,
    path = "/contractors/{contractor_user_id}/evaluations",
    tag = "ContractorEvaluation",
    responses(
        (status = 200, description = "List of evaluations (newest first)", body = Vec<ContractorEvaluationDto>),
    ),
)]
// Cloisonnement (#882) : classée, non corrigée. L'identité est prise et
// jetée (`_user`) délibérément pour l'instant — trancher entre deux lectures
// opposées est un choix produit, pas un oubli :
// - un historique de notation d'un prestataire peut être une donnée de
//   RÉPUTATION, légitimement transverse aux ACP (un prestataire malhonnête
//   chez A intéresse B) ;
// - mais `comment` et `linked_ticket_ids` peuvent révéler des détails
//   internes d'une ACP tierce (nature exacte d'un litige, ticket lié).
// Tant que l'arbitrage n'est pas rendu, ce gestionnaire rend le détail BRUT
// hors périmètre. Un DTO agrégé (moyenne, nombre d'avis) réglerait les deux
// lectures à la fois, mais c'est un changement de contrat d'API.
#[get("/contractors/{contractor_user_id}/evaluations")]
pub async fn list_contractor_evaluations(
    state: web::Data<AppState>,
    _user: AuthenticatedUser,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let contractor_user_id = path.into_inner();
    let evaluations = state
        .contractor_evaluation_use_cases
        .list_for_contractor(contractor_user_id)
        .await?;
    let dtos: Vec<ContractorEvaluationDto> = evaluations
        .into_iter()
        .map(ContractorEvaluationDto::from)
        .collect();
    Ok(HttpResponse::Ok().json(dtos))
}
