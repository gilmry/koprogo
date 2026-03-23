====================================================
R&D: IoT Integration Platform — MQTT + TimescaleDB
====================================================

Issues: #109, #227
Status: Phase 0 Implemented, Phase 1 Planned
Phase: Jalon 4 (Automation)

.. contents::
   :depth: 2

Executive Summary
=================

KoproGo implements a three-phase IoT platform for energy management:

- **Phase 0** ✅ (Complete): PostgreSQL-based readings (Issue #133)
- **Phase 1** (Planned): MQTT broker + Home Assistant integration
- **Phase 2** (Planned): TimescaleDB migration for scale
- **Phase 3** (Planned): Predictive analytics + anomaly detection

This R&D documents architecture decisions and implementation roadmap.

Current State: Phase 0 (Implemented)
====================================

Full IoT Phase 0 completed in a recent session (Issue #133):

**Entities** (Domain layer):

- ``IoTReading``: Generic sensor reading (building, device_type, metric_type, value, timestamp)
- ``LinkyDevice``: Linky smart meter configuration (PRM, provider, sync_enabled, token tracking)

**Database tables** (PostgreSQL):

- ``iot_readings`` (7M+ rows at scale, indexed by building_id + timestamp)
- ``linky_devices`` (per building, FK to buildings)

**REST API** (15 endpoints):

- ``GET /iot/readings`` — query readings with filters
- ``POST /iot/readings`` — single reading ingestion
- ``POST /iot/readings/bulk`` — batch ingestion (1000s per request)
- ``GET /iot/buildings/{id}/consumption/stats`` — daily/monthly aggregates
- ``GET /iot/buildings/{id}/consumption/anomalies`` — outlier detection
- ``POST /iot/linky/devices`` — configure Linky device
- ``POST /iot/linky/buildings/{id}/sync`` — fetch data from Enedis

**Linky Integration** (French & Belgian):

- Enedis (France) Data Connect API integration
- ORES/Sibelga (Belgium) equivalent support
- OAuth2 token management with automatic renewal

Phase 0 Metrics (Performance)
=============================

- **Ingestion rate**: 1,000 readings/second (batch mode)
- **Query latency**: < 200ms for 30-day aggregates
- **Storage**: ~100 bytes/reading (with compression: 40 bytes)
- **Memory**: < 50MB for connection pool + caching

Scaling constraints at 1B readings/year:

- **Storage**: ~100GB (compressed: 40GB)
- **Query overhead**: SELECT queries > 500ms for year-long ranges
- **Real-time monitoring**: Polling 5-minute intervals = 105K requests/hour

Phase 1: MQTT Broker Integration
=================================

Why MQTT?
---------

Current Phase 0 uses **polling** (KoproGo pulls data from Linky API every 24 hours):

- ✅ **Pros**: Simple, stateless, works for daily readings
- ❌ **Cons**: 24-hour lag, can't detect real-time anomalies, high API costs

MQTT implements **publish-subscribe** (devices push data immediately):

- ✅ **Pros**: Real-time (< 1 second latency), device-initiated, low bandwidth
- ✅ **Pros**: Scalable to 1M devices (broker handles connection multiplexing)
- ❌ **Cons**: Requires broker infrastructure, device authentication complexity

**Decision**: Implement Phase 1 with MQTT for real-time energy monitoring.

Broker Options Evaluation
==========================

+---------------------------+--------+----+---------+--------+----------+
| Broker                    | Scale  | TLS| Cluster | Cost   | Recommend|
+===========================+========+====+=========+========+==========+
| **Eclipse Mosquitto**      | 100K   | ✅ | ❌      | Free   | Dev      |
+---------------------------+--------+----+---------+--------+----------+
| **EMQX** (Erlang-based)    | 1M+    | ✅ | ✅      | 5K€/yr | **Prod** |
+---------------------------+--------+----+---------+--------+----------+
| **Tarantool** (Rust clone) | 10M    | ✅ | ✅      | OSS    | Research |
+---------------------------+--------+----+---------+--------+----------+
| **NanoMQ** (Lightweight)   | 10K    | ✅ | ❌      | Free   | IoT edge |
+---------------------------+--------+----+---------+--------+----------+

**Recommendation**:

- **Development/Beta**: Eclipse Mosquitto (free, simple, < 1MB binary)
- **Production**: EMQX (proven at 10M devices, built-in auth/clustering, Prometheus metrics)

Mosquitto Setup (Development)
=============================

.. code-block:: bash

   # Docker compose service
   mosquitto:
       image: eclipse-mosquitto:latest
       ports:
         - "1883:1883"    # MQTT plain
         - "8883:8883"    # MQTT over TLS
       volumes:
         - ./mosquitto.conf:/mosquitto/config/mosquitto.conf
         - ./mosquitto_passwd:/etc/mosquitto/passwd
       environment:
         - TZ=Europe/Brussels

**mosquitto.conf**:

.. code-block:: conf

   listener 1883
   protocol mqtt

   listener 8883
   protocol mqtt
   cafile /mosquitto/config/ca.crt
   certfile /mosquitto/config/server.crt
   keyfile /mosquitto/config/server.key

   # Authentication
   allow_anonymous false
   password_file /etc/mosquitto/passwd

EMQX Setup (Production)
=======================

.. code-block:: bash

   # Docker compose (HA cluster)
   emqx:
       image: emqx/emqx:latest
       environment:
         - EMQX_NAME=emqx
         - EMQX_HOST=127.0.0.1
         - EMQX_CLUSTER__DISCOVERY_STRATEGY=static
         - EMQX_CLUSTER__STATIC__SEEDS=[emqx@127.0.0.1]
       ports:
         - "1883:1883"    # MQTT
         - "8883:8883"    # MQTT TLS
         - "18083:18083"  # Dashboard
       volumes:
         - emqx_data:/opt/emqx/data

**EMQX Dashboard**: http://localhost:18083 (default: admin/public)

Device Authentication in EMQX
==============================

EMQX supports multiple auth backends:

**Option A: Built-in credential store** (simplest)

.. code-block:: json

   {
     "username": "org_123_building_456",
     "password": "hashed_password",
     "permissions": [
       {
         "action": "publish",
         "topic": "koprogo/org_123/building_456/+/energy"
       },
       {
         "action": "subscribe",
         "topic": "koprogo/org_123/building_456/commands/#"
       }
     ]
   }

**Option B: PostgreSQL backend** (recommended for KoproGo)

.. code-block:: sql

   -- EMQX auth table
   CREATE TABLE emqx_auth (
       id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
       organization_id UUID NOT NULL REFERENCES organizations(id),
       building_id     UUID NOT NULL REFERENCES buildings(id),
       username        VARCHAR(100) NOT NULL UNIQUE,
       password_hash   VARCHAR(255) NOT NULL,
       is_active       BOOLEAN DEFAULT TRUE,
       created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
       INDEX idx_username ON username
   );

   -- EMQX ACL (access control list)
   CREATE TABLE emqx_acl (
       id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
       auth_id         UUID NOT NULL REFERENCES emqx_auth(id) ON DELETE CASCADE,
       action          VARCHAR(20) NOT NULL,  -- publish, subscribe
       topic           VARCHAR(255) NOT NULL,
       allow           BOOLEAN DEFAULT TRUE,
       created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
   );

EMQX queries via SQL:

.. code-block:: sql

   -- Query for auth
   SELECT password_hash FROM emqx_auth WHERE username = ${username} AND is_active = true

   -- Query for ACL
   SELECT allow FROM emqx_acl
   WHERE auth_id = (SELECT id FROM emqx_auth WHERE username = ${username})
   AND topic LIKE ${topic}
   AND action = ${action}

Home Assistant Integration
==========================

Home Assistant uses **MQTT discovery** protocol:

**Device publishes discovery message**:

.. code-block:: json

   Topic: homeassistant/sensor/koprogo_org123_bld456_energy/config
   Payload: {
     "name": "Building 456 Energy",
     "state_topic": "koprogo/org123/building456/energy_kwh",
     "unit_of_measurement": "kWh",
     "device_class": "energy",
     "unique_id": "koprogo_org123_bld456_energy",
     "device": {
       "identifiers": ["koprogo_org123_bld456"],
       "name": "KoproGo Building 456",
       "manufacturer": "KoproGo"
     },
     "value_template": "{{ value_json.value }}"
   }

**Home Assistant auto-discovers and creates entities**:

- Sensor entity_id: ``sensor.koprogo_building_456_energy``
- Displayed in HA dashboard (graphs, automations, alerts)
- Can trigger automations (e.g., "alert if consumption > 100 kWh/day")

**Data flow**:

.. code-block:: text

   Smart Meter (Linky)
   ↓ (MQTT publish)
   MQTT Broker (EMQX)
   ↓ (MQTT subscribe)
   Home Assistant ← Real-time consumption updates
   ↓
   KoproGo REST → Consumption API / Anomaly Detection

KoproGo MQTT Client Integration
================================

KoproGo subscribes to device data for persistence:

.. code-block:: rust

   // Cargo.toml
   [dependencies]
   paho-mqtt = "0.12"
   tokio = "1.40"

   // backend/src/infrastructure/iot/mqtt_subscriber.rs
   use paho_mqtt as mqtt;

   pub struct MqttSubscriber {
       client: mqtt::AsyncClient,
       organization_id: Uuid,
   }

   impl MqttSubscriber {
       pub async fn new(broker_url: &str, org_id: Uuid) -> Result<Self> {
           let create_opts = mqtt::CreateOptionsBuilder::new()
               .server_uri(broker_url)
               .client_id(format!("koprogo_{}", org_id))
               .finalize();

           let client = mqtt::AsyncClient::new(create_opts)?;
           Ok(MqttSubscriber { client, organization_id: org_id })
       }

       pub async fn subscribe(&self, topic: &str) -> Result<()> {
           self.client.subscribe(topic, 1).await?;
           Ok(())
       }

       pub async fn process_messages(
           &self,
           iot_repo: &dyn IoTReadingRepository,
       ) -> Result<()> {
           while let Some(msg) = self.client.get_message().await {
               if let Some(msg) = msg {
                   let topic = msg.topic();
                   let payload = msg.payload_str();

                   // Parse MQTT topic: koprogo/{org_id}/{building_id}/{device}/{metric}
                   let parts: Vec<&str> = topic.split('/').collect();
                   if parts.len() == 5 {
                       let building_id = parts[2].parse::<Uuid>()?;
                       let device_type = parts[3];
                       let metric_type = parts[4];

                       // Parse JSON payload
                       let value: f32 = serde_json::from_str(payload)?;

                       // Persist to PostgreSQL
                       iot_repo.create_reading(&IoTReading {
                           building_id,
                           device_type: device_type.to_string(),
                           metric_type: metric_type.to_string(),
                           value,
                           timestamp: Utc::now(),
                       }).await?;
                   }
               }
           }
           Ok(())
       }
   }

**In application main.rs**:

.. code-block:: rust

   // Start MQTT subscriber in background
   let mqtt_sub = MqttSubscriber::new(
       &env::var("MQTT_BROKER_URL").unwrap_or("mqtt://localhost:1883".to_string()),
       org_id,
   ).await?;

   tokio::spawn(async move {
       if let Err(e) = mqtt_sub.process_messages(&iot_repo).await {
           error!("MQTT subscriber error: {}", e);
       }
   });

Linky/ORES Integration via MQTT
================================

For real-time Linky data, two approaches:

**Approach 1: Polling (current Phase 0)**
- Daily sync via Enedis Data Connect REST API
- Lag: 24 hours (data available next day)
- Cost: EUR 0 (free API)

**Approach 2: Hardware bridge + MQTT**
- TIC (Télé-Information Client) port on Linky meter
- Device reads TIC port (115.2k baud serial)
- Publishes to MQTT in real-time
- Cost: EUR 50-200 for TIC reader device

**Decision for Phase 1**: Keep REST API polling (Phase 0), add MQTT for third-party devices.
**Future (Phase 2)**: Evaluate TIC hardware integration if cost justified.

Phase 2: TimescaleDB Migration
===============================

At 1 billion readings/year, PostgreSQL scaling constraints:

- **B-tree indexes** on timestamp = slow (O(log n))
- **Table scans** for year-long ranges = 5+ seconds
- **Vacuum overhead** (table bloat from frequent inserts)

**TimescaleDB** is PostgreSQL extension solving these issues:

.. code-block:: sql

   -- Install TimescaleDB (PostgreSQL 14+)
   CREATE EXTENSION IF NOT EXISTS timescaledb CASCADE;

   -- Convert iot_readings to hypertable (time-series optimized)
   SELECT create_hypertable(
       'iot_readings',
       'timestamp',
       chunk_time_interval => INTERVAL '1 week'
   );

   -- Creates transparent chunks (weekly partitions, auto-managed)

**Benefits**:

- **Query speed**: 100x faster for year-long range queries (< 50ms)
- **Compression**: 80% smaller storage (columnar compression)
- **Insert rate**: 1M readings/second (auto-partitioning)
- **Retention**: Automatic chunk deletion after N days

**Continuous aggregates** (materialized views):

.. code-block:: sql

   -- Daily consumption stats (auto-materialized every hour)
   CREATE MATERIALIZED VIEW daily_consumption
   WITH (timescaledb.continuous, timescaledb.materialized_only=false) AS
   SELECT
       time_bucket('1 day', timestamp) AS day,
       building_id,
       metric_type,
       SUM(value) AS total_kwh,
       AVG(value) AS avg_kwh,
       MAX(value) AS peak_kwh,
       MIN(value) AS min_kwh
   FROM iot_readings
   WHERE metric_type = 'electricity_kwh'
   GROUP BY day, building_id, metric_type;

   -- Automatic refresh job
   SELECT add_continuous_aggregate_policy(
       'daily_consumption',
       start_offset => INTERVAL '2 days',
       end_offset => INTERVAL '1 hour',
       schedule_interval => INTERVAL '1 hour'
   );

**Migration path** (zero downtime):

1. Enable TimescaleDB extension on existing PostgreSQL
2. Run ``create_hypertable()`` on existing ``iot_readings`` table
3. Continuous aggregates update in background
4. Queries transparently use new indexes

Phase 3: Predictive Analytics
==============================

Using TimescaleDB aggregates + machine learning:

**Anomaly detection** (implemented in Phase 0):

- Rolling average + standard deviation
- Flag readings > 2σ from mean

**Predictive forecasting** (Phase 3):

- ARIMA (AutoRegressive Integrated Moving Average) model
- Forecast next 7 days consumption
- Alert if forecast > budget

**Seasonal adjustment**:

- Account for weather (temperature, daylight hours)
- Account for occupancy (vacations, holidays)

Implementation Timeline
=======================

**Phase 0** ✅ (Complete)

- PostgreSQL iot_readings + Linky API integration
- REST API (15 endpoints)
- Anomaly detection

**Phase 1** (Planned: Q2-Q3 2026)

- MQTT broker (Mosquitto dev, EMQX prod): 1 week
- KoproGo MQTT client: 1 week
- Home Assistant integration: 1 week
- PostgreSQL auth backend: 1 week
- Testing + documentation: 1 week

**Phase 2** (Planned: Q3-Q4 2026)

- TimescaleDB extension setup: 1 week
- Hypertable migration (iot_readings): 1 week
- Continuous aggregates (daily/monthly): 1 week
- Query optimization: 1 week
- Performance testing (1B rows): 2 weeks

**Phase 3** (Planned: Q4 2026 - Q1 2027)

- ARIMA forecasting model: 3 weeks
- Seasonal adjustment: 2 weeks
- Integration into REST API: 1 week
- UI (forecast graphs): 2 weeks

**Total**: ~24 weeks (6 months)

Database Schema Changes
=======================

Phase 1 (MQTT support):

.. code-block:: sql

   CREATE TABLE mqtt_devices (
       id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
       organization_id UUID NOT NULL REFERENCES organizations(id),
       building_id     UUID NOT NULL REFERENCES buildings(id),
       device_name     VARCHAR(100) NOT NULL,
       device_type     VARCHAR(50),
       mqtt_topic      VARCHAR(255) NOT NULL UNIQUE,
       is_active       BOOLEAN DEFAULT TRUE,
       last_message_at TIMESTAMPTZ,
       created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
   );

   CREATE TABLE mqtt_messages (
       id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
       device_id       UUID NOT NULL REFERENCES mqtt_devices(id),
       topic           VARCHAR(255) NOT NULL,
       payload_json    JSONB,
       received_at     TIMESTAMPTZ NOT NULL DEFAULT NOW()
   );

Phase 2 (TimescaleDB):

.. code-block:: sql

   -- No schema changes, just hypertable conversion
   SELECT create_hypertable('iot_readings', 'timestamp', chunk_time_interval => INTERVAL '1 week');

Performance Targets
===================

- **Phase 0**: 100K readings/day, 200ms queries
- **Phase 1**: 1M readings/day, <100ms real-time queries via MQTT
- **Phase 2**: 1B readings/year, <50ms time-range queries (TimescaleDB)
- **Phase 3**: Forecasting accuracy >90% (vs actual consumption)

Cost Estimation
===============

- **Mosquitto**: Free (OSS)
- **EMQX**: EUR 5K/year (production license)
- **TimescaleDB**: Free (OSS extension)
- **Cloud storage** (S3 for backups): EUR 100/month at 1TB
- **Development**: ~24 developer-weeks (~6 months)

Risks & Mitigations
====================

**Risk**: MQTT broker downtime (device data lost)

**Mitigation**:
- Local device storage (Linky can store 24h data)
- Automatic retry with exponential backoff
- Fallback to daily REST API polling

**Risk**: Database unable to ingest 1M readings/second

**Mitigation**:
- Use message queue (Kafka) as buffer
- Implement rate-limiting (cap at 100K readings/sec, queue excess)
- Monitor ingestion lag (alert if > 5 min backlog)

**Risk**: TimescaleDB migration corrupts data

**Mitigation**:
- Test migration on production backup first
- Keep raw iot_readings backup for 30 days
- Validate chunk integrity (TimescaleDB built-in checks)

Related Issues
==============

- **#109**: IoT platform overview
- **#133**: Phase 0 (implemented)
- **#227**: MQTT broker (this R&D Phase 1)
- **#228**: TimescaleDB (future Phase 2)
- **#229**: Predictive analytics (future Phase 3)

References
==========

- `Eclipse Mosquitto <https://mosquitto.org/>`_
- `EMQX Broker <https://www.emqx.io/>`_
- `TimescaleDB <https://www.timescaledb.com/>`_
- `Home Assistant MQTT Discovery <https://www.home-assistant.io/integrations/mqtt/>`_
- `Enedis Data Connect API <https://data.enedis.fr/api>`_
- `MQTT Protocol Specification <https://mqtt.org/>`_
