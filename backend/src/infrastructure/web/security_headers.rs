use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error,
};
use futures_util::future::LocalBoxFuture;
use std::future::{ready, Ready};

/// Security headers middleware for production-ready security
///
/// Adds the following security headers to all responses:
/// - Strict-Transport-Security (HSTS): Force HTTPS
/// - X-Content-Type-Options: Prevent MIME sniffing
/// - X-Frame-Options: Prevent clickjacking
/// - X-XSS-Protection: Enable browser XSS filter
/// - Content-Security-Policy: Restrict resource loading
/// - Referrer-Policy: Control referrer information
/// - Permissions-Policy: Control browser features
pub struct SecurityHeaders;

impl<S, B> Transform<S, ServiceRequest> for SecurityHeaders
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = SecurityHeadersMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(SecurityHeadersMiddleware { service }))
    }
}

pub struct SecurityHeadersMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for SecurityHeadersMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let fut = self.service.call(req);

        Box::pin(async move {
            let mut res = fut.await?;

            let headers = res.headers_mut();

            // HSTS: Force HTTPS for 1 year (31536000 seconds)
            // includeSubDomains: Apply to all subdomains
            // preload: Submit to HSTS preload list
            headers.insert(
                actix_web::http::header::HeaderName::from_static("strict-transport-security"),
                actix_web::http::header::HeaderValue::from_static(
                    "max-age=31536000; includeSubDomains; preload"
                ),
            );

            // Prevent MIME type sniffing
            headers.insert(
                actix_web::http::header::HeaderName::from_static("x-content-type-options"),
                actix_web::http::header::HeaderValue::from_static("nosniff"),
            );

            // Prevent clickjacking attacks
            headers.insert(
                actix_web::http::header::HeaderName::from_static("x-frame-options"),
                actix_web::http::header::HeaderValue::from_static("DENY"),
            );

            // Enable browser XSS protection (legacy, but still useful)
            headers.insert(
                actix_web::http::header::HeaderName::from_static("x-xss-protection"),
                actix_web::http::header::HeaderValue::from_static("1; mode=block"),
            );

            // Content Security Policy (CSP)
            // - default-src 'self': Only load resources from same origin
            // - script-src 'self' 'unsafe-inline': Allow inline scripts (needed for frontend frameworks)
            // - style-src 'self' 'unsafe-inline': Allow inline styles
            // - img-src 'self' data: https:: Allow images from same origin, data URLs, and HTTPS
            // - font-src 'self': Only load fonts from same origin
            // - connect-src 'self': Only connect to same origin APIs
            // - frame-ancestors 'none': Prevent framing (same as X-Frame-Options)
            // - base-uri 'self': Prevent base tag hijacking
            // - form-action 'self': Only submit forms to same origin
            headers.insert(
                actix_web::http::header::HeaderName::from_static("content-security-policy"),
                actix_web::http::header::HeaderValue::from_static(
                    "default-src 'self'; \
                     script-src 'self' 'unsafe-inline' 'unsafe-eval'; \
                     style-src 'self' 'unsafe-inline'; \
                     img-src 'self' data: https:; \
                     font-src 'self' data:; \
                     connect-src 'self'; \
                     frame-ancestors 'none'; \
                     base-uri 'self'; \
                     form-action 'self'"
                ),
            );

            // Referrer Policy: Don't send referrer to cross-origin requests
            headers.insert(
                actix_web::http::header::HeaderName::from_static("referrer-policy"),
                actix_web::http::header::HeaderValue::from_static("strict-origin-when-cross-origin"),
            );

            // Permissions Policy (formerly Feature-Policy)
            // Disable potentially dangerous browser features
            headers.insert(
                actix_web::http::header::HeaderName::from_static("permissions-policy"),
                actix_web::http::header::HeaderValue::from_static(
                    "geolocation=(), microphone=(), camera=(), payment=(), usb=()"
                ),
            );

            Ok(res)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, web, App, HttpResponse};

    #[actix_web::test]
    async fn test_security_headers_are_added() {
        let app = test::init_service(
            App::new()
                .wrap(SecurityHeaders)
                .route("/test", web::get().to(|| async { HttpResponse::Ok().finish() })),
        )
        .await;

        let req = test::TestRequest::get().uri("/test").to_request();
        let resp = test::call_service(&app, req).await;

        // Verify HSTS header
        assert!(resp.headers().contains_key("strict-transport-security"));
        assert_eq!(
            resp.headers().get("strict-transport-security").unwrap(),
            "max-age=31536000; includeSubDomains; preload"
        );

        // Verify X-Content-Type-Options
        assert!(resp.headers().contains_key("x-content-type-options"));
        assert_eq!(
            resp.headers().get("x-content-type-options").unwrap(),
            "nosniff"
        );

        // Verify X-Frame-Options
        assert!(resp.headers().contains_key("x-frame-options"));
        assert_eq!(resp.headers().get("x-frame-options").unwrap(), "DENY");

        // Verify CSP
        assert!(resp.headers().contains_key("content-security-policy"));

        // Verify Referrer-Policy
        assert!(resp.headers().contains_key("referrer-policy"));

        // Verify Permissions-Policy
        assert!(resp.headers().contains_key("permissions-policy"));
    }
}
