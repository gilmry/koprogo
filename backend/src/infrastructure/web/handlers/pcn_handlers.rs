use crate::application::dto::PcnReportRequest;
use crate::infrastructure::web::AppState;
use actix_web::{post, web, HttpResponse, Responder};
use uuid::Uuid;

/// Generate PCN report for a building
/// POST /api/v1/pcn/report/:building_id
#[post("/pcn/report/{building_id}")]
pub async fn generate_pcn_report(
    app_state: web::Data<AppState>,
    building_id: web::Path<Uuid>,
) -> impl Responder {
    let request = PcnReportRequest {
        building_id: *building_id,
        start_date: None,
        end_date: None,
    };

    match app_state
        .pcn_use_cases
        .generate_report(request)
        .await
    {
        Ok(report) => HttpResponse::Ok().json(report),
        Err(err) => HttpResponse::InternalServerError().body(err),
    }
}
