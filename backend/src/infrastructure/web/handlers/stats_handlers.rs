use crate::infrastructure::web::{AppState, AuthenticatedUser};
use actix_web::{get, web, HttpResponse, Responder};
use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Serialize)]
pub struct DashboardStats {
    pub total_organizations: i64,
    pub total_users: i64,
    pub total_buildings: i64,
    pub active_subscriptions: i64,
    pub total_owners: i64,
    pub total_units: i64,
    pub total_expenses: i64,
    pub total_meetings: i64,
}

#[derive(Serialize)]
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

#[derive(Serialize)]
pub struct SyndicDashboardStats {
    pub total_buildings: i64,
    pub total_units: i64,
    pub total_owners: i64,
    pub pending_expenses_count: i64,
    pub pending_expenses_amount: f64,
    pub next_meeting: Option<NextMeetingInfo>,
}

#[derive(Serialize)]
pub struct NextMeetingInfo {
    pub id: String,
    pub date: DateTime<Utc>,
    pub building_name: String,
}

#[derive(Serialize)]
pub struct UrgentTask {
    pub task_type: String, // "expense" | "meeting" | "other"
    pub title: String,
    pub description: String,
    pub priority: String, // "urgent" | "high" | "medium"
    pub building_name: Option<String>,
    pub entity_id: Option<String>,
    pub due_date: Option<DateTime<Utc>>,
}

/// Get dashboard statistics (SuperAdmin only)
#[get("/stats/dashboard")]
pub async fn get_dashboard_stats(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
) -> impl Responder {
    // Only SuperAdmin can access these stats
    if user.role != "superadmin" {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Only SuperAdmin can access dashboard statistics"
        }));
    }

    // Count organizations
    let orgs_result = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM organizations")
        .fetch_one(&state.pool)
        .await;

    // Count users
    let users_result = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users")
        .fetch_one(&state.pool)
        .await;

    // Count buildings
    let buildings_result = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM buildings")
        .fetch_one(&state.pool)
        .await;

    // Count active subscriptions (organizations with is_active = true)
    let active_subs_result =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM organizations WHERE is_active = true")
            .fetch_one(&state.pool)
            .await;

    // Count owners
    let owners_result = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM owners")
        .fetch_one(&state.pool)
        .await;

    // Count units
    let units_result = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM units")
        .fetch_one(&state.pool)
        .await;

    // Count expenses
    let expenses_result = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM expenses")
        .fetch_one(&state.pool)
        .await;

    // Count meetings
    let meetings_result = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM meetings")
        .fetch_one(&state.pool)
        .await;

    // Handle errors
    match (
        orgs_result,
        users_result,
        buildings_result,
        active_subs_result,
        owners_result,
        units_result,
        expenses_result,
        meetings_result,
    ) {
        (
            Ok(total_organizations),
            Ok(total_users),
            Ok(total_buildings),
            Ok(active_subscriptions),
            Ok(total_owners),
            Ok(total_units),
            Ok(total_expenses),
            Ok(total_meetings),
        ) => {
            let stats = DashboardStats {
                total_organizations,
                total_users,
                total_buildings,
                active_subscriptions,
                total_owners,
                total_units,
                total_expenses,
                total_meetings,
            };
            HttpResponse::Ok().json(stats)
        }
        _ => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "Failed to fetch dashboard statistics"
        })),
    }
}

/// Get Syndic dashboard statistics (Syndic and Accountant roles)
#[get("/stats/syndic")]
pub async fn get_syndic_stats(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
) -> impl Responder {
    // Only Syndic and Accountant can access these stats
    if user.role != "syndic" && user.role != "accountant" && user.role != "superadmin" {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Only Syndic and Accountant can access these statistics"
        }));
    }

    // Count buildings in user's organization
    let buildings_result =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM buildings WHERE organization_id = $1")
            .bind(user.organization_id)
            .fetch_one(&state.pool)
            .await;

    // Count units in user's organization
    let units_result = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM units u
         INNER JOIN buildings b ON u.building_id = b.id
         WHERE b.organization_id = $1",
    )
    .bind(user.organization_id)
    .fetch_one(&state.pool)
    .await;

    // Count owners in user's organization (via unit_owners)
    let owners_result = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(DISTINCT o.id) FROM owners o
         INNER JOIN unit_owners uo ON o.id = uo.owner_id
         INNER JOIN units u ON uo.unit_id = u.id
         INNER JOIN buildings b ON u.building_id = b.id
         WHERE b.organization_id = $1 AND uo.end_date IS NULL",
    )
    .bind(user.organization_id)
    .fetch_one(&state.pool)
    .await;

    // Count pending expenses and total amount
    let pending_expenses = sqlx::query!(
        "SELECT COUNT(*) as count, COALESCE(SUM(amount), 0) as total
         FROM expenses e
         INNER JOIN buildings b ON e.building_id = b.id
         WHERE b.organization_id = $1 AND e.payment_status = 'pending'",
        user.organization_id
    )
    .fetch_one(&state.pool)
    .await;

    // Get next meeting
    let next_meeting = sqlx::query!(
        "SELECT m.id, m.scheduled_date, b.name as building_name
         FROM meetings m
         INNER JOIN buildings b ON m.building_id = b.id
         WHERE b.organization_id = $1 AND m.scheduled_date > NOW() AND m.status = 'scheduled'
         ORDER BY m.scheduled_date ASC
         LIMIT 1",
        user.organization_id
    )
    .fetch_optional(&state.pool)
    .await;

    match (
        buildings_result,
        units_result,
        owners_result,
        pending_expenses,
        next_meeting,
    ) {
        (
            Ok(total_buildings),
            Ok(total_units),
            Ok(total_owners),
            Ok(expenses_data),
            Ok(meeting_data),
        ) => {
            let next_meeting_info = meeting_data.map(|m| NextMeetingInfo {
                id: m.id.to_string(),
                date: m.scheduled_date,
                building_name: m.building_name,
            });

            let stats = SyndicDashboardStats {
                total_buildings,
                total_units,
                total_owners,
                pending_expenses_count: expenses_data.count.unwrap_or(0),
                pending_expenses_amount: expenses_data.total.unwrap_or(0.0),
                next_meeting: next_meeting_info,
            };
            HttpResponse::Ok().json(stats)
        }
        _ => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "Failed to fetch syndic dashboard statistics"
        })),
    }
}

/// Get urgent tasks for Syndic dashboard (Syndic and Accountant roles)
#[get("/stats/syndic/urgent-tasks")]
pub async fn get_syndic_urgent_tasks(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
) -> impl Responder {
    // Only Syndic and Accountant can access these tasks
    if user.role != "syndic" && user.role != "accountant" && user.role != "superadmin" {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Only Syndic and Accountant can access these tasks"
        }));
    }

    let mut urgent_tasks: Vec<UrgentTask> = Vec::new();

    // 1. Get overdue expenses (payment_status = 'overdue')
    let overdue_expenses = sqlx::query!(
        "SELECT e.id, e.description, e.amount, b.name as building_name, e.expense_date
         FROM expenses e
         INNER JOIN buildings b ON e.building_id = b.id
         WHERE b.organization_id = $1
         AND e.payment_status = 'overdue'
         ORDER BY e.expense_date ASC
         LIMIT 5",
        user.organization_id
    )
    .fetch_all(&state.pool)
    .await;

    if let Ok(expenses) = overdue_expenses {
        for expense in expenses {
            urgent_tasks.push(UrgentTask {
                task_type: "expense".to_string(),
                title: format!("Charge en retard - {:.2}€", expense.amount),
                description: expense.description,
                priority: "urgent".to_string(),
                building_name: Some(expense.building_name),
                entity_id: Some(expense.id.to_string()),
                due_date: Some(expense.expense_date),
            });
        }
    }

    // 2. Get upcoming meetings (within 7 days)
    let upcoming_meetings = sqlx::query!(
        "SELECT m.id, m.title, m.scheduled_date, b.name as building_name
         FROM meetings m
         INNER JOIN buildings b ON m.building_id = b.id
         WHERE b.organization_id = $1
         AND m.status = 'scheduled'
         AND m.scheduled_date BETWEEN NOW() AND NOW() + INTERVAL '7 days'
         ORDER BY m.scheduled_date ASC
         LIMIT 3",
        user.organization_id
    )
    .fetch_all(&state.pool)
    .await;

    if let Ok(meetings) = upcoming_meetings {
        for meeting in meetings {
            let days_until = (meeting.scheduled_date - Utc::now()).num_days();
            let priority = if days_until <= 3 { "urgent" } else { "high" };

            urgent_tasks.push(UrgentTask {
                task_type: "meeting".to_string(),
                title: meeting.title,
                description: format!("AG dans {} jours", days_until),
                priority: priority.to_string(),
                building_name: Some(meeting.building_name),
                entity_id: Some(meeting.id.to_string()),
                due_date: Some(meeting.scheduled_date),
            });
        }
    }

    // 3. Get pending expenses (payment_status = 'pending' and overdue)
    let pending_overdue = sqlx::query!(
        "SELECT COUNT(*) as count
         FROM expenses e
         INNER JOIN buildings b ON e.building_id = b.id
         WHERE b.organization_id = $1
         AND e.payment_status = 'pending'
         AND e.expense_date < NOW() - INTERVAL '30 days'",
        user.organization_id
    )
    .fetch_one(&state.pool)
    .await;

    if let Ok(data) = pending_overdue {
        if let Some(count) = data.count {
            if count > 0 {
                urgent_tasks.push(UrgentTask {
                    task_type: "expense".to_string(),
                    title: "Relance paiements".to_string(),
                    description: format!("{} charges en attente depuis plus de 30 jours", count),
                    priority: "high".to_string(),
                    building_name: None,
                    entity_id: None,
                    due_date: None,
                });
            }
        }
    }

    // Sort by priority and due date
    urgent_tasks.sort_by(|a, b| {
        let priority_order = |p: &str| match p {
            "urgent" => 0,
            "high" => 1,
            _ => 2,
        };
        priority_order(&a.priority).cmp(&priority_order(&b.priority))
    });

    HttpResponse::Ok().json(urgent_tasks)
}

/// Get seed data statistics (SuperAdmin only) - shows breakdown of seed vs production data
#[get("/stats/seed-data")]
pub async fn get_seed_data_stats(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
) -> impl Responder {
    // Only SuperAdmin can access these stats
    if user.role != "superadmin" {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Only SuperAdmin can access seed data statistics"
        }));
    }

    // Count seed organizations
    let seed_orgs_result = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM organizations WHERE is_seed_data = true",
    )
    .fetch_one(&state.pool)
    .await;

    // Count production organizations
    let prod_orgs_result = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM organizations WHERE is_seed_data = false",
    )
    .fetch_one(&state.pool)
    .await;

    // Count buildings in seed organizations
    let seed_buildings_result = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM buildings b
         INNER JOIN organizations o ON b.organization_id = o.id
         WHERE o.is_seed_data = true",
    )
    .fetch_one(&state.pool)
    .await;

    // Count units in seed organizations
    let seed_units_result = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM units u
         INNER JOIN buildings b ON u.building_id = b.id
         INNER JOIN organizations o ON b.organization_id = o.id
         WHERE o.is_seed_data = true",
    )
    .fetch_one(&state.pool)
    .await;

    // Count owners in seed organizations (via unit_owners)
    let seed_owners_result = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(DISTINCT o.id) FROM owners o
         INNER JOIN unit_owners uo ON o.id = uo.owner_id
         INNER JOIN units u ON uo.unit_id = u.id
         INNER JOIN buildings b ON u.building_id = b.id
         INNER JOIN organizations org ON b.organization_id = org.id
         WHERE org.is_seed_data = true",
    )
    .fetch_one(&state.pool)
    .await;

    // Count unit_owners relationships in seed organizations
    let seed_unit_owners_result = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM unit_owners uo
         INNER JOIN units u ON uo.unit_id = u.id
         INNER JOIN buildings b ON u.building_id = b.id
         INNER JOIN organizations o ON b.organization_id = o.id
         WHERE o.is_seed_data = true",
    )
    .fetch_one(&state.pool)
    .await;

    // Count expenses in seed organizations
    let seed_expenses_result = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM expenses e
         INNER JOIN buildings b ON e.building_id = b.id
         INNER JOIN organizations o ON b.organization_id = o.id
         WHERE o.is_seed_data = true",
    )
    .fetch_one(&state.pool)
    .await;

    // Count meetings in seed organizations
    let seed_meetings_result = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM meetings m
         INNER JOIN buildings b ON m.building_id = b.id
         INNER JOIN organizations o ON b.organization_id = o.id
         WHERE o.is_seed_data = true",
    )
    .fetch_one(&state.pool)
    .await;

    // Count users in seed organizations
    let seed_users_result = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM users u
         INNER JOIN organizations o ON u.organization_id = o.id
         WHERE o.is_seed_data = true",
    )
    .fetch_one(&state.pool)
    .await;

    // Handle results
    match (
        seed_orgs_result,
        prod_orgs_result,
        seed_buildings_result,
        seed_units_result,
        seed_owners_result,
        seed_unit_owners_result,
        seed_expenses_result,
        seed_meetings_result,
        seed_users_result,
    ) {
        (
            Ok(seed_organizations),
            Ok(production_organizations),
            Ok(seed_buildings),
            Ok(seed_units),
            Ok(seed_owners),
            Ok(seed_unit_owners),
            Ok(seed_expenses),
            Ok(seed_meetings),
            Ok(seed_users),
        ) => {
            let stats = SeedDataStats {
                seed_organizations,
                production_organizations,
                seed_buildings,
                seed_units,
                seed_owners,
                seed_unit_owners,
                seed_expenses,
                seed_meetings,
                seed_users,
            };
            HttpResponse::Ok().json(stats)
        }
        _ => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "Failed to fetch seed data statistics"
        })),
    }
}
