use crate::core::{CarbonCredit, CreditStatus};
use crate::ports::{CarbonCreditRepository, CreditStats};
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

pub struct PostgresCarbonCreditRepository {
    pool: PgPool,
}

impl PostgresCarbonCreditRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl CarbonCreditRepository for PostgresCarbonCreditRepository {
    async fn create(&self, credit: &CarbonCredit) -> Result<CarbonCredit, String> {
        let status_str = credit_status_to_str(credit.status);

        sqlx::query!(
            r#"
            INSERT INTO grid_carbon_credits (
                id, node_id, task_id, proof_id, amount_kg_co2, euro_value,
                status, cooperative_share, node_share, created_at, redeemed_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            "#,
            credit.id,
            credit.node_id,
            credit.task_id,
            credit.proof_id,
            credit.amount_kg_co2,
            credit.euro_value,
            status_str,
            credit.cooperative_share,
            credit.node_share,
            credit.created_at,
            credit.redeemed_at
        )
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to create carbon credit: {}", e))?;

        Ok(credit.clone())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<CarbonCredit>, String> {
        sqlx::query_as!(
            CreditRow,
            "SELECT * FROM grid_carbon_credits WHERE id = $1",
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map(|opt| opt.map(|row| row.into()))
        .map_err(|e| format!("Failed to find carbon credit: {}", e))
    }

    async fn update(&self, credit: &CarbonCredit) -> Result<CarbonCredit, String> {
        let status_str = credit_status_to_str(credit.status);

        sqlx::query!(
            r#"
            UPDATE grid_carbon_credits
            SET status = $2, redeemed_at = $3
            WHERE id = $1
            "#,
            credit.id,
            status_str,
            credit.redeemed_at
        )
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to update carbon credit: {}", e))?;

        Ok(credit.clone())
    }

    async fn find_by_node(&self, node_id: Uuid) -> Result<Vec<CarbonCredit>, String> {
        sqlx::query_as!(
            CreditRow,
            "SELECT * FROM grid_carbon_credits WHERE node_id = $1",
            node_id
        )
        .fetch_all(&self.pool)
        .await
        .map(|rows| rows.into_iter().map(|row| row.into()).collect())
        .map_err(|e| format!("Failed to find credits by node: {}", e))
    }

    async fn find_by_status(&self, status: CreditStatus) -> Result<Vec<CarbonCredit>, String> {
        let status_str = credit_status_to_str(status);

        sqlx::query_as!(
            CreditRow,
            "SELECT * FROM grid_carbon_credits WHERE status = $1",
            status_str
        )
        .fetch_all(&self.pool)
        .await
        .map(|rows| rows.into_iter().map(|row| row.into()).collect())
        .map_err(|e| format!("Failed to find credits by status: {}", e))
    }

    async fn get_cooperative_fund(&self) -> Result<f64, String> {
        let result = sqlx::query!(
            r#"
            SELECT COALESCE(SUM(cooperative_share), 0) as "fund!"
            FROM grid_carbon_credits
            WHERE status IN ('verified', 'redeemed')
            "#
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to get cooperative fund: {}", e))?;

        Ok(result.fund)
    }

    async fn get_node_stats(&self, node_id: Uuid) -> Result<CreditStats, String> {
        let stats = sqlx::query!(
            r#"
            SELECT
                COUNT(*) as "total_credits!",
                COALESCE(SUM(amount_kg_co2), 0) as "total_kg_co2!",
                COALESCE(SUM(euro_value), 0) as "total_euro_value!",
                COALESCE(SUM(node_share), 0) as "node_share_eur!",
                COALESCE(SUM(cooperative_share), 0) as "cooperative_share_eur!"
            FROM grid_carbon_credits
            WHERE node_id = $1
            "#,
            node_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to get credit stats: {}", e))?;

        Ok(CreditStats {
            total_credits: stats.total_credits,
            total_kg_co2: stats.total_kg_co2,
            total_euro_value: stats.total_euro_value,
            node_share_eur: stats.node_share_eur,
            cooperative_share_eur: stats.cooperative_share_eur,
        })
    }
}

fn credit_status_to_str(status: CreditStatus) -> &'static str {
    match status {
        CreditStatus::Pending => "pending",
        CreditStatus::Verified => "verified",
        CreditStatus::Redeemed => "redeemed",
        CreditStatus::Expired => "expired",
    }
}

#[derive(Debug)]
struct CreditRow {
    id: Uuid,
    node_id: Uuid,
    task_id: Uuid,
    proof_id: Uuid,
    amount_kg_co2: f64,
    euro_value: f64,
    status: String,
    cooperative_share: f64,
    node_share: f64,
    created_at: chrono::DateTime<chrono::Utc>,
    redeemed_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl From<CreditRow> for CarbonCredit {
    fn from(row: CreditRow) -> Self {
        let status = match row.status.as_str() {
            "pending" => CreditStatus::Pending,
            "verified" => CreditStatus::Verified,
            "redeemed" => CreditStatus::Redeemed,
            "expired" => CreditStatus::Expired,
            _ => CreditStatus::Pending,
        };

        CarbonCredit {
            id: row.id,
            node_id: row.node_id,
            task_id: row.task_id,
            proof_id: row.proof_id,
            amount_kg_co2: row.amount_kg_co2,
            euro_value: row.euro_value,
            status,
            cooperative_share: row.cooperative_share,
            node_share: row.node_share,
            created_at: row.created_at,
            redeemed_at: row.redeemed_at,
        }
    }
}
