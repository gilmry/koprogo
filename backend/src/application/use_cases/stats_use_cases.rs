use crate::application::dto::{
    AdminDashboardStats, SeedDataStats, SyndicDashboardStats, UrgentTask,
};
use crate::application::ports::StatsRepository;
use std::sync::Arc;
use uuid::Uuid;

pub struct StatsUseCases {
    repo: Arc<dyn StatsRepository>,
}

impl StatsUseCases {
    pub fn new(repo: Arc<dyn StatsRepository>) -> Self {
        Self { repo }
    }

    pub async fn get_admin_dashboard_stats(&self) -> Result<AdminDashboardStats, String> {
        self.repo.get_admin_dashboard_stats().await
    }

    pub async fn get_seed_data_stats(&self) -> Result<SeedDataStats, String> {
        self.repo.get_seed_data_stats().await
    }

    pub async fn get_syndic_stats(
        &self,
        organization_id: Uuid,
    ) -> Result<SyndicDashboardStats, String> {
        self.repo.get_syndic_stats(organization_id).await
    }

    /// Returns owner stats. If the user has no owner record returns empty stats.
    pub async fn get_owner_stats_by_user_id(
        &self,
        user_id: Uuid,
    ) -> Result<SyndicDashboardStats, String> {
        match self.repo.find_owner_id_by_user_id(user_id).await? {
            None => Ok(SyndicDashboardStats {
                total_buildings: 0,
                total_units: 0,
                total_owners: 0,
                pending_expenses_count: 0,
                pending_expenses_amount: 0.0,
                next_meeting: None,
            }),
            Some(owner_id) => self.repo.get_owner_stats(owner_id).await,
        }
    }

    pub async fn get_syndic_urgent_tasks(
        &self,
        organization_id: Uuid,
    ) -> Result<Vec<UrgentTask>, String> {
        self.repo.get_syndic_urgent_tasks(organization_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::dto::NextMeetingInfo;
    use async_trait::async_trait;

    struct MockStatsRepository {
        owner_id: Option<Uuid>,
    }

    #[async_trait]
    impl StatsRepository for MockStatsRepository {
        async fn get_admin_dashboard_stats(&self) -> Result<AdminDashboardStats, String> {
            Ok(AdminDashboardStats {
                total_organizations: 5,
                total_users: 50,
                total_buildings: 10,
                active_subscriptions: 4,
                total_owners: 30,
                total_units: 100,
                total_expenses: 200,
                total_meetings: 20,
            })
        }
        async fn get_seed_data_stats(&self) -> Result<SeedDataStats, String> {
            Ok(SeedDataStats {
                seed_organizations: 1,
                production_organizations: 4,
                seed_buildings: 3,
                seed_units: 15,
                seed_owners: 10,
                seed_unit_owners: 15,
                seed_expenses: 20,
                seed_meetings: 5,
                seed_users: 8,
            })
        }
        async fn get_syndic_stats(
            &self,
            _organization_id: Uuid,
        ) -> Result<SyndicDashboardStats, String> {
            Ok(SyndicDashboardStats {
                total_buildings: 2,
                total_units: 10,
                total_owners: 8,
                pending_expenses_count: 3,
                pending_expenses_amount: 1500.0,
                next_meeting: None,
            })
        }
        async fn get_owner_stats(&self, _owner_id: Uuid) -> Result<SyndicDashboardStats, String> {
            Ok(SyndicDashboardStats {
                total_buildings: 1,
                total_units: 2,
                total_owners: 5,
                pending_expenses_count: 1,
                pending_expenses_amount: 500.0,
                next_meeting: None,
            })
        }
        async fn find_owner_id_by_user_id(&self, _user_id: Uuid) -> Result<Option<Uuid>, String> {
            Ok(self.owner_id)
        }
        async fn get_syndic_urgent_tasks(
            &self,
            _organization_id: Uuid,
        ) -> Result<Vec<UrgentTask>, String> {
            Ok(vec![])
        }
    }

    #[tokio::test]
    async fn test_get_admin_dashboard_stats() {
        let repo = Arc::new(MockStatsRepository { owner_id: None });
        let use_cases = StatsUseCases::new(repo);
        let stats = use_cases.get_admin_dashboard_stats().await.unwrap();
        assert_eq!(stats.total_organizations, 5);
        assert_eq!(stats.total_buildings, 10);
    }

    #[tokio::test]
    async fn test_get_owner_stats_no_owner_record_returns_empty() {
        let repo = Arc::new(MockStatsRepository { owner_id: None });
        let use_cases = StatsUseCases::new(repo);
        let stats = use_cases
            .get_owner_stats_by_user_id(Uuid::new_v4())
            .await
            .unwrap();
        assert_eq!(stats.total_buildings, 0);
        assert_eq!(stats.total_units, 0);
        assert!(stats.next_meeting.is_none());
    }

    #[tokio::test]
    async fn test_get_owner_stats_with_owner_record() {
        let owner_id = Uuid::new_v4();
        let repo = Arc::new(MockStatsRepository {
            owner_id: Some(owner_id),
        });
        let use_cases = StatsUseCases::new(repo);
        let stats = use_cases
            .get_owner_stats_by_user_id(Uuid::new_v4())
            .await
            .unwrap();
        assert_eq!(stats.total_buildings, 1);
        assert_eq!(stats.pending_expenses_count, 1);
    }
}
