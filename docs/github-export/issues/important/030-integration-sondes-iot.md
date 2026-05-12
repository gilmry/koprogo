# Issue #030: Intégration Sondes IoT (Monitoring Temps Réel)

**Priorité**: Haute
**Effort estimé**: 18-24h
**Phase**: Phase 3 (K8s - Real-time & Performance)
**Dépendances**: Issue #029 (optionnel - validation croisée ISTA), Infrastructure WebSocket/SSE
**Labels**: `feature`, `iot`, `real-time`, `monitoring`, `energy`

---

## 📋 Contexte

Les **capteurs IoT** permettent le monitoring en temps réel des consommations énergétiques et hydriques dans les copropriétés. En Belgique, plusieurs fournisseurs proposent des solutions IoT pour immeubles:
- **Kamstrup** (compteurs intelligents eau/chaleur)
- **Siemens Building Technologies** (BMS - Building Management Systems)
- **LoRaWAN** (réseaux bas débit pour capteurs)
- **Shelly** (capteurs électriques Wi-Fi/MQTT)

**Objectif**: Intégrer les remontées de capteurs IoT dans KoproGo pour:
1. **Monitoring temps réel** des consommations (eau froide, eau chaude, gaz, électricité, cogénération)
2. **Détection immédiate** d'anomalies (fuites, surconsommation, pannes)
3. **Alertes automatiques** (SMS, email, push notifications)
4. **Validation croisée** avec relevés ISTA (Issue #029)
5. **Optimisation énergétique** (identifier gaspillages, recommandations)
6. **Tableaux de bord temps réel** pour syndics et copropriétaires

---

## 🎯 Objectifs

### Fonctionnels
- ✅ Recevoir les données de capteurs IoT via **MQTT** (protocole standard IoT)
- ✅ Supporter plusieurs types de capteurs (eau froide, eau chaude, gaz, électricité, cogénération)
- ✅ Stocker l'historique des mesures avec **time-series database** (TimescaleDB extension PostgreSQL)
- ✅ Détecter les anomalies en temps réel (règles configurables par type de capteur)
- ✅ Envoyer des alertes instantanées (email, SMS, push) en cas d'anomalie
- ✅ Calculer des statistiques agrégées (consommation horaire, journalière, mensuelle)
- ✅ Exposer les données temps réel via **WebSocket** pour dashboards frontend
- ✅ Valider la cohérence avec les relevés ISTA (écart < 5%)
- ✅ Gérer le provisioning et la configuration des capteurs (ajout, suppression, calibration)

### Techniques
- ✅ Architecture événementielle avec **MQTT broker** (Mosquitto ou EMQX)
- ✅ **TimescaleDB** pour stockage optimisé time-series (compression automatique)
- ✅ **Actix WebSocket** pour streaming temps réel vers frontend
- ✅ **Redis Streams** pour buffering et résilience (si MQTT broker down)
- ✅ **Rule engine** configurable pour détection d'anomalies (seuils, tendances, ML)
- ✅ **Multi-tenancy strict** (isolation des données par organization_id)
- ✅ **Scalabilité horizontale** (traitement distribué avec K8s StatefulSets)
- ✅ Latence cible: **< 500ms** entre mesure capteur et affichage dashboard

---

## 🏗️ Architecture Technique

### 1. Architecture Globale

```
┌─────────────────┐
│  Capteurs IoT   │ (Kamstrup, Siemens, Shelly, LoRaWAN)
│  - Eau froide   │
│  - Eau chaude   │
│  - Gaz          │
│  - Électricité  │
│  - Cogénération │
└────────┬────────┘
         │ MQTT (TLS)
         ▼
┌─────────────────────────────┐
│   MQTT Broker (Mosquitto)   │
│   Topic: koprogo/{org}/{building}/{sensor_id}/readings
└────────┬────────────────────┘
         │ Subscribe
         ▼
┌──────────────────────────────────────┐
│  IoT Ingestion Service (Rust)        │
│  - Validation des messages           │
│  - Détection anomalies temps réel    │
│  - Persistance TimescaleDB           │
│  - Buffering Redis Streams           │
│  - Envoi alertes (email/SMS)         │
└────────┬─────────────────────────────┘
         │
         ├──────────────────┬──────────────────┐
         ▼                  ▼                  ▼
┌──────────────┐   ┌─────────────┐   ┌──────────────────┐
│ TimescaleDB  │   │ Redis       │   │ Alert Service    │
│ (time-series)│   │ (buffer)    │   │ (email/SMS/push) │
└──────────────┘   └─────────────┘   └──────────────────┘
         │
         │ Query API
         ▼
┌─────────────────────────────┐
│  KoproGo Backend (Actix)    │
│  - REST API (historique)    │
│  - WebSocket (temps réel)   │
└────────┬────────────────────┘
         │ WebSocket
         ▼
┌─────────────────────────────┐
│  Frontend (Svelte)          │
│  - Dashboard temps réel     │
│  - Graphiques (Chart.js)    │
│  - Alertes & notifications  │
└─────────────────────────────┘
```

### 2. Nouvelles Entités Domain

#### `IoTSensor` (Capteur IoT)
```rust
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Représente un capteur IoT installé dans un bâtiment.
pub struct IoTSensor {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub building_id: Uuid,
    pub unit_id: Option<Uuid>, // None si capteur commun (ex: compteur général bâtiment)

    // Identification
    pub sensor_id: String, // ID unique fourni par le fabricant (ex: MAC address, serial number)
    pub sensor_type: SensorType,
    pub manufacturer: String, // "Kamstrup", "Siemens", "Shelly", etc.
    pub model: String,
    pub firmware_version: Option<String>,

    // Localisation
    pub location: String, // "Sous-sol compteur général", "Appartement 101 cuisine", etc.
    pub floor: Option<i32>,

    // Configuration
    pub unit_of_measure: String, // "m³", "kWh", "L", "W", etc.
    pub sampling_interval: i32, // Intervalle de mesure en secondes (ex: 300 = 5 min)
    pub calibration_factor: f64, // Facteur de calibration (default: 1.0)

    // MQTT
    pub mqtt_topic: String, // "koprogo/{org_id}/{building_id}/{sensor_id}/readings"

    // Anomaly detection config
    pub alert_threshold_min: Option<f64>, // Seuil min (ex: débit minimum attendu)
    pub alert_threshold_max: Option<f64>, // Seuil max (ex: pic de consommation anormal)
    pub alert_enabled: bool,

    // Statut
    pub status: SensorStatus, // Active, Inactive, Maintenance, Faulty
    pub last_reading_at: Option<DateTime<Utc>>,
    pub last_battery_level: Option<f64>, // Niveau batterie (0-100%)

    pub installed_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SensorType {
    ColdWater,      // Eau froide
    HotWater,       // Eau chaude sanitaire
    Gas,            // Gaz naturel
    Electricity,    // Électricité
    Cogeneration,   // Cogénération (production combinée chaleur-électricité)
    Heating,        // Chauffage (température)
    Temperature,    // Température ambiante
    Humidity,       // Humidité
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SensorStatus {
    Active,      // En service
    Inactive,    // Désactivé (temporaire)
    Maintenance, // En maintenance
    Faulty,      // Défaillant
}

impl IoTSensor {
    pub fn new(
        organization_id: Uuid,
        building_id: Uuid,
        sensor_id: String,
        sensor_type: SensorType,
        manufacturer: String,
        model: String,
        location: String,
        unit_of_measure: String,
        sampling_interval: i32,
    ) -> Result<Self, String> {
        // Validations
        if sensor_id.trim().is_empty() {
            return Err("Sensor ID cannot be empty".to_string());
        }
        if sampling_interval < 10 || sampling_interval > 86400 {
            return Err("Sampling interval must be between 10s and 86400s (24h)".to_string());
        }

        let mqtt_topic = format!(
            "koprogo/{}/{}/{}/readings",
            organization_id, building_id, sensor_id
        );

        Ok(Self {
            id: Uuid::new_v4(),
            organization_id,
            building_id,
            unit_id: None,
            sensor_id,
            sensor_type,
            manufacturer,
            model,
            firmware_version: None,
            location,
            floor: None,
            unit_of_measure,
            sampling_interval,
            calibration_factor: 1.0,
            mqtt_topic,
            alert_threshold_min: None,
            alert_threshold_max: None,
            alert_enabled: false,
            status: SensorStatus::Active,
            last_reading_at: None,
            last_battery_level: None,
            installed_at: Utc::now(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }

    /// Configure les seuils d'alerte
    pub fn set_alert_thresholds(&mut self, min: Option<f64>, max: Option<f64>) {
        self.alert_threshold_min = min;
        self.alert_threshold_max = max;
        self.alert_enabled = min.is_some() || max.is_some();
        self.updated_at = Utc::now();
    }

    /// Vérifie si une valeur déclenche une alerte
    pub fn check_alert(&self, value: f64) -> Option<AlertReason> {
        if !self.alert_enabled {
            return None;
        }

        if let Some(min) = self.alert_threshold_min {
            if value < min {
                return Some(AlertReason::BelowThreshold { value, threshold: min });
            }
        }

        if let Some(max) = self.alert_threshold_max {
            if value > max {
                return Some(AlertReason::AboveThreshold { value, threshold: max });
            }
        }

        None
    }
}

#[derive(Debug, Clone)]
pub enum AlertReason {
    BelowThreshold { value: f64, threshold: f64 },
    AboveThreshold { value: f64, threshold: f64 },
    NoDataReceived { duration_hours: i32 },
    SensorFaulty,
}
```

#### `IoTReading` (Mesure capteur)
```rust
/// Représente une mesure individuelle d'un capteur IoT.
/// Stocké dans TimescaleDB (hypertable optimisée time-series).
pub struct IoTReading {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub sensor_id: Uuid, // FK vers IoTSensor
    pub building_id: Uuid,
    pub unit_id: Option<Uuid>,

    // Mesure
    pub timestamp: DateTime<Utc>, // Timestamp de la mesure (fourni par capteur)
    pub value: f64, // Valeur mesurée
    pub unit_of_measure: String,

    // Métadonnées
    pub battery_level: Option<f64>, // Niveau batterie au moment de la mesure
    pub signal_strength: Option<i32>, // Force du signal (RSSI en dBm)
    pub quality: ReadingQuality, // Good, Warning, Poor

    // Flags
    pub is_anomaly: bool, // true si détecté comme anormal
    pub anomaly_reason: Option<String>,

    pub received_at: DateTime<Utc>, // Timestamp de réception par le serveur
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReadingQuality {
    Good,    // Mesure fiable
    Warning, // Mesure douteuse (batterie faible, signal faible)
    Poor,    // Mesure non fiable
}

impl IoTReading {
    pub fn new(
        organization_id: Uuid,
        sensor_id: Uuid,
        building_id: Uuid,
        timestamp: DateTime<Utc>,
        value: f64,
        unit_of_measure: String,
    ) -> Result<Self, String> {
        if value < 0.0 {
            return Err("Reading value cannot be negative".to_string());
        }

        Ok(Self {
            id: Uuid::new_v4(),
            organization_id,
            sensor_id,
            building_id,
            unit_id: None,
            timestamp,
            value,
            unit_of_measure,
            battery_level: None,
            signal_strength: None,
            quality: ReadingQuality::Good,
            is_anomaly: false,
            anomaly_reason: None,
            received_at: Utc::now(),
        })
    }

    /// Évalue la qualité de la mesure
    pub fn evaluate_quality(&mut self) {
        if let Some(battery) = self.battery_level {
            if battery < 10.0 {
                self.quality = ReadingQuality::Poor;
                return;
            } else if battery < 20.0 {
                self.quality = ReadingQuality::Warning;
            }
        }

        if let Some(rssi) = self.signal_strength {
            if rssi < -100 { // Signal très faible
                self.quality = ReadingQuality::Poor;
                return;
            } else if rssi < -80 {
                self.quality = ReadingQuality::Warning;
            }
        }
    }
}
```

#### `IoTAlert` (Alerte déclenchée)
```rust
/// Représente une alerte déclenchée par un capteur IoT.
pub struct IoTAlert {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub sensor_id: Uuid,
    pub building_id: Uuid,
    pub unit_id: Option<Uuid>,

    // Alerte
    pub alert_type: AlertType,
    pub severity: AlertSeverity,
    pub message: String,
    pub triggered_at: DateTime<Utc>,

    // Contexte
    pub reading_value: Option<f64>, // Valeur ayant déclenché l'alerte
    pub threshold_value: Option<f64>,

    // Notifications envoyées
    pub email_sent: bool,
    pub sms_sent: bool,
    pub push_sent: bool,

    // Résolution
    pub acknowledged: bool,
    pub acknowledged_by: Option<Uuid>, // User ID
    pub acknowledged_at: Option<DateTime<Utc>>,
    pub resolution_note: Option<String>,

    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlertType {
    ThresholdExceeded,   // Seuil dépassé
    AnomalyDetected,     // Anomalie statistique détectée
    SensorOffline,       // Capteur hors ligne (pas de données depuis X heures)
    LowBattery,          // Batterie faible
    SensorFaulty,        // Capteur défaillant
    LeakDetected,        // Fuite détectée (consommation continue anormale)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlertSeverity {
    Info,     // Informatif
    Warning,  // Avertissement
    Critical, // Critique (intervention requise)
}
```

---

### 3. IoT Ingestion Service (Service indépendant)

#### Service principal
```rust
// backend/src/iot_ingestion/main.rs

use rumqttc::{AsyncClient, MqttOptions, QoS, Event, Packet};
use tokio::time::{interval, Duration};
use sqlx::PgPool;

/// Service d'ingestion des messages MQTT des capteurs IoT.
/// Tourne en parallèle du backend principal (microservice).
pub struct IoTIngestionService {
    mqtt_client: AsyncClient,
    db_pool: PgPool,
    redis_client: redis::Client,
    sensor_repo: Arc<dyn IoTSensorRepository>,
    reading_repo: Arc<dyn IoTReadingRepository>,
    alert_repo: Arc<dyn IoTAlertRepository>,
    alert_service: Arc<AlertService>,
}

impl IoTIngestionService {
    pub async fn new(config: IoTConfig) -> Result<Self, String> {
        // Configure MQTT client
        let mut mqtt_options = MqttOptions::new(
            "koprogo-iot-ingestion",
            &config.mqtt_broker_host,
            config.mqtt_broker_port,
        );
        mqtt_options.set_keep_alive(Duration::from_secs(60));
        mqtt_options.set_credentials(&config.mqtt_username, &config.mqtt_password);

        // TLS si activé
        if config.mqtt_use_tls {
            mqtt_options.set_transport(rumqttc::Transport::tls_with_config(
                rumqttc::TlsConfiguration::Simple {
                    ca: config.mqtt_ca_cert.into(),
                    alpn: None,
                    client_auth: None,
                }
            ));
        }

        let (mqtt_client, mut eventloop) = AsyncClient::new(mqtt_options, 100);

        // Subscribe to all topics: koprogo/+/+/+/readings
        mqtt_client.subscribe("koprogo/+/+/+/readings", QoS::AtLeastOnce).await?;

        // Connect DB
        let db_pool = PgPool::connect(&config.database_url).await?;

        // Connect Redis
        let redis_client = redis::Client::open(config.redis_url)?;

        Ok(Self {
            mqtt_client,
            db_pool,
            redis_client,
            sensor_repo: Arc::new(PostgresIoTSensorRepository::new(db_pool.clone())),
            reading_repo: Arc::new(TimescaleIoTReadingRepository::new(db_pool.clone())),
            alert_repo: Arc::new(PostgresIoTAlertRepository::new(db_pool.clone())),
            alert_service: Arc::new(AlertService::new(/* email, SMS services */)),
        })
    }

    /// Démarre le service (boucle infinie)
    pub async fn run(&mut self) -> Result<(), String> {
        println!("🚀 IoT Ingestion Service started");

        loop {
            match self.eventloop.poll().await {
                Ok(Event::Incoming(Packet::Publish(publish))) => {
                    // Parse topic: koprogo/{org_id}/{building_id}/{sensor_id}/readings
                    let topic_parts: Vec<&str> = publish.topic.split('/').collect();
                    if topic_parts.len() != 5 {
                        eprintln!("❌ Invalid topic format: {}", publish.topic);
                        continue;
                    }

                    let org_id = Uuid::parse_str(topic_parts[1]).ok();
                    let building_id = Uuid::parse_str(topic_parts[2]).ok();
                    let sensor_id_str = topic_parts[3];

                    if org_id.is_none() || building_id.is_none() {
                        eprintln!("❌ Invalid UUIDs in topic: {}", publish.topic);
                        continue;
                    }

                    // Parse payload JSON
                    let payload: MqttReadingPayload = match serde_json::from_slice(&publish.payload) {
                        Ok(p) => p,
                        Err(e) => {
                            eprintln!("❌ JSON parsing error: {}", e);
                            continue;
                        }
                    };

                    // Process reading
                    if let Err(e) = self.process_reading(
                        org_id.unwrap(),
                        building_id.unwrap(),
                        sensor_id_str,
                        payload,
                    ).await {
                        eprintln!("❌ Error processing reading: {}", e);
                    }
                }
                Ok(_) => {}
                Err(e) => {
                    eprintln!("❌ MQTT error: {}", e);
                    tokio::time::sleep(Duration::from_secs(5)).await; // Retry delay
                }
            }
        }
    }

    /// Traite une mesure reçue
    async fn process_reading(
        &self,
        org_id: Uuid,
        building_id: Uuid,
        sensor_id_str: &str,
        payload: MqttReadingPayload,
    ) -> Result<(), String> {
        // 1. Récupérer le capteur en DB
        let sensor = self.sensor_repo
            .find_by_sensor_id(org_id, sensor_id_str)
            .await?
            .ok_or_else(|| format!("Sensor not found: {}", sensor_id_str))?;

        // 2. Créer IoTReading
        let mut reading = IoTReading::new(
            org_id,
            sensor.id,
            building_id,
            payload.timestamp,
            payload.value * sensor.calibration_factor, // Appliquer calibration
            sensor.unit_of_measure.clone(),
        )?;

        reading.unit_id = sensor.unit_id;
        reading.battery_level = payload.battery_level;
        reading.signal_strength = payload.signal_strength;
        reading.evaluate_quality();

        // 3. Détection d'anomalie
        if let Some(alert_reason) = sensor.check_alert(reading.value) {
            reading.is_anomaly = true;
            reading.anomaly_reason = Some(format!("{:?}", alert_reason));

            // Créer alerte
            let alert = IoTAlert {
                id: Uuid::new_v4(),
                organization_id: org_id,
                sensor_id: sensor.id,
                building_id,
                unit_id: sensor.unit_id,
                alert_type: AlertType::ThresholdExceeded,
                severity: AlertSeverity::Warning,
                message: format!(
                    "Alerte capteur {}: {} {} (seuil dépassé)",
                    sensor.location, reading.value, reading.unit_of_measure
                ),
                triggered_at: Utc::now(),
                reading_value: Some(reading.value),
                threshold_value: sensor.alert_threshold_max.or(sensor.alert_threshold_min),
                email_sent: false,
                sms_sent: false,
                push_sent: false,
                acknowledged: false,
                acknowledged_by: None,
                acknowledged_at: None,
                resolution_note: None,
                created_at: Utc::now(),
            };

            // Persister alerte
            self.alert_repo.create(&alert).await?;

            // Envoyer notifications (async)
            tokio::spawn({
                let alert_service = self.alert_service.clone();
                let alert_clone = alert.clone();
                async move {
                    if let Err(e) = alert_service.send_notifications(&alert_clone).await {
                        eprintln!("❌ Failed to send alert notifications: {}", e);
                    }
                }
            });
        }

        // 4. Persister reading dans TimescaleDB
        self.reading_repo.create(&reading).await?;

        // 5. Buffer dans Redis Streams pour WebSocket
        let mut redis_conn = self.redis_client.get_async_connection().await
            .map_err(|e| format!("Redis error: {}", e))?;

        redis::cmd("XADD")
            .arg(format!("iot_readings:{}", org_id))
            .arg("MAXLEN")
            .arg("~") // Approximative trimming
            .arg(10000) // Keep last 10k readings
            .arg("*") // Auto-generate ID
            .arg("sensor_id").arg(sensor.id.to_string())
            .arg("value").arg(reading.value)
            .arg("timestamp").arg(reading.timestamp.to_rfc3339())
            .query_async(&mut redis_conn)
            .await
            .map_err(|e| format!("Redis XADD error: {}", e))?;

        println!("✅ Reading processed: sensor={}, value={} {}",
                 sensor.location, reading.value, reading.unit_of_measure);

        Ok(())
    }
}

#[derive(Debug, Deserialize)]
struct MqttReadingPayload {
    pub timestamp: DateTime<Utc>,
    pub value: f64,
    pub battery_level: Option<f64>,
    pub signal_strength: Option<i32>,
}
```

---

### 4. API Endpoints

#### Routes
```rust
// backend/src/infrastructure/web/routes.rs

cfg.service(
    web::scope("/api/v1")
        // Sensors management
        .service(create_sensor)
        .service(list_sensors)
        .service(get_sensor)
        .service(update_sensor)
        .service(delete_sensor)
        .service(calibrate_sensor)

        // Readings
        .service(list_readings)
        .service(get_readings_time_series)
        .service(get_consumption_aggregates)

        // Alerts
        .service(list_alerts)
        .service(acknowledge_alert)
        .service(get_alert_statistics)

        // Real-time WebSocket
        .service(websocket_iot_stream)
);
```

#### Handlers
```rust
// backend/src/infrastructure/web/handlers/iot_handlers.rs

/// Créer un capteur IoT
#[post("/iot/sensors")]
pub async fn create_sensor(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    dto: web::Json<CreateIoTSensorDto>,
) -> impl Responder {
    // Role: Syndic, SuperAdmin
    if !matches!(user.role.as_str(), "syndic" | "superadmin") {
        return HttpResponse::Forbidden().json(ErrorResponse {
            error: "Insufficient permissions".to_string(),
        });
    }

    match state.iot_use_cases.create_sensor(user.organization_id, dto.into_inner()).await {
        Ok(sensor) => HttpResponse::Created().json(sensor),
        Err(e) => HttpResponse::BadRequest().json(ErrorResponse { error: e }),
    }
}

/// Récupérer time-series pour un capteur
#[get("/iot/sensors/{sensor_id}/time-series")]
pub async fn get_readings_time_series(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    path: web::Path<String>,
    query: web::Query<TimeSeriesQuery>,
) -> impl Responder {
    let sensor_id = Uuid::parse_str(&path.into_inner()).unwrap();

    match state.iot_use_cases.get_time_series(
        user.organization_id,
        sensor_id,
        query.start,
        query.end,
        query.aggregation_interval.unwrap_or(300), // Default 5 min
    ).await {
        Ok(series) => HttpResponse::Ok().json(series),
        Err(e) => HttpResponse::InternalServerError().json(ErrorResponse { error: e }),
    }
}

#[derive(Deserialize)]
pub struct TimeSeriesQuery {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub aggregation_interval: Option<i32>, // Secondes
}

/// WebSocket pour streaming temps réel
#[get("/iot/stream")]
pub async fn websocket_iot_stream(
    req: HttpRequest,
    stream: web::Payload,
    state: web::Data<AppState>,
    user: AuthenticatedUser,
) -> Result<HttpResponse, actix_web::Error> {
    // Upgrade to WebSocket
    ws::start(
        IoTWebSocket::new(user.organization_id, state.redis_client.clone()),
        &req,
        stream,
    )
}

struct IoTWebSocket {
    organization_id: Uuid,
    redis_client: redis::Client,
}

impl IoTWebSocket {
    fn new(organization_id: Uuid, redis_client: redis::Client) -> Self {
        Self { organization_id, redis_client }
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for IoTWebSocket {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Text(text)) => {
                // Client peut subscribe à des sensors spécifiques
                // Format: {"action": "subscribe", "sensor_ids": ["uuid1", "uuid2"]}
            }
            _ => {}
        }
    }

    fn started(&mut self, ctx: &mut Self::Context) {
        // Stream Redis readings to WebSocket client
        let org_id = self.organization_id;
        let redis_client = self.redis_client.clone();

        ctx.run_interval(Duration::from_secs(1), move |act, ctx| {
            // Read from Redis Streams and send to WebSocket
            tokio::spawn(async move {
                // XREAD from Redis Stream
                // Send readings via ctx.text()
            });
        });
    }
}
```

---

### 5. TimescaleDB Configuration

#### Migration SQL
```sql
-- backend/migrations/20250XXX_create_iot_tables.sql

-- Activer l'extension TimescaleDB
CREATE EXTENSION IF NOT EXISTS timescaledb CASCADE;

CREATE TYPE sensor_type AS ENUM (
    'cold_water', 'hot_water', 'gas', 'electricity',
    'cogeneration', 'heating', 'temperature', 'humidity'
);

CREATE TYPE sensor_status AS ENUM ('active', 'inactive', 'maintenance', 'faulty');
CREATE TYPE reading_quality AS ENUM ('good', 'warning', 'poor');
CREATE TYPE alert_type AS ENUM (
    'threshold_exceeded', 'anomaly_detected', 'sensor_offline',
    'low_battery', 'sensor_faulty', 'leak_detected'
);
CREATE TYPE alert_severity AS ENUM ('info', 'warning', 'critical');

-- Table capteurs
CREATE TABLE iot_sensors (
    id UUID PRIMARY KEY,
    organization_id UUID NOT NULL REFERENCES organizations(id),
    building_id UUID NOT NULL REFERENCES buildings(id),
    unit_id UUID REFERENCES units(id),

    sensor_id VARCHAR(255) NOT NULL, -- ID capteur fabricant
    sensor_type sensor_type NOT NULL,
    manufacturer VARCHAR(100) NOT NULL,
    model VARCHAR(100) NOT NULL,
    firmware_version VARCHAR(50),

    location VARCHAR(255) NOT NULL,
    floor INTEGER,

    unit_of_measure VARCHAR(20) NOT NULL,
    sampling_interval INTEGER NOT NULL DEFAULT 300, -- 5 min
    calibration_factor DOUBLE PRECISION NOT NULL DEFAULT 1.0,

    mqtt_topic VARCHAR(500) NOT NULL,

    alert_threshold_min DOUBLE PRECISION,
    alert_threshold_max DOUBLE PRECISION,
    alert_enabled BOOLEAN NOT NULL DEFAULT false,

    status sensor_status NOT NULL DEFAULT 'active',
    last_reading_at TIMESTAMP WITH TIME ZONE,
    last_battery_level DOUBLE PRECISION,

    installed_at TIMESTAMP WITH TIME ZONE NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),

    UNIQUE(organization_id, sensor_id)
);

-- Table mesures (hypertable TimescaleDB)
CREATE TABLE iot_readings (
    id UUID NOT NULL,
    organization_id UUID NOT NULL,
    sensor_id UUID NOT NULL REFERENCES iot_sensors(id) ON DELETE CASCADE,
    building_id UUID NOT NULL,
    unit_id UUID,

    timestamp TIMESTAMP WITH TIME ZONE NOT NULL, -- Colonne partitionnement
    value DOUBLE PRECISION NOT NULL,
    unit_of_measure VARCHAR(20) NOT NULL,

    battery_level DOUBLE PRECISION,
    signal_strength INTEGER,
    quality reading_quality NOT NULL DEFAULT 'good',

    is_anomaly BOOLEAN NOT NULL DEFAULT false,
    anomaly_reason TEXT,

    received_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),

    PRIMARY KEY (timestamp, id) -- Composite key pour TimescaleDB
);

-- Convertir en hypertable (time-series optimisé)
SELECT create_hypertable('iot_readings', 'timestamp', chunk_time_interval => INTERVAL '1 day');

-- Compression automatique après 7 jours
ALTER TABLE iot_readings SET (
    timescaledb.compress,
    timescaledb.compress_segmentby = 'sensor_id, organization_id'
);

SELECT add_compression_policy('iot_readings', INTERVAL '7 days');

-- Rétention automatique: supprimer données > 2 ans
SELECT add_retention_policy('iot_readings', INTERVAL '2 years');

-- Table alertes
CREATE TABLE iot_alerts (
    id UUID PRIMARY KEY,
    organization_id UUID NOT NULL REFERENCES organizations(id),
    sensor_id UUID NOT NULL REFERENCES iot_sensors(id) ON DELETE CASCADE,
    building_id UUID NOT NULL,
    unit_id UUID,

    alert_type alert_type NOT NULL,
    severity alert_severity NOT NULL,
    message TEXT NOT NULL,
    triggered_at TIMESTAMP WITH TIME ZONE NOT NULL,

    reading_value DOUBLE PRECISION,
    threshold_value DOUBLE PRECISION,

    email_sent BOOLEAN NOT NULL DEFAULT false,
    sms_sent BOOLEAN NOT NULL DEFAULT false,
    push_sent BOOLEAN NOT NULL DEFAULT false,

    acknowledged BOOLEAN NOT NULL DEFAULT false,
    acknowledged_by UUID REFERENCES users(id),
    acknowledged_at TIMESTAMP WITH TIME ZONE,
    resolution_note TEXT,

    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Index pour performance
CREATE INDEX idx_iot_sensors_org_building ON iot_sensors(organization_id, building_id);
CREATE INDEX idx_iot_sensors_status ON iot_sensors(status) WHERE status = 'active';
CREATE INDEX idx_iot_readings_sensor ON iot_readings(sensor_id, timestamp DESC);
CREATE INDEX idx_iot_readings_building ON iot_readings(building_id, timestamp DESC);
CREATE INDEX idx_iot_readings_anomaly ON iot_readings(sensor_id, timestamp DESC) WHERE is_anomaly = true;
CREATE INDEX idx_iot_alerts_unacked ON iot_alerts(organization_id, triggered_at DESC) WHERE acknowledged = false;
```

---

### 6. Frontend

#### Pages Svelte
- **`IoTDashboard.svelte`**: Dashboard temps réel avec graphiques live (Chart.js Streaming)
- **`SensorManagementPage.svelte`**: Liste et configuration des capteurs
- **`AlertsPage.svelte`**: Centre d'alertes avec filtres et acknowledgment
- **`ConsumptionAnalyticsPage.svelte`**: Analyses historiques et prévisions
- **`SensorDetailsPage.svelte`**: Détails d'un capteur (historique, config, calibration)

#### Composants
- **`LiveChart.svelte`**: Graphique temps réel avec WebSocket
- **`SensorCard.svelte`**: Carte capteur (status, dernière mesure, batterie)
- **`AlertNotification.svelte`**: Toast notifications pour alertes critiques
- **`SensorConfigForm.svelte`**: Formulaire config capteur (seuils, calibration)
- **`ConsumptionComparison.svelte`**: Comparaison conso entre unités/périodes

---

## 🧪 Tests

### Tests Unitaires
```rust
#[test]
fn test_sensor_alert_threshold_check() {
    let mut sensor = IoTSensor::new(/* ... */).unwrap();
    sensor.set_alert_thresholds(Some(10.0), Some(100.0));

    // Test dépassement max
    assert!(sensor.check_alert(150.0).is_some());

    // Test sous seuil min
    assert!(sensor.check_alert(5.0).is_some());

    // Test valeur normale
    assert!(sensor.check_alert(50.0).is_none());
}
```

### Tests d'Intégration
```rust
#[tokio::test]
async fn test_mqtt_reading_ingestion() {
    // 1. Créer capteur en DB
    // 2. Publier message MQTT simulé
    // 3. Vérifier que reading est persisté dans TimescaleDB
    // 4. Vérifier que alerte est créée si seuil dépassé
}

#[tokio::test]
async fn test_timescaledb_aggregation() {
    // 1. Insérer 1000 readings sur 24h
    // 2. Query aggregation (AVG par heure)
    // 3. Vérifier résultats
}
```

### Tests E2E (BDD)
```gherkin
Feature: IoT Real-time Monitoring
  As a Syndic
  I want to monitor sensors in real-time
  So that I can detect issues immediately

  Scenario: Receive live sensor data
    Given I am authenticated as a Syndic
    And I have 5 active sensors in Building "Résidence du Parc"
    When I open the IoT Dashboard
    Then I should see live data updating every 5 seconds
    And sensor status should show "Active" with green indicator

  Scenario: Alert on threshold exceeded
    Given I have a water sensor with max threshold 50 L/h
    When the sensor reports 75 L/h
    Then a "Critical" alert should be created
    And I should receive an email notification
    And the dashboard should show a red alert badge
```

---

## 📚 Documentation Utilisateur

### Guide Installation Capteurs

**Étape 1: Provisioning capteur**
- Installer physiquement le capteur (eau froide, électricité, etc.)
- Noter le `sensor_id` (numéro de série sur le boîtier)
- Configurer le capteur pour publier sur le broker MQTT KoproGo

**Étape 2: Enregistrer dans KoproGo**
- Aller dans `Bâtiment > Capteurs IoT > Ajouter`
- Saisir: `sensor_id`, type, emplacement, unité de mesure
- Configurer seuils d'alerte (optionnel)

**Étape 3: Vérifier réception données**
- Aller dans `Dashboard IoT`
- Vérifier que le capteur apparaît avec statut "Actif"
- Attendre 5-10 minutes pour voir les premières mesures

---

## 🔒 Sécurité

- **MQTT TLS 1.3**: Chiffrement obligatoire broker ↔ capteurs
- **MQTT Authentication**: Username/password par organisation
- **Topic isolation**: Pattern `koprogo/{org_id}/...` vérifié côté broker
- **Rate limiting**: Max 1 message/seconde par capteur
- **Validation payload**: JSON Schema strict
- **Multi-tenancy**: Isolation stricte via `organization_id`

---

## 🚀 Déploiement K8s

### StatefulSet pour IoT Ingestion Service
```yaml
apiVersion: apps/v1
kind: StatefulSet
metadata:
  name: iot-ingestion
spec:
  serviceName: iot-ingestion
  replicas: 3 # Scalabilité horizontale
  selector:
    matchLabels:
      app: iot-ingestion
  template:
    metadata:
      labels:
        app: iot-ingestion
    spec:
      containers:
      - name: iot-ingestion
        image: koprogo/iot-ingestion:latest
        env:
        - name: MQTT_BROKER_HOST
          value: "mosquitto.iot.svc.cluster.local"
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: postgres-secret
              key: connection-string
        resources:
          requests:
            memory: "256Mi"
            cpu: "250m"
          limits:
            memory: "512Mi"
            cpu: "500m"
```

### MQTT Broker (Mosquitto)
```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: mosquitto
spec:
  replicas: 1
  selector:
    matchLabels:
      app: mosquitto
  template:
    spec:
      containers:
      - name: mosquitto
        image: eclipse-mosquitto:2.0
        ports:
        - containerPort: 1883 # MQTT
        - containerPort: 8883 # MQTT over TLS
        volumeMounts:
        - name: mosquitto-config
          mountPath: /mosquitto/config
        - name: mosquitto-data
          mountPath: /mosquitto/data
      volumes:
      - name: mosquitto-config
        configMap:
          name: mosquitto-config
      - name: mosquitto-data
        persistentVolumeClaim:
          claimName: mosquitto-pvc
```

---

## 📊 Évolutions Futures

### Machine Learning
- **Prédiction de consommation** (LSTM, Prophet)
- **Détection d'anomalies avancée** (Isolation Forest, Autoencoders)
- **Recommandations d'économies** basées sur patterns

### Intégrations
- **Validation croisée ISTA** (Issue #029): Comparer relevés manuels vs IoT
- **Commande groupée énergie** (Issue #028): Utiliser données IoT pour estimer besoins
- **Domotique**: Intégrer capteurs de température pour optimisation chauffage

---

## ✅ Checklist de Complétion

- [ ] Entités Domain créées (IoTSensor, IoTReading, IoTAlert)
- [ ] IoT Ingestion Service (MQTT subscriber)
- [ ] Repositories avec TimescaleDB (hypertable + compression)
- [ ] API endpoints (CRUD sensors, time-series queries, WebSocket)
- [ ] Frontend dashboard temps réel (WebSocket + Chart.js)
- [ ] Rule engine détection anomalies
- [ ] Service alertes (email/SMS/push)
- [ ] Tests unitaires + intégration
- [ ] Tests E2E (BDD)
- [ ] Documentation utilisateur (installation capteurs)
- [ ] Déploiement K8s (StatefulSet + Mosquitto)
- [ ] Monitoring Prometheus/Grafana
- [ ] Tests de charge (10k mesures/s)

---

**Responsable**: À assigner
**Milestone**: Phase 3 - K8s Real-time & Performance
**Estimation**: 18-24h
**Dépendances**: Infrastructure K8s, TimescaleDB, MQTT Broker
