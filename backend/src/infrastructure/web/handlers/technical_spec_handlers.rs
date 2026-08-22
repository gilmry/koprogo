//! HTTP handlers for TechnicalSpec (Story 3.8 — FR33).
//!
//! Routes:
//! - `POST /technical-specs`              — create a Draft spec (syndic/superadmin)
//! - `POST /technical-specs/{id}/bump`    — create a new version (syndic/superadmin)
//! - `POST /technical-specs/{id}/submit`  — Draft -> PendingSignatures (syndic/superadmin)
//! - `POST /technical-specs/{id}/signatures` — record a signature (signatory)
//! - `GET  /technical-specs/{id}`         — spec details
//! - `GET  /technical-specs?acp_id={uuid}` — list specs for an ACP
//!
//! All routes are JWT-protected. Scope tightening (only members of the ACP
//! may read the spec) is a Phase B follow-up tracked in the Story 3.8
//! acceptance notes.

use crate::application::error::AppError;
use crate::domain::entities::{
    SemVer, SignatoryRole, TechnicalSpec, TechnicalSpecSignature, TechnicalSpecStatus,
};
use crate::infrastructure::web::{AppState, AuthenticatedUser};
use actix_web::{get, post, web, HttpResponse};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// DTOs
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateTechnicalSpecRequest {
    pub acp_id: Uuid,
    #[serde(default)]
    pub building_id: Option<Uuid>,
    pub title: String,
    pub description: String,
    /// SemVer string (`major.minor.patch`, e.g. `1.0.0`). Strict — no
    /// `v`-prefix, no pre-release / build metadata.
    pub version: String,
    pub deliverables: Vec<String>,
    /// Roles required to sign: `syndic`, `amo`, `lawyer`, `architect`,
    /// `acp_representative`.
    pub required_signatures: Vec<String>,
    #[serde(default)]
    pub attachments: Vec<String>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct BumpTechnicalSpecRequest {
    /// New SemVer. Must be strictly greater than the previous one.
    pub version: String,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub deliverables: Option<Vec<String>>,
    #[serde(default)]
    pub required_signatures: Option<Vec<String>>,
    #[serde(default)]
    pub attachments: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct SignTechnicalSpecRequest {
    /// Role under which the caller signs. Must be in the spec's
    /// `required_signatures` list.
    pub role: String,
    /// Optional Mandate id. REQUIRED for mandataire roles (amo / lawyer /
    /// architect) — Story 3.4 chain.
    #[serde(default)]
    pub mandate_id: Option<Uuid>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct ListTechnicalSpecsQuery {
    /// Restrict the listing to the given ACP.
    pub acp_id: Uuid,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct TechnicalSpecDto {
    pub id: Uuid,
    pub acp_id: Uuid,
    pub building_id: Option<Uuid>,
    pub title: String,
    pub description: String,
    pub version: String,
    pub status: String,
    pub deliverables: Vec<String>,
    pub required_signatures: Vec<String>,
    pub attachments: Vec<String>,
    pub previous_version_id: Option<Uuid>,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<TechnicalSpec> for TechnicalSpecDto {
    fn from(s: TechnicalSpec) -> Self {
        Self {
            id: s.id,
            acp_id: s.acp_id,
            building_id: s.building_id,
            title: s.title,
            description: s.description,
            version: s.version.to_string(),
            status: s.status.to_string(),
            deliverables: s.deliverables,
            required_signatures: s
                .required_signatures
                .iter()
                .map(|r| r.to_string())
                .collect(),
            attachments: s.attachments,
            previous_version_id: s.previous_version_id,
            created_by: s.created_by,
            created_at: s.created_at,
            updated_at: s.updated_at,
        }
    }
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct TechnicalSpecSignatureDto {
    pub id: Uuid,
    pub technical_spec_id: Uuid,
    pub signatory_user_id: Uuid,
    pub role: String,
    pub mandate_id: Option<Uuid>,
    pub signed_at: DateTime<Utc>,
}

impl From<TechnicalSpecSignature> for TechnicalSpecSignatureDto {
    fn from(s: TechnicalSpecSignature) -> Self {
        Self {
            id: s.id,
            technical_spec_id: s.technical_spec_id,
            signatory_user_id: s.signatory_user_id,
            role: s.role.to_string(),
            mandate_id: s.mandate_id,
            signed_at: s.signed_at,
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
            "Only syndic or superadmin can manage a TechnicalSpec".to_string(),
        )),
    }
}

fn parse_required(strs: &[String]) -> Result<Vec<SignatoryRole>, AppError> {
    strs.iter().map(|s| SignatoryRole::from_str(s)).collect()
}

// ---------------------------------------------------------------------------
// POST /technical-specs
// ---------------------------------------------------------------------------

#[utoipa::path(
    post,
    path = "/technical-specs",
    tag = "TechnicalSpec",
    request_body = CreateTechnicalSpecRequest,
    responses(
        (status = 201, description = "Spec created (Draft)", body = TechnicalSpecDto),
        (status = 400, description = "Validation error"),
        (status = 403, description = "Forbidden — only syndic / superadmin"),
    ),
)]
#[post("/technical-specs")]
pub async fn create_technical_spec(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    body: web::Json<CreateTechnicalSpecRequest>,
) -> Result<HttpResponse, AppError> {
    require_syndic_or_superadmin(&user)?;
    let payload = body.into_inner();
    let version = SemVer::from_str(&payload.version)?;
    let required = parse_required(&payload.required_signatures)?;

    let spec = state
        .technical_spec_use_cases
        .create_spec(
            payload.acp_id,
            payload.building_id,
            payload.title,
            payload.description,
            version,
            payload.deliverables,
            required,
            payload.attachments,
            user.user_id,
        )
        .await?;
    Ok(HttpResponse::Created().json(TechnicalSpecDto::from(spec)))
}

// ---------------------------------------------------------------------------
// POST /technical-specs/{id}/bump
// ---------------------------------------------------------------------------

#[utoipa::path(
    post,
    path = "/technical-specs/{id}/bump",
    tag = "TechnicalSpec",
    request_body = BumpTechnicalSpecRequest,
    responses(
        (status = 201, description = "New version created (Draft)", body = TechnicalSpecDto),
        (status = 400, description = "Validation error"),
        (status = 403, description = "Forbidden — only syndic / superadmin"),
        (status = 404, description = "Previous spec not found"),
    ),
)]
#[post("/technical-specs/{id}/bump")]
pub async fn bump_technical_spec(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    path: web::Path<Uuid>,
    body: web::Json<BumpTechnicalSpecRequest>,
) -> Result<HttpResponse, AppError> {
    require_syndic_or_superadmin(&user)?;
    let prev_id = path.into_inner();
    let payload = body.into_inner();
    let new_version = SemVer::from_str(&payload.version)?;
    let new_required = match payload.required_signatures {
        Some(strs) => Some(parse_required(&strs)?),
        None => None,
    };

    let spec = state
        .technical_spec_use_cases
        .bump_version(
            prev_id,
            new_version,
            payload.title,
            payload.description,
            payload.deliverables,
            new_required,
            payload.attachments,
        )
        .await?;
    Ok(HttpResponse::Created().json(TechnicalSpecDto::from(spec)))
}

// ---------------------------------------------------------------------------
// POST /technical-specs/{id}/submit
// ---------------------------------------------------------------------------

#[utoipa::path(
    post,
    path = "/technical-specs/{id}/submit",
    tag = "TechnicalSpec",
    responses(
        (status = 200, description = "Spec submitted (PendingSignatures)", body = TechnicalSpecDto),
        (status = 403, description = "Forbidden — only syndic / superadmin"),
        (status = 404, description = "Spec not found"),
        (status = 409, description = "Spec already approved"),
    ),
)]
#[post("/technical-specs/{id}/submit")]
pub async fn submit_technical_spec(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    require_syndic_or_superadmin(&user)?;
    let id = path.into_inner();
    let spec = state
        .technical_spec_use_cases
        .submit_for_signatures(id)
        .await?;
    Ok(HttpResponse::Ok().json(TechnicalSpecDto::from(spec)))
}

// ---------------------------------------------------------------------------
// POST /technical-specs/{id}/signatures
// ---------------------------------------------------------------------------

#[utoipa::path(
    post,
    path = "/technical-specs/{id}/signatures",
    tag = "TechnicalSpec",
    request_body = SignTechnicalSpecRequest,
    responses(
        (status = 201, description = "Signature recorded", body = TechnicalSpecSignatureDto),
        (status = 400, description = "Spec not in PendingSignatures state"),
        (status = 403, description = "Signatory role not authorised"),
        (status = 404, description = "Spec not found"),
        (status = 409, description = "Signature already exists for (user, role)"),
    ),
)]
#[post("/technical-specs/{id}/signatures")]
pub async fn sign_technical_spec(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    path: web::Path<Uuid>,
    body: web::Json<SignTechnicalSpecRequest>,
) -> Result<HttpResponse, AppError> {
    let spec_id = path.into_inner();
    let payload = body.into_inner();
    let role = SignatoryRole::from_str(&payload.role)?;

    let sig = state
        .technical_spec_use_cases
        .sign_spec(spec_id, user.user_id, role, payload.mandate_id)
        .await?;
    Ok(HttpResponse::Created().json(TechnicalSpecSignatureDto::from(sig)))
}

// ---------------------------------------------------------------------------
// GET /technical-specs/{id}
// ---------------------------------------------------------------------------

#[utoipa::path(
    get,
    path = "/technical-specs/{id}",
    tag = "TechnicalSpec",
    responses(
        (status = 200, description = "Spec details", body = TechnicalSpecDto),
        (status = 404, description = "Spec not found"),
    ),
)]
#[get("/technical-specs/{id}")]
pub async fn get_technical_spec(
    state: web::Data<AppState>,
    _user: AuthenticatedUser,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let id = path.into_inner();
    let spec = state.technical_spec_use_cases.get(id).await?;
    Ok(HttpResponse::Ok().json(TechnicalSpecDto::from(spec)))
}

// ---------------------------------------------------------------------------
// GET /technical-specs?acp_id={uuid}
// ---------------------------------------------------------------------------

#[utoipa::path(
    get,
    path = "/technical-specs",
    tag = "TechnicalSpec",
    params(
        ("acp_id" = Uuid, Query, description = "ACP id to list specs for"),
    ),
    responses(
        (status = 200, description = "List of specs", body = Vec<TechnicalSpecDto>),
    ),
)]
#[get("/technical-specs")]
pub async fn list_technical_specs(
    state: web::Data<AppState>,
    _user: AuthenticatedUser,
    query: web::Query<ListTechnicalSpecsQuery>,
) -> Result<HttpResponse, AppError> {
    let acp_id = query.into_inner().acp_id;
    let specs = state.technical_spec_use_cases.list_for_acp(acp_id).await?;
    let dtos: Vec<TechnicalSpecDto> = specs.into_iter().map(TechnicalSpecDto::from).collect();
    Ok(HttpResponse::Ok().json(dtos))
}

// Silence unused-import warning when no consumer references the status type
// directly (utoipa derives keep it in the public surface).
#[allow(dead_code)]
fn _status_type_used(_s: TechnicalSpecStatus) {}
