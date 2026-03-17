use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::entities::iot_reading::{DeviceType, MetricType};

/// DTO pour les messages entrants depuis MQTT (Home Assistant → Mosquitto → KoproGo).
/// Correspond au JSON publié par Home Assistant automations.
///
/// Exemple de payload Home Assistant:
/// ```json
/// {
///   "value": 12.47,
///   "unit": "kWh",
///   "ts": "2026-03-17T14:00:00Z",
///   "device_type": "electricity_meter",
///   "metric_type": "electricity_consumption",
///   "quality": "good"
/// }
/// ```
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MqttIncomingReadingDto {
    /// Valeur mesurée
    pub value: f64,
    /// Unité (ex: "kWh", "°C", "%")
    pub unit: String,
    /// Timestamp ISO8601 émis par le capteur
    pub ts: DateTime<Utc>,
    /// Type d'appareil (serde snake_case)
    pub device_type: DeviceType,
    /// Type de métrique (serde snake_case)
    pub metric_type: MetricType,
    /// Qualité de la mesure ("good", "estimated", "bad")
    pub quality: Option<String>,
}

/// Erreurs spécifiques MQTT
#[derive(Debug, thiserror::Error)]
pub enum MqttError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Subscription failed on topic '{topic}': {reason}")]
    SubscriptionFailed { topic: String, reason: String },

    #[error("Payload parse error on topic '{topic}': {reason}")]
    PayloadParseError { topic: String, reason: String },

    #[error("Topic format invalid: {0}")]
    InvalidTopic(String),

    #[error("Already running")]
    AlreadyRunning,

    #[error("Not connected")]
    NotConnected,
}

/// Port (trait) définissant le contrat pour la couche MQTT.
/// Le domaine ne connaît pas rumqttc — seul ce trait existe dans l'application layer.
/// L'adapter `MqttEnergyAdapter` dans infrastructure/mqtt/ implémente ce trait.
///
/// Flux: capteur → Home Assistant → Mosquitto (TLS:8883) → MqttEnergyAdapter
///       → on_reading_received() → RegisterEnergyReadingFromIoTUseCase → DB
///
/// Topic pattern: koprogo/{copropriete_id}/energy/{unit_id}/{metric}
#[async_trait]
pub trait MqttEnergyPort: Send + Sync {
    /// Démarre l'écoute MQTT (connect + subscribe).
    /// Non-bloquant : lance un tokio::spawn en interne.
    async fn start_listening(&self) -> Result<(), MqttError>;

    /// Arrête proprement l'écoute MQTT.
    async fn stop_listening(&self) -> Result<(), MqttError>;

    /// Publie un message sortant (ex: alerte anomalie → Home Assistant).
    /// Topic: koprogo/{copropriete_id}/alerts/{alert_type}
    async fn publish_alert(
        &self,
        copropriete_id: Uuid,
        alert_type: &str,
        payload: &str,
    ) -> Result<(), MqttError>;

    /// Vérifie si le listener est actif.
    async fn is_running(&self) -> bool;
}
