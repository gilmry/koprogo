=============================================================================
Issue #227: R&D: Architecture IoT - Intégration capteurs et base time-series
=============================================================================

:State: **OPEN**
:Milestone: No milestone
:Labels: priority:medium,proptech:iot R&D
:Assignees: Unassigned
:Created: 2026-03-07
:Updated: 2026-03-07
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/227>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Contexte
   
   L'issue #109 a débuté l'intégration IoT (Phase 0 : Linky + lectures).
   Cette R&D couvre l'architecture complète pour la production.
   
   **Issue liée**: #109
   
   ## Objectifs de la R&D
   
   1. **Protocoles IoT** :
      - MQTT broker (Mosquitto/EMQX) pour capteurs en temps réel
      - REST/HTTP pour intégrations batch (Linky, Ores)
      - CoAP pour devices contraints (optionnel)
      - LoRaWAN pour bâtiments sans WiFi (gateway)
   
   2. **Base de données time-series** :
      - TimescaleDB (extension PostgreSQL, pas de nouvelle techno)
      - InfluxDB (dédié time-series, performant mais stack séparé)
      - VictoriaMetrics (compatible Prometheus, léger)
      - PostgreSQL + partitionnement (solution simple, déjà en place)
   
   3. **Types de capteurs prioritaires** :
      - Compteurs énergie (Linky ✅, Ores, SmartMeter)
      - Compteurs eau (Hydrobru, SWDE)
      - Température/humidité (confort, détection fuites)
      - Sécurité (détection ouverture, mouvement)
   
   4. **Détection d'anomalies** :
      - Seuils statiques (consommation > X kWh/jour)
      - Modèle statistique (écart-type, z-score)
      - ML (isolation forest, LSTM pour séries temporelles)
      - Alertes : notification push + email + SMS
   
   5. **Architecture edge** :
      - Raspberry Pi comme gateway IoT local
      - Buffer local en cas de perte réseau
      - Chiffrement TLS pour données en transit
   
   ## Points de décision
   
   - [ ] MQTT broker choice (Mosquitto vs. EMQX)
   - [ ] Time-series DB (TimescaleDB vs. partition PostgreSQL)
   - [ ] Rétention des données (raw 30j, agrégé 5 ans)
   - [ ] Edge gateway nécessaire ou pas
   
   ## Estimation
   
   12-16h

.. raw:: html

   </div>

