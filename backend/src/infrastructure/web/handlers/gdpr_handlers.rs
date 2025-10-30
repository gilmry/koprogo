use crate::infrastructure::audit::{AuditEventType, AuditLogEntry};
use crate::infrastructure::web::{AppState, AuthenticatedUser};
use actix_web::{delete, get, web, HttpResponse, Responder};
use tokio::spawn;

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
    data: web::Data<AppState>,
    auth: AuthenticatedUser,
) -> impl Responder {
    // Extract user_id from authenticated user
    let user_id = auth.user_id;

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
            // Audit log: successful GDPR data export (async with database persistence)
            let audit_entry = AuditLogEntry::new(
                AuditEventType::GdprDataExported,
                Some(user_id),
                organization_id,
            )
            .with_resource("User", user_id)
            .with_metadata(serde_json::json!({
                "total_items": export_data.total_items,
                "export_date": export_data.export_date
            }));

            let audit_logger = data.audit_logger.clone();
            spawn(async move {
                audit_logger.log(&audit_entry).await;
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
pub async fn erase_user_data(data: web::Data<AppState>, auth: AuthenticatedUser) -> impl Responder {
    // Extract user_id from authenticated user
    let user_id = auth.user_id;

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
            // Audit log: successful GDPR data erasure (async with database persistence)
            let audit_entry = AuditLogEntry::new(
                AuditEventType::GdprDataErased,
                Some(user_id),
                organization_id,
            )
            .with_resource("User", user_id)
            .with_metadata(serde_json::json!({
                "owners_anonymized": erase_response.owners_anonymized,
                "anonymized_at": erase_response.anonymized_at
            }));

            let audit_logger = data.audit_logger.clone();
            spawn(async move {
                audit_logger.log(&audit_entry).await;
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
pub async fn can_erase_user(data: web::Data<AppState>, auth: AuthenticatedUser) -> impl Responder {
    let user_id = auth.user_id;

    match data.gdpr_use_cases.can_erase_user(user_id).await {
        Ok(can_erase) => {
            // Audit log: erasure check requested (async with database persistence)
            let audit_entry = AuditLogEntry::new(
                AuditEventType::GdprErasureCheckRequested,
                Some(user_id),
                auth.organization_id,
            )
            .with_resource("User", user_id)
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
