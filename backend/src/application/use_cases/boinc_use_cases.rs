use crate::application::ports::grid_participation_port::{
    BoincConsent, GridParticipationPort, GridTask, GridTaskId, GridTaskKind, GridTaskStatus,
};
use crate::application::ports::iot_repository::IoTRepository;
use crate::domain::entities::MetricType;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{info, instrument};
use uuid::Uuid;

/// DTO pour soumettre une tâche d'optimisation énergétique groupée.
#[derive(Debug, Deserialize)]
pub struct SubmitOptimisationTaskDto {
    pub building_id: Uuid,
    /// Propriétaire qui initie la tâche (doit avoir consenti à BOINC)
    pub owner_id: Uuid,
    pub organization_id: Uuid,
    pub simulation_months: u32,
}

/// DTO de réponse pour une tâche grid soumise.
#[derive(Debug, Serialize)]
pub struct GridTaskResponseDto {
    pub task_id: String,
    pub message: String,
}

/// Use cases pour la gestion du calcul distribué BOINC.
/// Gère le cycle de vie des tâches + consentement GDPR.
///
/// Architecture hexagonale: dépend des ports (traits), pas des adapters.
/// L'adapter concret (BoincGridAdapter) est injecté à la construction.
pub struct BoincUseCases {
    grid_port: Arc<dyn GridParticipationPort>,
    iot_repo: Arc<dyn IoTRepository>,
}

impl BoincUseCases {
    pub fn new(
        grid_port: Arc<dyn GridParticipationPort>,
        iot_repo: Arc<dyn IoTRepository>,
    ) -> Self {
        Self {
            grid_port,
            iot_repo,
        }
    }

    /// Soumet une tâche d'optimisation énergétique groupée à BOINC.
    ///
    /// GDPR: vérifie le consentement explicite AVANT toute soumission.
    /// Anonymisation: les données kWh sont agrégées (pas de PII dans la tâche).
    #[instrument(skip(self))]
    pub async fn submit_optimisation_task(
        &self,
        dto: SubmitOptimisationTaskDto,
    ) -> Result<GridTaskResponseDto, String> {
        // GDPR: vérification consentement obligatoire
        let consented = self
            .grid_port
            .check_consent(dto.owner_id)
            .await
            .map_err(|e| e.to_string())?;
        if !consented {
            return Err(format!(
                "Owner {} has not consented to BOINC participation (GDPR Art. 6.1.a)",
                dto.owner_id
            ));
        }

        // Récupérer stats agrégées anonymisées (pas de PII) — 12 mois glissants
        let end = Utc::now();
        let start = end - chrono::Duration::days(365);
        let stats = self
            .iot_repo
            .get_consumption_stats(
                dto.building_id,
                MetricType::ElectricityConsumption,
                start,
                end,
            )
            .await
            .map_err(|e| e.to_string())?;
        let anonymised_json =
            serde_json::to_string(&stats).map_err(|e| format!("Serialization error: {e}"))?;

        let task = GridTask {
            internal_id: Uuid::new_v4(),
            copropriete_id: dto.building_id,
            organization_id: dto.organization_id,
            kind: GridTaskKind::EnergyGroupOptimisation {
                building_id: dto.building_id,
                anonymised_readings_json: anonymised_json,
                simulation_months: dto.simulation_months,
            },
            priority: 5,
            deadline: Utc::now() + chrono::Duration::hours(24),
        };

        let task_id = self
            .grid_port
            .submit_task(task)
            .await
            .map_err(|e| e.to_string())?;

        info!("BOINC task submitted: {}", task_id.0);
        Ok(GridTaskResponseDto {
            task_id: task_id.0,
            message: "Optimisation task submitted to BOINC grid".to_string(),
        })
    }

    /// Accorde le consentement BOINC pour un propriétaire (GDPR Art. 7).
    pub async fn grant_consent(
        &self,
        owner_id: Uuid,
        org_id: Uuid,
        ip: Option<&str>,
    ) -> Result<BoincConsent, String> {
        self.grid_port
            .grant_consent(owner_id, org_id, "v1.0", ip)
            .await
            .map_err(|e| e.to_string())
    }

    /// Révoque le consentement BOINC (GDPR Art. 7.3 - droit de retrait).
    pub async fn revoke_consent(&self, owner_id: Uuid) -> Result<(), String> {
        self.grid_port
            .revoke_consent(owner_id)
            .await
            .map_err(|e| e.to_string())
    }

    /// Récupère le consentement courant d'un propriétaire.
    pub async fn get_consent(&self, owner_id: Uuid) -> Result<Option<BoincConsent>, String> {
        self.grid_port
            .get_consent(owner_id)
            .await
            .map_err(|e| e.to_string())
    }

    /// Interroge le statut d'une tâche BOINC.
    pub async fn poll_task(&self, task_id: &str) -> Result<GridTaskStatus, String> {
        let id = GridTaskId(task_id.to_string());
        self.grid_port
            .poll_result(&id)
            .await
            .map_err(|e| e.to_string())
    }

    /// Annule une tâche BOINC en cours.
    pub async fn cancel_task(&self, task_id: &str) -> Result<(), String> {
        let id = GridTaskId(task_id.to_string());
        self.grid_port
            .cancel_task(&id)
            .await
            .map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::dto::{ConsumptionStatsDto, DailyAggregateDto, MonthlyAggregateDto};
    use crate::application::ports::grid_participation_port::{
        BoincConsent, GridError, GridParticipationPort, GridTask, GridTaskId, GridTaskStatus,
    };
    use crate::application::ports::iot_repository::IoTRepository;
    use crate::domain::entities::{DeviceType, IoTReading, LinkyDevice, MetricType};
    use async_trait::async_trait;
    use chrono::{DateTime, Utc};
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};
    use uuid::Uuid;

    // ── Mock GridParticipationPort ──────────────────────────────────────────
    struct MockGridPort {
        consents: Mutex<HashMap<Uuid, BoincConsent>>,
        tasks: Mutex<HashMap<String, GridTaskStatus>>,
    }

    impl MockGridPort {
        fn new() -> Self {
            Self {
                consents: Mutex::new(HashMap::new()),
                tasks: Mutex::new(HashMap::new()),
            }
        }
    }

    #[async_trait]
    impl GridParticipationPort for MockGridPort {
        async fn check_consent(&self, owner_id: Uuid) -> Result<bool, GridError> {
            let map = self.consents.lock().unwrap();
            Ok(map.get(&owner_id).map_or(false, |c| c.granted))
        }

        async fn get_consent(&self, owner_id: Uuid) -> Result<Option<BoincConsent>, GridError> {
            let map = self.consents.lock().unwrap();
            Ok(map.get(&owner_id).cloned())
        }

        async fn grant_consent(
            &self,
            owner_id: Uuid,
            organization_id: Uuid,
            consent_version: &str,
            consent_ip: Option<&str>,
        ) -> Result<BoincConsent, GridError> {
            let consent = BoincConsent {
                owner_id,
                organization_id,
                granted: true,
                granted_at: Some(Utc::now()),
                revoked_at: None,
                consent_ip: consent_ip.map(|s| s.to_string()),
                consent_version: consent_version.to_string(),
            };
            self.consents.lock().unwrap().insert(owner_id, consent.clone());
            Ok(consent)
        }

        async fn revoke_consent(&self, owner_id: Uuid) -> Result<(), GridError> {
            let mut map = self.consents.lock().unwrap();
            if let Some(consent) = map.get_mut(&owner_id) {
                consent.granted = false;
                consent.revoked_at = Some(Utc::now());
                Ok(())
            } else {
                Err(GridError::ConsentNotGranted(owner_id))
            }
        }

        async fn submit_task(&self, _task: GridTask) -> Result<GridTaskId, GridError> {
            let task_id = GridTaskId(Uuid::new_v4().to_string());
            self.tasks.lock().unwrap().insert(task_id.0.clone(), GridTaskStatus::Queued);
            Ok(task_id)
        }

        async fn poll_result(&self, task_id: &GridTaskId) -> Result<GridTaskStatus, GridError> {
            let map = self.tasks.lock().unwrap();
            map.get(&task_id.0).cloned().ok_or_else(|| GridError::TaskNotFound(task_id.0.clone()))
        }

        async fn cancel_task(&self, task_id: &GridTaskId) -> Result<(), GridError> {
            let mut map = self.tasks.lock().unwrap();
            if map.contains_key(&task_id.0) {
                map.insert(task_id.0.clone(), GridTaskStatus::Cancelled);
                Ok(())
            } else {
                Err(GridError::TaskNotFound(task_id.0.clone()))
            }
        }
    }

    // ── Mock IoTRepository (minimal, only needed for constructor) ───────────
    struct MockIoTRepo;

    #[async_trait]
    impl IoTRepository for MockIoTRepo {
        async fn create_reading(&self, _r: &IoTReading) -> Result<IoTReading, String> {
            Err("not impl".to_string())
        }
        async fn create_readings_bulk(&self, _r: &[IoTReading]) -> Result<usize, String> {
            Err("not impl".to_string())
        }
        async fn find_readings_by_building(
            &self, _b: Uuid, _dt: Option<DeviceType>, _mt: Option<MetricType>,
            _s: DateTime<Utc>, _e: DateTime<Utc>, _l: Option<usize>,
        ) -> Result<Vec<IoTReading>, String> {
            Ok(vec![])
        }
        async fn get_consumption_stats(
            &self, building_id: Uuid, mt: MetricType,
            s: DateTime<Utc>, e: DateTime<Utc>,
        ) -> Result<ConsumptionStatsDto, String> {
            Ok(ConsumptionStatsDto {
                building_id,
                metric_type: mt,
                period_start: s,
                period_end: e,
                total_consumption: 0.0,
                average_daily: 0.0,
                min_value: 0.0,
                max_value: 0.0,
                reading_count: 0,
                unit: "kWh".to_string(),
                source: "mock".to_string(),
            })
        }
        async fn get_daily_aggregates(
            &self, _b: Uuid, _dt: DeviceType, _mt: MetricType,
            _s: DateTime<Utc>, _e: DateTime<Utc>,
        ) -> Result<Vec<DailyAggregateDto>, String> {
            Ok(vec![])
        }
        async fn get_monthly_aggregates(
            &self, _b: Uuid, _dt: DeviceType, _mt: MetricType,
            _s: DateTime<Utc>, _e: DateTime<Utc>,
        ) -> Result<Vec<MonthlyAggregateDto>, String> {
            Ok(vec![])
        }
        async fn detect_anomalies(
            &self, _b: Uuid, _mt: MetricType, _t: f64, _d: i64,
        ) -> Result<Vec<IoTReading>, String> {
            Ok(vec![])
        }
        async fn create_linky_device(&self, _d: &LinkyDevice) -> Result<LinkyDevice, String> {
            Err("not impl".to_string())
        }
        async fn find_linky_device_by_id(&self, _id: Uuid) -> Result<Option<LinkyDevice>, String> {
            Ok(None)
        }
        async fn find_linky_device_by_building(&self, _b: Uuid) -> Result<Option<LinkyDevice>, String> {
            Ok(None)
        }
        async fn find_linky_device_by_prm(&self, _p: &str, _pr: &str) -> Result<Option<LinkyDevice>, String> {
            Ok(None)
        }
        async fn update_linky_device(&self, _d: &LinkyDevice) -> Result<LinkyDevice, String> {
            Err("not impl".to_string())
        }
        async fn delete_linky_device(&self, _id: Uuid) -> Result<(), String> {
            Ok(())
        }
        async fn find_devices_needing_sync(&self) -> Result<Vec<LinkyDevice>, String> {
            Ok(vec![])
        }
        async fn find_devices_with_expired_tokens(&self) -> Result<Vec<LinkyDevice>, String> {
            Ok(vec![])
        }
    }

    // ── Helpers ─────────────────────────────────────────────────────────────
    fn setup() -> (BoincUseCases, Arc<MockGridPort>) {
        let grid_port = Arc::new(MockGridPort::new());
        let iot_repo = Arc::new(MockIoTRepo);

        let uc = BoincUseCases::new(
            grid_port.clone() as Arc<dyn GridParticipationPort>,
            iot_repo as Arc<dyn IoTRepository>,
        );

        (uc, grid_port)
    }

    // ── Tests ───────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_grant_consent_success() {
        let (uc, _) = setup();
        let owner_id = Uuid::new_v4();
        let org_id = Uuid::new_v4();

        let result = uc.grant_consent(owner_id, org_id, Some("127.0.0.1")).await;
        assert!(result.is_ok());
        let consent = result.unwrap();
        assert!(consent.granted);
        assert_eq!(consent.owner_id, owner_id);
        assert_eq!(consent.organization_id, org_id);
        assert_eq!(consent.consent_version, "v1.0");
        assert_eq!(consent.consent_ip, Some("127.0.0.1".to_string()));
    }

    #[tokio::test]
    async fn test_get_consent_after_grant() {
        let (uc, _) = setup();
        let owner_id = Uuid::new_v4();
        let org_id = Uuid::new_v4();

        uc.grant_consent(owner_id, org_id, None).await.unwrap();

        let result = uc.get_consent(owner_id).await;
        assert!(result.is_ok());
        let consent = result.unwrap();
        assert!(consent.is_some());
        assert!(consent.unwrap().granted);
    }

    #[tokio::test]
    async fn test_get_consent_not_granted() {
        let (uc, _) = setup();
        let owner_id = Uuid::new_v4();

        let result = uc.get_consent(owner_id).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_revoke_consent_success() {
        let (uc, _) = setup();
        let owner_id = Uuid::new_v4();
        let org_id = Uuid::new_v4();

        uc.grant_consent(owner_id, org_id, None).await.unwrap();

        let result = uc.revoke_consent(owner_id).await;
        assert!(result.is_ok());

        // Check it's revoked
        let consent = uc.get_consent(owner_id).await.unwrap().unwrap();
        assert!(!consent.granted);
        assert!(consent.revoked_at.is_some());
    }

    #[tokio::test]
    async fn test_revoke_consent_not_granted_fails() {
        let (uc, _) = setup();
        let owner_id = Uuid::new_v4();

        let result = uc.revoke_consent(owner_id).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Consent not granted"));
    }

    #[tokio::test]
    async fn test_poll_task_queued() {
        let (uc, grid_port) = setup();

        // Manually insert a task in Queued state
        let task_id = "test-task-123".to_string();
        grid_port.tasks.lock().unwrap().insert(task_id.clone(), GridTaskStatus::Queued);

        let result = uc.poll_task(&task_id).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), GridTaskStatus::Queued);
    }

    #[tokio::test]
    async fn test_poll_task_not_found() {
        let (uc, _) = setup();
        let result = uc.poll_task("nonexistent-task").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Task not found"));
    }

    #[tokio::test]
    async fn test_cancel_task_success() {
        let (uc, grid_port) = setup();

        let task_id = "task-to-cancel".to_string();
        grid_port.tasks.lock().unwrap().insert(task_id.clone(), GridTaskStatus::Queued);

        let result = uc.cancel_task(&task_id).await;
        assert!(result.is_ok());

        // Verify it's cancelled
        let status = uc.poll_task(&task_id).await.unwrap();
        assert_eq!(status, GridTaskStatus::Cancelled);
    }

    #[tokio::test]
    async fn test_cancel_task_not_found() {
        let (uc, _) = setup();
        let result = uc.cancel_task("nonexistent").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Task not found"));
    }
}
