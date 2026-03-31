use crate::application::ports::security_incident_repository::{
    SecurityIncidentFilters, SecurityIncidentRepository,
};
use crate::domain::entities::SecurityIncident;
use std::sync::Arc;
use uuid::Uuid;

pub struct SecurityIncidentUseCases {
    repository: Arc<dyn SecurityIncidentRepository>,
}

impl SecurityIncidentUseCases {
    pub fn new(repository: Arc<dyn SecurityIncidentRepository>) -> Self {
        Self { repository }
    }

    pub async fn create(
        &self,
        organization_id: Option<Uuid>,
        reported_by: Uuid,
        severity: String,
        incident_type: String,
        title: String,
        description: String,
        data_categories_affected: Vec<String>,
        affected_subjects_count: Option<i32>,
    ) -> Result<SecurityIncident, String> {
        let incident = SecurityIncident::new(
            organization_id,
            reported_by,
            severity,
            incident_type,
            title,
            description,
            data_categories_affected,
            affected_subjects_count,
        )?;
        self.repository.create(&incident).await
    }

    pub async fn find_all(
        &self,
        organization_id: Option<Uuid>,
        severity: Option<String>,
        status: Option<String>,
        page: i64,
        per_page: i64,
    ) -> Result<(Vec<SecurityIncident>, i64), String> {
        let filters = SecurityIncidentFilters {
            severity,
            status,
            page,
            per_page,
        };
        self.repository.find_all(organization_id, filters).await
    }

    pub async fn find_by_id(
        &self,
        id: Uuid,
        organization_id: Option<Uuid>,
    ) -> Result<Option<SecurityIncident>, String> {
        self.repository.find_by_id(id, organization_id).await
    }

    pub async fn report_to_apd(
        &self,
        id: Uuid,
        organization_id: Option<Uuid>,
        apd_reference_number: String,
        investigation_notes: Option<String>,
    ) -> Result<Option<SecurityIncident>, String> {
        if apd_reference_number.is_empty() {
            return Err("apd_reference_number is required".to_string());
        }
        // Check if already reported
        match self.repository.find_by_id(id, organization_id).await? {
            None => Ok(None),
            Some(incident) if incident.notification_at.is_some() => {
                Err("already_reported".to_string())
            }
            Some(_) => {
                self.repository
                    .report_to_apd(id, organization_id, apd_reference_number, investigation_notes)
                    .await
            }
        }
    }

    pub async fn find_overdue(
        &self,
        organization_id: Option<Uuid>,
    ) -> Result<Vec<SecurityIncident>, String> {
        self.repository.find_overdue(organization_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::ports::security_incident_repository::SecurityIncidentFilters;
    use async_trait::async_trait;

    struct MockRepo {
        should_fail: bool,
    }

    #[async_trait]
    impl SecurityIncidentRepository for MockRepo {
        async fn create(&self, incident: &SecurityIncident) -> Result<SecurityIncident, String> {
            if self.should_fail {
                return Err("db error".to_string());
            }
            Ok(incident.clone())
        }

        async fn find_by_id(
            &self,
            _id: Uuid,
            _org: Option<Uuid>,
        ) -> Result<Option<SecurityIncident>, String> {
            Ok(None)
        }

        async fn find_all(
            &self,
            _org: Option<Uuid>,
            _filters: SecurityIncidentFilters,
        ) -> Result<(Vec<SecurityIncident>, i64), String> {
            Ok((vec![], 0))
        }

        async fn report_to_apd(
            &self,
            _id: Uuid,
            _org: Option<Uuid>,
            _ref: String,
            _notes: Option<String>,
        ) -> Result<Option<SecurityIncident>, String> {
            Ok(None)
        }

        async fn find_overdue(
            &self,
            _org: Option<Uuid>,
        ) -> Result<Vec<SecurityIncident>, String> {
            Ok(vec![])
        }
    }

    #[tokio::test]
    async fn test_create_validates_domain() {
        let uc = SecurityIncidentUseCases::new(Arc::new(MockRepo { should_fail: false }));
        // Empty title rejected by domain entity
        let result = uc
            .create(
                None,
                Uuid::new_v4(),
                "high".to_string(),
                "breach".to_string(),
                "".to_string(),
                "desc".to_string(),
                vec![],
                None,
            )
            .await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("title"));
    }

    #[tokio::test]
    async fn test_create_invalid_severity_rejected() {
        let uc = SecurityIncidentUseCases::new(Arc::new(MockRepo { should_fail: false }));
        let result = uc
            .create(
                None,
                Uuid::new_v4(),
                "extreme".to_string(),
                "breach".to_string(),
                "title".to_string(),
                "desc".to_string(),
                vec![],
                None,
            )
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_create_success() {
        let uc = SecurityIncidentUseCases::new(Arc::new(MockRepo { should_fail: false }));
        let result = uc
            .create(
                Some(Uuid::new_v4()),
                Uuid::new_v4(),
                "critical".to_string(),
                "data_breach".to_string(),
                "Production DB leaked".to_string(),
                "Details here".to_string(),
                vec!["email".to_string(), "address".to_string()],
                Some(500),
            )
            .await;
        assert!(result.is_ok());
        let inc = result.unwrap();
        assert_eq!(inc.status, "detected");
        assert_eq!(inc.severity, "critical");
    }

    #[tokio::test]
    async fn test_report_to_apd_empty_ref_rejected() {
        let uc = SecurityIncidentUseCases::new(Arc::new(MockRepo { should_fail: false }));
        let result = uc
            .report_to_apd(Uuid::new_v4(), None, "".to_string(), None)
            .await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("required"));
    }

    #[tokio::test]
    async fn test_find_all_empty() {
        let uc = SecurityIncidentUseCases::new(Arc::new(MockRepo { should_fail: false }));
        let (incidents, total) = uc.find_all(None, None, None, 1, 20).await.unwrap();
        assert_eq!(total, 0);
        assert!(incidents.is_empty());
    }
}
