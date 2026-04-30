use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Request to record user consent (GDPR Art. 7)
#[derive(Debug, Deserialize)]
pub struct RecordConsentRequest {
    pub consent_type: String,
    pub policy_version: Option<String>,
}

/// Response after recording consent
#[derive(Debug, Serialize)]
pub struct ConsentRecordedResponse {
    pub message: String,
    pub consent_type: String,
    pub accepted_at: DateTime<Utc>,
    pub policy_version: String,
}

/// Response for consent status query
#[derive(Debug, Serialize)]
pub struct ConsentStatusResponse {
    pub privacy_policy_accepted: bool,
    pub terms_accepted: bool,
    pub privacy_policy_accepted_at: Option<DateTime<Utc>>,
    pub terms_accepted_at: Option<DateTime<Utc>>,
    pub privacy_policy_version: Option<String>,
    pub terms_version: Option<String>,
}
