use crate::infrastructure::audit::{AuditEventType, AuditLogEntry};
use crate::infrastructure::web::{AppState, AuthenticatedUser};
use actix_web::{delete, get, web, HttpRequest, HttpResponse, Responder};
use tokio::spawn;

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

/// GET /api/v1/gdpr/export
/// Export all personal data for the authenticated user (GDPR Article 15 - Right to Access)
///
/// # Returns
/// * `200 OK` - JSON with complete user data export
/// * `401 Unauthorized` - Missing or invalid authentication
/// * `403 Forbidden` - User attempting to export another user's data
/// * `404 Not Found` - User not found
/// * `500 Internal Server Error` - Database or processing error
#[get("/gdpr/export")]
pub async fn export_user_data(
    req: HttpRequest,
    data: web::Data<AppState>,
    auth: AuthenticatedUser,
) -> impl Responder {
    // Extract user_id from authenticated user
    let user_id = auth.user_id;

    // Extract client information for audit logging
    let ip_address = extract_ip_address(&req);
    let user_agent = extract_user_agent(&req);

    // Determine organization scope based on role
    // SuperAdmin can export across all organizations (organization_id = None)
    // Regular users are scoped to their organization
    let organization_id = if auth.role == "superadmin" {
        None
    } else {
        auth.organization_id
    };

    // Call use case to export data
    match data
        .gdpr_use_cases
        .export_user_data(user_id, user_id, organization_id)
        .await
    {
        Ok(export_data) => {
            // Extract user info for email notification
            let user_email = export_data.user.email.clone();
            let user_name = format!(
                "{} {}",
                export_data.user.first_name, export_data.user.last_name
            );

            // Audit log: successful GDPR data export (async with database persistence)
            let audit_entry = AuditLogEntry::new(
                AuditEventType::GdprDataExported,
                Some(user_id),
                organization_id,
            )
            .with_resource("User", user_id)
            .with_client_info(ip_address, user_agent)
            .with_metadata(serde_json::json!({
                "total_items": export_data.total_items,
                "export_date": export_data.export_date
            }));

            let audit_logger = data.audit_logger.clone();
            spawn(async move {
                audit_logger.log(&audit_entry).await;
            });

            // Send email notification (async)
            let email_service = data.email_service.clone();
            spawn(async move {
                if let Err(e) = email_service
                    .send_gdpr_export_notification(&user_email, &user_name, user_id)
                    .await
                {
                    log::error!("Failed to send GDPR export email notification: {}", e);
                }
            });

            HttpResponse::Ok().json(export_data)
        }
        Err(e) => {
            // Audit log: failed GDPR data export (async with database persistence)
            let audit_entry = AuditLogEntry::new(
                AuditEventType::GdprDataExportFailed,
                Some(user_id),
                organization_id,
            )
            .with_resource("User", user_id)
            .with_client_info(ip_address, user_agent)
            .with_error(e.clone());

            let audit_logger = data.audit_logger.clone();
            spawn(async move {
                audit_logger.log(&audit_entry).await;
            });

            if e.contains("not found") {
                HttpResponse::NotFound().json(serde_json::json!({
                    "error": e
                }))
            } else if e.contains("Unauthorized") {
                HttpResponse::Forbidden().json(serde_json::json!({
                    "error": e
                }))
            } else if e.contains("anonymized") {
                HttpResponse::Gone().json(serde_json::json!({
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

/// DELETE /api/v1/gdpr/erase
/// Erase user personal data by anonymization (GDPR Article 17 - Right to Erasure)
///
/// This endpoint anonymizes the user's account and all linked owner profiles.
/// Data is not deleted entirely to preserve referential integrity and comply with
/// legal retention requirements (e.g., financial records must be kept for 7 years).
///
/// # Returns
/// * `200 OK` - JSON confirmation of successful anonymization
/// * `401 Unauthorized` - Missing or invalid authentication
/// * `403 Forbidden` - User attempting to erase another user's data
/// * `409 Conflict` - Legal holds prevent erasure (e.g., unpaid expenses)
/// * `410 Gone` - User already anonymized
/// * `500 Internal Server Error` - Database or processing error
#[delete("/gdpr/erase")]
pub async fn erase_user_data(
    req: HttpRequest,
    data: web::Data<AppState>,
    auth: AuthenticatedUser,
) -> impl Responder {
    // Extract user_id from authenticated user
    let user_id = auth.user_id;

    // Extract client information for audit logging
    let ip_address = extract_ip_address(&req);
    let user_agent = extract_user_agent(&req);

    // Determine organization scope based on role
    let organization_id = if auth.role == "superadmin" {
        None
    } else {
        auth.organization_id
    };

    // Call use case to erase data
    match data
        .gdpr_use_cases
        .erase_user_data(user_id, user_id, organization_id)
        .await
    {
        Ok(erase_response) => {
            // Extract user info for email notification
            let user_email = erase_response.user_email.clone();
            let user_name = format!(
                "{} {}",
                erase_response.user_first_name, erase_response.user_last_name
            );
            let owners_count = erase_response.owners_anonymized;

            // Audit log: successful GDPR data erasure (async with database persistence)
            let audit_entry = AuditLogEntry::new(
                AuditEventType::GdprDataErased,
                Some(user_id),
                organization_id,
            )
            .with_resource("User", user_id)
            .with_client_info(ip_address, user_agent)
            .with_metadata(serde_json::json!({
                "owners_anonymized": erase_response.owners_anonymized,
                "anonymized_at": erase_response.anonymized_at
            }));

            let audit_logger = data.audit_logger.clone();
            spawn(async move {
                audit_logger.log(&audit_entry).await;
            });

            // Send email notification (async)
            let email_service = data.email_service.clone();
            spawn(async move {
                if let Err(e) = email_service
                    .send_gdpr_erasure_notification(&user_email, &user_name, owners_count)
                    .await
                {
                    log::error!("Failed to send GDPR erasure email notification: {}", e);
                }
            });

            HttpResponse::Ok().json(erase_response)
        }
        Err(e) => {
            // Audit log: failed GDPR data erasure (async with database persistence)
            let audit_entry = AuditLogEntry::new(
                AuditEventType::GdprDataErasureFailed,
                Some(user_id),
                organization_id,
            )
            .with_resource("User", user_id)
            .with_client_info(ip_address, user_agent)
            .with_error(e.clone());

            let audit_logger = data.audit_logger.clone();
            spawn(async move {
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

/// GET /api/v1/gdpr/can-erase
/// Check if user data can be erased (no legal holds)
///
/// # Returns
/// * `200 OK` - JSON with erasure eligibility status
/// * `401 Unauthorized` - Missing or invalid authentication
/// * `500 Internal Server Error` - Database or processing error
#[get("/gdpr/can-erase")]
pub async fn can_erase_user(
    req: HttpRequest,
    data: web::Data<AppState>,
    auth: AuthenticatedUser,
) -> impl Responder {
    let user_id = auth.user_id;

    // Extract client information for audit logging
    let ip_address = extract_ip_address(&req);
    let user_agent = extract_user_agent(&req);

    match data.gdpr_use_cases.can_erase_user(user_id).await {
        Ok(can_erase) => {
            // Audit log: erasure check requested (async with database persistence)
            let audit_entry = AuditLogEntry::new(
                AuditEventType::GdprErasureCheckRequested,
                Some(user_id),
                auth.organization_id,
            )
            .with_resource("User", user_id)
            .with_client_info(ip_address, user_agent)
            .with_metadata(serde_json::json!({
                "can_erase": can_erase
            }));

            let audit_logger = data.audit_logger.clone();
            spawn(async move {
                audit_logger.log(&audit_entry).await;
            });

            HttpResponse::Ok().json(serde_json::json!({
                "can_erase": can_erase,
                "user_id": user_id.to_string()
            }))
        }
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to check erasure eligibility: {}", e)
        })),
    }
}

#[cfg(test)]
mod tests {
    // Note: Full integration tests with actual AppState would require proper initialization
    // of all use cases. These handler tests are covered by E2E tests in tests/e2e/

    #[test]
    fn test_handler_structure_export() {
        // This test just verifies the handler function signature compiles
        // Real testing happens in E2E tests with testcontainers
    }

    #[test]
    fn test_handler_structure_erase() {
        // This test just verifies the handler function signature compiles
    }

    #[test]
    fn test_handler_structure_can_erase() {
        // This test just verifies the handler function signature compiles
    }
}
