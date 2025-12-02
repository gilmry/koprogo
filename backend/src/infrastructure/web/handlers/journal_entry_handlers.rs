// Web Handlers: Journal Entry (Manual Accounting Operations)
//
// CREDITS & ATTRIBUTION:
// This implementation is inspired by the Noalyss project (https://gitlab.com/noalyss/noalyss)
// Noalyss is a free accounting software for Belgian and French accounting
// License: GPL-2.0-or-later (GNU General Public License version 2 or later)
// Copyright: (C) 1989, 1991 Free Software Foundation, Inc.
// Copyright: Dany De Bontridder <dany@alchimerys.eu>
//
// API endpoints for manual journal entry creation and management

use crate::infrastructure::audit::{AuditEventType, AuditLogEntry};
use crate::infrastructure::web::{AppState, AuthenticatedUser};
use actix_web::{delete, get, post, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct CreateJournalEntryRequest {
    pub building_id: Option<Uuid>,
    pub journal_type: String,
    pub entry_date: String, // ISO 8601
    pub description: String,
    pub document_ref: Option<String>,
    pub lines: Vec<JournalEntryLineRequest>,
}

#[derive(Debug, Deserialize)]
pub struct JournalEntryLineRequest {
    pub account_code: String,
    pub debit: f64,
    pub credit: f64,
    pub description: String,
}

#[derive(Debug, Serialize)]
pub struct JournalEntryResponse {
    pub id: String,
    pub organization_id: String,
    pub building_id: Option<String>,
    pub journal_type: Option<String>,
    pub entry_date: String,
    pub description: Option<String>,
    pub document_ref: Option<String>,
    pub expense_id: Option<String>,
    pub contribution_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize)]
pub struct JournalEntryLineResponse {
    pub id: String,
    pub journal_entry_id: String,
    pub account_code: String,
    pub debit: f64,
    pub credit: f64,
    pub description: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct JournalEntryWithLinesResponse {
    pub entry: JournalEntryResponse,
    pub lines: Vec<JournalEntryLineResponse>,
}

#[derive(Debug, Deserialize)]
pub struct ListJournalEntriesQuery {
    pub building_id: Option<Uuid>,
    pub journal_type: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

/// Create a manual journal entry (double-entry bookkeeping)
///
/// **Access:** Accountant, SuperAdmin
///
/// **Noalyss-Inspired Features:**
/// - Journal types: ACH (Purchases), VEN (Sales), FIN (Financial), ODS (Miscellaneous)
/// - Double-entry validation (debits = credits)
/// - Multi-line entries with account codes
///
/// **Example:**
/// ```json
/// POST /api/v1/journal-entries
/// {
///   "building_id": "uuid",
///   "journal_type": "ACH",
///   "entry_date": "2025-01-01T00:00:00Z",
///   "description": "Achat fournitures",
///   "reference": "FA-2025-001",
///   "lines": [
///     {"account_code": "604", "debit": 100.0, "credit": 0.0, "description": "Fournitures"},
///     {"account_code": "440", "debit": 0.0, "credit": 100.0, "description": "Fournisseur X"}
///   ]
/// }
/// ```
#[post("/journal-entries")]
pub async fn create_journal_entry(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    req: web::Json<CreateJournalEntryRequest>,
) -> impl Responder {
    // Only Accountant and SuperAdmin can create journal entries
    if !matches!(user.role.as_str(), "accountant" | "superadmin") {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Only accountants and superadmins can create journal entries"
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

    // Parse entry_date
    let entry_date = match chrono::DateTime::parse_from_rfc3339(&req.entry_date) {
        Ok(dt) => dt.with_timezone(&chrono::Utc),
        Err(_) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid entry_date format. Use ISO 8601 (e.g., 2025-01-01T00:00:00Z)"
            }))
        }
    };

    // Convert lines to tuple format
    let lines: Vec<(String, f64, f64, String)> = req
        .lines
        .iter()
        .map(|l| {
            (
                l.account_code.clone(),
                l.debit,
                l.credit,
                l.description.clone(),
            )
        })
        .collect();

    match state
        .journal_entry_use_cases
        .create_manual_entry(
            organization_id,
            req.building_id,
            Some(req.journal_type.clone()),
            entry_date,
            Some(req.description.clone()),
            req.document_ref.clone(),
            lines,
        )
        .await
    {
        Ok(entry) => {
            // Audit log
            AuditLogEntry::new(
                AuditEventType::JournalEntryCreated,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_metadata(serde_json::json!({
                "entity_type": "journal_entry",
                "entry_id": entry.id.to_string(),
                "journal_type": &req.journal_type
            }))
            .log();

            let response = JournalEntryResponse {
                id: entry.id.to_string(),
                organization_id: entry.organization_id.to_string(),
                building_id: entry.building_id.map(|id| id.to_string()),
                journal_type: entry.journal_type,
                entry_date: entry.entry_date.to_rfc3339(),
                description: entry.description,
                document_ref: entry.document_ref,
                expense_id: entry.expense_id.map(|id| id.to_string()),
                contribution_id: entry.contribution_id.map(|id| id.to_string()),
                created_at: entry.created_at.to_rfc3339(),
                updated_at: entry.updated_at.to_rfc3339(),
            };

            HttpResponse::Created().json(response)
        }
        Err(err) => {
            // Audit log failure
            AuditLogEntry::new(
                AuditEventType::JournalEntryCreated,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_metadata(serde_json::json!({
                "entity_type": "journal_entry",
                "journal_type": &req.journal_type
            }))
            .with_error(err.clone())
            .log();

            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": err
            }))
        }
    }
}

/// List journal entries with filters
///
/// **Access:** Accountant, SuperAdmin, Syndic
///
/// **Query Parameters:**
/// - `building_id`: Filter by building (optional)
/// - `journal_type`: Filter by journal type (ACH, VEN, FIN, ODS) (optional)
/// - `start_date`: Filter by start date (ISO 8601) (optional)
/// - `end_date`: Filter by end date (ISO 8601) (optional)
/// - `page`: Page number (default: 1)
/// - `per_page`: Items per page (default: 20, max: 100)
///
/// **Example:**
/// ```
/// GET /api/v1/journal-entries?journal_type=ACH&page=1&per_page=20
/// ```
#[get("/journal-entries")]
pub async fn list_journal_entries(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    query: web::Query<ListJournalEntriesQuery>,
) -> impl Responder {
    // Only Accountant, SuperAdmin, and Syndic can view journal entries
    if !matches!(user.role.as_str(), "accountant" | "superadmin" | "syndic") {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Only accountants, syndics, and superadmins can view journal entries"
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
    let start_date = query.start_date.as_ref().and_then(|s| {
        chrono::DateTime::parse_from_rfc3339(s)
            .ok()
            .map(|dt| dt.with_timezone(&chrono::Utc))
    });

    let end_date = query.end_date.as_ref().and_then(|s| {
        chrono::DateTime::parse_from_rfc3339(s)
            .ok()
            .map(|dt| dt.with_timezone(&chrono::Utc))
    });

    // Pagination
    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(20).clamp(1, 100);
    let offset = (page - 1) * per_page;

    match state
        .journal_entry_use_cases
        .list_entries(
            organization_id,
            query.building_id,
            query.journal_type.clone(),
            start_date,
            end_date,
            per_page,
            offset,
        )
        .await
    {
        Ok(entries) => {
            let responses: Vec<JournalEntryResponse> = entries
                .into_iter()
                .map(|entry| JournalEntryResponse {
                    id: entry.id.to_string(),
                    organization_id: entry.organization_id.to_string(),
                    building_id: entry.building_id.map(|id| id.to_string()),
                    journal_type: entry.journal_type,
                    entry_date: entry.entry_date.to_rfc3339(),
                    description: entry.description,
                    document_ref: entry.document_ref,
                    expense_id: entry.expense_id.map(|id| id.to_string()),
                    contribution_id: entry.contribution_id.map(|id| id.to_string()),
                    created_at: entry.created_at.to_rfc3339(),
                    updated_at: entry.updated_at.to_rfc3339(),
                })
                .collect();

            HttpResponse::Ok().json(serde_json::json!({
                "data": responses,
                "page": page,
                "per_page": per_page
            }))
        }
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": err
        })),
    }
}

/// Get a single journal entry with its lines
///
/// **Access:** Accountant, SuperAdmin, Syndic
///
/// **Example:**
/// ```
/// GET /api/v1/journal-entries/{id}
/// ```
#[get("/journal-entries/{id}")]
pub async fn get_journal_entry(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    entry_id: web::Path<Uuid>,
) -> impl Responder {
    if !matches!(user.role.as_str(), "accountant" | "superadmin" | "syndic") {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Only accountants, syndics, and superadmins can view journal entries"
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
        .journal_entry_use_cases
        .get_entry_with_lines(*entry_id, organization_id)
        .await
    {
        Ok((entry, lines)) => {
            let entry_response = JournalEntryResponse {
                id: entry.id.to_string(),
                organization_id: entry.organization_id.to_string(),
                building_id: entry.building_id.map(|id| id.to_string()),
                journal_type: entry.journal_type,
                entry_date: entry.entry_date.to_rfc3339(),
                description: entry.description,
                document_ref: entry.document_ref,
                expense_id: entry.expense_id.map(|id| id.to_string()),
                contribution_id: entry.contribution_id.map(|id| id.to_string()),
                created_at: entry.created_at.to_rfc3339(),
                updated_at: entry.updated_at.to_rfc3339(),
            };

            let lines_response: Vec<JournalEntryLineResponse> = lines
                .into_iter()
                .map(|line| JournalEntryLineResponse {
                    id: line.id.to_string(),
                    journal_entry_id: line.journal_entry_id.to_string(),
                    account_code: line.account_code,
                    debit: line.debit,
                    credit: line.credit,
                    description: line.description,
                    created_at: line.created_at.to_rfc3339(),
                })
                .collect();

            HttpResponse::Ok().json(JournalEntryWithLinesResponse {
                entry: entry_response,
                lines: lines_response,
            })
        }
        Err(err) => HttpResponse::NotFound().json(serde_json::json!({
            "error": err
        })),
    }
}

/// Delete a manual journal entry
///
/// **Access:** Accountant, SuperAdmin
///
/// **Note:** Only manual entries (not auto-generated from expenses/contributions) can be deleted.
///
/// **Example:**
/// ```
/// DELETE /api/v1/journal-entries/{id}
/// ```
#[delete("/journal-entries/{id}")]
pub async fn delete_journal_entry(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    entry_id: web::Path<Uuid>,
) -> impl Responder {
    if !matches!(user.role.as_str(), "accountant" | "superadmin") {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Only accountants and superadmins can delete journal entries"
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
        .journal_entry_use_cases
        .delete_manual_entry(*entry_id, organization_id)
        .await
    {
        Ok(_) => {
            // Audit log
            AuditLogEntry::new(
                AuditEventType::JournalEntryDeleted,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_metadata(serde_json::json!({
                "entity_type": "journal_entry",
                "entry_id": entry_id.to_string()
            }))
            .log();

            HttpResponse::NoContent().finish()
        }
        Err(err) => {
            // Audit log failure
            AuditLogEntry::new(
                AuditEventType::JournalEntryDeleted,
                Some(user.user_id),
                Some(organization_id),
            )
            .with_metadata(serde_json::json!({
                "entity_type": "journal_entry",
                "entry_id": entry_id.to_string()
            }))
            .with_error(err.clone())
            .log();

            HttpResponse::BadRequest().json(serde_json::json!({
                "error": err
            }))
        }
    }
}
