use crate::application::dto::PageRequest;
use crate::application::ports::{AuditLogFilters, AuditLogRepository};
use crate::infrastructure::web::{AppState, AuthenticatedUser};
use actix_web::{delete, get, web, HttpRequest, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Extract client IP address from request
fn extract_ip_address(req: &HttpRequest) -> Option<String> {
    // Try X-Forwarded-For first (for proxy/load balancer scenarios)
    req.headers()
        .get("X-Forwarded-For")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.split(',').next().unwrap_or("").trim().to_string())
        .filter(|s| !s.is_empty())
        .or_else(|| {
            // Try X-Real-IP header
            req.headers()
                .get("X-Real-IP")
                .and_then(|h| h.to_str().ok())
                .map(|s| s.to_string())
        })
        .or_else(|| {
            // Fall back to peer address
            req.peer_addr().map(|addr| addr.ip().to_string())
        })
}

/// Extract user agent from request
fn extract_user_agent(req: &HttpRequest) -> Option<String> {
    req.headers()
        .get("User-Agent")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string())
}

/// Query parameters for audit log filtering
#[derive(Debug, Deserialize)]
pub struct AuditLogQuery {
    /// Page number (default: 1)
    pub page: Option<i64>,
    /// Items per page (default: 20, max: 100)
    pub per_page: Option<i64>,
    /// Filter by user_id
    pub user_id: Option<Uuid>,
    /// Filter by organization_id
    pub organization_id: Option<Uuid>,
    /// Filter by event_type (e.g., "GdprDataExported")
    pub event_type: Option<String>,
    /// Filter by success status
    pub success: Option<bool>,
    /// Filter by start date (ISO 8601)
    pub start_date: Option<String>,
    /// Filter by end date (ISO 8601)
    pub end_date: Option<String>,
}

/// Response for paginated audit logs
#[derive(Debug, Serialize)]
pub struct AuditLogsResponse {
    pub logs: Vec<AuditLogDto>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
    pub total_pages: i64,
}

/// DTO for audit log entry
#[derive(Debug, Serialize)]
pub struct AuditLogDto {
    pub id: String,
    pub timestamp: String,
    pub event_type: String,
    pub user_id: Option<String>,
    pub organization_id: Option<String>,
    pub resource_type: Option<String>,
    pub resource_id: Option<String>,
    pub success: bool,
    pub error_message: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

/// GET /api/v1/admin/gdpr/audit-logs
/// List audit logs with pagination and filters (SuperAdmin only)
///
/// # Query Parameters
/// - page: Page number (default: 1)
/// - per_page: Items per page (default: 20, max: 100)
/// - user_id: Filter by user UUID
/// - organization_id: Filter by organization UUID
/// - event_type: Filter by event type (e.g., "GdprDataExported")
/// - success: Filter by success status (true/false)
/// - start_date: Filter by start date (ISO 8601)
/// - end_date: Filter by end date (ISO 8601)
///
/// # Returns
/// * `200 OK` - Paginated audit logs
/// * `401 Unauthorized` - Missing or invalid authentication
/// * `403 Forbidden` - User is not SuperAdmin
/// * `500 Internal Server Error` - Database error
#[get("/admin/gdpr/audit-logs")]
pub async fn list_audit_logs(
    data: web::Data<AppState>,
    auth: AuthenticatedUser,
    query: web::Query<AuditLogQuery>,
) -> impl Responder {
    // Only SuperAdmin can view audit logs
    if auth.role != "superadmin" {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Access denied. SuperAdmin role required."
        }));
    }

    // Build pagination
    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(20).clamp(1, 100);
    let page_request = PageRequest {
        page,
        per_page,
        sort_by: Some("timestamp".to_string()),
        order: crate::application::dto::SortOrder::Desc,
    };

    // Build filters
    let mut filters = AuditLogFilters {
        user_id: query.user_id,
        organization_id: query.organization_id,
        success: query.success,
        ..Default::default()
    };

    // Parse event_type string to enum
    if let Some(ref event_type_str) = query.event_type {
        filters.event_type = parse_event_type(event_type_str);
    }

    // Parse dates
    if let Some(ref start_date_str) = query.start_date {
        if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(start_date_str) {
            filters.start_date = Some(dt.with_timezone(&chrono::Utc));
        }
    }
    if let Some(ref end_date_str) = query.end_date {
        if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(end_date_str) {
            filters.end_date = Some(dt.with_timezone(&chrono::Utc));
        }
    }

    // Fetch audit logs from repository
    let audit_repo =
        crate::infrastructure::database::PostgresAuditLogRepository::new(data.pool.clone());
    match audit_repo.find_all_paginated(&page_request, &filters).await {
        Ok((logs, total)) => {
            let total_pages = (total as f64 / per_page as f64).ceil() as i64;

            let logs_dto: Vec<AuditLogDto> = logs
                .iter()
                .map(|log| AuditLogDto {
                    id: log.id.to_string(),
                    timestamp: log.timestamp.to_rfc3339(),
                    event_type: format!("{:?}", log.event_type),
                    user_id: log.user_id.map(|id| id.to_string()),
                    organization_id: log.organization_id.map(|id| id.to_string()),
                    resource_type: log.resource_type.clone(),
                    resource_id: log.resource_id.map(|id| id.to_string()),
                    success: log.success,
                    error_message: log.error_message.clone(),
                    metadata: log.metadata.clone(),
                })
                .collect();

            HttpResponse::Ok().json(AuditLogsResponse {
                logs: logs_dto,
                total,
                page,
                per_page,
                total_pages,
            })
        }
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to fetch audit logs: {}", e)
        })),
    }
}

/// Parse event_type string to AuditEventType enum
fn parse_event_type(s: &str) -> Option<crate::infrastructure::audit::AuditEventType> {
    use crate::infrastructure::audit::AuditEventType;
    match s {
        "UserLogin" => Some(AuditEventType::UserLogin),
        "UserLogout" => Some(AuditEventType::UserLogout),
        "UserRegistration" => Some(AuditEventType::UserRegistration),
        "GdprDataExported" => Some(AuditEventType::GdprDataExported),
        "GdprDataExportFailed" => Some(AuditEventType::GdprDataExportFailed),
        "GdprDataErased" => Some(AuditEventType::GdprDataErased),
        "GdprDataErasureFailed" => Some(AuditEventType::GdprDataErasureFailed),
        "GdprErasureCheckRequested" => Some(AuditEventType::GdprErasureCheckRequested),
        _ => None,
    }
}

/// GET /api/v1/admin/gdpr/users/:id/export
/// Export user data as SuperAdmin (for compliance requests)
///
/// # Returns
/// * `200 OK` - JSON with complete user data export
/// * `401 Unauthorized` - Missing or invalid authentication
/// * `403 Forbidden` - User is not SuperAdmin
/// * `404 Not Found` - User not found
/// * `500 Internal Server Error` - Database error
#[get("/admin/gdpr/users/{user_id}/export")]
pub async fn admin_export_user_data(
    req: HttpRequest,
    data: web::Data<AppState>,
    auth: AuthenticatedUser,
    path: web::Path<Uuid>,
) -> impl Responder {
    // Only SuperAdmin can perform admin exports
    if auth.role != "superadmin" {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Access denied. SuperAdmin role required."
        }));
    }

    let target_user_id = path.into_inner();

    // Extract client information for audit logging
    let ip_address = extract_ip_address(&req);
    let user_agent = extract_user_agent(&req);

    // SuperAdmin can export any user's data (no organization restriction)
    match data
        .gdpr_use_cases
        .export_user_data(auth.user_id, target_user_id, None)
        .await
    {
        Ok(export_data) => {
            // Extract user info for email notification
            let user_email = export_data.user.email.clone();
            let user_name = format!(
                "{} {}",
                export_data.user.first_name, export_data.user.last_name
            );
            let admin_email = auth.email.clone();

            // Audit log: admin-initiated export
            let audit_entry = crate::infrastructure::audit::AuditLogEntry::new(
                crate::infrastructure::audit::AuditEventType::GdprDataExported,
                Some(auth.user_id),
                auth.organization_id,
            )
            .with_resource("User", target_user_id)
            .with_client_info(ip_address, user_agent)
            .with_metadata(serde_json::json!({
                "total_items": export_data.total_items,
                "export_date": export_data.export_date,
                "admin_initiated": true,
                "target_user_id": target_user_id.to_string()
            }));

            let audit_logger = data.audit_logger.clone();
            tokio::spawn(async move {
                audit_logger.log(&audit_entry).await;
            });

            // Send admin notification email (async)
            let email_service = data.email_service.clone();
            tokio::spawn(async move {
                if let Err(e) = email_service
                    .send_admin_gdpr_notification(
                        &user_email,
                        &user_name,
                        "Data Export",
                        &admin_email,
                    )
                    .await
                {
                    log::error!("Failed to send admin GDPR export email notification: {}", e);
                }
            });

            HttpResponse::Ok().json(export_data)
        }
        Err(e) => {
            // Audit log: failed admin export
            let audit_entry = crate::infrastructure::audit::AuditLogEntry::new(
                crate::infrastructure::audit::AuditEventType::GdprDataExportFailed,
                Some(auth.user_id),
                auth.organization_id,
            )
            .with_resource("User", target_user_id)
            .with_client_info(ip_address, user_agent)
            .with_error(e.clone())
            .with_metadata(serde_json::json!({
                "admin_initiated": true,
                "target_user_id": target_user_id.to_string()
            }));

            let audit_logger = data.audit_logger.clone();
            tokio::spawn(async move {
                audit_logger.log(&audit_entry).await;
            });

            if e.contains("not found") {
                HttpResponse::NotFound().json(serde_json::json!({
                    "error": e
                }))
            } else {
                HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": format!("Failed to export user data: {}", e)
                }))
            }
        }
    }
}

/// DELETE /api/v1/admin/gdpr/users/:id/erase
/// Erase user data as SuperAdmin (for compliance requests or account cleanup)
///
/// # Returns
/// * `200 OK` - JSON confirmation of successful anonymization
/// * `401 Unauthorized` - Missing or invalid authentication
/// * `403 Forbidden` - User is not SuperAdmin
/// * `404 Not Found` - User not found
/// * `409 Conflict` - Legal holds prevent erasure
/// * `410 Gone` - User already anonymized
/// * `500 Internal Server Error` - Database error
#[delete("/admin/gdpr/users/{user_id}/erase")]
pub async fn admin_erase_user_data(
    req: HttpRequest,
    data: web::Data<AppState>,
    auth: AuthenticatedUser,
    path: web::Path<Uuid>,
) -> impl Responder {
    // Only SuperAdmin can perform admin erasures
    if auth.role != "superadmin" {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Access denied. SuperAdmin role required."
        }));
    }

    let target_user_id = path.into_inner();

    // Extract client information for audit logging
    let ip_address = extract_ip_address(&req);
    let user_agent = extract_user_agent(&req);

    // SuperAdmin can erase any user's data (no organization restriction)
    match data
        .gdpr_use_cases
        .erase_user_data(auth.user_id, target_user_id, None)
        .await
    {
        Ok(erase_response) => {
            // Extract user info for email notification
            let user_email = erase_response.user_email.clone();
            let user_name = format!(
                "{} {}",
                erase_response.user_first_name, erase_response.user_last_name
            );
            let admin_email = auth.email.clone();

            // Audit log: admin-initiated erasure
            let audit_entry = crate::infrastructure::audit::AuditLogEntry::new(
                crate::infrastructure::audit::AuditEventType::GdprDataErased,
                Some(auth.user_id),
                auth.organization_id,
            )
            .with_resource("User", target_user_id)
            .with_client_info(ip_address, user_agent)
            .with_metadata(serde_json::json!({
                "owners_anonymized": erase_response.owners_anonymized,
                "anonymized_at": erase_response.anonymized_at,
                "admin_initiated": true,
                "target_user_id": target_user_id.to_string()
            }));

            let audit_logger = data.audit_logger.clone();
            tokio::spawn(async move {
                audit_logger.log(&audit_entry).await;
            });

            // Send admin notification email (async)
            let email_service = data.email_service.clone();
            tokio::spawn(async move {
                if let Err(e) = email_service
                    .send_admin_gdpr_notification(
                        &user_email,
                        &user_name,
                        "Data Erasure",
                        &admin_email,
                    )
                    .await
                {
                    log::error!(
                        "Failed to send admin GDPR erasure email notification: {}",
                        e
                    );
                }
            });

            HttpResponse::Ok().json(erase_response)
        }
        Err(e) => {
            // Audit log: failed admin erasure
            let audit_entry = crate::infrastructure::audit::AuditLogEntry::new(
                crate::infrastructure::audit::AuditEventType::GdprDataErasureFailed,
                Some(auth.user_id),
                auth.organization_id,
            )
            .with_resource("User", target_user_id)
            .with_client_info(ip_address, user_agent)
            .with_error(e.clone())
            .with_metadata(serde_json::json!({
                "admin_initiated": true,
                "target_user_id": target_user_id.to_string()
            }));

            let audit_logger = data.audit_logger.clone();
            tokio::spawn(async move {
                audit_logger.log(&audit_entry).await;
            });

            if e.contains("Unauthorized") {
                HttpResponse::Forbidden().json(serde_json::json!({
                    "error": e
                }))
            } else if e.contains("already anonymized") {
                HttpResponse::Gone().json(serde_json::json!({
                    "error": e
                }))
            } else if e.contains("legal holds") {
                HttpResponse::Conflict().json(serde_json::json!({
                    "error": e,
                    "message": "Cannot erase data due to legal obligations. Please resolve pending issues before requesting erasure."
                }))
            } else if e.contains("not found") {
                HttpResponse::NotFound().json(serde_json::json!({
                    "error": e
                }))
            } else {
                HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": format!("Failed to erase user data: {}", e)
                }))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_handler_structure_list_audit_logs() {
        // Structural test - actual testing in E2E
    }

    #[test]
    fn test_handler_structure_admin_export() {
        // Structural test - actual testing in E2E
    }

    #[test]
    fn test_handler_structure_admin_erase() {
        // Structural test - actual testing in E2E
    }
}
