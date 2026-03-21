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
