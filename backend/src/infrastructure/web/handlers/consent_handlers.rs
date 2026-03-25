use crate::infrastructure::web::{AppState, AuthenticatedUser};
use actix_web::{get, post, web, HttpRequest, HttpResponse, Responder};
use serde::{Deserialize, Serialize};

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

/// Request body for recording user consent
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct RecordConsentRequest {
    /// Type of consent: 'privacy_policy' or 'terms'
    pub consent_type: String,
    /// Optional policy version (e.g., "1.0", "1.1")
    #[serde(default)]
    pub policy_version: Option<String>,
}

/// Response for consent status check
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ConsentStatusResponse {
    /// Whether the user has given consent to privacy policy
    pub privacy_policy_accepted: bool,
    /// Whether the user has given consent to terms
    pub terms_accepted: bool,
    /// Timestamp of latest privacy policy consent (if given)
    pub privacy_policy_accepted_at: Option<String>,
    /// Timestamp of latest terms consent (if given)
    pub terms_accepted_at: Option<String>,
    /// User ID
    pub user_id: String,
}

/// Response for successful consent recording
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ConsentRecordedResponse {
    /// Success message
    pub message: String,
    /// Consent type that was recorded
    pub consent_type: String,
    /// Timestamp when consent was recorded
    pub accepted_at: String,
}

/// POST /api/v1/consent
/// Record user consent to privacy policy or terms of service
///
/// Requires JWT authentication. Records the consent in the database with
/// audit trail (IP address, user agent, timestamp) for GDPR Art. 7 / Art. 13-14 compliance.
///
/// # Parameters
/// * `consent_type` - Type of consent: "privacy_policy" or "terms"
/// * `policy_version` - Optional version of the policy (e.g., "1.0")
///
/// # Returns
/// * `200 OK` - Consent recorded successfully
/// * `400 Bad Request` - Invalid consent_type or request body
/// * `401 Unauthorized` - Missing or invalid authentication
/// * `500 Internal Server Error` - Database error
#[utoipa::path(
    post,
    path = "/consent",
    tag = "GDPR",
    summary = "Record user consent to privacy policy or terms",
    request_body = RecordConsentRequest,
    responses(
        (status = 200, description = "Consent recorded", body = ConsentRecordedResponse),
        (status = 400, description = "Bad request"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error"),
    ),
    security(("bearer_auth" = []))
)]
#[post("/consent")]
pub async fn record_consent(
    req: HttpRequest,
    data: web::Data<AppState>,
    auth: AuthenticatedUser,
    body: web::Json<RecordConsentRequest>,
) -> impl Responder {
    // Extract client information for audit trail
    let ip_address = extract_ip_address(&req);
    let user_agent = extract_user_agent(&req);

    // Get organization_id (required for consent records)
    let organization_id = match auth.require_organization() {
        Ok(org_id) => org_id,
        Err(_) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Organization context required to record consent"
            }));
        }
    };

    // Call use case (validates consent_type, persists, creates audit trail)
    match data
        .consent_use_cases
        .record_consent(
            auth.user_id,
            organization_id,
            &body.consent_type,
            ip_address,
            user_agent,
            body.policy_version.clone(),
        )
        .await
    {
        Ok(response) => HttpResponse::Ok().json(ConsentRecordedResponse {
            message: response.message,
            consent_type: response.consent_type,
            accepted_at: response.accepted_at.to_rfc3339(),
        }),
        Err(e) if e.contains("Invalid consent type") => {
            HttpResponse::BadRequest().json(serde_json::json!({ "error": e }))
        }
        Err(e) => {
            log::error!("Failed to record consent: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({ "error": e }))
        }
    }
}

/// GET /api/v1/consent/status
/// Check current consent status for the authenticated user
///
/// Returns which types of consent the user has given (privacy policy and/or terms).
/// Useful for conditional display of consent modals in the frontend.
///
/// # Returns
/// * `200 OK` - Consent status returned
/// * `401 Unauthorized` - Missing or invalid authentication
/// * `500 Internal Server Error` - Database error
#[utoipa::path(
    get,
    path = "/consent/status",
    tag = "GDPR",
    summary = "Check user consent status",
    responses(
        (status = 200, description = "Consent status", body = ConsentStatusResponse),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error"),
    ),
    security(("bearer_auth" = []))
)]
#[get("/consent/status")]
pub async fn get_consent_status(
    _req: HttpRequest,
    data: web::Data<AppState>,
    auth: AuthenticatedUser,
) -> impl Responder {
    match data
        .consent_use_cases
        .get_consent_status(auth.user_id)
        .await
    {
        Ok(status) => HttpResponse::Ok().json(ConsentStatusResponse {
            privacy_policy_accepted: status.privacy_policy_accepted,
            terms_accepted: status.terms_accepted,
            privacy_policy_accepted_at: status.privacy_policy_accepted_at.map(|t| t.to_rfc3339()),
            terms_accepted_at: status.terms_accepted_at.map(|t| t.to_rfc3339()),
            user_id: auth.user_id.to_string(),
        }),
        Err(e) => {
            log::error!("Failed to get consent status: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({ "error": e }))
        }
    }
}
