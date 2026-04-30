use crate::application::dto::{
    AdminDashboardStats, NextMeetingInfo, SeedDataStats, SyndicDashboardStats, UrgentTask,
};
use crate::application::ports::StatsRepository;
use crate::infrastructure::pool::DbPool;
use async_trait::async_trait;
use chrono::Utc;
use sqlx::Row;
use uuid::Uuid;

pub struct PostgresStatsRepository {
    pool: DbPool,
}

impl PostgresStatsRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl StatsRepository for PostgresStatsRepository {
    async fn get_admin_dashboard_stats(&self) -> Result<AdminDashboardStats, String> {
        let total_organizations =
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM organizations")
                .fetch_one(&self.pool)
                .await
                .map_err(|e| e.to_string())?;

        let total_users = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| e.to_string())?;

        let total_buildings = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM buildings")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| e.to_string())?;

        let active_subscriptions = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM organizations WHERE is_active = true",
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        let total_owners = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM owners")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| e.to_string())?;

        let total_units = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM units")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| e.to_string())?;

        let total_expenses = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM expenses")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| e.to_string())?;

        let total_meetings = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM meetings")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| e.to_string())?;

        Ok(AdminDashboardStats {
            total_organizations,
            total_users,
            total_buildings,
            active_subscriptions,
            total_owners,
            total_units,
            total_expenses,
            total_meetings,
        })
    }

    async fn get_seed_data_stats(&self) -> Result<SeedDataStats, String> {
        let seed_organizations = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM organizations WHERE is_seed_data = true",
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        let production_organizations = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM organizations WHERE is_seed_data = false",
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        let seed_buildings = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM buildings b
             INNER JOIN organizations o ON b.organization_id = o.id
             WHERE o.is_seed_data = true",
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        let seed_units = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM units u
             INNER JOIN buildings b ON u.building_id = b.id
             INNER JOIN organizations o ON b.organization_id = o.id
             WHERE o.is_seed_data = true",
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        let seed_owners = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(DISTINCT o.id) FROM owners o
             INNER JOIN unit_owners uo ON o.id = uo.owner_id
             INNER JOIN units u ON uo.unit_id = u.id
             INNER JOIN buildings b ON u.building_id = b.id
             INNER JOIN organizations org ON b.organization_id = org.id
             WHERE org.is_seed_data = true",
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        let seed_unit_owners = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM unit_owners uo
             INNER JOIN units u ON uo.unit_id = u.id
             INNER JOIN buildings b ON u.building_id = b.id
             INNER JOIN organizations o ON b.organization_id = o.id
             WHERE o.is_seed_data = true",
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        let seed_expenses = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM expenses e
             INNER JOIN buildings b ON e.building_id = b.id
             INNER JOIN organizations o ON b.organization_id = o.id
             WHERE o.is_seed_data = true",
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        let seed_meetings = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM meetings m
             INNER JOIN buildings b ON m.building_id = b.id
             INNER JOIN organizations o ON b.organization_id = o.id
             WHERE o.is_seed_data = true",
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        let seed_users = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM users u
             INNER JOIN organizations o ON u.organization_id = o.id
             WHERE o.is_seed_data = true",
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(SeedDataStats {
            seed_organizations,
            production_organizations,
            seed_buildings,
            seed_units,
            seed_owners,
            seed_unit_owners,
            seed_expenses,
            seed_meetings,
            seed_users,
        })
    }

    async fn get_syndic_stats(
        &self,
        organization_id: Uuid,
    ) -> Result<SyndicDashboardStats, String> {
        let total_buildings = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM buildings WHERE organization_id = $1",
        )
        .bind(organization_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        let total_units = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM units u
             INNER JOIN buildings b ON u.building_id = b.id
             WHERE b.organization_id = $1",
        )
        .bind(organization_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        let total_owners = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(DISTINCT o.id) FROM owners o
             INNER JOIN unit_owners uo ON o.id = uo.owner_id
             INNER JOIN units u ON uo.unit_id = u.id
             INNER JOIN buildings b ON u.building_id = b.id
             WHERE b.organization_id = $1 AND uo.end_date IS NULL",
        )
        .bind(organization_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        let row = sqlx::query(
            "SELECT COUNT(*) as count, COALESCE(SUM(amount)::float8, 0::float8) as total
             FROM expenses e
             INNER JOIN buildings b ON e.building_id = b.id
             WHERE b.organization_id = $1 AND e.payment_status = 'pending'",
        )
        .bind(organization_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())?;
        let pending_count: i64 = row.try_get("count").unwrap_or(0);
        let pending_total: f64 = row.try_get("total").unwrap_or(0.0);

        let next_meeting_row = sqlx::query(
            "SELECT m.id, m.scheduled_date, b.name as building_name
             FROM meetings m
             INNER JOIN buildings b ON m.building_id = b.id
             WHERE b.organization_id = $1 AND m.scheduled_date > NOW() AND m.status = 'scheduled'
             ORDER BY m.scheduled_date ASC
             LIMIT 1",
        )
        .bind(organization_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(SyndicDashboardStats {
            total_buildings,
            total_units,
            total_owners,
            pending_expenses_count: pending_count,
            pending_expenses_amount: pending_total,
            next_meeting: next_meeting_row.map(|m| NextMeetingInfo {
                id: m.get::<Uuid, _>("id").to_string(),
                date: m.get("scheduled_date"),
                building_name: m.get("building_name"),
            }),
        })
    }

    async fn get_owner_stats(&self, owner_id: Uuid) -> Result<SyndicDashboardStats, String> {
        let total_buildings = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(DISTINCT b.id) FROM buildings b
             INNER JOIN units u ON b.id = u.building_id
             INNER JOIN unit_owners uo ON u.id = uo.unit_id
             WHERE uo.owner_id = $1 AND uo.end_date IS NULL",
        )
        .bind(owner_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        let total_units = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM unit_owners uo WHERE uo.owner_id = $1 AND uo.end_date IS NULL",
        )
        .bind(owner_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        let total_owners = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(DISTINCT uo2.owner_id) FROM unit_owners uo2
             INNER JOIN units u ON uo2.unit_id = u.id
             WHERE u.building_id IN (
                 SELECT DISTINCT u2.building_id FROM units u2
                 INNER JOIN unit_owners uo ON u2.id = uo.unit_id
                 WHERE uo.owner_id = $1 AND uo.end_date IS NULL
             ) AND uo2.end_date IS NULL",
        )
        .bind(owner_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        let row = sqlx::query(
            "SELECT COUNT(*) as count, COALESCE(SUM(amount)::float8, 0::float8) as total
             FROM expenses e
             WHERE e.building_id IN (
                 SELECT DISTINCT u.building_id FROM units u
                 INNER JOIN unit_owners uo ON u.id = uo.unit_id
                 WHERE uo.owner_id = $1 AND uo.end_date IS NULL
             ) AND e.payment_status = 'pending'",
        )
        .bind(owner_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())?;
        let pending_count: i64 = row.try_get("count").unwrap_or(0);
        let pending_total: f64 = row.try_get("total").unwrap_or(0.0);

        let next_meeting_row = sqlx::query(
            "SELECT m.id, m.scheduled_date, b.name as building_name
             FROM meetings m
             INNER JOIN buildings b ON m.building_id = b.id
             WHERE b.id IN (
                 SELECT DISTINCT u.building_id FROM units u
                 INNER JOIN unit_owners uo ON u.id = uo.unit_id
                 WHERE uo.owner_id = $1 AND uo.end_date IS NULL
             )
             AND m.scheduled_date > NOW() AND m.status = 'scheduled'
             ORDER BY m.scheduled_date ASC
             LIMIT 1",
        )
        .bind(owner_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(SyndicDashboardStats {
            total_buildings,
            total_units,
            total_owners,
            pending_expenses_count: pending_count,
            pending_expenses_amount: pending_total,
            next_meeting: next_meeting_row.map(|m| NextMeetingInfo {
                id: m.get::<Uuid, _>("id").to_string(),
                date: m.get("scheduled_date"),
                building_name: m.get("building_name"),
            }),
        })
    }

    async fn find_owner_id_by_user_id(&self, user_id: Uuid) -> Result<Option<Uuid>, String> {
        let row = sqlx::query("SELECT id FROM owners WHERE user_id = $1")
            .bind(user_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| e.to_string())?;
        Ok(row.map(|r| r.get("id")))
    }

    async fn get_syndic_urgent_tasks(
        &self,
        organization_id: Uuid,
    ) -> Result<Vec<UrgentTask>, String> {
        let mut tasks: Vec<UrgentTask> = Vec::new();

        let overdue_expenses = sqlx::query(
            "SELECT e.id, e.description, e.amount, b.name as building_name, e.expense_date
             FROM expenses e
             INNER JOIN buildings b ON e.building_id = b.id
             WHERE b.organization_id = $1
             AND e.payment_status = 'overdue'
             ORDER BY e.expense_date ASC
             LIMIT 5",
        )
        .bind(organization_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        for expense in overdue_expenses {
            let amount: f64 = expense.get("amount");
            let id: Uuid = expense.get("id");
            tasks.push(UrgentTask {
                task_type: "expense".to_string(),
                title: format!("Charge en retard - {:.2}€", amount),
                description: expense.get("description"),
                priority: "urgent".to_string(),
                building_name: Some(expense.get("building_name")),
                entity_id: Some(id.to_string()),
                due_date: Some(expense.get("expense_date")),
            });
        }

        let upcoming_meetings = sqlx::query(
            "SELECT m.id, m.title, m.scheduled_date, b.name as building_name
             FROM meetings m
             INNER JOIN buildings b ON m.building_id = b.id
             WHERE b.organization_id = $1
             AND m.status = 'scheduled'
             AND m.scheduled_date BETWEEN NOW() AND NOW() + INTERVAL '7 days'
             ORDER BY m.scheduled_date ASC
             LIMIT 3",
        )
        .bind(organization_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        for meeting in upcoming_meetings {
            let scheduled_date: chrono::DateTime<Utc> = meeting.get("scheduled_date");
            let days_until = (scheduled_date - Utc::now()).num_days();
            let priority = if days_until <= 3 { "urgent" } else { "high" };
            let id: Uuid = meeting.get("id");
            tasks.push(UrgentTask {
                task_type: "meeting".to_string(),
                title: meeting.get("title"),
                description: format!("AG dans {} jours", days_until),
                priority: priority.to_string(),
                building_name: Some(meeting.get("building_name")),
                entity_id: Some(id.to_string()),
                due_date: Some(scheduled_date),
            });
        }

        let pending_overdue_count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*)
             FROM expenses e
             INNER JOIN buildings b ON e.building_id = b.id
             WHERE b.organization_id = $1
             AND e.payment_status = 'pending'
             AND e.expense_date < NOW() - INTERVAL '30 days'",
        )
        .bind(organization_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        if pending_overdue_count > 0 {
            tasks.push(UrgentTask {
                task_type: "expense".to_string(),
                title: "Relance paiements".to_string(),
                description: format!(
                    "{} charges en attente depuis plus de 30 jours",
                    pending_overdue_count
                ),
                priority: "high".to_string(),
                building_name: None,
                entity_id: None,
                due_date: None,
            });
        }

        tasks.sort_by(|a, b| {
            let priority_order = |p: &str| match p {
                "urgent" => 0,
                "high" => 1,
                _ => 2,
            };
            priority_order(&a.priority).cmp(&priority_order(&b.priority))
        });

        Ok(tasks)
    }
}
