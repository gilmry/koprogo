// Story 1.3 — middleware scope_guard (ListScope + AcpNotInScope enforcement).
pub mod scope_guard;
pub use scope_guard::{AcpScope, ScopeGuard, ScopeGuardError};

use crate::infrastructure::web::app_state::AppState;
use actix_web::{
    body::MessageBody,
    dev::{forward_ready, Payload, Service, ServiceRequest, ServiceResponse, Transform},
    error::ErrorUnauthorized,
    http::StatusCode,
    web, Error, FromRequest, HttpRequest, HttpResponse,
};
use std::collections::HashMap;
use std::future::{ready, Future, Ready};
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use uuid::Uuid;

/// Authenticated user claims extracted from JWT token
///
/// This struct automatically extracts and validates JWT tokens from the Authorization header.
/// Use it as a parameter in your handler functions to require authentication:
///
/// ```rust,ignore
/// use actix_web::Responder;
/// use koprogo_api::infrastructure::web::middleware::AuthenticatedUser;
///
/// async fn protected_handler(claims: AuthenticatedUser) -> impl Responder {
///     // claims.user_id and claims.organization_id are now available
/// }
/// ```
#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub user_id: Uuid,
    pub email: String,
    pub role: String,
    pub role_id: Option<Uuid>,
    pub organization_id: Option<Uuid>,
}

impl AuthenticatedUser {
    /// Get the organization_id or return an error if not present
    pub fn require_organization(&self) -> Result<Uuid, Error> {
        self.organization_id
            .ok_or_else(|| ErrorUnauthorized("User does not belong to an organization"))
    }

    /// Check if user is superadmin (can access all organizations)
    pub fn is_superadmin(&self) -> bool {
        self.role == "superadmin"
    }

    /// Get effective organization_id for filtering:
    /// - SuperAdmin: None (sees everything)
    /// - Others: Some(org_id)
    pub fn effective_org_filter(&self) -> Option<Uuid> {
        if self.is_superadmin() {
            None
        } else {
            self.organization_id
        }
    }

    /// Verify that a resource's organization matches the user's organization.
    /// SuperAdmin bypasses this check.
    /// Returns Ok(()) if access is allowed, Err(message) if denied.
    pub fn verify_org_access(&self, resource_org_id: Uuid) -> Result<(), String> {
        if self.is_superadmin() {
            return Ok(());
        }
        match self.organization_id {
            Some(user_org_id) if user_org_id == resource_org_id => Ok(()),
            Some(_) => Err("Access denied: resource belongs to another organization".to_string()),
            None => Err("User does not belong to an organization".to_string()),
        }
    }
}

impl FromRequest for AuthenticatedUser {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        // Get AppState from request
        let app_state = match req.app_data::<web::Data<AppState>>() {
            Some(state) => state,
            None => return ready(Err(ErrorUnauthorized("Internal server error"))),
        };

        // Extract Authorization header
        let auth_header = match req.headers().get("Authorization") {
            Some(header) => match header.to_str() {
                Ok(s) => s,
                Err(_) => return ready(Err(ErrorUnauthorized("Invalid authorization header"))),
            },
            None => return ready(Err(ErrorUnauthorized("Missing authorization header"))),
        };

        // Extract token from "Bearer <token>"
        let token = auth_header.trim_start_matches("Bearer ").trim();

        // Verify token and extract claims
        match app_state.auth_use_cases.verify_token(token) {
            Ok(claims) => {
                // Parse user_id from claims.sub
                match Uuid::parse_str(&claims.sub) {
                    Ok(user_id) => ready(Ok(AuthenticatedUser {
                        user_id,
                        email: claims.email,
                        role: claims.role,
                        role_id: claims.role_id,
                        organization_id: claims.organization_id,
                    })),
                    Err(_) => ready(Err(ErrorUnauthorized("Invalid user ID in token"))),
                }
            }
            Err(e) => ready(Err(ErrorUnauthorized(e))),
        }
    }
}

/// Organization ID extracted from authenticated user's JWT token
///
/// This extractor requires that the user belongs to an organization.
/// Use it when you need to enforce organization-scoped operations:
///
/// ```rust,ignore
/// use actix_web::{Responder, web};
/// use koprogo_api::application::dto::CreateBuildingDto;
/// use koprogo_api::infrastructure::web::middleware::OrganizationId;
///
/// async fn create_building(
///     organization: OrganizationId,
///     dto: web::Json<CreateBuildingDto>
/// ) -> impl Responder {
///     // organization.0 contains the Uuid
/// }
/// ```
#[derive(Debug, Clone, Copy)]
pub struct OrganizationId(pub Uuid);

impl FromRequest for OrganizationId {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
        // First extract AuthenticatedUser
        let user_future = AuthenticatedUser::from_request(req, payload);

        // Get the result
        match user_future.into_inner() {
            Ok(user) => match user.organization_id {
                Some(org_id) => ready(Ok(OrganizationId(org_id))),
                None => ready(Err(ErrorUnauthorized(
                    "User does not belong to an organization",
                ))),
            },
            Err(e) => ready(Err(e)),
        }
    }
}

// ========================================
// GDPR Rate Limiting Middleware
// ========================================

/// Configuration for GDPR rate limiting
#[derive(Clone, Debug)]
pub struct GdprRateLimitConfig {
    /// Maximum number of requests allowed per window
    pub max_requests: usize,
    /// Duration of the rate limit window
    pub window_duration: Duration,
}

impl Default for GdprRateLimitConfig {
    fn default() -> Self {
        Self {
            max_requests: 10,
            window_duration: Duration::from_secs(3600), // 1 hour
        }
    }
}

/// Rate limit state tracking
#[derive(Clone)]
pub struct GdprRateLimitState {
    state: Arc<Mutex<HashMap<String, (usize, Instant)>>>,
    config: GdprRateLimitConfig,
}

impl GdprRateLimitState {
    pub fn new(config: GdprRateLimitConfig) -> Self {
        Self {
            state: Arc::new(Mutex::new(HashMap::new())),
            config,
        }
    }

    /// Check if user has exceeded rate limit
    pub fn check_rate_limit(&self, user_id: &str) -> Result<(), String> {
        let mut state = self.state.lock().unwrap();
        let now = Instant::now();
        let entry = state.entry(user_id.to_string()).or_insert((0, now));
        let (count, window_start) = entry;

        // Reset window if expired
        if now.duration_since(*window_start) > self.config.window_duration {
            *count = 0;
            *window_start = now;
        }

        // Check limit
        if *count >= self.config.max_requests {
            let reset_in = self
                .config
                .window_duration
                .saturating_sub(now.duration_since(*window_start));
            return Err(format!(
                "Rate limit exceeded. Try again in {} seconds.",
                reset_in.as_secs()
            ));
        }

        *count += 1;
        Ok(())
    }
}

/// GDPR-specific rate limiting middleware
///
/// Only applies rate limits to GDPR-related endpoints:
/// - `/api/v1/gdpr/*`
/// - `/api/v1/admin/gdpr/*`
#[derive(Clone)]
pub struct GdprRateLimit {
    state: GdprRateLimitState,
}

impl GdprRateLimit {
    pub fn new(config: GdprRateLimitConfig) -> Self {
        Self {
            state: GdprRateLimitState::new(config),
        }
    }
}

impl<S, B> Transform<S, ServiceRequest> for GdprRateLimit
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<actix_web::body::EitherBody<B>>;
    type Error = Error;
    type InitError = ();
    type Transform = GdprRateLimitMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(GdprRateLimitMiddleware {
            service: Arc::new(service),
            state: self.state.clone(),
        }))
    }
}

pub struct GdprRateLimitMiddleware<S> {
    service: Arc<S>,
    state: GdprRateLimitState,
}

impl<S, B> Service<ServiceRequest> for GdprRateLimitMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<actix_web::body::EitherBody<B>>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let path = req.path().to_string();

        // Only apply rate limiting to GDPR endpoints
        let is_gdpr_endpoint =
            path.starts_with("/api/v1/gdpr") || path.starts_with("/api/v1/admin/gdpr");

        if !is_gdpr_endpoint {
            let fut = self.service.call(req);
            return Box::pin(async move { fut.await.map(|res| res.map_into_left_body()) });
        }

        // Extract user_id from AuthenticatedUser
        let user_id = match req.app_data::<web::Data<AppState>>() {
            Some(app_state) => {
                // Extract Authorization header
                let auth_header = match req.headers().get("Authorization") {
                    Some(header) => match header.to_str() {
                        Ok(s) => s.to_string(),
                        Err(_) => {
                            // Let the handler deal with invalid auth
                            let fut = self.service.call(req);
                            return Box::pin(async move {
                                fut.await.map(|res| res.map_into_left_body())
                            });
                        }
                    },
                    None => {
                        // Let the handler deal with missing auth
                        let fut = self.service.call(req);
                        return Box::pin(
                            async move { fut.await.map(|res| res.map_into_left_body()) },
                        );
                    }
                };

                let token = auth_header.trim_start_matches("Bearer ").trim();

                match app_state.auth_use_cases.verify_token(token) {
                    Ok(claims) => claims.sub,
                    Err(_) => {
                        // Let the handler deal with invalid token
                        let fut = self.service.call(req);
                        return Box::pin(
                            async move { fut.await.map(|res| res.map_into_left_body()) },
                        );
                    }
                }
            }
            None => {
                let fut = self.service.call(req);
                return Box::pin(async move { fut.await.map(|res| res.map_into_left_body()) });
            }
        };

        // Check rate limit
        let state = self.state.clone();
        let service = self.service.clone();

        Box::pin(async move {
            match state.check_rate_limit(&user_id) {
                Ok(_) => {
                    // Rate limit not exceeded, proceed with request
                    service.call(req).await.map(|res| res.map_into_left_body())
                }
                Err(msg) => {
                    // Rate limit exceeded, return 429
                    let retry_after = state.config.window_duration.as_secs().to_string();
                    let response = HttpResponse::build(StatusCode::TOO_MANY_REQUESTS)
                        .insert_header(("Retry-After", retry_after.clone()))
                        .json(serde_json::json!({
                            "error": msg,
                            "retry_after_seconds": state.config.window_duration.as_secs()
                        }));

                    Ok(req.into_response(response).map_into_right_body())
                }
            }
        })
    }
}

// ========================================
// Global Rate Limiting (Issue #78)
// ========================================
//
// IP-based rate limiting (public endpoints + /auth/login brute-force
// protection) is enforced by Traefik middlewares, not the application — see
// the `traefik.http.middlewares.*.ratelimit.*` labels on the backend service
// in docker-compose.yml / docker-compose.prod.yml and the koprogo-rate-limit
// Middleware CRD in infrastructure/_shared/kustomize/base/ingress.yaml.
// GdprRateLimit above remains application-side because it is per
// authenticated user (JWT identity), which Traefik cannot see.

// ========================================
// Request Concurrency Limit (Issue #718)
// ========================================
//
// Constat #718 : sous rafale (2 workers Playwright créant des lots en
// séquence via `seedConformantUnits()`), une partie des `POST /units` et
// `GET /acps` sur la démo prod remontait en 502 Bad Gateway ou en timeout
// client 10-30s. Le pool sqlx (`DB_POOL_MAX_CONNECTIONS=10`,
// `acquire_timeout=30s`, cf. `infrastructure/database/pool.rs`) absorbe une
// pointe en faisant *attendre* les requêtes en excès jusqu'à 30s avant
// d'échouer — un délai qui colle exactement à la fenêtre observée, et qui ne
// dit rien à l'appelant : a-t-il été pris en compte ou non ?
//
// Ce middleware ferme la porte plus tôt et plus clairement : au-delà de
// `max_concurrent` requêtes en vol simultanément (toutes routes confondues,
// tous workers Actix confondus puisque le sémaphore est partagé), les
// suivantes reçoivent un 429 immédiat avec `Retry-After`, plutôt que
// d'attendre une connexion de pool qui n'arrivera peut-être jamais à temps.
//
// Ce n'est ni un remplacement du rate-limit Traefik par IP (abus / brute
// force), ni du rate-limit GDPR par utilisateur : ce middleware protège la
// ressource partagée (le pool de connexions, le seul vCPU de la VPS), pas
// l'identité de l'appelant — d'où son application uniforme, sans
// distinction d'IP ni de JWT. Le refus est transitoire : dès qu'un slot se
// libère, la requête suivante repasse — ce n'est pas un bannissement (celui
// de CrowdSec du 2026-09-01 reste seul juge de l'abus).
//
// `/health` est explicitement exclu : une sonde de vivacité étouffée par la
// charge applicative ferait déclarer le conteneur unhealthy et le ferait
// redémarrer (`restart: unless-stopped`) — ce qui aggraverait l'incident
// au lieu de l'absorber.
//
// Défaut conservateur (`DEFAULT_MAX_CONCURRENT_REQUESTS`), configurable via
// `MAX_CONCURRENT_REQUESTS` — à ajuster une fois le rejeu du scénario fait
// sur la pile de recette (ADR 0050, cf. DoD #718 : « la cause nommée avant
// tout correctif »). Ce middleware ne tranche pas contention d'hôte vs
// applicatif ; il rend le refus déterministe quel que soit le verdict, et
// n'a desserré aucune limite existante (Traefik, GDPR) pour l'obtenir.

/// Conservative starting point: comfortably above the sqlx pool's
/// `max_connections` default (10, cf. `DB_POOL_MAX_CONNECTIONS`) since a
/// single request typically issues several short, sequential queries rather
/// than holding one connection for its whole lifetime — see
/// `unit_repository_impl.rs`, which acquires per-call via `&self.pool`.
pub const DEFAULT_MAX_CONCURRENT_REQUESTS: usize = 20;

/// Configuration for [`RequestConcurrencyLimit`].
#[derive(Clone, Copy, Debug)]
pub struct ConcurrencyLimitConfig {
    /// Maximum number of requests allowed in flight at once, across all
    /// Actix workers.
    pub max_concurrent: usize,
    /// Value advertised in the `Retry-After` header when shedding a request.
    pub retry_after_secs: u64,
}

impl Default for ConcurrencyLimitConfig {
    fn default() -> Self {
        let max_concurrent = std::env::var("MAX_CONCURRENT_REQUESTS")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .filter(|v| *v > 0)
            .unwrap_or(DEFAULT_MAX_CONCURRENT_REQUESTS);
        Self {
            max_concurrent,
            retry_after_secs: 1,
        }
    }
}

/// Load-shedding middleware: bounds the number of requests in flight and
/// rejects the excess with an explicit `429 Too Many Requests` +
/// `Retry-After`, instead of letting them queue silently on the DB pool
/// until a client timeout or an upstream 502 (see module docs above).
#[derive(Clone)]
pub struct RequestConcurrencyLimit {
    semaphore: Arc<tokio::sync::Semaphore>,
    retry_after_secs: u64,
}

impl RequestConcurrencyLimit {
    pub fn new(config: ConcurrencyLimitConfig) -> Self {
        Self {
            semaphore: Arc::new(tokio::sync::Semaphore::new(config.max_concurrent)),
            retry_after_secs: config.retry_after_secs,
        }
    }
}

impl<S, B> Transform<S, ServiceRequest> for RequestConcurrencyLimit
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<actix_web::body::EitherBody<B>>;
    type Error = Error;
    type InitError = ();
    type Transform = RequestConcurrencyLimitMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RequestConcurrencyLimitMiddleware {
            service: Arc::new(service),
            semaphore: self.semaphore.clone(),
            retry_after_secs: self.retry_after_secs,
        }))
    }
}

pub struct RequestConcurrencyLimitMiddleware<S> {
    service: Arc<S>,
    semaphore: Arc<tokio::sync::Semaphore>,
    retry_after_secs: u64,
}

impl<S, B> Service<ServiceRequest> for RequestConcurrencyLimitMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<actix_web::body::EitherBody<B>>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        // Liveness probe: never shed (see module docs — an unhealthy
        // container restarts, which turns backpressure into an outage).
        if req.path().ends_with("/health") {
            let fut = self.service.call(req);
            return Box::pin(async move { fut.await.map(|res| res.map_into_left_body()) });
        }

        match self.semaphore.clone().try_acquire_owned() {
            Ok(permit) => {
                let fut = self.service.call(req);
                Box::pin(async move {
                    let res = fut.await;
                    drop(permit); // frees the slot as soon as the response is ready
                    res.map(|r| r.map_into_left_body())
                })
            }
            Err(_) => {
                let retry_after = self.retry_after_secs.to_string();
                let response = HttpResponse::build(StatusCode::TOO_MANY_REQUESTS)
                    .insert_header(("Retry-After", retry_after))
                    .json(serde_json::json!({
                        "error": "server_busy",
                        "message": "Trop de requêtes en cours de traitement, réessayez sous peu.",
                        "retry_after_seconds": self.retry_after_secs,
                    }));
                Box::pin(async move { Ok(req.into_response(response).map_into_right_body()) })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_authenticated_user_require_organization() {
        let user_with_org = AuthenticatedUser {
            user_id: Uuid::new_v4(),
            email: "test@example.com".to_string(),
            role: "admin".to_string(),
            role_id: None,
            organization_id: Some(Uuid::new_v4()),
        };

        assert!(user_with_org.require_organization().is_ok());

        let user_without_org = AuthenticatedUser {
            user_id: Uuid::new_v4(),
            email: "test@example.com".to_string(),
            role: "admin".to_string(),
            role_id: None,
            organization_id: None,
        };

        assert!(user_without_org.require_organization().is_err());
    }

    #[test]
    fn test_gdpr_rate_limit_config_default() {
        let config = GdprRateLimitConfig::default();
        assert_eq!(config.max_requests, 10);
        assert_eq!(config.window_duration, Duration::from_secs(3600));
    }

    #[test]
    fn test_gdpr_rate_limit_state_allows_within_limit() {
        let config = GdprRateLimitConfig {
            max_requests: 3,
            window_duration: Duration::from_secs(60),
        };
        let state = GdprRateLimitState::new(config);

        assert!(state.check_rate_limit("user1").is_ok());
        assert!(state.check_rate_limit("user1").is_ok());
        assert!(state.check_rate_limit("user1").is_ok());
    }

    #[test]
    fn test_gdpr_rate_limit_state_blocks_exceeding_limit() {
        let config = GdprRateLimitConfig {
            max_requests: 2,
            window_duration: Duration::from_secs(60),
        };
        let state = GdprRateLimitState::new(config);

        assert!(state.check_rate_limit("user1").is_ok());
        assert!(state.check_rate_limit("user1").is_ok());
        let result = state.check_rate_limit("user1");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("Rate limit exceeded. Try again in"));
    }
}
