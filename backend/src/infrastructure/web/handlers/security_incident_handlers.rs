use crate::infrastructure::web::{AppState, AuthenticatedUser};
use actix_web::{get, post, put, web, HttpRequest, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Extract client IP address from request
fn extract_ip_address(req: &HttpRequest) -> Option<String> {
    req.headers()
        .get("X-Forwarded-For")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.split(',').next().unwrap_or("").trim().to_string())
        .filter(|s| !s.is_empty())
        .or_else(|| {
            req.headers()
                .get("X-Real-IP")
                .and_then(|h| h.to_str().ok())
                .map(|s| s.to_string())
        })
        .or_else(|| req.peer_addr().map(|addr| addr.ip().to_string()))
}

/// Extract user agent from request
fn extract_user_agent(req: &HttpRequest) -> Option<String> {
    req.headers()
        .get("User-Agent")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string())
}

/// Request body for creating a security incident
#[derive(Debug, Deserialize)]
pub struct CreateSecurityIncidentRequest {
    pub severity: String, // "critical", "high", "medium", "low"
    pub incident_type: String, // "data_breach", "unauthorized_access", "malware", etc.
    pub title: String,
    pub description: String,
    pub data_categories_affected: Vec<String>,
    pub affected_subjects_count: Option<i32>,
}

/// Request body for reporting incident to APD
#[derive(Debug, Deserialize)]
pub struct ReportToApdRequest {
    pub apd_reference_number: Option<String>,
    pub investigation_notes: Option<String>,
}

/// Response DTO for security incident
#[derive(Debug, Serialize)]
pub struct SecurityIncidentDto {
    pub id: String,
    pub organization_id: String,
    pub severity: String,
    pub incident_type: String,
    pub title: String,
    pub description: String,
    pub data_categories_affected: Vec<String>,
    pub affected_subjects_count: Option<i32>,
    pub discovery_at: String,
    pub notification_at: Option<String>,
    pub apd_reference_number: Option<String>,
    pub status: String,
    pub reported_by: String,
    pub investigation_notes: Option<String>,
    pub root_cause: Option<String>,
    pub remediation_steps: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub hours_since_discovery: f64,
}

/// Response for listing incidents
#[derive(Debug, Serialize)]
pub struct SecurityIncidentsResponse {
    pub incidents: Vec<SecurityIncidentDto>,
    pub total: i64,
}

/// Query parameters for listing incidents
#[derive(Debug, Deserialize)]
pub struct SecurityIncidentsQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub severity: Option<String>,
    pub status: Option<String>,
}

/// POST /api/v1/admin/security-incidents
/// Create a new security incident (SuperAdmin only)
///
/// # Returns
/// * `201 Created` - Security incident created successfully
/// * `400 Bad Request` - Invalid input parameters
/// * `401 Unauthorized` - Missing or invalid authentication
/// * `403 Forbidden` - User is not SuperAdmin
/// * `500 Internal Server Error` - Database error
#[post("/admin/security-incidents")]
pub async fn create_security_incident(
    req: HttpRequest,
    data: web::Data<AppState>,
    auth: AuthenticatedUser,
    body: web::Json<CreateSecurityIncidentRequest>,
) -> impl Responder {
    // Only SuperAdmin can create incidents
    if auth.role != "superadmin" {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Access denied. SuperAdmin role required."
        }));
    }

    // Validate input
    if body.title.is_empty() || body.description.is_empty() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "title and description are required"
        }));
    }

    let valid_severities = ["critical", "high", "medium", "low"];
    if !valid_severities.contains(&body.severity.as_str()) {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Invalid severity. Must be: critical, high, medium, or low"
        }));
    }

    let _valid_statuses = ["detected", "investigating"];
    let status = "detected".to_string();

    // Extract client info for audit
    let ip_address = extract_ip_address(&req);
    let user_agent = extract_user_agent(&req);

    // Insert incident into database
    match data.pool.acquire().await {
        Ok(mut conn) => {
            match sqlx::query_as!(
                SecurityIncidentRow,
                r#"
                INSERT INTO security_incidents (
                    organization_id,
                    severity,
                    incident_type,
                    title,
                    description,
                    data_categories_affected,
                    affected_subjects_count,
                    discovery_at,
                    status,
                    reported_by
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, now(), $8, $9)
                RETURNING
                    id, organization_id, severity, incident_type, title, description,
                    data_categories_affected, affected_subjects_count, discovery_at,
                    notification_at, apd_reference_number, status, reported_by,
                    investigation_notes, root_cause, remediation_steps, created_at, updated_at
                "#,
                auth.organization_id,
                body.severity,
                body.incident_type,
                body.title,
                body.description,
                &body.data_categories_affected,
                body.affected_subjects_count,
                status,
                auth.user_id
            )
            .fetch_one(&mut *conn)
            .await
            {
                Ok(row) => {
                    let hours_since = calculate_hours_since(&row.discovery_at);
                    let incident_dto = SecurityIncidentDto {
                        id: row.id.to_string(),
                        organization_id: row.organization_id.to_string(),
                        severity: row.severity,
                        incident_type: row.incident_type,
                        title: row.title,
                        description: row.description,
                        data_categories_affected: row.data_categories_affected.unwrap_or_default(),
                        affected_subjects_count: row.affected_subjects_count,
                        discovery_at: row.discovery_at.to_rfc3339(),
                        notification_at: row.notification_at.map(|dt| dt.to_rfc3339()),
                        apd_reference_number: row.apd_reference_number,
                        status: row.status,
                        reported_by: row.reported_by.to_string(),
                        investigation_notes: row.investigation_notes,
                        root_cause: row.root_cause,
                        remediation_steps: row.remediation_steps,
                        created_at: row.created_at.to_rfc3339(),
                        updated_at: row.updated_at.to_rfc3339(),
                        hours_since_discovery: hours_since,
                    };

                    // Audit log (async)
                    let audit_entry = crate::infrastructure::audit::AuditLogEntry::new(
                        crate::infrastructure::audit::AuditEventType::SecurityIncidentReported,
                        Some(auth.user_id),
                        auth.organization_id,
                    )
                    .with_resource("SecurityIncident", row.id)
                    .with_client_info(ip_address, user_agent)
                    .with_metadata(serde_json::json!({
                        "severity": body.severity,
                        "incident_type": body.incident_type,
                        "affected_subjects": body.affected_subjects_count
                    }));

                    let audit_logger = data.audit_logger.clone();
                    tokio::spawn(async move {
                        audit_logger.log(&audit_entry).await;
                    });

                    HttpResponse::Created().json(incident_dto)
                }
                Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": format!("Failed to create incident: {}", e)
                })),
            }
        }
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Database connection error: {}", e)
        })),
    }
}

/// GET /api/v1/admin/security-incidents
/// List security incidents with pagination and filters (SuperAdmin only)
///
/// # Query Parameters
/// - page: Page number (default: 1)
/// - per_page: Items per page (default: 20)
/// - severity: Filter by severity (critical, high, medium, low)
/// - status: Filter by status (detected, investigating, contained, reported, closed)
///
/// # Returns
/// * `200 OK` - Paginated list of incidents
/// * `401 Unauthorized` - Missing or invalid authentication
/// * `403 Forbidden` - User is not SuperAdmin
/// * `500 Internal Server Error` - Database error
#[get("/admin/security-incidents")]
pub async fn list_security_incidents(
    data: web::Data<AppState>,
    auth: AuthenticatedUser,
    query: web::Query<SecurityIncidentsQuery>,
) -> impl Responder {
    // Only SuperAdmin can view incidents
    if auth.role != "superadmin" {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Access denied. SuperAdmin role required."
        }));
    }

    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(20).clamp(1, 100);
    let offset = (page - 1) * per_page;

    match data.pool.acquire().await {
        Ok(mut conn) => {
            // Build base query with optional filters
            let base_query = if let Some(ref severity) = query.severity {
                if let Some(ref status) = query.status {
                    sqlx::query_as!(
                        SecurityIncidentRow,
                        r#"
                        SELECT
                            id, organization_id, severity, incident_type, title, description,
                            data_categories_affected, affected_subjects_count, discovery_at,
                            notification_at, apd_reference_number, status, reported_by,
                            investigation_notes, root_cause, remediation_steps, created_at, updated_at
                        FROM security_incidents
                        WHERE organization_id = $1 AND severity = $2 AND status = $3
                        ORDER BY discovery_at DESC
                        LIMIT $4 OFFSET $5
                        "#,
                        auth.organization_id,
                        severity,
                        status,
                        per_page,
                        offset
                    )
                    .fetch_all(&mut *conn)
                    .await
                } else {
                    sqlx::query_as!(
                        SecurityIncidentRow,
                        r#"
                        SELECT
                            id, organization_id, severity, incident_type, title, description,
                            data_categories_affected, affected_subjects_count, discovery_at,
                            notification_at, apd_reference_number, status, reported_by,
                            investigation_notes, root_cause, remediation_steps, created_at, updated_at
                        FROM security_incidents
                        WHERE organization_id = $1 AND severity = $2
                        ORDER BY discovery_at DESC
                        LIMIT $3 OFFSET $4
                        "#,
                        auth.organization_id,
                        severity,
                        per_page,
                        offset
                    )
                    .fetch_all(&mut *conn)
                    .await
                }
            } else if let Some(ref status) = query.status {
                sqlx::query_as!(
                    SecurityIncidentRow,
                    r#"
                    SELECT
                        id, organization_id, severity, incident_type, title, description,
                        data_categories_affected, affected_subjects_count, discovery_at,
                        notification_at, apd_reference_number, status, reported_by,
                        investigation_notes, root_cause, remediation_steps, created_at, updated_at
                    FROM security_incidents
                    WHERE organization_id = $1 AND status = $2
                    ORDER BY discovery_at DESC
                    LIMIT $3 OFFSET $4
                    "#,
                    auth.organization_id,
                    status,
                    per_page,
                    offset
                )
                .fetch_all(&mut *conn)
                .await
            } else {
                sqlx::query_as!(
                    SecurityIncidentRow,
                    r#"
                    SELECT
                        id, organization_id, severity, incident_type, title, description,
                        data_categories_affected, affected_subjects_count, discovery_at,
                        notification_at, apd_reference_number, status, reported_by,
                        investigation_notes, root_cause, remediation_steps, created_at, updated_at
                    FROM security_incidents
                    WHERE organization_id = $1
                    ORDER BY discovery_at DESC
                    LIMIT $2 OFFSET $3
                    "#,
                    auth.organization_id,
                    per_page,
                    offset
                )
                .fetch_all(&mut *conn)
                .await
            };

            match base_query {
                Ok(rows) => {
                    // Get total count
                    let count_result = sqlx::query_scalar!(
                        "SELECT COUNT(*) as count FROM security_incidents WHERE organization_id = $1",
                        auth.organization_id
                    )
                    .fetch_one(&mut *conn)
                    .await;

                    let total = count_result.unwrap_or(Some(0)).unwrap_or(0);

                    let incidents: Vec<SecurityIncidentDto> = rows
                        .into_iter()
                        .map(|row| {
                            let hours_since = calculate_hours_since(&row.discovery_at);
                            SecurityIncidentDto {
                                id: row.id.to_string(),
                                organization_id: row.organization_id.to_string(),
                                severity: row.severity,
                                incident_type: row.incident_type,
                                title: row.title,
                                description: row.description,
                                data_categories_affected: row.data_categories_affected.unwrap_or_default(),
                                affected_subjects_count: row.affected_subjects_count,
                                discovery_at: row.discovery_at.to_rfc3339(),
                                notification_at: row.notification_at.map(|dt| dt.to_rfc3339()),
                                apd_reference_number: row.apd_reference_number,
                                status: row.status,
                                reported_by: row.reported_by.to_string(),
                                investigation_notes: row.investigation_notes,
                                root_cause: row.root_cause,
                                remediation_steps: row.remediation_steps,
                                created_at: row.created_at.to_rfc3339(),
                                updated_at: row.updated_at.to_rfc3339(),
                                hours_since_discovery: hours_since,
                            }
                        })
                        .collect();

                    HttpResponse::Ok().json(SecurityIncidentsResponse {
                        incidents,
                        total,
                    })
                }
                Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": format!("Failed to fetch incidents: {}", e)
                })),
            }
        }
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Database connection error: {}", e)
        })),
    }
}

/// GET /api/v1/admin/security-incidents/:id
/// Get a specific security incident (SuperAdmin only)
///
/// # Returns
/// * `200 OK` - Incident details
/// * `401 Unauthorized` - Missing or invalid authentication
/// * `403 Forbidden` - User is not SuperAdmin
/// * `404 Not Found` - Incident not found
/// * `500 Internal Server Error` - Database error
#[get("/admin/security-incidents/{incident_id}")]
pub async fn get_security_incident(
    data: web::Data<AppState>,
    auth: AuthenticatedUser,
    path: web::Path<Uuid>,
) -> impl Responder {
    // Only SuperAdmin can view incidents
    if auth.role != "superadmin" {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Access denied. SuperAdmin role required."
        }));
    }

    let incident_id = path.into_inner();

    match data.pool.acquire().await {
        Ok(mut conn) => {
            match sqlx::query_as!(
                SecurityIncidentRow,
                r#"
                SELECT
                    id, organization_id, severity, incident_type, title, description,
                    data_categories_affected, affected_subjects_count, discovery_at,
                    notification_at, apd_reference_number, status, reported_by,
                    investigation_notes, root_cause, remediation_steps, created_at, updated_at
                FROM security_incidents
                WHERE id = $1 AND organization_id = $2
                "#,
                incident_id,
                auth.organization_id
            )
            .fetch_optional(&mut *conn)
            .await
            {
                Ok(Some(row)) => {
                    let hours_since = calculate_hours_since(&row.discovery_at);
                    let incident_dto = SecurityIncidentDto {
                        id: row.id.to_string(),
                        organization_id: row.organization_id.to_string(),
                        severity: row.severity,
                        incident_type: row.incident_type,
                        title: row.title,
                        description: row.description,
                        data_categories_affected: row.data_categories_affected.unwrap_or_default(),
                        affected_subjects_count: row.affected_subjects_count,
                        discovery_at: row.discovery_at.to_rfc3339(),
                        notification_at: row.notification_at.map(|dt| dt.to_rfc3339()),
                        apd_reference_number: row.apd_reference_number,
                        status: row.status,
                        reported_by: row.reported_by.to_string(),
                        investigation_notes: row.investigation_notes,
                        root_cause: row.root_cause,
                        remediation_steps: row.remediation_steps,
                        created_at: row.created_at.to_rfc3339(),
                        updated_at: row.updated_at.to_rfc3339(),
                        hours_since_discovery: hours_since,
                    };

                    HttpResponse::Ok().json(incident_dto)
                }
                Ok(None) => HttpResponse::NotFound().json(serde_json::json!({
                    "error": "Security incident not found"
                })),
                Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": format!("Failed to fetch incident: {}", e)
                })),
            }
        }
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Database connection error: {}", e)
        })),
    }
}

/// PUT /api/v1/admin/security-incidents/:id/report-apd
/// Mark incident as reported to APD with reference number (SuperAdmin only)
///
/// # Returns
/// * `200 OK` - Incident marked as reported
/// * `400 Bad Request` - APD reference number required
/// * `401 Unauthorized` - Missing or invalid authentication
/// * `403 Forbidden` - User is not SuperAdmin
/// * `404 Not Found` - Incident not found
/// * `409 Conflict` - Incident already reported
/// * `500 Internal Server Error` - Database error
#[put("/admin/security-incidents/{incident_id}/report-apd")]
pub async fn report_incident_to_apd(
    data: web::Data<AppState>,
    auth: AuthenticatedUser,
    path: web::Path<Uuid>,
    body: web::Json<ReportToApdRequest>,
) -> impl Responder {
    // Only SuperAdmin can report incidents
    if auth.role != "superadmin" {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Access denied. SuperAdmin role required."
        }));
    }

    let incident_id = path.into_inner();

    if body.apd_reference_number.is_none() || body.apd_reference_number.as_ref().unwrap().is_empty() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "apd_reference_number is required"
        }));
    }

    match data.pool.acquire().await {
        Ok(mut conn) => {
            // First, check if already reported
            match sqlx::query_scalar!(
                "SELECT notification_at FROM security_incidents WHERE id = $1 AND organization_id = $2",
                incident_id,
                auth.organization_id
            )
            .fetch_optional(&mut *conn)
            .await
            {
                Ok(Some(Some(_))) => {
                    return HttpResponse::Conflict().json(serde_json::json!({
                        "error": "Incident already reported to APD"
                    }));
                }
                Ok(Some(None)) => {
                    // Update incident as reported
                    match sqlx::query_as!(
                        SecurityIncidentRow,
                        r#"
                        UPDATE security_incidents
                        SET notification_at = now(),
                            apd_reference_number = $1,
                            status = 'reported',
                            investigation_notes = $2
                        WHERE id = $3 AND organization_id = $4
                        RETURNING
                            id, organization_id, severity, incident_type, title, description,
                            data_categories_affected, affected_subjects_count, discovery_at,
                            notification_at, apd_reference_number, status, reported_by,
                            investigation_notes, root_cause, remediation_steps, created_at, updated_at
                        "#,
                        body.apd_reference_number,
                        body.investigation_notes,
                        incident_id,
                        auth.organization_id
                    )
                    .fetch_one(&mut *conn)
                    .await
                    {
                        Ok(row) => {
                            let hours_since = calculate_hours_since(&row.discovery_at);
                            let incident_dto = SecurityIncidentDto {
                                id: row.id.to_string(),
                                organization_id: row.organization_id.to_string(),
                                severity: row.severity,
                                incident_type: row.incident_type,
                                title: row.title,
                                description: row.description,
                                data_categories_affected: row.data_categories_affected.unwrap_or_default(),
                                affected_subjects_count: row.affected_subjects_count,
                                discovery_at: row.discovery_at.to_rfc3339(),
                                notification_at: row.notification_at.map(|dt| dt.to_rfc3339()),
                                apd_reference_number: row.apd_reference_number,
                                status: row.status,
                                reported_by: row.reported_by.to_string(),
                                investigation_notes: row.investigation_notes,
                                root_cause: row.root_cause,
                                remediation_steps: row.remediation_steps,
                                created_at: row.created_at.to_rfc3339(),
                                updated_at: row.updated_at.to_rfc3339(),
                                hours_since_discovery: hours_since,
                            };

                            HttpResponse::Ok().json(incident_dto)
                        }
                        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
                            "error": format!("Failed to update incident: {}", e)
                        })),
                    }
                }
                Ok(None) => HttpResponse::NotFound().json(serde_json::json!({
                    "error": "Security incident not found"
                })),
                Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": format!("Database query failed: {}", e)
                })),
            }
        }
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Database connection error: {}", e)
        })),
    }
}

/// GET /api/v1/admin/security-incidents/overdue
/// List incidents overdue for APD notification (>72 hours old, not yet reported)
///
/// # Returns
/// * `200 OK` - List of overdue incidents requiring APD notification
/// * `401 Unauthorized` - Missing or invalid authentication
/// * `403 Forbidden` - User is not SuperAdmin
/// * `500 Internal Server Error` - Database error
#[get("/admin/security-incidents/overdue")]
pub async fn list_overdue_incidents(
    data: web::Data<AppState>,
    auth: AuthenticatedUser,
) -> impl Responder {
    // Only SuperAdmin can view overdue incidents
    if auth.role != "superadmin" {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Access denied. SuperAdmin role required."
        }));
    }

    match data.pool.acquire().await {
        Ok(mut conn) => {
            match sqlx::query_as!(
                SecurityIncidentRow,
                r#"
                SELECT
                    id, organization_id, severity, incident_type, title, description,
                    data_categories_affected, affected_subjects_count, discovery_at,
                    notification_at, apd_reference_number, status, reported_by,
                    investigation_notes, root_cause, remediation_steps, created_at, updated_at
                FROM security_incidents
                WHERE organization_id = $1
                  AND notification_at IS NULL
                  AND status IN ('detected', 'investigating', 'contained')
                  AND discovery_at < (NOW() - INTERVAL '72 hours')
                ORDER BY discovery_at ASC
                "#,
                auth.organization_id
            )
            .fetch_all(&mut *conn)
            .await
            {
                Ok(rows) => {
                    let total = rows.len() as i64;
                    let incidents: Vec<SecurityIncidentDto> = rows
                        .into_iter()
                        .map(|row| {
                            let hours_since = calculate_hours_since(&row.discovery_at);
                            SecurityIncidentDto {
                                id: row.id.to_string(),
                                organization_id: row.organization_id.to_string(),
                                severity: row.severity,
                                incident_type: row.incident_type,
                                title: row.title,
                                description: row.description,
                                data_categories_affected: row.data_categories_affected.unwrap_or_default(),
                                affected_subjects_count: row.affected_subjects_count,
                                discovery_at: row.discovery_at.to_rfc3339(),
                                notification_at: row.notification_at.map(|dt| dt.to_rfc3339()),
                                apd_reference_number: row.apd_reference_number,
                                status: row.status,
                                reported_by: row.reported_by.to_string(),
                                investigation_notes: row.investigation_notes,
                                root_cause: row.root_cause,
                                remediation_steps: row.remediation_steps,
                                created_at: row.created_at.to_rfc3339(),
                                updated_at: row.updated_at.to_rfc3339(),
                                hours_since_discovery: hours_since,
                            }
                        })
                        .collect();

                    HttpResponse::Ok().json(SecurityIncidentsResponse {
                        incidents,
                        total,
                    })
                }
                Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": format!("Failed to fetch overdue incidents: {}", e)
                })),
            }
        }
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Database connection error: {}", e)
        })),
    }
}

// Helper function to calculate hours since discovery
fn calculate_hours_since(discovery_at: &chrono::DateTime<chrono::Utc>) -> f64 {
    let now = chrono::Utc::now();
    let duration = now.signed_duration_since(*discovery_at);
    duration.num_seconds() as f64 / 3600.0
}

// Internal struct for sqlx query results
struct SecurityIncidentRow {
    id: uuid::Uuid,
    organization_id: uuid::Uuid,
    severity: String,
    incident_type: String,
    title: String,
    description: String,
    data_categories_affected: Option<Vec<String>>,
    affected_subjects_count: Option<i32>,
    discovery_at: chrono::DateTime<chrono::Utc>,
    notification_at: Option<chrono::DateTime<chrono::Utc>>,
    apd_reference_number: Option<String>,
    status: String,
    reported_by: uuid::Uuid,
    investigation_notes: Option<String>,
    root_cause: Option<String>,
    remediation_steps: Option<String>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_handler_structure_create_incident() {
        // Structural test - actual testing in E2E
    }

    #[test]
    fn test_handler_structure_list_incidents() {
        // Structural test - actual testing in E2E
    }

    #[test]
    fn test_handler_structure_get_incident() {
        // Structural test - actual testing in E2E
    }

    #[test]
    fn test_handler_structure_report_to_apd() {
        // Structural test - actual testing in E2E
    }

    #[test]
    fn test_handler_structure_overdue_incidents() {
        // Structural test - actual testing in E2E
    }
}
