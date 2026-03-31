use crate::application::dto::PageRequest;
use crate::application::ports::{AuditLogFilters, AuditLogRepository};
use crate::infrastructure::audit::AuditLogEntry;
use std::sync::Arc;

pub struct AuditLogUseCases {
    repo: Arc<dyn AuditLogRepository>,
}

impl AuditLogUseCases {
    pub fn new(repo: Arc<dyn AuditLogRepository>) -> Self {
        Self { repo }
    }

    pub async fn find_all_paginated(
        &self,
        page_request: &PageRequest,
        filters: &AuditLogFilters,
    ) -> Result<(Vec<AuditLogEntry>, i64), String> {
        self.repo.find_all_paginated(page_request, filters).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::audit::{AuditEventType, AuditLogEntry};
    use async_trait::async_trait;
    use uuid::Uuid;

    struct MockAuditLogRepository {
        entries: Vec<AuditLogEntry>,
    }

    #[async_trait]
    impl AuditLogRepository for MockAuditLogRepository {
        async fn create(&self, entry: &AuditLogEntry) -> Result<AuditLogEntry, String> {
            Ok(entry.clone())
        }
        async fn find_by_id(&self, _id: Uuid) -> Result<Option<AuditLogEntry>, String> {
            Ok(None)
        }
        async fn find_all_paginated(
            &self,
            _page_request: &PageRequest,
            _filters: &AuditLogFilters,
        ) -> Result<(Vec<AuditLogEntry>, i64), String> {
            let total = self.entries.len() as i64;
            Ok((self.entries.clone(), total))
        }
        async fn find_recent(&self, _limit: i64) -> Result<Vec<AuditLogEntry>, String> {
            Ok(self.entries.clone())
        }
        async fn find_failed_operations(
            &self,
            _page_request: &PageRequest,
            _organization_id: Option<Uuid>,
        ) -> Result<(Vec<AuditLogEntry>, i64), String> {
            Ok((vec![], 0))
        }
        async fn delete_older_than(
            &self,
            _timestamp: chrono::DateTime<chrono::Utc>,
        ) -> Result<i64, String> {
            Ok(0)
        }
        async fn count_by_filters(&self, _filters: &AuditLogFilters) -> Result<i64, String> {
            Ok(self.entries.len() as i64)
        }
    }

    fn make_entry() -> AuditLogEntry {
        AuditLogEntry::new(AuditEventType::UserLogin, Some(Uuid::new_v4()), None)
    }

    #[tokio::test]
    async fn test_find_all_paginated_returns_entries() {
        let entry = make_entry();
        let repo = Arc::new(MockAuditLogRepository {
            entries: vec![entry],
        });
        let use_cases = AuditLogUseCases::new(repo);
        let page_request = PageRequest {
            page: 1,
            per_page: 20,
            sort_by: None,
            order: crate::application::dto::SortOrder::Desc,
        };
        let filters = AuditLogFilters::default();
        let (entries, total) = use_cases
            .find_all_paginated(&page_request, &filters)
            .await
            .unwrap();
        assert_eq!(total, 1);
        assert_eq!(entries.len(), 1);
    }

    #[tokio::test]
    async fn test_find_all_paginated_empty_returns_zero() {
        let repo = Arc::new(MockAuditLogRepository { entries: vec![] });
        let use_cases = AuditLogUseCases::new(repo);
        let page_request = PageRequest {
            page: 1,
            per_page: 20,
            sort_by: None,
            order: crate::application::dto::SortOrder::Desc,
        };
        let filters = AuditLogFilters::default();
        let (entries, total) = use_cases
            .find_all_paginated(&page_request, &filters)
            .await
            .unwrap();
        assert_eq!(total, 0);
        assert!(entries.is_empty());
    }
}
