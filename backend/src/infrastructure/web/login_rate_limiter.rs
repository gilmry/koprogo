use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpResponse,
};
use futures_util::future::LocalBoxFuture;
use std::collections::HashMap;
use std::future::{ready, Ready};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Login attempt tracking per IP address
#[derive(Debug, Clone)]
struct LoginAttempts {
    count: u32,
    first_attempt: Instant,
    last_attempt: Instant,
}

impl LoginAttempts {
    fn new() -> Self {
        let now = Instant::now();
        Self {
            count: 1,
            first_attempt: now,
            last_attempt: now,
        }
    }

    fn increment(&mut self) {
        self.count += 1;
        self.last_attempt = Instant::now();
    }

    fn is_expired(&self, window: Duration) -> bool {
        self.last_attempt.elapsed() > window
    }

    fn is_rate_limited(&self, max_attempts: u32, window: Duration) -> bool {
        if self.first_attempt.elapsed() > window {
            // Time window expired, reset allowed
            false
        } else {
            // Within time window, check count
            self.count >= max_attempts
        }
    }
}

/// Login rate limiter to prevent brute-force attacks
///
/// Default configuration:
/// - 5 login attempts per 15 minutes per IP
/// - Automatic cleanup of expired entries every 5 minutes
#[derive(Clone)]
pub struct LoginRateLimiter {
    store: Arc<Mutex<HashMap<String, LoginAttempts>>>,
    max_attempts: u32,
    window_duration: Duration,
}

impl Default for LoginRateLimiter {
    fn default() -> Self {
        Self::new(5, Duration::from_secs(15 * 60)) // 5 attempts per 15 minutes
    }
}

impl LoginRateLimiter {
    pub fn new(max_attempts: u32, window_duration: Duration) -> Self {
        let limiter = Self {
            store: Arc::new(Mutex::new(HashMap::new())),
            max_attempts,
            window_duration,
        };

        // Spawn cleanup task (simplified - in production use tokio::spawn with proper cleanup)
        limiter.cleanup_expired_entries();

        limiter
    }

    /// Check if IP is rate limited
    pub fn check_rate_limit(&self, ip: &str) -> bool {
        let mut store = self.store.lock().unwrap();

        match store.get_mut(ip) {
            Some(attempts) => {
                if attempts.is_expired(self.window_duration) {
                    // Expired, reset
                    *attempts = LoginAttempts::new();
                    false
                } else if attempts.is_rate_limited(self.max_attempts, self.window_duration) {
                    // Rate limited
                    true
                } else {
                    // Within limits, increment
                    attempts.increment();
                    false
                }
            }
            None => {
                // First attempt
                store.insert(ip.to_string(), LoginAttempts::new());
                false
            }
        }
    }

    /// Cleanup expired entries (call periodically)
    fn cleanup_expired_entries(&self) {
        let store = self.store.clone();
        let window = self.window_duration;

        // In a real implementation, use tokio::spawn for async cleanup
        // For MVP, rely on per-request cleanup in check_rate_limit
        std::thread::spawn(move || loop {
            std::thread::sleep(Duration::from_secs(300)); // Clean every 5 minutes

            let mut store_lock = store.lock().unwrap();
            store_lock.retain(|_, attempts| !attempts.is_expired(window));

            log::debug!(
                "Login rate limiter cleanup: {} active IPs tracked",
                store_lock.len()
            );
        });
    }

    /// Get current attempt count for IP (for testing/monitoring)
    #[allow(dead_code)]
    pub fn get_attempt_count(&self, ip: &str) -> u32 {
        self.store
            .lock()
            .unwrap()
            .get(ip)
            .map(|a| a.count)
            .unwrap_or(0)
    }
}

impl<S, B> Transform<S, ServiceRequest> for LoginRateLimiter
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: actix_web::body::MessageBody + 'static,
{
    type Response = ServiceResponse<actix_web::body::EitherBody<B>>;
    type Error = Error;
    type InitError = ();
    type Transform = LoginRateLimiterMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(LoginRateLimiterMiddleware {
            service,
            limiter: self.clone(),
        }))
    }
}

pub struct LoginRateLimiterMiddleware<S> {
    service: S,
    limiter: LoginRateLimiter,
}

impl<S, B> Service<ServiceRequest> for LoginRateLimiterMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: actix_web::body::MessageBody + 'static,
{
    type Response = ServiceResponse<actix_web::body::EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        // Only apply rate limiting to login endpoint
        let path = req.path();
        let is_login_endpoint = path == "/api/v1/auth/login" || path.ends_with("/login");

        if !is_login_endpoint {
            // Not a login endpoint, skip rate limiting
            let fut = self.service.call(req);
            return Box::pin(async move { fut.await.map(|res| res.map_into_left_body()) });
        }

        // Extract IP address
        let ip = req
            .connection_info()
            .realip_remote_addr()
            .unwrap_or("unknown")
            .to_string();

        // Check rate limit
        let is_limited = self.limiter.check_rate_limit(&ip);

        if is_limited {
            log::warn!("Login rate limit exceeded for IP: {}", ip);

            // Return 429 Too Many Requests
            let response = HttpResponse::TooManyRequests()
                .insert_header(("Retry-After", "900"))
                .json(serde_json::json!({
                    "error": "Too many login attempts. Please try again in 15 minutes.",
                    "retry_after": 900 // 15 minutes in seconds
                }));

            return Box::pin(async move { Ok(req.into_response(response).map_into_right_body()) });
        }

        // Not rate limited, proceed
        let fut = self.service.call(req);
        Box::pin(async move { fut.await.map(|res| res.map_into_left_body()) })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_login_rate_limiter() {
        let limiter = LoginRateLimiter::new(5, Duration::from_secs(60));
        let ip = "192.168.1.1";

        // First 5 attempts should be allowed
        for i in 1..=5 {
            assert!(
                !limiter.check_rate_limit(ip),
                "Attempt {} should be allowed",
                i
            );
        }

        // 6th attempt should be rate limited
        assert!(
            limiter.check_rate_limit(ip),
            "Attempt 6 should be rate limited"
        );

        // Verify count
        assert_eq!(limiter.get_attempt_count(ip), 5);
    }

    #[test]
    fn test_rate_limiter_expiration() {
        let limiter = LoginRateLimiter::new(2, Duration::from_millis(100));
        let ip = "192.168.1.2";

        // Use up the limit
        limiter.check_rate_limit(ip);
        limiter.check_rate_limit(ip);
        assert!(limiter.check_rate_limit(ip), "Should be rate limited");

        // Wait for expiration
        std::thread::sleep(Duration::from_millis(150));

        // Should be allowed again
        assert!(
            !limiter.check_rate_limit(ip),
            "Should be allowed after expiration"
        );
    }

    #[test]
    fn test_different_ips_independent() {
        let limiter = LoginRateLimiter::new(2, Duration::from_secs(60));

        let ip1 = "192.168.1.1";
        let ip2 = "192.168.1.2";

        // IP1 uses limit
        limiter.check_rate_limit(ip1);
        limiter.check_rate_limit(ip1);

        // IP2 should still have its own limit
        assert!(!limiter.check_rate_limit(ip2), "IP2 should be independent");
    }
}
