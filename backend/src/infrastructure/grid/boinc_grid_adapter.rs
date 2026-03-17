use crate::application::ports::grid_participation_port::{
    BoincConsent, GridError, GridParticipationPort, GridTask, GridTaskId, GridTaskStatus,
};
use async_trait::async_trait;
use chrono::Utc;
use sqlx::PgPool;
use tokio::process::Command;
use tracing::{info, warn};
use uuid::Uuid;

/// Adapter BOINC: implémente GridParticipationPort via boinccmd + PostgreSQL.
///
/// Consentement GDPR stocké dans la table `boinc_consents`.
/// Tâches stockées dans `grid_tasks` avec statut polling.
///
/// Variables d'environnement:
/// - BOINC_HOST (défaut: "localhost")
/// - BOINC_PORT (défaut: 31416)
/// - BOINC_RPC_PASSWORD
pub struct BoincGridAdapter {
    pool: PgPool,
    boinc_rpc_password: String,
    boinc_host: String,
    boinc_port: u16,
}

impl BoincGridAdapter {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            boinc_rpc_password: std::env::var("BOINC_RPC_PASSWORD").unwrap_or_default(),
            boinc_host: std::env::var("BOINC_HOST")
                .unwrap_or_else(|_| "localhost".to_string()),
            boinc_port: std::env::var("BOINC_PORT")
                .unwrap_or_else(|_| "31416".to_string())
                .parse()
                .unwrap_or(31416),
        }
    }

    /// Exécute boinccmd avec les arguments donnés.
    /// Retourne stdout en cas de succès, GridError::RpcFailed en cas d'échec.
    async fn run_boinccmd(&self, args: &[&str]) -> Result<String, GridError> {
        let mut cmd = Command::new("boinccmd");
        cmd.arg("--host")
            .arg(&self.boinc_host)
            .arg("--port")
            .arg(self.boinc_port.to_string())
            .arg("--passwd")
            .arg(&self.boinc_rpc_password);
        for arg in args {
            cmd.arg(arg);
        }
        let output = cmd.output().await.map_err(|e| {
            GridError::ProcessError(format!("boinccmd not found or spawn failed: {e}"))
        })?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(GridError::RpcFailed(stderr.to_string()));
        }
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}

#[async_trait]
impl GridParticipationPort for BoincGridAdapter {
    async fn check_consent(&self, owner_id: Uuid) -> Result<bool, GridError> {
        let row: Option<(bool,)> = sqlx::query_as(
            r#"SELECT granted FROM boinc_consents
               WHERE owner_id = $1 AND revoked_at IS NULL
               ORDER BY granted_at DESC LIMIT 1"#,
        )
        .bind(owner_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| GridError::RpcFailed(e.to_string()))?;

        Ok(row.map(|(g,)| g).unwrap_or(false))
    }

    async fn get_consent(&self, owner_id: Uuid) -> Result<Option<BoincConsent>, GridError> {
        let row: Option<(
            Uuid,
            Uuid,
            bool,
            Option<chrono::DateTime<Utc>>,
            Option<chrono::DateTime<Utc>>,
            Option<String>,
            String,
        )> = sqlx::query_as(
            r#"SELECT owner_id, organization_id, granted, granted_at, revoked_at,
                      consent_ip, consent_version
               FROM boinc_consents
               WHERE owner_id = $1
               ORDER BY COALESCE(granted_at, created_at) DESC
               LIMIT 1"#,
        )
        .bind(owner_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| GridError::RpcFailed(e.to_string()))?;

        Ok(row.map(
            |(oid, org_id, granted, granted_at, revoked_at, consent_ip, consent_version)| {
                BoincConsent {
                    owner_id: oid,
                    organization_id: org_id,
                    granted,
                    granted_at,
                    revoked_at,
                    consent_ip,
                    consent_version,
                }
            },
        ))
    }

    async fn grant_consent(
        &self,
        owner_id: Uuid,
        organization_id: Uuid,
        consent_version: &str,
        consent_ip: Option<&str>,
    ) -> Result<BoincConsent, GridError> {
        let now = Utc::now();
        sqlx::query(
            r#"INSERT INTO boinc_consents
               (id, owner_id, organization_id, granted, granted_at, consent_ip, consent_version)
               VALUES ($1, $2, $3, true, $4, $5, $6)
               ON CONFLICT (owner_id) DO UPDATE
               SET granted = true,
                   granted_at = $4,
                   revoked_at = NULL,
                   consent_ip = $5,
                   consent_version = $6,
                   updated_at = NOW()"#,
        )
        .bind(Uuid::new_v4())
        .bind(owner_id)
        .bind(organization_id)
        .bind(now)
        .bind(consent_ip)
        .bind(consent_version)
        .execute(&self.pool)
        .await
        .map_err(|e| GridError::RpcFailed(e.to_string()))?;

        info!("BOINC consent granted for owner {}", owner_id);
        Ok(BoincConsent {
            owner_id,
            organization_id,
            granted: true,
            granted_at: Some(now),
            revoked_at: None,
            consent_ip: consent_ip.map(|s| s.to_string()),
            consent_version: consent_version.to_string(),
        })
    }

    async fn revoke_consent(&self, owner_id: Uuid) -> Result<(), GridError> {
        sqlx::query(
            r#"UPDATE boinc_consents
               SET granted = false, revoked_at = NOW(), updated_at = NOW()
               WHERE owner_id = $1"#,
        )
        .bind(owner_id)
        .execute(&self.pool)
        .await
        .map_err(|e| GridError::RpcFailed(e.to_string()))?;

        info!("BOINC consent revoked for owner {}", owner_id);
        Ok(())
    }

    async fn submit_task(&self, task: GridTask) -> Result<GridTaskId, GridError> {
        let kind_json = serde_json::to_value(&task.kind)
            .map_err(|e| GridError::ProcessError(e.to_string()))?;

        sqlx::query(
            r#"INSERT INTO grid_tasks
               (id, copropriete_id, organization_id, kind_json, status, priority, deadline_at)
               VALUES ($1, $2, $3, $4, 'queued', $5, $6)"#,
        )
        .bind(task.internal_id)
        .bind(task.copropriete_id)
        .bind(task.organization_id)
        .bind(kind_json)
        .bind(task.priority as i32)
        .bind(task.deadline)
        .execute(&self.pool)
        .await
        .map_err(|e| GridError::RpcFailed(e.to_string()))?;

        // Tentative best-effort de notifier BOINC (ne bloque pas si boinccmd absent)
        match self.run_boinccmd(&["--get_simple_gui_info"]).await {
            Ok(_) => info!("BOINC daemon reachable, task {} queued", task.internal_id),
            Err(e) => warn!(
                "BOINC daemon not reachable (task stored in DB for retry): {}",
                e
            ),
        }

        Ok(GridTaskId(task.internal_id.to_string()))
    }

    async fn poll_result(&self, task_id: &GridTaskId) -> Result<GridTaskStatus, GridError> {
        let id = Uuid::parse_str(&task_id.0)
            .map_err(|_| GridError::TaskNotFound(task_id.0.clone()))?;

        let row: Option<(
            String,
            Option<String>,
            Option<chrono::DateTime<Utc>>,
            Option<chrono::DateTime<Utc>>,
            Option<chrono::DateTime<Utc>>,
            Option<String>,
        )> = sqlx::query_as(
            r#"SELECT status, result_json::text,
                      started_at, completed_at, failed_at, failure_reason
               FROM grid_tasks WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| GridError::RpcFailed(e.to_string()))?;

        let (status, result_json, started_at, completed_at, failed_at, failure_reason) = row
            .ok_or_else(|| GridError::TaskNotFound(task_id.0.clone()))?;

        Ok(match status.as_str() {
            "running" => GridTaskStatus::Running {
                started_at: started_at.unwrap_or_else(Utc::now),
            },
            "completed" => GridTaskStatus::Completed {
                completed_at: completed_at.unwrap_or_else(Utc::now),
                result_json: result_json.unwrap_or_default(),
            },
            "failed" => GridTaskStatus::Failed {
                failed_at: failed_at.unwrap_or_else(Utc::now),
                reason: failure_reason.unwrap_or_default(),
            },
            "cancelled" => GridTaskStatus::Cancelled,
            _ => GridTaskStatus::Queued,
        })
    }

    async fn cancel_task(&self, task_id: &GridTaskId) -> Result<(), GridError> {
        let id = Uuid::parse_str(&task_id.0)
            .map_err(|_| GridError::TaskNotFound(task_id.0.clone()))?;

        sqlx::query(
            "UPDATE grid_tasks SET status = 'cancelled', updated_at = NOW() WHERE id = $1",
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| GridError::RpcFailed(e.to_string()))?;

        Ok(())
    }
}
