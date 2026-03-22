======================================================================
Issue #109: feat: IoT Integration Platform (MQTT Broker + TimescaleDB)
======================================================================

:State: **OPEN**
:Milestone: Jalon 6: Intelligence & Expansion (PropTech 2.0) 🤖
:Labels: enhancement,phase:k8s track:software,track:infrastructure priority:medium,automation proptech:iot
:Assignees: Unassigned
:Created: 2025-11-07
:Updated: 2026-02-18
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/109>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## 📡 IoT Integration Platform
   
   **Priority**: 🟡 Medium | **Phase**: 3 (K8s Production) | **Track**: Software + Infrastructure
   
   ### Description
   Plateforme IoT complète pour intégrer capteurs énergie, eau, sécurité dans les copropriétés.
   
   ### PropTech 2.0 - Pilier IoT
   - 📊 Monitoring temps-réel consommations (électricité, gaz, eau)
   - 🚨 Alertes anomalies (fuites eau, pics consommation, intrusions)
   - 📈 Analytics prédictifs (prévisions factures, maintenance préventive)
   - 🔗 Intégration vendeurs IoT (Netatmo, Linky, capteurs LoRaWAN)
   
   ### Tâches Techniques
   
   #### Infrastructure
   - [ ] MQTT broker (Mosquitto/EMQX) sur K8s
   - [ ] TimescaleDB pour time-series data (retention 2 ans)
   - [ ] API Gateway pour devices (authentification JWT/mTLS)
   - [ ] Dashboard Grafana temps-réel
   
   #### Backend (Rust)
   - [ ] Entity `IoTDevice` (building_id, type, vendor, last_seen, status)
   - [ ] Entity `IoTReading` (device_id, timestamp, metric_type, value, unit)
   - [ ] Port `IoTRepository` + PostgreSQL impl
   - [ ] MQTT consumer service (actix-mqtt)
   - [ ] Anomaly detection engine (règles + ML)
   - [ ] Endpoints: `POST /api/v1/buildings/:id/iot/devices`, `GET /iot/devices/:id/readings`
   
   #### Frontend
   - [ ] IoT dashboard page (temps-réel WebSocket)
   - [ ] Configuration devices UI (onboarding wizard)
   - [ ] Alertes & notifications
   - [ ] Graphiques consommations (Chart.js)
   
   #### Intégrations
   - [ ] Linky (Enedis API) - compteurs électriques
   - [ ] Netatmo - capteurs météo/qualité air
   - [ ] LoRaWAN gateway support (The Things Network)
   
   ### Livrables
   - ✅ MQTT broker déployé K8s avec HA
   - ✅ TimescaleDB cluster (3 nodes)
   - ✅ 3+ types devices supportés
   - ✅ Alertes temps-réel (<5s latence)
   - ✅ Dashboard analytics 7j/30j/1an
   - ✅ Documentation intégration vendors
   
   ### Effort estimé
   **24-32 heures** (3-4 jours dev)
   
   ### Dépend de
   - Phase 3 K8s déployé
   - WebSocket real-time infrastructure (#094 partiel)
   
   ### Modèle Participatif
   💡 **Mutualisation IoT**: 1 gateway LoRaWAN partagé entre 50 copros = coût divisé par 50!
   
   ### Labels
   `enhancement`, `phase:k8s`, `track:software`, `track:infrastructure`, `priority:medium`, `proptech:iot`, `automation`

.. raw:: html

   </div>

