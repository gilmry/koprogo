use crate::application::ports::iot_repository::IoTRepository;
use crate::application::ports::mqtt_energy_port::{
    MqttEnergyPort, MqttError, MqttIncomingReadingDto,
};
use crate::domain::entities::iot_reading::IoTReading;
use async_trait::async_trait;
use rumqttc::{AsyncClient, Event, MqttOptions, Packet, QoS};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tracing::{error, info, warn};
use uuid::Uuid;

/// Configuration MQTT chargée depuis les variables d'environnement.
///
/// Variables attendues:
/// - MQTT_HOST (défaut: "localhost")
/// - MQTT_PORT (défaut: 1883, TLS: 8883)
/// - MQTT_CLIENT_ID (défaut: "koprogo-backend")
/// - MQTT_USERNAME / MQTT_PASSWORD
/// - MQTT_TOPIC (défaut: "koprogo/+/energy/#")
#[derive(Debug, Clone)]
pub struct MqttConfig {
    pub host: String,
    pub port: u16,
    pub client_id: String,
    pub username: String,
    pub password: String,
    pub subscribe_topic: String,
}

impl MqttConfig {
    pub fn from_env() -> Self {
        Self {
            host: std::env::var("MQTT_HOST").unwrap_or_else(|_| "localhost".to_string()),
            port: std::env::var("MQTT_PORT")
                .unwrap_or_else(|_| "1883".to_string())
                .parse()
                .unwrap_or(1883),
            client_id: std::env::var("MQTT_CLIENT_ID")
                .unwrap_or_else(|_| "koprogo-backend".to_string()),
            username: std::env::var("MQTT_USERNAME").unwrap_or_default(),
            password: std::env::var("MQTT_PASSWORD").unwrap_or_default(),
            subscribe_topic: std::env::var("MQTT_TOPIC")
                .unwrap_or_else(|_| "koprogo/+/energy/#".to_string()),
        }
    }
}

/// Adapter MQTT: subscribe aux topics Home Assistant → dispatch vers IoTRepository.
///
/// Flux: Mosquitto broker → rumqttc EventLoop (tokio::spawn)
///       → parse topic (copropriete_id + unit_id) + deserialize JSON payload
///       → IoTReading::new() (validation domaine) → iot_repo.create_reading()
pub struct MqttEnergyAdapter {
    config: MqttConfig,
    iot_repo: Arc<dyn IoTRepository>,
    listener_handle: Arc<Mutex<Option<JoinHandle<()>>>>,
}

impl MqttEnergyAdapter {
    pub fn new(config: MqttConfig, iot_repo: Arc<dyn IoTRepository>) -> Self {
        Self {
            config,
            iot_repo,
            listener_handle: Arc::new(Mutex::new(None)),
        }
    }

    /// Parse le topic MQTT pour extraire copropriete_id et unit_id.
    ///
    /// Format attendu: koprogo/{copropriete_id}/energy/{unit_id}/{metric}
    /// Exemple: koprogo/550e8400-.../energy/aaaa-bbbb-.../electricity
    pub fn parse_topic(topic: &str) -> Result<(Uuid, Uuid), MqttError> {
        let parts: Vec<&str> = topic.split('/').collect();
        if parts.len() < 5 || parts[0] != "koprogo" || parts[2] != "energy" {
            return Err(MqttError::InvalidTopic(format!(
                "Expected 'koprogo/{{copropriete_id}}/energy/{{unit_id}}/{{metric}}', got: {}",
                topic
            )));
        }
        let copropriete_id = Uuid::parse_str(parts[1]).map_err(|_| {
            MqttError::InvalidTopic(format!("Invalid copropriete_id UUID: {}", parts[1]))
        })?;
        let unit_id = Uuid::parse_str(parts[3])
            .map_err(|_| MqttError::InvalidTopic(format!("Invalid unit_id UUID: {}", parts[3])))?;
        Ok((copropriete_id, unit_id))
    }
}

#[async_trait]
impl MqttEnergyPort for MqttEnergyAdapter {
    async fn start_listening(&self) -> Result<(), MqttError> {
        let mut handle = self.listener_handle.lock().await;
        if handle.is_some() {
            return Err(MqttError::AlreadyRunning);
        }

        let mut opts =
            MqttOptions::new(&self.config.client_id, &self.config.host, self.config.port);
        if !self.config.username.is_empty() {
            opts.set_credentials(&self.config.username, &self.config.password);
        }
        opts.set_keep_alive(std::time::Duration::from_secs(30));
        // Persistent session: QoS 1 survit aux reconnexions
        opts.set_clean_session(false);

        let (client, mut eventloop) = AsyncClient::new(opts, 128);

        client
            .subscribe(&self.config.subscribe_topic, QoS::AtLeastOnce)
            .await
            .map_err(|e| MqttError::SubscriptionFailed {
                topic: self.config.subscribe_topic.clone(),
                reason: e.to_string(),
            })?;

        let iot_repo = Arc::clone(&self.iot_repo);
        let topic_pattern = self.config.subscribe_topic.clone();

        let join_handle = tokio::spawn(async move {
            info!("MQTT listener started on topic: {}", topic_pattern);
            loop {
                match eventloop.poll().await {
                    Ok(Event::Incoming(Packet::Publish(msg))) => {
                        let topic = msg.topic.clone();
                        match Self::parse_topic(&topic) {
                            Ok((copropriete_id, _unit_id)) => {
                                match serde_json::from_slice::<MqttIncomingReadingDto>(&msg.payload)
                                {
                                    Ok(dto) => {
                                        match IoTReading::new(
                                            copropriete_id,
                                            dto.device_type,
                                            dto.metric_type,
                                            dto.value,
                                            dto.unit.clone(),
                                            dto.ts,
                                            "mqtt_home_assistant".to_string(),
                                        ) {
                                            Ok(reading) => {
                                                if let Err(e) =
                                                    iot_repo.create_reading(&reading).await
                                                {
                                                    error!(
                                                        "Failed to persist MQTT reading from {}: {}",
                                                        topic, e
                                                    );
                                                }
                                            }
                                            Err(e) => {
                                                warn!(
                                                    "Domain validation failed for MQTT reading from {}: {}",
                                                    topic, e
                                                );
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        warn!("Failed to parse MQTT payload on {}: {}", topic, e);
                                    }
                                }
                            }
                            Err(e) => {
                                warn!("Invalid MQTT topic {}: {}", topic, e);
                            }
                        }
                    }
                    Ok(Event::Incoming(Packet::ConnAck(_))) => {
                        info!("MQTT connected to broker");
                    }
                    Ok(Event::Incoming(Packet::Disconnect)) => {
                        warn!("MQTT disconnected, rumqttc will reconnect automatically");
                    }
                    Err(e) => {
                        error!("MQTT event loop error: {}", e);
                        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                    }
                    _ => {}
                }
            }
        });

        *handle = Some(join_handle);
        info!(
            "MQTT listener spawned for topic pattern: {}",
            self.config.subscribe_topic
        );
        Ok(())
    }

    async fn stop_listening(&self) -> Result<(), MqttError> {
        let mut handle = self.listener_handle.lock().await;
        if let Some(h) = handle.take() {
            h.abort();
            info!("MQTT listener stopped");
        }
        Ok(())
    }

    async fn publish_alert(
        &self,
        _copropriete_id: Uuid,
        alert_type: &str,
        _payload: &str,
    ) -> Result<(), MqttError> {
        // TODO Phase 2: garder un Arc<Mutex<AsyncClient>> pour publier depuis le listener
        info!("MQTT alert publish queued (not yet wired): {}", alert_type);
        Ok(())
    }

    async fn is_running(&self) -> bool {
        self.listener_handle.lock().await.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_topic_valid() {
        let copropriete_id = Uuid::new_v4();
        let unit_id = Uuid::new_v4();
        let topic = format!("koprogo/{}/energy/{}/electricity", copropriete_id, unit_id);
        let (parsed_copro, parsed_unit) = MqttEnergyAdapter::parse_topic(&topic).unwrap();
        assert_eq!(parsed_copro, copropriete_id);
        assert_eq!(parsed_unit, unit_id);
    }

    #[test]
    fn test_parse_topic_invalid_prefix() {
        let topic = "other/abc/energy/def/electricity";
        assert!(MqttEnergyAdapter::parse_topic(topic).is_err());
    }

    #[test]
    fn test_parse_topic_missing_energy_segment() {
        let topic = "koprogo/abc-def/readings/xyz/power";
        assert!(MqttEnergyAdapter::parse_topic(topic).is_err());
    }

    #[test]
    fn test_parse_topic_invalid_uuid() {
        let topic = "koprogo/not-a-uuid/energy/also-not-a-uuid/electricity";
        assert!(MqttEnergyAdapter::parse_topic(topic).is_err());
    }

    #[test]
    fn test_parse_topic_too_short() {
        let topic = "koprogo/abc/energy";
        assert!(MqttEnergyAdapter::parse_topic(topic).is_err());
    }

    #[test]
    fn test_mqtt_config_defaults() {
        // Efface les éventuelles variables d'env pour tester les défauts
        std::env::remove_var("MQTT_HOST");
        std::env::remove_var("MQTT_PORT");
        std::env::remove_var("MQTT_CLIENT_ID");
        std::env::remove_var("MQTT_TOPIC");

        let config = MqttConfig::from_env();
        assert_eq!(config.host, "localhost");
        assert_eq!(config.port, 1883);
        assert_eq!(config.client_id, "koprogo-backend");
        assert_eq!(config.subscribe_topic, "koprogo/+/energy/#");
    }
}
