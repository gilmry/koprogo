use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct AdminDashboardStats {
    pub total_organizations: i64,
    pub total_users: i64,
    pub total_buildings: i64,
    pub active_subscriptions: i64,
    pub total_owners: i64,
    pub total_units: i64,
    pub total_expenses: i64,
    pub total_meetings: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct SeedDataStats {
    pub seed_organizations: i64,
    pub production_organizations: i64,
    pub seed_buildings: i64,
    pub seed_units: i64,
    pub seed_owners: i64,
    pub seed_unit_owners: i64,
    pub seed_expenses: i64,
    pub seed_meetings: i64,
    pub seed_users: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct NextMeetingInfo {
    pub id: String,
    pub date: DateTime<Utc>,
    pub building_name: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SyndicDashboardStats {
    pub total_buildings: i64,
    pub total_units: i64,
    pub total_owners: i64,
    pub pending_expenses_count: i64,
    /// Total des dépenses en attente, en EUR.
    ///
    /// `Decimal` : la colonne `expenses.amount` est en `NUMERIC(12,2)` depuis
    /// la migration 20260502000000. La lire en `f64` était une dégradation
    /// gratuite d'une valeur déjà exacte (même défaut que celui trouvé dans
    /// `find_overdue_expenses_without_reminders` sous #661).
    ///
    /// `serde::float` conserve la représentation JSON numérique attendue par
    /// `OwnerDashboard.svelte` / `SyndicDashboard.svelte`, qui typent ce champ
    /// `number` — aucun drift de contrat.
    #[serde(with = "rust_decimal::serde::float")]
    pub pending_expenses_amount: Decimal,
    pub next_meeting: Option<NextMeetingInfo>,
}

#[derive(Debug, Clone, Serialize)]
pub struct UrgentTask {
    pub task_type: String,
    pub title: String,
    pub description: String,
    pub priority: String,
    pub building_name: Option<String>,
    pub entity_id: Option<String>,
    pub due_date: Option<DateTime<Utc>>,
}
