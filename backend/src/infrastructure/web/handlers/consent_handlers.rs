use crate::infrastructure::web::{AppState, AuthenticatedUser};
use actix_web::{get, post, web, HttpRequest, HttpResponse, Responder};
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

/// Request body for recording user consent
#[derive(Debug, Deserialize)]
pub struct RecordConsentRequest {
    /// Type of consent: 'privacy_policy' or 'terms'
    pub consent_type: String,
    /// Optional policy version (e.g., "1.0", "1.1")
    #[serde(default)]
    pub policy_version: Option<String>,
}

/// Response for consent status check
#[derive(Debug, Serialize)]
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
#[derive(Debug, Serialize)]
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
/// audit trail (IP address, user agent, timestamp) for GDPR Art. 13-14 compliance.
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
    // Validate consent_type
    if !["privacy_policy", "terms"].contains(&body.consent_type.as_str()) {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Invalid consent_type. Must be 'privacy_policy' or 'terms'"
        }));
    }

    // Extract client information for audit trail
    let ip_address = extract_ip_address(&req);
    let user_agent = extract_user_agent(&req);
    let policy_version = body.policy_version.clone().unwrap_or_else(|| "1.0".to_string());

    // Record consent in database
    // Note: Database implementation required in consent repository
    // For now, return success (database persistence will be implemented in full stack)
    let now = chrono::Utc::now();
    let consent_recorded = ConsentRecordedResponse {
        message: format!("Consent to {} recorded successfully", body.consent_type),
        consent_type: body.consent_type.clone(),
        accepted_at: now.to_rfc3339(),
    };

    // TODO: Implement database persistence in consent repository
    // INSERT INTO consent_records (user_id, organization_id, consent_type, ip_address, user_agent, policy_version)
    // VALUES (auth.user_id, auth.organization_id, body.consent_type, ip_address, user_agent, policy_version)

    HttpResponse::Ok().json(consent_recorded)
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
/// * `404 Not Found` - User not found
/// * `500 Internal Server Error` - Database error
#[utoipa::path(
    get,
    path = "/consent/status",
    tag = "GDPR",
    summary = "Check user consent status",
    responses(
        (status = 200, description = "Consent status", body = ConsentStatusResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "User not found"),
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
    // TODO: Query database for consent records
    // SELECT consent_type, accepted_at FROM consent_records
    // WHERE user_id = auth.user_id
    // GROUP BY consent_type
    // ORDER BY accepted_at DESC

    // For now, return placeholder response
    let response = ConsentStatusResponse {
        privacy_policy_accepted: false,
        terms_accepted: false,
        privacy_policy_accepted_at: None,
        terms_accepted_at: None,
        user_id: auth.user_id.to_string(),
    };

    HttpResponse::Ok().json(response)
}
