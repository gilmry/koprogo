use chrono::{DateTime, Utc};
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
    pub pending_expenses_amount: f64,
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
