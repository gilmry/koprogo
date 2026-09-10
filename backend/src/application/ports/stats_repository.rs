use crate::application::dto::{
    AdminDashboardStats, DuAupresDuneAcp, SeedDataStats, SyndicDashboardStats, UrgentTask,
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

    /// Ce que le copropriétaire doit, **ventilé par association**.
    ///
    /// Séparée de `get_owner_stats` parce qu'elle répond à une autre question :
    /// celle-là dit « combien », celle-ci dit « à QUI ». Un copropriétaire peut
    /// détenir des lots dans plusieurs ACP, et chacune est une personne morale
    /// avec son propre compte bancaire (Art. 3.86 § 1er et § 3).
    ///
    /// Un montant global laisse croire qu'un virement unique suffit. Il paierait
    /// la mauvaise personne morale pour une partie de la somme (#867).
    ///
    /// Liste vide si le copropriétaire ne doit rien : c'est une bonne nouvelle,
    /// pas une erreur.
    async fn get_owner_dues_by_acp(&self, owner_id: Uuid)
        -> Result<Vec<DuAupresDuneAcp>, AppError>;

    /// Looks up the owner record id associated with a user id.
    async fn find_owner_id_by_user_id(&self, user_id: Uuid) -> Result<Option<Uuid>, AppError>;

    /// Returns urgent tasks (overdue expenses, upcoming meetings, old pending charges)
    /// for the given organization.
    async fn get_syndic_urgent_tasks(
        &self,
        organization_id: Uuid,
    ) -> Result<Vec<UrgentTask>, AppError>;
}
