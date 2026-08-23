use actix_cors::Cors;
use actix_web::{http::header, test, App};
use koprogo_api::infrastructure::web::configure_routes;

#[actix_web::test]
async fn cors_allows_configured_origin() {
    let allowed = "http://allowed.test";
    let cors = Cors::default()
        .allowed_origin(allowed)
        .allowed_methods(vec!["GET"])
        .allowed_header(header::CONTENT_TYPE);

    let app = test::init_service(App::new().wrap(cors).configure(configure_routes)).await;

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
    let hdr = resp
        .headers()
        .get(header::ACCESS_CONTROL_ALLOW_ORIGIN)
        .cloned();
    assert_eq!(hdr.unwrap().to_str().unwrap(), allowed);
}

#[actix_web::test]
async fn cors_blocks_disallowed_origin() {
    let cors = Cors::default()
        .allowed_origin("http://allowed.test")
        .allowed_methods(vec!["GET"])
        .allowed_header(header::CONTENT_TYPE);

    let app = test::init_service(App::new().wrap(cors).configure(configure_routes)).await;

    // Preflight from disallowed origin
    let req = test::TestRequest::default()
        .method(actix_web::http::Method::OPTIONS)
        .uri("/api/v1/health")
        .insert_header((header::ORIGIN, "http://evil.test"))
        .insert_header((header::ACCESS_CONTROL_REQUEST_METHOD, "GET"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    // Actix CORS should not allow this origin; assert ACAO header is missing
    if resp.status().is_success() {
        assert!(resp
            .headers()
            .get(header::ACCESS_CONTROL_ALLOW_ORIGIN)
            .is_none());
    } else {
        // Non-success is also acceptable for disallowed origin
        assert!(!resp.status().is_success());
    }
}
