// Web Handlers: Financial Reports for Buildings
use super::financial_report_handlers::{parse_date_flexible, IncomeStatementQuery};
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

    let period_start = match parse_date_flexible(&query.period_start) {
        Some(dt) => dt,
        None => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid period_start format. Use 2024-01-01 or 2024-01-01T00:00:00Z"
            }))
        }
    };

    let period_end = match parse_date_flexible(&query.period_end) {
        Some(dt) => dt,
        None => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid period_end format. Use 2024-12-31 or 2024-12-31T23:59:59Z"
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
