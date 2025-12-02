use crate::infrastructure::web::app_state::AppState;
// Note: Rate limiting is configured in main.rs using actix_governor
// The actix_governor imports are kept in main.rs, not here
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
// Rate limiting is configured directly in main.rs using GovernorConfigBuilder.
// Three-tier strategy:
// 1. Public endpoints: 100 req/min per IP (DDoS prevention)
// 2. Authenticated endpoints: 1000 req/min per IP (higher trust, still IP-based for simplicity)
// 3. Login endpoint: 5 attempts per 15min per IP (brute-force prevention via LoginRateLimiter)

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
