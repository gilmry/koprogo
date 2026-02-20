use crate::application::ports::TicketRepository;
use crate::domain::entities::{Ticket, TicketCategory, TicketPriority, TicketStatus};
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

/// PostgreSQL implementation of TicketRepository
pub struct PostgresTicketRepository {
    pool: PgPool,
}

impl PostgresTicketRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Convert TicketCategory enum to database string
    fn category_to_db(category: &TicketCategory) -> &'static str {
        match category {
            TicketCategory::Plumbing => "Plumbing",
            TicketCategory::Electrical => "Electrical",
            TicketCategory::Heating => "Heating",
            TicketCategory::CommonAreas => "CommonAreas",
            TicketCategory::Elevator => "Elevator",
            TicketCategory::Security => "Security",
            TicketCategory::Cleaning => "Cleaning",
            TicketCategory::Landscaping => "Landscaping",
            TicketCategory::Other => "Other",
        }
    }

    /// Convert database string to TicketCategory enum
    fn category_from_db(s: &str) -> Result<TicketCategory, String> {
        match s {
            "Plumbing" => Ok(TicketCategory::Plumbing),
            "Electrical" => Ok(TicketCategory::Electrical),
            "Heating" => Ok(TicketCategory::Heating),
            "CommonAreas" => Ok(TicketCategory::CommonAreas),
            "Elevator" => Ok(TicketCategory::Elevator),
            "Security" => Ok(TicketCategory::Security),
            "Cleaning" => Ok(TicketCategory::Cleaning),
            "Landscaping" => Ok(TicketCategory::Landscaping),
            "Other" => Ok(TicketCategory::Other),
            _ => Err(format!("Invalid ticket category: {}", s)),
        }
    }

    /// Convert TicketPriority enum to database string
    fn priority_to_db(priority: &TicketPriority) -> &'static str {
        match priority {
            TicketPriority::Low => "Low",
            TicketPriority::Medium => "Medium",
            TicketPriority::High => "High",
            TicketPriority::Critical => "Critical",
        }
    }

    /// Convert database string to TicketPriority enum
    fn priority_from_db(s: &str) -> Result<TicketPriority, String> {
        match s {
            "Low" => Ok(TicketPriority::Low),
            "Medium" => Ok(TicketPriority::Medium),
            "High" => Ok(TicketPriority::High),
            "Critical" => Ok(TicketPriority::Critical),
            _ => Err(format!("Invalid ticket priority: {}", s)),
        }
    }

    /// Convert TicketStatus enum to database string
    fn status_to_db(status: &TicketStatus) -> &'static str {
        match status {
            TicketStatus::Open => "Open",
            TicketStatus::InProgress => "InProgress",
            TicketStatus::Resolved => "Resolved",
            TicketStatus::Closed => "Closed",
            TicketStatus::Cancelled => "Cancelled",
        }
    }

    /// Convert database string to TicketStatus enum
    fn status_from_db(s: &str) -> Result<TicketStatus, String> {
        match s {
            "Open" => Ok(TicketStatus::Open),
            "InProgress" => Ok(TicketStatus::InProgress),
            "Resolved" => Ok(TicketStatus::Resolved),
            "Closed" => Ok(TicketStatus::Closed),
            "Cancelled" => Ok(TicketStatus::Cancelled),
            _ => Err(format!("Invalid ticket status: {}", s)),
        }
    }
}

#[async_trait]
impl TicketRepository for PostgresTicketRepository {
    async fn create(&self, ticket: &Ticket) -> Result<Ticket, String> {
        let category_str = Self::category_to_db(&ticket.category);
        let priority_str = Self::priority_to_db(&ticket.priority);
        let status_str = Self::status_to_db(&ticket.status);

        let row = sqlx::query!(
            r#"
            INSERT INTO tickets (
                id, organization_id, building_id, unit_id, created_by, assigned_to,
                title, description, category, priority, status, resolution_notes,
                created_at, updated_at, resolved_at, closed_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
            RETURNING id, organization_id, building_id, unit_id, created_by, assigned_to,
                      title, description, category, priority, status, resolution_notes,
                      created_at, updated_at, resolved_at, closed_at
            "#,
            ticket.id,
            ticket.organization_id,
            ticket.building_id,
            ticket.unit_id,
            ticket.created_by,
            ticket.assigned_to,
            ticket.title,
            ticket.description,
            category_str,
            priority_str,
            status_str,
            ticket.resolution_notes,
            ticket.created_at,
            ticket.updated_at,
            ticket.resolved_at,
            ticket.closed_at
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Database error creating ticket: {}", e))?;

        Ok(Ticket {
            id: row.id,
            organization_id: row.organization_id,
            building_id: row.building_id,
            unit_id: row.unit_id,
            created_by: row.created_by,
            assigned_to: row.assigned_to,
            title: row.title,
            description: row.description,
            category: Self::category_from_db(&row.category)?,
            priority: Self::priority_from_db(&row.priority)?,
            status: Self::status_from_db(&row.status)?,
            resolution_notes: row.resolution_notes,
            created_at: row.created_at,
            updated_at: row.updated_at,
            resolved_at: row.resolved_at,
            closed_at: row.closed_at,
        })
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Ticket>, String> {
        let row = sqlx::query!(
            r#"
            SELECT id, organization_id, building_id, unit_id, created_by, assigned_to,
                   title, description, category, priority, status, resolution_notes,
                   created_at, updated_at, resolved_at, closed_at
            FROM tickets
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Database error finding ticket: {}", e))?;

        match row {
            Some(r) => Ok(Some(Ticket {
                id: r.id,
                organization_id: r.organization_id,
                building_id: r.building_id,
                unit_id: r.unit_id,
                created_by: r.created_by,
                assigned_to: r.assigned_to,
                title: r.title,
                description: r.description,
                category: Self::category_from_db(&r.category)?,
                priority: Self::priority_from_db(&r.priority)?,
                status: Self::status_from_db(&r.status)?,
                resolution_notes: r.resolution_notes,
                created_at: r.created_at,
                updated_at: r.updated_at,
                resolved_at: r.resolved_at,
                closed_at: r.closed_at,
            })),
            None => Ok(None),
        }
    }

    async fn find_by_building(&self, building_id: Uuid) -> Result<Vec<Ticket>, String> {
        let rows = sqlx::query!(
            r#"
            SELECT id, organization_id, building_id, unit_id, created_by, assigned_to,
                   title, description, category, priority, status, resolution_notes,
                   created_at, updated_at, resolved_at, closed_at
            FROM tickets
            WHERE building_id = $1
            ORDER BY created_at DESC
            "#,
            building_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Database error finding tickets by building: {}", e))?;

        rows.into_iter()
            .map(|r| {
                Ok(Ticket {
                    id: r.id,
                    organization_id: r.organization_id,
                    building_id: r.building_id,
                    unit_id: r.unit_id,
                    created_by: r.created_by,
                    assigned_to: r.assigned_to,
                    title: r.title,
                    description: r.description,
                    category: Self::category_from_db(&r.category)?,
                    priority: Self::priority_from_db(&r.priority)?,
                    status: Self::status_from_db(&r.status)?,
                    resolution_notes: r.resolution_notes,
                    created_at: r.created_at,
                    updated_at: r.updated_at,
                    resolved_at: r.resolved_at,
                    closed_at: r.closed_at,
                })
            })
            .collect()
    }

    async fn find_by_organization(&self, organization_id: Uuid) -> Result<Vec<Ticket>, String> {
        let rows = sqlx::query!(
            r#"
            SELECT id, organization_id, building_id, unit_id, created_by, assigned_to,
                   title, description, category, priority, status, resolution_notes,
                   created_at, updated_at, resolved_at, closed_at
            FROM tickets
            WHERE organization_id = $1
            ORDER BY created_at DESC
            "#,
            organization_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Database error finding tickets by organization: {}", e))?;

        rows.into_iter()
            .map(|r| {
                Ok(Ticket {
                    id: r.id,
                    organization_id: r.organization_id,
                    building_id: r.building_id,
                    unit_id: r.unit_id,
                    created_by: r.created_by,
                    assigned_to: r.assigned_to,
                    title: r.title,
                    description: r.description,
                    category: Self::category_from_db(&r.category)?,
                    priority: Self::priority_from_db(&r.priority)?,
                    status: Self::status_from_db(&r.status)?,
                    resolution_notes: r.resolution_notes,
                    created_at: r.created_at,
                    updated_at: r.updated_at,
                    resolved_at: r.resolved_at,
                    closed_at: r.closed_at,
                })
            })
            .collect()
    }

    async fn find_by_created_by(&self, created_by: Uuid) -> Result<Vec<Ticket>, String> {
        let rows = sqlx::query!(
            r#"
            SELECT id, organization_id, building_id, unit_id, created_by, assigned_to,
                   title, description, category, priority, status, resolution_notes,
                   created_at, updated_at, resolved_at, closed_at
            FROM tickets
            WHERE created_by = $1
            ORDER BY created_at DESC
            "#,
            created_by
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Database error finding tickets by creator: {}", e))?;

        rows.into_iter()
            .map(|r| {
                Ok(Ticket {
                    id: r.id,
                    organization_id: r.organization_id,
                    building_id: r.building_id,
                    unit_id: r.unit_id,
                    created_by: r.created_by,
                    assigned_to: r.assigned_to,
                    title: r.title,
                    description: r.description,
                    category: Self::category_from_db(&r.category)?,
                    priority: Self::priority_from_db(&r.priority)?,
                    status: Self::status_from_db(&r.status)?,
                    resolution_notes: r.resolution_notes,
                    created_at: r.created_at,
                    updated_at: r.updated_at,
                    resolved_at: r.resolved_at,
                    closed_at: r.closed_at,
                })
            })
            .collect()
    }

    async fn find_by_assigned_to(&self, assigned_to: Uuid) -> Result<Vec<Ticket>, String> {
        let rows = sqlx::query!(
            r#"
            SELECT id, organization_id, building_id, unit_id, created_by, assigned_to,
                   title, description, category, priority, status, resolution_notes,
                   created_at, updated_at, resolved_at, closed_at
            FROM tickets
            WHERE assigned_to = $1
            ORDER BY created_at DESC
            "#,
            assigned_to
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Database error finding tickets by assignee: {}", e))?;

        rows.into_iter()
            .map(|r| {
                Ok(Ticket {
                    id: r.id,
                    organization_id: r.organization_id,
                    building_id: r.building_id,
                    unit_id: r.unit_id,
                    created_by: r.created_by,
                    assigned_to: r.assigned_to,
                    title: r.title,
                    description: r.description,
                    category: Self::category_from_db(&r.category)?,
                    priority: Self::priority_from_db(&r.priority)?,
                    status: Self::status_from_db(&r.status)?,
                    resolution_notes: r.resolution_notes,
                    created_at: r.created_at,
                    updated_at: r.updated_at,
                    resolved_at: r.resolved_at,
                    closed_at: r.closed_at,
                })
            })
            .collect()
    }

    async fn find_by_status(
        &self,
        building_id: Uuid,
        status: TicketStatus,
    ) -> Result<Vec<Ticket>, String> {
        let status_str = Self::status_to_db(&status);

        let rows = sqlx::query!(
            r#"
            SELECT id, organization_id, building_id, unit_id, created_by, assigned_to,
                   title, description, category, priority, status, resolution_notes,
                   created_at, updated_at, resolved_at, closed_at
            FROM tickets
            WHERE building_id = $1 AND status = $2
            ORDER BY created_at DESC
            "#,
            building_id,
            status_str
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Database error finding tickets by status: {}", e))?;

        rows.into_iter()
            .map(|r| {
                Ok(Ticket {
                    id: r.id,
                    organization_id: r.organization_id,
                    building_id: r.building_id,
                    unit_id: r.unit_id,
                    created_by: r.created_by,
                    assigned_to: r.assigned_to,
                    title: r.title,
                    description: r.description,
                    category: Self::category_from_db(&r.category)?,
                    priority: Self::priority_from_db(&r.priority)?,
                    status: Self::status_from_db(&r.status)?,
                    resolution_notes: r.resolution_notes,
                    created_at: r.created_at,
                    updated_at: r.updated_at,
                    resolved_at: r.resolved_at,
                    closed_at: r.closed_at,
                })
            })
            .collect()
    }

    async fn update(&self, ticket: &Ticket) -> Result<Ticket, String> {
        let category_str = Self::category_to_db(&ticket.category);
        let priority_str = Self::priority_to_db(&ticket.priority);
        let status_str = Self::status_to_db(&ticket.status);

        let row = sqlx::query!(
            r#"
            UPDATE tickets
            SET organization_id = $2,
                building_id = $3,
                unit_id = $4,
                created_by = $5,
                assigned_to = $6,
                title = $7,
                description = $8,
                category = $9,
                priority = $10,
                status = $11,
                resolution_notes = $12,
                updated_at = $13,
                resolved_at = $14,
                closed_at = $15
            WHERE id = $1
            RETURNING id, organization_id, building_id, unit_id, created_by, assigned_to,
                      title, description, category, priority, status, resolution_notes,
                      created_at, updated_at, resolved_at, closed_at
            "#,
            ticket.id,
            ticket.organization_id,
            ticket.building_id,
            ticket.unit_id,
            ticket.created_by,
            ticket.assigned_to,
            ticket.title,
            ticket.description,
            category_str,
            priority_str,
            status_str,
            ticket.resolution_notes,
            ticket.updated_at,
            ticket.resolved_at,
            ticket.closed_at
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Database error updating ticket: {}", e))?;

        Ok(Ticket {
            id: row.id,
            organization_id: row.organization_id,
            building_id: row.building_id,
            unit_id: row.unit_id,
            created_by: row.created_by,
            assigned_to: row.assigned_to,
            title: row.title,
            description: row.description,
            category: Self::category_from_db(&row.category)?,
            priority: Self::priority_from_db(&row.priority)?,
            status: Self::status_from_db(&row.status)?,
            resolution_notes: row.resolution_notes,
            created_at: row.created_at,
            updated_at: row.updated_at,
            resolved_at: row.resolved_at,
            closed_at: row.closed_at,
        })
    }

    async fn delete(&self, id: Uuid) -> Result<bool, String> {
        let result = sqlx::query!(
            r#"
            DELETE FROM tickets
            WHERE id = $1
            "#,
            id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Database error deleting ticket: {}", e))?;

        Ok(result.rows_affected() > 0)
    }

    async fn count_by_building(&self, building_id: Uuid) -> Result<i64, String> {
        let row = sqlx::query!(
            r#"
            SELECT COUNT(*) as count
            FROM tickets
            WHERE building_id = $1
            "#,
            building_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Database error counting tickets: {}", e))?;

        Ok(row.count.unwrap_or(0))
    }

    async fn count_by_status(
        &self,
        building_id: Uuid,
        status: TicketStatus,
    ) -> Result<i64, String> {
        let status_str = Self::status_to_db(&status);

        let row = sqlx::query!(
            r#"
            SELECT COUNT(*) as count
            FROM tickets
            WHERE building_id = $1 AND status = $2
            "#,
            building_id,
            status_str
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Database error counting tickets by status: {}", e))?;

        Ok(row.count.unwrap_or(0))
    }

    async fn count_by_organization(&self, organization_id: Uuid) -> Result<i64, String> {
        let count: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM tickets WHERE organization_id = $1")
                .bind(organization_id)
                .fetch_one(&self.pool)
                .await
                .map_err(|e| format!("Database error counting tickets by organization: {}", e))?;

        Ok(count.0)
    }

    async fn count_by_organization_and_status(
        &self,
        organization_id: Uuid,
        status: TicketStatus,
    ) -> Result<i64, String> {
        let status_str = Self::status_to_db(&status);

        let count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM tickets WHERE organization_id = $1 AND status = $2",
        )
        .bind(organization_id)
        .bind(status_str)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Database error counting tickets by org and status: {}", e))?;

        Ok(count.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_category_conversion() {
        assert_eq!(
            PostgresTicketRepository::category_to_db(&TicketCategory::Plumbing),
            "Plumbing"
        );
        assert_eq!(
            PostgresTicketRepository::category_from_db("Electrical").unwrap(),
            TicketCategory::Electrical
        );
    }

    #[test]
    fn test_priority_conversion() {
        assert_eq!(
            PostgresTicketRepository::priority_to_db(&TicketPriority::Critical),
            "Critical"
        );
        assert_eq!(
            PostgresTicketRepository::priority_from_db("Low").unwrap(),
            TicketPriority::Low
        );
    }

    #[test]
    fn test_status_conversion() {
        assert_eq!(
            PostgresTicketRepository::status_to_db(&TicketStatus::InProgress),
            "InProgress"
        );
        assert_eq!(
            PostgresTicketRepository::status_from_db("Resolved").unwrap(),
            TicketStatus::Resolved
        );
    }

    #[test]
    fn test_invalid_category() {
        assert!(PostgresTicketRepository::category_from_db("Invalid").is_err());
    }

    #[test]
    fn test_invalid_priority() {
        assert!(PostgresTicketRepository::priority_from_db("Invalid").is_err());
    }

    #[test]
    fn test_invalid_status() {
        assert!(PostgresTicketRepository::status_from_db("Invalid").is_err());
    }
}
