use crate::application::dto::consent_dto::{ConsentRecordedResponse, ConsentStatusResponse};
use crate::application::ports::consent_repository::ConsentRepository;
use crate::domain::entities::consent::ConsentRecord;
use crate::infrastructure::audit::{AuditEventType, AuditLogEntry};
use crate::application::ports::audit_log_repository::AuditLogRepository;
use std::sync::Arc;
use uuid::Uuid;

pub struct ConsentUseCases {
    consent_repository: Arc<dyn ConsentRepository>,
    audit_repository: Arc<dyn AuditLogRepository>,
}

impl ConsentUseCases {
    pub fn new(
        consent_repository: Arc<dyn ConsentRepository>,
        audit_repository: Arc<dyn AuditLogRepository>,
    ) -> Self {
        Self {
            consent_repository,
            audit_repository,
        }
    }

    /// Record user consent (GDPR Art. 7)
    ///
    /// Creates an immutable consent record with full audit trail.
    pub async fn record_consent(
        &self,
        user_id: Uuid,
        organization_id: Uuid,
        consent_type: &str,
        ip_address: Option<String>,
        user_agent: Option<String>,
        policy_version: Option<String>,
    ) -> Result<ConsentRecordedResponse, String> {
        // Create domain entity (validates consent_type)
        let record = ConsentRecord::new(
            user_id,
            organization_id,
            consent_type,
            ip_address.clone(),
            user_agent.clone(),
            policy_version,
        )?;

        // Persist
        let saved = self.consent_repository.create(&record).await?;

        // Async audit log
        let audit_entry = AuditLogEntry::new(
            AuditEventType::ConsentRecorded,
            Some(user_id),
            Some(organization_id),
        )
        .with_resource("ConsentRecord", saved.id)
        .with_client_info(ip_address, user_agent)
        .with_metadata(serde_json::json!({
            "consent_type": saved.consent_type,
            "policy_version": saved.policy_version,
        }));

        let audit_repo = self.audit_repository.clone();
        tokio::spawn(async move {
            let _ = audit_repo.create(&audit_entry).await;
        });

        Ok(ConsentRecordedResponse {
            message: format!("Consent for {} recorded successfully", saved.consent_type),
            consent_type: saved.consent_type,
            accepted_at: saved.accepted_at,
            policy_version: saved.policy_version,
        })
    }

    /// Get consent status for a user
    ///
    /// Returns summary of privacy policy and terms acceptance.
    pub async fn get_consent_status(
        &self,
        user_id: Uuid,
    ) -> Result<ConsentStatusResponse, String> {
        let status = self.consent_repository.get_consent_status(user_id).await?;

        Ok(ConsentStatusResponse {
            privacy_policy_accepted: status.privacy_policy_accepted,
            terms_accepted: status.terms_accepted,
            privacy_policy_accepted_at: status.privacy_policy_accepted_at,
            terms_accepted_at: status.terms_accepted_at,
            privacy_policy_version: status.privacy_policy_version,
            terms_version: status.terms_version,
        })
    }

    /// Check if user has accepted a specific consent type
    pub async fn has_accepted(
        &self,
        user_id: Uuid,
        consent_type: &str,
    ) -> Result<bool, String> {
        self.consent_repository
            .has_accepted(user_id, consent_type)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use crate::domain::entities::consent::ConsentStatus;
    use crate::application::dto::PageRequest;
    use crate::application::ports::audit_log_repository::AuditLogFilters;
    use std::sync::Mutex;

    // Mock ConsentRepository
    struct MockConsentRepository {
        records: Mutex<Vec<ConsentRecord>>,
    }

    impl MockConsentRepository {
        fn new() -> Self {
            Self {
                records: Mutex::new(Vec::new()),
            }
        }
    }

    #[async_trait]
    impl ConsentRepository for MockConsentRepository {
        async fn create(&self, record: &ConsentRecord) -> Result<ConsentRecord, String> {
            let mut records = self.records.lock().unwrap();
            records.push(record.clone());
            Ok(record.clone())
        }

        async fn find_latest_by_user_and_type(
            &self,
            user_id: Uuid,
            consent_type: &str,
        ) -> Result<Option<ConsentRecord>, String> {
            let records = self.records.lock().unwrap();
            Ok(records
                .iter()
                .filter(|r| r.user_id == user_id && r.consent_type == consent_type)
                .last()
                .cloned())
        }

        async fn find_all_by_user(&self, user_id: Uuid) -> Result<Vec<ConsentRecord>, String> {
            let records = self.records.lock().unwrap();
            Ok(records
                .iter()
                .filter(|r| r.user_id == user_id)
                .cloned()
                .collect())
        }

        async fn has_accepted(&self, user_id: Uuid, consent_type: &str) -> Result<bool, String> {
            let records = self.records.lock().unwrap();
            Ok(records
                .iter()
                .any(|r| r.user_id == user_id && r.consent_type == consent_type))
        }

        async fn get_consent_status(&self, user_id: Uuid) -> Result<ConsentStatus, String> {
            let records = self.records.lock().unwrap();
            let privacy = records
                .iter()
                .filter(|r| r.user_id == user_id && r.consent_type == "privacy_policy")
                .last()
                .cloned();
            let terms = records
                .iter()
                .filter(|r| r.user_id == user_id && r.consent_type == "terms")
                .last()
                .cloned();

            Ok(ConsentStatus {
                privacy_policy_accepted: privacy.is_some(),
                terms_accepted: terms.is_some(),
                privacy_policy_accepted_at: privacy.as_ref().map(|r| r.accepted_at),
                terms_accepted_at: terms.as_ref().map(|r| r.accepted_at),
                privacy_policy_version: privacy.map(|r| r.policy_version),
                terms_version: terms.map(|r| r.policy_version),
            })
        }
    }

    // Mock AuditLogRepository
    struct MockAuditLogRepository;

    #[async_trait]
    impl AuditLogRepository for MockAuditLogRepository {
        async fn create(&self, _entry: &AuditLogEntry) -> Result<AuditLogEntry, String> {
            Ok(AuditLogEntry::new(AuditEventType::ConsentRecorded, None, None))
        }
        async fn find_by_id(&self, _id: Uuid) -> Result<Option<AuditLogEntry>, String> {
            Ok(None)
        }
        async fn find_all_paginated(
            &self,
            _page_request: &PageRequest,
            _filters: &AuditLogFilters,
        ) -> Result<(Vec<AuditLogEntry>, i64), String> {
            Ok((vec![], 0))
        }
        async fn find_recent(&self, _limit: i64) -> Result<Vec<AuditLogEntry>, String> {
            Ok(vec![])
        }
        async fn find_failed_operations(
            &self,
            _page_request: &PageRequest,
            _organization_id: Option<Uuid>,
        ) -> Result<(Vec<AuditLogEntry>, i64), String> {
            Ok((vec![], 0))
        }
        async fn delete_older_than(&self, _timestamp: chrono::DateTime<chrono::Utc>) -> Result<i64, String> {
            Ok(0)
        }
        async fn count_by_filters(&self, _filters: &AuditLogFilters) -> Result<i64, String> {
            Ok(0)
        }
    }

    fn make_use_cases() -> ConsentUseCases {
        ConsentUseCases::new(
            Arc::new(MockConsentRepository::new()),
            Arc::new(MockAuditLogRepository),
        )
    }

    #[tokio::test]
    async fn test_record_privacy_policy_consent() {
        let uc = make_use_cases();
        let user_id = Uuid::new_v4();
        let org_id = Uuid::new_v4();

        let result = uc
            .record_consent(user_id, org_id, "privacy_policy", None, None, Some("1.0".to_string()))
            .await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.consent_type, "privacy_policy");
        assert_eq!(response.policy_version, "1.0");
        assert!(response.message.contains("privacy_policy"));
    }

    #[tokio::test]
    async fn test_record_terms_consent() {
        let uc = make_use_cases();
        let result = uc
            .record_consent(Uuid::new_v4(), Uuid::new_v4(), "terms", None, None, None)
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap().consent_type, "terms");
    }

    #[tokio::test]
    async fn test_record_invalid_consent_type() {
        let uc = make_use_cases();
        let result = uc
            .record_consent(Uuid::new_v4(), Uuid::new_v4(), "invalid", None, None, None)
            .await;

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid consent type"));
    }

    #[tokio::test]
    async fn test_get_consent_status_empty() {
        let uc = make_use_cases();
        let result = uc.get_consent_status(Uuid::new_v4()).await;

        assert!(result.is_ok());
        let status = result.unwrap();
        assert!(!status.privacy_policy_accepted);
        assert!(!status.terms_accepted);
    }

    #[tokio::test]
    async fn test_get_consent_status_after_recording() {
        let uc = make_use_cases();
        let user_id = Uuid::new_v4();
        let org_id = Uuid::new_v4();

        uc.record_consent(user_id, org_id, "privacy_policy", None, None, None)
            .await
            .unwrap();

        let status = uc.get_consent_status(user_id).await.unwrap();
        assert!(status.privacy_policy_accepted);
        assert!(!status.terms_accepted);
    }

    #[tokio::test]
    async fn test_has_accepted() {
        let uc = make_use_cases();
        let user_id = Uuid::new_v4();
        let org_id = Uuid::new_v4();

        assert!(!uc.has_accepted(user_id, "terms").await.unwrap());

        uc.record_consent(user_id, org_id, "terms", None, None, None)
            .await
            .unwrap();

        assert!(uc.has_accepted(user_id, "terms").await.unwrap());
    }
}
