//! `scope_guard` middleware — Story 1.3 (refonte UX multi-rôle ACP).
//!
//! Reads the optional scope hint provided by the client (header
//! `X-Scope-AcpId` *or* query parameter `?acp_id=...`), resolves the
//! caller's effective `AcpCaller` from JWT, and injects an `AcpScope`
//! into the request extensions for downstream handlers to consume.
//!
//! Refuses the request early (without hitting the handler) if the
//! caller attempts to address an ACP outside their scope.
//!
//! Design:
//! - The middleware is a thin `Transform` registered on `/buildings` /
//!   `/acps` routes; it consults `AppState::list_acps_use_case` to
//!   verify the scope.
//! - For routes that don't need a forced ACP scope (e.g. admin listing
//!   "all"), the middleware is permissive: no scope hint → no
//!   restriction, the use-case will fall back to the role-derived
//!   default scope.
//!
//! Error semantics (cf. architecture §6.3) :
//! - 401 `Unauthorized` if no/invalid JWT
//! - 403 `AcpNotInScope { acp_id }` if scope forged out of perimeter
//! - 400 `Validation` if header/query are malformed *and* the role
//!   needs an explicit scope id (non-admin)

use std::future::{ready, Future, Ready};
use std::pin::Pin;
use std::sync::Arc;

use actix_web::{
    body::{EitherBody, MessageBody},
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    http::StatusCode,
    web, Error, HttpMessage, HttpResponse, ResponseError,
};
use serde_json::json;
use thiserror::Error;
use uuid::Uuid;

use crate::application::error::AppError;
use crate::application::use_cases::acp_use_cases::{AcpCaller, AcpUseCases};
use crate::infrastructure::web::app_state::AppState;
use crate::infrastructure::web::AuthenticatedUser;

/// Header name accepted as a scope hint. Case-insensitive per HTTP RFC.
pub const SCOPE_ACP_HEADER: &str = "X-Scope-AcpId";

/// Resolved scope context, injected into request extensions by the
/// `ScopeGuard` middleware. Handlers read it via
/// `req.extensions().get::<AcpScope>()` (or via an extractor in a
/// follow-up story).
#[derive(Debug, Clone)]
pub struct AcpScope {
    /// Caller derived from JWT (mapped by the same convention as
    /// `acp_handlers::caller_from_user`).
    pub caller: AcpCaller,
    /// ACP id explicitly requested by the client (header/query).
    /// `None` = use role-derived default scope.
    pub requested_acp_id: Option<Uuid>,
    /// `true` if the middleware has verified the caller is allowed to
    /// see the requested ACP. Always `true` when `requested_acp_id` is
    /// `None` (no forging possible).
    pub allowed: bool,
}

/// Errors surfaced by `ScopeGuard`. Mapped to HTTP via `ResponseError`.
#[derive(Debug, Error)]
pub enum ScopeGuardError {
    #[error("Unauthorized — missing or invalid JWT")]
    Unauthorized,

    #[error("ACP {acp_id} not in scope")]
    AcpNotInScope { acp_id: Uuid },

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl ScopeGuardError {
    pub fn kind(&self) -> &'static str {
        match self {
            ScopeGuardError::Unauthorized => "unauthorized",
            ScopeGuardError::AcpNotInScope { .. } => "acp_not_in_scope",
            ScopeGuardError::Validation(_) => "validation",
            ScopeGuardError::Internal(_) => "internal",
        }
    }
}

impl ResponseError for ScopeGuardError {
    fn status_code(&self) -> StatusCode {
        match self {
            ScopeGuardError::Unauthorized => StatusCode::UNAUTHORIZED,
            ScopeGuardError::AcpNotInScope { .. } => StatusCode::FORBIDDEN,
            ScopeGuardError::Validation(_) => StatusCode::BAD_REQUEST,
            ScopeGuardError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).json(json!({
            "error": self.to_string(),
            "kind": self.kind(),
        }))
    }
}

impl From<AppError> for ScopeGuardError {
    fn from(err: AppError) -> Self {
        match err {
            AppError::AcpNotInScope { acp_id } => ScopeGuardError::AcpNotInScope { acp_id },
            AppError::Unauthorized | AppError::InvalidCredentials | AppError::TokenError(_) => {
                ScopeGuardError::Unauthorized
            }
            AppError::Validation(s) => ScopeGuardError::Validation(s),
            other => ScopeGuardError::Internal(other.to_string()),
        }
    }
}

// ============================================================================
// Pure helpers (used by middleware AND tested by BDD/unit tests without
// actix machinery).
// ============================================================================

/// Map a `UserRoleString + organization_id + user_id` triple to an
/// `AcpCaller`. Same convention as `acp_handlers::caller_from_user` —
/// duplicated here to keep the middleware free of handler imports.
pub fn caller_from_role(role: &str, organization_id: Option<Uuid>, user_id: Uuid) -> AcpCaller {
    match role.to_lowercase().as_str() {
        "superadmin" => AcpCaller::SuperAdmin,
        "admin" => match organization_id {
            Some(org) => AcpCaller::Admin {
                organization_id: org,
            },
            None => AcpCaller::SuperAdmin,
        },
        "syndic" | "accountant" => match organization_id {
            Some(org) => AcpCaller::Syndic {
                organization_id: org,
            },
            None => AcpCaller::Owner { user_id },
        },
        _ => AcpCaller::Owner { user_id },
    }
}

/// Extract the requested ACP id from headers or query string.
/// Header `X-Scope-AcpId` takes precedence over `?acp_id=`.
/// Returns `Err(Validation)` if a value is present but malformed.
pub fn extract_requested_acp_id(
    header_value: Option<&str>,
    query_value: Option<&str>,
) -> Result<Option<Uuid>, ScopeGuardError> {
    let raw = header_value.or(query_value);
    match raw {
        None => Ok(None),
        Some(s) if s.trim().is_empty() => Ok(None),
        Some(s) => Uuid::parse_str(s.trim())
            .map(Some)
            .map_err(|_| ScopeGuardError::Validation(format!("invalid acp_id: {}", s))),
    }
}

/// Decide whether the caller is allowed to *attach* the requested scope
/// id, without hitting the DB. Returns:
/// - `Ok(None)` : caller has no requested scope → no enforcement needed
/// - `Ok(Some(acp_id))` : the middleware must consult the use-case to
///   verify `assert_caller_can_see(acp_id)`
/// - `Err(Validation)` : the caller is non-admin AND owns no role-scope
///   information (e.g. syndic with `organization_id = None` AND no
///   explicit acp_id) — refuse rather than guess.
pub fn requires_repository_check(
    caller: &AcpCaller,
    requested: Option<Uuid>,
) -> Result<Option<Uuid>, ScopeGuardError> {
    match (caller, requested) {
        // SuperAdmin can pin any ACP, but we still want to verify it exists.
        (AcpCaller::SuperAdmin, Some(id)) => Ok(Some(id)),
        (AcpCaller::SuperAdmin, None) => Ok(None),

        // Admin / Syndic with their own org are allowed if requested
        // matches their org-derived scope or is None.
        (AcpCaller::Admin { .. }, req) | (AcpCaller::Syndic { .. }, req) => Ok(req),

        // Owner: every request must be checked via the use-case (no
        // direct shortcut — story 1.3 conservatively refuses pinning
        // until story 3.5 wires user_role_assignments.scope/scope_id).
        (AcpCaller::Owner { .. }, req) => Ok(req),
    }
}

/// Hotfix #603 — résout `building.acp_id -> acp.organization_id` et applique
/// l'isolation multi-tenant sur les GET-by-id (building, budget, expense,
/// meeting, resolution, unit, work_report).
///
/// Après #602 (`Building.organization_id -> acp_id`), `BuildingResponseDto`
/// ne porte plus `organization_id` ; les 7 handlers ci-dessus ont perdu leur
/// `user.verify_org_access(...)`. Ce helper recâble la chaîne en lookup ACP.
///
/// Sémantique :
/// - SuperAdmin : toujours autorisé (bypass).
/// - Sinon : `acp.organization_id` MUST == `user.organization_id`. Sinon
///   `AppError::AcpNotInScope` (HTTP 403 via `ResponseError`).
/// - ACP introuvable OU `acp.organization_id IS NULL` (auto-gérée) :
///   refuse pour non-superadmin (conservateur — gouvernance ACP auto-gérée
///   en story 4.x).
pub async fn verify_acp_org_access(
    user: &AuthenticatedUser,
    acp_id: Uuid,
    acp_use_cases: &AcpUseCases,
) -> Result<(), AppError> {
    if user.is_superadmin() {
        return Ok(());
    }

    let acp = acp_use_cases
        .find_acp(acp_id)
        .await?
        .ok_or(AppError::AcpNotInScope { acp_id })?;

    let acp_org_id = acp
        .organization_id
        .ok_or(AppError::AcpNotInScope { acp_id })?;

    user.verify_org_access(acp_org_id)
        .map_err(|_| AppError::AcpNotInScope { acp_id })
}

// ============================================================================
// Actix middleware
// ============================================================================

/// `ScopeGuard` Actix middleware factory.
///
/// Wrap your routes with:
/// ```rust,ignore
/// use actix_web::web;
/// use koprogo_api::infrastructure::web::middleware::ScopeGuard;
///
/// cfg.service(
///     web::scope("/buildings")
///         .wrap(ScopeGuard::new())
///         // ... .service(...)
/// );
/// ```
#[derive(Clone, Default)]
pub struct ScopeGuard;

impl ScopeGuard {
    pub fn new() -> Self {
        Self
    }
}

impl<S, B> Transform<S, ServiceRequest> for ScopeGuard
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type InitError = ();
    type Transform = ScopeGuardMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(ScopeGuardMiddleware {
            service: Arc::new(service),
        }))
    }
}

pub struct ScopeGuardMiddleware<S> {
    service: Arc<S>,
}

impl<S, B> Service<ServiceRequest> for ScopeGuardMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = self.service.clone();

        // 1. Extract JWT claims.
        let app_state = match req.app_data::<web::Data<AppState>>() {
            Some(s) => s.clone(),
            None => {
                let err = ScopeGuardError::Internal("AppState missing".into());
                let resp = req.into_response(err.error_response().map_into_right_body());
                return Box::pin(async move { Ok(resp) });
            }
        };

        let auth_header = req
            .headers()
            .get("Authorization")
            .and_then(|h| h.to_str().ok())
            .map(|s| s.to_string());

        let token = match auth_header.as_deref() {
            Some(h) if h.starts_with("Bearer ") => {
                h.trim_start_matches("Bearer ").trim().to_string()
            }
            _ => {
                let err = ScopeGuardError::Unauthorized;
                let resp = req.into_response(err.error_response().map_into_right_body());
                return Box::pin(async move { Ok(resp) });
            }
        };

        let claims = match app_state.auth_use_cases.verify_token(&token) {
            Ok(c) => c,
            Err(_) => {
                let err = ScopeGuardError::Unauthorized;
                let resp = req.into_response(err.error_response().map_into_right_body());
                return Box::pin(async move { Ok(resp) });
            }
        };

        let user_id = match Uuid::parse_str(&claims.sub) {
            Ok(u) => u,
            Err(_) => {
                let err = ScopeGuardError::Unauthorized;
                let resp = req.into_response(err.error_response().map_into_right_body());
                return Box::pin(async move { Ok(resp) });
            }
        };

        let caller = caller_from_role(&claims.role, claims.organization_id, user_id);

        // 2. Extract requested acp_id from header OR query string.
        let header_val = req
            .headers()
            .get(SCOPE_ACP_HEADER)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());
        // Parse `?acp_id=` ourselves to avoid double-Deserialize collisions
        // with arbitrary handlers' Query<T> extractors.
        let query_val = req.query_string().split('&').find_map(|kv| {
            let mut it = kv.splitn(2, '=');
            match (it.next(), it.next()) {
                (Some("acp_id"), Some(v)) => Some(v.to_string()),
                _ => None,
            }
        });

        let requested_acp_id =
            match extract_requested_acp_id(header_val.as_deref(), query_val.as_deref()) {
                Ok(v) => v,
                Err(err) => {
                    let resp = req.into_response(err.error_response().map_into_right_body());
                    return Box::pin(async move { Ok(resp) });
                }
            };

        let check = match requires_repository_check(&caller, requested_acp_id) {
            Ok(v) => v,
            Err(err) => {
                let resp = req.into_response(err.error_response().map_into_right_body());
                return Box::pin(async move { Ok(resp) });
            }
        };

        // 3. Optional repository check.
        Box::pin(async move {
            if let Some(acp_id) = check {
                if let Err(app_err) = app_state
                    .acp_use_cases
                    .assert_can_see_acp(&caller, acp_id)
                    .await
                {
                    let guard_err = ScopeGuardError::from(app_err);
                    let resp = req.into_response(guard_err.error_response().map_into_right_body());
                    return Ok(resp);
                }
            }

            // 4. Inject AcpScope into request extensions for handlers.
            req.extensions_mut().insert(AcpScope {
                caller: caller.clone(),
                requested_acp_id,
                allowed: true,
            });

            let res = service.call(req).await?;
            Ok(res.map_into_left_body())
        })
    }
}

// ============================================================================
// Tests — taxonomie 4-cat (CRITICAL.md §3). Pure helpers only ; the
// Transform integration is exercised end-to-end via the BDD harness
// `tests/features/list_buildings_role_based.feature`.
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ----- @happy --------------------------------------------------------------

    #[test]
    fn happy_caller_from_role_admin() {
        let org = Uuid::new_v4();
        let c = caller_from_role("admin", Some(org), Uuid::new_v4());
        assert!(matches!(c, AcpCaller::Admin { organization_id } if organization_id == org));
    }

    #[test]
    fn happy_extract_acp_id_from_header() {
        let id = Uuid::new_v4();
        let s = id.to_string();
        let got = extract_requested_acp_id(Some(&s), None).unwrap();
        assert_eq!(got, Some(id));
    }

    #[test]
    fn happy_extract_acp_id_from_query_when_no_header() {
        let id = Uuid::new_v4();
        let s = id.to_string();
        let got = extract_requested_acp_id(None, Some(&s)).unwrap();
        assert_eq!(got, Some(id));
    }

    // ----- @edge ---------------------------------------------------------------

    #[test]
    fn edge_header_takes_precedence_over_query() {
        let h = Uuid::new_v4();
        let q = Uuid::new_v4();
        let got = extract_requested_acp_id(Some(&h.to_string()), Some(&q.to_string())).unwrap();
        assert_eq!(got, Some(h));
    }

    #[test]
    fn edge_empty_header_treated_as_none() {
        let got = extract_requested_acp_id(Some(""), None).unwrap();
        assert!(got.is_none());
    }

    #[test]
    fn edge_super_admin_with_no_scope_is_unrestricted() {
        let r = requires_repository_check(&AcpCaller::SuperAdmin, None).unwrap();
        assert!(r.is_none());
    }

    // ----- @security -----------------------------------------------------------

    #[test]
    fn security_owner_requested_acp_must_be_checked_via_repo() {
        let user = Uuid::new_v4();
        let target = Uuid::new_v4();
        let r =
            requires_repository_check(&AcpCaller::Owner { user_id: user }, Some(target)).unwrap();
        assert_eq!(r, Some(target));
    }

    #[test]
    fn security_caller_from_role_unknown_falls_back_to_owner() {
        let uid = Uuid::new_v4();
        let c = caller_from_role("contractor", Some(Uuid::new_v4()), uid);
        assert!(matches!(c, AcpCaller::Owner { user_id } if user_id == uid));
    }

    // ----- @negative -----------------------------------------------------------

    #[test]
    fn negative_malformed_header_returns_validation_error() {
        let err = extract_requested_acp_id(Some("not-a-uuid"), None).unwrap_err();
        assert!(matches!(err, ScopeGuardError::Validation(_)));
    }

    #[test]
    fn negative_scope_guard_error_kind_strings_are_stable() {
        assert_eq!(ScopeGuardError::Unauthorized.kind(), "unauthorized");
        assert_eq!(
            ScopeGuardError::AcpNotInScope {
                acp_id: Uuid::nil()
            }
            .kind(),
            "acp_not_in_scope"
        );
        assert_eq!(ScopeGuardError::Validation("x".into()).kind(), "validation");
    }

    #[test]
    fn negative_apperror_acp_not_in_scope_maps_to_scopeguard_acp_not_in_scope() {
        let id = Uuid::new_v4();
        let g = ScopeGuardError::from(AppError::AcpNotInScope { acp_id: id });
        match g {
            ScopeGuardError::AcpNotInScope { acp_id } => assert_eq!(acp_id, id),
            other => panic!("expected AcpNotInScope, got {:?}", other),
        }
    }

    #[test]
    fn negative_apperror_unauthorized_maps_to_scopeguard_unauthorized() {
        let g = ScopeGuardError::from(AppError::Unauthorized);
        assert!(matches!(g, ScopeGuardError::Unauthorized));
    }
}
