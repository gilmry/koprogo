use crate::core::{Node, NodeStatus};
use crate::ports::{NodeRepository, NodeStats};
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

pub struct PostgresNodeRepository {
    pool: PgPool,
}

impl PostgresNodeRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl NodeRepository for PostgresNodeRepository {
    async fn create(&self, node: &Node) -> Result<Node, String> {
        let status_str = match node.status {
            NodeStatus::Active => "active",
            NodeStatus::Idle => "idle",
            NodeStatus::Offline => "offline",
            NodeStatus::Suspended => "suspended",
        };

        sqlx::query_as!(
            NodeRow,
            r#"
            INSERT INTO grid_nodes (
                id, name, cpu_cores, has_solar, location, status, eco_score,
                total_energy_saved_wh, total_carbon_credits, last_heartbeat, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            RETURNING *
            "#,
            node.id,
            node.name,
            node.cpu_cores as i32,
            node.has_solar,
            node.location,
            status_str,
            node.eco_score,
            node.total_energy_saved_wh,
            node.total_carbon_credits,
            node.last_heartbeat,
            node.created_at,
            node.updated_at
        )
        .fetch_one(&self.pool)
        .await
        .map(|row| row.into())
        .map_err(|e| format!("Failed to create node: {}", e))
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Node>, String> {
        sqlx::query_as!(
            NodeRow,
            "SELECT * FROM grid_nodes WHERE id = $1",
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map(|opt| opt.map(|row| row.into()))
        .map_err(|e| format!("Failed to find node: {}", e))
    }

    async fn update(&self, node: &Node) -> Result<Node, String> {
        let status_str = match node.status {
            NodeStatus::Active => "active",
            NodeStatus::Idle => "idle",
            NodeStatus::Offline => "offline",
            NodeStatus::Suspended => "suspended",
        };

        sqlx::query_as!(
            NodeRow,
            r#"
            UPDATE grid_nodes
            SET name = $2, cpu_cores = $3, has_solar = $4, location = $5, status = $6,
                eco_score = $7, total_energy_saved_wh = $8, total_carbon_credits = $9,
                last_heartbeat = $10, updated_at = $11
            WHERE id = $1
            RETURNING *
            "#,
            node.id,
            node.name,
            node.cpu_cores as i32,
            node.has_solar,
            node.location,
            status_str,
            node.eco_score,
            node.total_energy_saved_wh,
            node.total_carbon_credits,
            node.last_heartbeat,
            node.updated_at
        )
        .fetch_one(&self.pool)
        .await
        .map(|row| row.into())
        .map_err(|e| format!("Failed to update node: {}", e))
    }

    async fn delete(&self, id: Uuid) -> Result<(), String> {
        sqlx::query!("DELETE FROM grid_nodes WHERE id = $1", id)
            .execute(&self.pool)
            .await
            .map(|_| ())
            .map_err(|e| format!("Failed to delete node: {}", e))
    }

    async fn list(&self, limit: i64, offset: i64) -> Result<Vec<Node>, String> {
        sqlx::query_as!(
            NodeRow,
            "SELECT * FROM grid_nodes ORDER BY created_at DESC LIMIT $1 OFFSET $2",
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await
        .map(|rows| rows.into_iter().map(|row| row.into()).collect())
        .map_err(|e| format!("Failed to list nodes: {}", e))
    }

    async fn list_active(&self) -> Result<Vec<Node>, String> {
        sqlx::query_as!(
            NodeRow,
            r#"
            SELECT * FROM grid_nodes
            WHERE status = 'active'
              AND last_heartbeat IS NOT NULL
              AND last_heartbeat > NOW() - INTERVAL '5 minutes'
            ORDER BY eco_score DESC
            "#
        )
        .fetch_all(&self.pool)
        .await
        .map(|rows| rows.into_iter().map(|row| row.into()).collect())
        .map_err(|e| format!("Failed to list active nodes: {}", e))
    }

    async fn find_by_location(&self, location: &str) -> Result<Vec<Node>, String> {
        sqlx::query_as!(
            NodeRow,
            "SELECT * FROM grid_nodes WHERE location = $1",
            location
        )
        .fetch_all(&self.pool)
        .await
        .map(|rows| rows.into_iter().map(|row| row.into()).collect())
        .map_err(|e| format!("Failed to find nodes by location: {}", e))
    }

    async fn get_total_stats(&self) -> Result<NodeStats, String> {
        let stats = sqlx::query!(
            r#"
            SELECT
                COUNT(*) as "total_nodes!",
                COUNT(*) FILTER (
                    WHERE status = 'active'
                      AND last_heartbeat IS NOT NULL
                      AND last_heartbeat > NOW() - INTERVAL '5 minutes'
                ) as "active_nodes!",
                COALESCE(SUM(cpu_cores), 0) as "total_cpu_cores!",
                COUNT(*) FILTER (WHERE has_solar = true) as "nodes_with_solar!",
                COALESCE(SUM(total_energy_saved_wh), 0) as "total_energy_saved_wh!",
                COALESCE(SUM(total_carbon_credits), 0) as "total_carbon_credits!"
            FROM grid_nodes
            "#
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to get node stats: {}", e))?;

        Ok(NodeStats {
            total_nodes: stats.total_nodes,
            active_nodes: stats.active_nodes,
            total_cpu_cores: stats.total_cpu_cores,
            nodes_with_solar: stats.nodes_with_solar,
            total_energy_saved_wh: stats.total_energy_saved_wh,
            total_carbon_credits: stats.total_carbon_credits,
        })
    }
}

// Database row mapping
#[derive(Debug)]
struct NodeRow {
    id: Uuid,
    name: String,
    cpu_cores: i32,
    has_solar: bool,
    location: String,
    status: String,
    eco_score: f64,
    total_energy_saved_wh: f64,
    total_carbon_credits: f64,
    last_heartbeat: Option<chrono::DateTime<chrono::Utc>>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<NodeRow> for Node {
    fn from(row: NodeRow) -> Self {
        let status = match row.status.as_str() {
            "active" => NodeStatus::Active,
            "idle" => NodeStatus::Idle,
            "offline" => NodeStatus::Offline,
            "suspended" => NodeStatus::Suspended,
            _ => NodeStatus::Offline,
        };

        Node {
            id: row.id,
            name: row.name,
            cpu_cores: row.cpu_cores as u32,
            has_solar: row.has_solar,
            location: row.location,
            status,
            eco_score: row.eco_score,
            total_energy_saved_wh: row.total_energy_saved_wh,
            total_carbon_credits: row.total_carbon_credits,
            last_heartbeat: row.last_heartbeat,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}
