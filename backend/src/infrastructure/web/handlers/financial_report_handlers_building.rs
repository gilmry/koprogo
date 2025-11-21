// Web Handlers: Financial Reports for Buildings
use super::financial_report_handlers::IncomeStatementQuery;
use crate::infrastructure::audit::{AuditEventType, AuditLogEntry};
use crate::infrastructure::web::{AppState, AuthenticatedUser};
use actix_web::{get, web, HttpResponse, Responder};

/// Generate balance sheet report for a specific building
#[get("/buildings/{building_id}/reports/balance-sheet")]
pub async fn generate_balance_sheet_for_building(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    building_id: web::Path<uuid::Uuid>,
) -> impl Responder {
    if !matches!(user.role.as_str(), "accountant" | "superadmin" | "syndic") {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Only accountants, syndics and superadmins can generate financial reports"
        }));
    }

    let organization_id = match user.require_organization() {
        Ok(org_id) => org_id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": e.to_string()
            }))
        }
    };

    match state
        .financial_report_use_cases
        .generate_balance_sheet_for_building(organization_id, *building_id)
        .await
    {
        Ok(report) => {
            AuditLogEntry::new(
                AuditEventType::ReportGenerated,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_metadata(serde_json::json!({
                "report_type": "balance_sheet",
                "building_id": building_id.to_string()
            }))
            .log();

            HttpResponse::Ok().json(report)
        }
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": err
        })),
    }
}

/// Generate income statement for a specific building
#[get("/buildings/{building_id}/reports/income-statement")]
pub async fn generate_income_statement_for_building(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    building_id: web::Path<uuid::Uuid>,
    query: web::Query<IncomeStatementQuery>,
) -> impl Responder {
    if !matches!(user.role.as_str(), "accountant" | "superadmin" | "syndic") {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Only accountants, syndics and superadmins can generate financial reports"
        }));
    }

    let organization_id = match user.require_organization() {
        Ok(org_id) => org_id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": e.to_string()
            }))
        }
    };

    let period_start = match chrono::DateTime::parse_from_rfc3339(&query.period_start) {
        Ok(dt) => dt.with_timezone(&chrono::Utc),
        Err(_) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid period_start format"
            }))
        }
    };

    let period_end = match chrono::DateTime::parse_from_rfc3339(&query.period_end) {
        Ok(dt) => dt.with_timezone(&chrono::Utc),
        Err(_) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid period_end format"
            }))
        }
    };

    if period_start >= period_end {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "period_start must be before period_end"
        }));
    }

    match state
        .financial_report_use_cases
        .generate_income_statement_for_building(
            organization_id,
            *building_id,
            period_start,
            period_end,
        )
        .await
    {
        Ok(report) => {
            AuditLogEntry::new(
                AuditEventType::ReportGenerated,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_metadata(serde_json::json!({
                "report_type": "income_statement",
                "building_id": building_id.to_string()
            }))
            .log();

            HttpResponse::Ok().json(report)
        }
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": err
        })),
    }
}
