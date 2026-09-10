use crate::application::dto::{
    AdminDashboardStats, DuAupresDuneAcp, SeedDataStats, SyndicDashboardStats, UrgentTask,
};
use crate::application::error::AppError;
use crate::application::ports::StatsRepository;
use rust_decimal::Decimal;
use std::sync::Arc;
use uuid::Uuid;

pub struct StatsUseCases {
    repo: Arc<dyn StatsRepository>,
}

impl StatsUseCases {
    pub fn new(repo: Arc<dyn StatsRepository>) -> Self {
        Self { repo }
    }

    pub async fn get_admin_dashboard_stats(&self) -> Result<AdminDashboardStats, AppError> {
        self.repo.get_admin_dashboard_stats().await
    }

    pub async fn get_seed_data_stats(&self) -> Result<SeedDataStats, AppError> {
        self.repo.get_seed_data_stats().await
    }

    pub async fn get_syndic_stats(
        &self,
        organization_id: Uuid,
    ) -> Result<SyndicDashboardStats, AppError> {
        self.repo.get_syndic_stats(organization_id).await
    }

    /// Returns owner stats. If the user has no owner record returns empty stats.
    pub async fn get_owner_stats_by_user_id(
        &self,
        user_id: Uuid,
    ) -> Result<SyndicDashboardStats, AppError> {
        match self.repo.find_owner_id_by_user_id(user_id).await? {
            None => Ok(SyndicDashboardStats {
                total_buildings: 0,
                total_units: 0,
                declared_units: 0,
                total_owners: 0,
                pending_expenses_count: 0,
                pending_expenses_amount: Decimal::ZERO,
                next_meeting: None,
            }),
            Some(owner_id) => self.repo.get_owner_stats(owner_id).await,
        }
    }

    /// Ce que le copropriétaire doit, ventilé par association.
    ///
    /// Liste vide si l'utilisateur n'est rattaché à aucune fiche de
    /// copropriétaire : ce n'est pas une erreur, c'est un compte qui n'a pas
    /// encore de lot. Rendre une erreur ferait afficher une panne là où il n'y
    /// a rien à payer.
    pub async fn get_owner_dues_by_acp(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<DuAupresDuneAcp>, AppError> {
        match self.repo.find_owner_id_by_user_id(user_id).await? {
            None => Ok(Vec::new()),
            Some(owner_id) => self.repo.get_owner_dues_by_acp(owner_id).await,
        }
    }

    pub async fn get_syndic_urgent_tasks(
        &self,
        organization_id: Uuid,
    ) -> Result<Vec<UrgentTask>, AppError> {
        self.repo.get_syndic_urgent_tasks(organization_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use rust_decimal_macros::dec;

    struct MockStatsRepository {
        owner_id: Option<Uuid>,
    }

    #[async_trait]
    impl StatsRepository for MockStatsRepository {
        async fn get_admin_dashboard_stats(&self) -> Result<AdminDashboardStats, AppError> {
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
        async fn get_seed_data_stats(&self) -> Result<SeedDataStats, AppError> {
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
        ) -> Result<SyndicDashboardStats, AppError> {
            Ok(SyndicDashboardStats {
                total_buildings: 2,
                total_units: 10,
                declared_units: 12,
                total_owners: 8,
                pending_expenses_count: 3,
                pending_expenses_amount: dec!(1500.00),
                next_meeting: None,
            })
        }
        async fn get_owner_dues_by_acp(
            &self,
            _owner_id: Uuid,
        ) -> Result<Vec<crate::application::dto::DuAupresDuneAcp>, AppError> {
            // DEUX associations : c'est le cas qui compte. Une doublure à une
            // seule ACP laisserait passer un écran qui additionne les dettes
            // de personnes morales distinctes — le défaut de #867.
            Ok(vec![
                crate::application::dto::DuAupresDuneAcp {
                    acp_id: "acp-1".to_string(),
                    acp_name: "Les Érables".to_string(),
                    bce_number: Some("0123.456.789".to_string()),
                    charges_en_attente: 2,
                    montant: dec!(842.50),
                },
                crate::application::dto::DuAupresDuneAcp {
                    acp_id: "acp-2".to_string(),
                    acp_name: "Les Glycines".to_string(),
                    bce_number: Some("0987.654.321".to_string()),
                    charges_en_attente: 1,
                    montant: dec!(420.00),
                },
            ])
        }

        async fn get_owner_stats(&self, _owner_id: Uuid) -> Result<SyndicDashboardStats, AppError> {
            Ok(SyndicDashboardStats {
                total_buildings: 1,
                total_units: 2,
                declared_units: 2,
                total_owners: 5,
                pending_expenses_count: 1,
                pending_expenses_amount: dec!(500.00),
                next_meeting: None,
            })
        }
        async fn find_owner_id_by_user_id(&self, _user_id: Uuid) -> Result<Option<Uuid>, AppError> {
            Ok(self.owner_id)
        }
        async fn get_syndic_urgent_tasks(
            &self,
            _organization_id: Uuid,
        ) -> Result<Vec<UrgentTask>, AppError> {
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
