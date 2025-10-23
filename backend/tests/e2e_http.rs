use actix_cors::Cors;
use actix_governor::{Governor, GovernorConfigBuilder};
use actix_web::{test, App, http::header, web};
use koprogo_api::infrastructure::web::configure_routes;

#[actix_web::test]
async fn rate_limit_returns_429_on_burst() {
    // Configure aggressive rate limit for test: burst 1
    let governor_conf = GovernorConfigBuilder::default()
        .milliseconds_per_request(10_000) // slow refill
        .burst_size(1)
        .finish()
        .unwrap();

    let app = test::init_service(
        App::new()
            .wrap(Governor::new(&governor_conf))
            .configure(configure_routes),
    )
    .await;

    // First request should pass
    let req1 = test::TestRequest::get().uri("/api/v1/health").to_request();
    let resp1 = test::call_service(&app, req1).await;
    assert!(resp1.status().is_success());

    // Second immediate request should be rate-limited
    let req2 = test::TestRequest::get().uri("/api/v1/health").to_request();
    let resp2 = test::call_service(&app, req2).await;
    assert_eq!(resp2.status(), 429);
}

#[actix_web::test]
async fn cors_allows_configured_origin() {
    let allowed = "http://allowed.test";
    let cors = Cors::default()
        .allowed_origin(allowed)
        .allowed_methods(vec!["GET"]).allowed_header(header::CONTENT_TYPE);

    let app = test::init_service(
        App::new()
            .wrap(cors)
            .configure(configure_routes),
    )
    .await;

    // Simulate preflight OPTIONS
    let req = test::TestRequest::default()
        .method(actix_web::http::Method::OPTIONS)
        .uri("/api/v1/health")
        .insert_header((header::ORIGIN, allowed))
        .insert_header((header::ACCESS_CONTROL_REQUEST_METHOD, "GET"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    // Should be 200 or 204 with CORS headers present
    assert!(resp.status().is_success());
    let hdr = resp.headers().get(header::ACCESS_CONTROL_ALLOW_ORIGIN).cloned();
    assert_eq!(hdr.unwrap().to_str().unwrap(), allowed);
}

#[actix_web::test]
async fn cors_blocks_disallowed_origin() {
    let cors = Cors::default()
        .allowed_origin("http://allowed.test")
        .allowed_methods(vec!["GET"]).allowed_header(header::CONTENT_TYPE);

    let app = test::init_service(
        App::new()
            .wrap(cors)
            .configure(configure_routes),
    )
    .await;

    // Preflight from disallowed origin
    let req = test::TestRequest::default()
        .method(actix_web::http::Method::OPTIONS)
        .uri("/api/v1/health")
        .insert_header((header::ORIGIN, "http://evil.test"))
        .insert_header((header::ACCESS_CONTROL_REQUEST_METHOD, "GET"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    // Actix CORS responds with 200 but without ACAO header; assert header missing
    assert!(resp.status().is_success());
    assert!(resp.headers().get(header::ACCESS_CONTROL_ALLOW_ORIGIN).is_none());
}

