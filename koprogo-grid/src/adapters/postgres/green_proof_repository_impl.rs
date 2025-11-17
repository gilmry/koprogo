use crate::core::GreenProof;
use crate::ports::GreenProofRepository;
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

pub struct PostgresGreenProofRepository {
    pool: PgPool,
}

impl PostgresGreenProofRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl GreenProofRepository for PostgresGreenProofRepository {
    async fn create(&self, proof: &GreenProof) -> Result<GreenProof, String> {
        sqlx::query!(
            r#"
            INSERT INTO grid_green_proofs (
                id, task_id, node_id, block_hash, previous_hash, timestamp,
                energy_used_wh, solar_contribution_wh, carbon_saved_kg, nonce
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
            proof.id,
            proof.task_id,
            proof.node_id,
            proof.block_hash,
            proof.previous_hash,
            proof.timestamp,
            proof.energy_used_wh,
            proof.solar_contribution_wh,
            proof.carbon_saved_kg,
            proof.nonce as i64
        )
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to create green proof: {}", e))?;

        Ok(proof.clone())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<GreenProof>, String> {
        sqlx::query_as!(
            GreenProofRow,
            "SELECT * FROM grid_green_proofs WHERE id = $1",
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map(|opt| opt.map(|row| row.into()))
        .map_err(|e| format!("Failed to find green proof: {}", e))
    }

    async fn find_by_task(&self, task_id: Uuid) -> Result<Vec<GreenProof>, String> {
        sqlx::query_as!(
            GreenProofRow,
            "SELECT * FROM grid_green_proofs WHERE task_id = $1",
            task_id
        )
        .fetch_all(&self.pool)
        .await
        .map(|rows| rows.into_iter().map(|row| row.into()).collect())
        .map_err(|e| format!("Failed to find proofs by task: {}", e))
    }

    async fn find_by_node(&self, node_id: Uuid) -> Result<Vec<GreenProof>, String> {
        sqlx::query_as!(
            GreenProofRow,
            "SELECT * FROM grid_green_proofs WHERE node_id = $1",
            node_id
        )
        .fetch_all(&self.pool)
        .await
        .map(|rows| rows.into_iter().map(|row| row.into()).collect())
        .map_err(|e| format!("Failed to find proofs by node: {}", e))
    }

    async fn get_latest(&self) -> Result<Option<GreenProof>, String> {
        sqlx::query_as!(
            GreenProofRow,
            "SELECT * FROM grid_green_proofs ORDER BY timestamp DESC LIMIT 1"
        )
        .fetch_optional(&self.pool)
        .await
        .map(|opt| opt.map(|row| row.into()))
        .map_err(|e| format!("Failed to get latest proof: {}", e))
    }

    async fn list(&self, limit: i64, offset: i64) -> Result<Vec<GreenProof>, String> {
        sqlx::query_as!(
            GreenProofRow,
            "SELECT * FROM grid_green_proofs ORDER BY timestamp DESC LIMIT $1 OFFSET $2",
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await
        .map(|rows| rows.into_iter().map(|row| row.into()).collect())
        .map_err(|e| format!("Failed to list proofs: {}", e))
    }

    async fn verify_chain(&self) -> Result<bool, String> {
        // Simple verification: check that all proofs are valid
        let proofs = self.list(1000, 0).await?;

        for proof in &proofs {
            if !proof.verify() {
                return Ok(false);
            }
        }

        // Check chain integrity (each previous_hash matches the previous block's hash)
        for i in 1..proofs.len() {
            if let Some(prev_hash) = &proofs[i].previous_hash {
                if prev_hash != &proofs[i - 1].block_hash {
                    return Ok(false);
                }
            }
        }

        Ok(true)
    }
}

#[derive(Debug)]
struct GreenProofRow {
    id: Uuid,
    task_id: Uuid,
    node_id: Uuid,
    block_hash: String,
    previous_hash: Option<String>,
    timestamp: chrono::DateTime<chrono::Utc>,
    energy_used_wh: f64,
    solar_contribution_wh: f64,
    carbon_saved_kg: f64,
    nonce: i64,
}

impl From<GreenProofRow> for GreenProof {
    fn from(row: GreenProofRow) -> Self {
        GreenProof {
            id: row.id,
            task_id: row.task_id,
            node_id: row.node_id,
            block_hash: row.block_hash,
            previous_hash: row.previous_hash,
            timestamp: row.timestamp,
            energy_used_wh: row.energy_used_wh,
            solar_contribution_wh: row.solar_contribution_wh,
            carbon_saved_kg: row.carbon_saved_kg,
            nonce: row.nonce as u64,
        }
    }
}
