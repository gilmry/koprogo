use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Valid consent types matching database CHECK constraint
const VALID_CONSENT_TYPES: [&str; 2] = ["privacy_policy", "terms"];

/// ConsentRecord - GDPR Art. 7 / Art. 13-14 compliance
///
/// Tracks explicit user consent to privacy policy and terms of service.
/// Each record is immutable (append-only) — new consent creates a new record.
/// Audit trail includes IP address, user agent, and policy version.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConsentRecord {
    pub id: Uuid,
    pub user_id: Uuid,
    pub organization_id: Uuid,
    pub consent_type: String,
    pub accepted_at: DateTime<Utc>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub policy_version: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl ConsentRecord {
    /// Create a new consent record with validation
    ///
    /// # Arguments
    /// - `user_id`: The user giving consent
    /// - `organization_id`: The organization context
    /// - `consent_type`: Must be "privacy_policy" or "terms"
    /// - `ip_address`: Optional IP for audit trail
    /// - `user_agent`: Optional browser user-agent for audit trail
    /// - `policy_version`: Version of the policy being accepted (default "1.0")
    pub fn new(
        user_id: Uuid,
        organization_id: Uuid,
        consent_type: &str,
        ip_address: Option<String>,
        user_agent: Option<String>,
        policy_version: Option<String>,
    ) -> Result<Self, String> {
        // Validate consent_type
        if !VALID_CONSENT_TYPES.contains(&consent_type) {
            return Err(format!(
                "Invalid consent type '{}'. Must be one of: {}",
                consent_type,
                VALID_CONSENT_TYPES.join(", ")
            ));
        }

        let version = policy_version.unwrap_or_else(|| "1.0".to_string());
        if version.is_empty() {
            return Err("Policy version cannot be empty".to_string());
        }

        let now = Utc::now();

        Ok(Self {
            id: Uuid::new_v4(),
            user_id,
            organization_id,
            consent_type: consent_type.to_string(),
            accepted_at: now,
            ip_address,
            user_agent,
            policy_version: version,
            created_at: now,
            updated_at: now,
        })
    }

    /// Check if this consent is for privacy policy
    pub fn is_privacy_policy(&self) -> bool {
        self.consent_type == "privacy_policy"
    }

    /// Check if this consent is for terms of service
    pub fn is_terms(&self) -> bool {
        self.consent_type == "terms"
    }
}

/// Consent status summary for a user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsentStatus {
    pub privacy_policy_accepted: bool,
    pub terms_accepted: bool,
    pub privacy_policy_accepted_at: Option<DateTime<Utc>>,
    pub terms_accepted_at: Option<DateTime<Utc>>,
    pub privacy_policy_version: Option<String>,
    pub terms_version: Option<String>,
}

impl Default for ConsentStatus {
    fn default() -> Self {
        Self {
            privacy_policy_accepted: false,
            terms_accepted: false,
            privacy_policy_accepted_at: None,
            terms_accepted_at: None,
            privacy_policy_version: None,
            terms_version: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_privacy_policy_consent() {
        let user_id = Uuid::new_v4();
        let org_id = Uuid::new_v4();
        let record = ConsentRecord::new(
            user_id,
            org_id,
            "privacy_policy",
            Some("192.168.1.1".to_string()),
            Some("Mozilla/5.0".to_string()),
            Some("1.2".to_string()),
        )
        .unwrap();

        assert_eq!(record.user_id, user_id);
        assert_eq!(record.organization_id, org_id);
        assert_eq!(record.consent_type, "privacy_policy");
        assert_eq!(record.policy_version, "1.2");
        assert!(record.is_privacy_policy());
        assert!(!record.is_terms());
        assert_eq!(record.ip_address.as_deref(), Some("192.168.1.1"));
        assert_eq!(record.user_agent.as_deref(), Some("Mozilla/5.0"));
    }

    #[test]
    fn test_new_terms_consent() {
        let record = ConsentRecord::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "terms",
            None,
            None,
            None,
        )
        .unwrap();

        assert_eq!(record.consent_type, "terms");
        assert_eq!(record.policy_version, "1.0"); // default
        assert!(record.is_terms());
        assert!(!record.is_privacy_policy());
    }

    #[test]
    fn test_invalid_consent_type() {
        let result = ConsentRecord::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "invalid_type",
            None,
            None,
            None,
        );
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("Invalid consent type 'invalid_type'"));
    }

    #[test]
    fn test_empty_policy_version() {
        let result = ConsentRecord::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "privacy_policy",
            None,
            None,
            Some("".to_string()),
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Policy version cannot be empty"));
    }

    #[test]
    fn test_consent_generates_unique_ids() {
        let r1 = ConsentRecord::new(Uuid::new_v4(), Uuid::new_v4(), "terms", None, None, None).unwrap();
        let r2 = ConsentRecord::new(Uuid::new_v4(), Uuid::new_v4(), "terms", None, None, None).unwrap();
        assert_ne!(r1.id, r2.id);
    }

    #[test]
    fn test_consent_status_default() {
        let status = ConsentStatus::default();
        assert!(!status.privacy_policy_accepted);
        assert!(!status.terms_accepted);
        assert!(status.privacy_policy_accepted_at.is_none());
        assert!(status.terms_accepted_at.is_none());
    }
}
