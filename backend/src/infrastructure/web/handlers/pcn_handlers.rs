use crate::application::dto::PcnReportRequest;
use crate::infrastructure::web::{AppState, AuthenticatedUser};
use actix_web::{get, post, web, HttpResponse, Responder, ResponseError};
use uuid::Uuid;

/// Generate PCN report for a building (JSON)
/// POST /api/v1/pcn/report/:building_id
#[post("/pcn/report/{building_id}")]
pub async fn generate_pcn_report(
    app_state: web::Data<AppState>,
    user: AuthenticatedUser,
    building_id: web::Path<Uuid>,
) -> impl Responder {
    // Aucune identité n'était exigée ici : ni `AuthenticatedUser`, ni jeton
    // lu à la main. N'importe qui pouvait exporter la comptabilité complète
    // de n'importe quel immeuble.
    //
    // Le cliquet d'identité de #772 ne pouvait pas le voir : il compte les
    // routes qui PRENNENT `AuthenticatedUser` sans s'en servir. Une route qui
    // ne le prend pas du tout lui échappait entièrement. Cf. #845.
    if let Err(err) =
        crate::infrastructure::web::middleware::scope_guard::verify_building_org_access(
            &user,
            *building_id,
            &app_state.building_use_cases,
            &app_state.acp_use_cases,
        )
        .await
    {
        return err.error_response();
    }

    let request = PcnReportRequest {
        building_id: *building_id,
        start_date: None,
        end_date: None,
    };

    match app_state.pcn_use_cases.generate_report(request).await {
        Ok(report) => HttpResponse::Ok().json(report),
        Err(err) => HttpResponse::InternalServerError().body(err),
    }
}

/// Export PCN report as PDF
/// GET /api/v1/pcn/export/pdf/:building_id?name=MyBuilding
#[get("/pcn/export/pdf/{building_id}")]
pub async fn export_pcn_pdf(
    app_state: web::Data<AppState>,
    user: AuthenticatedUser,
    building_id: web::Path<Uuid>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> impl Responder {
    // Aucune identité n'était exigée ici : ni `AuthenticatedUser`, ni jeton
    // lu à la main. N'importe qui pouvait exporter la comptabilité complète
    // de n'importe quel immeuble.
    //
    // Le cliquet d'identité de #772 ne pouvait pas le voir : il compte les
    // routes qui PRENNENT `AuthenticatedUser` sans s'en servir. Une route qui
    // ne le prend pas du tout lui échappait entièrement. Cf. #845.
    if let Err(err) =
        crate::infrastructure::web::middleware::scope_guard::verify_building_org_access(
            &user,
            *building_id,
            &app_state.building_use_cases,
            &app_state.acp_use_cases,
        )
        .await
    {
        return err.error_response();
    }

    let building_name = query
        .get("name")
        .cloned()
        .unwrap_or_else(|| "Building".to_string());

    let request = PcnReportRequest {
        building_id: *building_id,
        start_date: None,
        end_date: None,
    };

    match app_state
        .pcn_use_cases
        .export_pdf(&building_name, request)
        .await
    {
        Ok(pdf_bytes) => HttpResponse::Ok()
            .content_type("application/pdf")
            .append_header((
                "Content-Disposition",
                "attachment; filename=\"rapport-pcn.pdf\"",
            ))
            .body(pdf_bytes),
        Err(err) => HttpResponse::InternalServerError().body(err),
    }
}

/// Export PCN report as Excel
/// GET /api/v1/pcn/export/excel/:building_id?name=MyBuilding
#[get("/pcn/export/excel/{building_id}")]
pub async fn export_pcn_excel(
    app_state: web::Data<AppState>,
    user: AuthenticatedUser,
    building_id: web::Path<Uuid>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> impl Responder {
    // Aucune identité n'était exigée ici : ni `AuthenticatedUser`, ni jeton
    // lu à la main. N'importe qui pouvait exporter la comptabilité complète
    // de n'importe quel immeuble.
    //
    // Le cliquet d'identité de #772 ne pouvait pas le voir : il compte les
    // routes qui PRENNENT `AuthenticatedUser` sans s'en servir. Une route qui
    // ne le prend pas du tout lui échappait entièrement. Cf. #845.
    if let Err(err) =
        crate::infrastructure::web::middleware::scope_guard::verify_building_org_access(
            &user,
            *building_id,
            &app_state.building_use_cases,
            &app_state.acp_use_cases,
        )
        .await
    {
        return err.error_response();
    }

    let building_name = query
        .get("name")
        .cloned()
        .unwrap_or_else(|| "Building".to_string());

    let request = PcnReportRequest {
        building_id: *building_id,
        start_date: None,
        end_date: None,
    };

    match app_state
        .pcn_use_cases
        .export_excel(&building_name, request)
        .await
    {
        Ok(excel_bytes) => HttpResponse::Ok()
            .content_type("application/vnd.openxmlformats-officedocument.spreadsheetml.sheet")
            .append_header((
                "Content-Disposition",
                "attachment; filename=\"rapport-pcn.xlsx\"",
            ))
            .body(excel_bytes),
        Err(err) => HttpResponse::InternalServerError().body(err),
    }
}
