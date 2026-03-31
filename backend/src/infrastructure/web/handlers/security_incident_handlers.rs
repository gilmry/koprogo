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

#[derive(Debug, Deserialize)]
pub struct CreateSecurityIncidentRequest {
    pub severity: String,
    pub incident_type: String,
    pub title: String,
    pub description: String,
    pub data_categories_affected: Vec<String>,
    pub affected_subjects_count: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct ReportToApdRequest {
    pub apd_reference_number: Option<String>,
    pub investigation_notes: Option<String>,
}

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

#[derive(Debug, Serialize)]
pub struct SecurityIncidentsResponse {
    pub incidents: Vec<SecurityIncidentDto>,
    pub total: i64,
}

#[derive(Debug, Deserialize)]
pub struct SecurityIncidentsQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub severity: Option<String>,
    pub status: Option<String>,
}

fn to_dto(inc: crate::domain::entities::SecurityIncident) -> SecurityIncidentDto {
    let hours = inc.hours_since_discovery();
    SecurityIncidentDto {
        id: inc.id.to_string(),
        organization_id: inc
            .organization_id
            .map(|u| u.to_string())
            .unwrap_or_default(),
        severity: inc.severity,
        incident_type: inc.incident_type,
        title: inc.title,
        description: inc.description,
        data_categories_affected: inc.data_categories_affected,
        affected_subjects_count: inc.affected_subjects_count,
        discovery_at: inc.discovery_at.to_rfc3339(),
        notification_at: inc.notification_at.map(|dt| dt.to_rfc3339()),
        apd_reference_number: inc.apd_reference_number,
        status: inc.status,
        reported_by: inc.reported_by.to_string(),
        investigation_notes: inc.investigation_notes,
        root_cause: inc.root_cause,
        remediation_steps: inc.remediation_steps,
        created_at: inc.created_at.to_rfc3339(),
        updated_at: inc.updated_at.to_rfc3339(),
        hours_since_discovery: hours,
    }
}

/// POST /api/v1/admin/security-incidents — SuperAdmin only
#[post("/admin/security-incidents")]
pub async fn create_security_incident(
    req: HttpRequest,
    data: web::Data<AppState>,
    auth: AuthenticatedUser,
    body: web::Json<CreateSecurityIncidentRequest>,
) -> impl Responder {
    if auth.role != "superadmin" {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Access denied. SuperAdmin role required."
        }));
    }

    let ip_address = extract_ip_address(&req);
    let user_agent = extract_user_agent(&req);

    match data
        .security_incident_use_cases
        .create(
            auth.organization_id,
            auth.user_id,
            body.severity.clone(),
            body.incident_type.clone(),
            body.title.clone(),
            body.description.clone(),
            body.data_categories_affected.clone(),
            body.affected_subjects_count,
        )
        .await
    {
        Ok(incident) => {
            let audit_entry = crate::infrastructure::audit::AuditLogEntry::new(
                crate::infrastructure::audit::AuditEventType::SecurityIncidentReported,
                Some(auth.user_id),
                auth.organization_id,
            )
            .with_resource("SecurityIncident", incident.id)
            .with_client_info(ip_address, user_agent)
            .with_metadata(serde_json::json!({
                "severity": &incident.severity,
                "incident_type": &incident.incident_type,
                "affected_subjects": incident.affected_subjects_count
            }));

            let audit_logger = data.audit_logger.clone();
            tokio::spawn(async move {
                audit_logger.log(&audit_entry).await;
            });

            HttpResponse::Created().json(to_dto(incident))
        }
        Err(e) => HttpResponse::BadRequest().json(serde_json::json!({ "error": e })),
    }
}

/// GET /api/v1/admin/security-incidents — SuperAdmin only
#[get("/admin/security-incidents")]
pub async fn list_security_incidents(
    data: web::Data<AppState>,
    auth: AuthenticatedUser,
    query: web::Query<SecurityIncidentsQuery>,
) -> impl Responder {
    if auth.role != "superadmin" {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Access denied. SuperAdmin role required."
        }));
    }

    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(20).clamp(1, 100);

    match data
        .security_incident_use_cases
        .find_all(
            auth.organization_id,
            query.severity.clone(),
            query.status.clone(),
            page,
            per_page,
        )
        .await
    {
        Ok((incidents, total)) => HttpResponse::Ok().json(SecurityIncidentsResponse {
            incidents: incidents.into_iter().map(to_dto).collect(),
            total,
        }),
        Err(e) => HttpResponse::InternalServerError()
            .json(serde_json::json!({ "error": format!("Failed to fetch incidents: {}", e) })),
    }
}

/// GET /api/v1/admin/security-incidents/overdue — SuperAdmin only
#[get("/admin/security-incidents/overdue")]
pub async fn list_overdue_incidents(
    data: web::Data<AppState>,
    auth: AuthenticatedUser,
) -> impl Responder {
    if auth.role != "superadmin" {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Access denied. SuperAdmin role required."
        }));
    }

    match data
        .security_incident_use_cases
        .find_overdue(auth.organization_id)
        .await
    {
        Ok(incidents) => {
            let total = incidents.len() as i64;
            HttpResponse::Ok().json(SecurityIncidentsResponse {
                incidents: incidents.into_iter().map(to_dto).collect(),
                total,
            })
        }
        Err(e) => HttpResponse::InternalServerError().json(
            serde_json::json!({ "error": format!("Failed to fetch overdue incidents: {}", e) }),
        ),
    }
}

/// GET /api/v1/admin/security-incidents/:id — SuperAdmin only
#[get("/admin/security-incidents/{incident_id}")]
pub async fn get_security_incident(
    data: web::Data<AppState>,
    auth: AuthenticatedUser,
    path: web::Path<Uuid>,
) -> impl Responder {
    if auth.role != "superadmin" {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Access denied. SuperAdmin role required."
        }));
    }

    let incident_id = path.into_inner();
    match data
        .security_incident_use_cases
        .find_by_id(incident_id, auth.organization_id)
        .await
    {
        Ok(Some(incident)) => HttpResponse::Ok().json(to_dto(incident)),
        Ok(None) => HttpResponse::NotFound()
            .json(serde_json::json!({ "error": "Security incident not found" })),
        Err(e) => HttpResponse::InternalServerError()
            .json(serde_json::json!({ "error": format!("Failed to fetch incident: {}", e) })),
    }
}

/// PUT /api/v1/admin/security-incidents/:id/report-apd — SuperAdmin only
#[put("/admin/security-incidents/{incident_id}/report-apd")]
pub async fn report_incident_to_apd(
    data: web::Data<AppState>,
    auth: AuthenticatedUser,
    path: web::Path<Uuid>,
    body: web::Json<ReportToApdRequest>,
) -> impl Responder {
    if auth.role != "superadmin" {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Access denied. SuperAdmin role required."
        }));
    }

    let incident_id = path.into_inner();
    let apd_ref = body.apd_reference_number.clone().unwrap_or_default();

    match data
        .security_incident_use_cases
        .report_to_apd(
            incident_id,
            auth.organization_id,
            apd_ref,
            body.investigation_notes.clone(),
        )
        .await
    {
        Ok(Some(incident)) => HttpResponse::Ok().json(to_dto(incident)),
        Ok(None) => HttpResponse::NotFound()
            .json(serde_json::json!({ "error": "Security incident not found" })),
        Err(e) if e == "already_reported" => HttpResponse::Conflict()
            .json(serde_json::json!({ "error": "Incident already reported to APD" })),
        Err(e) => HttpResponse::BadRequest().json(serde_json::json!({ "error": e })),
    }
}
