// Web Handlers: Financial Reports
//
// CREDITS & ATTRIBUTION:
// This implementation is inspired by the Noalyss project (https://gitlab.com/noalyss/noalyss)
// Noalyss is a free accounting software for Belgian and French accounting
// License: GPL-2.0-or-later (GNU General Public License version 2 or later)
// Copyright: (C) 1989, 1991 Free Software Foundation, Inc.
// Copyright: Dany De Bontridder <dany@alchimerys.eu>
//
// API endpoints for generating Belgian PCMN financial reports

use crate::infrastructure::audit::{AuditEventType, AuditLogEntry};
use crate::infrastructure::web::{AppState, AuthenticatedUser};
use actix_web::{get, web, HttpResponse, Responder};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct IncomeStatementQuery {
    /// Period start date (ISO 8601 format, e.g., "2024-01-01T00:00:00Z")
    pub period_start: String,
    /// Period end date (ISO 8601 format, e.g., "2024-12-31T23:59:59Z")
    pub period_end: String,
}

/// Generate balance sheet report for an organization
///
/// **Access:** Accountant, SuperAdmin
///
/// **Belgian PCMN Balance Sheet:**
/// - Assets (Classes 2-5): Buildings, receivables, bank, cash
/// - Liabilities (Class 1): Capital, reserves, provisions, payables
///
/// **Example:**
/// ```
/// GET /api/v1/reports/balance-sheet
/// Authorization: Bearer <token>
/// ```
#[get("/reports/balance-sheet")]
pub async fn generate_balance_sheet(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
) -> impl Responder {
    // Only Accountant and SuperAdmin can generate financial reports
    if !matches!(user.role.as_str(), "accountant" | "superadmin") {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Only accountants and superadmins can generate financial reports"
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
        .generate_balance_sheet(organization_id)
        .await
    {
        Ok(report) => {
            // Audit log: balance sheet generated
            AuditLogEntry::new(
                AuditEventType::ReportGenerated,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_metadata(serde_json::json!({
                "report_type": "balance_sheet"
            }))
            .log();

            HttpResponse::Ok().json(report)
        }
        Err(err) => {
            // Audit log: failed to generate balance sheet
            AuditLogEntry::new(
                AuditEventType::ReportGenerated,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_metadata(serde_json::json!({
                "report_type": "balance_sheet"
            }))
            .with_error(err.clone())
            .log();

            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": err
            }))
        }
    }
}

/// Generate income statement (profit & loss) report for a time period
///
/// **Access:** Accountant, SuperAdmin
///
/// **Belgian PCMN Income Statement:**
/// - Expenses (Class 6): Operating costs, maintenance, utilities
/// - Revenue (Class 7): Regular fees, extraordinary fees, interest income
///
/// **Query Parameters:**
/// - `period_start`: ISO 8601 datetime (e.g., "2024-01-01T00:00:00Z")
/// - `period_end`: ISO 8601 datetime (e.g., "2024-12-31T23:59:59Z")
///
/// **Example:**
/// ```
/// GET /api/v1/reports/income-statement?period_start=2024-01-01T00:00:00Z&period_end=2024-12-31T23:59:59Z
/// Authorization: Bearer <token>
/// ```
#[get("/reports/income-statement")]
pub async fn generate_income_statement(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    query: web::Query<IncomeStatementQuery>,
) -> impl Responder {
    // Only Accountant and SuperAdmin can generate financial reports
    if !matches!(user.role.as_str(), "accountant" | "superadmin") {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Only accountants and superadmins can generate financial reports"
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

    // Parse dates
    let period_start =
        match chrono::DateTime::parse_from_rfc3339(&query.period_start) {
            Ok(dt) => dt.with_timezone(&chrono::Utc),
            Err(_) => return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid period_start format. Use ISO 8601 (e.g., 2024-01-01T00:00:00Z)"
            })),
        };

    let period_end = match chrono::DateTime::parse_from_rfc3339(&query.period_end) {
        Ok(dt) => dt.with_timezone(&chrono::Utc),
        Err(_) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid period_end format. Use ISO 8601 (e.g., 2024-12-31T23:59:59Z)"
            }))
        }
    };

    // Validate date range
    if period_start >= period_end {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "period_start must be before period_end"
        }));
    }

    match state
        .financial_report_use_cases
        .generate_income_statement(organization_id, period_start, period_end)
        .await
    {
        Ok(report) => {
            // Audit log: income statement generated
            AuditLogEntry::new(
                AuditEventType::ReportGenerated,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_metadata(serde_json::json!({
                "report_type": "income_statement",
                "period_start": &query.period_start,
                "period_end": &query.period_end
            }))
            .log();

            HttpResponse::Ok().json(report)
        }
        Err(err) => {
            // Audit log: failed to generate income statement
            AuditLogEntry::new(
                AuditEventType::ReportGenerated,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_metadata(serde_json::json!({
                "report_type": "income_statement"
            }))
            .with_error(err.clone())
            .log();

            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": err
            }))
        }
    }
}
