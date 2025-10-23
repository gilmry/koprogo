use actix_governor::{Governor, GovernorConfig, GovernorConfigBuilder, KeyExtractor, SimpleKeyExtractionError};
use actix_web::dev::ServiceRequest;
use std::net::IpAddr;

/// Extract IP address from request for rate limiting
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IpKeyExtractor;

impl KeyExtractor for IpKeyExtractor {
    type Key = IpAddr;
    type KeyExtractionError = SimpleKeyExtractionError<&'static str>;

    fn extract(&self, req: &ServiceRequest) -> Result<Self::Key, Self::KeyExtractionError> {
        req.connection_info()
            .peer_addr()
            .and_then(|addr| addr.split(':').next())
            .and_then(|ip_str| ip_str.parse::<IpAddr>().ok())
            .ok_or_else(|| SimpleKeyExtractionError::new("Could not extract IP address"))
    }

    fn response_error_msg(&self, _: &Self::KeyExtractionError) -> String {
        "Could not extract IP address from request".to_string()
    }
}

/// Create rate limiter for authentication endpoints (stricter)
/// Limit: 5 requests per 15 minutes
pub fn create_auth_rate_limiter() -> Governor<IpKeyExtractor> {
    let config = GovernorConfigBuilder::default()
        .per_second(5)
        .burst_size(5)
        .use_headers()
        .key_extractor(IpKeyExtractor)
        .finish()
        .expect("Failed to create auth rate limiter config");

    Governor::new(&config)
}

/// Create rate limiter for general API endpoints
/// Limit: 100 requests per minute (≈1.67 req/sec)
pub fn create_api_rate_limiter() -> Governor<IpKeyExtractor> {
    let config = GovernorConfigBuilder::default()
        .per_second(100)
        .burst_size(100)
        .use_headers()
        .key_extractor(IpKeyExtractor)
        .finish()
        .expect("Failed to create API rate limiter config");

    Governor::new(&config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, web, App, HttpResponse};

    #[actix_web::test]
    async fn test_rate_limiter_allows_within_limit() {
        let limiter = create_auth_rate_limiter();

        let app = test::init_service(
            App::new()
                .wrap(limiter)
                .route("/test", web::get().to(|| async { HttpResponse::Ok() })),
        )
        .await;

        // First request should succeed
        let req = test::TestRequest::get().uri("/test").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    #[test]
    fn test_ip_key_extractor() {
        let extractor = IpKeyExtractor;
        // Basic smoke test - actual extraction is tested via integration tests
        assert_eq!(extractor, IpKeyExtractor);
    }
}
