=============================================================================
Issue #133: feat: Linky/Ores API Integration for Smart Electricity Monitoring
=============================================================================

:State: **OPEN**
:Milestone: Jalon 3: Features Différenciantes 🎯
:Labels: enhancement,phase:vps track:software,priority:high automation,proptech:iot
:Assignees: Unassigned
:Created: 2025-11-18
:Updated: 2025-11-18
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/133>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## 📡 Linky/Ores API Integration - Phase 0 IoT (0€ Budget)
   
   **Priority**: 🟢 High | **Phase**: VPS (Jalon 3-4) | **Track**: Software | **Cost**: 0€
   
   ### Description
   Intégration API Linky (Enedis France) et Ores (Belgique) pour monitoring intelligent consommation électricité **sans hardware IoT**.
   
   ### Proposition de Valeur
   - ✅ **0€ coût** (API gratuite, pas d'achat capteurs)
   - ✅ **0 installation physique** (API call uniquement)
   - ✅ **80%+ couverture** (Linky obligatoire Belgique/France)
   - ✅ **95% bénéfices IoT** pour 0% du coût
   - ✅ **Time-to-market: 1 semaine** vs 3-6 mois hardware
   
   ### Use Cases
   
   **Monitoring Temps-Réel:**
   - Consommation électrique quotidienne/mensuelle/annuelle
   - Courbe de charge (granularité 30 min)
   - Historique 36 mois
   
   **Alertes Intelligentes:**
   - Détection surconsommation (> 120% moyenne)
   - Prévision factures énergie
   - Recommandations économies CO₂
   
   **Analytics:**
   - Graphiques consommations (Chart.js/Recharts)
   - Comparaison périodes (MoM, YoY)
   - Export PDF rapports énergétiques
   
   ### Tâches Techniques
   
   #### Backend (Rust)
   
   **Domain Layer:**
   - [ ] Entity \`IoTReading\` (building_id, device_type, metric_type, value, unit, timestamp, source)
   - [ ] Entity \`LinkyDevice\` (building_id, prm, api_key_encrypted, last_sync)
   - [ ] Enum \`DeviceType\`: ElectricityMeter, WaterMeter, TemperatureSensor, etc.
   - [ ] Enum \`MetricType\`: ElectricityConsumption, WaterConsumption, Temperature, etc.
   - [ ] Validation rules (temperature -40/+80°C, humidity 0-100%, consumption >= 0)
   
   **Application Layer:**
   - [ ] Port \`IoTRepository\` (create, find_by_building, find_by_metric, get_statistics)
   - [ ] Port \`LinkyApiClient\` (get_daily_consumption, get_monthly_consumption)
   - [ ] Use Cases \`IoTUseCases\` (sync_linky_data, detect_anomaly, get_consumption_stats)
   - [ ] DTO \`IoTReadingDto\`, \`LinkyConfigDto\`, \`ConsumptionStatsDto\`
   
   **Infrastructure Layer:**
   - [ ] \`LinkyApiClient\` (Ores Belgium API: https://ext.prod-eu.oresnet.be/v1/)
   - [ ] \`PostgresIoTRepository\` + TimescaleDB hypertable
   - [ ] Handlers \`iot_handlers.rs\`:
     - POST /api/v1/buildings/:id/iot/linky/configure
     - POST /api/v1/buildings/:id/iot/linky/sync
     - GET /api/v1/buildings/:id/iot/readings
     - GET /api/v1/buildings/:id/iot/statistics
   - [ ] Cron job daily sync (actix-web background task)
   - [ ] Notification integration (Issue #86 - alertes surconsommation)
   
   **Database:**
   - [ ] Migration \`20251201000000_create_iot_readings.sql\`:
     - Table \`iot_readings\` (TimescaleDB hypertable)
     - Table \`linky_devices\` (PRM, encrypted API keys)
     - Indexes (building_id, timestamp), (metric_type, timestamp)
     - Retention policy: 2 ans (730M records)
     - Compression policy: 7 jours (10-20x savings)
   
   #### Frontend (Astro + Svelte)
   
   - [ ] Page \`/buildings/[id]/iot.astro\`
   - [ ] Component \`LinkyConfiguration.svelte\` (OAuth2 consent, PRM input)
   - [ ] Component \`ConsumptionChart.svelte\` (Chart.js line chart)
   - [ ] Component \`ConsumptionStats.svelte\` (cards: daily, monthly, yearly, avg)
   - [ ] Component \`AnomalyAlerts.svelte\` (liste alertes surconsommation)
   - [ ] Component \`ExportReport.svelte\` (PDF export rapports)
   
   #### API Linky/Ores
   
   **Ores Belgium API:**
   - Endpoint: https://ext.prod-eu.oresnet.be/v1/consumption_load_curve
   - Authentication: OAuth2 Bearer token
   - Parameters: prm (Point Reference Measure), start date, end date
   - Rate limit: Pas documenté (à tester)
   - Documentation: https://www.ores.be/api
   
   **Enedis Linky France API:**
   - Endpoint: https://ext.hml.myelectricaldata.fr/v1/
   - Authentication: OAuth2 (consent utilisateur)
   - Mêmes paramètres que Ores
   
   **OAuth2 Flow:**
   1. User consent (redirect to Ores/Enedis)
   2. Authorization code exchange
   3. Access token storage (encrypted)
   4. Refresh token rotation
   
   ### Dependencies
   
   **Backend (Cargo.toml):**
   \`\`\`toml
   [dependencies]
   reqwest = { version = "0.12", features = ["json"] }
   oauth2 = "4.4"
   serde_json = "1.0"
   chrono = "0.4"
   \`\`\`
   
   **Frontend (package.json):**
   \`\`\`json
   {
     "dependencies": {
       "chart.js": "^4.4.0",
       "react-chartjs-2": "^5.2.0"
     }
   }
   \`\`\`
   
   ### Environment Variables
   
   \`\`\`bash
   # Backend .env
   LINKY_ORES_CLIENT_ID=<ores-client-id>
   LINKY_ORES_CLIENT_SECRET=<ores-client-secret>
   LINKY_ORES_REDIRECT_URI=https://koprogo.com/auth/linky/callback
   
   LINKY_ENEDIS_CLIENT_ID=<enedis-client-id>
   LINKY_ENEDIS_CLIENT_SECRET=<enedis-client-secret>
   
   IOT_ENCRYPTION_KEY=<32-byte-key>  # For API keys encryption
   \`\`\`
   
   ### Livrables
   
   - ✅ Linky/Ores OAuth2 integration working
   - ✅ Daily consumption sync automated (cron)
   - ✅ IoTReading entity + TimescaleDB storage
   - ✅ Anomaly detection (> 120% avg) with notifications
   - ✅ Frontend dashboard with charts
   - ✅ E2E tests (sync, anomaly detection, charts)
   - ✅ Documentation API Linky/Ores
   - ✅ GDPR compliance (encrypted API keys, user consent)
   
   ### Effort estimé
   **7-10 jours** (1-2 semaines)
   - Day 1-2: Ores/Enedis OAuth2 registration + backend client
   - Day 3-4: Domain entities + TimescaleDB migration
   - Day 5-6: IoT repository + use cases + handlers
   - Day 7-8: Frontend dashboard + charts
   - Day 9: E2E tests + anomaly detection
   - Day 10: Documentation + GDPR review
   
   ### Modèle de Revenus (Optionnel)
   - **Gratuit** pour tous (feature incluse 5€/mois)
   - Alternative: Module optionnel **+1€/mois** si analytics avancés (ML prévisions)
   
   ### Dépend de
   - Issue #86 (Notifications) - Pour alertes surconsommation
   - PostgreSQL TimescaleDB extension (déjà disponible)
   
   ### Alternative: Phase 2 IoT Hardware
   Si API Linky insuffisant (granularité 30 min vs temps-réel):
   - **Issue #109** (IoT Integration Platform) - MQTT + hardware sensors
   - Coût: 50-200€/device + infrastructure MQTT
   - Délai: 3-6 mois (logistique hardware)
   
   ### Labels
   \`enhancement\`, \`phase:vps\`, \`track:software\`, \`priority:high\`, \`proptech:iot\`, \`automation\`
   
   ### Milestone
   **Jalon 3: Features Différenciantes** (ou Jalon 4: Automation & Intégrations)
   
   ### Acceptance Criteria
   
   **Backend:**
   - [ ] Ores/Enedis OAuth2 flow completes successfully
   - [ ] Daily consumption fetched and stored in TimescaleDB
   - [ ] Anomaly detection triggers notification (> 120% avg)
   - [ ] API endpoints return consumption data with pagination
   - [ ] Cron job syncs data daily at 2:00 AM
   - [ ] E2E tests pass (sync, anomaly, stats)
   
   **Frontend:**
   - [ ] Linky configuration page allows OAuth2 consent
   - [ ] Consumption chart displays last 30 days
   - [ ] Statistics cards show daily/monthly/yearly averages
   - [ ] Anomaly alerts display in UI (real-time via WebSocket)
   - [ ] Export PDF report works
   
   **Security:**
   - [ ] API keys encrypted at rest (AES-256)
   - [ ] OAuth2 tokens stored securely (HttpOnly cookies)
   - [ ] User consent recorded (GDPR Article 6)
   - [ ] Audit logs track all API syncs
   
   **Performance:**
   - [ ] TimescaleDB compression reduces storage by 10-20x
   - [ ] API response time < 500ms for 30-day data
   - [ ] Cron job completes in < 5 min for 100 buildings
   
   ### Related Issues
   - #86 (Notifications) - Integration pour alertes
   - #109 (IoT Platform) - Phase 2 hardware sensors
   - #96 (Sustainability) - Carbon footprint tracking
   - #110 (Energy Buying Groups) - Optimisation consommation
   
   ### References
   - Ores API Documentation: https://www.ores.be/api
   - Enedis My Electrical Data: https://www.enedis.fr/mes-donnees-de-consommation
   - TimescaleDB Hypertables: https://docs.timescale.com/
   - Chart.js Documentation: https://www.chartjs.org/
   
   ### Future Enhancements (Post-MVP)
   - ML prévisions factures (ARIMA models)
   - Recommandations économies énergie (AI assistant)
   - Comparaison benchmarks (consommation vs moyenne copros similaires)
   - Integration Netatmo API (température/humidité)
   - Integration compteurs eau (si API disponible)

.. raw:: html

   </div>

