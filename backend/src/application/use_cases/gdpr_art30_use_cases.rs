use crate::application::ports::gdpr_art30_repository::GdprArt30Repository;
use crate::domain::entities::gdpr_art30::{ProcessingActivity, ProcessorAgreement};
use std::sync::Arc;

pub struct GdprArt30UseCases {
    repository: Arc<dyn GdprArt30Repository>,
}

impl GdprArt30UseCases {
    pub fn new(repository: Arc<dyn GdprArt30Repository>) -> Self {
        Self { repository }
    }

    pub async fn list_processing_activities(&self) -> Result<Vec<ProcessingActivity>, String> {
        self.repository.list_processing_activities().await
    }

    pub async fn list_processor_agreements(&self) -> Result<Vec<ProcessorAgreement>, String> {
        self.repository.list_processor_agreements().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use chrono::Utc;
    use uuid::Uuid;

    struct MockRepo;

    #[async_trait]
    impl GdprArt30Repository for MockRepo {
        async fn list_processing_activities(&self) -> Result<Vec<ProcessingActivity>, String> {
            let now = Utc::now();
            Ok(vec![ProcessingActivity {
                id: Uuid::new_v4(),
                activity_name: "User management".to_string(),
                controller_name: "KoproGo".to_string(),
                purpose: "Contract performance".to_string(),
                legal_basis: "Art. 6(1)(b)".to_string(),
                data_categories: vec!["contact".to_string()],
                data_subjects: vec!["owners".to_string()],
                recipients: vec![],
                retention_period: "7 years".to_string(),
                security_measures: "Encryption".to_string(),
                created_at: now,
                updated_at: now,
            }])
        }

        async fn list_processor_agreements(&self) -> Result<Vec<ProcessorAgreement>, String> {
            Ok(vec![])
        }
    }

    #[tokio::test]
    async fn test_list_activities_returns_results() {
        let uc = GdprArt30UseCases::new(Arc::new(MockRepo));
        let activities = uc.list_processing_activities().await.unwrap();
        assert_eq!(activities.len(), 1);
        assert_eq!(activities[0].activity_name, "User management");
    }

    #[tokio::test]
    async fn test_list_processors_empty() {
        let uc = GdprArt30UseCases::new(Arc::new(MockRepo));
        let processors = uc.list_processor_agreements().await.unwrap();
        assert!(processors.is_empty());
    }
}
