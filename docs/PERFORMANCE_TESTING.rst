
Performance Testing & Optimization Guide
========================================

Guide complet pour tester et optimiser les performances de KoproGo sur un serveur 1 vCPU / 2GB RAM.

📊 Nouveau Seed Réaliste
------------------------

Caractéristiques
^^^^^^^^^^^^^^^^

Le nouveau seed ``seed_realistic`` génère des données proportionnelles à la capacité d'un serveur 1 vCPU :


* **3 organisations** (petite, moyenne, grande)
* **23 buildings** au total
* **~190 units** au total
* **~127 owners** au total
* **~60 expenses** au total

Utilisation
^^^^^^^^^^^

**Option 1: Via l'interface Superadmin (recommandé)**

.. code-block:: bash

   # Se connecter en tant que superadmin
   curl -X POST https://api2.koprogo.com/api/v1/auth/login \
     -H "Content-Type: application/json" \
     -d '{"email":"admin@koprogo.com","password":"admin123"}'

   # Utiliser le token reçu pour lancer le seed
   curl -X POST https://api2.koprogo.com/api/v1/seed/realistic \
     -H "Authorization: Bearer <TOKEN>"

**Option 2: Via script bash (nécessite SSH sur le VPS)**

.. code-block:: bash

   cd backend

   # Script automatisé
   ./run-realistic-seed.sh

   # Ou manuelle
   SQLX_OFFLINE=true cargo build --bin seed_realistic --release
   cargo run --bin seed_realistic --release

**Option 3: Depuis l'interface web**

Une fois connecté en tant que superadmin dans l'interface web, accéder à la section "Seed Data" et cliquer sur "Generate Realistic Data".

Credentials de Test
^^^^^^^^^^^^^^^^^^^

Après le seed, vous pouvez vous connecter avec :

.. list-table::
   :header-rows: 1

   * - Organisation
     - Email
     - Password
   * - Petite
     - ``admin@small.be``
     - ``admin123``
   * - Moyenne
     - ``admin@medium.be``
     - ``admin123``
   * - Grande
     - ``admin@large.be``
     - ``admin123``


🎯 Tests de Charge Réalistes
----------------------------

Test Mixte POST/GET
^^^^^^^^^^^^^^^^^^^

Le nouveau script ``realistic-load.sh`` simule un comportement utilisateur réel :


* **70% lectures** (GET) : buildings, units, owners, expenses
* **30% écritures** (POST) : créer des buildings, units, owners, expenses
* Distribution pondérée par fréquence d'usage

Lancer le Test
^^^^^^^^^^^^^^

.. code-block:: bash

   cd load-tests

   # Local
   export BASE_URL=http://localhost:8080
   ./scripts/realistic-load.sh

   # Production
   export BASE_URL=https://api2.koprogo.com
   ./scripts/realistic-load.sh

Objectifs de Performance (1 vCPU / 2GB RAM)
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

.. list-table::
   :header-rows: 1

   * - Métrique
     - Cible
     - Notes
   * - **Throughput**
     - > 200 req/s
     - Avec mix 70/30 GET/POST
   * - **P99 Latency**
     - < 100ms
     - Avec écritures
   * - **P50 Latency**
     - < 30ms
     - Médiane
   * - **Error Rate**
     - < 1%
     - Erreurs HTTP 4xx/5xx
   * - **Memory Backend**
     - < 128MB
     - Actuellement ~3.6MB ✅
   * - **Memory PostgreSQL**
     - < 512MB
     - Actuellement ~64MB ✅
   * - **Load Average**
     - < 2.0
     - Actuellement pic à 4.81 ⚠️


🚀 Optimisations Récentes
-------------------------

1. Indexes PostgreSQL
^^^^^^^^^^^^^^^^^^^^^

Migration ``20250103000003_add_performance_indexes.sql`` :

.. code-block:: sql

   -- Foreign key indexes (améliore les JOINs)
   CREATE INDEX idx_units_organization_id ON units(organization_id);
   CREATE INDEX idx_units_building_id ON units(building_id);
   CREATE INDEX idx_buildings_organization_id ON buildings(organization_id);

   -- Composite indexes (améliore la pagination)
   CREATE INDEX idx_units_org_number ON units(organization_id, unit_number);
   CREATE INDEX idx_buildings_org_name ON buildings(organization_id, name);
   CREATE INDEX idx_owners_org_name ON owners(organization_id, last_name, first_name);
   CREATE INDEX idx_expenses_org_date ON expenses(organization_id, expense_date DESC);

   -- Auth indexes (améliore login/refresh)
   CREATE INDEX idx_refresh_tokens_user_id ON refresh_tokens(user_id);
   CREATE INDEX idx_refresh_tokens_expires_at ON refresh_tokens(expires_at) WHERE NOT revoked;

**Impact attendu** : P99 latency de 801ms → < 50ms

2. Docker Compose
^^^^^^^^^^^^^^^^^

**Traefik** :


* Memory limit : 50MB → 80MB
* Raison : Était à 77-78% d'utilisation sous charge

3. Dimensionnement des Données
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

Ancien seed (trop petit pour être réaliste) :


* 3 orgs, 3-4 buildings, ~10 units

Nouveau seed (proportionnel à 1 vCPU) :


* 3 orgs, 23 buildings, ~190 units
* Permet de tester les performances avec un volume réaliste

📈 Monitoring
-------------

Pendant les Tests
^^^^^^^^^^^^^^^^^

.. code-block:: bash

   # Sur le VPS
   cd ~/koprogo/load-tests
   ./monitor-server.sh

Métriques à Surveiller
^^^^^^^^^^^^^^^^^^^^^^

.. list-table::
   :header-rows: 1

   * - Métrique
     - Normal
     - Alerte
   * - **Backend CPU**
     - 15-30%
     - > 80%
   * - **Backend Memory**
     - < 50MB
     - > 200MB
   * - **PostgreSQL CPU**
     - 20-30%
     - > 60%
   * - **PostgreSQL Memory**
     - < 100MB
     - > 400MB
   * - **Traefik Memory**
     - < 60MB
     - > 75MB
   * - **Load Average**
     - < 1.5
     - > 3.0
   * - **Disk I/O %util**
     - < 20%
     - > 80%


Résultats Monitoring Récent
^^^^^^^^^^^^^^^^^^^^^^^^^^^

.. code-block::

   Load Average Peak: 4.81 (⚠️ très élevé pour 1 vCPU)
   Backend Memory: 3.6MB (✅ excellent)
   PostgreSQL Memory: 64MB (✅ stable)
   Traefik Memory: 38MB / 80MB (✅ bon avec nouveau limit)

🔧 Workflow de Test Complet
---------------------------

1. Préparer les Données
^^^^^^^^^^^^^^^^^^^^^^^

.. code-block:: bash

   # Sur le VPS via SSH ou localement
   cd ~/koprogo/backend
   ./run-realistic-seed.sh

2. Lancer le Monitoring
^^^^^^^^^^^^^^^^^^^^^^^

.. code-block:: bash

   # Terminal 1 : Monitoring
   cd ~/koprogo/load-tests
   ./monitor-server.sh

3. Exécuter les Tests
^^^^^^^^^^^^^^^^^^^^^

.. code-block:: bash

   # Terminal 2 : Load tests
   cd ~/koprogo/load-tests
   export BASE_URL=https://api2.koprogo.com

   # Test réaliste mixte
   ./scripts/realistic-load.sh

   # Ou suite complète
   ./run-all-tests.sh

4. Analyser les Résultats
^^^^^^^^^^^^^^^^^^^^^^^^^

.. code-block:: bash

   # Voir les résultats
   ls -lth results/

   # Dernier test réaliste
   cat results/realistic-load_*.txt | tail -30

   # Monitoring
   cat monitoring-results/server-monitoring_*.log | grep "Load Average\|Backend\|PostgreSQL"

📊 Comparaison Avant/Après Optimisations
----------------------------------------

Avant (seed minimal, pas d'indexes)
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^


* Throughput: ~312 req/s (GET only)
* P99 Latency: 801ms
* Error Rate: 0.16%
* Load Average Peak: 4.81

Après (seed réaliste + indexes) - À TESTER
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^


* Throughput attendu: > 250 req/s (mix 70/30)
* P99 Latency attendu: < 100ms
* Error Rate attendu: < 1%
* Load Average attendu: < 2.5

🎯 Prochaines Étapes
--------------------


#. 
   **Déployer les indexes** :

   .. code-block:: bash

      # Sur le VPS
      cd ~/koprogo/backend
      sqlx migrate run

#. 
   **Lancer le nouveau seed** :

   .. code-block:: bash

      ./run-realistic-seed.sh

#. 
   **Tester avec le nouveau script** :

   .. code-block:: bash

      cd ~/koprogo/load-tests
      export BASE_URL=https://api2.koprogo.com
      ./scripts/realistic-load.sh

#. 
   **Comparer les résultats** :


   * P99 latency doit passer de 801ms à < 100ms
   * Throughput doit rester > 200 req/s malgré les POST
   * Error rate doit rester < 1%

🔍 Debugging
------------

Queries Lentes
^^^^^^^^^^^^^^

.. code-block:: sql

   -- Voir les queries les plus lentes
   SELECT query, calls, total_time, mean_time
   FROM pg_stat_statements
   ORDER BY total_time DESC
   LIMIT 10;

Utilisation des Index
^^^^^^^^^^^^^^^^^^^^^

.. code-block:: sql

   -- Vérifier qu'un index est utilisé
   EXPLAIN ANALYZE
   SELECT * FROM units
   WHERE organization_id = '...'
   ORDER BY unit_number;

Logs Backend
^^^^^^^^^^^^

.. code-block:: bash

   # Sur le VPS
   docker compose -f deploy/production/docker-compose.yml logs -f backend | grep -E "WARN|ERROR|slow"

📚 Références
-------------


* **Objectifs P99** : backend/CLAUDE.md (< 5ms target ambitieux)
* **Config PostgreSQL** : deploy/production/docker-compose.yml (tuning pour 1 vCPU)
* **GitOps Deployment** : deploy/production/GITOPS.md
* **Load Tests** : load-tests/README.md
