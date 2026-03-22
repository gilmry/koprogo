===============================================================================
Issue #300: feat(iot): MQTT Home Assistant + BOINC Grid Computing (IoT Phase 1)
===============================================================================

:State: **OPEN**
:Milestone: Jalon 3: Features Différenciantes 🎯
:Labels: enhancement,track:software proptech:iot
:Assignees: Unassigned
:Created: 2026-03-21
:Updated: 2026-03-21
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/300>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Contexte
   
   Code implémenté sur la branche `integration` (commit `31350eb`) sans issue de suivi.
   
   ## Implémenté
   
   ### MQTT Home Assistant
   - Adapter MQTT (`rumqttc`) : subscribe aux topics `koprogo/{building_id}/energy/{unit_id}/{metric}`
   - 6 device types (ElectricityMeter, WaterMeter, GasMeter, TemperatureSensor, HumiditySensor, PowerMeter)
   - 7 metric types avec validation domaine (plages, unités, timestamps)
   - Endpoints : `POST /iot/mqtt/start`, `POST /iot/mqtt/stop`, `GET /iot/mqtt/status`
   - Authentification requise pour démarrer le listener
   
   ### BOINC Grid Computing
   - Consentement GDPR (Art. 6.1.a + Art. 7) avec audit trail
   - 2 types de tâches : EnergyGroupOptimisation, BuildingThermalSimulation
   - Endpoints : consent CRUD, task submit/poll/cancel
   - Migration : `20260317000000_create_iot_grid_tables.sql`
   
   ### Tests
   - BDD : `iot_mqtt_boinc.feature` (18 scénarios)
   - Domain : 8 unit tests IoTReading
   
   ## Fichiers
   - `backend/src/infrastructure/mqtt/mqtt_energy_adapter.rs`
   - `backend/src/infrastructure/grid/boinc_grid_adapter.rs`
   - `backend/src/application/ports/mqtt_energy_port.rs`
   - `backend/src/application/ports/grid_participation_port.rs`
   - `backend/src/application/use_cases/boinc_use_cases.rs`
   - `backend/src/infrastructure/web/handlers/iot_grid_handlers.rs`
   
   ## Statut
   - ✅ Backend complet (adapters, use cases, handlers, routes)
   - ✅ BDD tests
   - ⚠️ `publish_alert()` stubbé (Phase 2)
   - ⚠️ Pas de cluster BOINC réel (POC local)
   - ❌ Pas de frontend dédié
   - ❌ Pas de E2E backend dédié

.. raw:: html

   </div>

