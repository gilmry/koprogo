use actix_web::{get, http::header::AUTHORIZATION, HttpRequest, HttpResponse, Responder};
use once_cell::sync::Lazy;
use prometheus::{Encoder, TextEncoder};

static METRICS_TOKEN: Lazy<Option<String>> = Lazy::new(|| std::env::var("METRICS_AUTH_TOKEN").ok());

#[get("/metrics")]
pub async fn metrics_endpoint(req: HttpRequest) -> impl Responder {
    if let Some(expected) = METRICS_TOKEN.as_ref() {
        let expected_header = format!("Bearer {}", expected);
        let authorized = req
            .headers()
            .get(AUTHORIZATION)
            .and_then(|h| h.to_str().ok())
            .map(|header| header == expected_header)
            .unwrap_or(false);

        if !authorized {
            return HttpResponse::Unauthorized().body("Unauthorized");
        }
    }

    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();

    let mut buffer = Vec::new();
    if let Err(err) = encoder.encode(&metric_families, &mut buffer) {
        return HttpResponse::InternalServerError()
            .body(format!("failed to encode metrics: {}", err));
    }

    HttpResponse::Ok()
        .content_type(encoder.format_type())
        .body(buffer)
}
