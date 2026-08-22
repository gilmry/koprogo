use crate::application::dto::{
    AdminDashboardStats, SeedDataStats, SyndicDashboardStats, UrgentTask,
};
use crate::application::error::AppError;
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait StatsRepository: Send + Sync {
    async fn get_admin_dashboard_stats(&self) -> Result<AdminDashboardStats, AppError>;

    async fn get_seed_data_stats(&self) -> Result<SeedDataStats, AppError>;

    /// Returns stats for all buildings in the given organization.
    async fn get_syndic_stats(
        &self,
        organization_id: Uuid,
    ) -> Result<SyndicDashboardStats, AppError>;

    /// Returns stats for buildings where the given owner has active units.
    async fn get_owner_stats(&self, owner_id: Uuid) -> Result<SyndicDashboardStats, AppError>;

    /// Looks up the owner record id associated with a user id.
    async fn find_owner_id_by_user_id(&self, user_id: Uuid) -> Result<Option<Uuid>, AppError>;

    /// Returns urgent tasks (overdue expenses, upcoming meetings, old pending charges)
    /// for the given organization.
    async fn get_syndic_urgent_tasks(
        &self,
        organization_id: Uuid,
    ) -> Result<Vec<UrgentTask>, AppError>;
}
