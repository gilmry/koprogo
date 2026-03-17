use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Identifiant opaque d'une tâche BOINC soumise.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridTaskId(pub String);

/// Type de calcul batch délégué à BOINC.
/// IMPORTANT: Toutes les variantes sont anonymisées — pas de PII.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GridTaskKind {
    /// Optimisation achat groupé énergie (historique kWh anonymisé)
    EnergyGroupOptimisation {
        building_id: Uuid,
        /// JSON agrégé anonymisé (kWh par période, pas de données personnelles)
        anonymised_readings_json: String,
        simulation_months: u32,
    },
    /// Simulation thermique bâtiment (données météo + consommation agrégée)
    BuildingThermalSimulation {
        building_id: Uuid,
        insulation_score: f64,
        surface_m2: f64,
        heating_degree_days: f64,
    },
}

/// Tâche soumise à BOINC.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridTask {
    /// Référence interne KoproGo (UUID stocké en DB)
    pub internal_id: Uuid,
    /// Copropriété propriétaire (isolation multi-tenant)
    pub copropriete_id: Uuid,
    /// Organisation propriétaire
    pub organization_id: Uuid,
    /// Type de calcul
    pub kind: GridTaskKind,
    /// Priorité (0-10, défaut 5)
    pub priority: u8,
    /// Date limite pour le résultat (BOINC deadline)
    pub deadline: DateTime<Utc>,
}

/// Statut d'une tâche BOINC.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum GridTaskStatus {
    /// Soumise, en attente d'allocation
    Queued,
    /// Prise en charge par un worker BOINC
    Running { started_at: DateTime<Utc> },
    /// Terminée avec succès
    Completed {
        completed_at: DateTime<Utc>,
        /// JSON agrégé anonymisé
        result_json: String,
    },
    /// Échec (timeout, erreur worker, quota BOINC)
    Failed {
        failed_at: DateTime<Utc>,
        reason: String,
    },
    /// Annulée par l'opérateur
    Cancelled,
}

/// Consentement BOINC d'un propriétaire (GDPR Article 6.1.a).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoincConsent {
    pub owner_id: Uuid,
    pub organization_id: Uuid,
    pub granted: bool,
    pub granted_at: Option<DateTime<Utc>>,
    pub revoked_at: Option<DateTime<Utc>>,
    /// IP au moment du consentement (GDPR Article 30)
    pub consent_ip: Option<String>,
    /// Version de la clause de consentement acceptée
    pub consent_version: String,
}

/// Erreurs grid computing
#[derive(Debug, thiserror::Error)]
pub enum GridError {
    #[error("Consent not granted for owner {0}")]
    ConsentNotGranted(Uuid),

    #[error("BOINC RPC failed: {0}")]
    RpcFailed(String),

    #[error("Task not found: {0}")]
    TaskNotFound(String),

    #[error("K-anonymity constraint violated: minimum {min} participants required, got {got}")]
    KAnonymityViolated { min: usize, got: usize },

    #[error("Process error: {0}")]
    ProcessError(String),

    #[error("Result parse error: {0}")]
    ResultParseError(String),
}

/// Port pour la participation au calcul distribué BOINC.
/// Gère consentement GDPR + soumission/poll de tâches.
/// L'adapter `BoincGridAdapter` dans infrastructure/grid/ implémente ce trait.
///
/// GDPR: le consentement est explicite (Art. 6.1.a), révocable à tout moment (Art. 7.3).
/// K-anonymité: minimum 5 participants pour toute tâche de groupe.
#[async_trait]
pub trait GridParticipationPort: Send + Sync {
    // ── Consentement GDPR ────────────────────────────────────────────────────

    /// Vérifie si le propriétaire a consenti à la participation BOINC.
    async fn check_consent(&self, owner_id: Uuid) -> Result<bool, GridError>;

    /// Récupère les détails du consentement (pour affichage RGPD).
    async fn get_consent(&self, owner_id: Uuid) -> Result<Option<BoincConsent>, GridError>;

    /// Enregistre le consentement explicite (GDPR Article 7).
    async fn grant_consent(
        &self,
        owner_id: Uuid,
        organization_id: Uuid,
        consent_version: &str,
        consent_ip: Option<&str>,
    ) -> Result<BoincConsent, GridError>;

    /// Révoque le consentement (GDPR Article 7.3 - droit de retrait immédiat).
    async fn revoke_consent(&self, owner_id: Uuid) -> Result<(), GridError>;

    // ── Gestion des tâches ───────────────────────────────────────────────────

    /// Soumet une tâche de calcul au cluster BOINC.
    /// Pré-condition: check_consent() doit être true (vérifié par le use case).
    async fn submit_task(&self, task: GridTask) -> Result<GridTaskId, GridError>;

    /// Interroge le statut d'une tâche (polling).
    async fn poll_result(&self, task_id: &GridTaskId) -> Result<GridTaskStatus, GridError>;

    /// Annule une tâche en cours.
    async fn cancel_task(&self, task_id: &GridTaskId) -> Result<(), GridError>;
}
